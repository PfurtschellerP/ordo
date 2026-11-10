//! Med package, module `med.patients` (prototype: registration and re-triage).
//!
//! All events live in the protected compartment `med.clinical`. The only thing that
//! leaves the package is the connector `med.patientCounts.v1`: numbers per triage
//! category and on-site care unit, derived on the server ([`PatientCountsDeriver`]).

use std::collections::{BTreeMap, HashMap};

use ordo_contracts::{PatientCountsV1, TriageCategory};
use ordo_core::codec::{self, CodecError};
use ordo_core::{CompartmentId, Hlc, IncidentId, UserId};
use ordo_sync::{AppliedEvent, Derived, Deriver};
use serde::{Deserialize, Serialize};

pub mod hospitals;
pub mod orders;
pub mod overview;
pub mod questionnaire;
pub mod record;
pub mod staff;
pub mod transport;
pub mod triage;

/// Module ID.
pub const MODULE: &str = "med.patients";
/// Compartment of patient events.
pub const COMPARTMENT: &str = "med.clinical";
/// Event type: a patient was registered.
pub const REGISTERED: &str = "patients.registered";
/// Event type: a patient was re-triaged.
pub const RETRIAGED: &str = "patients.retriaged";

/// Module of the on-site care units (UHS, treatment areas, collection points).
pub const UHS_MODULE: &str = "med.uhs";
/// Event type: an on-site care unit was set up.
pub const UHS_CREATED: &str = "uhs.created";
/// UHS definitions are not clinical: they live in the shared incident compartment, so
/// command sees names and capacities next to the patient counts.
pub const UHS_COMPARTMENT: &str = "incident";

/// Kind of on-site care unit.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UhsKind {
    /// Unfallhilfsstelle (UHS), e.g. at a planned event.
    Uhs,
    /// Behandlungsplatz in a mass-casualty incident.
    TreatmentArea,
    /// Patientenablage.
    CollectionPoint,
}

/// Payload of `uhs.created` (UHS-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UhsCreated {
    /// Stable ID, referenced by patients and counts.
    pub id: String,
    /// Display name, e.g. "UHS Bühne Nord".
    pub name: String,
    /// Kind.
    pub kind: UhsKind,
    /// Number of patients it can take.
    pub capacity: u32,
}

impl UhsCreated {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Event type: the treatment places of an on-site care unit were set (the whole list).
pub const UHS_PLACES: &str = "uhs.placesSet";
/// Event type: a patient was assigned to a treatment place (or released from it).
pub const PLACE_ASSIGNED: &str = "patients.placeAssigned";

/// A treatment place (Behandlungsplatz) within an on-site care unit.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    /// Stable ID within the unit, e.g. `p3`.
    pub id: String,
    /// Label, e.g. "Platz 3" (chosen by the app or the person, never by this package).
    pub label: String,
}

/// Payload of `uhs.placesSet`: the treatment places of one unit, replacing earlier lists.
/// Shared data (incident compartment); which patient lies where is clinical.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacesSet {
    /// The unit ([`UhsCreated::id`]).
    pub uhs: String,
    /// Its places, in order.
    pub places: Vec<Place>,
}

/// Payload of `patients.placeAssigned` (clinical compartment).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceAssigned {
    /// Number of the triage tag.
    pub tag: String,
    /// The place ([`Place::id`]) in the patient's unit, or `None` when released.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
}

