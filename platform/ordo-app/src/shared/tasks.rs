//! Aufträge (TASK-1…4, REP-1…5, ADR-0081): what the incident command gives to a unit, a
//! position or a person, and what reaches it as a report, in one list with one state each.
//! Each card shows kind, place, description, unit and state, and the next steps; the form
//! "Neuer Auftrag" records reports and gives tasks alike, with the pin set on the form.

use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_map::ObjectKind;
use ordo_tasks::{
    Phase, TakenOver, Task, TaskAssigned, TaskGiven, TaskKind, TaskList, TaskStatus, TaskStep,
    TaskUpdated, TemplateCatalogue,
};
use ordo_units::UnitRegister;

use crate::core::{self, Handle};
use crate::shared::text;
use crate::shell::ui::{Action, Actions, Disclosure, Icon};

/// Task templates from the profile (`tasks/templates.toml`, checked at build time).
pub static TEMPLATES: LazyLock<TemplateCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/templates.json")))
        .expect("task templates were checked at build time")
});

/// Ten minutes, for "follow up later".
const LATER_MS: u64 = 10 * 60 * 1000;

/// The ID prefix of an open report (ADR-0075) that no task names yet, shown as received.
const LEGACY: &str = "report:";

// [impl->dsn~reports~3]
/// The incident's Aufträge, computed from the shared events, with the reports of an
/// incident recorded before ADR-0081: a task naming a report shows the report's kind, place
/// and description, and an open report without a task shows as received.
pub fn task_list(core: Handle) -> TaskList {
    let mut list = core.with_value(|c| TaskList::from_events(&c.events(ordo_tasks::COMPARTMENT)));
    for r in crate::shared::reports::list(core).reports.values() {
        let id = &r.recorded.report;
        let mut named = false;
        for t in list
            .tasks
            .values_mut()
            .filter(|t| t.given.report.as_deref() == Some(id.as_str()))
        {
            named = true;
            let g = &mut t.given;
            if g.kind.is_none() {
                g.kind = Some(crate::shared::reports::task_kind(r.recorded.kind));
                g.place = Some(r.recorded.place.clone());
                g.text.clone_from(&r.recorded.text);
                g.received_at = Some(r.recorded.received_at);
            }
            t.patients.extend(r.patients.iter().map(|p| TakenOver {
                unit: p.unit.clone(),
                at: p.at,
                by: p.by.clone(),
            }));
        }
        if !named && r.is_open() {
            let task = format!("{LEGACY}{id}");
            list.tasks.insert(
                task.clone(),
                Task {
                    given: TaskGiven {
                        task,
                        kind: Some(crate::shared::reports::task_kind(r.recorded.kind)),
                        place: Some(r.recorded.place.clone()),
                        text: r.recorded.text.clone(),
                        received_at: Some(r.recorded.received_at),
                        report: Some(id.clone()),
                        ..TaskGiven::default()
                    },
                    status: TaskStatus::Open,
                    follow_up: None,
                    units: Vec::new(),
                    patients: Vec::new(),
                    history: vec![TaskStep {
                        status: TaskStatus::Open,
                        at: r.at,
                        by: r.by.clone(),
                    }],
                },
            );
        }
    }
    list
}

/// Whether it stands for an open report no task names yet (ADR-0075).
fn is_legacy(task: &Task) -> bool {
    task.given.task.starts_with(LEGACY)
}

// [impl->dsn~reports~3]
/// The map object of an Auftrag's pin: the first one's of a linked group, or the report's
/// of an incident recorded before ADR-0081.
pub fn pin_object(task: &Task) -> String {
    match &task.given.report {
        Some(report) => ObjectKind::Report.object_id(report),
        None => ObjectKind::Task.object_id(task.pin_task()),
    }
}

// [impl->dsn~reports~3]
/// The situation map without the pins no open Auftrag needs (REP-5): the map events stay,
/// and the log shows them.
pub fn needed_pins(core: Handle, mut map: ordo_map::SituationMap) -> ordo_map::SituationMap {
    let tasks = task_list(core);
    let reports = crate::shared::reports::list(core);
    map.objects.retain(|_, o| {
        let reference = o.placed.reference.as_deref().unwrap_or_default();
        match o.placed.kind {
            ObjectKind::Task => tasks.pin_needed(reference),
            ObjectKind::Report => {
                reports
                    .reports
                    .get(reference)
                    .is_some_and(ordo_reports::Report::is_open)
                    && tasks
                        .tasks
                        .values()
                        .any(|t| t.status.is_open() && t.given.report.as_deref() == Some(reference))
            }
            _ => true,
        }
    });
    map
}

