//! Device state, connection and local storage of the web app.
//!
//! The replica works without a connection; the WebSocket connects, reconnects after a
//! loss, and exchanges messages. Everything is saved to browser storage after each
//! change, so a reload or a closed tab loses nothing (UX-5).
//!
//! Prototype limitations: the pairing data (including the device's private key) and the
//! replica state (including compartment data keys) are kept in `localStorage` without
//! extra protection. A native shell or WebCrypto-protected keys come later.

use leptos::prelude::*;
use ordo_contracts::TriageCategory;
use ordo_core::{DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_med_patients::{self as patients, Registered, Retriaged, UhsCreated, UhsKind};
use ordo_sync::message::{self, ClientMessage, ServerMessage};
use ordo_sync::{Replica, ReplicaState};
use ordo_units::{self as units, StatusChanged, UnitAdded, UnitRegister};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{BinaryType, CloseEvent, MessageEvent, WebSocket};

use crate::text;

const PAIRING_KEY: &str = "ordo.pairing";
const REPLICA_KEY: &str = "ordo.replica";
const RECONNECT_MS: i32 = 3_000;

/// What a device needs to know about itself (prototype pairing).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pairing {
    /// User ID, e.g. `anna@ordo.example.org`.
    pub user: String,
    /// Device ID (ULID).
    pub device: String,
    /// Ed25519 private key, 64 hex characters.
    pub private_key: String,
    /// The incident to work on.
    pub incident: String,
}

/// State of the connection to the server.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Connection {
    /// No connection; retrying.
    Offline,
    /// Connecting.
    Connecting,
    /// Connected.
    Online,
}

/// Everything the app works with.
pub struct Core {
    /// The incident.
    pub incident: IncidentId,
    /// The device's local copy.
    pub replica: Replica,
    /// Connection state.
    pub connection: Connection,
    /// Last text for the screen-reader announcement region.
    pub announcement: String,
    /// Last error, if any.
    pub error: Option<String>,
    socket: Option<WebSocket>,
}

/// Handle to the app state, shared by all views.
pub type Handle = StoredValue<Core, LocalStorage>;

impl Core {
    /// Builds the state from pairing data and, if present, the saved replica.
    pub fn new(pairing: &Pairing) -> Result<Self, String> {
        let user: UserId = pairing.user.trim().parse().map_err(|e| format!("{e}"))?;
        let server: ServerName = user.server().parse().map_err(|e| format!("{e}"))?;
        let device = DeviceId(
            pairing
                .device
                .trim()
                .parse::<Ulid>()
                .map_err(|e| format!("Geräte-ID: {e}"))?,
        );
        let key: [u8; 32] = from_hex(pairing.private_key.trim())
            .ok_or("Privater Schlüssel: 64 Hex-Zeichen erwartet")?;
        let incident: IncidentId = pairing
            .incident
            .trim()
            .parse()
            .map_err(|e| format!("{e}"))?;
        let signer = || {
            EventSigner::new(
                SigningKey::from_bytes(&key),
                KeyId::new(&format!("device-{}", device.ulid()), &server).expect("valid key name"),
            )
        };
        let saved = load_replica();
        let mut replica = match saved {
            Some(state) => Replica::restore(user.clone(), device, signer(), entropy, state)
                .unwrap_or_else(|_| Replica::new(user.clone(), device, signer(), entropy)),
            None => Replica::new(user, device, signer(), entropy),
        };
        replica.subscribe(&incident, "incident");
        replica.subscribe(&incident, patients::COMPARTMENT);
        Ok(Self {
            incident,
            replica,
            connection: Connection::Offline,
            announcement: String::new(),
            error: None,
            socket: None,
        })
    }

    fn send(&self, message: &ClientMessage) {
        if let (Some(socket), Ok(bytes)) = (&self.socket, message::encode(message))
            && socket.ready_state() == WebSocket::OPEN
        {
            let _ = socket.send_with_u8_array(&bytes);
        }
    }

    fn push(&self) {
        if let Some(push) = self.replica.push() {
            self.send(&push);
        }
    }
}

