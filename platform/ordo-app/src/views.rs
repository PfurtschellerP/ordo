//! Views. Semantic HTML first (A11Y-4): real buttons, labels, fieldsets, tables, a skip
//! link and a live region; all colours and sizes come from design tokens.

use leptos::prelude::*;
use ordo_contracts::TriageCategory::{self, Deceased, Expectant, Immediate, Minor, Urgent};
use ordo_med_patients::{self as patients, PatientRegister, UhsCreated};
use ordo_units::{self as units, UnitRegister};

use crate::core::{self, Connection, Core, Handle, Pairing};
use crate::text;

const CATEGORIES: [TriageCategory; 5] = [Immediate, Urgent, Minor, Expectant, Deceased];

fn css_class(category: TriageCategory) -> &'static str {
    match category {
        Immediate => "triage triage--immediate",
        Urgent => "triage triage--urgent",
        Minor => "triage triage--minor",
        Expectant => "triage triage--expectant",
        Deceased => "triage triage--deceased",
    }
}

// [impl->dsn~triage-badge~1]
/// Colour plus label plus shape, never colour alone (A11Y-3).
#[component]
fn TriageBadge(category: TriageCategory) -> impl IntoView {
    view! {
        <span class=css_class(category) title=text::description(category)>
            <span class="triage__shape" aria-hidden="true"></span>
            <span aria-hidden="true">{text::label(category)}</span>
            <span class="visually-hidden">{text::description(category)}</span>
        </span>
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Med,
    Command,
    Resources,
}

/// The on-site care units of the incident (from the shared compartment).
fn uhs_list(core: Handle) -> Vec<UhsCreated> {
    core.with_value(|c| {
        let events = c
            .replica
            .events(&c.incident, patients::UHS_COMPARTMENT)
            .unwrap_or_default();
        patients::uhs_list(&events)
    })
}

/// Pairing or workspace.
#[component]
pub fn App() -> impl IntoView {
    let pairing = RwSignal::new(core::load_pairing());
    move || match pairing.get() {
        None => view! { <PairingForm pairing /> }.into_any(),
        Some(p) => view! { <Workspace pairing=p on_unpair=move || pairing.set(None) /> }.into_any(),
    }
}

#[component]
fn PairingForm(pairing: RwSignal<Option<Pairing>>) -> impl IntoView {
    let user = RwSignal::new(String::new());
    let device = RwSignal::new(String::new());
    let key = RwSignal::new(String::new());
    let incident = RwSignal::new(core::incident_from_url().unwrap_or_default());
    let error = RwSignal::new(None::<String>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let candidate = Pairing {
            user: user.get(),
            device: device.get(),
            private_key: key.get(),
            incident: incident.get(),
        };
        match Core::new(&candidate) {
            Ok(_) => {
                core::save_pairing(&candidate);
                pairing.set(Some(candidate));
            }
            Err(e) => error.set(Some(e)),
        }
    };
    let new_incident = move |_| {
        if let Some(id) = core::new_incident_id(&user.get()) {
            incident.set(id);
        }
    };
    view! {
        <main id="main" class="page page--narrow">
            <h1>{text::APP_NAME} " – " {text::PAIR_TITLE}</h1>
            <p>{text::PAIR_INTRO}</p>
            <form on:submit=submit class="form">
                <label for="user">{text::PAIR_USER}</label>
                <input id="user" type="text" autocomplete="username" required bind:value=user />
                <label for="device">{text::PAIR_DEVICE}</label>
                <input id="device" type="text" autocomplete="off" spellcheck="false" required bind:value=device />
                <label for="key">{text::PAIR_KEY}</label>
                <input id="key" type="password" autocomplete="off" spellcheck="false" required bind:value=key />
                <label for="incident">{text::PAIR_INCIDENT}</label>
                <div class="row">
                    <input id="incident" type="text" spellcheck="false" required bind:value=incident />
                    <button type="button" class="button" on:click=new_incident>{text::PAIR_NEW_INCIDENT}</button>
                </div>
                <p role="alert" class="error">{move || error.get()}</p>
                <button type="submit" class="button button--primary">{text::PAIR_SUBMIT}</button>
            </form>
        </main>
    }
}

#[component]
fn Workspace(pairing: Pairing, on_unpair: impl Fn() + 'static) -> impl IntoView {
    let core = match Core::new(&pairing) {
        Ok(core) => StoredValue::new_local(core),
        Err(error) => return view! { <p role="alert" class="error">{error}</p> }.into_any(),
    };
    let revision = RwSignal::new(0u64);
    core::connect(core, revision);
    let tab = RwSignal::new(Tab::Med);
    let unpair = move |_| {
        core::forget_device();
        on_unpair();
    };
    let tab_button = move |which: Tab, label: &'static str| {
        view! {
            <button
                type="button"
                class="tab"
                aria-pressed=move || (tab.get() == which).to_string()
                on:click=move |_| tab.set(which)
            >
                {label}
            </button>
        }
    };
    view! {
        <a class="skip-link" href="#main">{text::SKIP_LINK}</a>
        <header class="app-header">
            <p class="app-header__name">{text::APP_NAME}</p>
            <ConnectionStatus core revision />
            <nav aria-label=text::WORKSPACES class="tabs">
                {tab_button(Tab::Med, text::TAB_MED)}
                {tab_button(Tab::Command, text::TAB_COMMAND)}
                {tab_button(Tab::Resources, text::TAB_RESOURCES)}
            </nav>
            <button type="button" class="button button--quiet" on:click=unpair>{text::UNPAIR}</button>
        </header>
        <main id="main" class="page" tabindex="-1">
            {move || match tab.get() {
                Tab::Med => view! { <MedWorkspace core revision /> }.into_any(),
                Tab::Command => view! { <CommandWorkspace core revision /> }.into_any(),
                Tab::Resources => view! { <ResourcesWorkspace core revision /> }.into_any(),
            }}
        </main>
        <p class="visually-hidden" role="status" aria-live="polite">
            {move || {
                revision.track();
                core.with_value(|c| c.announcement.clone())
            }}
        </p>
    }
    .into_any()
}

