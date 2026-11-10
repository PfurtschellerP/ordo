//! Incidents and rosters (ADR-0046): choose or plan an incident after logging in; the
//! incident's command grants and withdraws roles, and changes the incident's state.

use leptos::prelude::*;
use ordo_sync::incidents::IncidentState;

use crate::core::{self, Handle, Pairing};
use crate::text;
use crate::views::{after_render, focus};

/// Permission to plan incidents and grant roles in all of them.
const MANAGE_INCIDENTS: &str = "manage_incidents";

// [impl->dsn~incident-roles~1]
/// The incidents the user may open, and — for planners — planning a new one.
#[component]
pub fn IncidentPicker(
    core: Handle,
    revision: RwSignal<u64>,
    paired: RwSignal<Option<Pairing>>,
) -> impl IntoView {
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    // Ask again after every new connection (an answer may be lost when it is cut).
    Effect::new(move |_| {
        if ready.get() && welcomes.get() > 0 {
            core::list_incidents(core);
        }
    });
    let incidents = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.incidents.clone())
    });
    let planner = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.has_permission(MANAGE_INCIDENTS))
    });
    // Opening an incident (or a new, planned one) changes the pairing; the device view is
    // then rebuilt for that incident.
    let open = move |incident: String, plan: Option<String>| {
        if let Some(mut pairing) = paired.get_untracked() {
            pairing.incident = incident;
            pairing.plan = plan;
            core::save_pairing(&pairing);
            paired.set(Some(pairing));
        }
    };
    let name = RwSignal::new(String::new());
    let plan = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let name = name.get().trim().to_owned();
        if name.is_empty() {
            return;
        }
        let server = core.with_value(|c| c.replica.user().server().to_owned());
        if let Some(id) = core::new_incident_id(&server) {
            open(id, Some(name));
        }
    };
    after_render(|| focus("incidents-title"));
    view! {
        <main id="main" class="page page--narrow">
            <h1 id="incidents-title" tabindex="-1">{text::APP_NAME} " – " {text::INCIDENTS_TITLE}</h1>
            <p>{text::INCIDENTS_INTRO}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::INCIDENTS_OFFLINE}</p> }.into_any();
                }
                match incidents.get() {
                    None => view! { <p>{text::INCIDENTS_LOADING}</p> }.into_any(),
                    Some(list) if list.is_empty() => view! { <p class="notice">{text::INCIDENTS_EMPTY}</p> }.into_any(),
                    Some(list) => view! {
                        <ul class="incident-list" data-keys="list">
                            {list
                                .into_iter()
                                .map(|i| {
                                    let id = i.incident.to_string();
                                    let roles = if i.roles.is_empty() {
                                        text::NO_ROLE.to_owned()
                                    } else {
                                        i.roles.iter().map(|r| text::role_label(r)).collect::<Vec<_>>().join(", ")
                                    };
                                    let label = format!("{}: {}", text::OPEN, i.name);
                                    view! {
                                        <li class="incident-card">
                                            <h2>{i.name.clone()}</h2>
                                            <p>
                                                <span class="incident-card__state">{text::state_label(i.state)}</span>
                                                " · " {text::MY_ROLES} ": " {roles}
                                            </p>
                                            <button type="button" class="button button--primary" aria-label=label
                                                on:click=move |_| open(id.clone(), None)>
                                                {text::OPEN}
                                            </button>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                    .into_any(),
                }
            }}
            <Show when=move || planner.get()>
                <section class="card" aria-labelledby="plan-title">
                    <h2 id="plan-title">{text::PLAN_TITLE}</h2>
                    <form class="form form--inline" on:submit=plan>
                        <div class="field">
                            <label for="plan-name">{text::PLAN_NAME}</label>
                            <input id="plan-name" type="text" required bind:value=name />
                        </div>
                        <button type="submit" class="button">{text::PLAN_SUBMIT}</button>
                    </form>
                </section>
            </Show>
        </main>
    }
}

// [impl->dsn~incident-roles~1]
/// The roster of the chosen incident; its command (or a planner) grants and withdraws
/// roles and changes the state.
#[component]
pub fn RosterView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let info = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.incident_info())
    });
    let manage = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            let me = c.replica.user().clone();
            c.replica.has_permission(MANAGE_INCIDENTS)
                || c.incident_info().is_some_and(|i| {
                    i.roster
                        .get(&me)
                        .is_some_and(|r| r.contains(core::COMMANDER))
                })
        })
    });
    // [impl->dsn~person-suggestions~1]
    // Those who grant roles get the people the server knows as suggestions.
    Effect::new(move || {
        if manage.get() {
            core::list_people(core);
        }
    });
    let people = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.people.iter().map(ToString::to_string).collect::<Vec<_>>())
    });
    let person = RwSignal::new(String::new());
    let role = RwSignal::new(text::ROLES[0].0.to_owned());
    let error = RwSignal::new(None::<String>);
    let grant = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match core::change_role(core, revision, &person.get(), &role.get(), true) {
            Ok(()) => {
                person.set(String::new());
                error.set(None);
            }
            Err(e) => error.set(Some(e)),
        }
    };
    let set_state = move |state: IncidentState| {
        if let Err(e) = core::set_incident_state(core, revision, state) {
            error.set(Some(e));
        }
    };
    view! {
        <section aria-labelledby="roster-title" class="card">
            <h2 id="roster-title">{text::ROSTER_TITLE}</h2>
            {move || info.get().map(|i| view! {
                <p>
                    <strong>{i.name.clone()}</strong> " · " {text::INCIDENT_STATE} ": "
                    <span id="incident-state">{text::state_label(i.state)}</span>
                </p>
            })}
            <Show when=move || manage.get()>
                <div class="row">
                    {move || info.get().map(|i| match i.state {
                        IncidentState::Planned => view! {
                            <button type="button" class="button" on:click=move |_| set_state(IncidentState::Active)>
                                {text::START_INCIDENT}
                            </button>
                        }.into_any(),
                        IncidentState::Active => view! {
                            <button type="button" class="button" on:click=move |_| set_state(IncidentState::Closed)>
                                {text::CLOSE_INCIDENT}
                            </button>
                        }.into_any(),
                        IncidentState::Closed => ().into_any(),
                    })}
                </div>
            </Show>
            {move || {
                // One entry per person, with all their roles.
                let roster: Vec<(String, Vec<String>)> = info
                    .get()
                    .map(|i| {
                        i.roster
                            .iter()
                            .filter(|(_, roles)| !roles.is_empty())
                            .map(|(u, roles)| (u.to_string(), roles.iter().cloned().collect()))
                            .collect()
                    })
                    .unwrap_or_default();
                if roster.is_empty() {
                    return view! { <p>{text::ROSTER_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="roster" data-keys="list">
                        {roster
                            .into_iter()
                            .map(|(user, roles)| {
                                view! {
                                    <li class="roster__entry">
                                        {user.parse::<ordo_core::UserId>().ok().map(|id| view! {
                                            <crate::profile::Avatar core revision user=id />
                                        })}
                                        <span class="roster__person">{user.clone()}</span>
                                        <ul class="roster__roles">
                                            {roles
                                                .into_iter()
                                                .map(|role| {
                                                    let label = text::withdraw_label(&user, &role);
                                                    let (u, r) = (user.clone(), role.clone());
                                                    view! {
                                                        <li class="roster__role">
                                                            {text::role_label(&role)}
                                                            <Show when=move || manage.get()>
                                                                <button type="button" class="button button--small" aria-label=label.clone()
                                                                    on:click={
                                                                        let (u, r) = (u.clone(), r.clone());
                                                                        move |_| {
                                                                            if let Err(e) = core::change_role(core, revision, &u, &r, false) {
                                                                                error.set(Some(e));
                                                                            }
                                                                        }
                                                                    }>
                                                                    {text::WITHDRAW}
                                                                </button>
                                                            </Show>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        </ul>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <Show when=move || manage.get()>
                <form class="form form--inline" on:submit=grant>
                    <div class="field">
                        <label for="roster-person">{text::ROSTER_PERSON}</label>
                        <input id="roster-person" type="text" required autocomplete="off" spellcheck="false"
                            list="roster-person-people" bind:value=person />
                        <crate::ui::Suggestions id="roster-person-people" values=people />
                    </div>
                    <div class="field">
                        <label for="roster-role">{text::ROSTER_ROLE}</label>
                        <select id="roster-role" prop:value=move || role.get() on:change=move |ev| role.set(event_target_value(&ev))>
                            {text::ROLES.iter().map(|(id, label)| view! { <option value=*id>{*label}</option> }).collect_view()}
                        </select>
                    </div>
                    <button type="submit" class="button">{text::GRANT}</button>
                </form>
            </Show>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}
