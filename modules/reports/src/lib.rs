//! Shared module `reports` (REP-1…5, ADR-0075): events and patient cases reported to the
//! incident command, with place and a short description. Shared incident data, no clinical
//! data: a patient registered from a case is named only by the unit that took it over.

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "reports";
/// Compartment: shared with the incident.
pub const COMPARTMENT: &str = "incident";
/// Event type: a report was recorded (REP-1).
pub const RECORDED: &str = "reports.recorded";
/// Event type: the incident command closed a report (REP-5).
pub const CLOSED: &str = "reports.closed";
/// Event type: a unit's crew registered the patient of a patient case (REP-3).
pub const PATIENT_REGISTERED: &str = "reports.patientRegistered";

/// Longest place, in characters.
pub const PLACE_MAX: usize = 120;
/// Longest description, in characters: a report is short.
pub const TEXT_MAX: usize = 280;

/// What was reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReportKind {
    /// Something happening on the site, e.g. a fight at a gate.
    Event,
    /// A person who needs help: becomes a patient (REP-3).
    Patient,
}

/// Payload of `reports.recorded` (REP-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recorded {
    /// Report ID (a ULID, made when it is recorded).
    pub report: String,
    /// Event or patient case.
    pub kind: ReportKind,
    /// Where, as said, e.g. "Bühne Nord, links vom Eingang".
    pub place: String,
    /// What, in short; no clinical data.
    pub text: String,
    /// When the report came in (Unix time in milliseconds).
    #[serde(rename = "receivedAt")]
    pub received_at: u64,
}

/// A report that can't be recorded.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReportError {
    /// No place.
    #[error("the report needs a place")]
    NoPlace,
    /// No description.
    #[error("the report needs a description")]
    NoText,
    /// Longer than a report is.
    #[error("the place or the description is too long")]
    TooLong,
}

impl Recorded {
    /// Checks a new report: it needs a place and a description, both short.
    pub fn check(&self) -> Result<(), ReportError> {
        if self.place.trim().is_empty() {
            return Err(ReportError::NoPlace);
        }
        if self.text.trim().is_empty() {
            return Err(ReportError::NoText);
        }
        if self.place.chars().count() > PLACE_MAX || self.text.chars().count() > TEXT_MAX {
            return Err(ReportError::TooLong);
        }
        Ok(())
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `reports.closed` (REP-5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closed {
    /// The report.
    pub report: String,
}

impl Closed {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `reports.patientRegistered` (REP-3): only the case and the unit, never the
/// triage tag, the category or anything else about the patient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientRegistered {
    /// The patient case.
    pub report: String,
    /// The unit whose crew took the patient over (call sign).
    pub unit: String,
}

impl PatientRegistered {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// A patient taken over from a case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TakenOver {
    /// By which unit.
    pub unit: String,
    /// When.
    pub at: Hlc,
    /// Who registered the patient.
    pub by: UserId,
}

/// A report with its current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// As recorded.
    pub recorded: Recorded,
    /// When it was recorded.
    pub at: Hlc,
    /// Who recorded it.
    pub by: UserId,
    /// When and by whom it was closed.
    pub closed: Option<(Hlc, UserId)>,
    /// Patients registered from it (patient cases only), oldest first.
    pub patients: Vec<TakenOver>,
}

impl Report {
    /// Whether it is still open.
    pub fn is_open(&self) -> bool {
        self.closed.is_none()
    }
}

/// All reports of an incident, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReportList {
    /// Reports by ID.
    pub reports: BTreeMap<String, Report>,
}

enum Change {
    Recorded(Recorded),
    Closed(Closed),
    Patient(PatientRegistered),
}

impl ReportList {
    // [impl->dsn~reports~1]
    /// Computes the list. The result doesn't depend on the order of `events`: changes
    /// apply in clock order, a report recorded twice keeps its first version, and changes
    /// of unknown reports are ignored. A patient counts only for a patient case; it counts
    /// after the case was closed too, since a crew may have been offline.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, UserId, Change)> = Vec::new();
        for e in events.iter().filter(|e| e.body.module.as_str() == MODULE) {
            let change = match e.body.event_type.as_str() {
                RECORDED => codec::from_cbor(&e.payload).ok().map(Change::Recorded),
                CLOSED => codec::from_cbor(&e.payload).ok().map(Change::Closed),
                PATIENT_REGISTERED => codec::from_cbor(&e.payload).ok().map(Change::Patient),
                _ => None,
            };
            if let Some(change) = change {
                changes.push((e.body.hlc, e.body.author.user.clone(), change));
            }
        }
        changes.sort_by_key(|(at, _, _)| *at);
        let mut reports: BTreeMap<String, Report> = BTreeMap::new();
        for (at, by, change) in changes {
            match change {
                Change::Recorded(r) => {
                    if !reports.contains_key(&r.report) {
                        reports.insert(
                            r.report.clone(),
                            Report {
                                recorded: r,
                                at,
                                by,
                                closed: None,
                                patients: Vec::new(),
                            },
                        );
                    }
                }
                Change::Closed(c) => {
                    if let Some(r) = reports.get_mut(&c.report)
                        && r.closed.is_none()
                    {
                        r.closed = Some((at, by));
                    }
                }
                Change::Patient(p) => {
                    if let Some(r) = reports.get_mut(&p.report)
                        && r.recorded.kind == ReportKind::Patient
                    {
                        r.patients.push(TakenOver {
                            unit: p.unit,
                            at,
                            by,
                        });
                    }
                }
            }
        }
        Self { reports }
    }