/// An Auftrag in a few words, for the map, the bell and accessible names: "Patient: Bühne
/// Nord", or its text.
pub fn label(task: &Task) -> String {
    match (task.given.kind, &task.given.place) {
        (Some(kind), Some(place)) => text::task_label(kind, place),
        _ => task.given.text.clone(),
    }
}

/// Who an Auftrag is for: its units' call signs, the position's title or the person's
/// name; empty for nobody yet.
pub(crate) fn assignee(
    core: Handle,
    task: &Task,
    structure: &ordo_command::CommandStructure,
) -> String {
    let units = task.unit_names();
    match (units.is_empty(), &task.given.sector, &task.given.person) {
        (false, _, _) => units.join(", "),
        (true, Some(id), _) => structure
            .positions
            .get(id)
            .map_or_else(|| id.clone(), crate::shared::command::position_title),
        (true, None, Some(person)) => crate::shell::profile::display_text(core, person),
        (true, None, None) => String::new(),
    }
}

/// How far one unit on an Auftrag is, as shown: *Zugewiesen*, *In Arbeit*, *Erledigt*.
fn part_phase(status: TaskStatus) -> &'static str {
    text::task_phase(match status {
        TaskStatus::Open => Phase::Assigned,
        TaskStatus::InProgress => Phase::InProgress,
        TaskStatus::Done => Phase::Done,
        TaskStatus::Cancelled => Phase::Cancelled,
    })
}

/// The steps that may follow `status` for one part of an Auftrag (a unit's, or with `None`
/// the whole Auftrag's), never cancelling: that is the command's, for the whole Auftrag.
fn step_actions(
    (core, revision): (Handle, RwSignal<u64>),
    task: &str,
    unit: Option<String>,
    status: TaskStatus,
    name: &str,
) -> Vec<Action> {
    status
        .next()
        .into_iter()
        .filter(|s| *s != TaskStatus::Cancelled)
        .map(|status| {
            let (task, unit) = (task.to_owned(), unit.clone());
            let step = text::task_step(status);
            Action {
                text: step,
                label: text::task_step_label(step, name),
                quiet: false,
                run: Callback::new(move |()| {
                    let update = TaskUpdated {
                        task: task.clone(),
                        status: Some(status),
                        follow_up: None,
                        unit: unit.clone(),
                    };
                    let _ = core::update_task(core, revision, &update);
                }),
            }
        })
        .collect()
}

/// The call signs of the units present.
fn present_units(core: Handle) -> Vec<String> {
    core.with_value(|c| {
        UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
            .present()
            .map(|u| u.unit.clone())
            .collect()
    })
}

/// What, where and when of a reported Auftrag, with a word and an icon for the kind (never
/// colour alone).
#[component]
fn TaskHead(task: Task, assignee: String) -> impl IntoView {
    let given = task.given;
    match given.kind {
        Some(kind) => {
            let icon = match kind {
                TaskKind::Event => "zap",
                TaskKind::Patient => "users",
            };
            let received = given
                .received_at
                .map(|at| view! { " · " {text::task_received(&core::clock(at))} });
            view! {
                <p class="task-card__kind"><Icon name=icon />{text::task_kind(kind)}{received}</p>
                <p class="task-card__what">
                    <strong>{given.place.clone().unwrap_or_default()}</strong> " – " {given.text.clone()}
                </p>
            }
            .into_any()
        }
        None => view! {
            <p class="task-card__what">
                <strong>{given.place.clone().unwrap_or(assignee)}</strong> " – " {given.text.clone()}
            </p>
        }
        .into_any(),
    }
}