/// Connection and outbox state, as text (OFF-5).
#[component]
fn ConnectionStatus(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let state = move || {
        revision.track();
        core.with_value(|c| (c.connection, c.replica.outbox_len(), c.error.clone()))
    };
    view! {
        <div class="status" data-state=move || match state().0 {
            Connection::Online => "online",
            Connection::Connecting => "connecting",
            Connection::Offline => "offline",
        }>
            <span class="status__dot" aria-hidden="true"></span>
            <span>
                {move || match state().0 {
                    Connection::Online => text::ONLINE,
                    Connection::Connecting => text::CONNECTING,
                    Connection::Offline => text::OFFLINE,
                }}
            </span>
            <span class="status__pending">{move || text::pending(state().1)}</span>
            {move || state().2.map(|e| view! { <span role="alert" class="error">{e}</span> })}
        </div>
    }
}

#[component]
fn MedWorkspace(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let allowed = move || {
        revision.track();
        core.with_value(|c| c.replica.is_granted(&c.incident, patients::COMPARTMENT))
    };
    move || {
        if allowed() {
            view! {
                <UhsSection core revision />
                <RegisterForm core revision />
                <UhsBoard core revision />
            }
            .into_any()
        } else {
            view! { <p class="notice">{text::NO_CLINICAL_ACCESS}</p> }.into_any()
        }
    }
}

