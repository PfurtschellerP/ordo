//! Device state, connection and local storage of the web app.
//!
//! The replica works without a connection; the WebSocket connects, reconnects after a
//! loss, and exchanges messages. Everything is saved to browser storage after each
//! change, so a reload or a closed tab loses nothing (UX-5).
//!
//! Pairing data (with the device's private key) and the replica (with compartment data
//! keys) are stored encrypted (ADR-0041, see `vault`); older plain-text items are encrypted
//! on first load.

use leptos::prelude::*;
use ordo_contracts::TriageCategory;
use ordo_core::{DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_med_patients::{
    self as patients, Registered, Retriaged, UhsCreated, UhsKind, hospitals, orders, record,
    transport,
};
use ordo_sync::incidents::{self, IncidentState, Planned, Registry, RoleChange, StateChanged};
use ordo_sync::message::{
    self, ClientMessage, DeviceInfo, IncidentSummary, Rejection, ServerMessage,
};
use ordo_sync::{AppliedEvent, Replica, ReplicaState};
use ordo_units::{self as units, StatusChanged, UnitAdded, UnitRegister};
use serde::{Deserialize, Serialize};
#[allow(unused_imports)]
use wasm_bindgen::JsCast as _;
use wasm_bindgen::JsCast;
use wasm_bindgen::closure::Closure;
use web_sys::{BinaryType, CloseEvent, MessageEvent, WebSocket};

use crate::{text, vault};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;

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
    /// The incident to work on; empty until one is chosen (ADR-0046).
    pub incident: String,
    /// Name of an incident this device is planning: recorded once the server lets the
    /// device write to it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<String>,
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
    /// The incident, once one is chosen.
    pub incident: Option<IncidentId>,
    /// The incidents the user may open, as last listed by the server.
    pub incidents: Option<Vec<IncidentSummary>>,
    /// The people the server knows, as last listed (for granting roles).
    pub people: Vec<UserId>,
    /// The installation's vehicles, as last listed (for adding units).
    pub vehicles: Vec<ordo_sync::message::Vehicle>,
    /// An incident being planned on this device (name), see [`Pairing::plan`].
    plan: Option<String>,
    /// Counts the server's welcomes: views that ask the server for lists ask again after
    /// every new connection.
    pub welcomes: u64,
    /// Reconnect at once after the connection closes (roles changed), not after a pause.
    reconnect_now: bool,
    /// Profile pictures as `data:` URLs (`None`: the person has none), while the app runs.
    pub avatars: std::collections::HashMap<UserId, Option<String>>,
    /// Pictures asked for and not answered yet.
    avatars_asked: std::collections::HashSet<UserId>,
    /// The device's local copy.
    pub replica: Replica,
    /// Connection state.
    pub connection: Connection,
    /// Last text for the screen-reader announcement region.
    pub announcement: String,
    /// Last error, if any.
    pub error: Option<String>,
    /// The device storage refused the last save (full or blocked): input is only kept
    /// in memory until it is sent.
    pub storage_failed: bool,
    /// The server refused the device because its login expired or was ended (ADR-0040).
    pub login_expired: bool,
    /// The device was revoked: its data is deleted, it does not connect again (SEC-4).
    pub revoked: bool,
    /// The devices the user may see, as last listed by the server.
    pub devices: Option<Vec<DeviceInfo>>,
    /// A revocation sent, announced once the server's new list no longer has the device.
    revoking: Option<DeviceId>,
    /// The view was closed: do not reconnect.
    stopped: bool,
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
        let incident: Option<IncidentId> = match pairing.incident.trim() {
            "" => None,
            id => Some(id.parse().map_err(|e| format!("{e}"))?),
        };
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
        if let Some(incident) = &incident {
            replica.subscribe(incident, "incident");
            replica.subscribe(incident, patients::COMPARTMENT);
        }
        Ok(Self {
            incident,
            incidents: None,
            people: Vec::new(),
            vehicles: Vec::new(),
            plan: pairing.plan.clone(),
            welcomes: 0,
            reconnect_now: false,
            avatars: std::collections::HashMap::new(),
            avatars_asked: std::collections::HashSet::new(),
            replica,
            connection: Connection::Offline,
            announcement: String::new(),
            error: None,
            storage_failed: false,
            login_expired: false,
            revoked: false,
            devices: None,
            revoking: None,
            stopped: false,
            socket: None,
        })
    }

    /// The events of `compartment` in the chosen incident (none without one).
    pub fn events(&self, compartment: &str) -> Vec<AppliedEvent> {
        self.incident
            .as_ref()
            .and_then(|i| self.replica.events(i, compartment).ok())
            .unwrap_or_default()
    }

    /// The chosen incident's name, state and roster, from its events.
    pub fn incident_info(&self) -> Option<incidents::Incident> {
        let incident = self.incident.as_ref()?;
        Registry::from_events(&self.events(incidents::COMPARTMENT))
            .get(incident)
            .cloned()
    }

    /// Records an event of the chosen incident.
    fn record_event(
        &mut self,
        compartment: &str,
        module: &str,
        event_type: &str,
        payload: &[u8],
    ) -> Result<(), String> {
        let incident = self.incident.clone().ok_or("Kein Einsatz gewählt")?;
        // Payload schema versions that are not 1 (rule: payload changes get a version).
        let schema = match (module, event_type) {
            (patients::MODULE, patients::REGISTERED | patients::RETRIAGED) => {
                patients::TRIAGE_SCHEMA
            }
            (transport::MODULE, transport::UPDATED) => transport::UPDATED_SCHEMA,
            (transport::MODULE, transport::REQUESTED) => transport::REQUESTED_SCHEMA,
            (units::MODULE, units::ADDED) => units::ADDED_SCHEMA,
            _ => 1,
        };
        self.replica
            .record(
                &incident,
                compartment,
                module,
                event_type,
                schema,
                payload,
                now_ms(),
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    // [impl->dsn~incident-roles~1]
    /// Records a planned incident once the server lets this device write to it: its name,
    /// and the planner as its commander.
    fn finish_plan(&mut self) {
        let (Some(name), Some(incident)) = (self.plan.clone(), self.incident.clone()) else {
            return;
        };
        if !self.replica.is_granted(&incident, incidents::COMPARTMENT) {
            return;
        }
        let planned = Planned { name }.encode();
        let commander = RoleChange {
            user: self.replica.user().clone(),
            role: COMMANDER.to_owned(),
        }
        .encode();
        if let (Ok(planned), Ok(commander)) = (planned, commander) {
            let ok = self
                .record_event(
                    incidents::COMPARTMENT,
                    incidents::MODULE,
                    incidents::PLANNED,
                    &planned,
                )
                .and_then(|()| {
                    self.record_event(
                        incidents::COMPARTMENT,
                        incidents::MODULE,
                        incidents::ROLE_GRANTED,
                        &commander,
                    )
                });
            if ok.is_ok() {
                self.plan = None;
                if let Some(mut pairing) = load_pairing() {
                    pairing.plan = None;
                    save_pairing(&pairing);
                }
            }
        }
    }

    pub(crate) fn send(&self, message: &ClientMessage) {
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

/// Opens the connection and keeps it open: reconnects after a loss, until the device view
/// is closed or the device was revoked.
pub fn connect(core: Handle, revision: RwSignal<u64>) {
    if core
        .try_with_value(|c| c.stopped || c.revoked)
        .unwrap_or(true)
    {
        return;
    }
    let Some(url) = sync_url() else { return };
    let Ok(socket) = WebSocket::new(&url) else {
        schedule_reconnect(core, revision);
        return;
    };
    socket.set_binary_type(BinaryType::Arraybuffer);

    // The server opens with a challenge; the hello answering it proves the device key
    // (ADR-0038), see `on_message`.
    let on_open = Closure::<dyn FnMut()>::new(move || {
        core.update_value(|c| c.connection = Connection::Online);
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
        let ticket = match &message {
            ServerMessage::BlobTicket(ticket) => Some(ticket.clone()),
            _ => None,
        };
        core.update_value(|c| {
            if let ServerMessage::Avatars(list) = &message {
                for a in list {
                    c.avatars_asked.remove(&a.user);
                    c.avatars.insert(
                        a.user.clone(),
                        a.image.as_deref().map(crate::images::data_url),
                    );
                }
            }
            if let ServerMessage::Incidents(list) = &message {
                c.incidents = Some(list.clone());
            }
            if let ServerMessage::People(list) = &message {
                c.people = list.clone();
            }
            if let ServerMessage::Vehicles(list) = &message {
                c.vehicles = list.clone();
            }
            if let ServerMessage::Devices(list) = &message {
                if let Some(device) = c.revoking
                    && list.iter().all(|d| d.device != device)
                {
                    c.revoking = None;
                    c.announcement = text::device_revoked(&short_device(&device));
                }
                c.devices = Some(list.clone());
            }
            match c.replica.receive(message, now_ms()) {
                Ok(()) => {
                    c.error = None;
                    // Only a welcome proves that the login is valid again.
                    if welcome {
                        c.login_expired = false;
                        c.welcomes += 1;
                        // Questions of a lost connection are asked again.
                        c.avatars_asked.clear();
                        c.finish_plan();
                    }
                }
                // Roles changed (ADR-0046): the server closes the connection and the
                // device reconnects to receive what it may read now.
                Err(ordo_sync::SyncError::Refused(Rejection::RolesChanged)) => {
                    c.incidents = None;
                    c.reconnect_now = true;
                }
                // An unknown device logs in again, which pairs it anew.
                Err(ordo_sync::SyncError::Refused(
                    Rejection::LoginExpired | Rejection::UnknownDevice,
                )) => {
                    c.login_expired = true;
                }
                // [impl->dsn~device-revocation~1]
                // Revoked: delete the key and all data on this device (remote wipe).
                Err(ordo_sync::SyncError::Refused(Rejection::Revoked)) => {
                    forget_device();
                    c.revoked = true;
                    c.devices = None;
                    return;
                }
                // [impl->dsn~protocol-versions~1]
                Err(ordo_sync::SyncError::Refused(Rejection::UnsupportedVersion)) => {
                    c.error = Some(text::VERSION_UNSUPPORTED.to_owned());
                }
                Err(error) => c.error = Some(error.to_string()),
            }
            if let Some(hello) = c.replica.hello() {
                c.send(&hello);
                c.push();
            }
            if welcome {
                c.push();
            }
            c.storage_failed = save_replica(&c.replica).is_err();
        });
        if let Some(ticket) = ticket {
            crate::attachments::on_ticket(core, revision, ticket);
        }
        if welcome && core.with_value(|c| c.replica.is_ready()) {
            crate::attachments::on_welcome(core, revision);
        }
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

/// Closes the connection for good (the device view is closed).
pub fn disconnect(core: Handle) {
    core.update_value(|c| {
        c.stopped = true;
        if let Some(socket) = c.socket.take() {
            let _ = socket.close();
        }
    });
}

// [impl->dsn~device-revocation~1]
/// Asks the server for the devices the user may see; the answer lands in `Core::devices`.
pub fn list_devices(core: Handle) {
    core.with_value(|c| c.send(&ClientMessage::ListDevices));
}

// [impl->dsn~device-revocation~1]
/// Revokes a device. The server cuts it off and sends the new list; revoking this device
/// itself wipes it.
pub fn revoke_device(core: Handle, device: DeviceId) {
    core.update_value(|c| {
        c.revoking = Some(device);
        c.send(&ClientMessage::RevokeDevice(device));
    });
}

/// A short, readable form of a device ID: its last six characters (the random part).
pub fn short_device(device: &DeviceId) -> String {
    let id = device.ulid().to_string();
    id[id.len() - 6..].to_owned()
}

fn schedule_reconnect(core: Handle, revision: RwSignal<u64>) {
    let now = core
        .try_update_value(|c| std::mem::take(&mut c.reconnect_now))
        .unwrap_or(false);
    let retry = Closure::once_into_js(move || connect(core, revision));
    if let Some(window) = web_sys::window() {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            retry.unchecked_ref(),
            if now { 0 } else { RECONNECT_MS },
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
        algorithm: triage_algorithm(),
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

/// Sets (or, with `None`, removes) a patient's transport priority.
pub fn set_transport_priority(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    level: Option<String>,
) -> Result<(), String> {
    let announcement = text::priority_changed(tag, level.is_some());
    let payload = patients::TransportPrioritySet {
        tag: tag.to_owned(),
        level,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::TRANSPORT_PRIORITY,
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
        algorithm: triage_algorithm(),
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
    let id = new_ulid().ok_or("Keine Zufallszahlen verfügbar")?;
    let uhs = format!("uhs-{id}");
    let payload = UhsCreated {
        id: uhs.clone(),
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
    )?;
    // One numbered treatment place per patient it can take; changeable later.
    set_places(core, revision, &uhs, capacity)
}

// [impl->dsn~treatment-places~1]
/// Sets the number of treatment places of a unit: places 1..=`count`, keeping the labels
/// of existing ones.
pub fn set_places(
    core: Handle,
    revision: RwSignal<u64>,
    uhs: &str,
    count: u32,
) -> Result<(), String> {
    let existing = core
        .with_value(|c| patients::places(&c.events(patients::UHS_COMPARTMENT)))
        .remove(uhs)
        .unwrap_or_default();
    let places = (1..=count)
        .map(|i| {
            let id = format!("p{i}");
            existing
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .unwrap_or(patients::Place {
                    label: text::place_label(i),
                    id,
                })
        })
        .collect();
    let payload = patients::PlacesSet {
        uhs: uhs.to_owned(),
        places,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        patients::UHS_COMPARTMENT,
        patients::UHS_MODULE,
        patients::UHS_PLACES,
        &payload,
        text::places_set(count),
    )
}

/// Assigns a patient to a treatment place of their unit (or releases them).
pub fn assign_place(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    place: Option<String>,
    label: Option<&str>,
) -> Result<(), String> {
    let payload = patients::PlaceAssigned {
        tag: tag.to_owned(),
        place,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::PLACE_ASSIGNED,
        &payload,
        text::place_assigned(tag, label),
    )
}

// [impl->dsn~uhs-discharge~1]
/// Discharges a patient from their unit (UHS-3).
pub fn discharge(
    core: Handle,
    revision: RwSignal<u64>,
    discharge: &patients::Discharged,
) -> Result<(), String> {
    let payload = discharge.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::DISCHARGED,
        &payload,
        text::discharged(&discharge.tag, discharge.outcome),
    )
}

// [impl->dsn~patient-pathway~1]
/// Moves a patient to another on-site unit (PAT-5).
pub fn move_patient(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    uhs: &str,
) -> Result<(), String> {
    let payload = patients::Moved {
        tag: tag.to_owned(),
        uhs: uhs.to_owned(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    let announcement = text::patient_moved(tag, &uhs_name(core, uhs));
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::MOVED,
        &payload,
        announcement,
    )
}

/// Readmits a discharged patient to their unit.
pub fn readmit(core: Handle, revision: RwSignal<u64>, tag: &str) -> Result<(), String> {
    let payload = patients::Readmitted {
        tag: tag.to_owned(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        patients::READMITTED,
        &payload,
        text::readmitted(tag),
    )
}

// [impl->dsn~command-structure~1]
/// Creates a position of the command structure (CMD-1) with a new ID.
pub fn create_position(
    core: Handle,
    revision: RwSignal<u64>,
    position: ordo_command::PositionCreated,
) -> Result<(), String> {
    let id = new_ulid().ok_or("Keine Zufallszahlen verfügbar")?;
    let position = ordo_command::PositionCreated {
        position: id.to_string(),
        ..position
    };
    let payload = position.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        ordo_command::COMPARTMENT,
        ordo_command::MODULE,
        ordo_command::POSITION_CREATED,
        &payload,
        text::position_created(&position.name),
    )
}

/// Records another event of the command structure: leader, closing, assignment.
pub fn command_event(
    core: Handle,
    revision: RwSignal<u64>,
    event_type: &str,
    payload: &[u8],
) -> Result<(), String> {
    record(
        core,
        revision,
        ordo_command::COMPARTMENT,
        ordo_command::MODULE,
        event_type,
        payload,
        text::STRUCTURE_SAVED.to_owned(),
    )
}

// [impl->dsn~situation-map~1]
/// Records an event of the situation map: an object placed or removed (MAP-1).
pub fn map_event(
    core: Handle,
    revision: RwSignal<u64>,
    event_type: &str,
    payload: &[u8],
) -> Result<(), String> {
    record(
        core,
        revision,
        ordo_map::COMPARTMENT,
        ordo_map::MODULE,
        event_type,
        payload,
        text::MAP_SAVED.to_owned(),
    )
}

// [impl->dsn~tasks~1]
/// Gives a task (TASK-1): a new ID, then the event in the shared incident compartment.
pub fn give_task(
    core: Handle,
    revision: RwSignal<u64>,
    task: ordo_tasks::TaskGiven,
) -> Result<(), String> {
    let id = new_ulid().ok_or("Keine Zufallszahlen verfügbar")?;
    let task = ordo_tasks::TaskGiven {
        task: id.to_string(),
        ..task
    };
    let payload = task.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        ordo_tasks::COMPARTMENT,
        ordo_tasks::MODULE,
        ordo_tasks::GIVEN,
        &payload,
        text::task_given(task.unit.as_deref().unwrap_or_default()),
    )
}

/// Moves a task on or sets a new follow-up time (TASK-2, TASK-4).
pub fn update_task(
    core: Handle,
    revision: RwSignal<u64>,
    update: &ordo_tasks::TaskUpdated,
) -> Result<(), String> {
    let payload = update.encode().map_err(|e| e.to_string())?;
    let announcement = update.status.map_or_else(
        || text::FOLLOW_UP_LATER_DONE.to_owned(),
        text::task_status_set,
    );
    record(
        core,
        revision,
        ordo_tasks::COMPARTMENT,
        ordo_tasks::MODULE,
        ordo_tasks::UPDATED,
        &payload,
        announcement,
    )
}

// [impl->dsn~uhs-details~1]
/// Changes the location, capacity or staff of an on-site care unit (UHS-1).
pub fn update_uhs(
    core: Handle,
    revision: RwSignal<u64>,
    update: &patients::UhsUpdated,
) -> Result<(), String> {
    let payload = update.encode().map_err(|e| e.to_string())?;
    let name = uhs_name(core, &update.id);
    record(
        core,
        revision,
        patients::UHS_COMPARTMENT,
        patients::UHS_MODULE,
        patients::UHS_UPDATED,
        &payload,
        text::uhs_updated(&name),
    )
}

/// Adds a unit (vehicle, team, other resource) to the incident (UNIT-1).
pub fn add_unit(core: Handle, revision: RwSignal<u64>, unit: UnitAdded) -> Result<(), String> {
    let payload = unit.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        units::COMPARTMENT,
        units::MODULE,
        units::ADDED,
        &payload,
        text::unit_added(&unit.unit),
    )
}

// [impl->dsn~unit-register~2]
/// Releases a unit from the incident (UNIT-6); it stays in the history.
pub fn release_unit(core: Handle, revision: RwSignal<u64>, unit: &str) -> Result<(), String> {
    let payload = units::Released {
        unit: unit.to_owned(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        units::COMPARTMENT,
        units::MODULE,
        units::RELEASED,
        &payload,
        text::unit_released(unit),
    )
}

/// Sets the whole crew of a unit (UNIT-5).
pub fn set_crew(
    core: Handle,
    revision: RwSignal<u64>,
    unit: &str,
    members: Vec<units::CrewMember>,
) -> Result<(), String> {
    let payload = units::CrewSet {
        unit: unit.to_owned(),
        members,
    }
    .encode()
    .map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        units::COMPARTMENT,
        units::MODULE,
        units::CREW_SET,
        &payload,
        text::crew_set(unit),
    )
}

/// Asks the server for the installation's vehicles (UNIT-1; answered to those who edit
/// units).
pub fn list_vehicles(core: Handle) {
    core.with_value(|c| c.send(&ClientMessage::ListVehicles));
}

/// A text as part of an element ID: letters and digits, the rest as `-`.
pub fn slug(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// Sets the status of a unit, recording which status it replaces (lost-update detection).
pub fn set_status(
    core: Handle,
    revision: RwSignal<u64>,
    unit: &str,
    status: &str,
) -> Result<(), String> {
    let replaces = core.with_value(|c| {
        let events = c.events(units::COMPARTMENT);
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

// [impl->dsn~transport-request~1]
/// Requests a transport for a patient (TRANS-1): the request without patient data in the
/// shared incident compartment, the patient it is for in `med.clinical`.
pub fn request_transport(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    request: transport::TransportRequested,
) -> Result<(), String> {
    let id = new_ulid().ok_or("Keine Zufallszahlen verfügbar")?;
    let request = transport::TransportRequested {
        request: id.to_string(),
        ..request
    };
    let link = transport::TransportLinked {
        tag: tag.to_owned(),
        request: request.request.clone(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    let payload = request.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        transport::COMPARTMENT,
        transport::MODULE,
        transport::REQUESTED,
        &payload,
        String::new(),
    )?;
    record(
        core,
        revision,
        patients::COMPARTMENT,
        patients::MODULE,
        transport::LINKED,
        &link,
        text::transport_requested(tag),
    )
}

/// Assigns a transport or moves its status on (TRANS-3).
pub fn update_transport(
    core: Handle,
    revision: RwSignal<u64>,
    update: &transport::TransportUpdated,
) -> Result<(), String> {
    let payload = update.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        transport::COMPARTMENT,
        transport::MODULE,
        transport::UPDATED,
        &payload,
        text::transport_status_set(update.status),
    )
}

// [impl->dsn~hospital-list~1]
/// Adds a hospital to the list (an empty ID makes a new one) or replaces its entry.
pub fn set_hospital(
    core: Handle,
    revision: RwSignal<u64>,
    entry: hospitals::HospitalSet,
) -> Result<(), String> {
    let entry = if entry.hospital.is_empty() {
        let id = new_ulid().ok_or("Keine Zufallszahlen verfügbar")?;
        hospitals::HospitalSet {
            hospital: id.to_string(),
            ..entry
        }
    } else {
        entry
    };
    let payload = entry.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        hospitals::COMPARTMENT,
        hospitals::MODULE,
        hospitals::SET,
        &payload,
        text::hospital_saved(&entry.name),
    )
}

/// Enters a hospital's current capacity.
pub fn set_capacity(
    core: Handle,
    revision: RwSignal<u64>,
    capacity: &hospitals::CapacitySet,
    name: &str,
) -> Result<(), String> {
    let payload = capacity.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        hospitals::COMPARTMENT,
        hospitals::MODULE,
        hospitals::CAPACITY_SET,
        &payload,
        text::capacity_saved(name),
    )
}

// [impl->dsn~medical-orders~1]
/// Gives a medical order (ORDER-1). The server accepts it only from roles that may give
/// orders (ORDER-3).
pub fn give_order(
    core: Handle,
    revision: RwSignal<u64>,
    order: orders::OrderGiven,
) -> Result<(), String> {
    let id = new_ulid().ok_or("Keine Zufallszahlen verfügbar")?;
    let order = orders::OrderGiven {
        order: id.to_string(),
        ..order
    };
    let payload = order.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        orders::COMPARTMENT,
        orders::MODULE,
        orders::GIVEN,
        &payload,
        text::order_given(&order.tag),
    )
}

/// Acknowledges, completes or withdraws an order (ORDER-2).
pub fn update_order(
    core: Handle,
    revision: RwSignal<u64>,
    update: &orders::OrderUpdated,
) -> Result<(), String> {
    let payload = update.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        orders::COMPARTMENT,
        orders::MODULE,
        orders::UPDATED,
        &payload,
        text::order_status_set(update.status),
    )
}

// [impl->dsn~patient-care-record~1]
/// Records values of one ePCR form for a patient (EPCR-1, EPCR-3). `replaces` names the
/// entry a correction replaces (EPCR-4).
pub fn record_entry(
    core: Handle,
    revision: RwSignal<u64>,
    entry: &record::EntryRecorded,
    announcement: String,
) -> Result<(), String> {
    let payload = entry.encode().map_err(|e| e.to_string())?;
    self::record(
        core,
        revision,
        patients::COMPARTMENT,
        record::MODULE,
        record::ENTRY_RECORDED,
        &payload,
        announcement,
    )
}

// [impl->dsn~patient-photo~2]
/// Records a photo of a patient: the preview inside the event, the full photo encrypted
/// with its own key as an attachment, kept on the device and queued for upload.
pub fn add_photo(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    full: &[u8],
    preview: Vec<u8>,
) -> Result<(), String> {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 24];
    entropy(&mut key);
    entropy(&mut nonce);
    if key == [0; 32] {
        return Err("Keine Zufallszahlen verfügbar".into());
    }
    let (reference, data) = ordo_crypto::attachment::seal(key, nonce, "image/jpeg", full);
    let photo = record::PhotoAttached {
        tag: tag.to_owned(),
        preview_type: "image/jpeg".to_owned(),
        preview,
        photo: reference.clone(),
    };
    let payload = photo.encode()?;
    let incident = core
        .with_value(|c| c.incident.clone())
        .ok_or("Kein Einsatz gewählt")?;
    let compartment = patients::COMPARTMENT
        .parse::<ordo_core::CompartmentId>()
        .map_err(|e| e.to_string())?;
    self::record(
        core,
        revision,
        patients::COMPARTMENT,
        record::MODULE,
        record::PHOTO_ATTACHED,
        &payload,
        text::photo_added(tag),
    )?;
    crate::attachments::add(core, incident, compartment, &reference, data);
    revision.update(|r| *r += 1);
    Ok(())
}

/// Display name of a UHS by ID (the ID if unknown).
pub fn uhs_name(core: Handle, id: &str) -> String {
    core.with_value(|c| {
        let events = c.events(patients::UHS_COMPARTMENT);
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
            c.record_event(compartment, module, event_type, payload)?;
            c.storage_failed = save_replica(&c.replica).is_err();
            c.announcement = announcement;
            c.push();
            Ok(())
        })
        .unwrap_or_else(|| Err("App-Zustand nicht verfügbar".into()));
    revision.update(|r| *r += 1);
    result
}

/// A new ULID: the time now and random bits.
fn new_ulid() -> Option<Ulid> {
    let mut random = [0u8; 16];
    entropy(&mut random);
    Ulid::from_parts(now_ms(), u128::from_be_bytes(random)).ok()
}

/// Asks the server for the profile pictures of `users` it has not sent yet (ADR-0047).
pub fn request_avatars(core: Handle, users: &[UserId]) {
    core.update_value(|c| {
        if !c.replica.is_ready() {
            return;
        }
        let missing: Vec<UserId> = users
            .iter()
            .filter(|u| !c.avatars.contains_key(*u) && !c.avatars_asked.contains(*u))
            .cloned()
            .collect();
        if !missing.is_empty() {
            c.avatars_asked.extend(missing.iter().cloned());
            c.send(&ClientMessage::GetAvatars(missing));
        }
    });
}

/// Sets (`Some`) or removes (`None`) the user's own profile picture.
pub fn set_avatar(core: Handle, image: Option<Vec<u8>>) {
    core.with_value(|c| {
        c.send(&ClientMessage::SetAvatar(ordo_sync::message::AvatarImage {
            image,
        }));
    });
}

/// The role of an incident's command: may grant roles (default policies).
pub const COMMANDER: &str = "commander";

// [impl->dsn~incident-roles~1]
/// Asks the server for the incidents the user may open.
pub fn list_incidents(core: Handle) {
    core.with_value(|c| c.send(&ClientMessage::ListIncidents));
}

// [impl->dsn~person-suggestions~1]
/// Asks the server for the people it knows (answered only to those who grant roles).
pub fn list_people(core: Handle) {
    core.with_value(|c| c.send(&ClientMessage::ListPeople));
}

/// The people of the chosen incident's roster, the person using the device first.
pub fn incident_people(core: Handle) -> Vec<String> {
    core.with_value(|c| {
        let me = c.replica.user().to_string();
        let others = c
            .incident_info()
            .map(|i| i.roster.keys().map(ToString::to_string).collect::<Vec<_>>())
            .unwrap_or_default();
        std::iter::once(me.clone())
            .chain(others.into_iter().filter(|u| *u != me))
            .collect()
    })
}

/// Grants (or withdraws) `role` in the chosen incident to `user`.
pub fn change_role(
    core: Handle,
    revision: RwSignal<u64>,
    user: &str,
    role: &str,
    granted: bool,
) -> Result<(), String> {
    let user: UserId = user.trim().parse().map_err(|e| format!("{e}"))?;
    let payload = RoleChange {
        user: user.clone(),
        role: role.to_owned(),
    }
    .encode()
    .map_err(|e| e.to_string())?;
    let (event_type, announcement) = if granted {
        (
            incidents::ROLE_GRANTED,
            text::role_granted(user.as_str(), role),
        )
    } else {
        (
            incidents::ROLE_WITHDRAWN,
            text::role_withdrawn(user.as_str(), role),
        )
    };
    record(
        core,
        revision,
        incidents::COMPARTMENT,
        incidents::MODULE,
        event_type,
        &payload,
        announcement,
    )
}

/// Changes the state of the chosen incident (planned, active, closed).
pub fn set_incident_state(
    core: Handle,
    revision: RwSignal<u64>,
    state: IncidentState,
) -> Result<(), String> {
    let payload = StateChanged { state }.encode().map_err(|e| e.to_string())?;
    record(
        core,
        revision,
        incidents::COMPARTMENT,
        incidents::MODULE,
        incidents::STATE_CHANGED,
        &payload,
        text::incident_state_changed(state),
    )
}

/// A new incident ID on the server of `user`.
pub fn new_incident_id(server: &str) -> Option<String> {
    let server: ServerName = server.trim().parse().ok()?;
    let ulid = new_ulid()?;
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
// Pairing (ADR-0038)
// ---------------------------------------------------------------------------

const PENDING_KEY: &str = "ordo.pending";

/// A device key made on this device, waiting for the login that registers it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingDevice {
    /// Device ID (ULID).
    pub device: String,
    /// Ed25519 private key, 64 hex characters; it never leaves the device.
    pub private_key: String,
    /// The incident to open after pairing.
    pub incident: String,
}

impl PendingDevice {
    /// A new device ID and key pair.
    pub fn create(incident: &str) -> Option<Self> {
        let mut seed = [0u8; 32];
        entropy(&mut seed);
        let device = new_ulid()?;
        Some(Self {
            device: device.to_string(),
            private_key: to_hex(&seed),
            incident: incident.to_owned(),
        })
    }

    /// The public key, 64 hex characters, as the server registers it.
    pub fn public_key(&self) -> Option<String> {
        let seed: [u8; 32] = from_hex(&self.private_key)?;
        Some(to_hex(
            SigningKey::from_bytes(&seed).verifying_key().as_bytes(),
        ))
    }

    /// The finished pairing for `user`.
    pub fn paired(self, user: &str) -> Pairing {
        Pairing {
            user: user.to_owned(),
            device: self.device,
            private_key: self.private_key,
            incident: self.incident,
            plan: None,
        }
    }

    /// Keeps the pending device across the login's page change (encrypted).
    pub fn save(&self) {
        if let Ok(json) = serde_json::to_vec(self) {
            store(PENDING_KEY, &json);
        }
    }

    /// The pending device of a login that started on this device, removed from storage.
    pub fn take() -> Option<Self> {
        let json = load(PENDING_KEY);
        storage()?.remove_item(PENDING_KEY).ok()?;
        serde_json::from_slice(&json?).ok()
    }
}

// [impl->dsn~protocol-versions~1]
/// Headers for a request to the ORDO server: the HTTP API version the app speaks
/// (ADR-0052).
pub fn api_headers() -> Option<web_sys::Headers> {
    let headers = web_sys::Headers::new().ok()?;
    headers
        .set(
            ordo_sync::version::HTTP_HEADER,
            &ordo_sync::version::HTTP.max.to_string(),
        )
        .ok()?;
    Some(headers)
}

/// Sends a request to the ORDO server and reads a JSON answer.
pub async fn fetch_json(
    method: &str,
    url: &str,
    body: Option<&serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let init = web_sys::RequestInit::new();
    init.set_method(method);
    let headers = api_headers().ok_or("headers")?;
    if let Some(body) = body {
        let _ = headers.set("content-type", "application/json");
        init.set_body(&wasm_bindgen::JsValue::from_str(&body.to_string()));
    }
    init.set_headers(&headers);
    let request = web_sys::Request::new_with_str_and_init(url, &init).map_err(|_| "request")?;
    let window = web_sys::window().ok_or("window")?;
    let response: web_sys::Response =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|_| text::NETWORK_ERROR.to_owned())?
            .dyn_into()
            .map_err(|_| "response")?;
    let text = wasm_bindgen_futures::JsFuture::from(response.text().map_err(|_| "body")?)
        .await
        .map_err(|_| "body")?
        .as_string()
        .unwrap_or_default();
    if !response.ok() {
        return Err(if text.is_empty() {
            format!("HTTP {}", response.status())
        } else {
            text
        });
    }
    serde_json::from_str(&text).map_err(|e| e.to_string())
}

/// Waits `ms` milliseconds (browser timer).
pub async fn sleep_ms(ms: i32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        if let Some(window) = web_sys::window() {
            let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms);
        }
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

/// A query parameter of the page URL.
pub fn url_param(name: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    web_sys::UrlSearchParams::new_with_str(&search)
        .ok()?
        .get(name)
}

/// Removes query parameters from the address bar (after a login returned).
pub fn clear_url_params() {
    if let Some(window) = web_sys::window()
        && let Ok(history) = window.history()
        && let Ok(path) = window.location().pathname()
    {
        let _ = history.replace_state_with_url(&wasm_bindgen::JsValue::NULL, "", Some(&path));
    }
}

// ---------------------------------------------------------------------------
// Browser storage
// ---------------------------------------------------------------------------

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

// [impl->dsn~device-storage-encryption~1]
/// Stores an item encrypted (ADR-0041). Without an open vault nothing is stored: plain
/// text never reaches the device storage.
fn store(name: &str, bytes: &[u8]) -> bool {
    let (Some(storage), Some(text)) = (storage(), vault::encrypt(name, bytes)) else {
        return false;
    };
    storage.set_item(name, &text).is_ok()
}

// [impl->dsn~device-storage-encryption~1]
/// Loads an encrypted item. An item that cannot be decrypted (key replaced) is removed.
fn load(name: &str) -> Option<Vec<u8>> {
    let storage = storage()?;
    let text = storage.get_item(name).ok()??;
    let plain = vault::decrypt(name, &text);
    if plain.is_none() {
        let _ = storage.remove_item(name);
    }
    plain
}

/// Saved pairing data. Older versions stored it as plain JSON: it is encrypted now.
pub fn load_pairing() -> Option<Pairing> {
    let text = storage()?.get_item(PAIRING_KEY).ok()??;
    if !vault::is_encrypted(&text) {
        let pairing: Pairing = serde_json::from_str(&text).ok()?;
        save_pairing(&pairing);
        return Some(pairing);
    }
    serde_json::from_slice(&load(PAIRING_KEY)?).ok()
}

/// Saves pairing data (encrypted).
pub fn save_pairing(pairing: &Pairing) {
    if let Ok(json) = serde_json::to_vec(pairing) {
        store(PAIRING_KEY, &json);
    }
}

/// A per-device preference (e.g. the colour theme), not synchronised.
pub fn preference(key: &str) -> Option<String> {
    storage()?.get_item(key).ok()?
}

/// Where the device keeps the triage algorithm the person uses (PAT-11).
const TRIAGE_ALGORITHM_KEY: &str = "ordo.triage-algorithm";

// [impl->dsn~triage-reference~1]
/// The triage algorithm the person chose on this device, recorded with every triage.
pub fn triage_algorithm() -> Option<String> {
    preference(TRIAGE_ALGORITHM_KEY).filter(|a| !a.is_empty())
}

/// Chooses (or, with an empty ID, clears) the triage algorithm on this device.
pub fn set_triage_algorithm(id: &str) {
    set_preference(TRIAGE_ALGORITHM_KEY, (!id.is_empty()).then_some(id));
}

/// Saves or (with `None`) removes a per-device preference.
pub fn set_preference(key: &str, value: Option<&str>) {
    if let Some(storage) = storage() {
        let _ = match value {
            Some(value) => storage.set_item(key, value),
            None => storage.remove_item(key),
        };
    }
}

// [impl->dsn~device-storage-encryption~1]
/// Removes pairing and local data from this browser, and replaces the storage key, so
/// nothing left behind in the browser's files can be decrypted (crypto-shredding).
pub fn forget_device() {
    if let Some(storage) = storage() {
        let _ = storage.remove_item(PAIRING_KEY);
        let _ = storage.remove_item(REPLICA_KEY);
        let _ = storage.remove_item(PENDING_KEY);
    }
    crate::storage::clear();
    crate::attachments::clear();
    vault::shred();
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Prefix of the Base64 storage format (older versions stored hex without prefix).
const BASE64_PREFIX: &str = "b64:";

// [impl->dsn~local-persistence~1]
fn load_replica() -> Option<ReplicaState> {
    // Device storage in the worker (ADR-0034); data of older versions moves there from
    // `localStorage` with the first write.
    if crate::storage::active() {
        if let Some(state) = crate::storage::take_loaded() {
            return Some(state);
        }
        let legacy = load_legacy_replica();
        if legacy.is_some() {
            crate::storage::after_first_write(|| {
                if let Some(s) = storage() {
                    let _ = s.remove_item(REPLICA_KEY);
                }
            });
        }
        return legacy;
    }
    load_legacy_replica()
}

/// The replica as older versions stored it in `localStorage` (and browsers without OPFS
/// in workers still do).
fn load_legacy_replica() -> Option<ReplicaState> {
    let text = storage()?.get_item(REPLICA_KEY).ok()??;
    let bytes = if vault::is_encrypted(&text) {
        load(REPLICA_KEY)?
    } else {
        // Older versions: plain Base64 or hex; encrypted from now on.
        let bytes = match text.strip_prefix(BASE64_PREFIX) {
            Some(b64) => BASE64.decode(b64).ok()?,
            None => from_hex_vec(&text)?,
        };
        store(REPLICA_KEY, &bytes);
        bytes
    };
    message::decode(&bytes).ok()
}

// [impl->dsn~local-persistence~1]
/// Saves the replica, encrypted. An error means the browser refused (storage full or
/// blocked); the caller shows it, because unsaved input would be lost on reload (UX-5).
fn save_replica(replica: &Replica) -> Result<(), ()> {
    if crate::storage::active() {
        return crate::storage::save(replica);
    }
    let bytes = message::encode(&replica.snapshot()).map_err(|_| ())?;
    if store(REPLICA_KEY, &bytes) {
        Ok(())
    } else {
        Err(())
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

/// Local time of day, `HH:MM`, of a timestamp in milliseconds.
pub fn clock(ms: u64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    format!("{:02}:{:02}", date.get_hours(), date.get_minutes())
}

/// The timestamp in milliseconds of a local date and time as a `datetime-local` input
/// gives it (`YYYY-MM-DDTHH:MM`), or `None` if it is not one.
pub fn local_time_ms(value: &str) -> Option<u64> {
    // ECMAScript reads a date-time form without an offset as local time.
    let ms = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(value)).get_time();
    (ms.is_finite() && ms >= 0.0).then_some(ms as u64)
}

/// Local time of day with seconds, `HH:MM:SS` (incident log).
pub fn clock_seconds(ms: u64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    format!("{}:{:02}", clock(ms), date.get_seconds())
}

/// Local date, `DD.MM.YYYY`, of a timestamp in milliseconds.
pub fn date(ms: u64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    format!(
        "{:02}.{:02}.{}",
        date.get_date(),
        date.get_month() + 1,
        date.get_full_year()
    )
}

/// A timestamp in milliseconds as ISO 8601 in UTC, for `<time datetime>`.
pub fn iso_time(ms: u64) -> String {
    js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64))
        .to_iso_string()
        .into()
}

/// Local date and time, `DD.MM.YYYY, HH:MM`, of a timestamp in milliseconds.
pub fn date_time(ms: u64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms as f64));
    format!(
        "{:02}.{:02}.{}, {}",
        date.get_date(),
        date.get_month() + 1,
        date.get_full_year(),
        clock(ms)
    )
}

/// The current local time of day, `HH:MM`.
pub fn now_clock() -> String {
    clock(now_ms())
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
