//! Login with OpenID Connect and device pairing (ADR-0038).
//!
//! The ORDO server is the relying party (backend for frontend): the browser never sees a
//! token. A login registers the public key of the device it was started on; from then on
//! the device proves that key on every connection (sync protocol draft 1) and works
//! without the identity provider, also offline.
//!
//! * `GET /auth/login?device=…&key=…`: authorization code flow with PKCE; a cookie binds
//!   the answer to the browser that started it (no login CSRF).
//! * `POST /auth/device`: OAuth 2.0 Device Authorization Grant (RFC 8628) for shared
//!   tablets: the device shows a QR code, a logged-in person approves on their phone.
//!
//! People get roles from the identity provider's `groups` claim, added to roles from the
//! seed file; Cedar decides what roles may do (ADR-0035).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Redirect, Response};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreAuthenticationFlow, CoreClaimName, CoreClaimType,
    CoreClientAuthMethod, CoreDeviceAuthorizationResponse, CoreErrorResponseType, CoreGenderClaim,
    CoreGrantType, CoreJsonWebKey, CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm, CoreJwsSigningAlgorithm, CoreResponseMode, CoreResponseType,
    CoreRevocableToken, CoreRevocationErrorResponse, CoreSubjectIdentifierType,
    CoreTokenIntrospectionResponse, CoreTokenType,
};
use openidconnect::{
    AdditionalClaims, AdditionalProviderMetadata, AuthType, AuthorizationCode, ClientId,
    ClientSecret, CsrfToken, DeviceAuthorizationUrl, EmptyExtraTokenFields, EndpointMaybeSet,
    EndpointNotSet, EndpointSet, IdTokenClaims, IdTokenFields, IssuerUrl, Nonce,
    OAuth2TokenResponse as _, PkceCodeChallenge, PkceCodeVerifier, ProviderMetadata, RedirectUrl,
    RefreshToken, RequestTokenError, Scope, StandardErrorResponse, StandardTokenResponse,
    TokenResponse as _, reqwest,
};
use ordo_core::{DeviceId, Ulid, UserId};
use ordo_crypto::ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::app::App;
use crate::config::OidcConfig;
use crate::store::AvatarSource;

/// How long a started login or pairing stays valid.
const PENDING_TTL: Duration = Duration::from_secs(600);
/// Most pairings by QR code waiting at the same time (each waits in a background task).
const MAX_PAIRINGS: usize = 64;
/// Cookie that binds a login to the browser that started it.
const COOKIE: &str = "ordo_login";

/// The `groups` claim of the identity provider.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
struct Groups {
    #[serde(default)]
    groups: Vec<String>,
}
impl AdditionalClaims for Groups {}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DeviceEndpoint {
    device_authorization_endpoint: DeviceAuthorizationUrl,
}
impl AdditionalProviderMetadata for DeviceEndpoint {}

type Metadata = ProviderMetadata<
    DeviceEndpoint,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;
type IdFields = IdTokenFields<
    Groups,
    EmptyExtraTokenFields,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJwsSigningAlgorithm,
>;
type Tokens = StandardTokenResponse<IdFields, CoreTokenType>;
/// The client with the endpoints in use: authorization, device authorization, token.
type OidcClient<HasDeviceAuthUrl = EndpointSet> = openidconnect::Client<
    Groups,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    Tokens,
    CoreTokenIntrospectionResponse,
    CoreRevocableToken,
    CoreRevocationErrorResponse,
    EndpointSet,
    HasDeviceAuthUrl,
    EndpointNotSet,
    EndpointNotSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;
type Claims = IdTokenClaims<Groups, CoreGenderClaim>;

struct PendingLogin {
    device: DeviceId,
    key: VerifyingKey,
    verifier: PkceCodeVerifier,
    nonce: Nonce,
    started: Instant,
}

#[derive(Clone)]
enum Status {
    Pending,
    Done(UserId),
    Failed(String),
}

struct DevicePairing {
    verification_uri: String,
    status: Status,
    started: Instant,
}