// [impl->dsn~tasks~8]
// [impl->dsn~reports~3]
/// One open Auftrag as a card: what, where, for whom, its state, the follow-up, patients
/// taken over, and the next steps. For the command: assign it, send a further unit, set or
/// move its pin, move each unit's part on, cancel it and ask again later. For the crew
/// (`crew`): its pin on a small map with the link for the maps app, "Patient aufnehmen" on
/// a patient Auftrag, and moving their unit's part on (ADR-0085). With more than one unit,
/// the card lists each with its state.
#[component]
pub(crate) fn TaskItem(
    core: Handle,
    revision: RwSignal<u64>,
    task: Task,
    editable: bool,
    now: u64,
    /// Shown to the unit's crew or the person it is for (UNIT-9): they move it on, but
    /// cancelling it and when to ask again are the incident command's.
    #[prop(optional)]
    crew: bool,
    /// The crew's unit, whose part they move on.
    #[prop(optional)]
    unit: Option<String>,
) -> impl IntoView {
    let who = assignee(core, &task, &crate::shared::command::structure(core));
    let name = label(&task);
    let due = task.is_due(now);
    let phase = task.phase();
    let legacy = is_legacy(&task);
    let follow_up = task.follow_up.filter(|_| !crew).map(|at| {
        let clock = core::clock(at);
        if due {
            text::follow_up_due(&clock).into_any()
        } else {
            view! { <FollowUpCountdown at clock revision /> }.into_any()
        }
    });
    let by = {
        let (at, user) = (
            core::clock(task.given_at().wall_ms),
            task.history[0].by.clone(),
        );
        crate::shell::profile::naming(move || {
            text::task_given_by(&at, &crate::shell::profile::display_name(core, &user))
        })
    };
    let pin = pin_object(&task);
    let placed = crate::shared::situation::placed_at(core, &pin);
    let taken: Vec<String> = task
        .patients
        .iter()
        .map(|p| text::task_taken(&p.unit, &core::clock(p.at.wall_ms)))
        .collect();
    // Whose steps the card offers (ADR-0085): the crew its own unit's part, the command
    // each open unit's, and an Auftrag without units its own.
    let parts: Vec<(Option<String>, TaskStatus)> = match (&unit, crew) {
        (Some(u), true) => task
            .part(u)
            .map(|p| vec![(Some(p.unit.clone()), p.status)])
            .unwrap_or_default(),
        (_, false) if !task.units.is_empty() => task
            .units
            .iter()
            .filter(|p| p.status.is_open())
            .map(|p| (Some(p.unit.clone()), p.status))
            .collect(),
        _ => vec![(None, task.status)],
    };
    // With more than one unit, a row per unit with its state, and for the command its steps.
    let unit_rows = (task.units.len() > 1).then(|| {
        let rows = task
            .units
            .iter()
            .filter(|p| p.status != TaskStatus::Cancelled)
            .map(|p| {
                let steps = (editable && !crew && p.status.is_open()).then(|| {
                    let named = text::unit_in_task(&p.unit, &name);
                    step_actions(
                        (core, revision),
                        &task.given.task,
                        Some(p.unit.clone()),
                        p.status,
                        &named,
                    )
                });
                view! {
                    <li class="task-card__unit" data-unit=p.unit.clone()>
                        <span>{p.unit.clone()} " · " <strong>{part_phase(p.status)}</strong></span>
                        {steps.map(|actions| view! { <Actions actions /> })}
                    </li>
                }
            })
            .collect_view();
        view! { <ul class="task-card__units" aria-label=text::TASK_UNITS>{rows}</ul> }
    });
    let actions = editable.then(|| {
        let id = task.given.task.clone();
        let mut list: Vec<Action> = Vec::new();
        if legacy {
            // An open report without a task (ADR-0075): cancelling closes it.
            let report = task.given.report.clone().unwrap_or_default();
            list.push(Action {
                text: text::task_step(TaskStatus::Cancelled),
                label: text::task_step_label(text::task_step(TaskStatus::Cancelled), &name),
                quiet: true,
                run: Callback::new(move |()| {
                    let _ = core::close_report(core, revision, &report);
                }),
            });
            return list;
        }
        // One part here: the crew's unit, the one unit, or an Auftrag without units. With
        // several units, each one's steps are in its row.
        if phase != Phase::Received
            && (crew || task.units.len() <= 1)
            && let [(part, status)] = parts.as_slice()
        {
            list.extend(step_actions(
                (core, revision),
                &id,
                part.clone(),
                *status,
                &name,
            ));
        }
        // Only the command cancels the whole Auftrag, and can ask again later.
        if !crew {
            let cancel = id.clone();
            list.push(Action {
                text: text::task_step(TaskStatus::Cancelled),
                label: text::task_step_label(text::task_step(TaskStatus::Cancelled), &name),
                quiet: true,
                run: Callback::new(move |()| {
                    let update = TaskUpdated {
                        task: cancel.clone(),
                        status: Some(TaskStatus::Cancelled),
                        follow_up: None,
                        unit: None,
                    };
                    let _ = core::update_task(core, revision, &update);
                }),
            });
            list.push(Action {
                text: text::FOLLOW_UP_LATER,
                label: text::task_step_label(text::FOLLOW_UP_LATER, &name),
                quiet: true,
                run: Callback::new(move |()| {
                    let update = TaskUpdated {
                        task: id.clone(),
                        status: None,
                        follow_up: Some(core::now_ms() + LATER_MS),
                        unit: None,
                    };
                    let _ = core::update_task(core, revision, &update);
                }),
            });
        }
        list
    });
    let command = editable && !crew;
    let reported = task.given.kind.is_some();
    let status_line = if who.is_empty() {
        view! { <strong>{text::task_phase(phase)}</strong> " · " {by} }.into_any()
    } else {
        view! { <strong>{text::task_phase(phase)}</strong> " · " {who.clone()} " · " {by} }
            .into_any()
    };
    let crew_part = crew.then(|| {
        let unit = unit.clone();
        view! { <CrewPart core revision task=task.clone() placed unit /> }
    });
    let command_part = command.then(|| {
        let assign = (phase == Phase::Received)
            .then(|| view! { <AssignForm core revision task=task.clone() further=false /> });
        let further = (reported && phase != Phase::Received && !legacy).then(|| {
            let open = RwSignal::new(false);
            let id = format!("further-{}", core::slug(&task.given.task));
            let task = task.clone();
            view! {
                <Disclosure id open class="task-card__more" button_class="button button--small button--quiet"
                    panel_class="task-card__panel"
                    button=Box::new(|| view! { <Icon name="plus" />{text::TASK_SEND_FURTHER} }.into_any())>
                    <AssignForm core revision task further=true />
                </Disclosure>
            }
        });
        view! {
            {placed.is_some().then(|| view! { <p class="hint"><Icon name="map-pin" />{text::TASK_ON_MAP}</p> })}
            {assign}
            <PinEditor core revision object=pin.clone() label=name.clone() placed />
            {further}
        }
    });
    view! {
        <li class="task-card" class:task-card--due=due class:task-card--patient=task.given.kind == Some(TaskKind::Patient)
            data-task=task.given.task.clone() data-phase=text::task_phase(phase)>
            <TaskHead task=task.clone() assignee=who.clone() />
            <p class="task-card__status">{status_line}</p>
            {unit_rows}
            {follow_up.map(|f| view! {
                <p class="task-card__follow-up">{due.then(|| view! { <span class="task-card__due" aria-hidden="true">"⏰ "</span> })}{f}</p>
            })}
            {taken.into_iter().map(|t| view! { <p class="task-card__taken">{t}</p> }).collect_view()}
            {crew_part}
            {command_part}
            {actions.map(|actions| view! { <Actions actions /> })}
        </li>
    }
}

