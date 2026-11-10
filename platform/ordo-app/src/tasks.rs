//! Tasks (TASK-1…4): the incident command's open tasks, always at the top of its views,
//! with their next steps, follow-ups, and the form to give a new one.

use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_tasks::{Task, TaskGiven, TaskList, TaskStatus, TaskUpdated, TemplateCatalogue};
use ordo_units::UnitRegister;

use crate::core::{self, Handle};
use crate::text;
use crate::ui::{Action, Actions, Disclosure};

/// Task templates from the profile (`tasks/templates.toml`, checked at build time).
pub static TEMPLATES: LazyLock<TemplateCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/templates.json")))
        .expect("task templates were checked at build time")
});

/// Ten minutes, for "follow up later".
const LATER_MS: u64 = 10 * 60 * 1000;

/// The incident's tasks, computed from the shared events.
pub fn task_list(core: Handle) -> TaskList {
    core.with_value(|c| TaskList::from_events(&c.events(ordo_tasks::COMPARTMENT)))
}

/// Who a task is for: the unit's call sign, or the position's title.
fn assignee(task: &Task, structure: &ordo_command::CommandStructure) -> String {
    match (&task.given.unit, &task.given.sector) {
        (Some(unit), _) => unit.clone(),
        (None, Some(id)) => structure
            .positions
            .get(id)
            .map_or_else(|| id.clone(), crate::command::position_title),
        (None, None) => String::new(),
    }
}

/// One open task: what, for whom, status, follow-up, and the next steps.
#[component]
fn TaskItem(
    core: Handle,
    revision: RwSignal<u64>,
    task: Task,
    /// Who it is for: a unit's call sign or a position's title.
    assignee: String,
    editable: bool,
    now: u64,
) -> impl IntoView {
    let unit = assignee;
    let what = task.given.text.clone();
    let due = task.is_due(now);
    let follow_up = task.follow_up.map(|at| {
        let at = core::clock(at);
        if due {
            text::follow_up_due(&at)
        } else {
            text::follow_up_at(&at)
        }
    });
    let by = text::task_given_by(
        &core::clock(task.given_at().wall_ms),
        task.history[0].by.as_str(),
    );
    let actions = editable.then(|| {
        let id = task.given.task.clone();
        let mut list: Vec<Action> = task
            .status
            .next()
            .into_iter()
            .map(|status| {
                let id = id.clone();
                let step = text::task_step(status);
                Action {
                    text: step,
                    label: text::task_step_label(step, &unit, &what),
                    quiet: status == TaskStatus::Cancelled,
                    run: Callback::new(move |()| {
                        let update = TaskUpdated {
                            task: id.clone(),
                            status: Some(status),
                            follow_up: None,
                        };
                        let _ = core::update_task(core, revision, &update);
                    }),
                }
            })
            .collect();
        if task.follow_up.is_some() {
            list.push(Action {
                text: text::FOLLOW_UP_LATER,
                label: text::task_step_label(text::FOLLOW_UP_LATER, &unit, &what),
                quiet: true,
                run: Callback::new(move |()| {
                    let update = TaskUpdated {
                        task: id.clone(),
                        status: None,
                        follow_up: Some(core::now_ms() + LATER_MS),
                    };
                    let _ = core::update_task(core, revision, &update);
                }),
            });
        }
        list
    });
    view! {
        <li class="task-card" class:task-card--due=due data-task=task.given.task.clone()>
            <p class="task-card__what"><strong>{unit.clone()}</strong> " – " {what.clone()}</p>
            <p class="task-card__status">{text::task_status(task.status)} " · " {by}</p>
            {follow_up.map(|f| view! {
                <p class="task-card__follow-up">{due.then(|| view! { <span class="task-card__due" aria-hidden="true">"⏰ "</span> })}{f}</p>
            })}
            {actions.map(|actions| view! { <Actions actions /> })}
        </li>
    }
}

