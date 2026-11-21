//! A UHS asks for a physician or a paramedic (UHS-15): the request goes to the incident
//! command, which puts a person on the UHS's staff list, giving them access to its patients
//! (UHS-10), and closes the request. Shared incident data: no patient data.

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "med.staffing";
/// Compartment: shared with the incident, so the command sees requests.
pub const COMPARTMENT: &str = "incident";
/// Event type: a UHS asked for staff.
pub const REQUESTED: &str = "staffing.requested";
/// Event type: the request was met or withdrawn.
pub const CLOSED: &str = "staffing.closed";

/// Whom a UHS asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Need {
    /// A physician (*Arzt/Ärztin*).
    Physician,
    /// A paramedic (*Notfallsanitäter/in*).
    Paramedic,
}

impl Need {
    /// The incident role that fits the need, to suggest people who have it.
    pub fn role(self) -> &'static str {
        match self {
            Self::Physician => "physician",
            Self::Paramedic => "paramedic",
        }
    }
}

/// Payload of `staffing.requested`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requested {
    /// Request ID (a ULID).
    pub request: String,
    /// The UHS that asks ([`crate::UhsCreated::id`]).
    pub uhs: String,
    /// Whom it asks for.
    pub need: Need,
}

impl Requested {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `staffing.closed`: met by `person`, who was put on the staff list, or
/// withdrawn (none).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closed {
    /// The request.
    pub request: String,
    /// The person sent, as put on the staff list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<String>,
}

impl Closed {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// A request with its state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaffRequest {
    /// As made.
    pub requested: Requested,
    /// When.
    pub at: Hlc,
    /// Who asked.
    pub by: UserId,
    /// How it was closed: when, by whom, and the person sent (if any).
    pub closed: Option<(Hlc, UserId, Option<String>)>,
}

/// The requests of an incident, computed from the shared events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StaffRequests {
    /// Requests by ID.
    pub requests: BTreeMap<String, StaffRequest>,
}

impl StaffRequests {
    // [impl->dsn~uhs-staff-request~2]
    /// Computes the requests. The result doesn't depend on the order of `events`: changes
    /// apply in clock order, a request made twice keeps its first version, the first close
    /// counts, and closes of unknown requests are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, UserId, Result<Requested, Closed>)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let change = match e.body.event_type.as_str() {
                    REQUESTED => Ok(codec::from_cbor(&e.payload).ok()?),
                    CLOSED => Err(codec::from_cbor(&e.payload).ok()?),
                    _ => return None,
                };
                Some((e.body.hlc, e.body.author.user.clone(), change))
            })
            .collect();
        changes.sort_by_key(|(at, _, _)| *at);
        let mut requests: BTreeMap<String, StaffRequest> = BTreeMap::new();
        for (at, by, change) in changes {
            match change {
                Ok(r) => {
                    requests.entry(r.request.clone()).or_insert(StaffRequest {
                        requested: r,
                        at,
                        by,
                        closed: None,
                    });
                }
                Err(c) => {
                    if let Some(r) = requests.get_mut(&c.request)
                        && r.closed.is_none()
                    {
                        r.closed = Some((at, by, c.person));
                    }
                }
            }
        }
        Self { requests }
    }

    /// The open requests, oldest first.
    pub fn open(&self) -> Vec<&StaffRequest> {
        let mut open: Vec<&StaffRequest> = self
            .requests
            .values()
            .filter(|r| r.closed.is_none())
            .collect();
        open.sort_by_key(|r| r.at);
        open
    }

    /// The requests of one UHS, newest first.
    pub fn of_uhs(&self, uhs: &str) -> Vec<&StaffRequest> {
        let mut list: Vec<&StaffRequest> = self
            .requests
            .values()
            .filter(|r| r.requested.uhs == uhs)
            .collect();
        list.sort_by_key(|r| std::cmp::Reverse(r.at));
        list
    }
}

// [impl->dsn~package-registry~1]
/// The kinds of event this module writes, with their compartment and schema version.
pub mod kinds {
    use ordo_sync::EventKind;

    /// See [`super::REQUESTED`].
    pub const REQUESTED: EventKind =
        EventKind::new(super::COMPARTMENT, super::MODULE, super::REQUESTED, 1);

    /// See [`super::CLOSED`].
    pub const CLOSED: EventKind =
        EventKind::new(super::COMPARTMENT, super::MODULE, super::CLOSED, 1);

    /// Every kind above, e.g. for the incident log to check it names them all.
    pub const ALL: &[EventKind] = &[REQUESTED, CLOSED];
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, IncidentId, ServerName, Ulid};

    fn event(n: u64, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
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
                    user: "anna@ordo.example.org".parse().expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload,
        }
    }

    fn requested(n: u64, id: &str, uhs: &str, need: Need) -> AppliedEvent {
        let r = Requested {
            request: id.into(),
            uhs: uhs.into(),
            need,
        };
        event(n, REQUESTED, r.encode().unwrap())
    }

    fn closed(n: u64, id: &str, person: Option<&str>) -> AppliedEvent {
        let c = Closed {
            request: id.into(),
            person: person.map(Into::into),
        };
        event(n, CLOSED, c.encode().unwrap())
    }

    // [utest->dsn~uhs-staff-request~2]
    #[test]
    fn requests_stay_open_until_met_and_the_first_close_counts() {
        let mut events = vec![
            requested(1, "r1", "nord", Need::Physician),
            requested(2, "r2", "eingang", Need::Paramedic),
            closed(3, "r1", Some("petra@ordo.example.org")),
            closed(4, "r1", None),
            closed(5, "unknown", None),
        ];
        let a = StaffRequests::from_events(&events);
        events.reverse();
        assert_eq!(
            a,
            StaffRequests::from_events(&events),
            "order does not matter"
        );
        let open: Vec<&str> = a
            .open()
            .iter()
            .map(|r| r.requested.request.as_str())
            .collect();
        assert_eq!(open, ["r2"]);
        let r1 = &a.requests["r1"];
        assert_eq!(
            r1.closed.as_ref().and_then(|(_, _, p)| p.as_deref()),
            Some("petra@ordo.example.org"),
            "met by the person sent"
        );
        assert_eq!(a.of_uhs("nord").len(), 1);
        assert_eq!(Need::Paramedic.role(), "paramedic");
    }
}
