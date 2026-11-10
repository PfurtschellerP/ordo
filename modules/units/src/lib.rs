//! Shared module `units`: the units of an incident (UNIT-1…6) with their status, service,
//! crew strength and crew, released units in the history.
//!
//! Unit status uses *last writer wins* by hybrid logical clock. Each change records which
//! change it replaces, so ORDO can also tell when a change was made without knowing the
//! previous one — a lost update — and show it instead of hiding it (requirement OFF-4).

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "units";
/// Compartment of unit events.
pub const COMPARTMENT: &str = "incident";
/// Event type of a status change.
pub const STATUS_CHANGED: &str = "units.statusChanged";
/// Event type: a unit (vehicle, team or other resource) joined the incident.
pub const ADDED: &str = "units.added";
/// Schema version of `units.added`: 2 since it carries service and strength; version 1
/// payloads decode without them.
pub const ADDED_SCHEMA: u32 = 2;
/// Event type: a unit left the incident (UNIT-6); it stays in the history.
pub const RELEASED: &str = "units.released";
/// Event type: the crew of a unit was set (UNIT-5), the whole list.
pub const CREW_SET: &str = "units.crewSet";

/// Crew strength in tactical notation (UNIT-2): leaders (*Führer*), sub-leaders
/// (*Unterführer*) and crew (*Mannschaft*); the total is computed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strength {
    /// *Führer*.
    pub leaders: u32,
    /// *Unterführer*.
    pub sub_leaders: u32,
    /// *Mannschaft*.
    pub crew: u32,
}

impl Strength {
    /// Everyone.
    pub fn total(&self) -> u32 {
        self.leaders + self.sub_leaders + self.crew
    }

    /// Adds another strength.
    pub fn add(&mut self, other: Strength) {
        self.leaders += other.leaders;
        self.sub_leaders += other.sub_leaders;
        self.crew += other.crew;
    }
}

impl From<[u32; 3]> for Strength {
    fn from([leaders, sub_leaders, crew]: [u32; 3]) -> Self {
        Self {
            leaders,
            sub_leaders,
            crew,
        }
    }
}

/// Payload of `units.added` (UNIT-1, UNIT-2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitAdded {
    /// Call sign, e.g. "Rotkreuz Musterstadt 83/1".
    pub unit: String,
    /// Type from the profile, e.g. "RTW", "KTW", "HLF", "Sanitätstrupp".
    pub unit_type: String,
    /// Organisation, e.g. "DRK Musterstadt".
    pub organisation: String,
    /// Service from the profile, e.g. `rescue` (schema version 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    /// Crew strength (schema version 2).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<Strength>,
}

impl UnitAdded {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `units.released`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Released {
    /// Call sign.
    pub unit: String,
}

impl Released {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// One member of a crew.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewMember {
    /// Name or user ID, as entered.
    pub name: String,
    /// Qualification IDs from the profile.
    #[serde(default)]
    pub qualifications: Vec<String>,
}

/// Payload of `units.crewSet`: the whole crew of a unit, replacing earlier lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrewSet {
    /// Call sign.
    pub unit: String,
    /// The crew.
    pub members: Vec<CrewMember>,
}

impl CrewSet {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// A unit joining or leaving the incident.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Presence {
    /// Joined (or came back).
    Joined,
    /// Released.
    Released,
}

/// One join or release with time and person.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PresenceEntry {
    /// What happened.
    pub change: Presence,
    /// When.
    pub at: Hlc,
    /// Who.
    pub by: UserId,
}

/// A unit of the incident with its current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Unit {
    /// Call sign.
    pub unit: String,
    /// Type.
    pub unit_type: String,
    /// Organisation.
    pub organisation: String,
    /// Service, if known.
    pub service: Option<String>,
    /// Crew strength, if reported.
    pub strength: Option<Strength>,
    /// Current status, if one was set.
    pub status: Option<UnitStatus>,
    /// The crew (latest list).
    pub crew: Vec<CrewMember>,
    /// Joins and releases, oldest first.
    pub presence: Vec<PresenceEntry>,
}

impl Unit {
    /// Whether the unit is part of the incident now (not released).
    pub fn is_present(&self) -> bool {
        self.presence
            .last()
            .is_none_or(|e| e.change == Presence::Joined)
    }
}