/// The login state of the server.
pub(crate) struct Login {
    config: OidcConfig,
    http: reqwest::Client,
    client: tokio::sync::Mutex<Option<OidcClient>>,
    pending: std::sync::Mutex<HashMap<String, PendingLogin>>,
    pairings: std::sync::Mutex<HashMap<String, DevicePairing>>,
}

/// Something went wrong during a login.
#[derive(Debug, thiserror::Error)]
pub(crate) enum LoginError {
    #[error("the identity provider is not reachable: {0}")]
    Provider(String),
    #[error("the login is invalid or expired")]
    Invalid,
    #[error("the login was not confirmed by the identity provider: {0}")]
    Token(String),
    #[error("storing the device failed")]
    Store(#[from] crate::store::StoreError),
    #[error("this device ID belongs to another person; reset the app on this device")]
    DeviceTaken,
}

impl Login {
    pub(crate) fn new(config: OidcConfig) -> Result<Self, LoginError> {
        let http = reqwest::ClientBuilder::new()
            // Following redirects would open the server to request forgery.
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(15))
            .build()
            .map_err(|e| LoginError::Provider(e.to_string()))?;
        Ok(Self {
            config,
            http,
            client: tokio::sync::Mutex::new(None),
            pending: std::sync::Mutex::new(HashMap::new()),
            pairings: std::sync::Mutex::new(HashMap::new()),
        })
    }

    /// The OpenID Connect client, discovered on first use, so the server also starts when
    /// the identity provider is not reachable (field servers, ADR-0038).
    async fn client(&self) -> Result<OidcClient, LoginError> {
        let mut cached = self.client.lock().await;
        if let Some(client) = cached.as_ref() {
            return Ok(client.clone());
        }
        let provider = |e: &dyn std::fmt::Display| LoginError::Provider(e.to_string());
        let issuer = IssuerUrl::new(self.config.issuer.clone()).map_err(|e| provider(&e))?;
        let metadata = Metadata::discover_async(issuer, &self.http)
            .await
            .map_err(|e| provider(&e))?;
        let device_url = metadata
            .additional_metadata()
            .device_authorization_endpoint
            .clone();
        let redirect = RedirectUrl::new(format!("{}/auth/callback", self.config.public_url))
            .map_err(|e| provider(&e))?;
        let client = OidcClient::<EndpointNotSet>::from_provider_metadata(
            metadata,
            ClientId::new(self.config.client_id.clone()),
            Some(ClientSecret::new(self.config.client_secret.clone())),
        )
        .set_device_authorization_url(device_url)
        .set_redirect_uri(redirect)
        // Client ID and secret in the request body (client_secret_post): some providers,
        // e.g. Dex, read the client ID of device requests only from there.
        .set_auth_type(AuthType::RequestBody);
        *cached = Some(client.clone());
        Ok(client)
    }

    /// Forgets the discovered provider, e.g. after its signing keys changed.
    async fn forget_client(&self) {
        *self.client.lock().await = None;
    }

    fn secure_cookie(&self) -> &'static str {
        if self.config.public_url.starts_with("https://") {
            "; Secure"
        } else {
            ""
        }
    }
}

/// `offline_access` asks for a renewal token, so the server can re-verify people in the
/// background (ADR-0040).
const SCOPES: [&str; 4] = ["profile", "email", "groups", "offline_access"];

/// The ORDO user name for a person: their preferred user name (or the local part of the
/// e-mail address), in the characters ORDO user IDs allow.
pub(crate) fn user_name(preferred: Option<&str>, email: Option<&str>) -> String {
    let source = preferred
        .filter(|p| !p.trim().is_empty())
        .or_else(|| email.and_then(|e| e.split('@').next()))
        .unwrap_or("user");
    let mut name = String::new();
    for c in source.trim().chars().flat_map(char::to_lowercase) {
        match c {
            'a'..='z' | '0'..='9' | '.' | '_' | '-' => name.push(c),
            'ä' => name.push_str("ae"),
            'ö' => name.push_str("oe"),
            'ü' => name.push_str("ue"),
            'ß' => name.push_str("ss"),
            _ => name.push('-'),
        }
    }
    let name: String = name.trim_matches(['-', '.']).chars().take(56).collect();
    if name.is_empty() {
        "user".to_owned()
    } else {
        name
    }
}

