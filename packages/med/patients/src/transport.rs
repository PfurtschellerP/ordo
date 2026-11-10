//! Module `med.transport`: transport requests from an on-site care unit to the incident
//! command, their assignment and status (TRANS-1, TRANS-3).
//!
//! Requests and updates live in the shared `incident` compartment and carry no patient
//! data: the command plans the vehicle, the UHS hands over the patient. Which patient a
//! request is for is recorded apart, in `med.clinical` ([`TransportLinked`]).

use std::collections::{BTreeMap, HashSet};

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID of requests and updates.
pub const MODULE: &str = "med.transport";
/// Compartment of requests and updates: shared with the incident command.
pub const COMPARTMENT: &str = "incident";
/// Event type: a transport was requested.
pub const REQUESTED: &str = "transport.requested";
/// Event type: a request was assigned or its status moved on.
pub const UPDATED: &str = "transport.updated";
/// Event type (module `med.patients`, compartment `med.clinical`): the patient a request
/// is for.
pub const LINKED: &str = "patients.transportRequested";

/// What a UHS crew can request: from the profile (`med/transport-requests.toml`), never
/// in code.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportCatalogue {
    /// Transport types, smallest first (DE: KTW, RTW, ITW, RTH).
    #[serde(rename = "type", default)]
    pub types: Vec<Choice>,
    /// Priorities, highest first.
    #[serde(rename = "priority", default)]
    pub priorities: Vec<Choice>,
    /// Special needs (physician escort, child, infectious, …).
    #[serde(rename = "need", default)]
    pub needs: Vec<Choice>,
}

/// One entry of the catalogue.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    /// Stable ID, e.g. `rtw`.
    pub id: String,
    /// Label, e.g. "RTW".
    pub label: String,
    /// Longer name, e.g. "Rettungswagen".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A catalogue that can't be used.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CatalogueError {
    /// No transport types or no priorities.
    #[error("the catalogue needs at least one {0}")]
    Empty(&'static str),
    /// An ID appears twice in one list.
    #[error("the {0} ID {1:?} appears twice")]
    Duplicate(&'static str, String),
}

impl TransportCatalogue {
    /// Checks that types and priorities exist and every ID is unique in its list.
    pub fn check(&self) -> Result<(), CatalogueError> {
        for (name, list, required) in [
            ("type", &self.types, true),
            ("priority", &self.priorities, true),
            ("need", &self.needs, false),
        ] {
            if required && list.is_empty() {
                return Err(CatalogueError::Empty(name));
            }
            let mut seen = HashSet::new();
            if let Some(c) = list.iter().find(|c| !seen.insert(c.id.as_str())) {
                return Err(CatalogueError::Duplicate(name, c.id.clone()));
            }
        }
        Ok(())
    }

    /// The label of a type, priority or need ID, or the ID itself if the profile no
    /// longer has it (requests made with an older profile stay readable).
    pub fn label<'a>(list: &'a [Choice], id: &'a str) -> &'a str {
        list.iter()
            .find(|c| c.id == id)
            .map_or(id, |c| c.label.as_str())
    }

    /// Rank of a priority, highest first; unknown priorities come last.
    pub fn priority_rank(&self, id: &str) -> usize {
        self.priorities
            .iter()
            .position(|c| c.id == id)
            .unwrap_or(self.priorities.len())
    }
}

/// Payload of `transport.requested` (TRANS-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportRequested {
    /// New request ID (a ULID).
    pub request: String,
    /// The requesting on-site care unit ([`crate::UhsCreated::id`]).
    pub uhs: String,
    /// Transport type ID from the catalogue.
    pub transport_type: String,
    /// Priority ID from the catalogue.
    pub priority: String,
    /// Special need IDs from the catalogue.
    #[serde(default)]
    pub needs: Vec<String>,
}

/// A request that doesn't fit the catalogue.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum RequestError {
    /// The type, priority or a need isn't in the catalogue.
    #[error("unknown {0} {1:?}")]
    Unknown(&'static str, String),
}

