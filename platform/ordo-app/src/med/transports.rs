//! Transport requests (TRANS-1, TRANS-3): requested on the patient record, assigned and
//! followed by the incident command in its view *Transporte*. The shared request carries
//! no patient data; which patient it is for is only on the record (clinical compartment).

use std::collections::BTreeSet;
use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_contracts::TriageCategory;
use ordo_med_patients::hospitals::{
    Capacity, CapacitySet, Hospital, HospitalCatalogue, HospitalList, HospitalSet,
};
use ordo_med_patients::transport::{
    Choice, Transport, TransportBoard, TransportCatalogue, TransportRequested, TransportStatus,
    TransportUpdated, requests_by_patient,
};
use ordo_med_patients::{self as med, PatientRegister};
use ordo_units::UnitRegister;

use crate::core::{self, Handle};
use crate::med::text;
use crate::shell::ui::{Action, Actions, CheckChips, History, RadioChips};

/// What can be requested, from the profile (checked at build time, see `build.rs`).
pub static CATALOGUE: LazyLock<TransportCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/transport-requests.json"
    )))
    .expect("transport request catalogue was checked at build time")
});

/// Hospital departments, capabilities and capacity categories, from the profile.
pub static HOSPITALS: LazyLock<HospitalCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/hospitals.json")))
        .expect("hospital catalogue was checked at build time")
});

/// Choices of the profile as options for chips: ID and label with description.
fn choices(list: &[Choice]) -> Vec<(String, String)> {
    list.iter()
        .map(|c| {
            let shown = match &c.description {
                Some(d) => format!("{} – {d}", c.label),
                None => c.label.clone(),
            };
            (c.id.clone(), shown)
        })
        .collect()
}

/// The chosen IDs in the order of the profile.
fn in_profile_order(list: &[Choice], chosen: &BTreeSet<String>) -> Vec<String> {
    list.iter()
        .filter(|c| chosen.contains(&c.id))
        .map(|c| c.id.clone())
        .collect()
}

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
        label(&CATALOGUE.types, &t.request.transport_type),
        core::uhs_name(core, &t.request.uhs),
        core::clock(t.requested_at().wall_ms),
    )
}

/// The transports waiting for a unit, highest priority first, then the oldest; not those
/// planned for later.
fn waiting(core: Handle) -> Vec<Transport> {
    core.with_value(|c| TransportBoard::from_events(&c.events(med::transport::COMPARTMENT)))
        .open(&CATALOGUE)
        .into_iter()
        .filter(|t| t.status == TransportStatus::Requested)
        .cloned()
        .collect()
}

// [impl->dsn~tasks~8]
/// The bell's items for the incident command: each transport waiting for a unit, linked to
/// *Transporte* (feedback of 2026-11-22: what is important and time-critical).
pub(crate) fn bell(core: Handle) -> Vec<(String, crate::shell::nav::Route)> {
    if !core.with_value(|c| c.may(core::permission::VIEW_SITUATION)) {
        return Vec::new();
    }
    let route = crate::shell::nav::Route::view("command.lead", "transports");
    waiting(core)
        .iter()
        .map(|t| {
            (
                text::transport_waiting(
                    summary(core, t),
                    label(&CATALOGUE.priorities, &t.request.priority).as_str(),
                ),
                route.clone(),
            )
        })
        .collect()
}

// [impl->dsn~tasks~8]
/// The overview's tile *Transporte*: the requests waiting for a unit (the first three, by
/// priority) and how many are under way.
#[component]
pub fn TransportsTile(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let state = Memo::new(move |_| {
        revision.track();
        let open = core.with_value(|c| {
            TransportBoard::from_events(&c.events(med::transport::COMPARTMENT))
                .open(&CATALOGUE)
                .into_iter()
                .filter(|t| t.status != TransportStatus::Requested)
                .count()
        });
        let waiting: Vec<(String, String, String, bool)> = waiting(core)
            .iter()
            .map(|t| {
                (
                    t.request.request.clone(),
                    summary(core, t),
                    label(&CATALOGUE.priorities, &t.request.priority),
                    CATALOGUE.priority_rank(&t.request.priority) == 0,
                )
            })
            .collect();
        (waiting, open)
    });
    let route = crate::shell::nav::Route::view("command.lead", "transports");
    view! {
        <section class="tile tile--transports" aria-labelledby="tile-transports">
            <crate::shared::summary::TileHead id="tile-transports" icon="ambulance" title=text::transports_title_short()
                route=route link=text::open_transports() />
            <p class="tile__figure">
                <span id="overview-transports-waiting" class="tile__number">{move || state.with(|(w, _)| w.len())}</span>
                " " {text::transports_waiting()}
                {move || state.with(|(_, open)| (*open > 0).then(|| view! { <span class="tag">{text::transports_under_way(*open)}</span> }))}
            </p>
            {move || {
                let waiting = state.with(|(w, _)| w.clone());
                if waiting.is_empty() {
                    return view! { <p class="hint">{text::no_transports_waiting()}</p> }.into_any();
                }
                view! {
                    <ul class="tile__list">
                        {waiting.into_iter().take(3).map(|(id, summary, priority, first)| view! {
                            <li class="tile__task" data-request=id>
                                <span class="tile__task-text"><strong>{summary}</strong></span>
                                <span class=if first { "tag tag--warning" } else { "tag" }>{priority}</span>
                            </li>
                        }).collect_view()}
                    </ul>
                }.into_any()
            }}
        </section>
    }
}

