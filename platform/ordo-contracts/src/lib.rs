//! Connector contracts (architecture chapter 8.3).
//!
//! A connector is a versioned contract for information that crosses a service-package
//! border. Consuming packages depend only on this crate, never on the providing package.
//! Contracts contain the minimum necessary: counts and statuses, never clinical details.

use serde::{Deserialize, Serialize};

/// Core meaning of a triage category. Profiles map their labels (e.g. "SK I") to these.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TriageCategory {
    /// Needs immediate treatment (DE: SK I, red).
    Immediate,
    /// Urgent (DE: SK II, yellow).
    Urgent,
    /// Minor injuries (DE: SK III, green).
    Minor,
    /// Expectant (DE: SK IV, blue).
    Expectant,
    /// Deceased.
    Deceased,
    // [impl->dsn~transport-priority~1]
    /// Unharmed but affected: registered for care, information and tracking, no medical
    /// treatment needed (DE: "Betroffene"). Since schema version 2 of the patient events.
    Uninjured,
}

/// `med.patientCounts.v1`: number of patients per triage category in one on-site care
/// unit. Published by the Med package into the shared `incident` compartment.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientCountsV1 {
    /// The on-site care unit (UHS, treatment area, collection point).
    pub uhs: String,
    /// Patients in category immediate.
    pub immediate: u32,
    /// Patients in category urgent.
    pub urgent: u32,
    /// Patients in category minor.
    pub minor: u32,
    /// Patients in category expectant.
    pub expectant: u32,
    /// Deceased.
    pub deceased: u32,
    /// Unharmed but affected (added later; absent in older counts).
    #[serde(default)]
    pub uninjured: u32,
    /// Patients discharged from the unit (added later; absent in older counts). The
    /// categories count only the patients in the unit.
    #[serde(default)]
    pub discharged: u32,
}

impl PatientCountsV1 {
    /// Module that publishes the contract.
    pub const MODULE: &'static str = "med.patients";
    /// Event type of the contract.
    pub const EVENT_TYPE: &'static str = "patientCounts.v1";
    /// Compartment the contract is published to.
    pub const COMPARTMENT: &'static str = "incident";

    /// Increments the counter of `category`.
    pub fn add(&mut self, category: TriageCategory) {
        *self.counter(category) += 1;
    }

    /// The count of `category`.
    pub fn get(&self, category: TriageCategory) -> u32 {
        match category {
            TriageCategory::Immediate => self.immediate,
            TriageCategory::Urgent => self.urgent,
            TriageCategory::Minor => self.minor,
            TriageCategory::Expectant => self.expectant,
            TriageCategory::Deceased => self.deceased,
            TriageCategory::Uninjured => self.uninjured,
        }
    }

    fn counter(&mut self, category: TriageCategory) -> &mut u32 {
        match category {
            TriageCategory::Immediate => &mut self.immediate,
            TriageCategory::Urgent => &mut self.urgent,
            TriageCategory::Minor => &mut self.minor,
            TriageCategory::Expectant => &mut self.expectant,
            TriageCategory::Deceased => &mut self.deceased,
            TriageCategory::Uninjured => &mut self.uninjured,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~transport-priority~1]
    #[test]
    fn unharmed_people_are_counted_apart_and_old_counts_still_decode() {
        let mut counts = PatientCountsV1::default();
        counts.add(TriageCategory::Uninjured);
        counts.add(TriageCategory::Minor);
        assert_eq!(counts.get(TriageCategory::Uninjured), 1);
        assert_eq!(counts.get(TriageCategory::Minor), 1);

        // Counts published before `uninjured` existed.
        #[derive(Serialize)]
        struct Old {
            uhs: String,
            immediate: u32,
            urgent: u32,
            minor: u32,
            expectant: u32,
            deceased: u32,
        }
        let mut bytes = Vec::new();
        let old = Old {
            uhs: "UHS 1".into(),
            immediate: 1,
            urgent: 0,
            minor: 2,
            expectant: 0,
            deceased: 0,
        };
        ciborium::into_writer(&old, &mut bytes).unwrap();
        let decoded: PatientCountsV1 = ciborium::from_reader(bytes.as_slice()).unwrap();
        assert_eq!((decoded.minor, decoded.uninjured), (2, 0));
    }
}
