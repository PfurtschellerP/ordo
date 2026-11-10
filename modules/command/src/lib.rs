//! Shared module `command` (CMD-1…4, ADR-0031): the command structure as incident data.
//! Positions grow with the incident, leaders hand over, units and UHS are assigned to
//! positions. Shared incident data, no patient data.

use std::collections::{BTreeMap, HashSet};

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "command";
/// Compartment: shared with the incident.
pub const COMPARTMENT: &str = "incident";
/// Event type: a position was created.
pub const POSITION_CREATED: &str = "command.positionCreated";
/// Event type: a position was closed (no longer needed); it stays in the history.
pub const POSITION_CLOSED: &str = "command.positionClosed";
/// Event type: a position got its leader (the first one, or a handover).
pub const LEADER_SET: &str = "command.leaderSet";
/// Event type: a unit or a UHS was put under a position, or taken out.
pub const ASSIGNED: &str = "command.assigned";
/// More direct subordinates than this are pointed out (span of control, FwDV 100 3.2.4).
pub const SPAN_OF_CONTROL: usize = 5;

/// Core meaning of a position kind, after ISO 22320.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Core {
    /// Leads (incident command, medical command).
    Command,
    /// A part of the scene or of the task (sector, sub-sector).
    Sector,
    /// A staff function (S1–S6).
    Staff,
    /// A liaison officer of another organisation.
    Liaison,
}

/// A position kind of the profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kind {
    /// Stable ID, e.g. `sector`.
    pub id: String,
    /// Name, e.g. "Einsatzabschnitt".
    pub label: String,
    /// Core meaning.
    pub core: Core,
}

/// The position kinds of the profile (`command/structure.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureCatalogue {
    /// Kinds in the order they are offered.
    #[serde(rename = "kind", default)]
    pub kinds: Vec<Kind>,
}

impl StructureCatalogue {
    /// Checks that there are kinds and every ID is unique.
    pub fn check(&self) -> Result<(), String> {
        if self.kinds.is_empty() {
            return Err("at least one position kind".into());
        }
        let mut seen = HashSet::new();
        match self.kinds.iter().find(|k| !seen.insert(k.id.as_str())) {
            Some(k) => Err(format!("kind {:?} appears twice", k.id)),
            None => Ok(()),
        }
    }

    /// The name of a kind, or its ID if the profile no longer has it.
    pub fn label<'a>(&'a self, id: &'a str) -> &'a str {
        self.kinds
            .iter()
            .find(|k| k.id == id)
            .map_or(id, |k| k.label.as_str())
    }
}

/// Payload of `command.positionCreated` (CMD-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionCreated {
    /// Position ID (a ULID, made when it is created).
    pub position: String,
    /// Kind ID from the profile.
    pub kind: String,
    /// Name, e.g. "EA 1 Bühne Nord".
    pub name: String,
    /// The position it reports to; none for the top of a tree.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    /// Service, e.g. `medical`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// Organisation, for one tree per organisation (ADR-0031).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organisation: Option<String>,
}