impl TransportRequested {
    /// Checks a new request against the catalogue (reading accepts any ID).
    pub fn check(&self, catalogue: &TransportCatalogue) -> Result<(), RequestError> {
        let known = |list: &[Choice], id: &str| list.iter().any(|c| c.id == id);
        if !known(&catalogue.types, &self.transport_type) {
            return Err(RequestError::Unknown("type", self.transport_type.clone()));
        }
        if !known(&catalogue.priorities, &self.priority) {
            return Err(RequestError::Unknown("priority", self.priority.clone()));
        }
        if let Some(n) = self.needs.iter().find(|n| !known(&catalogue.needs, n)) {
            return Err(RequestError::Unknown("need", n.clone()));
        }
        Ok(())
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Status of a transport (TRANS-3), in the order of the steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransportStatus {
    /// Waiting for a unit.
    Requested,
    /// A unit and a destination are assigned.
    Assigned,
    /// The patient is being loaded.
    Loading,
    /// On the way.
    Departed,
    /// At the destination.
    Arrived,
    /// Handed over to the destination (end).
    HandedOver,
    /// Not needed any more (end).
    Cancelled,
}

impl TransportStatus {
    /// Steps in order, without `Cancelled`.
    pub const STEPS: [Self; 6] = [
        Self::Requested,
        Self::Assigned,
        Self::Loading,
        Self::Departed,
        Self::Arrived,
        Self::HandedOver,
    ];

    /// Whether the transport is still going on.
    pub fn is_open(self) -> bool {
        !matches!(self, Self::HandedOver | Self::Cancelled)
    }

    /// Whether the patient has left with the transport (departed, arrived, handed over).
    pub fn has_left(self) -> bool {
        matches!(self, Self::Departed | Self::Arrived | Self::HandedOver)
    }

    /// The statuses that may follow: later steps (skipping is allowed, radio messages
    /// arrive late) and cancelling, as long as nobody departed. Never a step back.
    pub fn next(self) -> Vec<Self> {
        if !self.is_open() {
            return Vec::new();
        }
        let mut next: Vec<Self> = Self::STEPS.into_iter().filter(|s| *s > self).collect();
        if self < Self::Departed {
            next.push(Self::Cancelled);
        }
        next
    }
}

/// Payload of `transport.updated`: a new status, with the unit and destination when they
/// are assigned or changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportUpdated {
    /// The request ([`TransportRequested::request`]).
    pub request: String,
    /// The new status.
    pub status: TransportStatus,
    /// Call sign of the transport unit; `None` keeps the one assigned before.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Destination (free text until the hospital list, TRANS-2); `None` keeps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
}

impl TransportUpdated {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `patients.transportRequested` (clinical compartment): the patient a
/// request is for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportLinked {
    /// Number of the triage tag.
    pub tag: String,
    /// The request.
    pub request: String,
}

impl TransportLinked {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// One status change of a transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// The status from then on.
    pub status: TransportStatus,
    /// When.
    pub at: Hlc,
    /// Who set it.
    pub by: UserId,
}

/// A transport request with its current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Transport {
    /// The request as made.
    pub request: TransportRequested,
    /// Current status (of the latest update).
    pub status: TransportStatus,
    /// Assigned unit.
    pub unit: Option<String>,
    /// Destination.
    pub destination: Option<String>,
    /// Every status, oldest first; the first is the request itself.
    pub history: Vec<Step>,
}

impl Transport {
    /// When it was requested.
    pub fn requested_at(&self) -> Hlc {
        self.history[0].at
    }
}

/// All transports of an incident, computed from events of the `incident` compartment.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportBoard {
    /// Transports by request ID.
    pub transports: BTreeMap<String, Transport>,
}

