//! Incidents and rosters (ADR-0046): choose or plan an incident after logging in; the
//! incident's command grants and withdraws roles, and changes the incident's state.

use leptos::prelude::*;
use ordo_sync::incidents::IncidentState;

use crate::core::{self, Handle, Pairing};
use crate::shared::text;
use crate::shell::views::{after_render, focus};

/// Permission to plan incidents and grant roles in all of them.
const MANAGE_INCIDENTS: &str = "manage_incidents";

// [impl->dsn~incident-roles~1]
/// The incidents the user may open, and — for planners — planning a new one.
#[component]
pub fn IncidentPicker(
    core: Handle,
    revision: RwSignal<u64>,
    paired: RwSignal<Option<Pairing>>,
    on_unpair: impl Fn() + Clone + Send + Sync + 'static,
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
    // Logging out unpairs the device as in the profile menu (SEC-4): also without an
    // incident, so another person can log in on it.
    let me = core.with_value(|c| c.replica.user().clone());
    let log_out = move |_| {
        let device = core.with_value(|c| c.replica.device());
        core::revoke_device(core, device);
        core::forget_device();
        on_unpair();
    };
    after_render(|| focus("incidents-title"));
    view! {
        <main id="main" class="page page--narrow">
            <h1 id="incidents-title" tabindex="-1">{text::app_name()} " – " {text::incidents_title()}</h1>
            <p>{text::incidents_intro()}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::incidents_offline()}</p> }.into_any();
                }
                match incidents.get() {
                    None => view! { <p>{text::incidents_loading()}</p> }.into_any(),
                    Some(list) if list.is_empty() => view! { <p class="notice">{text::incidents_empty()}</p> }.into_any(),
                    Some(list) => view! {
                        <ul class="incident-list" data-keys="list">
                            {list
                                .into_iter()
                                .map(|i| {
                                    let id = i.incident.to_string();
                                    let roles = if i.roles.is_empty() {
                                        text::no_role().to_owned()
                                    } else {
                                        i.roles.iter().map(|r| text::role_label(r)).collect::<Vec<_>>().join(", ")
                                    };
                                    let label = format!("{}: {}", text::open(), i.name);
                                    view! {
                                        <li class="incident-card">
                                            <h2>{i.name.clone()}</h2>
                                            <p>
                                                <span class="incident-card__state">{text::incident_state(i.state, i.paused)}</span>
                                                " · " {text::my_roles()} ": " {roles}
                                            </p>
                                            <button type="button" class="button button--primary" aria-label=label
                                                on:click=move |_| open(id.clone(), None)>
                                                {text::open()}
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
                    <h2 id="plan-title">{text::plan_title()}</h2>
                    <form class="form form--inline" on:submit=plan>
                        <div class="field">
                            <label for="plan-name">{text::plan_name()}</label>
                            <input id="plan-name" type="text" required bind:value=name />
                        </div>
                        <button type="submit" class="button">{text::plan_submit()}</button>
                    </form>
                </section>
            </Show>
            <section class="card" aria-labelledby="log-out-title">
                <h2 id="log-out-title" class="visually-hidden">{text::log_out()}</h2>
                <p>{text::signed_in_as()} " " <strong>{crate::shell::profile::naming({
                    let me = me.clone();
                    move || crate::shell::profile::display_name(core, &me)
                })}</strong> " (" {me.to_string()} ")"</p>
                <p class="hint">{text::log_out_hint()}</p>
                <button id="log-out" type="button" class="button" on:click=log_out>{text::log_out()}</button>
            </section>
        </main>
    }
}

/// Whether the user leads the chosen incident or plans incidents: they grant roles and
/// change its state.
pub fn may_manage(core: Handle) -> bool {
    core.with_value(|c| {
        let me = c.replica.user().clone();
        c.replica.has_permission(MANAGE_INCIDENTS)
            || c.incident_info().is_some_and(|i| {
                i.roster
                    .get(&me)
                    .is_some_and(|r| r.contains(core::COMMANDER))
            })
    })
}

// [impl->dsn~workspaces-by-phase~6]
// [impl->dsn~incident-details~1]
/// The incident itself (*Vorbereitung – Einsatz*): name and state, its description and
/// planned times, and starting, pausing, resuming and closing it.
#[component]
pub fn IncidentPage(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let info = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.incident_info())
    });
    let manage = Memo::new(move |_| {
        revision.track();
        may_manage(core)
    });
    let error = RwSignal::new(None::<String>);
    let set_state = move |state: IncidentState| {
        if let Err(e) = core::set_incident_state(core, revision, state) {
            error.set(Some(e));
        }
    };
    let set_paused = move |paused: bool| {
        if let Err(e) = core::set_incident_paused(core, revision, paused) {
            error.set(Some(e));
        }
    };
    // The form starts with what is stored.
    let stored = info.get_untracked().map(|i| i.details).unwrap_or_default();
    let description = RwSignal::new(stored.description.clone());
    let starts = RwSignal::new(
        stored
            .starts_at
            .map(core::local_input_time)
            .unwrap_or_default(),
    );
    let ends = RwSignal::new(
        stored
            .ends_at
            .map(core::local_input_time)
            .unwrap_or_default(),
    );
    let strength_detailed = RwSignal::new(stored.strength_detailed);
    let details_status = RwSignal::new(None::<String>);
    let editing = RwSignal::new(false);
    let save_details = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let time = |v: String| -> Result<Option<u64>, ()> {
            if v.trim().is_empty() {
                Ok(None)
            } else {
                core::local_time_ms(v.trim()).map(Some).ok_or(())
            }
        };
        let (Ok(starts_at), Ok(ends_at)) =
            (time(starts.get_untracked()), time(ends.get_untracked()))
        else {
            details_status.set(Some(text::incident_times_invalid().to_owned()));
            return;
        };
        let details = ordo_sync::incidents::Details {
            description: description.get_untracked().trim().to_owned(),
            starts_at,
            ends_at,
            strength_detailed: strength_detailed.get_untracked(),
        };
        details_status.set(Some(
            match core::set_incident_details(core, revision, &details) {
                Ok(()) => {
                    editing.set(false);
                    text::incident_details_saved().to_owned()
                }
                Err(e) => e,
            },
        ));
    };
    // [impl->dsn~building-blocks~1]
    // What is stored, read compactly (ADR-0102); the form opens in a dialog.
    let read = move || {
        info.get().map(|i| {
            let d = i.details;
            let none = || text::not_set().to_owned();
            let items = vec![
                (
                    text::incident_description().to_owned(),
                    if d.description.trim().is_empty() {
                        none()
                    } else {
                        d.description.clone()
                    },
                ),
                (
                    text::incident_starts().to_owned(),
                    d.starts_at.map_or_else(none, core::date_time),
                ),
                (
                    text::incident_ends().to_owned(),
                    d.ends_at.map_or_else(none, core::date_time),
                ),
                (
                    text::strength_detailed_short().to_owned(),
                    if d.strength_detailed {
                        text::yes()
                    } else {
                        text::no()
                    }
                    .to_owned(),
                ),
            ];
            view! { <crate::shell::ui::KeyValues items /> }
        })
    };
    view! {
        <section aria-labelledby="incident-title" class="card">
            <h2 id="incident-title">{text::incident_page_title()}</h2>
            {move || info.get().map(|i| {
                let times = text::incident_times(
                    i.details.starts_at.map(core::date_time).as_deref(),
                    i.details.ends_at.map(core::date_time).as_deref(),
                );
                view! {
                    <p>
                        <strong>{i.name.clone()}</strong> " · " {text::incident_state_label()} ": "
                        <span id="incident-state">{text::incident_state(i.state, i.paused)}</span>
                    </p>
                    {times.map(|t| view! { <p id="incident-times">{t}</p> })}
                }
            })}
            <p class="hint">{text::incident_prepare()}</p>
            <Show when=move || manage.get()>
                <div class="row">
                    {move || info.get().map(|i| match (i.state, i.paused) {
                        (IncidentState::Planned, _) => view! {
                            <button type="button" class="button button--primary" on:click=move |_| set_state(IncidentState::Active)>
                                {text::start_incident()}
                            </button>
                        }.into_any(),
                        (IncidentState::Active, paused) => view! {
                            {if paused {
                                view! {
                                    <button type="button" class="button button--primary" on:click=move |_| set_paused(false)>
                                        {text::resume_incident()}
                                    </button>
                                }.into_any()
                            } else {
                                view! {
                                    <button type="button" class="button" on:click=move |_| set_paused(true)>
                                        {text::pause_incident()}
                                    </button>
                                }.into_any()
                            }}
                            <button type="button" class="button" on:click=move |_| set_state(IncidentState::Closed)>
                                {text::close_incident()}
                            </button>
                        }.into_any(),
                        (IncidentState::Closed, _) => ().into_any(),
                    })}
                </div>
                <Show when=move || info.get().is_some_and(|i| i.state == IncidentState::Active)>
                    <p class="hint">{text::incident_pause_hint()}</p>
                </Show>
            </Show>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
        <section aria-labelledby="incident-details-title" class="card">
            <div class="section-head">
                <h2 id="incident-details-title" class="section-head__title">{text::incident_details()}</h2>
                <Show when=move || manage.get()>
                    <div class="section-head__action">
                        <crate::shell::ui::IconButton icon="pencil" word=text::card_edit() id="incident-details-open"
                            label=text::incident_details_edit()
                            on_click=Callback::new(move |()| {
                                details_status.set(None);
                                editing.set(true);
                            }) />
                    </div>
                </Show>
            </div>
            {read}
            // After saving; while the dialog is open, its own line says what went wrong.
            <p role="status">{move || details_status.get().filter(|_| !editing.get())}</p>
            <crate::shell::ui::EditDialog id="incident-details" open=Signal::derive(move || editing.get())
                on_close=Callback::new(move |()| editing.set(false))
                title=Signal::derive(|| text::incident_details().to_owned())
                back=Signal::derive(|| "incident-details-open".to_owned())>
            <form class="form" novalidate on:submit=save_details>
                <div class="field">
                    <label for="incident-description">{text::incident_description()}</label>
                    <textarea id="incident-description" rows="4" aria-describedby="incident-description-hint"
                        maxlength=ordo_sync::incidents::DESCRIPTION_MAX.to_string()
                        disabled=move || !manage.get()
                        prop:value=move || description.get()
                        on:input=move |ev| description.set(event_target_value(&ev))></textarea>
                    <p id="incident-description-hint" class="hint">{text::incident_description_hint()}</p>
                </div>
                <div class="form--inline">
                    <div class="field">
                        <label for="incident-starts">{text::incident_starts()}</label>
                        <input id="incident-starts" type="datetime-local" disabled=move || !manage.get() bind:value=starts />
                    </div>
                    <div class="field">
                        <label for="incident-ends">{text::incident_ends()}</label>
                        <input id="incident-ends" type="datetime-local" disabled=move || !manage.get() bind:value=ends />
                    </div>
                </div>
                // [impl->dsn~detailed-strength~1]
                <div class="field">
                    <label class="choice">
                        <input id="incident-strength-detailed" type="checkbox" disabled=move || !manage.get()
                            bind:checked=strength_detailed />
                        {text::strength_detailed()}
                    </label>
                </div>
                <Show when=move || manage.get()>
                    <div class="row">
                        <button type="submit" class="button">{text::incident_details_save()}</button>
                    </div>
                </Show>
                <p role="alert" class="error">{move || details_status.get().filter(|_| editing.get())}</p>
            </form>
            </crate::shell::ui::EditDialog>
        </section>
    }
}
