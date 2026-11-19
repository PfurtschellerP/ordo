//! The automatic incident log, *Einsatztagebuch* (LOG-1, LOG-3, LOG-6): every event this
//! device holds for the incident, newest first, in words, with time, author and device.
//! The log is computed from the events (`Replica::log`), so it can only grow; a
//! correction is a new entry that links to the one it corrects. Each package puts its own
//! events into words ([`LogPart`], ADR-0010); this module is the shared modules' part.

use std::collections::BTreeMap;

use leptos::prelude::*;
use ordo_core::codec;
use ordo_sync::AppliedEvent;
use serde::de::DeserializeOwned;

use crate::core::{self, Handle};
use crate::shell::registry::PACKAGES;
use crate::text;

/// What other entries of the log say about the things an entry names, by kind and ID:
/// names of UHS (`uhs`), positions, hospitals and places, task texts, the patient of an
/// order. Every package fills in the kinds it knows and reads any.
#[derive(Default)]
pub(crate) struct Names {
    names: BTreeMap<(&'static str, String), String>,
    /// Event ID → time, for the entries corrections refer to.
    times: BTreeMap<String, u64>,
}

impl Names {
    /// Sets the name of `id` of `kind`; later names win.
    pub(crate) fn set(&mut self, kind: &'static str, id: String, name: String) {
        self.names.insert((kind, id), name);
    }

    /// The name of `id` of `kind`, or the ID itself.
    pub(crate) fn get<'a>(&'a self, kind: &'static str, id: &'a str) -> &'a str {
        self.names
            .get(&(kind, id.to_owned()))
            .map_or(id, String::as_str)
    }

    fn from_log(log: &[AppliedEvent]) -> Self {
        let mut names = Self::default();
        // Oldest first, so later names win.
        for e in log.iter().rev() {
            names
                .times
                .insert(e.body.id.to_string(), e.body.hlc.wall_ms);
            for package in PACKAGES {
                (package.log.learn)(e, &mut names);
            }
        }
        names
    }
}

/// The payload of `e`, if it reads as `T`.
pub(crate) fn decode<T: DeserializeOwned>(e: &AppliedEvent) -> Option<T> {
    codec::from_cbor(&e.payload).ok()
}

/// One entry in words, and the entry it corrects (LOG-3), if any.
pub(crate) struct Line {
    pub(crate) text: String,
    pub(crate) corrects: Option<String>,
}

impl From<String> for Line {
    fn from(text: String) -> Self {
        Self {
            text,
            corrects: None,
        }
    }
}

/// How a person is named in the log, from their user ID or a written name.
pub(crate) type Naming<'a> = &'a dyn Fn(&str) -> String;

// [impl->dsn~package-registry~1]
/// A package's part of the log: the names it learns from its events, and its events in
/// words (none for another package's events).
pub(crate) struct LogPart {
    pub(crate) learn: fn(&AppliedEvent, &mut Names),
    pub(crate) describe: fn(&AppliedEvent, &Names, Naming) -> Option<Line>,
}

impl LogPart {
    /// A part with no events.
    pub(crate) const NONE: Self = Self {
        learn: |_, _| {},
        describe: |_, _, _| None,
    };
}

// [impl->dsn~incident-log~1]
/// An event in words, from the package that knows it. An event type no package knows, or
/// cannot read, is shown with its module and type: the log leaves nothing out (LOG-1).
fn describe(e: &AppliedEvent, c: &Names, name: Naming) -> Line {
    PACKAGES
        .iter()
        .find_map(|p| (p.log.describe)(e, c, name))
        .unwrap_or_else(|| {
            text::log_unknown(e.body.module.as_str(), e.body.event_type.as_str()).into()
        })
}

/// The shared modules' part of the log.
pub(crate) const PART: LogPart = LogPart {
    learn: learn_shared,
    describe: describe_shared,
};

fn learn_shared(e: &AppliedEvent, c: &mut Names) {
    match e.body.event_type.as_str() {
        ordo_command::POSITION_CREATED => {
            if let Some(p) = decode::<ordo_command::PositionCreated>(e) {
                let title = format!(
                    "{} {}",
                    crate::shared::command::STRUCTURE.label(&p.kind),
                    p.name
                );
                c.set("position", p.position, title);
            }
        }
        ordo_tasks::GIVEN => {
            if let Some(p) = decode::<ordo_tasks::TaskGiven>(e) {
                let name = match (p.kind, &p.place) {
                    (Some(kind), Some(place)) => text::task_label(kind, place),
                    _ => p.text.clone(),
                };
                c.set("task", p.task, name);
            }
        }
        ordo_reports::RECORDED => {
            if let Some(p) = decode::<ordo_reports::Recorded>(e) {
                c.set(
                    "report",
                    p.report.clone(),
                    crate::shared::reports::label(&p),
                );
            }
        }
        ordo_map::PLACED => {
            if let Some(p) = decode::<ordo_map::ObjectPlaced>(e) {
                c.set("object", p.object, p.label);
            }
        }
        _ => {}
    }
}