impl PlacesSet {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

impl PlaceAssigned {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

// [impl->dsn~treatment-places~1]
/// The treatment places of every unit: the latest list per unit.
pub fn places(incident_events: &[AppliedEvent]) -> BTreeMap<String, Vec<Place>> {
    let mut sets: Vec<(Hlc, PlacesSet)> = incident_events
        .iter()
        .filter(|e| {
            e.body.module.as_str() == UHS_MODULE && e.body.event_type.as_str() == UHS_PLACES
        })
        .filter_map(|e| {
            codec::from_cbor::<PlacesSet>(&e.payload)
                .ok()
                .map(|s| (e.body.hlc, s))
        })
        .collect();
    sets.sort_by_key(|(hlc, _)| *hlc);
    sets.into_iter().map(|(_, s)| (s.uhs, s.places)).collect()
}

/// Event type: an on-site care unit's location, capacity or staff changed (UHS-1).
pub const UHS_UPDATED: &str = "uhs.updated";

/// Payload of `uhs.updated`: the fields given replace earlier values; a field left out
/// keeps its value.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UhsUpdated {
    /// The unit ([`UhsCreated::id`]).
    pub id: String,
    /// Where it is, e.g. "Bühne Nord, Zelt 2".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Number of patients it can take.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capacity: Option<u32>,
    /// Assigned staff (names or user IDs), the whole list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub staff: Option<Vec<String>>,
}

impl UhsUpdated {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Location and assigned staff of an on-site care unit (UHS-1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UhsDetails {
    /// Where it is.
    pub location: Option<String>,
    /// Assigned staff.
    pub staff: Vec<String>,
}

// [impl->dsn~uhs-registry~1]
// [impl->dsn~uhs-details~1]
/// The on-site care units of an incident, in order of creation (first wins per ID), with
/// the current capacity.
pub fn uhs_list(incident_events: &[AppliedEvent]) -> Vec<UhsCreated> {
    let mut created: Vec<(Hlc, UhsCreated)> = incident_events
        .iter()
        .filter(|e| {
            e.body.module.as_str() == UHS_MODULE && e.body.event_type.as_str() == UHS_CREATED
        })
        .filter_map(|e| {
            codec::from_cbor::<UhsCreated>(&e.payload)
                .ok()
                .map(|u| (e.body.hlc, u))
        })
        .collect();
    created.sort_by_key(|(hlc, _)| *hlc);
    let mut seen = std::collections::HashSet::new();
    let updates = uhs_updates(incident_events);
    created
        .into_iter()
        .map(|(_, u)| u)
        .filter(|u| seen.insert(u.id.clone()))
        .map(|mut u| {
            if let Some(capacity) = updates
                .iter()
                .rev()
                .find_map(|x| (x.id == u.id).then_some(x.capacity).flatten())
            {
                u.capacity = capacity;
            }
            u
        })
        .collect()
}

/// The updates of all units, in clock order.
fn uhs_updates(incident_events: &[AppliedEvent]) -> Vec<UhsUpdated> {
    let mut updates: Vec<(Hlc, UhsUpdated)> = incident_events
        .iter()
        .filter(|e| {
            e.body.module.as_str() == UHS_MODULE && e.body.event_type.as_str() == UHS_UPDATED
        })
        .filter_map(|e| codec::from_cbor(&e.payload).ok().map(|u| (e.body.hlc, u)))
        .collect();
    updates.sort_by_key(|(hlc, _)| *hlc);
    updates.into_iter().map(|(_, u)| u).collect()
}

// [impl->dsn~uhs-details~1]
/// Location and staff per unit ID: the latest value of each field.
pub fn uhs_details(incident_events: &[AppliedEvent]) -> BTreeMap<String, UhsDetails> {
    let mut details: BTreeMap<String, UhsDetails> = BTreeMap::new();
    for u in uhs_updates(incident_events) {
        let d = details.entry(u.id).or_default();
        if let Some(location) = u.location {
            d.location = (!location.trim().is_empty()).then_some(location);
        }
        if let Some(staff) = u.staff {
            d.staff = staff;
        }
    }
    details
}

/// Event type: a patient's transport priority was set or removed.
pub const TRANSPORT_PRIORITY: &str = "patients.transportPriority";

/// Which triage categories may get a transport priority, and its levels: from the profile
/// (e.g. `profiles/de/med/transport-priority.toml`), never in code.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportPriorityConfig {
    /// Categories that may get one (DE: SK I and SK II).
    pub categories: Vec<TriageCategory>,
    /// Levels, highest first (DE: one level; NRW: high and low).
    #[serde(rename = "level", default)]
    pub levels: Vec<TransportPriorityLevel>,
}

/// One level of transport priority.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportPriorityLevel {
    /// Stable ID, e.g. `high`.
    pub id: String,
    /// Label, e.g. "Transportpriorität".
    pub label: String,
}

impl TransportPriorityConfig {
    /// Whether a patient of `category` may get a transport priority.
    pub fn allows(&self, category: TriageCategory) -> bool {
        self.categories.contains(&category)
    }
}

/// Payload of `patients.transportPriority`: documented separately from the triage
/// category (the person decides; ORDO records it).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportPrioritySet {
    /// Number of the triage tag.
    pub tag: String,
    /// The level ID from the profile, or `None` to remove the priority.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
}

