//! Shared module `tasks` (TASK-1…4, REP-1…5, ADR-0081, ADR-0085): the Aufträge of the
//! incident command, given to units, a position or a person, with status, history and
//! follow-ups. A report is an Auftrag with a kind, a place and the time it came in, which
//! may wait for its units. Each unit on an Auftrag has its own state; the Auftrag's follows
//! from them. Shared incident data, no patient data.

use std::collections::{BTreeMap, HashSet};

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "tasks";
/// Compartment: shared with the incident.
pub const COMPARTMENT: &str = "incident";
/// Event type: a task was given.
pub const GIVEN: &str = "tasks.given";
/// Event type: a task moved on, or got a new follow-up time.
pub const UPDATED: &str = "tasks.updated";
/// Event type: a unit was added to an open Auftrag (REP-2; version 1: named as its one unit
/// before it started).
pub const ASSIGNED: &str = "tasks.assigned";
/// Event type: a unit's crew registered the patient of a patient Auftrag (REP-3).
pub const PATIENT_REGISTERED: &str = "tasks.patientRegistered";
/// Schema version of `tasks.given`: 2 since it may name a report (REP-2, ADR-0075), 3 since
/// it may be a report itself, with kind, place and time, or be for a person (ADR-0081);
/// older payloads decode unchanged.
pub const GIVEN_SCHEMA: u32 = 3;
/// Schema version of `tasks.assigned`: 2 adds a unit instead of naming the one unit
/// (ADR-0085); the payload is the same.
pub const ASSIGNED_SCHEMA: u32 = 2;
/// Schema version of `tasks.updated`: 2 may name the unit whose part moves on (ADR-0085).
pub const UPDATED_SCHEMA: u32 = 2;

/// Longest place, in characters.
pub const PLACE_MAX: usize = 120;
/// Longest description of a reported Auftrag, in characters: a report is short.
pub const TEXT_MAX: usize = 280;

/// What was reported (REP-1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskKind {
    /// Something happening on the site, e.g. a fight at a gate.
    Event,
    /// A person who needs help: becomes a patient (REP-3).
    Patient,
}

impl TaskKind {
    /// The kind as the profile's symbol rules name it.
    pub fn id(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::Patient => "patient",
        }
    }
}

/// Status of a task (TASK-2), in the order of the steps.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskStatus {
    /// Given, not started.
    Open,
    /// Being carried out.
    InProgress,
    /// Done (end).
    Done,
    /// Not needed any more (end).
    Cancelled,
}

impl TaskStatus {
    /// Whether the task still needs something.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Open | Self::InProgress)
    }

    /// The statuses that may follow; never a step back.
    pub fn next(self) -> Vec<Self> {
        match self {
            Self::Open => vec![Self::InProgress, Self::Done, Self::Cancelled],
            Self::InProgress => vec![Self::Done, Self::Cancelled],
            Self::Done | Self::Cancelled => Vec::new(),
        }
    }
}

/// Payload of `tasks.given` (TASK-1, REP-1).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskGiven {
    /// Task ID (a ULID, made when the task is given).
    pub task: String,
    /// The unit it is for (call sign).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// The sector it is for (with the command structure, CMD-1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sector: Option<String>,
    /// What to do.
    pub text: String,
    /// The template it started from, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    /// When to follow up (Unix time in milliseconds), if wanted (TASK-4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<u64>,
    /// The report of the module `reports` the task is about, if any (REP-2, ADR-0075; an
    /// incident recorded before ADR-0081).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<String>,
    /// The person it is for (an account), e.g. sent to a UHS (UHS-15). Since schema 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<String>,
    /// What was reported, if it came in as a report (REP-1). Since schema 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<TaskKind>,
    /// Where, as said, e.g. "Bühne Nord, links vom Eingang". Since schema 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
    /// When the report came in (Unix time in milliseconds). Since schema 3.
    #[serde(
        default,
        rename = "receivedAt",
        skip_serializing_if = "Option::is_none"
    )]
    pub received_at: Option<u64>,
    /// The Auftrag this one sends a further unit to, for the same place and pin (REP-2).
    /// Since schema 3.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follows: Option<String>,
}