/// Units and persons on scene: in total, per organisation and per service (UNIT-4).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StrengthOnScene {
    /// Units present.
    pub units: u32,
    /// Units present without a reported strength.
    pub unreported: u32,
    /// Strength of all units present.
    pub total: Strength,
    /// Units and strength per organisation.
    pub by_organisation: BTreeMap<String, (u32, Strength)>,
    /// Units and strength per service ID (`""`: none given).
    pub by_service: BTreeMap<String, (u32, Strength)>,
}

/// All units of an incident with their status, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitRegister {
    /// Units by call sign, released ones included.
    pub units: BTreeMap<String, Unit>,
    /// Status changes made without knowing the previous one.
    pub lost_updates: Vec<LostUpdate>,
}

enum UnitChange {
    Added(UnitAdded),
    Released(Released),
    Crew(CrewSet),
}

impl UnitRegister {
    // [impl->dsn~unit-register~2]
    /// Computes the register; the result doesn't depend on the order of `events`. A unit
    /// added while present keeps its first entry; added after its release, it comes back
    /// with the attributes given then. The latest crew list counts.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, UserId, UnitChange)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let change = match e.body.event_type.as_str() {
                    ADDED => UnitChange::Added(codec::from_cbor(&e.payload).ok()?),
                    RELEASED => UnitChange::Released(codec::from_cbor(&e.payload).ok()?),
                    CREW_SET => UnitChange::Crew(codec::from_cbor(&e.payload).ok()?),
                    _ => return None,
                };
                Some((e.body.hlc, e.body.author.user.clone(), change))
            })
            .collect();
        changes.sort_by_key(|(hlc, _, _)| *hlc);
        let board = StatusBoard::from_events(events);
        let mut units: BTreeMap<String, Unit> = BTreeMap::new();
        for (at, by, change) in changes {
            match change {
                UnitChange::Added(a) => {
                    let entry = PresenceEntry {
                        change: Presence::Joined,
                        at,
                        by,
                    };
                    match units.get_mut(&a.unit) {
                        Some(u) if u.is_present() => {}
                        Some(u) => {
                            u.unit_type = a.unit_type;
                            u.organisation = a.organisation;
                            u.service = a.service;
                            u.strength = a.strength;
                            u.presence.push(entry);
                        }
                        None => {
                            units.insert(
                                a.unit.clone(),
                                Unit {
                                    status: board.units.get(&a.unit).cloned(),
                                    unit: a.unit,
                                    unit_type: a.unit_type,
                                    organisation: a.organisation,
                                    service: a.service,
                                    strength: a.strength,
                                    crew: Vec::new(),
                                    presence: vec![entry],
                                },
                            );
                        }
                    }
                }
                UnitChange::Released(r) => {
                    if let Some(u) = units.get_mut(&r.unit).filter(|u| u.is_present()) {
                        u.presence.push(PresenceEntry {
                            change: Presence::Released,
                            at,
                            by,
                        });
                    }
                }
                UnitChange::Crew(c) => {
                    if let Some(u) = units.get_mut(&c.unit) {
                        u.crew = c.members;
                    }
                }
            }
        }
        Self {
            units,
            lost_updates: board.lost_updates,
        }
    }

    /// The units present (not released), by call sign.
    pub fn present(&self) -> impl Iterator<Item = &Unit> {
        self.units.values().filter(|u| u.is_present())
    }

    // [impl->dsn~unit-register~2]
    /// Units and persons on scene (UNIT-4): units released are not counted.
    pub fn strength_on_scene(&self) -> StrengthOnScene {
        let mut s = StrengthOnScene::default();
        for u in self.present() {
            let strength = u.strength.unwrap_or_default();
            s.units += 1;
            if u.strength.is_none() {
                s.unreported += 1;
            }
            s.total.add(strength);
            for (map, key) in [
                (&mut s.by_organisation, u.organisation.clone()),
                (&mut s.by_service, u.service.clone().unwrap_or_default()),
            ] {
                let entry = map.entry(key).or_default();
                entry.0 += 1;
                entry.1.add(strength);
            }
        }
        s
    }
}

/// An entry of a profile list (service, qualification).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Stable ID.
    pub id: String,
    /// Label.
    pub label: String,
}

/// Services and qualifications of the profile (`units/catalogue.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitCatalogue {
    /// Services a unit belongs to.
    #[serde(rename = "service", default)]
    pub services: Vec<Entry>,
    /// Qualifications of crew members.
    #[serde(rename = "qualification", default)]
    pub qualifications: Vec<Entry>,
}

