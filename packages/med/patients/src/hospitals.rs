//! Module `med.hospitals`: the hospitals of an incident with departments, capabilities
//! and current capacity (TRANS-2). Shared incident data, kept by the incident command; it
//! carries no patient data.

use std::collections::BTreeMap;

use ordo_contracts::TriageCategory;
use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

use crate::transport::{CatalogueError, Choice, check_lists};

/// Module ID.
pub const MODULE: &str = "med.hospitals";
/// Compartment: shared with everyone in the incident.
pub const COMPARTMENT: &str = "incident";
/// Event type: a hospital was added, or its entry replaced.
pub const SET: &str = "hospitals.set";
/// Event type: a hospital's current capacity was entered.
pub const CAPACITY_SET: &str = "hospitals.capacitySet";

/// What can be recorded about a hospital: from the profile (`med/hospitals.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HospitalCatalogue {
    /// Triage categories for which hospitals report capacity.
    #[serde(default)]
    pub capacity: Vec<TriageCategory>,
    /// Departments (trauma surgery, paediatrics, …).
    #[serde(rename = "department", default)]
    pub departments: Vec<Choice>,
    /// Capabilities (shock room, stroke unit, burns centre, …).
    #[serde(rename = "capability", default)]
    pub capabilities: Vec<Choice>,
}

impl HospitalCatalogue {
    /// Checks that there are capacity categories and every ID is unique in its list.
    pub fn check(&self) -> Result<(), CatalogueError> {
        if self.capacity.is_empty() {
            return Err(CatalogueError::Empty("capacity category"));
        }
        check_lists(&[
            ("department", &self.departments, false),
            ("capability", &self.capabilities, false),
        ])
    }
}

/// Payload of `hospitals.set`: the whole entry of a hospital.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HospitalSet {
    /// Hospital ID (a ULID, made when the hospital is added).
    pub hospital: String,
    /// Name, e.g. "Klinikum Mitte".
    pub name: String,
    /// Department IDs from the profile.
    #[serde(default)]
    pub departments: Vec<String>,
    /// Capability IDs from the profile.
    #[serde(default)]
    pub capabilities: Vec<String>,
}

/// An entry that can't be recorded.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HospitalError {
    /// No name.
    #[error("the hospital needs a name")]
    NoName,
    /// A department or capability the profile doesn't have.
    #[error("unknown {0} {1:?}")]
    Unknown(&'static str, String),
}

impl HospitalSet {
    /// Checks the name and that departments and capabilities are in the catalogue.
    pub fn check(&self, catalogue: &HospitalCatalogue) -> Result<(), HospitalError> {
        if self.name.trim().is_empty() {
            return Err(HospitalError::NoName);
        }
        for (what, ids, list) in [
            ("department", &self.departments, &catalogue.departments),
            ("capability", &self.capabilities, &catalogue.capabilities),
        ] {
            if let Some(id) = ids.iter().find(|id| !list.iter().any(|c| &c.id == *id)) {
                return Err(HospitalError::Unknown(what, id.clone()));
            }
        }
        Ok(())
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Patients a hospital takes now in one triage category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Capacity {
    /// Triage category.
    pub category: TriageCategory,
    /// Number of patients.
    pub patients: u32,
}

/// Payload of `hospitals.capacitySet`: the capacity as the hospital reports it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapacitySet {
    /// Hospital ID.
    pub hospital: String,
    /// Capacity per category; categories left out are unknown.
    pub capacity: Vec<Capacity>,
}

impl CapacitySet {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// The capacity of a hospital, with who entered it and when.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reported {
    /// Capacity per category.
    pub capacity: Vec<Capacity>,
    /// When it was entered.
    pub at: Hlc,
    /// Who entered it.
    pub by: UserId,
}

impl Reported {
    /// The capacity of one category, if reported.
    pub fn of(&self, category: TriageCategory) -> Option<u32> {
        self.capacity
            .iter()
            .find(|c| c.category == category)
            .map(|c| c.patients)
    }
}

/// A hospital of the incident.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hospital {
    /// The latest entry.
    pub entry: HospitalSet,
    /// The latest capacity, if any was entered.
    pub capacity: Option<Reported>,
}

/// All hospitals of an incident, computed from events of the `incident` compartment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HospitalList {
    /// Hospitals by ID.
    pub hospitals: BTreeMap<String, Hospital>,
}