/// A device ID and public key from a request.
fn parse_device(device: &str, key: &str) -> Option<(DeviceId, VerifyingKey)> {
    let device = DeviceId(device.parse::<Ulid>().ok()?);
    let bytes = (key.len() == 64)
        .then(|| {
            (0..32)
                .map(|i| u8::from_str_radix(&key[2 * i..2 * i + 2], 16).ok())
                .collect::<Option<Vec<u8>>>()
        })
        .flatten()?;
    let key = VerifyingKey::from_bytes(&bytes.try_into().ok()?).ok()?;
    Some((device, key))
}

/// Fetches a profile picture: `Ok(None)` if there is none (HTTP 404), `Err` if it could
/// not be fetched or is no acceptable image.
async fn fetch_avatar(http: &reqwest::Client, url: &str) -> Result<Option<Vec<u8>>, ()> {
    let mut response = http.get(url).send().await.map_err(|_| ())?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        return Err(());
    }
    let mut image = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| ())? {
        image.extend_from_slice(&chunk);
        if image.len() > ordo_sync::message::MAX_AVATAR_BYTES {
            return Err(());
        }
    }
    if ordo_sync::message::is_avatar_image(&image) {
        Ok(Some(image))
    } else {
        Err(())
    }
}

/// The Gravatar address of an e-mail address (SHA-256 of the trimmed, lower-case address;
/// "404" when there is no picture).
fn gravatar_url(email: &str) -> String {
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(email.trim().to_lowercase().as_bytes());
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("https://gravatar.com/avatar/{hex}?s=256&d=404")
}

/// Refreshes the pictures from the identity provider and from Gravatar (ADR-0047). A
/// source that has no picture any more is removed; one that cannot be reached keeps the
/// stored picture.
async fn refresh_avatars(
    app: Arc<App>,
    user: UserId,
    picture: Option<String>,
    email: Option<String>,
) {
    let Ok(http) = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
    else {
        return;
    };
    let now = crate::app::now_ms();
    let provider = match picture.filter(|u| u.starts_with("https://")) {
        Some(url) => fetch_avatar(&http, &url).await,
        None => Ok(None),
    };
    if let Ok(image) = provider {
        let _ = app
            .store
            .set_avatar(&user, AvatarSource::Provider, image.as_deref(), now)
            .await;
    }
    if app.gravatar
        && let Some(email) = email
        && let Ok(image) = fetch_avatar(&http, &gravatar_url(&email)).await
    {
        let _ = app
            .store
            .set_avatar(&user, AvatarSource::Gravatar, image.as_deref(), now)
            .await;
    }
}

// [impl->dsn~oidc-login~1]
/// Registers the device of a person who logged in: user mapping, roles, device key.
async fn finish(
    app: &Arc<App>,
    claims: &Claims,
    renewal: Option<&RefreshToken>,
    device: DeviceId,
    key: VerifyingKey,
) -> Result<UserId, LoginError> {
    // Identity only: roles are granted per incident in ORDO (ADR-0046).
    let wanted = user_name(
        claims.preferred_username().map(|u| u.as_str()),
        claims.email().map(|e| e.as_str()),
    );
    let user = app
        .store
        .identity(
            claims.issuer().as_str(),
            claims.subject().as_str(),
            &wanted,
            &app.server,
            &[],
        )
        .await?;
    // Profile pictures from the provider and Gravatar, in the background: a field server
    // without internet must not slow the login down (ADR-0047).
    tokio::spawn(refresh_avatars(
        app.clone(),
        user.clone(),
        claims
            .picture()
            .and_then(|p| p.get(None))
            .map(|u| u.as_str().to_owned()),
        claims.email().map(|e| e.as_str().to_owned()),
    ));
    if let Some(renewal) = renewal {
        app.store
            .set_refresh(
                claims.issuer().as_str(),
                claims.subject().as_str(),
                Some(renewal.secret()),
                &app.master,
            )
            .await?;
    }
    if !app.register_device(device, user.clone(), key).await? {
        return Err(LoginError::DeviceTaken);
    }
    tracing::info!(user = %user, device = %device.ulid(), "device paired by login");
    Ok(user)
}