// [impl->dsn~crew-workspace~3]
/// The crew's part of a card (REP-3, REP-4): the pin on a small map with the link for the
/// maps app, and on a patient Auftrag the unit has not taken a patient from yet, "Patient
/// aufnehmen", which opens the registration in *UHS – Patienten* with it.
#[component]
fn CrewPart(
    core: Handle,
    revision: RwSignal<u64>,
    task: Task,
    placed: Option<[f64; 2]>,
    /// The crew's unit, if the Auftrag is theirs by their unit.
    unit: Option<String>,
) -> impl IntoView {
    let nav = use_context::<crate::shell::views::Nav>();
    let may_register = Memo::new(move |_| {
        revision.track();
        core::granted(core, ordo_med_patients::COMPARTMENT)
    });
    let patient = task.given.kind == Some(TaskKind::Patient)
        && unit
            .as_ref()
            .is_some_and(|u| !task.patients.iter().any(|p| &p.unit == u));
    let object = pin_object(&task);
    let case = unit.map(|unit| crate::shared::reports::Case {
        task: task.given.task.clone(),
        unit,
        place: task.given.place.clone().unwrap_or_default(),
    });
    let take = move |_| {
        if let Some(case) = case.clone() {
            crate::shared::reports::set_pending_case(case);
        }
        if let Some(nav) = nav {
            // In the crew's own workspace (ADR-0093).
            nav.go(crate::shell::nav::Route::view("units.crew", "patients"));
        }
    };
    view! {
        {placed.map(|at| view! {
            <crate::shared::situation::PlaceMap core revision object />
            <p><a href=crate::shared::reports::maps_app_uri(at)><Icon name="map-pin" />{text::OPEN_IN_MAPS_APP}</a></p>
        })}
        {patient.then(|| view! {
            <Show when=move || may_register.get()>
                <div class="row">
                    <button type="button" class="button button--primary" on:click=take.clone()>
                        {text::CASE_REGISTER}
                    </button>
                </div>
            </Show>
        })}
    }
}

