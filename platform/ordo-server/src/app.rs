//! HTTP and WebSocket handling.

use ordo_sync::version::{HTTP, HTTP_HEADER, Version};
use std::collections::{HashMap, HashSet};
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
    /// The incidents' map regions and their cutting (ADR-0057, ADR-0062).
    pub(crate) map: Arc<crate::map_region::MapJob>,
    /// Address search for the incident's map (ADR-0066).
    pub(crate) geocoder: Arc<crate::geocode::Geocoder>,
    /// Organisation roles (ORG-9), shared with the access policy.
    roles: crate::authz::Roles,
    /// Dex's API for local accounts (ORG-10), if configured.
    dex_api: Option<crate::dex_api::DexApi>,
    /// Test logins through new or changed upstream providers (ORG-11).
    pub(crate) login_tests: crate::providers::Tests,
    /// Whether temporary connectors left from before the start were looked for.
    tests_cleaned: AtomicBool,
    /// Up to when lapsed role grants were looked for (ms).
    roles_checked: AtomicU64,
    sessions: std::sync::Mutex<HashMap<SessionId, mpsc::UnboundedSender<ServerMessage>>>,
    /// The people each connection asked about: it hears when their name or picture
    /// changes (ORG-7).
    asked: std::sync::Mutex<HashMap<SessionId, HashSet<UserId>>>,
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
        map: crate::map_region::MapJob,
        geocoder: crate::geocode::Geocoder,
        roles: crate::authz::Roles,
        dex_api: Option<crate::dex_api::DexApi>,
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
            map: Arc::new(map),
            geocoder: Arc::new(geocoder),
            roles,
            dex_api,
            login_tests: crate::providers::Tests::default(),
            tests_cleaned: AtomicBool::new(false),
            roles_checked: AtomicU64::new(now_ms()),
            sessions: std::sync::Mutex::new(HashMap::new()),
            asked: std::sync::Mutex::new(HashMap::new()),
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
        name: Option<String>,
    ) -> Result<bool, StoreError> {
        let valid_until = self.valid_from_now();
        if !self
            .store
            .save_device(
                &device,
                &user,
                key.as_bytes(),
                name.as_deref(),
                now_ms(),
                valid_until,
            )
            .await?
        {
            return Ok(false);
        }
        {
            let mut hub = self.hub.lock().await;
            hub.register_device_until(device, user.clone(), key, Some(valid_until));
            hub.set_paired(&device, now_ms(), name);
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

    // [impl->dsn~organisation-roles~1]
    /// Cuts the connections of people whose organisation role ran out since the last
    /// check: their devices reconnect with what they may do now.
    pub(crate) async fn expire_roles(&self) {
        let now = now_ms();
        let since = self.roles_checked.swap(now, Ordering::Relaxed);
        let lapsed = self.roles.lapsed(since, now);
        if !lapsed.is_empty() {
            tracing::info!(people = lapsed.len(), "organisation roles ran out");
            let out = self.hub.lock().await.organisation_roles_changed(&lapsed);
            self.send_refusals(out);
        }
    }

    // [impl->dsn~display-names~2]
    /// Names of the local accounts (ADR-0060) for people ORDO has no name of yet, e.g.
    /// before their first login, so rosters show them by name (ORG-7). A name from a login
    /// or one the person chose always wins.
    pub(crate) async fn names_from_accounts(&self) {
        let (Some(api), Some(login)) = (&self.dex_api, &self.login) else {
            return;
        };
        // Dex may not be up yet; the next minute tries again.
        let Ok(accounts) = api.list().await else {
            return;
        };
        let filled = async {
            for account in accounts {
                let Some(name) =
                    crate::login::tidy_name(Some(&account.username), crate::login::MAX_PERSON_NAME)
                else {
                    continue;
                };
                let subject = crate::dex_api::local_subject(&account.user_id);
                let user = match self
                    .store
                    .user_of_identity(login.issuer(), &subject)
                    .await?
                {
                    Some(user) => user,
                    // Not logged in yet: the user ID the login would give, if still free.
                    None => {
                        let wanted = crate::login::user_name(None, Some(&account.email));
                        let Ok(user) = UserId::new(&wanted, &self.server) else {
                            continue;
                        };
                        if self.store.has_identity(&user).await? {
                            continue;
                        }
                        user
                    }
                };
                if self.store.fill_provider_name(&user, &name).await? {
                    self.person_changed(&user).await?;
                }
            }
            Ok::<(), StoreError>(())
        };
        if let Err(error) = filled.await {
            tracing::error!(%error, "names from local accounts failed");
        }
    }

    /// Sends refusals and forgets their connections (they close after sending).
    fn send_refusals(&self, out: Vec<ordo_sync::Outgoing>) {
        let mut sessions = self.sessions.lock().expect("session registry lock");
        for out in out {
            if let Some(sender) = sessions.remove(&out.session) {
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
                _ = minute.tick() => {
                    self.expire_sessions().await;
                    self.expire_roles().await;
                    self.names_from_accounts().await;
                    self.clean_login_tests().await;
                }
                _ = quarter.tick() => login::reverify(&self).await,
                () = &mut stop => return,
            }
        }
    }

    /// Whether storage failed; the process should then exit so it is restarted.
    pub(crate) fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    pub(crate) fn router(self: Arc<Self>, app_dir: Option<&std::path::Path>) -> Router {
        use axum::http::{HeaderName, HeaderValue};
        use tower_http::services::{ServeDir, ServeFile};
        use tower_http::set_header::SetResponseHeaderLayer;
        // The HTTP API: versioned per request (ADR-0052).
        let versioned = Router::new()
            .route("/auth/config", get(login::config))
            .route("/auth/login", get(login::start))
            .route("/auth/callback", get(login::callback))
            .route("/auth/test/{token}", get(login::test))
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
            // [impl->dsn~situation-map~3]
            // An incident's map region (ADR-0006, ADR-0062), with range requests and a
            // ticket. Not found until one is cut.
            .route("/tiles/{name}", get(crate::map_region::serve))
            .merge(versioned)
            .with_state(self);
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
        let header = |name: &'static str, value: HeaderValue| {
            SetResponseHeaderLayer::if_not_present(HeaderName::from_static(name), value)
        };
        let policy =
            HeaderValue::from_str(&content_security_policy(app_dir)).expect("the policy is ASCII");
        router
            .layer(header(
                "x-content-type-options",
                HeaderValue::from_static("nosniff"),
            ))
            .layer(header(
                "referrer-policy",
                HeaderValue::from_static("no-referrer"),
            ))
            .layer(header("content-security-policy", policy))
    }

    // [impl->dsn~store-before-send~1]
    /// Handles one message: the hub decides, the store persists, then messages are sent.
    /// Sending happens while the hub is locked, so every device receives deliveries in
    /// sequence order.
    async fn handle(
        self: &Arc<Self>,
        session: SessionId,
        message: ClientMessage,
    ) -> Result<(), StoreError> {
        // Profile pictures live in the store, not in the hub (ADR-0047).
        if let ClientMessage::GetAvatars(_)
        | ClientMessage::SetAvatar(_)
        | ClientMessage::SetName(_) = &message
        {
            return self.avatars(session, message).await;
        }
        if let ClientMessage::BlobTicket(request) = message {
            return self.blob_ticket(session, request).await;
        }
        if let ClientMessage::GetLoginProviders
        | ClientMessage::TestLoginProvider(_)
        | ClientMessage::SaveLoginProvider(_)
        | ClientMessage::DeleteLoginProvider(_) = message
        {
            return self.login_providers(session, message).await;
        }
        if let ClientMessage::GetAccounts | ClientMessage::ChangeAccount(_) = message {
            return self.accounts(session, message).await;
        }
        if let ClientMessage::GetOrganisationRoles | ClientMessage::SetOrganisationRole(_) = message
        {
            return self.organisation_roles(session, message).await;
        }
        if let ClientMessage::GetMapRegion(_)
        | ClientMessage::CutMapRegion(_)
        | ClientMessage::MapTicket(_) = message
        {
            return self.map_region(session, message).await;
        }
        if let ClientMessage::Geocode(request) = message {
            self.geocode(session, request).await;
            return Ok(());
        }
        let mut hub = self.hub.lock().await;
        // [impl->dsn~incident-devices~1]
        // Who blocked which device in which incident is logged.
        let block = match &message {
            ClientMessage::BlockDevice(block) => Some((block.clone(), hub.session_user(session))),
            _ => None,
        };
        let outgoing = hub.handle(session, message, now_ms());
        let changes = hub.take_changes();
        if let Some((block, Some(user))) = block
            && changes.iter().any(
                |c| matches!(c, ordo_sync::Change::Revoked { device } if *device == block.device),
            )
        {
            tracing::warn!(%user, incident = %block.incident, device = %block.device.ulid(), "device blocked in the incident");
        }
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

    // [impl->dsn~map-region~2]
    /// Answers `getMapRegion` and `cutMapRegion` (protocol 1.2): an incident's region and a
    /// new cut, for users with the incident permission `manage_map`; everyone else learns
    /// nothing (state `none`). Answers `mapTicket` with a ticket to read the region for
    /// those who may read the incident's shared compartment.
    async fn map_region(
        &self,
        session: SessionId,
        message: ClientMessage,
    ) -> Result<(), StoreError> {
        use ordo_sync::message::{MapRegion, MapTicket};
        let incident = match &message {
            ClientMessage::GetMapRegion(incident) | ClientMessage::MapTicket(incident) => {
                incident.clone()
            }
            ClientMessage::CutMapRegion(cut) => cut.incident.clone(),
            _ => return Ok(()),
        };
        let shared: ordo_core::CompartmentId = "incident".parse().expect("valid compartment");
        let (user, version, may_cut, may_read) = {
            let hub = self.hub.lock().await;
            let Some(user) = hub.session_user(session) else {
                return Ok(()); // not authenticated: nothing
            };
            let may_cut = hub.may_in_incident(&user, &incident, "manage_map");
            let may_read = hub.can_access(&user, &incident, &shared, false);
            (user, hub.session_version(session), may_cut, may_read)
        };
        if version.is_none_or(|v| v < ordo_sync::version::SYNC_MAP_REGION) {
            return Ok(());
        }
        let answer = match (message, may_cut) {
            (ClientMessage::MapTicket(_), _) => ServerMessage::MapTicket(MapTicket {
                ticket: may_read.then(|| self.map.issue_ticket(&incident)),
                incident,
            }),
            (_, false) => ServerMessage::MapRegion(MapRegion {
                incident: Some(incident),
                ..MapRegion::default()
            }),
            (ClientMessage::CutMapRegion(cut), true) => {
                let area = cut.area;
                tracing::info!(%user, %incident, lat_e7 = area.lat_e7, lon_e7 = area.lon_e7, radius_m = area.radius_m, "cutting a map region");
                ServerMessage::MapRegion(self.map.start(&incident, area, now_ms()))
            }
            (_, true) => ServerMessage::MapRegion(self.map.state(&incident)),
        };
        if let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
        {
            let _ = sender.send(answer);
        }
        Ok(())
    }

    // [impl->dsn~address-search~1]
    /// Answers `geocode` (protocol 1.2, ADR-0066) for those who may prepare or edit the
    /// incident's map (`manage_map` or `edit_units`). The search runs apart, so a request
    /// waiting for the geocoder's pace holds up nothing else of the connection. The query
    /// is not logged: it may name where people are.
    async fn geocode(&self, session: SessionId, request: ordo_sync::message::GeocodeRequest) {
        use ordo_sync::message::{GeocodeStatus, Geocoded};
        let (may, version) = {
            let hub = self.hub.lock().await;
            let Some(user) = hub.session_user(session) else {
                return; // not authenticated: nothing
            };
            let may = hub.may_in_incident(&user, &request.incident, "manage_map")
                || hub.may_in_incident(&user, &request.incident, "edit_units");
            (may, hub.session_version(session))
        };
        if version.is_none_or(|v| v < ordo_sync::version::SYNC_GEOCODE) {
            return;
        }
        let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
            .cloned()
        else {
            return;
        };
        if !may {
            let _ = sender.send(ServerMessage::Geocoded(Geocoded {
                query: request.query,
                status: GeocodeStatus::Refused,
                places: Vec::new(),
            }));
            return;
        }
        let geocoder = self.geocoder.clone();
        tokio::spawn(async move {
            let (status, places) = geocoder.search(&request.query).await;
            let _ = sender.send(ServerMessage::Geocoded(Geocoded {
                query: request.query,
                status,
                places,
            }));
        });
    }

    // [impl->dsn~organisation-roles~1]
    /// Answers `getOrganisationRoles` and `setOrganisationRole`: the list for
    /// administrators (`manage_roles`), who may also change it; nothing for others.
    async fn organisation_roles(
        &self,
        session: SessionId,
        message: ClientMessage,
    ) -> Result<(), StoreError> {
        let (user, version, may) = {
            let hub = self.hub.lock().await;
            let Some(user) = hub.session_user(session) else {
                return Ok(()); // not authenticated: nothing
            };
            let may = hub.may_manage_roles(&user);
            (user, hub.session_version(session), may)
        };
        if version.is_none_or(|v| v < ordo_sync::version::SYNC_MAP_REGION) {
            return Ok(());
        }
        let mut changed = None;
        if may
            && let ClientMessage::SetOrganisationRole(change) = message
            && message::ORGANISATION_ROLES.contains(&change.role.as_str())
            // Nobody withdraws their own administrator role: an installation keeps one.
            && !(change.user == user && change.role == "admin")
        {
            let done = if change.grant {
                let until = change.valid_until.filter(|&u| u > now_ms());
                let granted = self.roles.grant(&change.user, &change.role, until);
                if granted {
                    self.store
                        .grant_organisation_role(&change.user, &change.role, until, &user, now_ms())
                        .await?;
                }
                granted
            } else {
                let withdrawn = self.roles.withdraw(&change.user, &change.role);
                if withdrawn {
                    self.store
                        .withdraw_organisation_role(&change.user, &change.role)
                        .await?;
                }
                withdrawn
            };
            if done {
                tracing::info!(%user, person = %change.user, role = change.role, grant = change.grant, "organisation role changed");
                changed = Some(change.user);
            }
        }
        let list = if may { self.roles.list() } else { Vec::new() };
        if let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
        {
            let _ = sender.send(ServerMessage::OrganisationRoles(list));
        }
        if let Some(person) = changed {
            let out = self.hub.lock().await.organisation_roles_changed(&[person]);
            self.send_refusals(out);
        }
        Ok(())
    }

    // [impl->dsn~local-accounts~1]
    /// Answers `getAccounts` and `changeAccount`: the local accounts in Dex for
    /// administrators (`manage_accounts`), who may also change them.
    async fn accounts(
        self: &Arc<Self>,
        session: SessionId,
        message: ClientMessage,
    ) -> Result<(), StoreError> {
        let (user, version, may) = {
            let hub = self.hub.lock().await;
            let Some(user) = hub.session_user(session) else {
                return Ok(()); // not authenticated: nothing
            };
            let may = hub.may_manage_accounts(&user);
            (user, hub.session_version(session), may)
        };
        if version.is_none_or(|v| v < ordo_sync::version::SYNC_MAP_REGION) {
            return Ok(());
        }
        let answer = match (&self.dex_api, may) {
            (Some(api), true) => {
                let error = match message {
                    ClientMessage::ChangeAccount(change) => {
                        self.change_account(api, &user, change).await.err()
                    }
                    _ => None,
                };
                match api.list().await {
                    Ok(list) => message::Accounts {
                        managed: true,
                        list: list
                            .into_iter()
                            .map(|p| message::Account {
                                email: p.email,
                                name: p.username,
                            })
                            .collect(),
                        error,
                    },
                    Err(e) => {
                        tracing::warn!(error = %e, "listing local accounts failed");
                        message::Accounts {
                            managed: true,
                            list: Vec::new(),
                            error: Some(message::AccountError::Unreachable),
                        }
                    }
                }
            }
            _ => message::Accounts::default(),
        };
        if let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
        {
            let _ = sender.send(ServerMessage::Accounts(answer));
        }
        Ok(())
    }

    // [impl->dsn~login-providers~1]
    /// Answers the login provider messages (ORG-11, ADR-0063): the providers, and changes,
    /// for users with `manage_accounts` on a server with Dex's API; nothing for others.
    async fn login_providers(
        self: &Arc<Self>,
        session: SessionId,
        message: ClientMessage,
    ) -> Result<(), StoreError> {
        let (user, version, may) = {
            let hub = self.hub.lock().await;
            let Some(user) = hub.session_user(session) else {
                return Ok(()); // not authenticated: nothing
            };
            let may = hub.may_manage_accounts(&user);
            (user, hub.session_version(session), may)
        };
        if version.is_none_or(|v| v < ordo_sync::version::SYNC_MAP_REGION) {
            return Ok(());
        }
        let answer = match (&self.dex_api, &self.login, may) {
            (Some(api), Some(login), true) => {
                let redirect_uri = format!("{}/callback", login.issuer().trim_end_matches('/'));
                let error = match message {
                    ClientMessage::TestLoginProvider(draft) => self
                        .test_provider(api, &user, draft, &redirect_uri)
                        .await
                        .err(),
                    ClientMessage::SaveLoginProvider(token) => {
                        self.save_provider(api, &user, &token).await.err()
                    }
                    ClientMessage::DeleteLoginProvider(id) => {
                        self.delete_provider(api, &user, &id).await.err()
                    }
                    _ => None,
                };
                match api.connectors().await {
                    Ok(list) => message::LoginProviders {
                        managed: true,
                        redirect_uri: Some(redirect_uri),
                        list: list
                            .iter()
                            .filter_map(crate::providers::provider_of)
                            .collect(),
                        test: self.login_tests.of(&user),
                        error,
                    },
                    Err(e) => message::LoginProviders {
                        managed: true,
                        redirect_uri: Some(redirect_uri),
                        error: Some(provider_error(&e)),
                        ..message::LoginProviders::default()
                    },
                }
            }
            _ => message::LoginProviders::default(),
        };
        if let Some(sender) = self
            .sessions
            .lock()
            .expect("session registry lock")
            .get(&session)
        {
            let _ = sender.send(ServerMessage::LoginProviders(answer));
        }
        Ok(())
    }

    /// Writes a temporary connector for a test login through a new or changed provider.
    async fn test_provider(
        &self,
        api: &crate::dex_api::DexApi,
        user: &UserId,
        draft: message::LoginProviderDraft,
        redirect_uri: &str,
    ) -> Result<(), message::LoginProviderError> {
        use crate::providers::{NewTest, TEST_PREFIX, changeable, issuer, oidc_config};
        use message::LoginProviderError as E;
        let name = crate::login::tidy_name(Some(&draft.name), 60).ok_or(E::Invalid)?;
        let issuer = issuer(&draft.issuer).ok_or(E::Invalid)?;
        let client_id = draft.client_id.trim().to_owned();
        if client_id.is_empty() {
            return Err(E::Invalid);
        }
        let target = draft.id.filter(|id| changeable(id));
        let secret = match draft.client_secret.filter(|s| !s.trim().is_empty()) {
            Some(secret) => secret.trim().to_owned(),
            // Changing without a new secret keeps the old one, which stays in Dex.
            None => {
                let target = target.as_deref().ok_or(E::Invalid)?;
                let list = api.connectors().await.map_err(|e| provider_error(&e))?;
                list.iter()
                    .find(|c| c.id == target)
                    .and_then(crate::providers::secret_of)
                    .ok_or(E::NotFound)?
            }
        };
        let token = random_hex(32);
        let connector = format!("{TEST_PREFIX}{}", random_hex(8));
        let config = oidc_config(
            &issuer,
            &client_id,
            &secret,
            redirect_uri,
            draft.skip_email_verified,
        );
        api.create_connector(crate::dex_api::Connector {
            id: connector.clone(),
            r#type: "oidc".to_owned(),
            name: format!("Test: {name}"),
            config: config.clone(),
            grant_types: Vec::new(),
        })
        .await
        .map_err(|e| provider_error(&e))?;
        tracing::info!(%user, provider = name, %issuer, "test login through a provider started");
        let earlier = self.login_tests.insert(
            token,
            NewTest {
                connector,
                target,
                name,
                config,
                by: user.clone(),
            },
        );
        for id in earlier {
            let _ = api.delete_connector(&id).await;
        }
        Ok(())
    }

    /// Saves the provider of a passed test login and removes the temporary connector.
    async fn save_provider(
        &self,
        api: &crate::dex_api::DexApi,
        user: &UserId,
        token: &str,
    ) -> Result<(), message::LoginProviderError> {
        use crate::dex_api::{Connector, Outcome};
        use message::LoginProviderError as E;
        let test = self
            .login_tests
            .take_passed(token, user)
            .ok_or(E::NotTested)?;
        let list = api.connectors().await.map_err(|e| provider_error(&e))?;
        let ids: Vec<String> = list.iter().map(|c| c.id.clone()).collect();
        let (id, exists) = match &test.target {
            Some(id) if ids.contains(id) => (id.clone(), true),
            _ => (crate::providers::id_for(&test.name, &ids), false),
        };
        let connector = Connector {
            id: id.clone(),
            r#type: "oidc".to_owned(),
            name: test.name.clone(),
            config: test.config.clone(),
            grant_types: Vec::new(),
        };
        let outcome = if exists {
            api.update_connector(connector).await
        } else {
            api.create_connector(connector).await
        }
        .map_err(|e| provider_error(&e))?;
        let _ = api.delete_connector(&test.connector).await;
        if outcome != Outcome::Done {
            return Err(E::NotFound);
        }
        // Every change of who may log in is logged (ADR-0063).
        tracing::warn!(%user, provider = id, name = test.name, "login provider saved");
        Ok(())
    }

    /// Deletes a provider.
    async fn delete_provider(
        &self,
        api: &crate::dex_api::DexApi,
        user: &UserId,
        id: &str,
    ) -> Result<(), message::LoginProviderError> {
        use message::LoginProviderError as E;
        if !crate::providers::changeable(id) {
            return Err(E::NotFound);
        }
        match api
            .delete_connector(id)
            .await
            .map_err(|e| provider_error(&e))?
        {
            crate::dex_api::Outcome::Done => {
                tracing::warn!(%user, provider = id, "login provider deleted");
                Ok(())
            }
            _ => Err(E::NotFound),
        }
    }

    /// Deletes temporary connectors of expired tests, and at the first run those left from
    /// before the start.
    async fn clean_login_tests(&self) {
        let Some(api) = &self.dex_api else { return };
        let first = !self.tests_cleaned.swap(true, Ordering::SeqCst);
        let (live, expired) = self.login_tests.expire();
        if !first && !expired {
            return;
        }
        let Ok(list) = api.connectors().await else {
            return;
        };
        for c in list
            .iter()
            .filter(|c| c.id.starts_with(crate::providers::TEST_PREFIX) && !live.contains(&c.id))
        {
            if api.delete_connector(&c.id).await.is_ok() {
                tracing::info!(connector = c.id, "temporary login connector removed");
            }
        }
    }

    /// Makes one change to a local account in Dex.
    async fn change_account(
        self: &Arc<Self>,
        api: &crate::dex_api::DexApi,
        user: &UserId,
        change: message::AccountChange,
    ) -> Result<(), message::AccountError> {
        use crate::dex_api::{DexApiError, Outcome};
        use message::{AccountChange, AccountError};
        let failed = |e: DexApiError| {
            tracing::warn!(error = %e, "changing a local account failed");
            match e {
                DexApiError::Refused(_) => AccountError::Refused,
                DexApiError::Unreachable(_) | DexApiError::Config(_) => AccountError::Unreachable,
            }
        };
        let outcome = match change {
            AccountChange::Create {
                email,
                name,
                password,
            } => {
                let email = email.trim().to_lowercase();
                let name = crate::login::tidy_name(Some(&name), crate::login::MAX_PERSON_NAME)
                    .filter(|_| is_email(&email))
                    .ok_or(AccountError::Invalid)?;
                let hash = hash_password(password).await?;
                let outcome = api.create(&email, &name, hash).await.map_err(failed)?;
                tracing::info!(%user, account = email, "local account created");
                if outcome == Outcome::Done {
                    self.names_from_accounts().await;
                }
                outcome
            }
            AccountChange::SetPassword { email, password } => {
                let hash = hash_password(password).await?;
                let account = find_account(api, &email)
                    .await
                    .map_err(|e| e.map_or(AccountError::NotFound, failed))?;
                let outcome = api.set_password(&account, hash).await.map_err(failed)?;
                tracing::info!(%user, account = account.email, "local account's password set");
                outcome
            }
            AccountChange::Delete { email } => {
                let account = find_account(api, &email)
                    .await
                    .map_err(|e| e.map_or(AccountError::NotFound, failed))?;
                let client_id = self.login.as_ref().map_or("ordo", |l| l.client_id());
                let outcome = api.delete(&account, client_id).await.map_err(failed)?;
                tracing::info!(%user, account = account.email, "local account deleted");
                if outcome == Outcome::Done {
                    self.end_login_of(&account.user_id).await;
                }
                outcome
            }
        };
        match outcome {
            Outcome::Done => Ok(()),
            Outcome::Exists => Err(AccountError::Exists),
            Outcome::NotFound => Err(AccountError::NotFound),
        }
    }

    /// Ends the ORDO login of a deleted local account's person, if they ever logged in:
    /// their devices lock at once (ADR-0040, ADR-0060).
    async fn end_login_of(&self, dex_user_id: &str) {
        let Some(login) = &self.login else { return };
        let subject = crate::dex_api::local_subject(dex_user_id);
        let ended = async {
            if let Some(person) = self
                .store
                .user_of_identity(login.issuer(), &subject)
                .await?
            {
                self.store
                    .set_refresh(login.issuer(), &subject, None, &self.master)
                    .await?;
                self.set_valid_until(&person, now_ms()).await?;
                tracing::info!(%person, "login ended: local account deleted");
            }
            Ok::<(), StoreError>(())
        };
        if let Err(error) = ended.await {
            tracing::error!(%error, "ending the login of a deleted account failed");
        }
    }

    /// Answers `getAvatars` and `setAvatar` of an authenticated connection.
    async fn avatars(&self, session: SessionId, message: ClientMessage) -> Result<(), StoreError> {
        let Some(user) = self.hub.lock().await.session_user(session) else {
            return Ok(()); // not authenticated: nothing
        };
        let (users, changed) = match message {
            ClientMessage::GetAvatars(mut users) => {
                users.truncate(200);
                let mut asked = self.asked.lock().expect("asked registry lock");
                let seen = asked.entry(session).or_default();
                if seen.len() < MAX_ASKED {
                    seen.extend(users.iter().cloned());
                }
                (users, false)
            }
            ClientMessage::SetAvatar(set) => {
                let image = set.image.filter(|i| ordo_sync::message::is_avatar_image(i));
                self.store
                    .set_avatar(&user, AvatarSource::Ordo, image.as_deref(), now_ms())
                    .await?;
                (vec![user], true)
            }
            // [impl->dsn~display-names~2]
            ClientMessage::SetName(name) => {
                let name = crate::login::tidy_name(name.as_deref(), crate::login::MAX_PERSON_NAME);
                self.store.set_chosen_name(&user, name.as_deref()).await?;
                (vec![user], true)
            }
            _ => return Ok(()),
        };
        let mut avatars = Vec::new();
        for user in users {
            avatars.push(self.person(user).await?);
        }
        if changed && let Some(avatar) = avatars.first() {
            self.tell_askers(avatar, Some(session));
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

    /// A person's picture and display name.
    async fn person(&self, user: UserId) -> Result<Avatar, StoreError> {
        let image = self.store.avatar(&user).await?;
        let name = self.store.display_name(&user).await?;
        Ok(Avatar { user, image, name })
    }

    // [impl->dsn~display-names~2]
    /// After `user`'s name or picture changed elsewhere (a login): tells the connections
    /// that asked about them.
    pub(crate) async fn person_changed(&self, user: &UserId) -> Result<(), StoreError> {
        let avatar = self.person(user.clone()).await?;
        self.tell_askers(&avatar, None);
        Ok(())
    }

    /// Sends the changed `avatar` to every connection that asked about the person, except
    /// `except` (it gets its answer anyway).
    fn tell_askers(&self, avatar: &Avatar, except: Option<SessionId>) {
        let sessions = self.sessions.lock().expect("session registry lock");
        let mut asked = self.asked.lock().expect("asked registry lock");
        asked.retain(|session, _| sessions.contains_key(session));
        for (session, users) in asked.iter() {
            if Some(*session) != except
                && users.contains(&avatar.user)
                && let Some(sender) = sessions.get(session)
            {
                let _ = sender.send(ServerMessage::Avatars(vec![avatar.clone()]));
            }
        }
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
        self.asked.lock().expect("asked registry lock").remove(&id);
        self.hub.lock().await.disconnect(id);
        self.metrics.sessions.fetch_sub(1, Ordering::Relaxed);
        let _ = writer.await;
        tracing::info!(session = id, "disconnected");
    }
}

// [impl->dsn~content-security-policy~1]
/// The Content Security Policy of every response: scripts, styles, workers and
/// connections only from ORDO itself; WebAssembly may compile; the app page's inline
/// scripts (Trunk's loader) only by their hash, read from the built page at start; images
/// also as `data:` and `blob:` URLs (symbols, photos). No inline styles, no eval.
fn content_security_policy(app_dir: Option<&std::path::Path>) -> String {
    let hashes = app_dir
        .and_then(|dir| std::fs::read_to_string(dir.join("index.html")).ok())
        .map(|html| inline_script_hashes(&html))
        .unwrap_or_default();
    let scripts: String = hashes.iter().map(|h| format!(" '{h}'")).collect();
    format!(
        "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'{scripts}; \
         style-src 'self'; img-src 'self' data: blob:; font-src 'self'; connect-src 'self'; \
         worker-src 'self'; manifest-src 'self'; frame-ancestors 'none'; object-src 'none'; \
         base-uri 'self'; form-action 'self'"
    )
}

/// `sha256-…` of every inline `<script>` (one without `src`) of a page.
fn inline_script_hashes(html: &str) -> Vec<String> {
    use base64::Engine as _;
    use sha2::{Digest, Sha256};
    let mut hashes = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<script") {
        let after = &rest[start..];
        let Some(open_end) = after.find('>') else {
            break;
        };
        let tag = &after[..open_end];
        let body = &after[open_end + 1..];
        let Some(close) = body.find("</script>") else {
            break;
        };
        if !tag.contains(" src=") {
            let digest = Sha256::digest(&body.as_bytes()[..close]);
            hashes.push(format!(
                "sha256-{}",
                base64::engine::general_purpose::STANDARD.encode(digest)
            ));
        }
        rest = &body[close..];
    }
    hashes
}

/// Largest WebSocket message: a push carries at most 4 MiB of events (protocol §2), one
/// event at most 1 MiB; more is refused before it is read into memory (SEC-17).
const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;

async fn sync(State(app): State<Arc<App>>, upgrade: WebSocketUpgrade) -> Response {
    upgrade
        .max_message_size(MAX_MESSAGE_BYTES)
        .max_frame_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| app.session(socket))
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
/// A plausible e-mail address: one `@` with text on both sides, no spaces.
fn is_email(text: &str) -> bool {
    text.len() <= 254
        && !text.chars().any(char::is_whitespace)
        && text.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !domain.contains('@')
        })
}