/// A task that can't be given.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TaskError {
    /// Nothing to do.
    #[error("the task needs a text")]
    NoText,
    /// Nobody to do it.
    #[error("the task needs a unit, a position or a person")]
    NoAssignee,
    /// A report without a place.
    #[error("the report needs a place")]
    NoPlace,
    /// Longer than a report is.
    #[error("the place or the description is too long")]
    TooLong,
}

fn named(value: Option<&String>) -> bool {
    value.is_some_and(|v| !v.trim().is_empty())
}

impl TaskGiven {
    /// Checks a new task: a text and someone to do it; a report needs a place instead of
    /// someone, and both are short.
    pub fn check(&self) -> Result<(), TaskError> {
        if self.text.trim().is_empty() {
            return Err(TaskError::NoText);
        }
        if self.kind.is_some() {
            if !named(self.place.as_ref()) {
                return Err(TaskError::NoPlace);
            }
            if self.text.chars().count() > TEXT_MAX {
                return Err(TaskError::TooLong);
            }
        } else if !self.is_assigned() {
            return Err(TaskError::NoAssignee);
        }
        if self
            .place
            .as_ref()
            .is_some_and(|p| p.chars().count() > PLACE_MAX)
        {
            return Err(TaskError::TooLong);
        }
        Ok(())
    }

    /// Whether it names a unit, a position or a person.
    pub fn is_assigned(&self) -> bool {
        named(self.unit.as_ref()) || named(self.sector.as_ref()) || named(self.person.as_ref())
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `tasks.updated` (TASK-2, TASK-4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskUpdated {
    /// The task.
    pub task: String,
    /// The new status; `None` keeps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<TaskStatus>,
    /// A new follow-up time; `None` keeps it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub follow_up: Option<u64>,
    /// The unit whose part the status is (ADR-0085); `None`: the whole Auftrag. Since
    /// schema 2.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
}

impl TaskUpdated {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `tasks.assigned` (REP-2): a unit added to an open Auftrag (schema 2), or the
/// one unit of an Auftrag that had not started (schema 1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskAssigned {
    /// The task.
    pub task: String,
    /// The unit (call sign).
    pub unit: String,
}

impl TaskAssigned {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `tasks.patientRegistered` (REP-3): only the Auftrag and the unit, never the
/// triage tag, the category or anything else about the patient.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatientRegistered {
    /// The patient Auftrag.
    pub task: String,
    /// The unit whose crew took the patient over (call sign).
    pub unit: String,
}

impl PatientRegistered {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// A patient taken over from a patient Auftrag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TakenOver {
    /// By which unit.
    pub unit: String,
    /// When.
    pub at: Hlc,
    /// Who registered the patient.
    pub by: UserId,
}

/// How far an Auftrag is, as shown (TASK-2).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Open, nobody named yet (*Eingegangen*).
    Received,
    /// Open, someone named (*Zugewiesen*).
    Assigned,
    /// Being carried out.
    InProgress,
    /// Done.
    Done,
    /// Cancelled by the incident command.
    Cancelled,
}

/// One change of a task.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskStep {
    /// The status from then on.
    pub status: TaskStatus,
    /// When.
    pub at: Hlc,
    /// Who.
    pub by: UserId,
}

/// One unit on an Auftrag, with its own state (ADR-0085).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitPart {
    /// The unit (call sign).
    pub unit: String,
    /// How far this unit is.
    pub status: TaskStatus,
    /// Every step of this unit, oldest first; the first is it being sent.
    pub history: Vec<TaskStep>,
}

impl UnitPart {
    fn new(unit: String, at: Hlc, by: UserId) -> Self {
        Self {
            unit,
            status: TaskStatus::Open,
            history: vec![TaskStep {
                status: TaskStatus::Open,
                at,
                by,
            }],
        }
    }

