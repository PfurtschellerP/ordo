//! Incidents and their rosters (ADR-0046): planned, active, closed; who has which role.
//!
//! Roles are granted per incident by events in the incident's `incident` compartment, so
//! every change is ordered, signed and audited like any other event, and works offline on
//! a field server. The [`Registry`] is computed from these events — on the server, where it
//! feeds the access policy, and on devices, where it shows the roster.

use std::collections::{BTreeMap, BTreeSet};

use ordo_core::codec::{self, CodecError};
use ordo_core::{IncidentId, UserId};
use serde::{Deserialize, Serialize};

use crate::AppliedEvent;

/// Module ID of incident events.
pub const MODULE: &str = "incidents";
/// Compartment of incident events.
pub const COMPARTMENT: &str = "incident";
/// An incident was planned (its first event).
pub const PLANNED: &str = "incidents.planned";
/// The incident's state changed.
pub const STATE_CHANGED: &str = "incidents.stateChanged";
/// The incident's description and planned times were set (the latest counts).
pub const DETAILS_CHANGED: &str = "incidents.detailsChanged";
/// A running incident was paused or resumed, e.g. overnight at a festival.
pub const PAUSE_CHANGED: &str = "incidents.pauseChanged";
/// A person got a role in the incident.
pub const ROLE_GRANTED: &str = "incidents.roleGranted";
/// A person lost a role in the incident.
pub const ROLE_WITHDRAWN: &str = "incidents.roleWithdrawn";

/// State of an incident.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum IncidentState {
    /// Prepared ahead, e.g. a medical service at a concert.
    #[default]
    Planned,
    /// Under way.
    Active,
    /// Finished; kept for the record.
    Closed,
}

/// Payload of [`PLANNED`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Planned {
    /// Name, e.g. "Sanitätsdienst Stadtfest 2026".
    pub name: String,
}

/// Payload of [`STATE_CHANGED`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateChanged {
    /// The new state.
    pub state: IncidentState,
}

/// The longest description of an incident, in characters.
pub const DESCRIPTION_MAX: usize = 4000;

/// Payload of [`DETAILS_CHANGED`]: what the incident is and when it is planned.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Details {
    /// Free text, e.g. place, organiser, expected visitors.
    #[serde(default)]
    pub description: String,
    /// Planned start, ms since the Unix epoch (UTC).
    #[serde(default, rename = "startsAt", skip_serializing_if = "Option::is_none")]
    pub starts_at: Option<u64>,
    /// Planned end, ms since the Unix epoch (UTC); not before the start.
    #[serde(default, rename = "endsAt", skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<u64>,
    /// Whether strengths are shown in the detailed notation, with leaders of formations and
    /// special functions (UNIT-7; schema version 2).
    #[serde(
        default,
        rename = "strengthDetailed",
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub strength_detailed: bool,
}

/// Payload schema of [`DETAILS_CHANGED`]: 2 adds `strengthDetailed`.
pub const DETAILS_SCHEMA: u32 = 2;

impl Details {
    /// Whether the details can be stored: a bounded description, an end not before the
    /// start.
    pub fn check(&self) -> bool {
        self.description.chars().count() <= DESCRIPTION_MAX
            && match (self.starts_at, self.ends_at) {
                (Some(start), Some(end)) => end >= start,
                _ => true,
            }
    }
}

/// Payload of [`PAUSE_CHANGED`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PauseChange {
    /// `true`: paused; `false`: resumed.
    pub paused: bool,
}

/// Payload of [`ROLE_GRANTED`] and [`ROLE_WITHDRAWN`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleChange {
    /// The person.
    pub user: UserId,
    /// The role, e.g. `commander`, `medic` (defined by the profile and the policies).
    pub role: String,
}

macro_rules! encode {
    ($($t:ty),*) => {$(
        impl $t {
            /// CBOR payload.
            pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
                codec::to_cbor(self)
            }
        }
    )*};
}
encode!(Planned, StateChanged, RoleChange, Details, PauseChange);

/// Whether `payload` is a valid payload of the incident event type `event_type`.
pub fn is_valid(event_type: &str, payload: &[u8]) -> bool {
    match event_type {
        PLANNED => codec::from_cbor::<Planned>(payload).is_ok_and(|p| !p.name.trim().is_empty()),
        STATE_CHANGED => codec::from_cbor::<StateChanged>(payload).is_ok(),
        DETAILS_CHANGED => codec::from_cbor::<Details>(payload).is_ok_and(|d| d.check()),
        PAUSE_CHANGED => codec::from_cbor::<PauseChange>(payload).is_ok(),
        ROLE_GRANTED | ROLE_WITHDRAWN => {
            codec::from_cbor::<RoleChange>(payload).is_ok_and(|c| !c.role.trim().is_empty())
        }
        _ => false,
    }
}

