//! Helpers without an account (ADR-0077): the incident command makes a guest of one
//! incident and a one-time code for them (`createGuest`); a device pairs with the code
//! (`POST /auth/guest`) and gets the guest's roles until their duty ends
//! (`setGuestAccess`). Codes are kept only as hashes and never logged.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use ordo_core::{GUEST_PREFIX, Ulid, UserId};
use ordo_sync::SessionId;
use ordo_sync::message::{
    ClientMessage, GUEST_TAG_MAX, GuestAccess, GuestPass, GuestRequest, ServerMessage,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::app::{App, now_ms};
use crate::store::StoreError;

/// How long a code works.
pub(crate) const CODE_TTL_MS: u64 = 15 * 60 * 1000;
/// Refused codes per minute, after which the server answers `429` until the minute is over.
pub(crate) const MAX_REFUSED_PER_MINUTE: u64 = 20;

/// Counts refused codes per minute (all clients together: a code has 256 bits, so this is
/// a brake on noise, not the protection).
#[derive(Default)]
pub(crate) struct Refusals {
    /// The minute counted (ms since the epoch / 60 000).
    minute: AtomicU64,
    count: AtomicU64,
}

impl Refusals {
    /// Whether another try is allowed at `now`.
    fn allowed(&self, now: u64) -> bool {
        let minute = now / 60_000;
        if self.minute.swap(minute, Ordering::SeqCst) != minute {
            self.count.store(0, Ordering::SeqCst);
        }
        self.count.load(Ordering::SeqCst) < MAX_REFUSED_PER_MINUTE
    }

    fn refused(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }
}

/// The SHA-256 hash under which a code is kept.
fn hash(code: &str) -> [u8; 32] {
    Sha256::digest(code.as_bytes()).into()
}

/// A new code: 256 bits from the operating system, base64url without padding.
fn new_code() -> Option<String> {
    use base64::Engine;
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).ok()?;
    Some(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes))
}

/// The QR code of a link, as SVG.
fn qr(link: &str) -> Option<String> {
    let code = qrcode::QrCode::new(link.as_bytes()).ok()?;
    Some(
        code.render::<qrcode::render::svg::Color<'_>>()
            .min_dimensions(256, 256)
            .quiet_zone(true)
            .build(),
    )
}

