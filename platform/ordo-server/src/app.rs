//! HTTP and WebSocket handling.

use ordo_sync::version::{HTTP, HTTP_HEADER, Version};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::Router;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use futures_util::{SinkExt, StreamExt};
use ordo_core::{DeviceId, ServerName, UserId};
use ordo_crypto::ed25519_dalek::VerifyingKey;
use ordo_crypto::seal::DataKey;
use ordo_sync::message::{self, Avatar, BlobRequest, BlobTicket, ClientMessage, ServerMessage};
use ordo_sync::{Hub, SessionId};

use crate::authz::RolePolicy;
use crate::login::{self, Login};
use tokio::sync::{Mutex, mpsc, watch};

use crate::blobs::Blobs;
use crate::store::{AvatarSource, Store, StoreError};

/// Shared state of the running server.
pub struct App {
    pub(crate) hub: Mutex<Hub<RolePolicy>>,
    pub(crate) store: Store,
    pub(crate) master: DataKey,
    /// This server's name.
    pub(crate) server: ServerName,
    /// Login with OpenID Connect, if configured (ADR-0038).
    pub(crate) login: Option<Login>,
    /// How long a login lasts without re-verification (ADR-0040).
    pub(crate) access_timeout: std::time::Duration,
    /// Look up profile pictures at Gravatar (ADR-0047).
    pub(crate) gravatar: bool,
    /// Attachments (ADR-0033).
    pub(crate) blobs: Blobs,
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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        hub: Hub<RolePolicy>,
        store: Store,
        master: DataKey,
        server: ServerName,
        login: Option<Login>,
        access_timeout: std::time::Duration,
        gravatar: bool,
        blobs: Blobs,
        stop: watch::Sender<bool>,
    ) -> Self {
        Self {
            hub: Mutex::new(hub),
            store,
            master,
            server,
            login,
            access_timeout,
            gravatar,
            blobs,
            sessions: std::sync::Mutex::new(HashMap::new()),
            next_session: AtomicU64::new(1),
            shutdown: stop.subscribe(),
            stop,
            failed: AtomicBool::new(false),
            metrics: Metrics::default(),
        }
    }

    /// The end of a login that is verified now.
    pub(crate) fn valid_from_now(&self) -> u64 {
        now_ms().saturating_add(u64::try_from(self.access_timeout.as_millis()).unwrap_or(u64::MAX))
    }

    /// Pairs a device with a user: stored first, then known to the hub, so the device can
    /// connect at once and after a restart. The person just logged in, so the logins of
    /// all their devices are renewed. Returns `false` if the device ID belongs to another
    /// user (no takeover).
    pub(crate) async fn register_device(
        &self,
        device: DeviceId,
        user: UserId,
        key: VerifyingKey,
    ) -> Result<bool, StoreError> {
        let valid_until = self.valid_from_now();
        if !self
            .store
            .save_device(&device, &user, key.as_bytes(), now_ms(), valid_until)
            .await?
        {
            return Ok(false);
        }
        {
            let mut hub = self.hub.lock().await;
            hub.register_device_until(device, user.clone(), key, Some(valid_until));
            hub.set_paired(&device, now_ms());
        }
        self.set_valid_until(&user, valid_until).await?;
        Ok(true)
    }

    // [impl->dsn~access-timeout~1]
    /// Sets the end of the login of all devices of `user`, and cuts connections whose login
    /// has ended.
    pub(crate) async fn set_valid_until(
        &self,
        user: &UserId,
        valid_until: u64,
    ) -> Result<(), StoreError> {
        self.store.set_valid_until(user, valid_until).await?;
        self.hub.lock().await.set_valid_until(user, valid_until);
        self.expire_sessions().await;
        Ok(())
    }

    /// Cuts the connections of devices whose login has expired: they receive the refusal,
    /// then their connection closes.
    pub(crate) async fn expire_sessions(&self) {
        let refused = self.hub.lock().await.expire(now_ms());
        let mut sessions = self.sessions.lock().expect("session registry lock");
        for out in refused {
            if let Some(sender) = sessions.remove(&out.session) {
                tracing::info!(session = out.session, "login expired; connection closed");
                let _ = sender.send(out.message);
            }
        }
    }

    /// Regular work: cut expired connections every minute; re-verify logins in the
    /// background every 15 minutes (ADR-0040).
    pub(crate) async fn maintenance(self: Arc<Self>) {
        let mut minute = tokio::time::interval(std::time::Duration::from_secs(60));
        let mut quarter = tokio::time::interval(std::time::Duration::from_secs(15 * 60));
        let stop = stopped(self.shutdown.clone());
        tokio::pin!(stop);
        loop {
            tokio::select! {
                _ = minute.tick() => self.expire_sessions().await,
                _ = quarter.tick() => login::reverify(&self).await,
                () = &mut stop => return,
            }
        }
    }

    /// Whether storage failed; the process should then exit so it is restarted.
    pub(crate) fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    pub(crate) fn router(
        self: Arc<Self>,
        app_dir: Option<&std::path::Path>,
        map_file: Option<&std::path::Path>,
    ) -> Router {
        use axum::http::{HeaderName, HeaderValue};
        use tower_http::services::{ServeDir, ServeFile};
        use tower_http::set_header::SetResponseHeaderLayer;
        // The HTTP API: versioned per request (ADR-0052).
        let versioned = Router::new()
            .route("/auth/config", get(login::config))
            .route("/auth/login", get(login::start))
            .route("/auth/callback", get(login::callback))
            .route("/auth/device", post(login::device_start))
            .route("/auth/device/{id}", get(login::device_status))
            .route("/auth/device/{id}/qr.svg", get(login::device_qr))
            .route(
                "/blobs/{hash}",
                get(crate::blobs::get).put(crate::blobs::put).layer(
                    axum::extract::DefaultBodyLimit::max(
                        usize::try_from(self.blobs.max_bytes).unwrap_or(usize::MAX),
                    ),
                ),
            )
            .layer(axum::middleware::from_fn(api_version));
        let api = Router::new()
            .route("/sync", get(sync))
            .route("/healthz", get(|| async { "ok\n" }))
            .route("/readyz", get(readyz))
            .route("/metrics", get(metrics))
            .route("/.well-known/ordo", get(well_known))
            .merge(versioned)
            .with_state(self);
        // [impl->dsn~situation-map~1]
        // The map region (ADR-0006), with range requests; map data, not personal data.
        let api = match map_file {
            Some(file) => api.route_service("/tiles/region.pmtiles", ServeFile::new(file)),
            None => api,
        };
        let router = match app_dir {
            Some(dir) => {
                // Unknown paths fall back to index.html, so the app handles its own routes.
                let files = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
                Router::new()
                    .fallback_service(files)
                    .layer(axum::middleware::from_fn(cache_control))
                    .merge(api)
            }
            None => api,
        };
        let header = |name: &'static str, value: &'static str| {
            SetResponseHeaderLayer::if_not_present(
                HeaderName::from_static(name),
                HeaderValue::from_static(value),
            )
        };
        router
            .layer(header("x-content-type-options", "nosniff"))
            .layer(header("referrer-policy", "no-referrer"))
            .layer(header(
                "content-security-policy",
                "frame-ancestors 'none'; object-src 'none'; base-uri 'self'",
            ))
    }

    // [impl->dsn~store-before-send~1]
    /// Handles one message: the hub decides, the store persists, then messages are sent.
    /// Sending happens while the hub is locked, so every device receives deliveries in
    /// sequence order.
    async fn handle(&self, session: SessionId, message: ClientMessage) -> Result<(), StoreError> {
        // Profile pictures live in the store, not in the hub (ADR-0047).
        if let ClientMessage::GetAvatars(_) | ClientMessage::SetAvatar(_) = &message {
            return self.avatars(session, message).await;
        }
        if let ClientMessage::BlobTicket(request) = message {
            return self.blob_ticket(session, request).await;
        }
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
        let mut sessions = self.sessions.lock().expect("session registry lock");
        for out in outgoing {
            if matches!(out.message, ServerMessage::Rejected(_)) {
                self.metrics.rejected.fetch_add(1, Ordering::Relaxed);
            }
            // A refusal ends the connection: it is sent, then the connection closes.
            let sender = if matches!(out.message, ServerMessage::Refused(_)) {
                sessions.remove(&out.session)
            } else {
                sessions.get(&out.session).cloned()
            };
            if let Some(sender) = sender {
                let _ = sender.send(out.message);
            }
        }
        Ok(())
    }

    // [impl->dsn~attachments~1]
    /// Answers `blobTicket`: a ticket if the user may write (upload) or read (download) the
    /// attachment's stream, and — for downloads — the attachment belongs to that stream.
    async fn blob_ticket(
        &self,
        session: SessionId,
        request: BlobRequest,
    ) -> Result<(), StoreError> {
        let allowed = {
            let hub = self.hub.lock().await;
            hub.session_user(session).is_some_and(|user| {
                hub.can_access(
                    &user,
                    &request.incident,
                    &request.compartment,
                    request.upload,
                )
            })
        };
        let stream = (request.incident.clone(), request.compartment.clone());
        let allowed = allowed
            && if request.upload {
                request.size <= self.blobs.max_bytes
            } else {
                self.store
                    .blob_in(&crate::blobs::hex(&request.hash), &stream)
                    .await?
            };
        let ticket = allowed.then(|| {
            self.blobs
                .issue(request.hash, request.size, request.upload, stream)
        });
        if let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
        {
            let _ = sender.send(ServerMessage::BlobTicket(BlobTicket {
                hash: request.hash,
                upload: request.upload,
                ticket,
            }));
        }
        Ok(())
    }

    /// Answers `getAvatars` and `setAvatar` of an authenticated connection.
    async fn avatars(&self, session: SessionId, message: ClientMessage) -> Result<(), StoreError> {
        let Some(user) = self.hub.lock().await.session_user(session) else {
            return Ok(()); // not authenticated: nothing
        };
        let users = match message {
            ClientMessage::GetAvatars(mut users) => {
                users.truncate(200);
                users
            }
            ClientMessage::SetAvatar(set) => {
                let image = set.image.filter(|i| ordo_sync::message::is_avatar_image(i));
                self.store
                    .set_avatar(&user, AvatarSource::Ordo, image.as_deref(), now_ms())
                    .await?;
                vec![user]
            }
            _ => return Ok(()),
        };
        let mut avatars = Vec::new();
        for user in users {
            let image = self.store.avatar(&user).await?;
            avatars.push(Avatar { user, image });
        }
        if let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
        {
            let _ = sender.send(ServerMessage::Avatars(avatars));
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
        // The device must sign this challenge in its hello (ADR-0038).
        for out in self.hub.lock().await.connect(id) {
            if let Some(sender) = self
                .sessions
                .lock()
                .expect("session registry lock")
                .get(&out.session)
            {
                let _ = sender.send(out.message);
            }
        }

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

/// Cache policy for the web app: files with a content hash in their name never change and
/// are cached for a year; everything else (app shell, service worker, manifest) is
/// revalidated on every load, so updates arrive at once and offline starts come from the
/// service worker rather than from a stale HTTP cache.
// [impl->dsn~protocol-versions~1]
/// Answers in the HTTP API version the request names (none: the oldest), and refuses a
/// version outside the supported range with `406` and the range.
async fn api_version(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::{HeaderValue, StatusCode};
    use axum::response::IntoResponse;
    let asked = request.headers().get(HTTP_HEADER).map(|v| {
        v.to_str()
            .ok()
            .and_then(|s| s.trim().parse::<Version>().ok())
    });
    let version = match asked {
        None => HTTP.min,
        Some(Some(v)) if HTTP.contains(v) => v,
        Some(_) => {
            return (
                StatusCode::NOT_ACCEPTABLE,
                axum::Json(serde_json::json!({ "http": HTTP })),
            )
                .into_response();
        }
    };
    let mut response = next.run(request).await;
    if let Ok(value) = HeaderValue::from_str(&version.to_string()) {
        response.headers_mut().insert(HTTP_HEADER, value);
    }
    response
}

// [impl->dsn~protocol-versions~1]
/// `/.well-known/ordo`: the versions of each specification this server supports.
async fn well_known() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "sync": ordo_sync::version::SYNC,
        "http": HTTP,
    }))
}

async fn cache_control(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let hashed = is_hashed_asset(request.uri().path());
    let mut response = next.run(request).await;
    let value = if hashed {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    response
        .headers_mut()
        .entry(axum::http::header::CACHE_CONTROL)
        .or_insert(axum::http::HeaderValue::from_static(value));
    response
}

/// Whether a path names a build asset with a content hash (Trunk: `name-<16 hex>`).
fn is_hashed_asset(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or_default();
    let stem = name.split('.').next().unwrap_or_default();
    stem.split(['-', '_'])
        .any(|part| part.len() == 16 && part.bytes().all(|b| b.is_ascii_hexdigit()))
}

#[cfg(test)]
mod cache_tests {
    use super::is_hashed_asset;

    #[test]
    fn only_hashed_assets_are_cached_for_long() {
        assert!(is_hashed_asset("/ordo-app-4047fc351e362fcb_bg.wasm"));
        assert!(is_hashed_asset("/app-4f18922c059077f8.css"));
        assert!(!is_hashed_asset("/"));
        assert!(!is_hashed_asset("/index.html"));
        assert!(!is_hashed_asset("/sw.js"));
        assert!(!is_hashed_asset("/med.uhs/patients/DE-0042"));
    }
}
