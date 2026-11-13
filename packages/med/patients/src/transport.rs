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
use ordo_units::{StatusCatalogue, Unit, UnitRegister};
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

/// Checks lists of choices: `(name, list, required)`; IDs must be unique in each list.
pub(crate) fn check_lists(lists: &[(&'static str, &[Choice], bool)]) -> Result<(), CatalogueError> {
    for (name, list, required) in lists {
        if *required && list.is_empty() {
            return Err(CatalogueError::Empty(name));
        }
        let mut seen = HashSet::new();
        if let Some(c) = list.iter().find(|c| !seen.insert(c.id.as_str())) {
            return Err(CatalogueError::Duplicate(name, c.id.clone()));
        }
    }
    Ok(())
}

impl TransportCatalogue {
    /// Checks that types and priorities exist and every ID is unique in its list.
    pub fn check(&self) -> Result<(), CatalogueError> {
        check_lists(&[
            ("type", &self.types, true),
            ("priority", &self.priorities, true),
            ("need", &self.needs, false),
        ])
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

// [impl->dsn~epcr-diagnosis~1]
/// Payload of `transport.requested` (TRANS-1), in the `incident` compartment: no patient
/// data, so no diagnosis either (EPCR-11); the diagnosis stays in the record.
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
    /// When the transport is planned for (Unix time in milliseconds); `None` is now
    /// (TRANS-5, schema version 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planned: Option<u64>,
}

/// Schema version of `transport.requested`: 2 since a request can be planned; version 1
/// payloads decode unchanged.
pub const REQUESTED_SCHEMA: u32 = 2;

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
    /// Destination as shown: the hospital's name or free text; `None` keeps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<String>,
    /// The hospital of the list (TRANS-2), if the destination is one; set together with
    /// `destination` (schema version 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hospital: Option<String>,
}

/// Schema version of `transport.updated`: 2 since it names the hospital; version 1
/// payloads decode unchanged.
pub const UPDATED_SCHEMA: u32 = 2;

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
    /// The hospital of the list, if the destination is one.
    pub hospital: Option<String>,
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
                    hospital: None,
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
                t.hospital = update.hospital;
            }
            t.history.push(Step {
                status: update.status,
                at,
                by,
            });
        }
        Self { transports }
    }

    // [impl->dsn~transport-board~1]
    /// Patients sent to each hospital of the list: transports to it that have departed
    /// (and so were not cancelled).
    pub fn sent_to_hospitals(&self) -> BTreeMap<String, usize> {
        let mut sent = BTreeMap::new();
        for t in self.transports.values().filter(|t| t.status.has_left()) {
            if let Some(h) = &t.hospital {
                *sent.entry(h.clone()).or_insert(0) += 1;
            }
        }
        sent
    }

    // [impl->dsn~transport-board~1]
    /// The units free for a transport: present (not released), of a transport type of the
    /// profile (by label or ID, ignoring case), in a status the profile marks as available,
    /// and on no open transport.
    pub fn available_units<'a>(
        &self,
        units: &'a UnitRegister,
        statuses: &StatusCatalogue,
        catalogue: &TransportCatalogue,
    ) -> Vec<&'a Unit> {
        let busy: HashSet<&str> = self
            .transports
            .values()
            .filter(|t| t.status.is_open())
            .filter_map(|t| t.unit.as_deref())
            .collect();
        units
            .present()
            .filter(|u| {
                catalogue.types.iter().any(|t| {
                    t.label.eq_ignore_ascii_case(&u.unit_type)
                        || t.id.eq_ignore_ascii_case(&u.unit_type)
                })
            })
            .filter(|u| {
                u.status
                    .as_ref()
                    .is_some_and(|s| statuses.is_available(&s.status))
            })
            .filter(|u| !busy.contains(u.unit.as_str()))
            .collect()
    }

    /// The open transports wanted now (not planned): highest priority first, then the
    /// oldest.
    pub fn open(&self, catalogue: &TransportCatalogue) -> Vec<&Transport> {
        let mut open: Vec<&Transport> = self
            .transports
            .values()
            .filter(|t| t.status.is_open() && t.request.planned.is_none())
            .collect();
        open.sort_by_key(|t| {
            (
                catalogue.priority_rank(&t.request.priority),
                t.requested_at(),
            )
        });
        open
    }

    // [impl->dsn~planned-transports~1]
    /// The open planned transports: the earliest first, then the oldest request.
    pub fn planned(&self) -> Vec<&Transport> {
        let mut planned: Vec<&Transport> = self
            .transports
            .values()
            .filter(|t| t.status.is_open() && t.request.planned.is_some())
            .collect();
        planned.sort_by_key(|t| (t.request.planned, t.requested_at()));
        planned
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
            planned: None,
        };
        event(n, "anna", MODULE, REQUESTED, r.encode().unwrap())
    }

    fn planned(n: u64, id: &str, at: u64) -> AppliedEvent {
        let r = TransportRequested {
            request: id.into(),
            uhs: "uhs-1".into(),
            transport_type: "ktw".into(),
            priority: "normal".into(),
            needs: Vec::new(),
            planned: Some(at),
        };
        event(n, "anna", MODULE, REQUESTED, r.encode().unwrap())
    }

    // [utest->dsn~planned-transports~1]
    #[test]
    fn planned_transports_are_listed_apart_by_time_and_old_requests_still_decode() {
        let events = vec![
            request(1, "now", "normal"),
            planned(2, "late", 90_000),
            planned(3, "early", 60_000),
            planned(4, "done", 30_000),
            update(5, "erik", "done", TransportStatus::Cancelled, None),
        ];
        let board = TransportBoard::from_events(&events);
        let ids = |list: Vec<&Transport>| -> Vec<String> {
            list.iter().map(|t| t.request.request.clone()).collect()
        };
        assert_eq!(ids(board.open(&catalogue())), ["now"]);
        assert_eq!(
            ids(board.planned()),
            ["early", "late"],
            "the earliest first, open ones only"
        );

        // A version 1 payload has no `planned`: it is wanted now.
        #[derive(Serialize)]
        struct V1 {
            request: String,
            uhs: String,
            transport_type: String,
            priority: String,
            needs: Vec<String>,
        }
        let v1 = codec::to_cbor(&V1 {
            request: "old".into(),
            uhs: "uhs-1".into(),
            transport_type: "rtw".into(),
            priority: "normal".into(),
            needs: Vec::new(),
        })
        .unwrap();
        let decoded: TransportRequested = codec::from_cbor(&v1).unwrap();
        assert_eq!(decoded.planned, None);
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
            hospital: unit.map(|_| "h1".into()),
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

    // [utest->dsn~transport-board~1]
    #[test]
    fn the_board_counts_patients_sent_and_lists_free_transport_units() {
        use TransportStatus::*;
        let events = vec![
            request(1, "r1", "normal"),
            update(2, "erik", "r1", Assigned, Some("RTW 1")),
            request(3, "r2", "normal"),
            update(4, "erik", "r2", Assigned, Some("RTW 2")),
            update(5, "erik", "r2", Departed, None),
            request(6, "r3", "normal"),
            update(7, "erik", "r3", Assigned, Some("RTW 3")),
            update(8, "erik", "r3", HandedOver, None),
        ];
        let board = TransportBoard::from_events(&events);
        assert_eq!(
            board.sent_to_hospitals(),
            BTreeMap::from([("h1".to_owned(), 2)]),
            "assigned is not sent yet; departed and handed over are"
        );

        let unit = |name: &str, unit_type: &str, status: Option<&str>| {
            let at = events[0].body.hlc;
            let by = events[0].body.author.user.clone();
            (
                name.to_owned(),
                Unit {
                    unit: name.into(),
                    unit_type: unit_type.into(),
                    organisation: "DRK".into(),
                    service: None,
                    strength: None,
                    special: Default::default(),
                    status: status.map(|s| ordo_units::UnitStatus {
                        status: s.into(),
                        since: at,
                        by: by.clone(),
                    }),
                    crew: Vec::new(),
                    presence: if name.starts_with("RTW 7") {
                        vec![ordo_units::PresenceEntry {
                            change: ordo_units::Presence::Released,
                            at,
                            by,
                        }]
                    } else {
                        Vec::new()
                    },
                },
            )
        };
        let units = UnitRegister {
            units: BTreeMap::from([
                unit("RTW 1", "RTW", Some("2")), // on an open transport
                unit("RTW 2", "rtw", Some("1")), // on the road, but its status says free
                unit("RTW 3", "RTW", Some("2")), // handed over: free again
                unit("KTW 4", "KTW", Some("6")), // not available
                unit("HLF 5", "HLF", Some("2")), // no transport type
                unit("RTW 6", "RTW", None),      // no status yet
                unit("RTW 7", "RTW", Some("2")), // released
            ]),
            lost_updates: Vec::new(),
        };
        let statuses = StatusCatalogue {
            codes: ["1", "2", "6"]
                .map(|c| ordo_units::StatusCode {
                    code: c.into(),
                    label: c.into(),
                    name: c.into(),
                    available: c != "6",
                })
                .into(),
        };
        let free: Vec<&str> = board
            .available_units(&units, &statuses, &catalogue())
            .iter()
            .map(|u| u.unit.as_str())
            .collect();
        assert_eq!(free, ["RTW 3"]);
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
            planned: None,
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