/// The bcrypt hash of a password for Dex, if it fits the limits (ORG-10).
async fn hash_password(password: String) -> Result<Vec<u8>, message::AccountError> {
    if !message::password_fits(&password) {
        return Err(message::AccountError::PasswordLength);
    }
    // bcrypt takes a moment of CPU on purpose.
    tokio::task::spawn_blocking(move || crate::dex_api::hash_password(&password))
        .await
        .ok()
        .and_then(Result::ok)
        .ok_or(message::AccountError::Unreachable)
}

/// The account with `email` (`Err(None)`: no such account).
async fn find_account(
    api: &crate::dex_api::DexApi,
    email: &str,
) -> Result<crate::dex_api::Password, Option<crate::dex_api::DexApiError>> {
    api.list()
        .await
        .map_err(Some)?
        .into_iter()
        .find(|p| p.email.eq_ignore_ascii_case(email.trim()))
        .ok_or(None)
}

/// The most people one connection is told about when they change.
const MAX_ASKED: usize = 2000;

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
/// `/.well-known/ordo`: the versions of each specification this server supports, and the
/// server's own build (shown under *Administration – Info*).
async fn well_known() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "sync": ordo_sync::version::SYNC,
        "http": HTTP,
        "build": {
            "version": env!("CARGO_PKG_VERSION"),
            "commit": option_env!("ORDO_COMMIT").unwrap_or_default(),
        },
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