impl PositionCreated {
    /// Checks a new position: it needs a name and a kind.
    pub fn check(&self) -> Result<(), String> {
        if self.name.trim().is_empty() || self.kind.trim().is_empty() {
            return Err("a position needs a kind and a name".into());
        }
        Ok(())
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `command.positionClosed`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PositionClosed {
    /// The position.
    pub position: String,
}

impl PositionClosed {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `command.leaderSet` (CMD-1, CMD-4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderSet {
    /// The position.
    pub position: String,
    /// The leader: a user ID or a name.
    pub leader: String,
}

impl LeaderSet {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `command.assigned` (CMD-2): a unit or a UHS under a position, or (without
/// a position) taken out.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assigned {
    /// A unit (call sign).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// A UHS (its ID).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uhs: Option<String>,
    /// The position; none takes it out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
}

impl Assigned {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// A change of leader (CMD-4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Handover {
    /// The leader before; none for the first.
    pub from: Option<String>,
    /// The leader after.
    pub to: String,
    /// When.
    pub at: Hlc,
    /// Who recorded it.
    pub by: UserId,
}

/// A position with its current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Position {
    /// As created.
    pub created: PositionCreated,
    /// Whether it still exists (not closed).
    pub open: bool,
    /// Every leader, oldest first; the last is the current one.
    pub leaders: Vec<Handover>,
    /// Units under it (call signs), sorted.
    pub units: Vec<String>,
    /// UHS under it (IDs), sorted.
    pub uhs: Vec<String>,
    /// Open positions reporting to it, in order of creation.
    pub children: Vec<String>,
}

impl Position {
    /// The current leader.
    pub fn leader(&self) -> Option<&str> {
        self.leaders.last().map(|h| h.to.as_str())
    }

    /// Direct subordinates: positions, units and UHS.
    pub fn subordinates(&self) -> usize {
        self.children.len() + self.units.len() + self.uhs.len()
    }

    /// Whether it leads more direct subordinates than the doctrine suggests; shown, never
    /// enforced (ADR-0031).
    pub fn span_exceeded(&self) -> bool {
        self.subordinates() > SPAN_OF_CONTROL
    }
}

enum Change {
    Created(PositionCreated),
    Closed(PositionClosed),
    Leader(LeaderSet),
    Assigned(Assigned),
}

/// The command structure of an incident, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CommandStructure {
    /// Positions by ID, closed ones included.
    pub positions: BTreeMap<String, Position>,
    /// Open positions without a (known, open) parent, in order of creation.
    pub roots: Vec<String>,
    /// The position of each assigned unit.
    pub unit_positions: BTreeMap<String, String>,
    /// The position of each assigned UHS.
    pub uhs_positions: BTreeMap<String, String>,
}

impl CommandStructure {
    // [impl->dsn~command-structure~1]
    /// Computes the structure. The result doesn't depend on the order of `events`: changes
    /// apply in clock order; a position created twice keeps its first version; leaders,
    /// closings and assignments for unknown positions are ignored; the latest assignment of
    /// a unit or UHS counts, and one to a closed position takes it out.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, UserId, Change)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let change = match e.body.event_type.as_str() {
                    POSITION_CREATED => Change::Created(codec::from_cbor(&e.payload).ok()?),
                    POSITION_CLOSED => Change::Closed(codec::from_cbor(&e.payload).ok()?),
                    LEADER_SET => Change::Leader(codec::from_cbor(&e.payload).ok()?),
                    ASSIGNED => Change::Assigned(codec::from_cbor(&e.payload).ok()?),
                    _ => return None,
                };
                Some((e.body.hlc, e.body.author.user.clone(), change))
            })
            .collect();
        changes.sort_by_key(|(at, _, _)| *at);
        let mut s = Self::default();
        let mut order: Vec<String> = Vec::new();
        for (at, by, change) in changes {
            match change {
                Change::Created(c) => {
                    if !s.positions.contains_key(&c.position) {
                        order.push(c.position.clone());
                        s.positions.insert(
                            c.position.clone(),
                            Position {
                                created: c,
                                open: true,
                                leaders: Vec::new(),
                                units: Vec::new(),
                                uhs: Vec::new(),
                                children: Vec::new(),
                            },
                        );
                    }
                }
                Change::Closed(c) => {
                    if let Some(p) = s.positions.get_mut(&c.position) {
                        p.open = false;
                    }
                }
                Change::Leader(l) => {
                    if let Some(p) = s.positions.get_mut(&l.position).filter(|p| p.open) {
                        let from = p.leader().map(str::to_owned);
                        if from.as_deref() != Some(l.leader.as_str()) {
                            p.leaders.push(Handover {
                                from,
                                to: l.leader,
                                at,
                                by,
                            });
                        }
                    }
                }
                Change::Assigned(a) => {
                    let target = a
                        .position
                        .filter(|id| s.positions.get(id).is_some_and(|p| p.open));
                    for (key, map) in [
                        (a.unit, &mut s.unit_positions),
                        (a.uhs, &mut s.uhs_positions),
                    ] {
                        if let Some(key) = key {
                            match &target {
                                Some(id) => {
                                    map.insert(key, id.clone());
                                }
                                None => {
                                    map.remove(&key);
                                }
                            }
                        }
                    }
                }
            }
        }
        // Units and UHS under a position that was closed since are taken out.
        let open = |id: &String, positions: &BTreeMap<String, Position>| {
            positions.get(id).is_some_and(|p| p.open)
        };
        s.unit_positions.retain(|_, id| open(id, &s.positions));
        s.uhs_positions.retain(|_, id| open(id, &s.positions));
        for (unit, id) in &s.unit_positions {
            if let Some(p) = s.positions.get_mut(id) {
                p.units.push(unit.clone());
            }
        }
        for (uhs, id) in &s.uhs_positions {
            if let Some(p) = s.positions.get_mut(id) {
                p.uhs.push(uhs.clone());
            }
        }
        for id in order {
            let Some(p) = s.positions.get(&id).filter(|p| p.open) else {
                continue;
            };
            match p
                .created
                .parent
                .clone()
                .filter(|parent| open(parent, &s.positions))
            {
                Some(parent) => {
                    if let Some(parent) = s.positions.get_mut(&parent) {
                        parent.children.push(id);
                    }
                }
                None => s.roots.push(id),
            }
        }
        s
    }

    /// The open positions, in order of creation within the tree (depth first).
    pub fn open_positions(&self) -> Vec<&Position> {
        let mut out = Vec::new();
        let mut stack: Vec<&String> = self.roots.iter().rev().collect();
        while let Some(id) = stack.pop() {
            if let Some(p) = self.positions.get(id) {
                out.push(p);
                stack.extend(p.children.iter().rev());
            }
        }
        out
    }
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
                    user: "ben@ordo.example.org".parse().expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload,
        }
    }

    fn created(n: u64, id: &str, kind: &str, parent: Option<&str>) -> AppliedEvent {
        let c = PositionCreated {
            position: id.into(),
            kind: kind.into(),
            name: format!("{kind} {id}"),
            parent: parent.map(Into::into),
            service: None,
            organisation: None,
        };
        event(n, POSITION_CREATED, c.encode().unwrap())
    }

    fn leader(n: u64, id: &str, who: &str) -> AppliedEvent {
        let l = LeaderSet {
            position: id.into(),
            leader: who.into(),
        };
        event(n, LEADER_SET, l.encode().unwrap())
    }

    fn assign(n: u64, unit: &str, position: Option<&str>) -> AppliedEvent {
        let a = Assigned {
            unit: Some(unit.into()),
            uhs: None,
            position: position.map(Into::into),
        };
        event(n, ASSIGNED, a.encode().unwrap())
    }

    // [utest->dsn~command-structure~1]
    #[test]
    fn the_tree_grows_with_leaders_handovers_and_units_in_any_order() {
        let mut events = vec![
            created(1, "el", "incident-command", None),
            created(2, "sanel", "medical-command", Some("el")),
            created(3, "bhp", "treatment-area", Some("sanel")),
            leader(4, "sanel", "LNA Dr. M."),
            leader(5, "sanel", "LNA Dr. K."),
            leader(6, "sanel", "LNA Dr. K."), // the same leader again: no handover
            assign(7, "RTW 1", Some("bhp")),
            assign(8, "RTW 2", Some("bhp")),
            assign(9, "RTW 2", None),          // taken out
            created(10, "el", "sector", None), // created again later: the first counts
        ];
        events.reverse();
        let s = CommandStructure::from_events(&events);
        assert_eq!(s.roots, ["el"]);
        assert_eq!(s.positions["el"].created.kind, "incident-command");
        assert_eq!(s.positions["el"].children, ["sanel"]);
        let sanel = &s.positions["sanel"];
        assert_eq!(sanel.leader(), Some("LNA Dr. K."));
        assert_eq!(sanel.leaders.len(), 2);
        assert_eq!(
            sanel.leaders[1].from.as_deref(),
            Some("LNA Dr. M."),
            "a handover keeps both people"
        );
        assert_eq!(s.positions["bhp"].units, ["RTW 1"]);
        assert_eq!(s.unit_positions.get("RTW 2"), None);
        let order: Vec<&str> = s
            .open_positions()
            .iter()
            .map(|p| p.created.position.as_str())
            .collect();
        assert_eq!(order, ["el", "sanel", "bhp"]);
    }

    // [utest->dsn~command-structure~1]
    #[test]
    fn closing_a_position_frees_its_units_and_span_of_control_is_only_pointed_out() {
        let mut events = vec![
            created(1, "ea1", "sector", None),
            created(2, "ua", "sub-sector", Some("ea1")),
        ];
        for i in 0..6 {
            events.push(assign(3 + i, &format!("RTW {i}"), Some("ea1")));
        }
        let s = CommandStructure::from_events(&events);
        assert!(
            s.positions["ea1"].span_exceeded(),
            "seven direct subordinates"
        );

        events.push(event(
            20,
            POSITION_CLOSED,
            PositionClosed {
                position: "ea1".into(),
            }
            .encode()
            .unwrap(),
        ));
        events.push(assign(21, "RTW 9", Some("ea1"))); // to a closed position: nothing
        let s = CommandStructure::from_events(&events);
        assert!(!s.positions["ea1"].open);
        assert!(
            s.unit_positions.is_empty(),
            "units of a closed position are free again"
        );
        assert_eq!(s.roots, ["ua"], "its open sub-position moves to the top");
    }

    #[test]
    fn the_profile_is_valid() {
        let de: StructureCatalogue =
            toml::from_str(include_str!("../../../profiles/de/command/structure.toml")).unwrap();
        de.check().unwrap();
        assert_eq!(de.label("medical-command"), "Sanitätseinsatzleitung");
        assert!(de.kinds.iter().any(|k| k.core == Core::Staff));
    }
}
