//! HTTP and WebSocket handling.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use ordo_crypto::seal::DataKey;
use ordo_sync::message::{self, ClientMessage, ServerMessage};
use ordo_sync::{Hub, SessionId, StaticPolicy};
use tokio::sync::{Mutex, mpsc, watch};

use crate::store::{Store, StoreError};

/// Shared state of the running server.
pub struct App {
    pub(crate) hub: Mutex<Hub<StaticPolicy>>,
    pub(crate) store: Store,
    pub(crate) master: DataKey,
    sessions: std::sync::Mutex<HashMap<SessionId, mpsc::UnboundedSender<ServerMessage>>>,
    next_session: AtomicU64,
    shutdown: watch::Receiver<bool>,
    stop: watch::Sender<bool>,
    failed: AtomicBool,
    metrics: Metrics,
}

#[derive(Default)]
struct Metrics {
    events: AtomicU64,
    rejected: AtomicU64,
    sessions: AtomicI64,
}

impl App {
    pub(crate) fn new(
        hub: Hub<StaticPolicy>,
        store: Store,
        master: DataKey,
        stop: watch::Sender<bool>,
    ) -> Self {
        Self {
            hub: Mutex::new(hub),
            store,
            master,
            sessions: std::sync::Mutex::new(HashMap::new()),
            next_session: AtomicU64::new(1),
            shutdown: stop.subscribe(),
            stop,
            failed: AtomicBool::new(false),
            metrics: Metrics::default(),
        }
    }

    /// Whether storage failed; the process should then exit so it is restarted.
    pub(crate) fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    pub(crate) fn router(self: Arc<Self>) -> Router {
        Router::new()
            .route("/sync", get(sync))
            .route("/healthz", get(|| async { "ok\n" }))
            .route("/readyz", get(readyz))
            .route("/metrics", get(metrics))
            .with_state(self)
    }

    // [impl->dsn~store-before-send~1]
    /// Handles one message: the hub decides, the store persists, then messages are sent.
    /// Sending happens while the hub is locked, so every device receives deliveries in
    /// sequence order.
    async fn handle(&self, session: SessionId, message: ClientMessage) -> Result<(), StoreError> {
        let mut hub = self.hub.lock().await;
        let outgoing = hub.handle(session, message, now_ms());
        let changes = hub.take_changes();
        self.store.save(&changes, &self.master).await?;
        let new_events = changes
            .iter()
            .filter(|c| matches!(c, ordo_sync::Change::Event { .. }))
            .count();
        self.metrics
            .events
            .fetch_add(new_events as u64, Ordering::Relaxed);
        let sessions = self.sessions.lock().expect("session registry lock");
        for out in outgoing {
            if matches!(out.message, ServerMessage::Rejected(_)) {
                self.metrics.rejected.fetch_add(1, Ordering::Relaxed);
            }
            if let Some(sender) = sessions.get(&out.session) {
                let _ = sender.send(out.message);
            }
        }
        Ok(())
    }

    async fn session(self: Arc<Self>, socket: WebSocket) {
        let id = self.next_session.fetch_add(1, Ordering::SeqCst);
        let (sender, mut receiver) = mpsc::unbounded_channel::<ServerMessage>();
        self.sessions
            .lock()
            .expect("session registry lock")
            .insert(id, sender);
        self.metrics.sessions.fetch_add(1, Ordering::Relaxed);
        tracing::info!(session = id, "connected");

        let (mut sink, mut stream) = socket.split();
        let writer = tokio::spawn(async move {
            while let Some(message) = receiver.recv().await {
                let Ok(bytes) = message::encode(&message) else {
                    continue;
                };
                if sink.send(Message::Binary(bytes.into())).await.is_err() {
                    break;
                }
            }
            let _ = sink.close().await;
        });

        let stop = stopped(self.shutdown.clone());
        tokio::pin!(stop);
        loop {
            tokio::select! {
                frame = stream.next() => match frame {
                    Some(Ok(Message::Binary(bytes))) => match message::decode::<ClientMessage>(&bytes) {
                        Ok(message) => {
                            if let Err(error) = self.handle(id, message).await {
                                tracing::error!(%error, "storing failed; shutting down so the server is restarted");
                                self.failed.store(true, Ordering::SeqCst);
                                let _ = self.stop.send(true);
                                break;
                            }
                        }
                        Err(error) => {
                            tracing::warn!(session = id, %error, "invalid message; closing");
                            break;
                        }
                    },
                    Some(Ok(Message::Close(_))) | None | Some(Err(_)) => break,
                    Some(Ok(_)) => {}
                },
                () = &mut stop => break,
            }
        }

        self.sessions
            .lock()
            .expect("session registry lock")
            .remove(&id);
        self.hub.lock().await.disconnect(id);
        self.metrics.sessions.fetch_sub(1, Ordering::Relaxed);
        let _ = writer.await;
        tracing::info!(session = id, "disconnected");
    }
}

async fn sync(State(app): State<Arc<App>>, upgrade: WebSocketUpgrade) -> Response {
    upgrade.on_upgrade(move |socket| app.session(socket))
}

async fn readyz(State(app): State<Arc<App>>) -> Response {
    if !app.failed() && app.store.ping().await {
        "ready\n".into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready\n").into_response()
    }
}

async fn metrics(State(app): State<Arc<App>>) -> String {
    let m = &app.metrics;
    format!(
        "# HELP ordo_events_total Events ordered by this server since start.\n\
         # TYPE ordo_events_total counter\n\
         ordo_events_total {}\n\
         # HELP ordo_rejected_total Events rejected since start.\n\
         # TYPE ordo_rejected_total counter\n\
         ordo_rejected_total {}\n\
         # HELP ordo_sessions Connected devices.\n\
         # TYPE ordo_sessions gauge\n\
         ordo_sessions {}\n",
        m.events.load(Ordering::Relaxed),
        m.rejected.load(Ordering::Relaxed),
        m.sessions.load(Ordering::Relaxed),
    )
}

/// Completes when `true` is sent (or the sender is gone).
pub(crate) async fn stopped(mut receiver: watch::Receiver<bool>) {
    loop {
        if *receiver.borrow_and_update() {
            return;
        }
        if receiver.changed().await.is_err() {
            return;
        }
    }
}

/// Milliseconds since the Unix epoch.
pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}