/// What the app is told when Dex's connector API fails.
fn provider_error(error: &crate::dex_api::DexApiError) -> message::LoginProviderError {
    use crate::dex_api::DexApiError;
    tracing::warn!(%error, "a call to Dex's connector API failed");
    match error {
        DexApiError::Refused(_) => message::LoginProviderError::Refused,
        DexApiError::Unreachable(_) | DexApiError::Config(_) => {
            message::LoginProviderError::Unreachable
        }
    }
}

/// `n` random bytes, hex.
fn random_hex(n: usize) -> String {
    let mut bytes = vec![0u8; n];
    getrandom::fill(&mut bytes).expect("the operating system provides randomness");
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod cache_tests {
    // [utest->dsn~content-security-policy~1]
    #[test]
    fn the_policy_allows_exactly_the_pages_inline_scripts() {
        let page = r#"<head><script src="/boot.js"></script>
<script type="module">import init from '/app.js';</script></head>"#;
        let hashes = super::inline_script_hashes(page);
        assert_eq!(hashes.len(), 1, "files are allowed by 'self', not by hash");
        // echo -n "import init from '/app.js';" | openssl dgst -sha256 -binary | base64
        assert_eq!(
            hashes[0],
            "sha256-lIu2QzD3kpYH4IKFiA/ZCfzcckv8zd+3oxfRlLTdixk="
        );
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), page).unwrap();
        let policy = super::content_security_policy(Some(dir.path()));
        assert!(policy.contains(&format!(
            "script-src 'self' 'wasm-unsafe-eval' '{}'",
            hashes[0]
        )));
        for directive in [
            "style-src 'self';",
            "connect-src 'self';",
            "object-src 'none';",
            "frame-ancestors 'none';",
        ] {
            assert!(policy.contains(directive), "{directive}");
        }
        assert!(!policy.contains("unsafe-inline") && !policy.contains("'unsafe-eval'"));
    }

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