fn describe_shared(e: &AppliedEvent, c: &Names, name: Naming) -> Option<Line> {
    use ordo_sync::incidents as inc;
    Some(match e.body.event_type.as_str() {
        inc::PLANNED => text::log_planned(&decode::<inc::Planned>(e)?.name).into(),
        inc::STATE_CHANGED => {
            text::incident_state_changed(decode::<inc::StateChanged>(e)?.state).into()
        }
        inc::DETAILS_CHANGED => {
            decode::<inc::Details>(e)?;
            text::INCIDENT_DETAILS_SAVED.to_owned().into()
        }
        inc::PAUSE_CHANGED => text::incident_paused(decode::<inc::PauseChange>(e)?.paused)
            .to_owned()
            .into(),
        inc::ROLE_GRANTED => {
            let p = decode::<inc::RoleChange>(e)?;
            text::role_granted(&name(p.user.as_str()), &p.role).into()
        }
        inc::ROLE_WITHDRAWN => {
            let p = decode::<inc::RoleChange>(e)?;
            text::role_withdrawn(&name(p.user.as_str()), &p.role).into()
        }
        ordo_units::ADDED => {
            let p = decode::<ordo_units::UnitAdded>(e)?;
            text::log_unit_added(&p.unit, &p.unit_type, &p.organisation).into()
        }
        ordo_units::STATUS_CHANGED => {
            let p = decode::<ordo_units::StatusChanged>(e)?;
            text::status_set(&p.unit, &p.status).into()
        }
        ordo_units::RELEASED => {
            text::unit_released(&decode::<ordo_units::Released>(e)?.unit).into()
        }
        ordo_units::CREW_SET => {
            let p = decode::<ordo_units::CrewSet>(e)?;
            text::log_crew(&p.unit, p.members.len()).into()
        }
        // [impl->dsn~reports~3]
        ordo_tasks::GIVEN => {
            let p = decode::<ordo_tasks::TaskGiven>(e)?;
            let person = p.person.as_deref().map(name);
            let to = p
                .unit
                .as_deref()
                .or(p.sector.as_deref().map(|s| c.get("position", s)))
                .or(person.as_deref())
                .unwrap_or_default();
            match (p.kind, &p.place) {
                (Some(kind), Some(place)) => text::log_task_received(
                    kind,
                    &crate::core::clock(p.received_at.unwrap_or(e.body.hlc.wall_ms)),
                    place,
                    &p.text,
                    to,
                )
                .into(),
                _ => text::log_task_given(to, &p.text).into(),
            }
        }
        ordo_tasks::ASSIGNED => {
            let p = decode::<ordo_tasks::TaskAssigned>(e)?;
            text::log_task_assigned(c.get("task", &p.task), &p.unit).into()
        }
        ordo_tasks::PATIENT_REGISTERED => {
            let p = decode::<ordo_tasks::PatientRegistered>(e)?;
            text::log_report_patient(c.get("task", &p.task), &p.unit).into()
        }
        // [impl->dsn~reports~3]
        ordo_reports::RECORDED => {
            let p = decode::<ordo_reports::Recorded>(e)?;
            text::log_report_recorded(&p).into()
        }
        ordo_reports::CLOSED => {
            let p = decode::<ordo_reports::Closed>(e)?;
            text::log_report_closed(c.get("report", &p.report)).into()
        }
        ordo_reports::PATIENT_REGISTERED => {
            let p = decode::<ordo_reports::PatientRegistered>(e)?;
            text::log_report_patient(c.get("report", &p.report), &p.unit).into()
        }
        ordo_tasks::UPDATED => {
            let p = decode::<ordo_tasks::TaskUpdated>(e)?;
            let task = c.get("task", &p.task);
            match &p.unit {
                Some(unit) => text::log_unit_step(task, unit, p.status.map(text::task_status)),
                None => text::log_task_updated(task, p.status.map(text::task_status)),
            }
            .into()
        }
        ordo_command::POSITION_CREATED => {
            let p = decode::<ordo_command::PositionCreated>(e)?;
            text::position_created(c.get("position", &p.position)).into()
        }
        ordo_command::POSITION_CLOSED => {
            let p = decode::<ordo_command::PositionClosed>(e)?;
            text::log_position_closed(c.get("position", &p.position)).into()
        }
        ordo_command::LEADER_SET => {
            let p = decode::<ordo_command::LeaderSet>(e)?;
            text::log_leader(c.get("position", &p.position), &name(&p.leader)).into()
        }
        ordo_command::ASSIGNED => {
            let p = decode::<ordo_command::Assigned>(e)?;
            let what = p
                .unit
                .as_deref()
                .or(p.uhs.as_deref().map(|u| c.get("uhs", u)))
                .unwrap_or_default();
            text::log_assigned(what, p.position.as_deref().map(|p| c.get("position", p))).into()
        }
        ordo_map::PLACED => {
            text::log_map_placed(&decode::<ordo_map::ObjectPlaced>(e)?.label).into()
        }
        ordo_map::REMOVED => {
            let p = decode::<ordo_map::ObjectRemoved>(e)?;
            text::log_map_removed(c.get("object", &p.object)).into()
        }
        _ => return None,
    })
}