impl TransportBoard {
    // [impl->dsn~transport-request~1]
    /// Computes the board. The result doesn't depend on the order of `events`: a request
    /// made twice keeps its first version; updates apply in clock order, the latest
    /// sets the status; updates for unknown requests are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut requests: Vec<(Hlc, UserId, TransportRequested)> = Vec::new();
        let mut updates: Vec<(Hlc, UserId, TransportUpdated)> = Vec::new();
        for e in events.iter().filter(|e| e.body.module.as_str() == MODULE) {
            let who = (e.body.hlc, e.body.author.user.clone());
            match e.body.event_type.as_str() {
                REQUESTED => {
                    if let Ok(r) = codec::from_cbor(&e.payload) {
                        requests.push((who.0, who.1, r));
                    }
                }
                UPDATED => {
                    if let Ok(u) = codec::from_cbor(&e.payload) {
                        updates.push((who.0, who.1, u));
                    }
                }
                _ => {}
            }
        }
        requests.sort_by_key(|(hlc, _, _)| *hlc);
        updates.sort_by_key(|(hlc, _, _)| *hlc);
        let mut transports = BTreeMap::new();
        for (at, by, request) in requests {
            transports
                .entry(request.request.clone())
                .or_insert_with(|| Transport {
                    request,
                    status: TransportStatus::Requested,
                    unit: None,
                    destination: None,
                    history: vec![Step {
                        status: TransportStatus::Requested,
                        at,
                        by,
                    }],
                });
        }
        for (at, by, update) in updates {
            let Some(t) = transports.get_mut(&update.request) else {
                continue;
            };
            if at <= t.requested_at() {
                continue;
            }
            t.status = update.status;
            if update.unit.is_some() {
                t.unit = update.unit;
            }
            if update.destination.is_some() {
                t.destination = update.destination;
            }
            t.history.push(Step {
                status: update.status,
                at,
                by,
            });
        }
        Self { transports }
    }

    /// The open transports: highest priority first, then the oldest.
    pub fn open(&self, catalogue: &TransportCatalogue) -> Vec<&Transport> {
        let mut open: Vec<&Transport> = self
            .transports
            .values()
            .filter(|t| t.status.is_open())
            .collect();
        open.sort_by_key(|t| {
            (
                catalogue.priority_rank(&t.request.priority),
                t.requested_at(),
            )
        });
        open
    }
}