impl App {
    // [impl->dsn~guest-access~1]
    /// Answers `createGuest` and `setGuestAccess` (protocol 1.3) for those who may grant
    /// roles in the incident.
    pub(crate) async fn guests(
        &self,
        session: SessionId,
        message: ClientMessage,
    ) -> Result<(), StoreError> {
        let incident = match &message {
            ClientMessage::CreateGuest(r) => r.incident.clone(),
            ClientMessage::SetGuestAccess(a) => a.incident.clone(),
            _ => return Ok(()),
        };
        let (user, version, may) = {
            let hub = self.hub.lock().await;
            let Some(user) = hub.session_user(session) else {
                return Ok(()); // not authenticated: nothing
            };
            let may = hub.may_assign_roles(&user, &incident) && !user.is_guest();
            (user, hub.session_version(session), may)
        };
        if version.is_none_or(|v| v < ordo_sync::version::SYNC_GUESTS) {
            return Ok(());
        }
        match message {
            ClientMessage::CreateGuest(request) => {
                let answer = self.create_guest(&user, may, request).await?;
                if let Some(sender) = self
                    .sessions
                    .lock()
                    .expect("session registry lock")
                    .get(&session)
                {
                    let _ = sender.send(ServerMessage::Guest(answer));
                }
            }
            ClientMessage::SetGuestAccess(access) => {
                self.set_guest_access(&user, may, access).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn create_guest(
        &self,
        user: &UserId,
        may: bool,
        request: GuestRequest,
    ) -> Result<GuestPass, StoreError> {
        let now = now_ms();
        let refused = GuestPass {
            tag: request.tag.chars().take(GUEST_TAG_MAX).collect(),
            incident: request.incident.clone(),
            guest: None,
            link: None,
            qr: None,
            expires: 0,
        };
        // The login's public address, else the server's name (an installation without login
        // still has guests).
        let public_url = self.login.as_ref().map_or_else(
            || format!("https://{}", self.server),
            |l| l.public_url().to_owned(),
        );
        if !may || request.until <= now || request.tag.chars().count() > GUEST_TAG_MAX {
            return Ok(refused);
        }
        // A new code for a guest of this incident, or a new guest.
        let guest = match request.guest {
            Some(guest) => match self.store.guest(&guest).await? {
                Some((incident, _)) if incident == request.incident => guest,
                _ => return Ok(refused),
            },
            None => {
                let Some(id) = Ulid::from_parts(now, u128::from_le_bytes(random16())).ok() else {
                    return Ok(refused);
                };
                let name = format!("{GUEST_PREFIX}{}", id.to_string().to_lowercase());
                match UserId::new(&name, &self.server) {
                    Ok(guest) => guest,
                    Err(_) => return Ok(refused),
                }
            }
        };
        let Some(code) = new_code() else {
            return Ok(refused);
        };
        let expires = now + CODE_TTL_MS;
        self.store
            .save_guest(&guest, &request.incident, request.until, user, now)
            .await?;
        self.store
            .save_guest_code(&guest, &hash(&code), expires)
            .await?;
        self.set_valid_until(&guest, request.until).await?;
        // Shown by their name in the incident, as a login is (ORG-7).
        if let Some(name) =
            crate::login::tidy_name(request.name.as_deref(), crate::login::MAX_PERSON_NAME)
            && self.store.set_provider_name(&guest, &name).await?
        {
            self.person_changed(&guest).await?;
        }
        tracing::info!(by = %user, %guest, incident = %request.incident, "guest code made");
        let link = format!("{}/?guest={code}", public_url.trim_end_matches('/'));
        Ok(GuestPass {
            qr: qr(&link),
            link: Some(link),
            guest: Some(guest),
            expires,
            ..refused
        })
    }

    async fn set_guest_access(
        &self,
        user: &UserId,
        may: bool,
        access: GuestAccess,
    ) -> Result<(), StoreError> {
        if !may {
            return Ok(());
        }
        match self.store.guest(&access.guest).await? {
            Some((incident, _)) if incident == access.incident => {}
            _ => return Ok(()),
        }
        self.store
            .save_guest(
                &access.guest,
                &access.incident,
                access.until,
                user,
                now_ms(),
            )
            .await?;
        // Ended or moved: the guest's devices follow, and are cut off if it has passed.
        self.set_valid_until(&access.guest, access.until).await?;
        tracing::info!(by = %user, guest = %access.guest, until = access.until, "guest access set");
        Ok(())
    }
}

/// 16 random bytes for a ULID.
fn random16() -> [u8; 16] {
    let mut bytes = [0u8; 16];
    let _ = getrandom::fill(&mut bytes);
    bytes
}

/// `POST /auth/guest`: the device and the code it was given.
#[derive(Deserialize)]
pub(crate) struct GuestPairing {
    code: String,
    device: String,
    key: String,
    /// The device's name (ORG-8), optional.
    #[serde(default)]
    name: Option<String>,
}

// [impl->dsn~guest-access~1]
/// `POST /auth/guest` (HTTP API 1.1): pairs the device with the guest of the code, if the
/// code is known, unused and not expired; any other code is refused the same way. Answers
/// the guest's ID and incident.
pub(crate) async fn pair(
    State(app): State<Arc<App>>,
    Json(request): Json<GuestPairing>,
) -> Response {
    let now = now_ms();
    if !app.guest_refusals.allowed(now) {
        return (StatusCode::TOO_MANY_REQUESTS, "too many refused codes").into_response();
    }
    let Some((device, key)) = crate::login::parse_device(&request.device, &request.key) else {
        return (StatusCode::BAD_REQUEST, "invalid device or key").into_response();
    };
    let used = match app
        .store
        .use_guest_code(&hash(request.code.trim()), now)
        .await
    {
        Ok(used) => used,
        Err(e) => {
            tracing::error!(error = %e, "reading a guest code failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some((guest, incident, until)) = used.filter(|(_, _, until)| *until > now) else {
        app.guest_refusals.refused();
        tracing::info!("a guest code was refused");
        return (StatusCode::FORBIDDEN, "code not valid").into_response();
    };
    let name = crate::login::device_name(request.name.as_deref());
    match app
        .register_guest_device(device, guest.clone(), key, name, until)
        .await
    {
        Ok(true) => {}
        Ok(false) => return (StatusCode::CONFLICT, "device taken").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "pairing a guest's device failed");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    tracing::info!(%guest, %incident, device = %device.ulid(), "device paired by a guest code");
    Json(serde_json::json!({ "user": guest.to_string(), "incident": incident.to_string() }))
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~guest-access~1]
    #[test]
    fn codes_have_256_bits_and_are_kept_by_hash() {
        let (a, b) = (new_code().unwrap(), new_code().unwrap());
        assert_eq!(a.len(), 43, "32 bytes in base64url without padding");
        assert_ne!(a, b);
        assert!(
            a.bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
        );
        assert_eq!(hash(&a), hash(&a));
        assert_ne!(hash(&a), hash(&b));
    }

    // [utest->dsn~guest-access~1]
    #[test]
    fn refused_codes_are_braked_per_minute() {
        let r = Refusals::default();
        for _ in 0..MAX_REFUSED_PER_MINUTE {
            assert!(r.allowed(60_000));
            r.refused();
        }
        assert!(!r.allowed(60_001));
        assert!(r.allowed(120_000), "a new minute");
    }

    #[test]
    fn a_guest_id_is_one_of_ordo() {
        let server: ordo_core::ServerName = "ordo.example.org".parse().unwrap();
        let id = Ulid::from_parts(1, 1).unwrap().to_string().to_lowercase();
        let guest = UserId::new(&format!("{GUEST_PREFIX}{id}"), &server).unwrap();
        assert!(guest.is_guest());
    }
}