/// Status, unit and destination, the request, and every change with who and when.
#[component]
pub(crate) fn TransportDetails(
    core: Handle,
    transport: Transport,
    show_uhs: bool,
) -> impl IntoView {
    let t = transport;
    let uhs = show_uhs.then(|| core::uhs_name(core, &t.request.uhs));
    let state = state_label(&t);
    view! {
        <p class="transport-card__status">
            <strong>{state}</strong>
            {t.unit.clone().map(|u| view! {
                " · " {text::transport_unit(&u, t.destination.as_deref().unwrap_or(text::destination_open()))}
            })}
        </p>
        {t.request.planned.map(|ms| view! {
            <p class="transport-card__planned"><strong>{text::planned_for(core::date_time(ms))}</strong></p>
        })}
        <p>{uhs.map(|u| format!("{u} · "))}{request_line(&t.request)}</p>
        <History entries=crate::shell::profile::naming({
            let history = t.history.clone();
            move || history
                .iter()
                .map(|s| (core::clock(s.at.wall_ms), step_label(s), crate::shell::profile::display_name(core, &s.by)))
                .collect()
        }) />
    }
}

/// The transports requested for a patient, oldest first.
pub fn patient_transports(core: Handle, tag: &str) -> Vec<Transport> {
    core.with_value(|c| {
        let mine = requests_by_patient(&c.events(med::COMPARTMENT))
            .remove(tag)
            .unwrap_or_default();
        let board = TransportBoard::from_events(&c.events(med::transport::COMPARTMENT));
        mine.iter()
            .filter_map(|r| board.transports.get(r).cloned())
            .collect()
    })
}

/// Type, unit and destination of a transport, e.g. "RTW · RTW 83/1 → Klinikum Mitte".
pub fn transport_line(t: &Transport) -> String {
    let kind = label(&CATALOGUE.types, &t.request.transport_type);
    match &t.unit {
        Some(unit) => format!(
            "{kind} · {}",
            text::transport_unit(
                unit,
                t.destination.as_deref().unwrap_or(text::destination_open())
            )
        ),
        None => kind,
    }
}

// [impl->dsn~transport-request~3]
/// A transport's state in words: the unit status of its last step where it has one
/// ("7 – Patient aufgenommen", ADR-0098), else its own status.
pub fn state_label(t: &Transport) -> String {
    match (
        t.status,
        t.history
            .iter()
            .rev()
            .find_map(|h| h.status_code.as_deref()),
    ) {
        (TransportStatus::Cancelled, _) | (_, None) => text::transport_status(t.status).to_owned(),
        (_, Some(code)) => crate::text::status_label(code),
    }
}

/// One step of a transport's history in words: the unit status that moved it, the
/// destination it set, or its status.
pub fn step_label(step: &ordo_med_patients::transport::Step) -> String {
    match (&step.status_code, &step.destination) {
        (Some(code), _) => crate::text::status_label(code),
        // A destination set without assigning a unit: set later (ADR-0098).
        (None, Some(d)) if step.unit.is_none() => text::destination_set(d),
        _ => text::transport_status(step.status).to_owned(),
    }
}

/// The text of a status step's button, kept once per code (actions take `&'static str`).
fn step_text(code: &str) -> &'static str {
    thread_local! {
        static KEPT: std::cell::RefCell<std::collections::HashMap<String, &'static str>> =
            std::cell::RefCell::new(std::collections::HashMap::new());
    }
    KEPT.with(|kept| {
        *kept
            .borrow_mut()
            .entry(code.to_owned())
            .or_insert_with(|| Box::leak(crate::text::status_label(code).into_boxed_str()))
    })
}

