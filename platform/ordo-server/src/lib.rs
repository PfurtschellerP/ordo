//! The ORDO server (`ordo serve`).
//!
//! One process: WebSocket sync at `/sync`, health at `/healthz` and `/readyz`, metrics at
//! `/metrics` (Prometheus text format), storage in one SQLite file. Configuration comes
//! from environment variables ([`Config`]). See architecture chapter 7.

mod app;
pub mod config;
pub mod store;

use std::net::SocketAddr;
use std::sync::Arc;

use ordo_core::{Author, DeviceId, KeyId, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::{SigningKey, VerifyingKey};
use ordo_med_patients::PatientCountsDeriver;
use ordo_sync::{Hub, HubConfig, StaticPolicy};
use tokio::sync::watch;
use tokio::task::JoinHandle;

pub use config::Config;

use crate::app::App;
use crate::store::{Store, StoreError};

/// The server could not start or stopped with an error.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    /// Configuration error.
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    /// Storage error.
    #[error(transparent)]
    Store(#[from] StoreError),
    /// Stored events are inconsistent (changed data?).
    #[error("restoring stored events failed: {0}")]
    Restore(#[from] ordo_sync::RestoreError),
    /// Network error.
    #[error("network: {0}")]
    Io(#[from] std::io::Error),
    /// Storing failed while running; the server stopped so that it can be restarted.
    #[error("storing failed while running")]
    StorageFailed,
}

/// A running server.
pub struct Running {
    /// The address it listens on.
    pub addr: SocketAddr,
    stop: watch::Sender<bool>,
    task: JoinHandle<Result<(), ServerError>>,
}

impl Running {
    /// Stops the server gracefully: devices are disconnected and reconnect later.
    pub async fn stop(self) -> Result<(), ServerError> {
        let _ = self.stop.send(true);
        self.task.await.unwrap_or(Err(ServerError::StorageFailed))
    }

    /// Waits until the server stops by itself (e.g. after a storage failure).
    pub async fn wait(self) -> Result<(), ServerError> {
        self.task.await.unwrap_or(Err(ServerError::StorageFailed))
    }

    /// A handle that stops the server when sent `true`.
    pub fn stopper(&self) -> watch::Sender<bool> {
        self.stop.clone()
    }
}

/// Opens storage, restores all streams and starts listening.
pub async fn start(config: Config) -> Result<Running, ServerError> {
    std::fs::create_dir_all(&config.data_dir)?;
    let store = Store::open(&config.data_dir.join("ordo.db")).await?;
    let master = config.master_key.clone();

    // The server's own identity: signing key and device ID, stored wrapped.
    let signing = store
        .secret("server-signing-key", &master, || {
            random_bytes::<32>().to_vec()
        })
        .await?;
    let signing: [u8; 32] = signing
        .try_into()
        .map_err(|_| StoreError::Invalid("server signing key length".into()))?;
    let device = store
        .secret("server-device", &master, || {
            let random = u128::from_be_bytes(random_bytes::<16>());
            ordo_core::Ulid::from_parts(app::now_ms(), random)
                .expect("current time fits into 48 bits")
                .to_bytes()
                .to_vec()
        })
        .await?;
    let device: [u8; 16] = device
        .try_into()
        .map_err(|_| StoreError::Invalid("server device length".into()))?;
    let server = &config.server_name;
    let identity = Author {
        user: UserId::new("ordo-system", server).expect("valid system user"),
        device: DeviceId(ordo_core::Ulid::from_bytes(device)),
    };
    let signer = EventSigner::new(
        SigningKey::from_bytes(&signing),
        KeyId::new("server-1", server).expect("valid key name"),
    );

    // Access rules and devices from the seed.
    let mut policy = StaticPolicy::default();
    for user in &config.seed.users {
        let compartments: Vec<&str> = user.compartments.iter().map(|c| c.as_str()).collect();
        policy = policy.allow(&user.id, &compartments);
    }
    let mut hub = Hub::new(
        HubConfig {
            server: server.clone(),
            protected: config.protected.clone(),
            max_clock_ahead_ms: 60_000,
        },
        policy,
        signer,
        identity,
        |buffer: &mut [u8]| {
            getrandom::fill(buffer).expect("the operating system provides randomness")
        },
    );
    for device in &config.seed.devices {
        let key = VerifyingKey::from_bytes(&device.key_bytes()?).map_err(|e| {
            config::ConfigError::Invalid {
                name: format!("device {}", device.id),
                message: e.to_string(),
            }
        })?;
        hub.register_device(device.device_id()?, device.user.clone(), key);
    }
    hub.add_deriver(PatientCountsDeriver::default());

    // Restore keys first, then events in sequence order.
    for (stream, key_id, key) in store.load_keys(&master).await? {
        hub.restore_key(stream, key_id, key);
    }
    let events = store.load_events().await?;
    let restored = events.len();
    for (stream, event) in events {
        hub.restore_event(stream, event)?;
    }
    let _ = hub.take_changes(); // restoring creates no changes, but be sure

    let listener = tokio::net::TcpListener::bind(config.listen).await?;
    let addr = listener.local_addr()?;
    let (stop, stopped) = watch::channel(false);
    let app = Arc::new(App::new(hub, store, master, stop.clone()));
    let router = app.clone().router();
    tracing::info!(%addr, server = %config.server_name, restored_events = restored, "listening");
    let task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(app::stopped(stopped))
            .await?;
        if app.failed() {
            return Err(ServerError::StorageFailed);
        }
        Ok(())
    });
    Ok(Running { addr, stop, task })
}

fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    getrandom::fill(&mut bytes).expect("the operating system provides randomness");
    bytes
}
