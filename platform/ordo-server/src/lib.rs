//! The ORDO server (`ordo serve`).
//!
//! One process: WebSocket sync at `/sync`, health at `/healthz` and `/readyz`, metrics at
//! `/metrics` (Prometheus text format), storage in one SQLite file. Configuration comes
//! from environment variables ([`Config`]). See architecture chapter 7.

mod app;
pub mod authz;
mod blobs;
pub mod config;
mod login;
pub mod store;

use std::net::SocketAddr;
use std::sync::Arc;

use ordo_core::{Author, DeviceId, KeyId, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::{SigningKey, VerifyingKey};
use ordo_med_patients::PatientCountsDeriver;
use ordo_sync::{Hub, HubConfig};
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
    /// A policy file is invalid.
    #[error(transparent)]
    Policy(#[from] authz::PolicyError),
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

    // [impl->dsn~role-policies~1]
    // Organisation roles: administrators from the configuration, plus roles from the seed
    // (development only). Roles in an incident come from its roster (ADR-0046); identity
    // provider groups grant nothing. Permissions come from Cedar policies (defaults plus
    // the organisation's files); invalid policies stop the start.
    let roles = authz::Roles::new(
        config
            .seed
            .users
            .iter()
            .map(|u| (u.id.clone(), u.roles.clone())),
    );
    for admin in &config.admins {
        roles.add(admin, &["admin".to_owned()]);
    }
    let mut extra = Vec::new();
    if let Some(dir) = &config.policy_dir {
        let mut files: Vec<_> = std::fs::read_dir(dir)?
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "cedar"))
            .collect();
        files.sort();
        for path in files {
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            extra.push((name, std::fs::read_to_string(&path)?));
        }
    }
    let policy = authz::RolePolicy::new(roles.clone(), &extra)?;
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
    // Devices paired by login (ADR-0038), each with the end of its login (ADR-0040); a
    // device without a recorded end has to log in again.
    for (device, user, key, valid_until) in store.load_devices().await? {
        let key = VerifyingKey::from_bytes(&key)
            .map_err(|e| StoreError::Invalid(format!("device key: {e}")))?;
        hub.register_device_until(device, user, key, Some(valid_until.unwrap_or(0)));
    }
    for (device, paired) in store.load_paired().await? {
        hub.set_paired(&device, paired);
    }
    for device in store.load_revoked().await? {
        hub.restore_revoked(device);
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
    let login = match &config.oidc {
        Some(oidc) => Some(
            login::Login::new(oidc.clone())
                .map_err(|e| config::ConfigError::invalid("ORDO_OIDC_ISSUER", e.to_string()))?,
        ),
        None => None,
    };
    let app = Arc::new(App::new(
        hub,
        store,
        master,
        config.server_name.clone(),
        login,
        config.access_timeout,
        config.gravatar,
        blobs::Blobs::local(&config.data_dir.join("blobs"), config.max_blob_bytes)?,
        stop.clone(),
    ));
    tokio::spawn(app.clone().maintenance());
    let router = app.clone().router(config.app_dir.as_deref());
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
