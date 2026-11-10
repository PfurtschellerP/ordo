//! Transport requests (TRANS-1, TRANS-3): requested on the patient record, assigned and
//! followed by the incident command in its view *Transporte*. The shared request carries
//! no patient data; which patient it is for is only on the record (clinical compartment).

use std::collections::BTreeSet;
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_med_patients::transport::{
    Choice, Transport, TransportBoard, TransportCatalogue, TransportRequested, TransportStatus,
    TransportUpdated, requests_by_patient,
};
use ordo_med_patients::{self as med, PatientRegister};
use ordo_units::UnitRegister;

use crate::core::{self, Handle};
use crate::text;

/// What can be requested, from the profile (checked at build time, see `build.rs`).
pub static CATALOGUE: LazyLock<TransportCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/transport-requests.json"
    )))
    .expect("transport request catalogue was checked at build time")
});

fn label(list: &[Choice], id: &str) -> String {
    TransportCatalogue::label(list, id).to_owned()
}

/// Type, priority and needs of a request as one line, e.g. "RTW · Sofort · Infektiös".
fn request_line(r: &TransportRequested) -> String {
    let c = &*CATALOGUE;
    std::iter::once(label(&c.types, &r.transport_type))
        .chain(std::iter::once(label(&c.priorities, &r.priority)))
        .chain(r.needs.iter().map(|n| label(&c.needs, n)))
        .collect::<Vec<_>>()
        .join(" · ")
}

fn summary(core: Handle, t: &Transport) -> String {
    text::transport_summary(
        &label(&CATALOGUE.types, &t.request.transport_type),
        &core::uhs_name(core, &t.request.uhs),
        &core::clock(t.requested_at().wall_ms),
    )
}

/// Status, unit and destination, the request, and every change with who and when.
#[component]
fn TransportDetails(core: Handle, transport: Transport, show_uhs: bool) -> impl IntoView {
    let t = transport;
    let uhs = show_uhs.then(|| core::uhs_name(core, &t.request.uhs));
    view! {
        <p class="transport-card__status">
            <strong>{text::transport_status(t.status)}</strong>
            {t.unit.clone().map(|u| view! {
                " · " {text::transport_unit(&u, t.destination.as_deref().unwrap_or("–"))}
            })}
        </p>
        <p>{uhs.map(|u| format!("{u} · "))}{request_line(&t.request)}</p>
        <details class="transport-card__history">
            <summary>{text::TRANSPORT_HISTORY}</summary>
            <ol>
                {t.history
                    .iter()
                    .map(|s| view! {
                        <li>{core::clock(s.at.wall_ms)} " " {text::transport_status(s.status)} " · " {s.by.to_string()}</li>
                    })
                    .collect_view()}
            </ol>
        </details>
    }
}