// [impl->dsn~uhs-registry~1]
/// On-site care units with their load, and a form to set one up (UHS-1, UHS-4).
#[component]
fn UhsSection(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let name = RwSignal::new(String::new());
    let capacity = RwSignal::new("10".to_owned());
    let error = RwSignal::new(None::<String>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (n, cap) = (
            name.get().trim().to_owned(),
            capacity.get().trim().parse::<u32>(),
        );
        match cap {
            Ok(cap) if !n.is_empty() && cap > 0 => {
                match core::create_uhs(core, revision, &n, cap) {
                    Ok(()) => {
                        name.set(String::new());
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e)),
                }
            }
            _ => error.set(Some(format!("{}: > 0", text::UHS_CAPACITY))),
        }
    };
    let rows = move || {
        revision.track();
        let counts = core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, patients::COMPARTMENT)
                .unwrap_or_default();
            PatientRegister::from_events(&events).counts()
        });
        uhs_list(core)
            .into_iter()
            .map(|u| {
                let n = counts
                    .get(&u.id)
                    .map_or(0, |c| CATEGORIES.iter().map(|k| c.get(*k)).sum());
                (u.name, n, u.capacity)
            })
            .collect::<Vec<_>>()
    };
    view! {
        <section aria-labelledby="uhs-title" class="card">
            <h2 id="uhs-title">{text::UHS_TITLE}</h2>
            {move || {
                let rows = rows();
                if rows.is_empty() {
                    return view! { <p>{text::UHS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="load-list">
                        {rows
                            .into_iter()
                            .map(|(name, n, cap)| {
                                view! {
                                    <li class="load-list__item">
                                        <span class="load-list__name">{name}</span>
                                        <span>{text::COL_LOAD} ": " {text::load(n, cap)}</span>
                                        <meter min="0" max=cap value=n aria-hidden="true"></meter>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <form on:submit=submit class="form form--inline">
                <div class="field">
                    <label for="uhs-name">{text::UHS_NAME}</label>
                    <input id="uhs-name" type="text" required bind:value=name />
                </div>
                <div class="field field--small">
                    <label for="uhs-capacity">{text::UHS_CAPACITY}</label>
                    <input id="uhs-capacity" type="text" inputmode="numeric" pattern="[0-9]+" required bind:value=capacity />
                </div>
                <button type="submit" class="button">{text::UHS_SUBMIT}</button>
            </form>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}

#[component]
fn RegisterForm(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let tag = RwSignal::new(String::new());
    // Remembered between registrations: a device usually works at one UHS.
    let uhs = RwSignal::new(String::new());
    // No preselection: a forgotten choice must not become a wrong triage category.
    let category = RwSignal::new(None::<TriageCategory>);
    let error = RwSignal::new(None::<String>);
    let list = move || {
        revision.track();
        uhs_list(core)
    };
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let value = tag.get().trim().to_owned();
        let Some(chosen) = category.get() else { return };
        if value.is_empty() || uhs.get().is_empty() {
            return;
        }
        match core::register(core, revision, &value, chosen, &uhs.get()) {
            Ok(()) => {
                tag.set(String::new());
                category.set(None);
                error.set(None);
            }
            Err(e) => error.set(Some(e)),
        }
    };
    view! {
        <section aria-labelledby="register-title" class="card">
            <h2 id="register-title">{text::REGISTER_TITLE}</h2>
            {move || {
                let list = list();
                if list.is_empty() {
                    return view! { <p class="notice">{text::NO_UHS}</p> }.into_any();
                }
                view! {
                    <form on:submit=submit class="form">
                        <label for="tag">{text::REGISTER_TAG}</label>
                        <input id="tag" type="text" inputmode="numeric" autocomplete="off" required bind:value=tag class="input--code" />
                        <fieldset class="choices">
                            <legend>{text::REGISTER_CATEGORY}</legend>
                            {CATEGORIES
                                .into_iter()
                                .map(|c| {
                                    view! {
                                        <label class="choice">
                                            <input
                                                type="radio"
                                                name="category"
                                                required
                                                prop:checked=move || category.get() == Some(c)
                                                on:change=move |_| category.set(Some(c))
                                            />
                                            <TriageBadge category=c />
                                        </label>
                                    }
                                })
                                .collect_view()}
                        </fieldset>
                        <label for="uhs">{text::REGISTER_UHS}</label>
                        <select id="uhs" required on:change=move |ev| uhs.set(event_target_value(&ev)) prop:value=move || uhs.get()>
                            <option value="">{text::UHS_CHOOSE}</option>
                            {list.into_iter().map(|u| view! { <option value=u.id.clone()>{u.name}</option> }).collect_view()}
                        </select>
                        <p role="alert" class="error">{move || error.get()}</p>
                        <button type="submit" class="button button--primary">{text::REGISTER_SUBMIT}</button>
                    </form>
                }
                .into_any()
            }}
        </section>
    }
}

/// The triage tag number of a patient event (both event types carry `tag`).
fn tag_of(event: &ordo_sync::AppliedEvent) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Tag {
        tag: String,
    }
    ordo_core::codec::from_cbor::<Tag>(&event.payload)
        .ok()
        .map(|t| t.tag)
}

#[component]
fn UhsBoard(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = move || {
        revision.track();
        let names: std::collections::HashMap<String, String> =
            uhs_list(core).into_iter().map(|u| (u.id, u.name)).collect();
        core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, patients::COMPARTMENT)
                .unwrap_or_default();
            // Patients with changes that are still in the outbox.
            let pending: Vec<String> = events
                .iter()
                .filter(|e| e.seq.is_none())
                .filter_map(tag_of)
                .collect();
            PatientRegister::from_events(&events)
                .patients()
                .map(|p| {
                    let uhs = names.get(&p.uhs).cloned().unwrap_or_else(|| p.uhs.clone());
                    (
                        p.tag.clone(),
                        uhs,
                        p.category(),
                        p.history.len(),
                        pending.contains(&p.tag),
                    )
                })
                .collect::<Vec<_>>()
        })
    };
    view! {
        <section aria-labelledby="board-title" class="card">
            <h2 id="board-title">{text::BOARD_TITLE}</h2>
            {move || {
                let rows = rows();
                if rows.is_empty() {
                    return view! { <p>{text::BOARD_EMPTY}</p> }.into_any();
                }
                view! {
                    <table class="table">
                        <thead>
                            <tr>
                                <th scope="col">{text::COL_TAG}</th>
                                <th scope="col">{text::COL_CATEGORY}</th>
                                <th scope="col">{text::COL_UHS}</th>
                                <th scope="col">{text::COL_HISTORY}</th>
                                <th scope="col">{text::COL_STATE}</th>
                                <th scope="col">{text::COL_RETRIAGE}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows
                                .into_iter()
                                .map(|(tag, uhs, category, history, pending)| {
                                    let tag_for_buttons = tag.clone();
                                    view! {
                                        <tr>
                                            <th scope="row" class="code">{tag.clone()}</th>
                                            <td><TriageBadge category /></td>
                                            <td>{uhs}</td>
                                            <td>{history}</td>
                                            <td>{if pending { text::NOT_SENT } else { text::SENT }}</td>
                                            <td>
                                                <div class="retriage">
                                                    {CATEGORIES
                                                        .into_iter()
                                                        .filter(|c| *c != category)
                                                        .map(|c| {
                                                            let tag = tag_for_buttons.clone();
                                                            view! {
                                                                <button
                                                                    type="button"
                                                                    class="button button--small"
                                                                    aria-label=format!("{} {}: {}", text::COL_RETRIAGE, tag, text::label(c))
                                                                    on:click=move |_| {
                                                                        let _ = core::retriage(core, revision, &tag, c);
                                                                    }
                                                                >
                                                                    {text::label(c)}
                                                                </button>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </div>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any()
            }}
        </section>
    }
}

#[component]
fn CommandWorkspace(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = move || {
        revision.track();
        let list = uhs_list(core);
        let counts = core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, "incident")
                .unwrap_or_default();
            patients::latest_counts(&events)
        });
        list.into_iter()
            .map(|u| {
                (
                    u.name,
                    u.capacity,
                    counts.get(&u.id).cloned().unwrap_or_default(),
                )
            })
            .collect::<Vec<_>>()
    };
    view! {
        <section aria-labelledby="counts-title" class="card">
            <h2 id="counts-title">{text::COUNTS_TITLE}</h2>
            {move || {
                let rows = rows();
                if rows.is_empty() {
                    return view! { <p>{text::COUNTS_EMPTY}</p> }.into_any();
                }
                view! {
                    <table class="table">
                        <thead>
                            <tr>
                                <th scope="col">{text::COL_UHS}</th>
                                {CATEGORIES.into_iter().map(|c| view! { <th scope="col" class="number"><TriageBadge category=c /></th> }).collect_view()}
                                <th scope="col" class="number">{text::COL_TOTAL}</th>
                                <th scope="col" class="number">{text::COL_LOAD}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows
                                .into_iter()
                                .map(|(name, capacity, c)| {
                                    let total: u32 = CATEGORIES.iter().map(|k| c.get(*k)).sum();
                                    view! {
                                        <tr>
                                            <th scope="row">{name}</th>
                                            {CATEGORIES.into_iter().map(|k| view! { <td class="number">{c.get(k)}</td> }).collect_view()}
                                            <td class="number total">{total}</td>
                                            <td class="number">{text::load(total, capacity)}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any()
            }}
        </section>
    }
}

/// Units (vehicles, teams, other resources) with status (UNIT-1…3).
#[component]
fn ResourcesWorkspace(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let call_sign = RwSignal::new(String::new());
    let unit_type = RwSignal::new(String::new());
    let organisation = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (u, t, o) = (call_sign.get(), unit_type.get(), organisation.get());
        if u.trim().is_empty() || t.trim().is_empty() {
            return;
        }
        match core::add_unit(core, revision, u.trim(), t.trim(), o.trim()) {
            Ok(()) => {
                call_sign.set(String::new());
                error.set(None);
            }
            Err(e) => error.set(Some(e)),
        }
    };
    let register = move || {
        revision.track();
        core.with_value(|c| {
            let events = c
                .replica
                .events(&c.incident, units::COMPARTMENT)
                .unwrap_or_default();
            UnitRegister::from_events(&events)
        })
    };
    view! {
        <section aria-labelledby="units-title" class="card">
            <h2 id="units-title">{text::UNITS_TITLE}</h2>
            {move || {
                register()
                    .lost_updates
                    .into_iter()
                    .map(|l| view! { <p class="notice notice--warning">{text::lost_update(&l.unit, &l.overwritten, &l.kept)}</p> })
                    .collect_view()
            }}
            {move || {
                let units = register().units;
                if units.is_empty() {
                    return view! { <p>{text::UNITS_EMPTY}</p> }.into_any();
                }
                view! {
                    <table class="table">
                        <thead>
                            <tr>
                                <th scope="col">{text::COL_CALL_SIGN}</th>
                                <th scope="col">{text::COL_TYPE}</th>
                                <th scope="col">{text::COL_ORGANISATION}</th>
                                <th scope="col">{text::COL_STATUS}</th>
                                <th scope="col">{text::COL_SET_STATUS}</th>
                            </tr>
                        </thead>
                        <tbody>
                            {units
                                .into_values()
                                .map(|u| {
                                    let unit = u.unit.clone();
                                    let current = u.status.as_ref().map(|s| s.status.clone()).unwrap_or_default();
                                    let shown = if current.is_empty() { text::NO_STATUS.to_owned() } else { text::status_label(&current) };
                                    view! {
                                        <tr>
                                            <th scope="row">{u.unit.clone()}</th>
                                            <td>{u.unit_type}</td>
                                            <td>{u.organisation}</td>
                                            <td>{shown}</td>
                                            <td>
                                                <select
                                                    aria-label=text::set_status_label(&unit)
                                                    on:change=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        if !value.is_empty() {
                                                            let _ = core::set_status(core, revision, &unit, &value);
                                                        }
                                                    }
                                                >
                                                    <option value="" selected=current.is_empty()>{text::COL_SET_STATUS}</option>
                                                    {text::STATUS_CODES
                                                        .iter()
                                                        .map(|(code, _)| view! { <option value=*code selected=*code == current>{text::status_label(code)}</option> })
                                                        .collect_view()}
                                                </select>
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()}
                        </tbody>
                    </table>
                }
                .into_any()
            }}
            <form on:submit=submit class="form form--inline">
                <div class="field">
                    <label for="unit-call-sign">{text::UNIT_CALL_SIGN}</label>
                    <input id="unit-call-sign" type="text" required bind:value=call_sign />
                </div>
                <div class="field">
                    <label for="unit-type">{text::UNIT_TYPE}</label>
                    <input id="unit-type" type="text" required bind:value=unit_type />
                </div>
                <div class="field">
                    <label for="unit-organisation">{text::UNIT_ORGANISATION}</label>
                    <input id="unit-organisation" type="text" bind:value=organisation />
                </div>
                <button type="submit" class="button">{text::UNIT_SUBMIT}</button>
            </form>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}