/// One incident with its roster.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Incident {
    /// Name.
    pub name: String,
    /// State.
    pub state: IncidentState,
    /// Whether a running incident is paused (e.g. overnight); ends when it is closed.
    pub paused: bool,
    /// Description and planned times.
    pub details: Details,
    /// Roles per person.
    pub roster: BTreeMap<UserId, BTreeSet<String>>,
}

/// What an event changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Effect {
    /// Nothing (not an incident event, or a repeat).
    None,
    /// The incident was planned or its state changed.
    Incident,
    /// The roles of this person changed.
    Roles(UserId),
}

/// All incidents and their rosters, computed from incident events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Registry {
    incidents: BTreeMap<IncidentId, Incident>,
}

impl Registry {
    /// Computes the registry from events (any order of incidents; each incident's events
    /// in sequence order).
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut registry = Self::default();
        for e in events {
            registry.apply(e);
        }
        registry
    }

    /// Applies one ordered event.
    pub fn apply(&mut self, event: &AppliedEvent) -> Effect {
        let body = &event.body;
        if body.module.as_str() != MODULE || body.compartment.as_str() != COMPARTMENT {
            return Effect::None;
        }
        let incident = &body.incident;
        match body.event_type.as_str() {
            PLANNED => {
                let Ok(p) = codec::from_cbor::<Planned>(&event.payload) else {
                    return Effect::None;
                };
                if self.incidents.contains_key(incident) {
                    return Effect::None;
                }
                self.incidents.insert(
                    incident.clone(),
                    Incident {
                        name: p.name,
                        ..Incident::default()
                    },
                );
                Effect::Incident
            }
            STATE_CHANGED => {
                let (Ok(s), Some(i)) = (
                    codec::from_cbor::<StateChanged>(&event.payload),
                    self.incidents.get_mut(incident),
                ) else {
                    return Effect::None;
                };
                i.state = s.state;
                if s.state != IncidentState::Active {
                    i.paused = false;
                }
                Effect::Incident
            }
            // [impl->dsn~incident-details~1]
            DETAILS_CHANGED => {
                let (Ok(d), Some(i)) = (
                    codec::from_cbor::<Details>(&event.payload),
                    self.incidents.get_mut(incident),
                ) else {
                    return Effect::None;
                };
                if !d.check() {
                    return Effect::None;
                }
                i.details = d;
                Effect::Incident
            }
            // Only a running incident pauses.
            PAUSE_CHANGED => {
                let (Ok(p), Some(i)) = (
                    codec::from_cbor::<PauseChange>(&event.payload),
                    self.incidents.get_mut(incident),
                ) else {
                    return Effect::None;
                };
                if i.state != IncidentState::Active || i.paused == p.paused {
                    return Effect::None;
                }
                i.paused = p.paused;
                Effect::Incident
            }
            ROLE_GRANTED | ROLE_WITHDRAWN => {
                let (Ok(c), Some(i)) = (
                    codec::from_cbor::<RoleChange>(&event.payload),
                    self.incidents.get_mut(incident),
                ) else {
                    return Effect::None;
                };
                let roles = i.roster.entry(c.user.clone()).or_default();
                let changed = if body.event_type.as_str() == ROLE_GRANTED {
                    roles.insert(c.role)
                } else {
                    roles.remove(&c.role)
                };
                if roles.is_empty() {
                    i.roster.remove(&c.user);
                }
                if changed {
                    Effect::Roles(c.user)
                } else {
                    Effect::None
                }
            }
            _ => Effect::None,
        }
    }

    /// The incident, if it was planned.
    pub fn get(&self, incident: &IncidentId) -> Option<&Incident> {
        self.incidents.get(incident)
    }

    /// All incidents.
    pub fn iter(&self) -> impl Iterator<Item = (&IncidentId, &Incident)> {
        self.incidents.iter()
    }

    /// The roles of `user` in `incident`.
    pub fn roles(&self, user: &UserId, incident: &IncidentId) -> Vec<String> {
        self.incidents
            .get(incident)
            .and_then(|i| i.roster.get(user))
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::{
        Author, CompartmentId, DeviceId, EventBody, EventId, EventType, Hlc, ModuleId, Payload,
        ServerName, Ulid,
    };

    fn event(incident: &IncidentId, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        let server: ServerName = "ordo.example.org".parse().unwrap();
        AppliedEvent {
            seq: Some(1),
            body: EventBody {
                version: 1,
                id: EventId(Ulid::from_parts(1, 1).unwrap()),
                schema_version: 1,
                hlc: Hlc {
                    wall_ms: 1,
                    counter: 0,
                    node: DeviceId(Ulid::from_parts(1, 2).unwrap()),
                },
                incident: incident.clone(),
                module: MODULE.parse::<ModuleId>().unwrap(),
                compartment: COMPARTMENT.parse::<CompartmentId>().unwrap(),
                event_type: event_type.parse::<EventType>().unwrap(),
                author: Author {
                    user: "dora@ordo.example.org".parse().unwrap(),
                    device: DeviceId(Ulid::from_parts(1, 2).unwrap()),
                },
                origin: server,
                payload: Payload::Plain(payload.clone()),
            },
            payload,
        }
    }

    #[test]
    fn the_roster_follows_grants_and_withdrawals() {
        let server: ServerName = "ordo.example.org".parse().unwrap();
        let incident = IncidentId::new(Ulid::from_parts(5, 5).unwrap(), &server);
        let anna: UserId = "anna@ordo.example.org".parse().unwrap();
        let grant = |role: &str| {
            RoleChange {
                user: anna.clone(),
                role: role.into(),
            }
            .encode()
            .unwrap()
        };
        let mut r = Registry::default();
        assert_eq!(
            r.apply(&event(&incident, ROLE_GRANTED, grant("medic"))),
            Effect::None,
            "no roles before the incident is planned"
        );
        let planned = Planned {
            name: "Stadtfest".into(),
        }
        .encode()
        .unwrap();
        assert_eq!(
            r.apply(&event(&incident, PLANNED, planned.clone())),
            Effect::Incident
        );
        assert_eq!(r.apply(&event(&incident, PLANNED, planned)), Effect::None);
        assert_eq!(
            r.apply(&event(&incident, ROLE_GRANTED, grant("medic"))),
            Effect::Roles(anna.clone())
        );
        assert_eq!(r.roles(&anna, &incident), ["medic"]);
        let active = StateChanged {
            state: IncidentState::Active,
        }
        .encode()
        .unwrap();
        r.apply(&event(&incident, STATE_CHANGED, active));
        assert_eq!(r.get(&incident).unwrap().state, IncidentState::Active);
        assert_eq!(
            r.apply(&event(&incident, ROLE_WITHDRAWN, grant("medic"))),
            Effect::Roles(anna.clone())
        );
        assert!(r.roles(&anna, &incident).is_empty());
        assert!(r.get(&incident).unwrap().roster.is_empty());
    }

    // [utest->dsn~incident-details~1]
    #[test]
    fn details_are_kept_and_a_running_incident_pauses() {
        let server: ServerName = "ordo.example.org".parse().unwrap();
        let incident = IncidentId::new(Ulid::from_parts(6, 6).unwrap(), &server);
        let mut r = Registry::default();
        let planned = Planned {
            name: "Festival".into(),
        };
        r.apply(&event(&incident, PLANNED, planned.encode().unwrap()));
        let details = Details {
            description: "Drei Tage, 20 000 Gäste".into(),
            starts_at: Some(1_000),
            ends_at: Some(2_000),
            strength_detailed: true,
        };
        assert!(is_valid(DETAILS_CHANGED, &details.encode().unwrap()));
        r.apply(&event(
            &incident,
            DETAILS_CHANGED,
            details.encode().unwrap(),
        ));
        assert_eq!(r.get(&incident).unwrap().details, details);
        let backwards = Details {
            ends_at: Some(500),
            ..details.clone()
        };
        assert!(!is_valid(DETAILS_CHANGED, &backwards.encode().unwrap()));

        let pause = |paused| PauseChange { paused }.encode().unwrap();
        assert_eq!(
            r.apply(&event(&incident, PAUSE_CHANGED, pause(true))),
            Effect::None,
            "a planned incident does not pause"
        );
        let state = |state| StateChanged { state }.encode().unwrap();
        r.apply(&event(
            &incident,
            STATE_CHANGED,
            state(IncidentState::Active),
        ));
        assert_eq!(
            r.apply(&event(&incident, PAUSE_CHANGED, pause(true))),
            Effect::Incident
        );
        assert!(r.get(&incident).unwrap().paused);
        r.apply(&event(&incident, PAUSE_CHANGED, pause(false)));
        assert!(!r.get(&incident).unwrap().paused, "resumed");
        r.apply(&event(&incident, PAUSE_CHANGED, pause(true)));
        r.apply(&event(
            &incident,
            STATE_CHANGED,
            state(IncidentState::Closed),
        ));
        assert!(!r.get(&incident).unwrap().paused, "closing ends the pause");
    }
}