/// What a background renewal found out.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Renewal {
    /// The identity provider confirmed the person; the login is extended.
    Confirmed,
    /// The identity provider definitively refused (`invalid_grant`); access ends now.
    Refused,
    /// No answer that decides anything (unreachable, generic errors): nothing changes;
    /// the timeout still applies (ADR-0040).
    Undecided,
}

/// Classifies a failed renewal: only `invalid_grant` is a definitive refusal. Dex, for
/// example, reports an unreachable upstream provider and an upstream refusal with the same
/// generic error, so those stay undecided.
fn classify<RE: std::error::Error + 'static>(
    error: &RequestTokenError<RE, StandardErrorResponse<CoreErrorResponseType>>,
) -> Renewal {
    match error {
        RequestTokenError::ServerResponse(response)
            if *response.error() == CoreErrorResponseType::InvalidGrant =>
        {
            Renewal::Refused
        }
        _ => Renewal::Undecided,
    }
}

// [impl->dsn~access-timeout~1]
/// Re-verifies everyone who logged in, with the identity provider's renewal tokens: a
/// confirmation extends the login of all their devices and updates their roles; a
/// definitive refusal ends it at once; anything else changes nothing (ADR-0040).
pub(crate) async fn reverify(app: &App) {
    let Some(login) = app.login.as_ref() else {
        return;
    };
    let people = match app.store.load_refresh(&app.master).await {
        Ok(people) => people,
        Err(error) => {
            tracing::error!(%error, "reading renewal tokens failed");
            return;
        }
    };
    if people.is_empty() {
        return;
    }
    let client = match login.client().await {
        Ok(client) => client,
        Err(error) => {
            tracing::info!(%error, "identity provider not reachable; logins stay as they are");
            return;
        }
    };
    for (issuer, subject, user, token) in people {
        let token = RefreshToken::new(token);
        let request = match client.exchange_refresh_token(&token) {
            Ok(request) => request,
            Err(error) => {
                tracing::error!(%error, "renewal not possible");
                return;
            }
        };
        let outcome = match request.request_async(&login.http).await {
            Ok(tokens) => {
                let no_nonce = |_: Option<&Nonce>| Ok::<(), String>(());
                // The ID token is checked; roles do not come from it (ADR-0046).
                let _ = tokens
                    .id_token()
                    .and_then(|t| t.claims(&client.id_token_verifier(), no_nonce).ok());
                let renewed = tokens.refresh_token().map(|t| t.secret().as_str());
                let stored = renewed.unwrap_or(token.secret().as_str());
                if let Err(error) = app
                    .store
                    .set_refresh(&issuer, &subject, Some(stored), &app.master)
                    .await
                {
                    tracing::error!(%error, "storing the renewal token failed");
                }
                Renewal::Confirmed
            }
            Err(error) => classify(&error),
        };
        let result = match outcome {
            Renewal::Confirmed => app.set_valid_until(&user, app.valid_from_now()).await,
            Renewal::Refused => {
                tracing::warn!(user = %user, "identity provider refused the renewal; access ends");
                let _ = app
                    .store
                    .set_refresh(&issuer, &subject, None, &app.master)
                    .await;
                app.set_valid_until(&user, crate::app::now_ms()).await
            }
            Renewal::Undecided => {
                tracing::info!(user = %user, "renewal undecided; login stays until its end");
                Ok(())
            }
        };
        if let Err(error) = result {
            tracing::error!(%error, "storing the login state failed");
        }
    }
}

/// `GET /auth/config`: whether login is available, and the server name (for new incidents).
pub(crate) async fn config(State(app): State<Arc<App>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "server": app.server.to_string(),
        "login": app.login.is_some(),
    }))
}

#[derive(Deserialize)]
pub(crate) struct LoginQuery {
    device: String,
    key: String,
}

