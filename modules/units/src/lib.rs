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
/// Schema version of `units.added`: 2 since it carries service and strength, 3 since the
/// strength may name leaders of formations and special functions; older payloads decode
/// without them.
pub const ADDED_SCHEMA: u32 = 3;
/// Event type: a unit left the incident (UNIT-6); it stays in the history.
pub const RELEASED: &str = "units.released";
/// Event type: the crew of a unit was set (UNIT-5), the whole list.
pub const CREW_SET: &str = "units.crewSet";
/// Schema version of `units.crewSet`: 2 since a member may name the helper of the
/// incident's register (HELP-3); older payloads decode without it.
pub const CREW_SCHEMA: u32 = 2;

/// Crew strength in tactical notation (UNIT-2): leaders (*Führer*), sub-leaders
/// (*Unterführer*) and crew (*Mannschaft*); the total is computed. Leaders of formations
/// (*Führer von Verbänden*) are kept apart from the other leaders (*Zugführer*) for the
/// detailed notation (UNIT-7); the three-part notation shows both as leaders.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strength {
    /// *Führer von Verbänden* (schema version 3 of `units.added`; 0 if not given).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub unit_leaders: u32,
    /// *Führer*: in the detailed notation *Zugführer*.
    pub leaders: u32,
    /// *Unterführer*.
    pub sub_leaders: u32,
    /// *Mannschaft*.
    pub crew: u32,
}

fn is_zero(n: &u32) -> bool {
    *n == 0
}

/// Special functions of a unit's people (UNIT-7), by ID from the profile, e.g. physicians
/// or respirator wearers: how many of its strength have them ("davon"), not added to it.
pub type Special = BTreeMap<String, u32>;

impl Strength {
    /// Everyone.
    pub fn total(&self) -> u32 {
        self.unit_leaders + self.leaders + self.sub_leaders + self.crew
    }

    // [impl->dsn~detailed-strength~1]
    /// The numbers of the notation, the total last: four in the three-part notation (leaders
    /// of formations counted as leaders), five in the detailed one.
    pub fn parts(&self, detailed: bool) -> Vec<u32> {
        if detailed {
            vec![
                self.unit_leaders,
                self.leaders,
                self.sub_leaders,
                self.crew,
                self.total(),
            ]
        } else {
            vec![
                self.unit_leaders + self.leaders,
                self.sub_leaders,
                self.crew,
                self.total(),
            ]
        }
    }

    /// Adds another strength.
    pub fn add(&mut self, other: Strength) {
        self.unit_leaders += other.unit_leaders;
        self.leaders += other.leaders;
        self.sub_leaders += other.sub_leaders;
        self.crew += other.crew;
    }
}

impl From<[u32; 3]> for Strength {
    fn from([leaders, sub_leaders, crew]: [u32; 3]) -> Self {
        Self {
            unit_leaders: 0,
            leaders,
            sub_leaders,
            crew,
        }
    }
}

/// Adds the special functions of `other` to `into`.
pub fn add_special(into: &mut Special, other: &Special) {
    for (id, n) in other {
        *into.entry(id.clone()).or_default() += n;
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
    /// Special functions of its people (schema version 3).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub special: Special,
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
    /// The helper of the incident's register (HELP-3, schema version 2): only the ID; the
    /// helper's other data stays in the compartment `people`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub helper: Option<String>,
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
    /// Special functions of its people.
    pub special: Special,
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
    /// Special functions of all units present.
    pub special: Special,
    /// Where each person on a crew (or a package's staff list) is assigned (UNIT-8).
    pub people: Assignments,
}

// [impl->dsn~double-booking~1]
/// Where people are assigned: to the crews of units present, and to what packages add, e.g.
/// a UHS's staff list (UNIT-8). A person is matched by name as entered, ignoring case and
/// spaces around it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assignments {
    /// By person (matched form): the name as first noted and the places, in the order noted
    /// (units by call sign, then what packages add).
    people: BTreeMap<String, (String, Vec<Post>)>,
}