/// Buttons for the next steps: the unit statuses the profile names (ADR-0098), each setting
/// the unit's status, which moves the transport; else ORDO's own steps. Cancelling while
/// nobody is aboard; assigning is offered apart (incident command).
#[component]
pub(crate) fn StepButtons(
    core: Handle,
    revision: RwSignal<u64>,
    transport: Transport,
) -> impl IntoView {
    let name = summary(core, &transport);
    let request = transport.request.request.clone();
    let update = move |status: TransportStatus| {
        let request = request.clone();
        Callback::new(move |()| {
            let update = TransportUpdated {
                request: request.clone(),
                status,
                unit: None,
                destination: None,
                hospital: None,
                status_code: None,
            };
            let _ = core::update_transport(core, revision, &update);
        })
    };
    let mut actions: Vec<Action> = if CATALOGUE.steps.is_empty() {
        transport
            .status
            .next()
            .into_iter()
            .filter(|s| !matches!(s, TransportStatus::Assigned | TransportStatus::Cancelled))
            .map(|status| {
                let text = text::transport_step(status);
                Action {
                    text,
                    label: text::transport_step_label(text, &name),
                    quiet: false,
                    run: update(status),
                }
            })
            .collect()
    } else {
        // [impl->dsn~transport-request~3]
        let unit = transport.unit.clone().unwrap_or_default();
        CATALOGUE
            .next_steps(&transport)
            .into_iter()
            .map(|step| {
                let (unit, code) = (unit.clone(), step.status.clone());
                let text = step_text(&code);
                Action {
                    text,
                    label: text::transport_step_label(text, &name),
                    quiet: false,
                    run: Callback::new(move |()| {
                        let _ = core::set_status(core, revision, &unit, &code);
                    }),
                }
            })
            .collect()
    };
    if transport
        .status
        .next()
        .contains(&TransportStatus::Cancelled)
    {
        let text = text::transport_step(TransportStatus::Cancelled);
        actions.push(Action {
            text,
            label: text::transport_step_label(text, &name),
            quiet: true,
            run: update(TransportStatus::Cancelled),
        });
    }
    view! { <Actions actions /> }
}

// [impl->dsn~transport-request~3]
/// The transports of one patient on the record, and the form to request one while none
/// is going on.
#[component]
pub fn TransportSection(core: Handle, revision: RwSignal<u64>, tag: String) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            let uhs = core.with_value(|c| {
                PatientRegister::from_events(&c.events(med::COMPARTMENT))
                    .patient(&tag)
                    .map(|p| p.uhs.clone())
            });
            (uhs, patient_transports(core, &tag))
        })
    };
    let transport_type = RwSignal::new(String::new());
    let priority = RwSignal::new(String::new());
    let needs = RwSignal::new(BTreeSet::<String>::new());
    let planned = RwSignal::new(String::new());
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
                needs: needs.with_untracked(|s| in_profile_order(&CATALOGUE.needs, s)),
                planned: None,
            };
            // [impl->dsn~planned-transports~1]
            let when = planned.get_untracked();
            let request = if when.is_empty() {
                request
            } else {
                match core::local_time_ms(&when) {
                    Some(ms) => TransportRequested {
                        planned: Some(ms),
                        ..request
                    },
                    None => {
                        message.set(Some(text::transport_planned_invalid()));
                        return;
                    }
                }
            };
            if request.check(&CATALOGUE).is_err() {
                message.set(Some(text::transport_choose()));
                return;
            }
            if core::request_transport(core, revision, &tag, request).is_ok() {
                transport_type.set(String::new());
                priority.set(String::new());
                needs.set(BTreeSet::new());
                planned.set(String::new());
                message.set(None);
            }
        }
    };
    view! {
        <section class="record__section" aria-labelledby="section-transport">
            <h3 id="section-transport" tabindex="-1">{text::transport_title()}</h3>
            {move || {
                let (_, transports) = state.get();
                if transports.is_empty() {
                    return view! { <p class="hint">{text::transport_none()}</p> }.into_any();
                }
                view! {
                    <ul class="transport-list" data-keys="list">
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
                    <RadioChips legend=text::transport_type() name="transport-type" options=choices(&CATALOGUE.types) value=transport_type />
                    <RadioChips legend=text::transport_priority_of_request() name="transport-priority" options=choices(&CATALOGUE.priorities) value=priority />
                    <CheckChips legend=text::transport_needs() options=choices(&CATALOGUE.needs) value=needs />
                    <div class="field">
                        <label for="transport-planned">{text::transport_planned()}</label>
                        <input id="transport-planned" type="datetime-local" bind:value=planned />
                    </div>
                    <p class="error" role="alert">{move || message.get()}</p>
                    <div class="row">
                        <button type="submit" class="button button--primary">{text::transport_request()}</button>
                    </div>
                </form>
            </Show>
        </section>
    }
}

// [impl->dsn~transport-request~3]
// [impl->dsn~transport-board~1]
/// The incident command's transport board, as the parts of *Führung – Transporte*: open
/// and planned requests (assign a unit and a destination, follow the status; place 10),
/// free transport units (20), and the hospitals with their capacity and the patients sent
/// to each (30), side by side or in tabs (`nav::View::tabs`). It shows no patient data,
/// because it has none.
pub fn transport_sections(
    core: Handle,
    revision: RwSignal<u64>,
) -> Vec<crate::shell::registry::Section> {
    let board = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| TransportBoard::from_events(&c.events(med::transport::COMPARTMENT)))
    });
    let open = Memo::new(move |_| {
        board.with(|b| b.open(&CATALOGUE).into_iter().cloned().collect::<Vec<_>>())
    });
    let hospitals = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| HospitalList::from_events(&c.events(med::hospitals::COMPARTMENT)))
    });
    let units = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)))
    });
    let unit_names =
        Memo::new(move |_| units.with(|u| u.units.keys().cloned().collect::<Vec<_>>()));
    let planned =
        Memo::new(move |_| board.with(|b| b.planned().into_iter().cloned().collect::<Vec<_>>()));
    vec![
        (
            10,
            view! {
                <RequestList core revision list=open units=unit_names hospitals
                    id="transports-title" title=text::transports_title() empty=text::transports_empty() />
                // [impl->dsn~planned-transports~1]
                <RequestList core revision list=planned units=unit_names hospitals
                    id="planned-title" title=text::planned_title() empty=text::planned_empty() />
            }
            .into_any(),
        ),
        (20, view! { <FreeUnits board units /> }.into_any()),
        (30, view! { <HospitalBoard core revision hospitals board /> }.into_any()),
    ]
}

