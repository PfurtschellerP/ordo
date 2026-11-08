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

/// Module ID.
pub const MODULE: &str = "med.patients";
/// Compartment of patient events.
pub const COMPARTMENT: &str = "med.clinical";
/// Event type: a patient was registered.
pub const REGISTERED: &str = "patients.registered";
/// Event type: a patient was re-triaged.
pub const RETRIAGED: &str = "patients.retriaged";

/// Payload of `patients.registered`: the minimum needed (requirement PAT-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registered {
    /// Number of the triage tag.
    pub tag: String,
    /// Triage category.
    pub category: TriageCategory,
    /// On-site care unit.
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
}

impl Patient {
    /// The current triage category (the latest decision).
    pub fn category(&self) -> TriageCategory {
        self.history
            .last()
            .expect("history is never empty")
            .category
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