impl TransportPrioritySet {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Event type: a patient left their on-site care unit (UHS-3).
pub const DISCHARGED: &str = "patients.discharged";
/// Event type: a patient was moved to another on-site unit (PAT-5).
pub const MOVED: &str = "patients.moved";

/// Payload of `patients.moved` (clinical compartment).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Moved {
    /// Number of the triage tag.
    pub tag: String,
    /// The unit the patient is in from now on ([`UhsCreated::id`]).
    pub uhs: String,
}

impl Moved {
    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Event type: a discharged patient is back in their unit (returned, or discharged by
/// mistake: events are never changed, the readmission corrects it).
pub const READMITTED: &str = "patients.readmitted";

/// How a patient left the on-site care unit (UHS-3). ORDO records what the crew states.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Outcome {
    /// Released on site.
    ReleasedOnSite,
    /// Transported (to a hospital or elsewhere).
    Transported,
    /// Refused treatment or transport.
    Refused,
    /// Deceased.
    Deceased,
}

impl Outcome {
    /// Every outcome, in the order the app offers them.
    pub const ALL: [Self; 4] = [
        Self::ReleasedOnSite,
        Self::Transported,
        Self::Refused,
        Self::Deceased,
    ];

    /// The ID used in events, e.g. `releasedOnSite`.
    pub fn id(self) -> &'static str {
        match self {
            Self::ReleasedOnSite => "releasedOnSite",
            Self::Transported => "transported",
            Self::Refused => "refused",
            Self::Deceased => "deceased",
        }
    }

    /// The outcome with this ID.
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|o| o.id() == id)
    }
}

/// Payload of `patients.discharged` (clinical compartment).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discharged {
    /// Number of the triage tag.
    pub tag: String,
    /// How the patient left.
    pub outcome: Outcome,
    /// The transport request the patient left with ([`transport::TransportRequested`]),
    /// if transported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>,
}

/// Payload of `patients.readmitted` (clinical compartment).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Readmitted {
    /// Number of the triage tag.
    pub tag: String,
}

impl Discharged {
    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

impl Readmitted {
    /// Encodes the payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// An admission to or a discharge from the patient's unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Stay {
    /// Admitted: at registration, or readmitted later.
    Admitted,
    /// Moved from one unit to another (PAT-5).
    Moved {
        /// The unit before.
        from: String,
        /// The unit after.
        to: String,
    },
    /// Discharged, with the outcome and the transport request, if any.
    Discharged {
        /// How the patient left.
        outcome: Outcome,
        /// The transport request, if transported.
        transport: Option<String>,
    },
}

/// One admission or discharge with time and person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StayEntry {
    /// What happened.
    pub change: Stay,
    /// When.
    pub at: Hlc,
    /// Who recorded it.
    pub by: UserId,
}

/// Schema version of `patients.registered` and `patients.retriaged`: 2 since the category
/// `uninjured` ("Betroffene") exists, 3 since they name the algorithm used (PAT-11);
/// older payloads decode unchanged.
pub const TRIAGE_SCHEMA: u32 = 3;

/// Payload of `patients.registered`: the minimum needed (requirement PAT-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registered {
    /// Number of the triage tag.
    pub tag: String,
    /// Triage category.
    pub category: TriageCategory,
    /// ID of the on-site care unit ([`UhsCreated::id`]).
    pub uhs: String,
    /// The triage algorithm used ([`triage::Algorithm::id`]), if the person named one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
}

/// Payload of `patients.retriaged`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retriaged {
    /// Number of the triage tag.
    pub tag: String,
    /// New triage category.
    pub category: TriageCategory,
    /// The triage algorithm used, if the person named one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub algorithm: Option<String>,
}