    /// Moves it on, if `status` may follow; whether it did.
    fn step(&mut self, status: TaskStatus, at: Hlc, by: &UserId) -> bool {
        if !self.status.next().contains(&status) {
            return false;
        }
        self.status = status;
        self.history.push(TaskStep {
            status,
            at,
            by: by.clone(),
        });
        true
    }
}

/// A task with its current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    /// As given.
    pub given: TaskGiven,
    /// Current status: the Auftrag's, following its units where it has some.
    pub status: TaskStatus,
    /// Current follow-up time.
    pub follow_up: Option<u64>,
    /// Its units, in the order they were sent (REP-2, ADR-0085).
    pub units: Vec<UnitPart>,
    /// Patients taken over from it (REP-3).
    pub patients: Vec<TakenOver>,
    /// Every change, oldest first; the first is the task being given.
    pub history: Vec<TaskStep>,
}

impl Task {
    /// When it was given.
    pub fn given_at(&self) -> Hlc {
        self.history[0].at
    }

    /// Whether a follow-up is due at `now_ms`: the time has passed and the task is open.
    pub fn is_due(&self, now_ms: u64) -> bool {
        self.status.is_open() && self.follow_up.is_some_and(|t| t <= now_ms)
    }

    /// Whether it names someone now: a unit not withdrawn, a position or a person.
    pub fn is_assigned(&self) -> bool {
        self.units.iter().any(|p| p.status != TaskStatus::Cancelled)
            || named(self.given.sector.as_ref())
            || named(self.given.person.as_ref())
    }

    /// The part of `unit`, if it is on the Auftrag.
    pub fn part(&self, unit: &str) -> Option<&UnitPart> {
        self.units.iter().find(|p| p.unit == unit)
    }

    /// The units on it that were not withdrawn, in the order they were sent.
    pub fn unit_names(&self) -> Vec<&str> {
        self.units
            .iter()
            .filter(|p| p.status != TaskStatus::Cancelled)
            .map(|p| p.unit.as_str())
            .collect()
    }

    /// The Auftrag follows its units: in progress once one of them is (or is done), done
    /// once every unit is done or withdrawn and one is done.
    fn follow_units(&mut self, at: Hlc, by: &UserId) {
        if !self.status.is_open() || self.units.is_empty() {
            return;
        }
        let started = self
            .units
            .iter()
            .any(|p| matches!(p.status, TaskStatus::InProgress | TaskStatus::Done));
        let ended = self.units.iter().all(|p| !p.status.is_open());
        let done = self.units.iter().any(|p| p.status == TaskStatus::Done);
        let status = if ended && done {
            TaskStatus::Done
        } else if started {
            TaskStatus::InProgress
        } else {
            return;
        };
        if self.status != status && self.status.next().contains(&status) {
            self.status = status;
            self.history.push(TaskStep {
                status,
                at,
                by: by.clone(),
            });
        }
    }

    /// How far it is (TASK-2): an open Auftrag without anyone has been received.
    pub fn phase(&self) -> Phase {
        match self.status {
            TaskStatus::Open if self.is_assigned() => Phase::Assigned,
            TaskStatus::Open => Phase::Received,
            TaskStatus::InProgress => Phase::InProgress,
            TaskStatus::Done => Phase::Done,
            TaskStatus::Cancelled => Phase::Cancelled,
        }
    }

    /// The Auftrag whose pin it shows: the first one of a linked group (REP-2).
    pub fn pin_task(&self) -> &str {
        self.given.follows.as_deref().unwrap_or(&self.given.task)
    }

    /// Whether `user` (an account) is the person it is for.
    pub fn is_for(&self, user: &str) -> bool {
        self.given
            .person
            .as_deref()
            .is_some_and(|p| p.trim().eq_ignore_ascii_case(user.trim()))
    }
}

/// All tasks of an incident, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskList {
    /// Tasks by ID.
    pub tasks: BTreeMap<String, Task>,
}

