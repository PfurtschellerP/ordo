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

pub mod questionnaire;
pub mod record;
pub mod transport;

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

// [impl->dsn~uhs-registry~1]
/// The on-site care units of an incident, in order of creation (first wins per ID).
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
    created
        .into_iter()
        .map(|(_, u)| u)
        .filter(|u| seen.insert(u.id.clone()))
        .collect()
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

/// Schema version of `patients.registered` and `patients.retriaged`: 2 since the category
/// `uninjured` ("Betroffene") exists; version 1 payloads decode unchanged.
pub const TRIAGE_SCHEMA: u32 = 2;

/// Payload of `patients.registered`: the minimum needed (requirement PAT-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registered {
    /// Number of the triage tag.
    pub tag: String,
    /// Triage category.
    pub category: TriageCategory,
    /// ID of the on-site care unit ([`UhsCreated::id`]).
    pub uhs: String,
}

/// Payload of `patients.retriaged`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Retriaged {
    /// Number of the triage tag.
    pub tag: String,
    /// New triage category.
    pub category: TriageCategory,
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
}

impl Patient {
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
                        at,
                        by,
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
                                },
                            );
                        }
                    }
                }
                Change::Retriaged(r) => match register.patients.get_mut(&r.tag) {
                    Some(patient) => patient.history.push(TriageEntry {
                        category: r.category,
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

    /// Patient counts per on-site care unit, sorted by unit.
    pub fn counts(&self) -> BTreeMap<String, PatientCountsV1> {
        let mut counts: BTreeMap<String, PatientCountsV1> = BTreeMap::new();
        for patient in self.patients.values() {
            counts
                .entry(patient.uhs.clone())
                .or_insert_with(|| PatientCountsV1 {
                    uhs: patient.uhs.clone(),
                    ..Default::default()
                })
                .add(patient.category());
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