impl Registered {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

impl Retriaged {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// One triage decision in a patient's history.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriageEntry {
    /// Category.
    pub category: TriageCategory,
    /// The algorithm used, if named (PAT-11).
    pub algorithm: Option<String>,
    /// When.
    pub at: Hlc,
    /// Who.
    pub by: UserId,
}

/// A registered patient.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Patient {
    /// Number of the triage tag.
    pub tag: String,
    /// On-site care unit.
    pub uhs: String,
    /// All triage decisions, oldest first. Never empty.
    pub history: Vec<TriageEntry>,
    /// Transport priority level and when it was set (see [`Patient::transport_priority`]).
    priority: Option<(String, Hlc)>,
    /// The treatment place ([`Place::id`]) in the patient's unit, if assigned.
    pub place: Option<String>,
    /// Admissions, moves and discharges, oldest first; starts with the registration.
    pub stay: Vec<StayEntry>,
}

impl Patient {
    // [impl->dsn~uhs-discharge~1]
    /// The discharge, while the patient is discharged (the latest entry of the stay).
    pub fn discharge(&self) -> Option<&StayEntry> {
        self.stay
            .iter()
            .rev()
            .find(|e| !matches!(e.change, Stay::Moved { .. }))
            .filter(|e| matches!(e.change, Stay::Discharged { .. }))
    }

    /// Whether the patient is in their unit (admitted and not discharged since).
    pub fn is_present(&self) -> bool {
        self.discharge().is_none()
    }

    /// The current triage category (the latest decision).
    pub fn category(&self) -> TriageCategory {
        self.history
            .last()
            .expect("history is never empty")
            .category
    }

    // [impl->dsn~transport-priority~1]
    /// The transport priority level, if one is set and no triage since then was to a
    /// category without transport priority (a priority set in SK I ends for good when the
    /// patient is re-triaged to SK III).
    pub fn transport_priority(&self, config: &TransportPriorityConfig) -> Option<&str> {
        let (level, set_at) = self.priority.as_ref()?;
        let ended = self
            .history
            .iter()
            .any(|t| t.at > *set_at && !config.allows(t.category));
        (!ended && config.allows(self.category())).then_some(level.as_str())
    }
}

/// Something in the events that needs attention.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Notice {
    /// The same triage tag was registered more than once; the first registration counts.
    RegisteredTwice(String),
    /// A re-triage refers to a triage tag that was never registered.
    UnknownPatient(String),
}

/// All patients of an incident, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PatientRegister {
    patients: BTreeMap<String, Patient>,
    /// Notices to show to the UHS lead.
    pub notices: Vec<Notice>,
}

enum Change {
    Registered(Registered),
    Retriaged(Retriaged),
    Priority(TransportPrioritySet),
    Place(PlaceAssigned),
    Discharged(Discharged),
    Readmitted(Readmitted),
    Moved(Moved),
}