impl TaskList {
    // [impl->dsn~tasks~8]
    /// Computes the list. The result doesn't depend on the order of `events`: a task given
    /// twice keeps its first version, updates apply in clock order and never go back,
    /// updates for unknown tasks are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut given: Vec<(Hlc, UserId, TaskGiven)> = Vec::new();
        let mut updates: Vec<(Hlc, UserId, Change)> = Vec::new();
        for e in events.iter().filter(|e| e.body.module.as_str() == MODULE) {
            let (at, by) = (e.body.hlc, e.body.author.user.clone());
            let change = match e.body.event_type.as_str() {
                GIVEN => {
                    if let Ok(t) = codec::from_cbor(&e.payload) {
                        given.push((at, by, t));
                    }
                    continue;
                }
                UPDATED => codec::from_cbor(&e.payload).ok().map(Change::Updated),
                ASSIGNED => codec::from_cbor(&e.payload)
                    .ok()
                    .map(|a| Change::Assigned(a, e.body.schema_version)),
                PATIENT_REGISTERED => codec::from_cbor(&e.payload).ok().map(Change::Patient),
                _ => None,
            };
            if let Some(change) = change {
                updates.push((at, by, change));
            }
        }
        given.sort_by_key(|(at, _, _)| *at);
        updates.sort_by_key(|(at, _, _)| *at);
        let mut tasks = BTreeMap::new();
        let mut seen = HashSet::new();
        for (at, by, t) in given {
            if !seen.insert(t.task.clone()) {
                continue;
            }
            let units = t
                .unit
                .clone()
                .filter(|u| !u.trim().is_empty())
                .map(|u| UnitPart::new(u, at, by.clone()))
                .into_iter()
                .collect();
            tasks.insert(
                t.task.clone(),
                Task {
                    follow_up: t.follow_up,
                    units,
                    patients: Vec::new(),
                    given: t,
                    status: TaskStatus::Open,
                    history: vec![TaskStep {
                        status: TaskStatus::Open,
                        at,
                        by,
                    }],
                },
            );
        }
        for (at, by, change) in updates {
            let Some(t) = tasks.get_mut(change.task()) else {
                continue;
            };
            if at <= t.given_at() {
                continue;
            }
            match change {
                // A unit's step (ADR-0085): its part moves on, and the Auftrag follows.
                Change::Updated(TaskUpdated {
                    status: Some(status),
                    unit: Some(unit),
                    ..
                }) if t.status.is_open() => {
                    if let Some(part) = t.units.iter_mut().find(|p| p.unit == unit)
                        && part.step(status, at, &by)
                    {
                        t.follow_units(at, &by);
                    }
                }
                Change::Updated(u) => {
                    if u.unit.is_none()
                        && let Some(status) = u.status
                        && t.status.next().contains(&status)
                    {
                        t.status = status;
                        t.history.push(TaskStep {
                            status,
                            at,
                            by: by.clone(),
                        });
                        // The whole Auftrag's step is every open unit's too.
                        for part in &mut t.units {
                            part.step(status, at, &by);
                        }
                    }
                    if u.follow_up.is_some() {
                        t.follow_up = u.follow_up;
                    }
                }
                // Version 1: the one unit, named before the work starts.
                Change::Assigned(a, 1) => {
                    if t.status == TaskStatus::Open && !a.unit.trim().is_empty() {
                        t.units = vec![UnitPart::new(a.unit, at, by)];
                    }
                }
                // A further unit on an open Auftrag, never twice.
                Change::Assigned(a, _) => {
                    if t.status.is_open()
                        && !a.unit.trim().is_empty()
                        && !t.units.iter().any(|p| p.unit == a.unit)
                    {
                        t.units.push(UnitPart::new(a.unit, at, by));
                    }
                }
                Change::Patient(p) => t.patients.push(TakenOver {
                    unit: p.unit,
                    at,
                    by,
                }),
            }
        }
        Self { tasks }
    }

    // [impl->dsn~tasks~8]
    /// The open tasks (TASK-3): due follow-ups first (earliest due first), then the oldest.
    pub fn open(&self, now_ms: u64) -> Vec<&Task> {
        let mut open: Vec<&Task> = self.tasks.values().filter(|t| t.status.is_open()).collect();
        open.sort_by_key(|t| {
            (
                !t.is_due(now_ms),
                if t.is_due(now_ms) { t.follow_up } else { None },
                t.given_at(),
            )
        });
        open
    }

    // [impl->dsn~reports~3]
    /// Every task about a report, open or not, oldest first.
    pub fn of_report(&self, report: &str) -> Vec<&Task> {
        let mut list: Vec<&Task> = self
            .tasks
            .values()
            .filter(|t| t.given.report.as_deref() == Some(report))
            .collect();
        list.sort_by_key(|t| t.given_at());
        list
    }

    /// The open tasks on which `unit` still works, oldest first.
    pub fn of_unit(&self, unit: &str) -> Vec<&Task> {
        let mut list: Vec<&Task> = self
            .tasks
            .values()
            .filter(|t| t.status.is_open() && t.part(unit).is_some_and(|p| p.status.is_open()))
            .collect();
        list.sort_by_key(|t| t.given_at());
        list
    }

    /// The open tasks for one person (an account), oldest first (UHS-15).
    pub fn of_person(&self, user: &str) -> Vec<&Task> {
        let mut list: Vec<&Task> = self
            .tasks
            .values()
            .filter(|t| t.status.is_open() && t.is_for(user))
            .collect();
        list.sort_by_key(|t| t.given_at());
        list
    }

    /// Whether the pin of `task` (the first of a linked group) is still needed: one of its
    /// Aufträge is open (REP-4, REP-5).
    pub fn pin_needed(&self, task: &str) -> bool {
        self.tasks
            .values()
            .any(|t| t.status.is_open() && t.pin_task() == task)
    }
}