/// Buttons for the next statuses; assigning needs a unit and a destination and is offered
/// apart (incident command).
#[component]
fn StepButtons(core: Handle, revision: RwSignal<u64>, transport: Transport) -> impl IntoView {
    let name = summary(core, &transport);
    let request = transport.request.request.clone();
    view! {
        <div class="row">
            {transport
                .status
                .next()
                .into_iter()
                .filter(|s| *s != TransportStatus::Assigned)
                .map(|status| {
                    let request = request.clone();
                    let step = text::transport_step(status);
                    let class = if status == TransportStatus::Cancelled {
                        "button button--small button--quiet"
                    } else {
                        "button button--small"
                    };
                    view! {
                        <button type="button" class=class
                            aria-label=text::transport_step_label(step, &name)
                            on:click=move |_| {
                                let update = TransportUpdated {
                                    request: request.clone(),
                                    status,
                                    unit: None,
                                    destination: None,
                                };
                                let _ = core::update_transport(core, revision, &update);
                            }>
                            {step}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

// [impl->dsn~transport-request~1]
/// The transports of one patient on the record, and the form to request one while none
/// is going on.
#[component]
pub fn TransportSection(core: Handle, revision: RwSignal<u64>, tag: String) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            core.with_value(|c| {
                let uhs = PatientRegister::from_events(&c.events(med::COMPARTMENT))
                    .patient(&tag)
                    .map(|p| p.uhs.clone());
                let mine = requests_by_patient(&c.events(med::COMPARTMENT))
                    .remove(&tag)
                    .unwrap_or_default();
                let board = TransportBoard::from_events(&c.events(med::transport::COMPARTMENT));
                let transports: Vec<Transport> = mine
                    .iter()
                    .filter_map(|r| board.transports.get(r).cloned())
                    .collect();
                (uhs, transports)
            })
        })
    };
    let transport_type = RwSignal::new(String::new());
    let priority = RwSignal::new(String::new());
    let needs = RwSignal::new(BTreeSet::<String>::new());
    let message = RwSignal::new(None::<&'static str>);
    let submit = {
        let tag = tag.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let Some(uhs) = state.get_untracked().0 else {
                return;
            };
            let request = TransportRequested {
                request: String::new(),
                uhs,
                transport_type: transport_type.get_untracked(),
                priority: priority.get_untracked(),
                // In the order of the profile.
                needs: CATALOGUE
                    .needs
                    .iter()
                    .filter(|n| needs.with_untracked(|s| s.contains(&n.id)))
                    .map(|n| n.id.clone())
                    .collect(),
            };
            if request.check(&CATALOGUE).is_err() {
                message.set(Some(text::TRANSPORT_CHOOSE));
                return;
            }
            if core::request_transport(core, revision, &tag, request).is_ok() {
                transport_type.set(String::new());
                priority.set(String::new());
                needs.set(BTreeSet::new());
                message.set(None);
            }
        }
    };
    let radios = move |name: &'static str, list: &'static [Choice], signal: RwSignal<String>| {
        list.iter()
            .map(|c| {
                let id = c.id.clone();
                let checked = {
                    let id = id.clone();
                    move || signal.get() == id
                };
                let shown = match &c.description {
                    Some(d) => format!("{} – {d}", c.label),
                    None => c.label.clone(),
                };
                view! {
                    <label class="choice chip">
                        <input type="radio" name=name prop:checked=checked
                            on:change=move |_| signal.set(id.clone()) />
                        {shown}
                    </label>
                }
            })
            .collect_view()
    };
    view! {
        <section class="record__section" aria-labelledby="section-transport">
            <h3 id="section-transport" tabindex="-1">{text::TRANSPORT_TITLE}</h3>
            {move || {
                let (_, transports) = state.get();
                if transports.is_empty() {
                    return view! { <p class="hint">{text::TRANSPORT_NONE}</p> }.into_any();
                }
                view! {
                    <ul class="transport-list">
                        {transports
                            .into_iter()
                            .rev()
                            .map(|t| {
                                let open = t.status.is_open();
                                view! {
                                    <li class="transport-card" data-request=t.request.request.clone()>
                                        <TransportDetails core transport=t.clone() show_uhs=false />
                                        {open.then(|| view! { <StepButtons core revision transport=t.clone() /> })}
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <Show when=move || state.with(|(uhs, t)| uhs.is_some() && !t.iter().any(|t| t.status.is_open()))>
                <form class="form transport-request" novalidate on:submit=submit.clone()>
                    <fieldset class="choices">
                        <legend>{text::TRANSPORT_TYPE}</legend>
                        {radios("transport-type", &CATALOGUE.types, transport_type)}
                    </fieldset>
                    <fieldset class="choices">
                        <legend>{text::TRANSPORT_PRIORITY_OF_REQUEST}</legend>
                        {radios("transport-priority", &CATALOGUE.priorities, priority)}
                    </fieldset>
                    <fieldset class="choices">
                        <legend>{text::TRANSPORT_NEEDS}</legend>
                        {CATALOGUE
                            .needs
                            .iter()
                            .map(|n| {
                                let id = n.id.clone();
                                let checked = {
                                    let id = id.clone();
                                    move || needs.with(|s| s.contains(&id))
                                };
                                view! {
                                    <label class="choice chip">
                                        <input type="checkbox" prop:checked=checked
                                            on:change=move |ev| {
                                                let on = event_target_checked(&ev);
                                                needs.update(|s| {
                                                    if on { s.insert(id.clone()); } else { s.remove(&id); }
                                                });
                                            } />
                                        {n.label.clone()}
                                    </label>
                                }
                            })
                            .collect_view()}
                    </fieldset>
                    <p class="error" role="alert">{move || message.get()}</p>
                    <div class="row">
                        <button type="submit" class="button button--primary">{text::TRANSPORT_REQUEST}</button>
                    </div>
                </form>
            </Show>
        </section>
    }
}

// [impl->dsn~transport-request~1]
/// The incident command's view of open transport requests: assign a unit and a
/// destination, follow the status. It shows no patient data, because it has none.
#[component]
pub fn TransportsView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let open = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            TransportBoard::from_events(&c.events(med::transport::COMPARTMENT))
                .open(&CATALOGUE)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let units = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT))
                .units
                .into_keys()
                .collect::<Vec<_>>()
        })
    });
    view! {
        <section aria-labelledby="transports-title" class="card">
            <h2 id="transports-title">{text::TRANSPORTS_TITLE}</h2>
            {move || {
                let list = open.get();
                if list.is_empty() {
                    return view! { <p>{text::TRANSPORTS_EMPTY}</p> }.into_any();
                }
                view! {
                    <ul class="transport-list">
                        {list
                            .into_iter()
                            .map(|t| view! {
                                <li class="transport-card" data-request=t.request.request.clone()>
                                    <h3>{summary(core, &t)}</h3>
                                    <TransportDetails core transport=t.clone() show_uhs=true />
                                    {(t.status == TransportStatus::Requested)
                                        .then(|| view! { <AssignForm core revision transport=t.clone() units /> })}
                                    <StepButtons core revision transport=t.clone() />
                                </li>
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}

/// Assigns a unit of the incident and a destination to a request.
#[component]
fn AssignForm(
    core: Handle,
    revision: RwSignal<u64>,
    transport: Transport,
    units: Memo<Vec<String>>,
) -> impl IntoView {
    let request = transport.request.request.clone();
    let name = summary(core, &transport);
    let unit = RwSignal::new(String::new());
    let destination = RwSignal::new(String::new());
    let message = RwSignal::new(None::<&'static str>);
    let (unit_id, destination_id) = (format!("unit-{request}"), format!("destination-{request}"));
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (u, d) = (
            unit.get_untracked(),
            destination.get_untracked().trim().to_owned(),
        );
        if u.is_empty() || d.is_empty() {
            message.set(Some(text::TRANSPORT_ASSIGN_MISSING));
            return;
        }
        let update = TransportUpdated {
            request: request.clone(),
            status: TransportStatus::Assigned,
            unit: Some(u),
            destination: Some(d),
        };
        if core::update_transport(core, revision, &update).is_ok() {
            message.set(None);
        }
    };
    view! {
        <form class="form transport-assign" novalidate on:submit=submit>
            {move || {
                let list = units.get();
                if list.is_empty() {
                    return view! { <p class="hint">{text::TRANSPORT_NO_UNITS}</p> }.into_any();
                }
                view! {
                    <div class="field">
                        <label for=unit_id.clone()>{text::TRANSPORT_UNIT}</label>
                        <select id=unit_id.clone() on:change=move |ev| unit.set(event_target_value(&ev))>
                            <option value="" selected=move || unit.get().is_empty()>{text::TRANSPORT_CHOOSE_UNIT}</option>
                            {list
                                .into_iter()
                                .map(|u| view! { <option value=u.clone() selected=move || unit.get() == u>{u.clone()}</option> })
                                .collect_view()}
                        </select>
                    </div>
                }
                .into_any()
            }}
            <div class="field">
                <label for=destination_id.clone()>{text::TRANSPORT_DESTINATION}</label>
                <input id=destination_id.clone() type="text" autocomplete="off" bind:value=destination />
            </div>
            <p class="error" role="alert">{move || message.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary"
                    aria-label=text::transport_step_label(text::TRANSPORT_ASSIGN, &name)>
                    {text::TRANSPORT_ASSIGN}
                </button>
            </div>
        </form>
    }
}