impl PatientRegister {
    // [impl->dsn~patient-triage-history~1]
    /// Computes the register. The result doesn't depend on the order of `events`.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, UserId, Change)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let change = match e.body.event_type.as_str() {
                    REGISTERED => Change::Registered(codec::from_cbor(&e.payload).ok()?),
                    RETRIAGED => Change::Retriaged(codec::from_cbor(&e.payload).ok()?),
                    TRANSPORT_PRIORITY => Change::Priority(codec::from_cbor(&e.payload).ok()?),
                    PLACE_ASSIGNED => Change::Place(codec::from_cbor(&e.payload).ok()?),
                    DISCHARGED => Change::Discharged(codec::from_cbor(&e.payload).ok()?),
                    READMITTED => Change::Readmitted(codec::from_cbor(&e.payload).ok()?),
                    MOVED => Change::Moved(codec::from_cbor(&e.payload).ok()?),
                    _ => return None,
                };
                Some((e.body.hlc, e.body.author.user.clone(), change))
            })
            .collect();
        changes.sort_by_key(|(hlc, _, _)| *hlc);
        let mut register = Self::default();
        for (at, by, change) in changes {
            match change {
                Change::Registered(r) => {
                    let entry = TriageEntry {
                        category: r.category,
                        algorithm: r.algorithm.clone(),
                        at,
                        by: by.clone(),
                    };
                    match register.patients.get_mut(&r.tag) {
                        Some(patient) => {
                            patient.history.push(entry);
                            register.notices.push(Notice::RegisteredTwice(r.tag));
                        }
                        None => {
                            register.patients.insert(
                                r.tag.clone(),
                                Patient {
                                    tag: r.tag,
                                    uhs: r.uhs,
                                    history: vec![entry],
                                    priority: None,
                                    place: None,
                                    stay: vec![StayEntry {
                                        change: Stay::Admitted,
                                        at,
                                        by,
                                    }],
                                },
                            );
                        }
                    }
                }
                Change::Retriaged(r) => match register.patients.get_mut(&r.tag) {
                    Some(patient) => patient.history.push(TriageEntry {
                        category: r.category,
                        algorithm: r.algorithm,
                        at,
                        by,
                    }),
                    None => register.notices.push(Notice::UnknownPatient(r.tag)),
                },
                Change::Priority(p) => match register.patients.get_mut(&p.tag) {
                    Some(patient) => patient.priority = p.level.map(|l| (l, at)),
                    None => register.notices.push(Notice::UnknownPatient(p.tag)),
                },
                Change::Place(p) => match register.patients.get_mut(&p.tag) {
                    Some(patient) => patient.place = p.place,
                    None => register.notices.push(Notice::UnknownPatient(p.tag)),
                },
                // [impl->dsn~uhs-discharge~1]
                // A discharge frees the treatment place; a later one (two devices offline)
                // replaces the outcome, and both stay in the history.
                Change::Discharged(d) => match register.patients.get_mut(&d.tag) {
                    Some(patient) => {
                        patient.place = None;
                        patient.stay.push(StayEntry {
                            change: Stay::Discharged {
                                outcome: d.outcome,
                                transport: d.transport,
                            },
                            at,
                            by,
                        });
                    }
                    None => register.notices.push(Notice::UnknownPatient(d.tag)),
                },
                // [impl->dsn~patient-pathway~1]
                // A move frees the place; moving a discharged patient, or to the unit the
                // patient is in, changes nothing.
                Change::Moved(m) => match register.patients.get_mut(&m.tag) {
                    Some(patient) if patient.is_present() && patient.uhs != m.uhs => {
                        patient.place = None;
                        let from = std::mem::replace(&mut patient.uhs, m.uhs.clone());
                        patient.stay.push(StayEntry {
                            change: Stay::Moved { from, to: m.uhs },
                            at,
                            by,
                        });
                    }
                    Some(_) => {}
                    None => register.notices.push(Notice::UnknownPatient(m.tag)),
                },
                // Readmitting a patient who is in the unit changes nothing.
                Change::Readmitted(r) => match register.patients.get_mut(&r.tag) {
                    Some(patient) if !patient.is_present() => patient.stay.push(StayEntry {
                        change: Stay::Admitted,
                        at,
                        by,
                    }),
                    Some(_) => {}
                    None => register.notices.push(Notice::UnknownPatient(r.tag)),
                },
            }
        }
        register
    }

    /// A patient by triage tag number.
    pub fn patient(&self, tag: &str) -> Option<&Patient> {
        self.patients.get(tag)
    }

    /// All patients, sorted by triage tag number.
    pub fn patients(&self) -> impl Iterator<Item = &Patient> {
        self.patients.values()
    }

    /// Number of registered patients.
    pub fn len(&self) -> usize {
        self.patients.len()
    }

    /// Whether no patient is registered.
    pub fn is_empty(&self) -> bool {
        self.patients.is_empty()
    }

    // [impl->dsn~uhs-discharge~1]
    /// Patient counts per on-site care unit, sorted by unit: the patients in the unit per
    /// triage category, and how many were discharged.
    pub fn counts(&self) -> BTreeMap<String, PatientCountsV1> {
        let mut counts: BTreeMap<String, PatientCountsV1> = BTreeMap::new();
        for patient in self.patients.values() {
            let unit = counts
                .entry(patient.uhs.clone())
                .or_insert_with(|| PatientCountsV1 {
                    uhs: patient.uhs.clone(),
                    ..Default::default()
                });
            if patient.is_present() {
                unit.add(patient.category());
            } else {
                unit.discharged += 1;
            }
        }
        counts
    }
}