/// A change to a given task, applied in clock order.
enum Change {
    Updated(TaskUpdated),
    /// With the payload's schema version.
    Assigned(TaskAssigned, u32),
    Patient(PatientRegistered),
}

impl Change {
    fn task(&self) -> &str {
        match self {
            Self::Updated(u) => &u.task,
            Self::Assigned(a, _) => &a.task,
            Self::Patient(p) => &p.task,
        }
    }
}

/// A task template of the profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    /// Stable ID.
    pub id: String,
    /// Short label.
    pub label: String,
    /// The text it starts with.
    pub text: String,
}

/// The task templates of the profile (`tasks/templates.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateCatalogue {
    /// Templates in the order they are offered.
    #[serde(rename = "template", default)]
    pub templates: Vec<Template>,
}

impl TemplateCatalogue {
    /// Checks that IDs are unique and every template has a text.
    pub fn check(&self) -> Result<(), String> {
        let mut seen = HashSet::new();
        for t in &self.templates {
            if !seen.insert(t.id.as_str()) {
                return Err(format!("template {:?} appears twice", t.id));
            }
            if t.text.trim().is_empty() {
                return Err(format!("template {:?} has no text", t.id));
            }
        }
        Ok(())
    }
}

// [impl->dsn~package-registry~1]
/// The kinds of event this module writes, with their compartment and schema version.
pub mod kinds {
    use ordo_sync::EventKind;

    /// See [`super::GIVEN`].
    pub const GIVEN: EventKind = EventKind::new(
        super::COMPARTMENT,
        super::MODULE,
        super::GIVEN,
        super::GIVEN_SCHEMA,
    );

    /// See [`super::UPDATED`].
    pub const UPDATED: EventKind = EventKind::new(
        super::COMPARTMENT,
        super::MODULE,
        super::UPDATED,
        super::UPDATED_SCHEMA,
    );

    /// See [`super::ASSIGNED`].
    pub const ASSIGNED: EventKind = EventKind::new(
        super::COMPARTMENT,
        super::MODULE,
        super::ASSIGNED,
        super::ASSIGNED_SCHEMA,
    );