    // [impl->dsn~reports~1]
    /// The open reports, oldest first by the time they came in.
    pub fn open(&self) -> Vec<&Report> {
        let mut open: Vec<&Report> = self.reports.values().filter(|r| r.is_open()).collect();
        open.sort_by_key(|r| (r.recorded.received_at, r.at));
        open
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, IncidentId, ServerName, Ulid};

    fn event(n: u64, user: &str, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
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
                module: MODULE.parse().expect("valid"),
                compartment: COMPARTMENT.parse().expect("valid"),
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

    fn recorded(n: u64, id: &str, kind: ReportKind, received_at: u64) -> AppliedEvent {
        let r = Recorded {
            report: id.into(),
            kind,
            place: format!("Ort {id}"),
            text: format!("Meldung {id}"),
            received_at,
        };
        event(n, "ben", RECORDED, r.encode().unwrap())
    }

    fn closed(n: u64, id: &str) -> AppliedEvent {
        let c = Closed { report: id.into() };
        event(n, "ben", CLOSED, c.encode().unwrap())
    }

    fn patient(n: u64, id: &str, unit: &str) -> AppliedEvent {
        let p = PatientRegistered {
            report: id.into(),
            unit: unit.into(),
        };
        event(n, "anna", PATIENT_REGISTERED, p.encode().unwrap())
    }

    // [utest->dsn~reports~1]
    #[test]
    fn a_report_needs_a_short_place_and_description() {
        let ok = Recorded {
            report: "r1".into(),
            kind: ReportKind::Patient,
            place: "Bühne Nord".into(),
            text: "Person liegt am Boden".into(),
            received_at: 1,
        };
        assert_eq!(ok.check(), Ok(()));
        let place = Recorded {
            place: " ".into(),
            ..ok.clone()
        };
        assert_eq!(place.check(), Err(ReportError::NoPlace));
        let text = Recorded {
            text: String::new(),
            ..ok.clone()
        };
        assert_eq!(text.check(), Err(ReportError::NoText));
        let long = Recorded {
            text: "x".repeat(TEXT_MAX + 1),
            ..ok
        };
        assert_eq!(long.check(), Err(ReportError::TooLong));
    }

    // [utest->dsn~reports~1]
    #[test]
    fn open_reports_oldest_first_and_closed_ones_kept() {
        let events = vec![
            recorded(1, "r1", ReportKind::Event, 500),
            recorded(2, "r2", ReportKind::Patient, 400),
            recorded(3, "r3", ReportKind::Event, 600),
            closed(4, "r3"),
            closed(5, "unknown"),
        ];
        let list = ReportList::from_events(&events);
        let open: Vec<&str> = list
            .open()
            .iter()
            .map(|r| r.recorded.report.as_str())
            .collect();
        assert_eq!(open, ["r2", "r1"], "by the time they came in");
        assert!(!list.reports["r3"].is_open(), "closed, still there");
        assert_eq!(list.reports.len(), 3);
    }

    // [utest->dsn~reports~1]
    #[test]
    fn the_result_does_not_depend_on_the_order_of_events() {
        let mut events = vec![
            recorded(1, "r1", ReportKind::Patient, 500),
            patient(2, "r1", "SanTrp 1"),
            closed(3, "r1"),
            closed(4, "r1"),
        ];
        let a = ReportList::from_events(&events);
        events.reverse();
        let b = ReportList::from_events(&events);
        assert_eq!(a, b);
        let r = &a.reports["r1"];
        assert_eq!(r.closed.as_ref().map(|(at, _)| at.wall_ms), Some(1003));
        assert_eq!(r.patients.len(), 1);
        assert_eq!(r.patients[0].unit, "SanTrp 1");
    }

    // [utest->dsn~reports~1]
    #[test]
    fn a_report_recorded_twice_keeps_its_first_version() {
        let second = Recorded {
            report: "r1".into(),
            kind: ReportKind::Event,
            place: "anders".into(),
            text: "anders".into(),
            received_at: 1,
        };
        let events = vec![
            recorded(1, "r1", ReportKind::Patient, 500),
            event(2, "ben", RECORDED, second.encode().unwrap()),
        ];
        let list = ReportList::from_events(&events);
        assert_eq!(list.reports["r1"].recorded.kind, ReportKind::Patient);
        assert_eq!(list.reports["r1"].recorded.place, "Ort r1");
    }

    // [utest->dsn~reports~1]
    #[test]
    fn only_a_patient_case_takes_a_patient() {
        let events = vec![
            recorded(1, "r1", ReportKind::Event, 500),
            patient(2, "r1", "SanTrp 1"),
            patient(3, "r9", "SanTrp 1"),
        ];
        let list = ReportList::from_events(&events);
        assert!(list.reports["r1"].patients.is_empty());
        assert_eq!(list.reports.len(), 1, "no report from a patient alone");
    }

    // [utest->dsn~reports~1]
    #[test]
    fn the_patient_event_names_only_the_case_and_the_unit() {
        let p = PatientRegistered {
            report: "r1".into(),
            unit: "SanTrp 1".into(),
        };
        let fields: BTreeMap<String, String> = codec::from_cbor(&p.encode().unwrap()).unwrap();
        assert_eq!(fields.keys().collect::<Vec<_>>(), ["report", "unit"]);
    }
}