/// Open requests on the board: each with its details, the assignment while requested,
/// and the next steps.
#[component]
fn RequestList(
    core: Handle,
    revision: RwSignal<u64>,
    list: Memo<Vec<Transport>>,
    units: Memo<Vec<String>>,
    hospitals: Memo<HospitalList>,
    id: &'static str,
    title: &'static str,
    empty: &'static str,
) -> impl IntoView {
    view! {
        <section aria-labelledby=id class="card">
            <h2 id=id>{title}</h2>
            {move || {
                let list = list.get();
                if list.is_empty() {
                    return view! { <p>{empty}</p> }.into_any();
                }
                view! {
                    <ul class="transport-list" data-keys="list">
                        {list
                            .into_iter()
                            .map(|t| view! {
                                <li class="transport-card" data-request=t.request.request.clone()>
                                    <h3>{summary(core, &t)}</h3>
                                    <TransportDetails core transport=t.clone() show_uhs=true />
                                    {(t.status == TransportStatus::Requested)
                                        .then(|| view! { <AssignForm core revision transport=t.clone() units hospitals /> })}
                                    // [impl->dsn~transport-request~3]
                                    // Sent without a destination: it is set here, until the unit is there.
                                    {(t.unit.is_some() && t.destination.is_none() && t.status.is_open() && t.status < TransportStatus::Arrived)
                                        .then(|| view! { <AssignForm core revision transport=t.clone() units hospitals destination_only=true /> })}
                                    // Changed on the way, e.g. when a hospital stops admitting.
                                    {(t.unit.is_some() && t.destination.is_some() && t.status.is_open() && t.status < TransportStatus::Arrived)
                                        .then(|| {
                                            // Closed once saved: the card is updated in place.
                                            let details = NodeRef::<leptos::html::Details>::new();
                                            let saved = Callback::new(move |()| {
                                                if let Some(d) = details.get_untracked() {
                                                    d.set_open(false);
                                                }
                                            });
                                            view! {
                                                <details class="transport-change" node_ref=details>
                                                    <summary>{text::destination_change()}</summary>
                                                    <AssignForm core revision transport=t.clone() units hospitals destination_only=true on_saved=saved />
                                                </details>
                                            }
                                        })}
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

/// Transport units free for a new transport (TRANS-4).
#[component]
fn FreeUnits(board: Memo<TransportBoard>, units: Memo<UnitRegister>) -> impl IntoView {
    let free = Memo::new(move |_| {
        units.with(|units| {
            board.with(|b| {
                b.available_units(units, &crate::shared::units::STATUS, &CATALOGUE)
                    .into_iter()
                    .map(|u| {
                        let status = u.status.as_ref().map(|s| text::status_label(&s.status));
                        (
                            u.unit.clone(),
                            u.unit_type.clone(),
                            status.unwrap_or_default(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
        })
    });
    view! {
        <section aria-labelledby="free-units-title" class="card">
            <h2 id="free-units-title">{text::free_units_title()}</h2>
            {move || {
                let list = free.get();
                if list.is_empty() {
                    return view! { <p>{text::free_units_empty()}</p> }.into_any();
                }
                view! {
                    <ul class="free-units">
                        {list
                            .into_iter()
                            .map(|(unit, kind, status)| view! {
                                <li><strong>{unit}</strong> " · " {kind} " · " {status}</li>
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}

// [impl->dsn~hospital-list~2]
/// The hospitals while the incident runs: their capacity, entered here, and the patients
/// sent. The list itself is kept in *Vorbereitung – Einsatzmittel* ([`HospitalsSetup`]).
#[component]
fn HospitalBoard(
    core: Handle,
    revision: RwSignal<u64>,
    hospitals: Memo<HospitalList>,
    board: Memo<TransportBoard>,
) -> impl IntoView {
    let sent = Memo::new(move |_| board.with(TransportBoard::sent_to_hospitals));
    view! {
        <section aria-labelledby="hospitals-title" class="card">
            <h2 id="hospitals-title">{text::hospitals_title()}</h2>
            {move || {
                let list: Vec<Hospital> = hospitals.with(|h| h.by_name().into_iter().cloned().collect());
                if list.is_empty() {
                    return view! { <p>{text::hospitals_empty()} " " {text::hospitals_where()}</p> }.into_any();
                }
                let sent = sent.get();
                view! {
                    <ul class="hospital-list" data-keys="list">
                        {list
                            .into_iter()
                            .map(|hospital| {
                                let n = sent.get(&hospital.entry.hospital).copied().unwrap_or(0);
                                view! { <HospitalCard core revision hospital sent=n /> }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
        </section>
    }
}

// [impl->dsn~hospital-list~2]
/// *Vorbereitung – Einsatzmittel*, tab *Kliniken*: the incident's hospitals, added here and
/// changed in the list's edit dialog, as the units are.
#[component]
pub fn HospitalsSetup(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let hospitals = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| HospitalList::from_events(&c.events(med::hospitals::COMPARTMENT)))
    });
    // [impl->dsn~edit-dialog~3]
    // One dialog for the list; the hospital it shows changes only when its entry does.
    let editing = RwSignal::new(None::<String>);
    let back = RwSignal::new(String::new());
    let edited = Memo::new(move |_| {
        let id = editing.get()?;
        hospitals.with(|h| {
            h.by_name()
                .into_iter()
                .find(|x| x.entry.hospital == id)
                .map(|x| x.entry.clone())
        })
    });
    let close = Callback::new(move |()| editing.set(None));
    view! {
        <section aria-labelledby="hospitals-setup-title" class="card">
            <h2 id="hospitals-setup-title">{text::hospitals_title()}</h2>
            {move || {
                let list: Vec<Hospital> = hospitals.with(|h| h.by_name().into_iter().cloned().collect());
                if list.is_empty() {
                    return view! { <p>{text::hospitals_empty()}</p> }.into_any();
                }
                view! {
                    <ul class="hospital-list" data-keys="list">
                        {list
                            .into_iter()
                            .map(|hospital| view! { <HospitalEntryCard hospital editing back /> })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <crate::shell::ui::EditDialog id="hospital-edit" open=Signal::derive(move || edited.with(Option::is_some))
                on_close=close back
                title=Signal::derive(move || edited.with(|e| e.as_ref().map(|e| text::hospital_edit_title(&e.name)).unwrap_or_default()))>
                {move || edited.get().map(|entry| view! {
                    <div class="edit-dialog__part">
                        <HospitalForm core revision entry done=close />
                    </div>
                })}
            </crate::shell::ui::EditDialog>
            <h3>{text::hospital_add()}</h3>
            <HospitalForm core revision />
        </section>
    }
}

/// What a hospital offers, in words.
fn offers(entry: &HospitalSet) -> Vec<String> {
    entry
        .departments
        .iter()
        .map(|d| label(&HOSPITALS.departments, d))
        .chain(
            entry
                .capabilities
                .iter()
                .map(|c| label(&HOSPITALS.capabilities, c)),
        )
        .collect()
}

/// One hospital's entry, with its button to the list's edit dialog.
#[component]
fn HospitalEntryCard(
    hospital: Hospital,
    /// The list's dialog: which hospital it edits, and the button to return to.
    editing: RwSignal<Option<String>>,
    back: RwSignal<String>,
) -> impl IntoView {
    let entry = hospital.entry;
    let offers = offers(&entry);
    let (id, name) = (entry.hospital.clone(), entry.name.clone());
    let button = format!("hospital-edit-{}-button", core::slug(&id));
    let chosen = id.clone();
    view! {
        <li class="hospital-card" data-hospital=id>
            <div class="card-head">
                <h3>{name.clone()}</h3>
                <crate::shell::ui::EditButton id=button.clone() icon="pencil" word=text::hospital_edit()
                    label=text::hospital_edit_label(&name)
                    on_click=Callback::new(move |()| {
                        back.set(button.clone());
                        editing.set(Some(chosen.clone()));
                    }) />
            </div>
            {(!offers.is_empty()).then(|| view! { <p>{offers.join(" · ")}</p> })}
        </li>
    }
}

/// The capacity of a hospital as one line, or that none was reported.
fn capacity_line(core: Handle, hospital: &Hospital) -> String {
    match &hospital.capacity {
        Some(r) => {
            let parts: Vec<String> = HOSPITALS
                .capacity
                .iter()
                .map(|c| {
                    let n = r.of(*c).map_or_else(|| "–".to_owned(), |n| n.to_string());
                    format!("{} {n}", text::label(*c))
                })
                .collect();
            text::capacity_line(
                &parts,
                &core::clock(r.at.wall_ms),
                &crate::shell::profile::display_name(core, &r.by),
            )
        }
        None => text::capacity_none().to_owned(),
    }
}

/// One hospital while the incident runs: what it offers, its capacity and the patients
/// sent; entering its capacity.
#[component]
fn HospitalCard(
    core: Handle,
    revision: RwSignal<u64>,
    hospital: Hospital,
    sent: usize,
) -> impl IntoView {
    let offers = offers(&hospital.entry);
    let (id, title) = (hospital.entry.hospital.clone(), hospital.entry.name.clone());
    view! {
        <li class="hospital-card" data-hospital=id>
            <h3>{title}</h3>
            {(!offers.is_empty()).then(|| view! { <p>{offers.join(" · ")}</p> })}
            <p class="hospital-card__capacity">{crate::shell::profile::naming({
                let hospital = hospital.clone();
                move || capacity_line(core, &hospital)
            })}</p>
            <p class="hospital-card__sent"><strong>{text::sent(sent)}</strong></p>
            <CapacityForm core revision hospital=hospital.clone() />
        </li>
    }
}

/// Entering the capacity a hospital reports, per category of the profile; empty fields
/// are "not reported".
#[component]
fn CapacityForm(core: Handle, revision: RwSignal<u64>, hospital: Hospital) -> impl IntoView {
    let id = hospital.entry.hospital.clone();
    let name = hospital.entry.name.clone();
    let fields: Vec<(TriageCategory, RwSignal<String>)> = HOSPITALS
        .capacity
        .iter()
        .map(|c| {
            let now = hospital.capacity.as_ref().and_then(|r| r.of(*c));
            (
                *c,
                RwSignal::new(now.map(|n| n.to_string()).unwrap_or_default()),
            )
        })
        .collect();
    let message = RwSignal::new(None::<&'static str>);
    let submit = {
        let (id, name, fields) = (id.clone(), name.clone(), fields.clone());
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let mut capacity = Vec::new();
            for (category, value) in &fields {
                let value = value.get_untracked();
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }
                match value.parse::<u32>() {
                    Ok(patients) => capacity.push(Capacity {
                        category: *category,
                        patients,
                    }),
                    Err(_) => {
                        message.set(Some(text::capacity_numbers()));
                        return;
                    }
                }
            }
            let set = CapacitySet {
                hospital: id.clone(),
                capacity,
            };
            if core::set_capacity(core, revision, &set, &name).is_ok() {
                message.set(None);
            }
        }
    };
    view! {
        <form class="form capacity-form" novalidate on:submit=submit>
            <fieldset class="choices">
                <legend>{text::capacity_legend()}</legend>
                {fields
                    .into_iter()
                    .map(|(category, value)| {
                        let field = format!("capacity-{id}-{category:?}").to_lowercase();
                        view! {
                            <div class="field field--small">
                                <label for=field.clone()>{text::label(category)}</label>
                                <input id=field type="text" inputmode="numeric" autocomplete="off" bind:value=value />
                            </div>
                        }
                    })
                    .collect_view()}
            </fieldset>
            <p class="error" role="alert">{move || message.get()}</p>
            <div class="row">
                <button type="submit" class="button button--small" aria-label=text::capacity_save_label(&name)>
                    {text::capacity_save()}
                </button>
            </div>
        </form>
    }
}

/// Adding a hospital, or changing the entry of one (`entry`).
#[component]
fn HospitalForm(
    core: Handle,
    revision: RwSignal<u64>,
    #[prop(optional)] entry: Option<HospitalSet>,
    /// Called after saving.
    #[prop(optional)]
    done: Option<Callback<()>>,
) -> impl IntoView {
    let existing = entry.as_ref().map(|e| e.hospital.clone());
    let field = format!("hospital-name-{}", existing.as_deref().unwrap_or("new"));
    let name = RwSignal::new(entry.as_ref().map(|e| e.name.clone()).unwrap_or_default());
    let set =
        |ids: Option<&Vec<String>>| ids.map(|v| v.iter().cloned().collect()).unwrap_or_default();
    let departments = RwSignal::new(set(entry.as_ref().map(|e| &e.departments)));
    let capabilities = RwSignal::new(set(entry.as_ref().map(|e| &e.capabilities)));
    let message = RwSignal::new(None::<&'static str>);
    let changed = RwSignal::new(false);
    let save = {
        let existing = existing.clone();
        Callback::new(move |()| -> bool {
            let entry = HospitalSet {
                hospital: existing.clone().unwrap_or_default(),
                name: name.get_untracked().trim().to_owned(),
                departments: departments
                    .with_untracked(|s| in_profile_order(&HOSPITALS.departments, s)),
                capabilities: capabilities
                    .with_untracked(|s| in_profile_order(&HOSPITALS.capabilities, s)),
            };
            if entry.check(&HOSPITALS).is_err() {
                message.set(Some(text::hospital_name_missing()));
                return false;
            }
            if core::set_hospital(core, revision, entry).is_err() {
                return false;
            }
            message.set(None);
            changed.set(false);
            if existing.is_none() {
                name.set(String::new());
                departments.set(BTreeSet::new());
                capabilities.set(BTreeSet::new());
            }
            true
        })
    };
    // [impl->dsn~edit-dialog~3]
    // In the edit dialog, closing it saves what was changed.
    crate::shell::ui::PendingSaves::register(Callback::new(move |()| {
        !changed.get_untracked() || save.run(())
    }));
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if save.run(())
            && let Some(done) = done
        {
            done.run(());
        }
    };
    let edited_now = move |_| changed.set(true);
    let button = if existing.is_some() {
        text::hospital_save()
    } else {
        text::hospital_add()
    };
    view! {
        <form class="form hospital-form" novalidate on:submit=submit on:input=edited_now on:change=edited_now>
            <div class="field">
                <label for=field.clone()>{text::hospital_name()}</label>
                <input id=field type="text" autocomplete="off" bind:value=name />
            </div>
            <CheckChips legend=text::hospital_departments() options=choices(&HOSPITALS.departments) value=departments />
            <CheckChips legend=text::hospital_capabilities() options=choices(&HOSPITALS.capabilities) value=capabilities />
            <p class="error" role="alert">{move || message.get()}</p>
            <div class="row">
                <button type="submit" class="button">{button}</button>
            </div>
        </form>
    }
}

/// The destination option for anything not on the hospital list.
const OTHER: &str = "other";

/// Assigns a unit of the incident and a destination to a request: a hospital of the list
/// or, for anything else, free text.
#[component]
fn AssignForm(
    core: Handle,
    revision: RwSignal<u64>,
    transport: Transport,
    units: Memo<Vec<String>>,
    hospitals: Memo<HospitalList>,
    /// Only the destination, for a transport on its way without one (ADR-0098).
    #[prop(optional)]
    destination_only: bool,
    /// Called once the update is recorded.
    #[prop(optional)]
    on_saved: Option<Callback<()>>,
) -> impl IntoView {
    let request = transport.request.request.clone();
    let current = transport.status;
    let name = summary(core, &transport);
    let unit = RwSignal::new(String::new());
    // A destination set before is chosen already: a hospital of the list, else free text.
    let change = transport.destination.is_some();
    let destination = RwSignal::new(match (&transport.hospital, change) {
        (Some(h), _) => h.clone(),
        (None, true) => OTHER.to_owned(),
        (None, false) => String::new(),
    });
    let other = RwSignal::new(
        transport
            .destination
            .clone()
            .filter(|_| transport.hospital.is_none())
            .unwrap_or_default(),
    );
    let message = RwSignal::new(None::<&'static str>);
    let (unit_id, destination_id, other_id) = (
        format!("unit-{request}"),
        format!("destination-{request}"),
        format!("other-{request}"),
    );
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let chosen = destination.get_untracked();
        // [impl->dsn~hospital-list~2]
        let (shown, hospital) = hospitals.with_untracked(|h| match h.hospitals.get(&chosen) {
            Some(h) => (h.entry.name.clone(), Some(chosen.clone())),
            None => (other.get_untracked().trim().to_owned(), None),
        });
        let free_text = chosen == OTHER || hospitals.with_untracked(|h| h.hospitals.is_empty());
        let u = unit.get_untracked();
        // [impl->dsn~transport-request~3]
        // A unit is needed; the destination may come later (ADR-0098).
        let has_destination = !shown.is_empty() && (hospital.is_some() || free_text);
        let update = if destination_only {
            if !has_destination {
                message.set(Some(text::transport_destination_missing()));
                return;
            }
            TransportUpdated {
                request: request.clone(),
                status: current,
                unit: None,
                destination: Some(shown),
                hospital,
                status_code: None,
            }
        } else {
            if u.is_empty() {
                message.set(Some(text::transport_assign_missing()));
                return;
            }
            TransportUpdated {
                request: request.clone(),
                status: TransportStatus::Assigned,
                unit: Some(u),
                destination: has_destination.then_some(shown),
                hospital: hospital.filter(|_| has_destination),
                status_code: None,
            }
        };
        if core::update_transport(core, revision, &update).is_ok() {
            message.set(None);
            if let Some(saved) = on_saved {
                saved.run(());
            }
        }
    };
    let other_field = {
        let other_id = other_id.clone();
        move |label: &'static str| {
            view! {
                <div class="field">
                    <label for=other_id.clone()>{label}</label>
                    <input id=other_id.clone() type="text" autocomplete="off" bind:value=other />
                </div>
            }
        }
    };
    view! {
        <form class="form transport-assign" novalidate on:submit=submit>
            {move || {
                if destination_only {
                    return ().into_any();
                }
                let list = units.get();
                if list.is_empty() {
                    return view! { <p class="hint">{text::transport_no_units()}</p> }.into_any();
                }
                view! {
                    <div class="field">
                        <label for=unit_id.clone()>{text::transport_unit_label()}</label>
                        <select id=unit_id.clone() on:change=move |ev| unit.set(event_target_value(&ev))>
                            <option value="" selected=move || unit.get().is_empty()>{text::transport_choose_unit()}</option>
                            {list
                                .into_iter()
                                .map(|u| view! { <option value=u.clone() selected=move || unit.get() == u>{u.clone()}</option> })
                                .collect_view()}
                        </select>
                    </div>
                }
                .into_any()
            }}
            {move || {
                let list: Vec<(String, String)> = hospitals.with(|h| {
                    h.by_name().iter().map(|h| (h.entry.hospital.clone(), h.entry.name.clone())).collect()
                });
                if list.is_empty() {
                    return other_field(text::transport_destination()).into_any();
                }
                let other_field = other_field.clone();
                view! {
                    <div class="field">
                        <label for=destination_id.clone()>{text::destination()}</label>
                        <select id=destination_id.clone() on:change=move |ev| destination.set(event_target_value(&ev))>
                            <option value="" selected=move || destination.get().is_empty()>{text::destination_choose()}</option>
                            {list
                                .into_iter()
                                .map(|(id, name)| {
                                    let chosen = id.clone();
                                    view! { <option value=id selected=move || destination.get() == chosen>{name}</option> }
                                })
                                .collect_view()}
                            <option value=OTHER selected=move || destination.get() == OTHER>{text::destination_other_option()}</option>
                        </select>
                    </div>
                    <Show when=move || destination.get() == OTHER>
                        {other_field(text::destination_other())}
                    </Show>
                }
                .into_any()
            }}
            <p class="error" role="alert">{move || message.get()}</p>
            <div class="row">
                {move || {
                    let word = match (destination_only, change) {
                        (false, _) => text::transport_assign(),
                        (true, false) => text::destination_set_label(),
                        (true, true) => text::destination_change(),
                    };
                    view! {
                        <button type="submit" class="button button--primary"
                            aria-label=text::transport_step_label(word, &name)>
                            {word}
                        </button>
                    }
                }}
            </div>
        </form>
    }
}