    /// See [`super::PATIENT_REGISTERED`].
    pub const PATIENT_REGISTERED: EventKind = EventKind::new(
        super::COMPARTMENT,
        super::MODULE,
        super::PATIENT_REGISTERED,
        1,
    );

    /// Every kind above, e.g. for the incident log to check it names them all.
    pub const ALL: &[EventKind] = &[GIVEN, UPDATED, ASSIGNED, PATIENT_REGISTERED];
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

    fn given(n: u64, id: &str, unit: &str, follow_up: Option<u64>) -> AppliedEvent {
        let t = TaskGiven {
            task: id.into(),
            unit: Some(unit.into()),
            text: "Lagemeldung abgeben".into(),
            template: Some("situation-report".into()),
            follow_up,
            ..TaskGiven::default()
        };
        event(n, "ben", GIVEN, t.encode().unwrap())
    }

    fn update(
        n: u64,
        id: &str,
        status: Option<TaskStatus>,
        follow_up: Option<u64>,
    ) -> AppliedEvent {
        let u = TaskUpdated {
            task: id.into(),
            status,
            follow_up,
            unit: None,
        };
        event(n, "ben", UPDATED, u.encode().unwrap())
    }

    /// A step of one unit's part (schema 2), by one of its crew.
    fn unit_step(n: u64, id: &str, unit: &str, status: TaskStatus) -> AppliedEvent {
        let u = TaskUpdated {
            task: id.into(),
            status: Some(status),
            follow_up: None,
            unit: Some(unit.into()),
        };
        let mut e = event(n, "anna", UPDATED, u.encode().unwrap());
        e.body.schema_version = UPDATED_SCHEMA;
        e
    }

    // [utest->dsn~tasks~8]
    #[test]
    fn tasks_move_forward_with_history_and_never_back() {
        let mut events = vec![
            given(1, "t1", "RTW 1", None),
            update(2, "t1", Some(TaskStatus::InProgress), None),
            update(3, "t1", Some(TaskStatus::Open), None), // a step back: ignored
            update(4, "t1", Some(TaskStatus::Done), None),
            update(5, "unknown", Some(TaskStatus::Done), None),
        ];
        events.reverse();
        let list = TaskList::from_events(&events);
        let t = &list.tasks["t1"];
        assert_eq!(t.status, TaskStatus::Done);
        assert_eq!(
            t.history.iter().map(|s| s.status).collect::<Vec<_>>(),
            [TaskStatus::Open, TaskStatus::InProgress, TaskStatus::Done]
        );
        assert_eq!(list.tasks.len(), 1, "an update alone is no task");
        assert!(list.open(0).is_empty());
    }

    // [utest->dsn~tasks~8]
    #[test]
    fn due_follow_ups_come_first_and_a_new_time_postpones_them() {
        let events = vec![
            given(1, "old", "RTW 1", None),
            given(2, "later", "HLF 1", Some(9_000)),
            given(3, "due", "RTW 1", Some(5_000)),
            given(4, "done", "RTW 1", Some(1_000)),
            update(5, "done", Some(TaskStatus::Done), None),
        ];
        let list = TaskList::from_events(&events);
        let ids = |l: Vec<&Task>| l.iter().map(|t| t.given.task.clone()).collect::<Vec<_>>();
        assert_eq!(
            ids(list.open(6_000)),
            ["due", "old", "later"],
            "due first, then by age"
        );
        assert!(
            !list.tasks["done"].is_due(6_000),
            "a done task is never due"
        );
        assert_eq!(ids(list.of_unit("RTW 1")), ["old", "due"]);

        let mut events = events;
        events.push(update(6, "due", None, Some(20_000)));
        let list = TaskList::from_events(&events);
        assert!(!list.tasks["due"].is_due(6_000), "followed up later");
    }