/// The requests of each patient, oldest first, from events of `med.clinical`.
pub fn requests_by_patient(clinical_events: &[AppliedEvent]) -> BTreeMap<String, Vec<String>> {
    let mut links: Vec<(Hlc, TransportLinked)> = clinical_events
        .iter()
        .filter(|e| e.body.module.as_str() == crate::MODULE && e.body.event_type.as_str() == LINKED)
        .filter_map(|e| codec::from_cbor(&e.payload).ok().map(|l| (e.body.hlc, l)))
        .collect();
    links.sort_by_key(|(hlc, _)| *hlc);
    let mut by_patient: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (_, link) in links {
        let list = by_patient.entry(link.tag).or_default();
        if !list.contains(&link.request) {
            list.push(link.request);
        }
    }
    by_patient
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shape of `profiles/de/med/transport-requests.toml`, small.
    fn catalogue() -> TransportCatalogue {
        let choice = |id: &str, label: &str| Choice {
            id: id.into(),
            label: label.into(),
            description: None,
        };
        TransportCatalogue {
            types: vec![choice("ktw", "KTW"), choice("rtw", "RTW")],
            priorities: vec![choice("immediate", "Sofort"), choice("normal", "Normal")],
            needs: vec![choice("child", "Kind"), choice("infectious", "Infektiös")],
        }
    }

    fn event(n: u64, user: &str, module: &str, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        crate::testing::event(n, user, module, COMPARTMENT, event_type, payload)
    }

    fn request(n: u64, id: &str, priority: &str) -> AppliedEvent {
        let r = TransportRequested {
            request: id.into(),
            uhs: "uhs-1".into(),
            transport_type: "rtw".into(),
            priority: priority.into(),
            needs: vec!["infectious".into()],
        };
        event(n, "anna", MODULE, REQUESTED, r.encode().unwrap())
    }

    fn update(
        n: u64,
        user: &str,
        id: &str,
        status: TransportStatus,
        unit: Option<&str>,
    ) -> AppliedEvent {
        let u = TransportUpdated {
            request: id.into(),
            status,
            unit: unit.map(Into::into),
            destination: unit.map(|_| "Klinikum Mitte".into()),
        };
        event(n, user, MODULE, UPDATED, u.encode().unwrap())
    }

    // [utest->dsn~transport-request~1]
    #[test]
    fn a_request_is_assigned_and_moves_through_its_steps() {
        use TransportStatus::*;
        let mut events = vec![
            request(1, "r1", "normal"),
            update(2, "erik", "r1", Assigned, Some("RK 83/1")),
            update(3, "anna", "r1", Loading, None),
            update(4, "erik", "r1", Arrived, None), // "departed" came late over the radio
        ];
        events.reverse(); // order of arrival doesn't matter
        let board = TransportBoard::from_events(&events);
        let t = &board.transports["r1"];
        assert_eq!(t.status, Arrived);
        assert_eq!(
            t.unit.as_deref(),
            Some("RK 83/1"),
            "an update without unit keeps it"
        );
        assert_eq!(t.destination.as_deref(), Some("Klinikum Mitte"));
        let steps: Vec<_> = t
            .history
            .iter()
            .map(|s| (s.status, s.by.as_str().to_owned()))
            .collect();
        assert_eq!(
            steps,
            vec![
                (Requested, "anna@ordo.example.org".to_owned()),
                (Assigned, "erik@ordo.example.org".to_owned()),
                (Loading, "anna@ordo.example.org".to_owned()),
                (Arrived, "erik@ordo.example.org".to_owned()),
            ],
            "every change with who and when"
        );
        assert_eq!(t.request.needs, vec!["infectious"]);
    }

    // [utest->dsn~transport-request~1]
    #[test]
    fn the_open_list_is_by_priority_then_age_and_drops_ended_transports() {
        let c = catalogue();
        let events = vec![
            request(1, "old-normal", "normal"),
            request(2, "immediate", "immediate"),
            request(3, "new-normal", "normal"),
            request(4, "done", "immediate"),
            update(5, "erik", "done", TransportStatus::HandedOver, None),
            update(6, "erik", "unknown", TransportStatus::Assigned, Some("X")),
        ];
        let board = TransportBoard::from_events(&events);
        let open: Vec<_> = board
            .open(&c)
            .iter()
            .map(|t| t.request.request.as_str())
            .collect();
        assert_eq!(open, vec!["immediate", "old-normal", "new-normal"]);
        assert!(
            !board.transports.contains_key("unknown"),
            "an update alone is no request"
        );
    }

    // [utest->dsn~transport-request~1]
    #[test]
    fn steps_only_go_forward_and_cancelling_ends_before_departure() {
        use TransportStatus::*;
        assert_eq!(
            Requested.next(),
            vec![Assigned, Loading, Departed, Arrived, HandedOver, Cancelled]
        );
        assert_eq!(
            Loading.next(),
            vec![Departed, Arrived, HandedOver, Cancelled]
        );
        assert_eq!(
            Departed.next(),
            vec![Arrived, HandedOver],
            "no cancelling on the road"
        );
        assert!(HandedOver.next().is_empty() && Cancelled.next().is_empty());
    }

    // [utest->dsn~transport-request~1]
    #[test]
    fn new_requests_must_fit_the_profile_catalogue() {
        let c = catalogue();
        assert_eq!(c.check(), Ok(()));
        let mut r = TransportRequested {
            request: "r".into(),
            uhs: "uhs-1".into(),
            transport_type: "rtw".into(),
            priority: "normal".into(),
            needs: vec!["child".into()],
        };
        assert_eq!(r.check(&c), Ok(()));
        r.needs.push("hazmat".into());
        assert_eq!(
            r.check(&c),
            Err(RequestError::Unknown("need", "hazmat".into()))
        );
        r.transport_type = "bus".into();
        assert!(r.check(&c).is_err());

        let mut bad = c.clone();
        bad.priorities.clear();
        assert_eq!(bad.check(), Err(CatalogueError::Empty("priority")));
        let mut bad = c;
        bad.types.push(bad.types[0].clone());
        assert_eq!(
            bad.check(),
            Err(CatalogueError::Duplicate("type", "ktw".into()))
        );
    }

    // [utest->dsn~transport-request~1]
    #[test]
    fn the_patient_link_stays_in_the_clinical_events() {
        let link = |n, tag: &str, request: &str| {
            let l = TransportLinked {
                tag: tag.into(),
                request: request.into(),
            };
            event(n, "anna", crate::MODULE, LINKED, l.encode().unwrap())
        };
        let by_patient = requests_by_patient(&[
            link(2, "DE-1", "r2"),
            link(1, "DE-1", "r1"),
            link(3, "DE-2", "r3"),
        ]);
        assert_eq!(by_patient["DE-1"], vec!["r1", "r2"]);
        assert_eq!(by_patient["DE-2"], vec!["r3"]);
        // The shared request has no field for the patient.
        let shared = request(1, "r1", "normal");
        let value: ciborium::Value = ciborium::from_reader(shared.payload.as_slice()).unwrap();
        let keys: Vec<String> = value
            .as_map()
            .unwrap()
            .iter()
            .filter_map(|(k, _)| k.as_text().map(str::to_owned))
            .collect();
        assert_eq!(
            keys,
            vec!["request", "uhs", "transport_type", "priority", "needs"]
        );
    }
}