impl UnitCatalogue {
    /// Checks that there are services and every ID is unique in its list.
    pub fn check(&self) -> Result<(), String> {
        if self.services.is_empty() {
            return Err("at least one service".into());
        }
        for (name, list) in [
            ("service", &self.services),
            ("qualification", &self.qualifications),
        ] {
            let mut seen = std::collections::HashSet::new();
            if let Some(e) = list.iter().find(|e| !seen.insert(e.id.as_str())) {
                return Err(format!("{name} {:?} appears twice", e.id));
            }
        }
        Ok(())
    }

    /// The label of an ID in `list`, or the ID if the profile no longer has it.
    pub fn label<'a>(list: &'a [Entry], id: &'a str) -> &'a str {
        list.iter()
            .find(|e| e.id == id)
            .map_or(id, |e| e.label.as_str())
    }
}

/// Payload of `units.statusChanged`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusChanged {
    /// Call sign of the unit.
    pub unit: String,
    /// New status code (from the profile, e.g. "3").
    pub status: String,
    /// Timestamp of the status change this one replaces, as known to the author.
    pub replaces: Option<Hlc>,
}

impl StatusChanged {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Current status of a unit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitStatus {
    /// Status code.
    pub status: String,
    /// When it was set.
    pub since: Hlc,
    /// Who set it.
    pub by: UserId,
}

/// A status change that was made without knowing the previous one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LostUpdate {
    /// Call sign.
    pub unit: String,
    /// The status that was overwritten without being seen.
    pub overwritten: String,
    /// The status that won.
    pub kept: String,
    /// Who made the winning change.
    pub by: UserId,
}

/// Status of all units, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct StatusBoard {
    /// Current status per call sign.
    pub units: BTreeMap<String, UnitStatus>,
    /// Lost updates, to be shown in the log.
    pub lost_updates: Vec<LostUpdate>,
}

impl StatusBoard {
    // [impl->dsn~lost-update-detection~1]
    /// Computes the board. The result doesn't depend on the order of `events`.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: BTreeMap<String, Vec<(Hlc, UserId, StatusChanged)>> = BTreeMap::new();
        for event in events {
            if event.body.module.as_str() != MODULE
                || event.body.event_type.as_str() != STATUS_CHANGED
            {
                continue;
            }
            if let Ok(change) = codec::from_cbor::<StatusChanged>(&event.payload) {
                changes.entry(change.unit.clone()).or_default().push((
                    event.body.hlc,
                    event.body.author.user.clone(),
                    change,
                ));
            }
        }
        let mut board = Self::default();
        for (unit, mut list) in changes {
            list.sort_by_key(|(hlc, _, _)| *hlc);
            let mut previous: Option<(Hlc, String)> = None;
            for (hlc, user, change) in list {
                if let Some((prev_hlc, prev_status)) = &previous
                    && change.replaces != Some(*prev_hlc)
                {
                    board.lost_updates.push(LostUpdate {
                        unit: unit.clone(),
                        overwritten: prev_status.clone(),
                        kept: change.status.clone(),
                        by: user.clone(),
                    });
                }
                previous = Some((hlc, change.status.clone()));
                board.units.insert(
                    unit.clone(),
                    UnitStatus {
                        status: change.status,
                        since: hlc,
                        by: user,
                    },
                );
            }
        }
        board
    }
}

/// The status codes of units, from the profile (`units/status-codes.toml`), never in code.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCatalogue {
    /// Status codes in the order they are offered.
    #[serde(rename = "status", default)]
    pub codes: Vec<StatusCode>,
}

/// One status code.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCode {
    /// Code, e.g. "2".
    pub code: String,
    /// Label, e.g. "Einsatzbereit auf Wache".
    pub label: String,
    /// English name for search, e.g. "available station".
    pub name: String,
    /// Whether a unit in this status is available for a new task.
    #[serde(default)]
    pub available: bool,
}

impl StatusCatalogue {
    /// Checks that there are codes and each is unique.
    pub fn check(&self) -> Result<(), String> {
        if self.codes.is_empty() {
            return Err("at least one status code".into());
        }
        let mut seen = std::collections::HashSet::new();
        match self.codes.iter().find(|c| !seen.insert(c.code.as_str())) {
            Some(c) => Err(format!("status code {:?} appears twice", c.code)),
            None => Ok(()),
        }
    }

    /// The entry of a code, if the profile has it.
    pub fn get(&self, code: &str) -> Option<&StatusCode> {
        self.codes.iter().find(|c| c.code == code)
    }