/// Derives `med.patientCounts.v1` on the server (ADR-0024): whenever the counts of an
/// on-site care unit change, it publishes the new counts to the `incident` compartment.
pub struct PatientCountsDeriver {
    source: CompartmentId,
    events: HashMap<IncidentId, Vec<AppliedEvent>>,
    published: HashMap<IncidentId, BTreeMap<String, PatientCountsV1>>,
}

impl Default for PatientCountsDeriver {
    fn default() -> Self {
        Self {
            source: COMPARTMENT.parse().expect("valid compartment"),
            events: HashMap::new(),
            published: HashMap::new(),
        }
    }
}

// [impl->dsn~server-derived-connectors~1]
impl Deriver for PatientCountsDeriver {
    fn source(&self) -> &CompartmentId {
        &self.source
    }

    fn on_event(&mut self, event: &AppliedEvent) -> Vec<Derived> {
        let incident = event.body.incident.clone();
        let events = self.events.entry(incident.clone()).or_default();
        events.push(event.clone());
        let counts = PatientRegister::from_events(events).counts();
        let published = self.published.entry(incident).or_default();
        let mut derived = Vec::new();
        for (uhs, now) in &counts {
            if published.get(uhs) != Some(now)
                && let Ok(payload) = codec::to_cbor(now)
            {
                derived.push(Derived {
                    compartment: PatientCountsV1::COMPARTMENT.parse().expect("valid"),
                    module: PatientCountsV1::MODULE.parse().expect("valid"),
                    event_type: PatientCountsV1::EVENT_TYPE.parse().expect("valid"),
                    schema_version: 1,
                    payload,
                });
            }
        }
        *published = counts;
        derived
    }
}

/// The latest patient counts per on-site care unit, as any workspace (e.g. command)
/// sees them from the connector events in the `incident` compartment.
pub fn latest_counts(incident_events: &[AppliedEvent]) -> BTreeMap<String, PatientCountsV1> {
    let mut latest = BTreeMap::new();
    for event in incident_events {
        if event.body.module.as_str() == PatientCountsV1::MODULE
            && event.body.event_type.as_str() == PatientCountsV1::EVENT_TYPE
            && let Ok(counts) = codec::from_cbor::<PatientCountsV1>(&event.payload)
        {
            latest.insert(counts.uhs.clone(), counts);
        }
    }
    latest
}