impl HospitalList {
    // [impl->dsn~hospital-list~1]
    /// Computes the list. The result doesn't depend on the order of `events`: the latest
    /// entry and the latest capacity of each hospital count (by clock); capacities for
    /// hospitals never added are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut sets: Vec<(Hlc, HospitalSet)> = Vec::new();
        let mut capacities: Vec<(Hlc, UserId, CapacitySet)> = Vec::new();
        for e in events.iter().filter(|e| e.body.module.as_str() == MODULE) {
            match e.body.event_type.as_str() {
                SET => {
                    if let Ok(s) = codec::from_cbor(&e.payload) {
                        sets.push((e.body.hlc, s));
                    }
                }
                CAPACITY_SET => {
                    if let Ok(c) = codec::from_cbor(&e.payload) {
                        capacities.push((e.body.hlc, e.body.author.user.clone(), c));
                    }
                }
                _ => {}
            }
        }
        sets.sort_by_key(|(at, _)| *at);
        capacities.sort_by_key(|(at, _, _)| *at);
        let mut hospitals: BTreeMap<String, Hospital> = BTreeMap::new();
        for (_, entry) in sets {
            match hospitals.get_mut(&entry.hospital) {
                Some(h) => h.entry = entry,
                None => {
                    hospitals.insert(
                        entry.hospital.clone(),
                        Hospital {
                            entry,
                            capacity: None,
                        },
                    );
                }
            }
        }
        for (at, by, c) in capacities {
            if let Some(h) = hospitals.get_mut(&c.hospital) {
                h.capacity = Some(Reported {
                    capacity: c.capacity,
                    at,
                    by,
                });
            }
        }
        Self { hospitals }
    }

    /// The hospitals sorted by name.
    pub fn by_name(&self) -> Vec<&Hospital> {
        let mut list: Vec<&Hospital> = self.hospitals.values().collect();
        list.sort_by(|a, b| a.entry.name.cmp(&b.entry.name));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(n: u64, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        crate::testing::event(n, "ben", MODULE, COMPARTMENT, event_type, payload)
    }

    fn catalogue() -> HospitalCatalogue {
        HospitalCatalogue {
            capacity: vec![TriageCategory::Immediate, TriageCategory::Urgent],
            departments: vec![Choice {
                id: "trauma".into(),
                label: "Unfallchirurgie".into(),
                description: None,
            }],
            capabilities: Vec::new(),
        }
    }

    fn set(n: u64, id: &str, name: &str) -> AppliedEvent {
        let s = HospitalSet {
            hospital: id.into(),
            name: name.into(),
            departments: vec!["trauma".into()],
            capabilities: Vec::new(),
        };
        event(n, SET, s.encode().unwrap())
    }

    fn capacity(n: u64, id: &str, immediate: u32) -> AppliedEvent {
        let c = CapacitySet {
            hospital: id.into(),
            capacity: vec![Capacity {
                category: TriageCategory::Immediate,
                patients: immediate,
            }],
        };
        event(n, CAPACITY_SET, c.encode().unwrap())
    }

    // [utest->dsn~hospital-list~1]
    #[test]
    fn the_latest_entry_and_capacity_count_in_any_order() {
        let events = vec![
            capacity(5, "h1", 1),
            set(1, "h1", "Klinikum Mitte"),
            capacity(3, "h1", 4),
            set(2, "h2", "Kreisklinik"),
            set(4, "h1", "Klinikum Mitte (Haus A)"),
            capacity(6, "unknown", 9),
        ];
        let list = HospitalList::from_events(&events);
        assert_eq!(list.hospitals.len(), 2, "capacity alone adds no hospital");
        let h1 = &list.hospitals["h1"];
        assert_eq!(h1.entry.name, "Klinikum Mitte (Haus A)");
        let reported = h1.capacity.as_ref().unwrap();
        assert_eq!(reported.of(TriageCategory::Immediate), Some(1));
        assert_eq!(reported.of(TriageCategory::Urgent), None, "not reported");
        assert_eq!(reported.by.localpart(), "ben");
        let names: Vec<&str> = list
            .by_name()
            .iter()
            .map(|h| h.entry.name.as_str())
            .collect();
        assert_eq!(names, ["Klinikum Mitte (Haus A)", "Kreisklinik"]);
    }

    // [utest->dsn~hospital-list~1]
    #[test]
    fn entries_need_a_name_and_known_departments() {
        let c = catalogue();
        assert!(c.check().is_ok());
        let mut s = HospitalSet {
            hospital: "h1".into(),
            name: "Klinikum Mitte".into(),
            departments: vec!["trauma".into()],
            capabilities: Vec::new(),
        };
        assert_eq!(s.check(&c), Ok(()));
        s.departments.push("dentistry".into());
        assert_eq!(
            s.check(&c),
            Err(HospitalError::Unknown("department", "dentistry".into()))
        );
        s.name = " ".into();
        assert_eq!(s.check(&c), Err(HospitalError::NoName));
        assert!(HospitalCatalogue::default().check().is_err());
    }
}
