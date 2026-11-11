//! Shared module `tasks` (TASK-1…4): tasks the incident command gives to units, with
//! status, history and follow-ups. Shared incident data, no patient data.

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

/// Payload of `tasks.given` (TASK-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
}

/// A task that can't be given.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TaskError {
    /// Nothing to do.
    #[error("the task needs a text")]
    NoText,
    /// Nobody to do it.
    #[error("the task needs a unit or a sector")]
    NoAssignee,
}

impl TaskGiven {
    /// Checks a new task: a text and someone to do it.
    pub fn check(&self) -> Result<(), TaskError> {
        if self.text.trim().is_empty() {
            return Err(TaskError::NoText);
        }
        if self.unit.as_deref().is_none_or(|u| u.trim().is_empty())
            && self.sector.as_deref().is_none_or(|s| s.trim().is_empty())
        {
            return Err(TaskError::NoAssignee);
        }
        Ok(())
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
}

impl TaskUpdated {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
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

/// A task with its current state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    /// As given.
    pub given: TaskGiven,
    /// Current status.
    pub status: TaskStatus,
    /// Current follow-up time.
    pub follow_up: Option<u64>,
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
}

/// All tasks of an incident, computed from events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TaskList {
    /// Tasks by ID.
    pub tasks: BTreeMap<String, Task>,
}

impl TaskList {
    // [impl->dsn~tasks~2]
    /// Computes the list. The result doesn't depend on the order of `events`: a task given
    /// twice keeps its first version, updates apply in clock order and never go back,
    /// updates for unknown tasks are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut given: Vec<(Hlc, UserId, TaskGiven)> = Vec::new();
        let mut updates: Vec<(Hlc, UserId, TaskUpdated)> = Vec::new();
        for e in events.iter().filter(|e| e.body.module.as_str() == MODULE) {
            let who = (e.body.hlc, e.body.author.user.clone());
            match e.body.event_type.as_str() {
                GIVEN => {
                    if let Ok(t) = codec::from_cbor(&e.payload) {
                        given.push((who.0, who.1, t));
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
        given.sort_by_key(|(at, _, _)| *at);
        updates.sort_by_key(|(at, _, _)| *at);
        let mut tasks = BTreeMap::new();
        let mut seen = HashSet::new();
        for (at, by, t) in given {
            if !seen.insert(t.task.clone()) {
                continue;
            }
            tasks.insert(
                t.task.clone(),
                Task {
                    follow_up: t.follow_up,
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
        for (at, by, u) in updates {
            let Some(t) = tasks.get_mut(&u.task) else {
                continue;
            };
            if at <= t.given_at() {
                continue;
            }
            if let Some(status) = u.status
                && t.status.next().contains(&status)
            {
                t.status = status;
                t.history.push(TaskStep { status, at, by });
            }
            if u.follow_up.is_some() {
                t.follow_up = u.follow_up;
            }
        }
        Self { tasks }
    }

    // [impl->dsn~tasks~2]
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

    /// The open tasks of one unit, oldest first.
    pub fn of_unit(&self, unit: &str) -> Vec<&Task> {
        let mut list: Vec<&Task> = self
            .tasks
            .values()
            .filter(|t| t.status.is_open() && t.given.unit.as_deref() == Some(unit))
            .collect();
        list.sort_by_key(|t| t.given_at());
        list
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
            sector: None,
            text: "Lagemeldung abgeben".into(),
            template: Some("situation-report".into()),
            follow_up,
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
        };
        event(n, "ben", UPDATED, u.encode().unwrap())
    }

    // [utest->dsn~tasks~2]
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

    // [utest->dsn~tasks~2]
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
            sector: None,
            text: " ".into(),
            template: None,
            follow_up: None,
        };
        assert_eq!(t.check(), Err(TaskError::NoText));
        t.text = "Erkundung".into();
        t.unit = None;
        assert_eq!(t.check(), Err(TaskError::NoAssignee));
        let de: TemplateCatalogue =
            toml::from_str(include_str!("../../../profiles/de/tasks/templates.toml")).unwrap();
        de.check().unwrap();
    }
}