    #[test]
    fn a_task_needs_a_text_and_someone_and_the_profile_is_valid() {
        let mut t = TaskGiven {
            task: "t".into(),
            unit: Some("RTW 1".into()),
            text: " ".into(),
            ..TaskGiven::default()
        };
        assert_eq!(t.check(), Err(TaskError::NoText));
        t.text = "Erkundung".into();
        t.unit = None;
        assert_eq!(t.check(), Err(TaskError::NoAssignee));
        t.person = Some("lena@ordo.example.org".into());
        assert_eq!(t.check(), Ok(()), "a person is someone");
        // A report waits for its unit, but needs a place, and both are short.
        let mut r = TaskGiven {
            task: "r".into(),
            kind: Some(TaskKind::Patient),
            text: "Person liegt am Boden".into(),
            ..TaskGiven::default()
        };
        assert_eq!(r.check(), Err(TaskError::NoPlace));
        r.place = Some("Bühne Nord".into());
        assert_eq!(r.check(), Ok(()));
        r.text = "x".repeat(TEXT_MAX + 1);
        assert_eq!(r.check(), Err(TaskError::TooLong));
        let de: TemplateCatalogue =
            toml::from_str(include_str!("../../../profiles/de/tasks/templates.toml")).unwrap();
        de.check().unwrap();
    }

    fn report(n: u64, id: &str, follows: Option<&str>) -> AppliedEvent {
        let t = TaskGiven {
            task: id.into(),
            kind: Some(TaskKind::Patient),
            place: Some("Bühne Nord".into()),
            text: "Person liegt am Boden".into(),
            received_at: Some(900),
            follows: follows.map(Into::into),
            ..TaskGiven::default()
        };
        event(n, "ben", GIVEN, t.encode().unwrap())
    }

    /// `tasks.assigned` of version 1: the one unit, named before it starts.
    fn assigned(n: u64, id: &str, unit: &str) -> AppliedEvent {
        let a = TaskAssigned {
            task: id.into(),
            unit: unit.into(),
        };
        event(n, "ben", ASSIGNED, a.encode().unwrap())
    }

    /// `tasks.assigned` of version 2: a further unit.
    fn added(n: u64, id: &str, unit: &str) -> AppliedEvent {
        let mut e = assigned(n, id, unit);
        e.body.schema_version = ASSIGNED_SCHEMA;
        e
    }

    // [utest->dsn~tasks~8]
    // [utest->dsn~reports~3]
    #[test]
    fn several_units_work_on_one_auftrag_and_the_last_one_done_ends_it() {
        let mut events = vec![
            report(1, "r1", None),
            added(2, "r1", "RTW 1"),
            added(3, "r1", "RTW 2"),
            added(4, "r1", "RTW 1"), // already on it: not twice
            unit_step(5, "r1", "RTW 1", TaskStatus::InProgress),
        ];
        let list = TaskList::from_events(&events);
        let t = &list.tasks["r1"];
        assert_eq!(t.unit_names(), ["RTW 1", "RTW 2"]);
        assert_eq!(
            t.phase(),
            Phase::InProgress,
            "one unit works: the Auftrag does"
        );
        assert_eq!(t.part("RTW 2").unwrap().status, TaskStatus::Open);
        assert_eq!(list.of_unit("RTW 2").len(), 1);

        events.push(unit_step(6, "r1", "RTW 1", TaskStatus::Done));
        events.push(unit_step(7, "r1", "RTW 1", TaskStatus::InProgress)); // back: ignored
        let list = TaskList::from_events(&events);
        assert_eq!(
            list.tasks["r1"].status,
            TaskStatus::InProgress,
            "RTW 2 still works"
        );
        assert!(list.of_unit("RTW 1").is_empty(), "done: off RTW 1's list");
        assert!(list.pin_needed("r1"));

        events.push(unit_step(8, "r1", "RTW 2", TaskStatus::Done));
        events.reverse();
        let list = TaskList::from_events(&events);
        let t = &list.tasks["r1"];
        assert_eq!(t.status, TaskStatus::Done, "the last unit done ends it");
        assert_eq!(
            t.history.iter().map(|s| s.status).collect::<Vec<_>>(),
            [TaskStatus::Open, TaskStatus::InProgress, TaskStatus::Done]
        );
        assert!(!list.pin_needed("r1"), "and its pin leaves the map");
        assert!(list.of_unit("RTW 2").is_empty());
    }