/// `GET /auth/login`: starts the login for the device that sends its public key.
pub(crate) async fn start(State(app): State<Arc<App>>, Query(q): Query<LoginQuery>) -> Response {
    let Some(login) = app.login.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some((device, key)) = parse_device(&q.device, &q.key) else {
        return (StatusCode::BAD_REQUEST, "invalid device or key").into_response();
    };
    let client = match login.client().await {
        Ok(client) => client,
        Err(e) => return failed(&e),
    };
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    let mut request = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    );
    for scope in SCOPES {
        request = request.add_scope(Scope::new(scope.to_owned()));
    }
    let (url, state, nonce) = request.set_pkce_challenge(challenge).url();
    {
        let mut pending = login.pending.lock().expect("pending logins");
        pending.retain(|_, p| p.started.elapsed() < PENDING_TTL);
        pending.insert(
            state.secret().clone(),
            PendingLogin {
                device,
                key,
                verifier,
                nonce,
                started: Instant::now(),
            },
        );
    }
    let cookie = format!(
        "{COOKIE}={}; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=600{}",
        state.secret(),
        login.secure_cookie()
    );
    ([(header::SET_COOKIE, cookie)], Redirect::to(url.as_str())).into_response()
}

#[derive(Deserialize)]
pub(crate) struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

fn cookie(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|pair| {
            pair.trim()
                .strip_prefix(&format!("{COOKIE}="))
                .map(str::to_owned)
        })
}

fn failed(error: &LoginError) -> Response {
    tracing::warn!(%error, "login failed");
    let message: String = error
        .to_string()
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect();
    Redirect::to(&format!("/?login_error={message}")).into_response()
}

/// `GET /auth/callback`: the identity provider returns; the device is registered.
pub(crate) async fn callback(
    State(app): State<Arc<App>>,
    headers: HeaderMap,
    Query(q): Query<CallbackQuery>,
) -> Response {
    let Some(login) = app.login.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let clear = format!("{COOKIE}=; Path=/auth; HttpOnly; SameSite=Lax; Max-Age=0");
    let result = async {
        if let Some(error) = q.error {
            return Err(LoginError::Token(error));
        }
        let (code, state) = q.code.zip(q.state).ok_or(LoginError::Invalid)?;
        // The answer must come back to the browser that started the login.
        if cookie(&headers).as_deref() != Some(state.as_str()) {
            return Err(LoginError::Invalid);
        }
        let pending = login
            .pending
            .lock()
            .expect("pending logins")
            .remove(&state)
            .filter(|p| p.started.elapsed() < PENDING_TTL)
            .ok_or(LoginError::Invalid)?;
        let client = login.client().await?;
        let tokens = client
            .exchange_code(AuthorizationCode::new(code))
            .map_err(|e| LoginError::Provider(e.to_string()))?
            .set_pkce_verifier(pending.verifier)
            .request_async(&login.http)
            .await
            .map_err(|e| LoginError::Token(e.to_string()))?;
        let id_token = tokens.id_token().ok_or(LoginError::Invalid)?;
        let claims = match id_token.claims(&client.id_token_verifier(), &pending.nonce) {
            Ok(claims) => claims.clone(),
            Err(e) => {
                login.forget_client().await;
                return Err(LoginError::Token(e.to_string()));
            }
        };
        finish(
            &app,
            &claims,
            tokens.refresh_token(),
            pending.device,
            pending.key,
        )
        .await
    }
    .await;
    match result {
        Ok(user) => (
            [(header::SET_COOKIE, clear)],
            Redirect::to(&format!("/?paired={user}")),
        )
            .into_response(),
        Err(e) => ([(header::SET_COOKIE, clear)], failed(&e)).into_response(),
    }
}

#[derive(Deserialize)]
pub(crate) struct DeviceRequest {
    device: String,
    key: String,
}

#[derive(Serialize)]
struct DeviceStarted {
    id: String,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: Option<String>,
    expires_in: u64,
}