// [impl->dsn~reports~3]
/// Assigning a received Auftrag to a unit (REP-2), or with `further` sending a further unit
/// to it (ADR-0085); only units not on it yet are offered.
#[component]
fn AssignForm(core: Handle, revision: RwSignal<u64>, task: Task, further: bool) -> impl IntoView {
    let unit = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let on_it: Vec<String> = task.units.iter().map(|p| p.unit.clone()).collect();
    let units = Memo::new(move |_| {
        revision.track();
        let mut units = present_units(core);
        units.retain(|u| !on_it.contains(u));
        units
    });
    let field = format!(
        "{}-{}",
        if further { "further" } else { "assign" },
        core::slug(&task.given.task)
    );
    let task = StoredValue::new(task);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let chosen = unit.get_untracked();
        if chosen.is_empty() {
            error.set(Some(text::TASK_UNIT_MISSING));
            return;
        }
        let t = task.get_value();
        let done = if is_legacy(&t) {
            // An open report without a task (ADR-0075): an Auftrag naming it.
            let given = TaskGiven {
                task: String::new(),
                unit: Some(chosen),
                report: t.given.report.clone(),
                follow_up: None,
                template: None,
                sector: None,
                person: None,
                ..t.given.clone()
            };
            core::give_task(core, revision, given).is_ok()
        } else {
            let assigned = TaskAssigned {
                task: t.given.task.clone(),
                unit: chosen,
            };
            core::assign_task(core, revision, &assigned).is_ok()
        };
        if done {
            unit.set(String::new());
            error.set(None);
        }
    };
    let label = if further {
        text::TASK_SEND_TO
    } else {
        text::TASK_ASSIGN_TO
    };
    view! {
        <form class="form--inline task-card__assign" novalidate on:submit=submit>
            <div class="field">
                <label for=field.clone()>{label}</label>
                <select id=field prop:value=move || unit.get() on:change=move |ev| unit.set(event_target_value(&ev))>
                    <option value="" disabled selected>{text::TASK_UNIT_ONLY}</option>
                    {move || units.get().into_iter().map(|u| view! { <option value=u.clone()>{u.clone()}</option> }).collect_view()}
                </select>
            </div>
            <button type="submit" class="button button--small">
                {if further { text::TASK_SEND } else { text::TASK_ASSIGN }}
            </button>
            <p role="alert" class="error">{move || error.get()}</p>
        </form>
    }
}

// [impl->dsn~reports~3]
/// Setting or moving the pin of an Auftrag in place (REP-4): the small map of the form,
/// then "Pin speichern"; "Pin entfernen" takes it off the map.
#[component]
fn PinEditor(
    core: Handle,
    revision: RwSignal<u64>,
    object: String,
    label: String,
    placed: Option<[f64; 2]>,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let pin = RwSignal::new(placed);
    let id = format!("pin-{}", core::slug(&object));
    let object = StoredValue::new(object);
    let label = StoredValue::new(label);
    let save = move |_| {
        let done = match pin.get_untracked() {
            Some(at) => crate::shared::situation::set_pin(
                core,
                revision,
                &object.get_value(),
                &label.get_value(),
                at,
            ),
            None => {
                crate::shared::situation::remove_pin(core, revision, &object.get_value());
                true
            }
        };
        if done {
            open.set(false);
        }
    };
    let picker = id.clone();
    view! {
        <Disclosure id open class="task-card__more" button_class="button button--small button--quiet"
            panel_class="task-card__panel"
            label=Some(Signal::derive(move || text::pin_edit_label(placed.is_some(), &label.get_value())))
            button=Box::new(move || view! {
                <Icon name="map-pin" />{if placed.is_some() { text::PIN_MOVE } else { text::PIN_SET }}
            }.into_any())>
            <Show when=move || open.get()>
                <crate::shared::situation::PinPicker core revision id=picker.clone() pin />
                <div class="row">
                    <button type="button" class="button button--small" on:click=save>{text::PIN_SAVE}</button>
                </div>
            </Show>
        </Disclosure>
    }
}

// [impl->dsn~tasks~8]
/// When the incident command asks the unit again: the time and the minutes and seconds
/// left, counted down every second. Not announced each second (no live region); when it
/// reaches zero, the views recompute and the task shows as due.
#[component]
fn FollowUpCountdown(at: u64, clock: String, revision: RwSignal<u64>) -> impl IntoView {
    let now = RwSignal::new(core::now_ms());
    if let Ok(tick) = set_interval_with_handle(
        move || {
            let t = core::now_ms();
            now.set(t);
            if t >= at {
                revision.update(|r| *r += 1);
            }
        },
        std::time::Duration::from_secs(1),
    ) {
        on_cleanup(move || tick.clear());
    }
    view! {
        {text::follow_up_at(&clock)} " · "
        <span class="task-card__countdown">
            {move || text::countdown(at.saturating_sub(now.get()))}
        </span>
    }
}