/// Opens the connection and keeps it open: reconnects after a loss.
pub fn connect(core: Handle, revision: RwSignal<u64>) {
    let Some(url) = sync_url() else { return };
    let Ok(socket) = WebSocket::new(&url) else {
        schedule_reconnect(core, revision);
        return;
    };
    socket.set_binary_type(BinaryType::Arraybuffer);

    let on_open = Closure::<dyn FnMut()>::new(move || {
        core.update_value(|c| {
            c.connection = Connection::Online;
            c.send(&c.replica.hello());
            c.push();
        });
        revision.update(|r| *r += 1);
    });
    let on_message = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
        let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() else {
            return;
        };
        let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        let Ok(message) = message::decode::<ServerMessage>(&bytes) else {
            return;
        };
        let welcome = matches!(message, ServerMessage::Welcome(_));
        core.update_value(|c| {
            match c.replica.receive(message, now_ms()) {
                Ok(()) => c.error = None,
                Err(error) => c.error = Some(error.to_string()),
            }
            if welcome {
                c.push();
            }
            save_replica(&c.replica);
        });
        revision.update(|r| *r += 1);
    });
    let on_close = Closure::<dyn FnMut(CloseEvent)>::new(move |_| {
        core.update_value(|c| {
            c.connection = Connection::Offline;
            c.socket = None;
        });
        revision.update(|r| *r += 1);
        schedule_reconnect(core, revision);
    });
    socket.set_onopen(Some(on_open.as_ref().unchecked_ref()));
    socket.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    socket.set_onclose(Some(on_close.as_ref().unchecked_ref()));
    // The callbacks live as long as the socket; one set per connection attempt.
    on_open.forget();
    on_message.forget();
    on_close.forget();
    core.update_value(|c| {
        c.connection = Connection::Connecting;
        c.socket = Some(socket);
    });
    revision.update(|r| *r += 1);
}

fn schedule_reconnect(core: Handle, revision: RwSignal<u64>) {
    let retry = Closure::once_into_js(move || connect(core, revision));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            retry.unchecked_ref(),
            RECONNECT_MS,
        );
    }
}