// [impl->dsn~qr-pairing~1]
/// `POST /auth/device`: starts pairing a shared device by QR code (RFC 8628).
pub(crate) async fn device_start(
    State(app): State<Arc<App>>,
    Json(request): Json<DeviceRequest>,
) -> Response {
    let Some(login) = app.login.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some((device, key)) = parse_device(&request.device, &request.key) else {
        return (StatusCode::BAD_REQUEST, "invalid device or key").into_response();
    };
    {
        let mut pairings = login.pairings.lock().expect("pairings");
        pairings.retain(|_, p| p.started.elapsed() < PENDING_TTL);
        let waiting = pairings
            .values()
            .filter(|p| matches!(p.status, Status::Pending))
            .count();
        if waiting >= MAX_PAIRINGS {
            return (StatusCode::TOO_MANY_REQUESTS, "too many pairings at once").into_response();
        }
    }
    let client = match login.client().await {
        Ok(client) => client,
        Err(e) => return (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    };
    let mut ask = client.exchange_device_code();
    for scope in SCOPES {
        ask = ask.add_scope(Scope::new(scope.to_owned()));
    }
    let details: CoreDeviceAuthorizationResponse = match ask.request_async(&login.http).await {
        Ok(details) => details,
        Err(e) => return (StatusCode::BAD_GATEWAY, e.to_string()).into_response(),
    };
    let id = CsrfToken::new_random().secret().clone();
    let complete = details
        .verification_uri_complete()
        .map(|u| u.secret().clone());
    let started = DeviceStarted {
        id: id.clone(),
        user_code: details.user_code().secret().clone(),
        verification_uri: details.verification_uri().to_string(),
        verification_uri_complete: complete.clone(),
        expires_in: details.expires_in().as_secs(),
    };
    {
        let mut pairings = login.pairings.lock().expect("pairings");
        pairings.retain(|_, p| p.started.elapsed() < PENDING_TTL);
        pairings.insert(
            id.clone(),
            DevicePairing {
                verification_uri: complete.unwrap_or_else(|| started.verification_uri.clone()),
                status: Status::Pending,
                started: Instant::now(),
            },
        );
    }
    // Wait for the approval in the background; the device polls `GET /auth/device/{id}`.
    let waiting = app.clone();
    tokio::spawn(async move {
        let Some(login) = waiting.login.as_ref() else {
            return;
        };
        let result = async {
            let tokens = client
                .exchange_device_access_token(&details)
                .map_err(|e| LoginError::Provider(e.to_string()))?
                .request_async(&login.http, tokio::time::sleep, Some(PENDING_TTL))
                .await
                .map_err(|e| LoginError::Token(e.to_string()))?;
            let id_token = tokens.id_token().ok_or(LoginError::Invalid)?;
            // The device flow has no nonce; the token is bound to this device code.
            let no_nonce = |_: Option<&Nonce>| Ok::<(), String>(());
            let claims = id_token
                .claims(&client.id_token_verifier(), no_nonce)
                .map_err(|e| LoginError::Token(e.to_string()))?
                .clone();
            finish(&waiting, &claims, tokens.refresh_token(), device, key).await
        }
        .await;
        let status = match result {
            Ok(user) => Status::Done(user),
            Err(e) => {
                tracing::warn!(error = %e, "device pairing failed");
                Status::Failed(e.to_string())
            }
        };
        if let Some(p) = login.pairings.lock().expect("pairings").get_mut(&id) {
            p.status = status;
        }
    });
    Json(started).into_response()
}

/// `GET /auth/device/{id}`: the state of a pairing.
pub(crate) async fn device_status(State(app): State<Arc<App>>, Path(id): Path<String>) -> Response {
    let Some(login) = app.login.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let status = login
        .pairings
        .lock()
        .expect("pairings")
        .get(&id)
        .map(|p| p.status.clone());
    let body = match status {
        None => return StatusCode::NOT_FOUND.into_response(),
        Some(Status::Pending) => serde_json::json!({ "status": "pending" }),
        Some(Status::Done(user)) => {
            serde_json::json!({ "status": "done", "user": user.to_string() })
        }
        Some(Status::Failed(error)) => serde_json::json!({ "status": "failed", "error": error }),
    };
    Json(body).into_response()
}

/// `GET /auth/device/{id}/qr.svg`: the QR code to scan with a phone.
pub(crate) async fn device_qr(State(app): State<Arc<App>>, Path(id): Path<String>) -> Response {
    let Some(login) = app.login.as_ref() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let uri = login
        .pairings
        .lock()
        .expect("pairings")
        .get(&id)
        .map(|p| p.verification_uri.clone());
    let Some(uri) = uri else {
        return StatusCode::NOT_FOUND.into_response();
    };
    match qrcode::QrCode::new(uri.as_bytes()) {
        Ok(code) => {
            let svg = code
                .render::<qrcode::render::svg::Color<'_>>()
                .min_dimensions(256, 256)
                .quiet_zone(true)
                .build();
            ([(header::CONTENT_TYPE, "image/svg+xml")], svg).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gravatar_uses_the_sha256_of_the_normalised_address() {
        // Example from Gravatar's documentation of SHA-256 hashes.
        assert_eq!(
            gravatar_url(" MyEmailAddress@example.com "),
            "https://gravatar.com/avatar/84059b07d4be67b806386c0aad8070a23f18836bbaae342275dc0a83414c32ee?s=256&d=404"
        );
    }

    #[test]
    fn user_names_fit_ordo_user_ids() {
        assert_eq!(user_name(Some("Anna"), None), "anna");
        assert_eq!(user_name(Some("Jörg Müller"), None), "joerg-mueller");
        assert_eq!(
            user_name(None, Some("max.mustermann@drk.example")),
            "max.mustermann"
        );
        assert_eq!(user_name(Some("  "), None), "user");
        assert_eq!(user_name(Some("-@-"), None), "user");
        let long = "a".repeat(100);
        assert_eq!(user_name(Some(&long), None).len(), 56);
        let server: ordo_core::ServerName = "ordo.example.org".parse().unwrap();
        for name in ["Jörg Müller", "x/y\\z", "ÆØÅ", "a..b"] {
            assert!(
                UserId::new(&user_name(Some(name), None), &server).is_ok(),
                "{name}"
            );
        }
    }

    #[test]
    fn device_and_key_are_checked() {
        let key = VerifyingKey::from_bytes(&[
            0xd7, 0x5a, 0x98, 0x01, 0x82, 0xb1, 0x0a, 0xb7, 0xd5, 0x4b, 0xfe, 0xd3, 0xc9, 0x64,
            0x07, 0x3a, 0x0e, 0xe1, 0x72, 0xf3, 0xda, 0xa6, 0x23, 0x25, 0xaf, 0x02, 0x1a, 0x68,
            0xf7, 0x07, 0x51, 0x1a,
        ])
        .unwrap();
        let hex: String = key.as_bytes().iter().map(|b| format!("{b:02x}")).collect();
        let device = Ulid::from_parts(1, 1).unwrap().to_string();
        assert!(parse_device(&device, &hex).is_some());
        assert!(parse_device("not-a-ulid", &hex).is_none());
        assert!(parse_device(&device, "abc").is_none());
    }

    // [utest->dsn~access-timeout~1]
    #[test]
    fn only_invalid_grant_ends_access() {
        type Error =
            RequestTokenError<std::io::Error, StandardErrorResponse<CoreErrorResponseType>>;
        let response = |kind| StandardErrorResponse::new(kind, None, None);
        let refused: Error =
            RequestTokenError::ServerResponse(response(CoreErrorResponseType::InvalidGrant));
        assert_eq!(classify(&refused), Renewal::Refused);
        // Dex answers an unreachable upstream with a generic invalid_request (HTTP 500).
        let generic: Error =
            RequestTokenError::ServerResponse(response(CoreErrorResponseType::InvalidRequest));
        assert_eq!(classify(&generic), Renewal::Undecided);
        let offline: Error = RequestTokenError::Request(std::io::Error::other("unreachable"));
        assert_eq!(classify(&offline), Renewal::Undecided);
    }

    #[test]
    fn the_login_cookie_is_read_from_the_header() {
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, "a=1; ordo_login=xyz; b=2".parse().unwrap());
        assert_eq!(cookie(&headers).as_deref(), Some("xyz"));
        assert_eq!(cookie(&HeaderMap::new()), None);
    }
}