    // [utest->dsn~tasks~8]
    #[test]
    fn cancelling_the_auftrag_ends_every_units_part() {
        let events = vec![
            report(1, "r1", None),
            added(2, "r1", "RTW 1"),
            added(3, "r1", "RTW 2"),
            unit_step(4, "r1", "RTW 1", TaskStatus::InProgress),
            update(5, "r1", Some(TaskStatus::Cancelled), None),
            added(6, "r1", "RTW 3"), // ended: no further unit
        ];
        let list = TaskList::from_events(&events);
        let t = &list.tasks["r1"];
        assert_eq!(t.status, TaskStatus::Cancelled);
        assert!(t.units.iter().all(|p| p.status == TaskStatus::Cancelled));
        assert_eq!(t.units.len(), 2);
    }

    // [utest->dsn~tasks~8]
    // [utest->dsn~reports~3]
    #[test]
    fn version_1_names_the_one_unit_before_it_starts() {
        let mut events = vec![
            report(1, "r1", None),
            assigned(2, "r1", "RTW 1"),
            assigned(3, "r1", "RTW 2"), // changed before it started: counts
            update(4, "r1", Some(TaskStatus::InProgress), None),
            assigned(5, "r1", "RTW 3"), // started: ignored
        ];
        let list = TaskList::from_events(&events[..1]);
        assert_eq!(list.tasks["r1"].phase(), Phase::Received);
        assert!(list.of_unit("RTW 1").is_empty());
        events.reverse();
        let list = TaskList::from_events(&events);
        let t = &list.tasks["r1"];
        assert_eq!(t.unit_names(), ["RTW 2"]);
        assert_eq!(t.phase(), Phase::InProgress);
        assert_eq!(
            t.part("RTW 2").unwrap().status,
            TaskStatus::InProgress,
            "the whole Auftrag's step is its unit's too"
        );
        assert_eq!(list.of_unit("RTW 2").len(), 1);
    }

    // [utest->dsn~reports~3]
    #[test]
    fn a_linked_auftrag_shares_the_pin_until_all_of_them_end() {
        let patient = PatientRegistered {
            task: "r2".into(),
            unit: "RTW 2".into(),
        };
        let mut events = vec![
            report(1, "r1", None),
            assigned(2, "r1", "RTW 1"),
            report(3, "r2", Some("r1")),
            assigned(4, "r2", "RTW 2"),
            update(5, "r1", Some(TaskStatus::Done), None),
            event(6, "anna", PATIENT_REGISTERED, patient.encode().unwrap()),
        ];
        let list = TaskList::from_events(&events);
        assert_eq!(list.tasks["r2"].pin_task(), "r1");
        assert!(list.pin_needed("r1"), "r2 is still open");
        assert_eq!(list.tasks["r2"].patients[0].unit, "RTW 2");
        events.push(update(7, "r2", Some(TaskStatus::Done), None));
        assert!(!TaskList::from_events(&events).pin_needed("r1"));
    }

    // [utest->dsn~tasks~8]
    #[test]
    fn a_person_finds_the_auftrag_for_them() {
        let t = TaskGiven {
            task: "p1".into(),
            person: Some("Lena@ordo.example.org".into()),
            place: Some("UHS Nord".into()),
            text: "Zur UHS Nord als Arzt/Ärztin".into(),
            ..TaskGiven::default()
        };
        let list = TaskList::from_events(&[event(1, "ben", GIVEN, t.encode().unwrap())]);
        assert_eq!(list.of_person("lena@ordo.example.org").len(), 1);
        assert_eq!(list.tasks["p1"].phase(), Phase::Assigned);
        assert!(list.of_person("ben@ordo.example.org").is_empty());
    }
}