    /// Whether a unit with this status is available (unknown codes are not).
    pub fn is_available(&self, code: &str) -> bool {
        self.get(code).is_some_and(|c| c.available)
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

    fn added(
        n: u64,
        unit: &str,
        org: &str,
        service: &str,
        strength: Option<[u32; 3]>,
    ) -> AppliedEvent {
        let a = UnitAdded {
            unit: unit.into(),
            unit_type: "RTW".into(),
            organisation: org.into(),
            service: Some(service.into()),
            strength: strength.map(Strength::from),
        };
        event(n, ADDED, a.encode().unwrap())
    }

    fn released(n: u64, unit: &str) -> AppliedEvent {
        event(
            n,
            RELEASED,
            Released { unit: unit.into() }.encode().unwrap(),
        )
    }

    // [utest->dsn~unit-register~2]
    #[test]
    fn strength_on_scene_counts_present_units_by_organisation_and_service() {
        let crew = CrewSet {
            unit: "RTW 1".into(),
            members: vec![CrewMember {
                name: "Anna".into(),
                qualifications: vec!["paramedic".into()],
            }],
        };
        let events = vec![
            added(1, "RTW 1", "DRK", "rescue", Some([0, 1, 1])),
            added(2, "HLF 1", "FF Musterstadt", "fire", Some([0, 1, 8])),
            added(3, "KTW 2", "DRK", "rescue", None),
            event(4, CREW_SET, crew.encode().unwrap()),
            released(5, "HLF 1"),
        ];
        let register = UnitRegister::from_events(&events);
        assert_eq!(
            register.units.len(),
            3,
            "a released unit stays in the register"
        );
        assert!(!register.units["HLF 1"].is_present());
        assert_eq!(
            register.units["RTW 1"].crew[0].qualifications,
            ["paramedic"]
        );
        let s = register.strength_on_scene();
        assert_eq!((s.units, s.unreported, s.total.total()), (2, 1, 2));
        assert_eq!(s.by_organisation["DRK"], (2, Strength::from([0, 1, 1])));
        assert!(
            !s.by_service.contains_key("fire"),
            "released units are not counted"
        );
        assert_eq!(Strength::from([0, 1, 8]).total(), 9);
    }

    // [utest->dsn~unit-register~2]
    #[test]
    fn a_released_unit_comes_back_when_added_again_and_old_payloads_decode() {
        let mut events = vec![
            added(1, "RTW 1", "DRK", "rescue", Some([0, 1, 1])),
            added(2, "RTW 1", "ASB", "rescue", None), // while present: ignored
            released(3, "RTW 1"),
            released(4, "RTW 1"), // already released: nothing
            added(5, "RTW 1", "DRK", "rescue", Some([0, 1, 2])),
        ];
        events.reverse();
        let register = UnitRegister::from_events(&events);
        let u = &register.units["RTW 1"];
        assert!(u.is_present());
        assert_eq!(u.strength, Some(Strength::from([0, 1, 2])));
        assert_eq!(
            u.presence.iter().map(|e| e.change).collect::<Vec<_>>(),
            [Presence::Joined, Presence::Released, Presence::Joined]
        );

        #[derive(Serialize)]
        struct V1 {
            unit: String,
            unit_type: String,
            organisation: String,
        }
        let v1 = codec::to_cbor(&V1 {
            unit: "RTW 9".into(),
            unit_type: "RTW".into(),
            organisation: "DRK".into(),
        })
        .unwrap();
        let decoded: UnitAdded = codec::from_cbor(&v1).unwrap();
        assert_eq!((decoded.service, decoded.strength), (None, None));
    }

    #[test]
    fn the_profile_catalogue_is_valid() {
        let de: UnitCatalogue =
            toml_from(include_str!("../../../profiles/de/units/catalogue.toml"));
        de.check().unwrap();
        assert_eq!(
            UnitCatalogue::label(&de.services, "rescue"),
            "Rettungsdienst"
        );
    }

    fn toml_from(text: &str) -> UnitCatalogue {
        toml::from_str(text).expect("valid catalogue")
    }

    #[test]
    fn status_codes_are_unique_and_unknown_ones_are_not_available() {
        let code = |code: &str, available| StatusCode {
            code: code.into(),
            label: code.into(),
            name: code.into(),
            available,
        };
        let mut catalogue = StatusCatalogue {
            codes: vec![code("2", true), code("3", false)],
        };
        assert!(catalogue.check().is_ok());
        assert!(catalogue.is_available("2"));
        assert!(!catalogue.is_available("3"));
        assert!(!catalogue.is_available("9"));
        catalogue.codes.push(code("2", false));
        assert!(catalogue.check().is_err());
        assert!(StatusCatalogue::default().check().is_err());
    }
}