// [impl->dsn~tasks~8]
/// The shared modules' items for the bell (TASK-3): for the incident command the follow-ups
/// that are due and the Aufträge without anyone, linking to *Disposition*; for everyone else
/// the Aufträge given to them or their unit that nobody has started yet.
pub(crate) fn bell(core: Handle) -> Vec<(String, crate::shell::nav::Route)> {
    let now = core::now_ms();
    let (leads, me) = core.with_value(|c| {
        (
            c.may(core::permission::VIEW_SITUATION),
            c.replica.user().to_string(),
        )
    });
    let tasks = task_list(core);
    let mut items: Vec<(String, crate::shell::nav::Route)> = Vec::new();
    if leads {
        let dispatch = crate::shell::nav::Route::view("command.lead", "dispatch");
        let structure = crate::shared::command::structure(core);
        for t in tasks.open(now) {
            if t.is_due(now) {
                items.push((
                    text::attention_due(&assignee(core, t, &structure), &label(t)),
                    dispatch.clone(),
                ));
            } else if t.phase() == Phase::Received {
                items.push((text::attention_unassigned(&label(t)), dispatch.clone()));
            }
        }
    } else {
        // Not started yet: given to this person, or to a unit whose crew names them.
        let crew = crate::shell::nav::Route::view("units.crew", "tasks");
        let units: Vec<String> = core.with_value(|c| {
            ordo_units::UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .crewed_by(&me)
                .map(|u| u.unit.clone())
                .collect()
        });
        let mine = tasks
            .of_person(&me)
            .into_iter()
            .filter(|t| t.phase() == Phase::Assigned)
            .chain(units.iter().flat_map(|u| {
                // Their unit's part not started yet (ADR-0085).
                tasks
                    .of_unit(u)
                    .into_iter()
                    .filter(|t| t.part(u).is_some_and(|p| p.status == TaskStatus::Open))
            }));
        let mut seen = std::collections::BTreeSet::new();
        for t in mine {
            if seen.insert(t.given.task.clone()) {
                items.push((text::attention_new_task(&label(t)), crew.clone()));
            }
        }
    }
    items
}