// [impl->dsn~tasks~1]
/// The open tasks (TASK-3), at the top of every command view: due follow-ups first; the
/// command gives new tasks here (TASK-1).
#[component]
pub fn OpenTasks(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let editable = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, "edit_units"))
        })
    });
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
                        {
                            let structure = crate::command::structure(core);
                            list.into_iter()
                                .map(|task| {
                                    let assignee = assignee(&task, &structure);
                                    view! { <TaskItem core revision task assignee editable now /> }
                                })
                                .collect_view()
                        }
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
                    button=Box::new(|| view! { {text::TASK_GIVE} }.into_any())
                >
                    <GiveTask core revision />
                </Disclosure>
            </Show>
        </section>
    }
}

/// Giving a task to a unit (TASK-1): a template fills in the text, which can be changed;
/// optionally a time to follow up (TASK-4).
#[component]
fn GiveTask(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let unit = RwSignal::new(String::new());
    let template = RwSignal::new(String::new());
    let text_value = RwSignal::new(String::new());
    let minutes = RwSignal::new(String::new());
    let error = RwSignal::new(None::<&'static str>);
    let units = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .present()
                .map(|u| u.unit.clone())
                .collect::<Vec<_>>()
        })
    });
    // [impl->dsn~command-structure~1]
    // Tasks can also go to a position (CMD-1).
    let positions = Memo::new(move |_| {
        revision.track();
        crate::command::position_choices(&crate::command::structure(core))
    });
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let minutes_text = minutes.get_untracked().trim().to_owned();
        let follow_up = if minutes_text.is_empty() {
            None
        } else {
            match minutes_text.parse::<u64>() {
                Ok(m) => Some(core::now_ms() + m * 60 * 1000),
                Err(_) => {
                    error.set(Some(text::TASK_FOLLOW_UP_NUMBER));
                    return;
                }
            }
        };
        let chosen = template.get_untracked();
        let chosen_for = unit.get_untracked();
        let (for_unit, for_sector) = match chosen_for.split_once(':') {
            Some(("position", id)) => (None, Some(id.to_owned())),
            Some(("unit", call_sign)) => (Some(call_sign.to_owned()), None),
            _ => (None, None),
        };
        let task = TaskGiven {
            task: String::new(),
            unit: for_unit,
            sector: for_sector,
            text: text_value.get_untracked().trim().to_owned(),
            template: (!chosen.is_empty()).then_some(chosen),
            follow_up,
        };
        if task.check().is_err() {
            error.set(Some(text::TASK_MISSING));
            return;
        }
        if core::give_task(core, revision, task).is_ok() {
            for s in [template, text_value, minutes] {
                s.set(String::new());
            }
            error.set(None);
        }
    };
    view! {
        <form class="form give-task" novalidate on:submit=submit>
            <div class="form--inline">
                <div class="field">
                    <label for="task-unit">{text::TASK_UNIT}</label>
                    <select id="task-unit" prop:value=move || unit.get() on:change=move |ev| unit.set(event_target_value(&ev))>
                        <option value="">{text::TASK_UNIT_CHOOSE}</option>
                        <optgroup label=text::TAB_RESOURCES>
                            {move || units.get().into_iter().map(|u| view! { <option value=format!("unit:{u}")>{u.clone()}</option> }).collect_view()}
                        </optgroup>
                        <optgroup label=text::STRUCTURE_TITLE>
                            {move || positions.get().into_iter().map(|(id, title)| view! { <option value=format!("position:{id}")>{title}</option> }).collect_view()}
                        </optgroup>
                    </select>
                </div>
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
            </div>
            <div class="field">
                <label for="task-text">{text::TASK_TEXT}</label>
                <input id="task-text" type="text" autocomplete="off" bind:value=text_value />
            </div>
            <div class="field field--small">
                <label for="task-follow-up">{text::TASK_FOLLOW_UP}</label>
                <input id="task-follow-up" type="text" inputmode="numeric" autocomplete="off" bind:value=minutes />
            </div>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button">{text::TASK_GIVE}</button>
            </div>
        </form>
    }
}