/// A place a person is assigned to: an ID its owner understands (e.g. `unit:<call sign>`)
/// and the label shown.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Post {
    /// ID, e.g. `unit:RTW 1`; [`Post::unit`] for units.
    pub id: String,
    /// Label, e.g. the call sign.
    pub label: String,
}

impl Post {
    /// The post of a unit's crew.
    pub fn unit(call_sign: &str) -> Self {
        Self {
            id: format!("unit:{call_sign}"),
            label: call_sign.to_owned(),
        }
    }

    /// The call sign, if this is a unit's crew.
    pub fn call_sign(&self) -> Option<&str> {
        self.id.strip_prefix("unit:")
    }
}

/// The form in which names are matched.
fn person_key(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Whether two names, as entered, are the same person for [`Assignments`].
pub fn same_person(a: &str, b: &str) -> bool {
    person_key(a) == person_key(b)
}

/// The key of a person: their helper if linked (HELP-3), else their name.
fn key_of(person: &str, helper: Option<&str>) -> String {
    helper.map_or_else(|| person_key(person), |h| format!("helper:{h}"))
}

impl Assignments {
    /// Notes `person` at `post`, as the helper `helper` if linked; empty names and repeats
    /// at the same post are ignored.
    pub fn add(&mut self, person: &str, helper: Option<&str>, post: &Post) {
        if person_key(person).is_empty() && helper.is_none() {
            return;
        }
        let entry = self
            .people
            .entry(key_of(person, helper))
            .or_insert_with(|| (person.trim().to_owned(), Vec::new()));
        if !entry.1.contains(post) {
            entry.1.push(post.clone());
        }
    }

    /// Counts the name `name` (e.g. a user ID on a UHS staff list) as the helper `helper`,
    /// whose account it is (HELP-2): their posts are one person's.
    pub fn link(&mut self, name: &str, helper: &str) {
        if let Some((_, posts)) = self.people.remove(&person_key(name)) {
            let entry = self
                .people
                .entry(key_of(name, Some(helper)))
                .or_insert_with(|| (name.trim().to_owned(), Vec::new()));
            for post in posts {
                if !entry.1.contains(&post) {
                    entry.1.push(post);
                }
            }
        }
    }

    /// Where `person` (the helper `helper` if linked) is assigned, in order.
    pub fn of(&self, person: &str, helper: Option<&str>) -> &[Post] {
        self.people
            .get(&key_of(person, helper))
            .map_or(&[], |(_, posts)| posts.as_slice())
    }

    /// Where `person` (the helper `helper` if linked) is assigned besides `here`.
    pub fn elsewhere(&self, person: &str, helper: Option<&str>, here: &Post) -> Vec<Post> {
        self.of(person, helper)
            .iter()
            .filter(|p| *p != here)
            .cloned()
            .collect()
    }

    /// The people assigned to more than one post, by name, with their posts: they may be
    /// counted twice.
    pub fn several(&self) -> Vec<(String, Vec<Post>)> {
        self.people
            .values()
            .filter(|(_, posts)| posts.len() > 1)
            .cloned()
            .collect()
    }
}

impl StrengthOnScene {
    /// Counts one unit present, of `organisation` and `service` (`""`: none given), with
    /// its strength if reported and its special functions. Packages count their own units
    /// this way, e.g. a UHS.
    pub fn count(
        &mut self,
        organisation: &str,
        service: &str,
        strength: Option<Strength>,
        special: &Special,
    ) {
        add_special(&mut self.special, special);
        self.units += 1;
        if strength.is_none() {
            self.unreported += 1;
        }
        let strength = strength.unwrap_or_default();
        self.total.add(strength);
        for (map, key) in [
            (&mut self.by_organisation, organisation),
            (&mut self.by_service, service),
        ] {
            let entry = map.entry(key.to_owned()).or_default();
            entry.0 += 1;
            entry.1.add(strength);
        }
    }
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
    // [impl->dsn~unit-register~3]
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
                            u.special = a.special;
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
                                    special: a.special,
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

    // [impl->dsn~unit-register~3]
    /// Units and persons on scene (UNIT-4): units released are not counted.
    pub fn strength_on_scene(&self) -> StrengthOnScene {
        let mut s = StrengthOnScene::default();
        for u in self.present() {
            s.count(
                &u.organisation,
                u.service.as_deref().unwrap_or_default(),
                u.strength,
                &u.special,
            );
            let post = Post::unit(&u.unit);
            for m in &u.crew {
                s.people.add(&m.name, m.helper.as_deref(), &post);
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
    /// Special functions counted apart in the detailed strength (UNIT-7), e.g. physicians.
    #[serde(rename = "special_function", default)]
    pub special_functions: Vec<Entry>,
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
            ("special function", &self.special_functions),
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
            special: Special::new(),
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

    // [utest->dsn~unit-register~3]
    #[test]
    fn strength_on_scene_counts_present_units_by_organisation_and_service() {
        let crew = CrewSet {
            unit: "RTW 1".into(),
            members: vec![CrewMember {
                name: "Anna".into(),
                qualifications: vec!["paramedic".into()],
                helper: None,
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

    // [utest->dsn~double-booking~1]
    #[test]
    fn people_on_more_than_one_crew_are_found_by_name() {
        let crew = |unit: &str, names: &[&str]| CrewSet {
            unit: unit.into(),
            members: names
                .iter()
                .map(|n| CrewMember {
                    name: (*n).into(),
                    qualifications: Vec::new(),
                    helper: None,
                })
                .collect(),
        };
        let events = vec![
            added(1, "RTW 1", "DRK", "rescue", None),
            added(2, "KTW 2", "DRK", "rescue", None),
            added(3, "HLF 1", "FF", "fire", None),
            event(
                4,
                CREW_SET,
                crew("RTW 1", &["Anna", "Ben"]).encode().unwrap(),
            ),
            event(
                5,
                CREW_SET,
                crew("KTW 2", &[" anna ", "Clara"]).encode().unwrap(),
            ),
            event(6, CREW_SET, crew("HLF 1", &["Clara"]).encode().unwrap()),
            released(7, "HLF 1"),
        ];
        let mut people = UnitRegister::from_events(&events)
            .strength_on_scene()
            .people;
        assert_eq!(
            people.several(),
            vec![(
                "anna".to_owned(),
                vec![Post::unit("KTW 2"), Post::unit("RTW 1")]
            )],
            "matched ignoring case and spaces, units by call sign; a released unit's crew does not count"
        );
        assert_eq!(
            people.elsewhere("ANNA", None, &Post::unit("KTW 2")),
            [Post::unit("RTW 1")]
        );
        assert!(
            people
                .elsewhere("Ben", None, &Post::unit("RTW 1"))
                .is_empty()
        );
        let uhs = Post {
            id: "uhs:u1".into(),
            label: "UHS Nord".into(),
        };
        people.add("ben", None, &uhs);
        people.add("ben", None, &uhs);
        assert_eq!(
            people.of("Ben", None),
            [Post::unit("RTW 1"), uhs],
            "a post counts once"
        );
        assert_eq!(Post::unit("RTW 1").call_sign(), Some("RTW 1"));
    }

    // [utest->dsn~crews-from-register~1]
    #[test]
    fn crews_linked_to_helpers_are_matched_by_helper_and_account() {
        let member = |name: &str, helper: Option<&str>| CrewMember {
            name: name.into(),
            qualifications: Vec::new(),
            helper: helper.map(Into::into),
        };
        let crew = |unit: &str, members: Vec<CrewMember>| CrewSet {
            unit: unit.into(),
            members,
        };
        let events = vec![
            added(1, "RTW 1", "DRK", "rescue", None),
            added(2, "KTW 2", "DRK", "rescue", None),
            event(
                3,
                CREW_SET,
                crew("RTW 1", vec![member("Lena", Some("h1"))])
                    .encode()
                    .unwrap(),
            ),
            event(
                4,
                CREW_SET,
                crew(
                    "KTW 2",
                    vec![member("Lena B.", Some("h1")), member("Lena", None)],
                )
                .encode()
                .unwrap(),
            ),
        ];
        let mut people = UnitRegister::from_events(&events)
            .strength_on_scene()
            .people;
        assert_eq!(
            people.several(),
            vec![(
                "Lena B.".to_owned(),
                vec![Post::unit("KTW 2"), Post::unit("RTW 1")]
            )],
            "the same helper under two names; a namesake without a link is someone else"
        );
        assert_eq!(
            people.elsewhere("Lena", Some("h1"), &Post::unit("RTW 1")),
            [Post::unit("KTW 2")]
        );
        // The helper's account on a UHS staff list is the same person (HELP-2).
        let uhs = Post {
            id: "uhs:u1".into(),
            label: "UHS Nord".into(),
        };
        people.add("lena@ordo.example.org", None, &uhs);
        people.link("lena@ordo.example.org", "h1");
        assert_eq!(people.of("", Some("h1")).len(), 3);
        // Old crew lists (schema version 1) decode without the link.
        #[derive(Serialize)]
        struct MemberV1 {
            name: String,
            qualifications: Vec<String>,
        }
        let old = MemberV1 {
            name: "Anna".into(),
            qualifications: Vec::new(),
        };
        let v1: CrewMember = codec::from_cbor(&codec::to_cbor(&old).unwrap()).unwrap();
        assert_eq!(v1, member("Anna", None));
    }

    // [utest->dsn~detailed-strength~1]
    #[test]
    fn the_detailed_notation_keeps_leaders_of_formations_and_special_functions_apart() {
        let s = Strength {
            unit_leaders: 1,
            leaders: 1,
            sub_leaders: 6,
            crew: 40,
        };
        assert_eq!(
            s.parts(true),
            [1, 1, 6, 40, 48],
            "the example of DV 102, 5.3"
        );
        assert_eq!(s.parts(false), [2, 6, 40, 48], "both leaders as Führer");
        let doctors = Special::from([("physician".to_owned(), 2), ("respirator".to_owned(), 6)]);
        let a = UnitAdded {
            unit: "SEG 1".into(),
            unit_type: "SEG".into(),
            organisation: "DRK".into(),
            service: None,
            strength: Some(s),
            special: doctors.clone(),
        };
        let mut b = a.clone();
        b.unit = "SEG 2".into();
        let register = UnitRegister::from_events(&[
            event(1, ADDED, a.encode().unwrap()),
            event(2, ADDED, b.encode().unwrap()),
        ]);
        let on_scene = register.strength_on_scene();
        assert_eq!(on_scene.total.total(), 96);
        assert_eq!(on_scene.special["physician"], 4, "special functions add up");
        // A strength of schema version 2 has no leaders of formations.
        #[derive(Serialize)]
        struct StrengthV2 {
            leaders: u32,
            sub_leaders: u32,
            crew: u32,
        }
        let old = StrengthV2 {
            leaders: 0,
            sub_leaders: 1,
            crew: 1,
        };
        let v2: Strength = codec::from_cbor(&codec::to_cbor(&old).unwrap()).unwrap();
        assert_eq!(v2, Strength::from([0, 1, 1]));
        assert!(
            !codec::to_cbor(&v2)
                .unwrap()
                .windows(12)
                .any(|w| w == b"unit_leaders"),
            "not written when 0"
        );
    }

    // [utest->dsn~unit-register~3]
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