/// Registers a patient: stored locally at once, sent when connected.
pub fn register(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    category: TriageCategory,
    uhs: &str,
) -> Result<(), String> {
    let payload = Registered {
        tag: tag.to_owned(),
        category,
        uhs: uhs.to_owned(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    let announcement = text::registered(tag, category, &uhs_name(core, uhs));
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::REGISTERED,
        &payload,
        announcement,
    )
}

/// Re-triages a patient.
pub fn retriage(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    category: TriageCategory,
) -> Result<(), String> {
    let payload = Retriaged {
        tag: tag.to_owned(),
        category,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    let announcement = text::retriaged(tag, category);
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::RETRIAGED,
        &payload,
        announcement,
    )
}

/// Sets up an on-site care unit (UHS-1).
pub fn create_uhs(
    core: Handle,
    revision: RwSignal<u64>,
    name: &str,
    capacity: u32,
) -> Result<(), String> {
    let mut random = [0u8; 16];
    entropy(&mut random);
    let id = Ulid::from_parts(now_ms(), u128::from_be_bytes(random)).map_err(|e| e.to_string())?;
    let payload = UhsCreated {
        id: format!("uhs-{id}"),
        name: name.to_owned(),
        kind: UhsKind::Uhs,
        capacity,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        patients::UHS_COMPARTMENT,
        patients::UHS_MODULE,
        patients::UHS_CREATED,
        &payload,
        text::uhs_created(name),
    )
}

/// Adds a unit (vehicle, team, other resource) to the incident (UNIT-1).
pub fn add_unit(
    core: Handle,
    revision: RwSignal<u64>,
    unit: &str,
    unit_type: &str,
    organisation: &str,
) -> Result<(), String> {
    let payload = UnitAdded {
        unit: unit.to_owned(),
        unit_type: unit_type.to_owned(),
        organisation: organisation.to_owned(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        units::COMPARTMENT,
        units::MODULE,
        units::ADDED,
        &payload,
        text::unit_added(unit),
    )
}

/// Sets the status of a unit, recording which status it replaces (lost-update detection).
pub fn set_status(
    core: Handle,
    revision: RwSignal<u64>,
    unit: &str,
    status: &str,
) -> Result<(), String> {
    let replaces = core.with_value(|c| {
        let events = c
            .replica
            .events(&c.incident, units::COMPARTMENT)
            .unwrap_or_default();
        UnitRegister::from_events(&events)
            .units
            .get(unit)
            .and_then(|u| u.status.as_ref().map(|s| s.since))
    });
    let payload = StatusChanged {
        unit: unit.to_owned(),
        status: status.to_owned(),
        replaces,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        units::COMPARTMENT,
        units::MODULE,
        units::STATUS_CHANGED,
        &payload,
        text::status_set(unit, status),
    )
}

/// Display name of a UHS by ID (the ID if unknown).
pub fn uhs_name(core: Handle, id: &str) -> String {
    core.with_value(|c| {
        let events = c
            .replica
            .events(&c.incident, patients::UHS_COMPARTMENT)
            .unwrap_or_default();
        patients::uhs_list(&events)
            .into_iter()
            .find(|u| u.id == id)
            .map_or_else(|| id.to_owned(), |u| u.name)
    })
}

#[allow(clippy::too_many_arguments)]
fn record(
    core: Handle,
    revision: RwSignal<u64>,
    compartment: &str,
    module: &str,
    event_type: &str,
    payload: &[u8],
    announcement: String,
) -> Result<(), String> {
    let result = core
        .try_update_value(|c| {
            let incident = c.incident.clone();
            c.replica
                .record(
                    &incident,
                    compartment,
                    module,
                    event_type,
                    1,
                    payload,
                    now_ms(),
                )
                .map_err(|e| e.to_string())?;
            save_replica(&c.replica);
            c.announcement = announcement;
            c.push();
            Ok(())
        })
        .unwrap_or_else(|| Err("App-Zustand nicht verfügbar".into()));
    revision.update(|r| *r += 1);
    result
}

/// A new incident ID on the server of `user`.
pub fn new_incident_id(user: &str) -> Option<String> {
    let user: UserId = user.trim().parse().ok()?;
    let server: ServerName = user.server().parse().ok()?;
    let mut random = [0u8; 16];
    entropy(&mut random);
    let ulid = Ulid::from_parts(now_ms(), u128::from_be_bytes(random)).ok()?;
    Some(IncidentId::new(ulid, &server).to_string())
}

/// The `incident` parameter of the page URL, for sharing an incident as a link.
pub fn incident_from_url() -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get("incident")
}

// ---------------------------------------------------------------------------
// Browser storage
// ---------------------------------------------------------------------------

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// Saved pairing data.
pub fn load_pairing() -> Option<Pairing> {
    serde_json::from_str(&storage()?.get_item(PAIRING_KEY).ok()??).ok()
}

/// Saves pairing data.
pub fn save_pairing(pairing: &Pairing) {
    if let (Some(storage), Ok(json)) = (storage(), serde_json::to_string(pairing)) {
        let _ = storage.set_item(PAIRING_KEY, &json);
    }
}

/// Removes pairing and local data from this browser.
pub fn forget_device() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(PAIRING_KEY);
        let _ = storage.remove_item(REPLICA_KEY);
    }
}

// [impl->dsn~local-persistence~1]
fn load_replica() -> Option<ReplicaState> {
    let hex = storage()?.get_item(REPLICA_KEY).ok()??;
    message::decode(&from_hex_vec(&hex)?).ok()
}

// [impl->dsn~local-persistence~1]
fn save_replica(replica: &Replica) {
    if let (Some(storage), Ok(bytes)) = (storage(), message::encode(&replica.snapshot())) {
        let _ = storage.set_item(REPLICA_KEY, &to_hex(&bytes));
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

/// Randomness from the browser (Web Crypto).
fn entropy(buffer: &mut [u8]) {
    if let Some(crypto) = web_sys::window().and_then(|w| w.crypto().ok()) {
        let _ = crypto.get_random_values_with_u8_array(buffer);
    }
}

fn sync_url() -> Option<String> {
    let location = web_sys::window()?.location();
    let scheme = if location.protocol().ok()? == "https:" {
        "wss"
    } else {
        "ws"
    };
    Some(format!("{scheme}://{}/sync", location.host().ok()?))
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex_vec(text: &str) -> Option<Vec<u8>> {
    if !text.len().is_multiple_of(2) || !text.is_ascii() {
        return None;
    }
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).ok())
        .collect()
}

fn from_hex<const N: usize>(text: &str) -> Option<[u8; N]> {
    from_hex_vec(text)?.try_into().ok()
}