/// The log of the open incident, as this device knows it.
fn log(core: Handle) -> Vec<AppliedEvent> {
    core.with_value(|c| {
        c.incident
            .as_ref()
            .map(|i| c.replica.log(i))
            .unwrap_or_default()
    })
}

/// The view *Einsatztagebuch*: the entries grouped by day, newest first.
#[component]
pub fn LogView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let days = Memo::new(move |_| {
        revision.track();
        let log = log(core);
        let context = Names::from_log(&log);
        let mut days: Vec<(String, Vec<(AppliedEvent, Line)>)> = Vec::new();
        for e in log {
            let day = core::date(e.body.hlc.wall_ms);
            let line = describe(&e, &context, &|text: &str| {
                crate::shell::profile::display_text(core, text)
            });
            let line = Line {
                corrects: line.corrects.filter(|id| context.times.contains_key(id)),
                ..line
            };
            match days.last_mut() {
                Some((d, entries)) if *d == day => entries.push((e, line)),
                _ => days.push((day, vec![(e, line)])),
            }
        }
        // Times of corrected entries, looked up once.
        let times = context.times;
        days.into_iter()
            .map(|(day, entries)| {
                let entries: Vec<Entry> = entries
                    .into_iter()
                    .map(|(e, line)| Entry {
                        id: e.body.id.to_string(),
                        at: e.body.hlc.wall_ms,
                        text: line.text,
                        corrects: line
                            .corrects
                            .and_then(|id| times.get(&id).map(|at| (id, *at))),
                        by: crate::shell::profile::display_name(core, &e.body.author.user),
                        device: core::short_device(&e.body.author.device),
                        event_type: e.body.event_type.as_str().to_owned(),
                        sent: e.seq.is_some(),
                    })
                    .collect();
                (day, entries)
            })
            .collect::<Vec<_>>()
    });
    view! {
        <section class="card incident-log" aria-labelledby="log-title">
            <h2 id="log-title">{text::LOG_TITLE}</h2>
            <p class="hint">{text::LOG_HINT}</p>
            {move || {
                let days = days.get();
                if days.is_empty() {
                    return view! { <p>{text::LOG_EMPTY}</p> }.into_any();
                }
                days.into_iter()
                    .map(|(day, entries)| {
                        let heading = format!("log-day-{}", day.replace('.', "-"));
                        view! {
                            <h3 id=heading.clone()>{day}</h3>
                            <ol class="log-entries" aria-labelledby=heading>
                                {entries.into_iter().map(|entry| view! { <LogEntry entry /> }).collect_view()}
                            </ol>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </section>
    }
}

/// One entry, ready to show.
#[derive(Clone, PartialEq)]
struct Entry {
    id: String,
    at: u64,
    text: String,
    corrects: Option<(String, u64)>,
    by: String,
    device: String,
    event_type: String,
    sent: bool,
}

#[component]
fn LogEntry(entry: Entry) -> impl IntoView {
    view! {
        <li class="log-entry" id=format!("log-{}", entry.id) data-type=entry.event_type>
            <time class="log-entry__time" datetime=core::iso_time(entry.at)>{core::clock_seconds(entry.at)}</time>
            <span class="log-entry__text">
                {entry.text}
                {entry.corrects.map(|(id, at)| view! {
                    " " <a class="log-entry__corrects" href=format!("#log-{id}")>{text::log_corrects(&core::clock_seconds(at))}</a>
                })}
            </span>
            <span class="log-entry__by">
                {text::log_by(&entry.by, &entry.device)}
                {(!entry.sent).then(|| view! { " · " <span class="log-entry__pending">{text::LOG_PENDING}</span> })}
            </span>
        </li>
    }
}