/// Events for unit tests, shared by the modules of this package.
#[cfg(test)]
pub(crate) mod testing {
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, Hlc, IncidentId, ServerName, Ulid};
    use ordo_sync::AppliedEvent;

    /// The `n`-th event of one incident (its clock and ID follow `n`), by `user`.
    pub fn event(
        n: u64,
        user: &str,
        module: &str,
        compartment: &str,
        event_type: &str,
        payload: Vec<u8>,
    ) -> AppliedEvent {
        let server: ServerName = "ordo.example.org".parse().expect("valid server");
        let device = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
        AppliedEvent {
            seq: Some(n),
            body: EventBody {
                version: 1,
                id: EventId(Ulid::from_parts(n, u128::from(n)).expect("valid")),
                schema_version: 1,
                hlc: Hlc {
                    wall_ms: 1000 + n,
                    counter: 0,
                    node: device,
                },
                incident: IncidentId::new(Ulid::from_parts(9, 9).expect("valid"), &server),
                module: module.parse().expect("valid"),
                compartment: compartment.parse().expect("valid"),
                event_type: event_type.parse().expect("valid"),
                author: Author {
                    user: format!("{user}@ordo.example.org")
                        .parse()
                        .expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(n: u64, user: &str, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        testing::event(n, user, MODULE, COMPARTMENT, event_type, payload)
    }

    fn registered(n: u64, tag: &str, category: TriageCategory) -> AppliedEvent {
        let r = Registered {
            tag: tag.into(),
            uhs: "uhs-1".into(),
            category,
            algorithm: None,
        };
        event(n, "anna", REGISTERED, r.encode().unwrap())
    }

    fn discharged(n: u64, tag: &str, outcome: Outcome) -> AppliedEvent {
        let d = Discharged {
            tag: tag.into(),
            outcome,
            transport: (outcome == Outcome::Transported).then(|| "request-1".into()),
        };
        event(n, "anna", DISCHARGED, d.encode().unwrap())
    }

    fn readmitted(n: u64, tag: &str) -> AppliedEvent {
        let r = Readmitted { tag: tag.into() };
        event(n, "clara", READMITTED, r.encode().unwrap())
    }

    #[test]
    fn outcome_ids_are_the_ids_in_events() {
        for outcome in Outcome::ALL {
            let json = serde_json::to_string(&outcome).unwrap();
            assert_eq!(json, format!("\"{}\"", outcome.id()));
            assert_eq!(Outcome::from_id(outcome.id()), Some(outcome));
        }
    }

    // [utest->dsn~uhs-discharge~1]
    #[test]
    fn discharge_frees_the_place_leaves_the_counts_and_readmission_brings_back() {
        let place = PlaceAssigned {
            tag: "DE-0001".into(),
            place: Some("p1".into()),
        };
        let mut events = vec![
            registered(1, "DE-0001", TriageCategory::Immediate),
            registered(2, "DE-0002", TriageCategory::Minor),
            event(3, "anna", PLACE_ASSIGNED, place.encode().unwrap()),
            discharged(4, "DE-0001", Outcome::Transported),
        ];
        let register = PatientRegister::from_events(&events);
        let patient = register.patient("DE-0001").unwrap();
        assert!(!patient.is_present());
        assert_eq!(patient.place, None, "the place is free again");
        assert_eq!(
            patient.discharge().map(|d| &d.change),
            Some(&Stay::Discharged {
                outcome: Outcome::Transported,
                transport: Some("request-1".into()),
            })
        );
        let counts = &register.counts()["uhs-1"];
        assert_eq!(
            (counts.immediate, counts.minor, counts.discharged),
            (0, 1, 1),
            "the load counts the patients in the unit"
        );

        // Discharged by mistake: readmitted, both entries stay.
        events.push(readmitted(5, "DE-0001"));
        events.push(readmitted(6, "DE-0002")); // in the unit: changes nothing
        let register = PatientRegister::from_events(&events);
        let patient = register.patient("DE-0001").unwrap();
        assert!(patient.is_present());
        let stay: Vec<(&Stay, &str)> = patient
            .stay
            .iter()
            .map(|e| (&e.change, e.by.localpart()))
            .collect();
        assert_eq!(stay.len(), 3);
        assert_eq!(
            (stay[0].0, stay[2]),
            (&Stay::Admitted, (&Stay::Admitted, "clara"))
        );
        assert_eq!(register.patient("DE-0002").unwrap().stay.len(), 1);
        let counts = &register.counts()["uhs-1"];
        assert_eq!((counts.immediate, counts.discharged), (1, 0));
    }

    // [utest->dsn~triage-reference~1]
    #[test]
    fn the_algorithm_used_is_recorded_and_older_triage_payloads_still_decode() {
        let r = Registered {
            tag: "DE-0001".into(),
            category: TriageCategory::Urgent,
            uhs: "uhs-1".into(),
            algorithm: Some("prior".into()),
        };
        let register =
            PatientRegister::from_events(&[event(1, "anna", REGISTERED, r.encode().unwrap())]);
        let entry = &register.patient("DE-0001").unwrap().history[0];
        assert_eq!(entry.algorithm.as_deref(), Some("prior"));

        // Schema version 2, before the algorithm was recorded.
        #[derive(Serialize)]
        struct V2 {
            tag: String,
            category: TriageCategory,
            uhs: String,
        }
        let v2 = codec::to_cbor(&V2 {
            tag: "DE-0002".into(),
            category: TriageCategory::Minor,
            uhs: "uhs-1".into(),
        })
        .unwrap();
        let decoded: Registered = codec::from_cbor(&v2).unwrap();
        assert_eq!(decoded.algorithm, None);
    }

    // [utest->dsn~uhs-details~1]
    #[test]
    fn the_latest_location_capacity_and_staff_count_per_field() {
        let shared = |n: u64, event_type: &str, payload: Vec<u8>| {
            testing::event(n, "dora", UHS_MODULE, UHS_COMPARTMENT, event_type, payload)
        };
        let created = UhsCreated {
            id: "uhs-1".into(),
            name: "UHS Nord".into(),
            kind: UhsKind::Uhs,
            capacity: 10,
        };
        let update =
            |location: Option<&str>, capacity: Option<u32>, staff: Option<&[&str]>| UhsUpdated {
                id: "uhs-1".into(),
                location: location.map(Into::into),
                capacity,
                staff: staff.map(|s| s.iter().map(|x| (*x).to_owned()).collect()),
            };
        let mut events = vec![
            shared(1, UHS_CREATED, created.encode().unwrap()),
            shared(
                2,
                UHS_UPDATED,
                update(Some("Zelt 1"), None, Some(&["anna"]))
                    .encode()
                    .unwrap(),
            ),
            shared(
                3,
                UHS_UPDATED,
                update(None, Some(14), None).encode().unwrap(),
            ),
            shared(
                4,
                UHS_UPDATED,
                update(Some("Zelt 2"), None, None).encode().unwrap(),
            ),
        ];
        events.reverse();
        assert_eq!(uhs_list(&events)[0].capacity, 14, "the current capacity");
        let d = &uhs_details(&events)["uhs-1"];
        assert_eq!(d.location.as_deref(), Some("Zelt 2"));
        assert_eq!(d.staff, ["anna"], "a field left out keeps its value");
    }

    // [utest->dsn~patient-pathway~1]
    #[test]
    fn a_patient_moves_between_units_and_counts_where_they_are() {
        let moved = |n: u64, tag: &str, uhs: &str| {
            let m = Moved {
                tag: tag.into(),
                uhs: uhs.into(),
            };
            event(n, "ben", MOVED, m.encode().unwrap())
        };
        let place = PlaceAssigned {
            tag: "DE-0001".into(),
            place: Some("p1".into()),
        };
        let events = vec![
            registered(1, "DE-0001", TriageCategory::Immediate),
            event(2, "anna", PLACE_ASSIGNED, place.encode().unwrap()),
            moved(3, "DE-0001", "treatment-area"),
            moved(4, "DE-0001", "treatment-area"), // already there: nothing
            moved(5, "DE-0404", "treatment-area"),
        ];
        let register = PatientRegister::from_events(&events);
        let p = register.patient("DE-0001").unwrap();
        assert_eq!(p.uhs, "treatment-area");
        assert_eq!(p.place, None, "the place in the old unit is free");
        assert!(p.is_present(), "a move is no discharge");
        assert_eq!(
            p.stay.iter().map(|e| &e.change).collect::<Vec<_>>(),
            [
                &Stay::Admitted,
                &Stay::Moved {
                    from: "uhs-1".into(),
                    to: "treatment-area".into()
                }
            ]
        );
        assert_eq!(p.stay[1].by.localpart(), "ben");
        assert_eq!(register.notices, [Notice::UnknownPatient("DE-0404".into())]);
        let counts = register.counts();
        assert!(
            !counts.contains_key("uhs-1"),
            "counted where the patient is"
        );
        assert_eq!(counts["treatment-area"].immediate, 1);

        // A discharged patient is not moved; after a move the discharge still counts.
        let mut events = events;
        events.push(discharged(6, "DE-0001", Outcome::Transported));
        events.push(moved(7, "DE-0001", "uhs-1"));
        let register = PatientRegister::from_events(&events);
        let p = register.patient("DE-0001").unwrap();
        assert!(!p.is_present());
        assert_eq!(p.uhs, "treatment-area");
    }

    // [utest->dsn~uhs-discharge~1]
    #[test]
    fn discharges_arriving_out_of_order_apply_in_clock_order() {
        // Arriving out of order (offline devices) gives the same result.
        let events = vec![
            discharged(3, "DE-0001", Outcome::Refused),
            registered(1, "DE-0001", TriageCategory::Urgent),
            discharged(5, "DE-0001", Outcome::ReleasedOnSite),
        ];
        let register = PatientRegister::from_events(&events);
        let patient = register.patient("DE-0001").unwrap();
        assert!(register.notices.is_empty());
        assert_eq!(
            patient.discharge().map(|d| &d.change),
            Some(&Stay::Discharged {
                outcome: Outcome::ReleasedOnSite,
                transport: None,
            }),
            "the latest discharge states the outcome"
        );
        assert_eq!(patient.stay.len(), 3);
    }
}