// [impl->dsn~tasks~8]
// [impl->dsn~reports~3]
/// The page *Disposition*: all open Aufträge, due follow-ups first, one card each; the form
/// "Neuer Auftrag" for reports and tasks; below, the units with their status, set here
/// (ADR-0070).
#[component]
pub fn OpenTasks(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let editable = core::permitted(core, revision, core::permission::EDIT_UNITS);
    // The app's revision ticks every minute, so "due" follows the clock.
    let open = Memo::new(move |_| {
        revision.track();
        let now = core::now_ms();
        (
            now,
            task_list(core)
                .open(now)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>(),
        )
    });
    let giving = RwSignal::new(false);
    // Whom the next task is for, also chosen from a unit's "Auftrag geben".
    let chosen = RwSignal::new(String::new());
    let kind = RwSignal::new(None::<NewKind>);
    let give = Callback::new(move |unit: String| {
        chosen.set(format!("unit:{unit}"));
        kind.set(Some(NewKind::Other));
        // Opened after this click, which would otherwise close it as a click outside.
        request_animation_frame(move || {
            giving.set(true);
            request_animation_frame(|| crate::shell::views::focus("task-template"));
        });
    });
    // [impl->dsn~map-object-cards~4]
    // Opened from a card on the map: the Auftrag comes into view, or the form opens for the
    // unit.
    crate::shared::object_card::reveal_pending();
    if let Some(unit) = crate::shared::object_card::take_give() {
        give.run(unit);
    }
    view! {
        <section class="card tasks-panel" aria-labelledby="tasks-title">
            <h2 id="tasks-title">{move || text::tasks_title(open.with(|(_, t)| t.len()))}</h2>
            {move || {
                let (now, list) = open.get();
                if list.is_empty() {
                    return view! { <p>{text::TASKS_EMPTY}</p> }.into_any();
                }
                let editable = editable.get();
                view! {
                    <ul class="task-list" data-keys="list">
                        {list.into_iter()
                            .map(|task| view! { <TaskItem core revision task editable now /> })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <Show when=move || editable.get()>
                <Disclosure
                    id="give-task"
                    open=giving
                    class="tasks-panel__give"
                    button_class="button button--small"
                    panel_class="tasks-panel__form"
                    button=Box::new(|| view! { <Icon name="plus" />{text::TASK_NEW} }.into_any())
                >
                    <NewTask core revision unit=chosen kind open=giving />
                </Disclosure>
            </Show>
        </section>
        <crate::shared::units::UnitStatusBoard core revision editable give />
    }
}

/// What a new Auftrag is: a report of a patient or an event, or another task.
#[derive(Clone, Copy, PartialEq, Eq)]
enum NewKind {
    Patient,
    Event,
    Other,
}

impl NewKind {
    fn kind(self) -> Option<TaskKind> {
        match self {
            Self::Patient => Some(TaskKind::Patient),
            Self::Event => Some(TaskKind::Event),
            Self::Other => None,
        }
    }

    fn id(self) -> &'static str {
        match self {
            Self::Patient => "patient",
            Self::Event => "event",
            Self::Other => "other",
        }
    }

    fn label(self) -> &'static str {
        match self.kind() {
            Some(kind) => text::task_kind(kind),
            None => text::TASK_KIND_OTHER,
        }
    }
}

// [impl->dsn~tasks~8]
// [impl->dsn~reports~3]
/// "Neuer Auftrag" (TASK-1, REP-1): the kind; for a report the place, what is going on and
/// when it came in, the unit optional; for another task whom it is for and a template; a
/// time to follow up (TASK-4); and for a report a pin, by address or on a small map, on the
/// same form. `unit` is whom it is for (`unit:<call sign>` or `position:<ID>`).
#[component]
fn NewTask(
    core: Handle,
    revision: RwSignal<u64>,
    unit: RwSignal<String>,
    kind: RwSignal<Option<NewKind>>,
    /// The form's disclosure: closed once the Auftrag is created.
    open: RwSignal<bool>,
) -> impl IntoView {
    let template = RwSignal::new(String::new());
    let place = RwSignal::new(String::new());
    let text_value = RwSignal::new(String::new());
    let received = RwSignal::new(String::new());
    let minutes = RwSignal::new(String::new());
    let pin = RwSignal::new(None::<[f64; 2]>);
    let error = RwSignal::new(None::<&'static str>);
    let units = Memo::new(move |_| {
        revision.track();
        present_units(core)
    });
    // [impl->dsn~command-structure~1]
    // Tasks can also go to a position (CMD-1).
    let positions = Memo::new(move |_| {
        revision.track();
        crate::shared::command::position_choices(&crate::shared::command::structure(core))
    });
    let reported = move || kind.get().is_some_and(|k| k != NewKind::Other);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let Some(chosen_kind) = kind.get_untracked() else {
            error.set(Some(text::TASK_KIND_MISSING));
            return;
        };
        let now = core::now_ms();
        let minutes_text = minutes.get_untracked().trim().to_owned();
        let follow_up = if minutes_text.is_empty() {
            None
        } else {
            match minutes_text.parse::<u64>() {
                Ok(m) => Some(now + m * 60 * 1000),
                Err(_) => {
                    error.set(Some(text::TASK_FOLLOW_UP_NUMBER));
                    return;
                }
            }
        };
        let task_kind = chosen_kind.kind();
        let received_at = match (task_kind, received.get_untracked().trim()) {
            (None, _) => None,
            (Some(_), "") => Some(now),
            (Some(_), time) => match core::today_at(time, now) {
                Some(at) => Some(at),
                None => {
                    error.set(Some(text::TASK_TIME_INVALID));
                    return;
                }
            },
        };
        let chosen = template.get_untracked();
        let chosen_for = unit.get_untracked();
        let (for_unit, for_sector) = match chosen_for.split_once(':') {
            Some(("position", id)) => (None, Some(id.to_owned())),
            Some(("unit", call_sign)) => (Some(call_sign.to_owned()), None),
            _ => (None, None),
        };
        let place_text = place.get_untracked().trim().to_owned();
        let task = TaskGiven {
            task: String::new(),
            unit: for_unit,
            sector: for_sector,
            text: text_value.get_untracked().trim().to_owned(),
            template: (task_kind.is_none() && !chosen.is_empty()).then_some(chosen),
            follow_up,
            kind: task_kind,
            place: (!place_text.is_empty()).then_some(place_text),
            received_at,
            ..TaskGiven::default()
        };
        if let Err(e) = task.check() {
            error.set(Some(match e {
                ordo_tasks::TaskError::NoText => text::TASK_TEXT_MISSING,
                ordo_tasks::TaskError::NoAssignee => text::TASK_MISSING,
                ordo_tasks::TaskError::NoPlace => text::TASK_PLACE_MISSING,
                ordo_tasks::TaskError::TooLong => text::TASK_TOO_LONG,
            }));
            return;
        }
        let shown = match (task.kind, &task.place) {
            (Some(k), Some(p)) => text::task_label(k, p),
            _ => task.text.clone(),
        };
        let Ok(id) = core::give_task(core, revision, task) else {
            return;
        };
        if let Some(at) = pin.get_untracked().filter(|_| task_kind.is_some()) {
            crate::shared::situation::set_pin(
                core,
                revision,
                &ObjectKind::Task.object_id(&id),
                &shown,
                at,
            );
        }
        for s in [template, place, text_value, received, minutes, unit] {
            s.set(String::new());
        }
        kind.set(None);
        pin.set(None);
        error.set(None);
        // The new Auftrag stands in the list; the focus goes back to the button.
        open.set(false);
        crate::shell::views::focus("give-task-button");
    };
    view! {
        <div class="new-task">
            <form id="new-task" class="form new-task__form" novalidate on:submit=submit>
                <fieldset class="choices">
                    <legend>{text::TASK_KIND}</legend>
                    {[NewKind::Patient, NewKind::Event, NewKind::Other].into_iter().map(|k| view! {
                        <label class="choice">
                            <input type="radio" name="task-kind" value=k.id()
                                prop:checked=move || kind.get() == Some(k)
                                on:change=move |_| kind.set(Some(k)) />
                            {k.label()}
                        </label>
                    }).collect_view()}
                </fieldset>
                <div class="form--inline">
                    <div class="field">
                        <label for="task-unit">{text::TASK_UNIT}</label>
                        <select id="task-unit" aria-describedby="task-unit-hint" prop:value=move || unit.get() on:change=move |ev| unit.set(event_target_value(&ev))>
                            <option value="">{move || if reported() { text::TASK_UNIT_LATER } else { text::TASK_UNIT_CHOOSE }}</option>
                            <optgroup label=text::TAB_RESOURCES>
                                {move || units.get().into_iter().map(|u| view! { <option value=format!("unit:{u}")>{u.clone()}</option> }).collect_view()}
                            </optgroup>
                            <optgroup label=text::STRUCTURE_TITLE>
                                {move || positions.get().into_iter().map(|(id, title)| view! { <option value=format!("position:{id}")>{title}</option> }).collect_view()}
                            </optgroup>
                        </select>
                        <p id="task-unit-hint" class="hint">{text::TASK_UNIT_HINT}</p>
                    </div>
                    <Show when=move || kind.get() == Some(NewKind::Other)>
                        <div class="field">
                            <label for="task-template">{text::TASK_TEMPLATE}</label>
                            <select id="task-template" prop:value=move || template.get() on:change=move |ev| {
                                let id = event_target_value(&ev);
                                if let Some(t) = TEMPLATES.templates.iter().find(|t| t.id == id) {
                                    text_value.set(t.text.clone());
                                }
                                template.set(id);
                            }>
                                <option value="">{text::TASK_TEMPLATE_NONE}</option>
                                {TEMPLATES.templates.iter().map(|t| view! { <option value=t.id.clone()>{t.label.clone()}</option> }).collect_view()}
                            </select>
                        </div>
                    </Show>
                </div>
                <div class="field">
                    <label for="task-place">{text::TASK_PLACE}</label>
                    <input id="task-place" type="text" autocomplete="off" maxlength=ordo_tasks::PLACE_MAX bind:value=place />
                </div>
                <div class="field">
                    <label for="task-text">{move || if reported() { text::TASK_WHAT } else { text::TASK_TEXT }}</label>
                    <input id="task-text" type="text" autocomplete="off" aria-describedby="task-text-hint" bind:value=text_value />
                    <p id="task-text-hint" class="hint">{text::TASK_WHAT_HINT}</p>
                </div>
                <div class="task-times">
                    <Show when=reported>
                        <div class="field">
                            <label for="task-received">{text::TASK_RECEIVED}</label>
                            <input id="task-received" type="time" autocomplete="off"
                                aria-describedby="task-received-hint" bind:value=received />
                            <p id="task-received-hint" class="hint">{text::TASK_RECEIVED_HINT}</p>
                        </div>
                    </Show>
                    <div class="field">
                        <label for="task-follow-up">{text::TASK_FOLLOW_UP}</label>
                        <input id="task-follow-up" type="text" inputmode="numeric" autocomplete="off" bind:value=minutes />
                    </div>
                </div>
            </form>
            // The pin's search is a form of its own, so it stands between the fields and
            // the button, which submits the form above.
            <Show when=reported>
                <fieldset class="new-task__pin">
                    <legend>{text::PIN_TITLE}</legend>
                    <crate::shared::situation::PinPicker core revision id="task" pin />
                </fieldset>
            </Show>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" form="new-task" class="button">{text::TASK_CREATE}</button>
            </div>
        </div>
    }
}
