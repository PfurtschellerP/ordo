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
use crate::text;
use crate::ui::{Action, Actions, CheckChips, Disclosure, History, RadioChips};

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
        {t.request.planned.map(|ms| view! {
            <p class="transport-card__planned"><strong>{text::planned_for(&core::date_time(ms))}</strong></p>
        })}
        <p>{uhs.map(|u| format!("{u} · "))}{request_line(&t.request)}</p>
        <History entries=t
            .history
            .iter()
            .map(|s| (core::clock(s.at.wall_ms), text::transport_status(s.status).to_owned(), s.by.to_string()))
            .collect() />
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
            text::transport_unit(unit, t.destination.as_deref().unwrap_or("–"))
        ),
        None => kind,
    }
}

/// Buttons for the next statuses; assigning needs a unit and a destination and is offered
/// apart (incident command).
#[component]
fn StepButtons(core: Handle, revision: RwSignal<u64>, transport: Transport) -> impl IntoView {
    let name = summary(core, &transport);
    let request = transport.request.request.clone();
    let actions = transport
        .status
        .next()
        .into_iter()
        .filter(|s| *s != TransportStatus::Assigned)
        .map(|status| {
            let request = request.clone();
            let text = text::transport_step(status);
            Action {
                text,
                label: text::transport_step_label(text, &name),
                quiet: status == TransportStatus::Cancelled,
                run: Callback::new(move |()| {
                    let update = TransportUpdated {
                        request: request.clone(),
                        status,
                        unit: None,
                        destination: None,
                        hospital: None,
                    };
                    let _ = core::update_transport(core, revision, &update);
                }),
            }
        })
        .collect();
    view! { <Actions actions /> }
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
                        message.set(Some(text::TRANSPORT_PLANNED_INVALID));
                        return;
                    }
                }
            };
            if request.check(&CATALOGUE).is_err() {
                message.set(Some(text::TRANSPORT_CHOOSE));
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
            <h3 id="section-transport" tabindex="-1">{text::TRANSPORT_TITLE}</h3>
            {move || {
                let (_, transports) = state.get();
                if transports.is_empty() {
                    return view! { <p class="hint">{text::TRANSPORT_NONE}</p> }.into_any();
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
                    <RadioChips legend=text::TRANSPORT_TYPE name="transport-type" options=choices(&CATALOGUE.types) value=transport_type />
                    <RadioChips legend=text::TRANSPORT_PRIORITY_OF_REQUEST name="transport-priority" options=choices(&CATALOGUE.priorities) value=priority />
                    <CheckChips legend=text::TRANSPORT_NEEDS options=choices(&CATALOGUE.needs) value=needs />
                    <div class="field">
                        <label for="transport-planned">{text::TRANSPORT_PLANNED}</label>
                        <input id="transport-planned" type="datetime-local" bind:value=planned />
                    </div>
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
// [impl->dsn~transport-board~1]
/// The incident command's transport board: open requests (assign a unit and a
/// destination, follow the status), free transport units, and the hospitals with their
/// capacity and the patients sent to each. It shows no patient data, because it has none.
#[component]
pub fn TransportsView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
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
    view! {
        <RequestList core revision list=open units=unit_names hospitals
            id="transports-title" title=text::TRANSPORTS_TITLE empty=text::TRANSPORTS_EMPTY />
        // [impl->dsn~planned-transports~1]
        <RequestList core revision list=planned units=unit_names hospitals
            id="planned-title" title=text::PLANNED_TITLE empty=text::PLANNED_EMPTY />
        <FreeUnits board units />
        <HospitalBoard core revision hospitals board />
    }
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
                b.available_units(units, &crate::units::STATUS, &CATALOGUE)
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
            <h2 id="free-units-title">{text::FREE_UNITS_TITLE}</h2>
            {move || {
                let list = free.get();
                if list.is_empty() {
                    return view! { <p>{text::FREE_UNITS_EMPTY}</p> }.into_any();
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

// [impl->dsn~hospital-list~1]
/// The hospitals of the incident with capacity and patients sent; adding and changing them.
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
            <h2 id="hospitals-title">{text::HOSPITALS_TITLE}</h2>
            {move || {
                let list: Vec<Hospital> = hospitals.with(|h| h.by_name().into_iter().cloned().collect());
                if list.is_empty() {
                    return view! { <p>{text::HOSPITALS_EMPTY}</p> }.into_any();
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
            <h3>{text::HOSPITAL_ADD}</h3>
            <HospitalForm core revision />
        </section>
    }
}

/// The capacity of a hospital as one line, or that none was reported.
fn capacity_line(hospital: &Hospital) -> String {
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
            text::capacity_line(&parts, &core::clock(r.at.wall_ms), r.by.as_str())
        }
        None => text::CAPACITY_NONE.to_owned(),
    }
}

/// One hospital: what it offers, its capacity and the patients sent; entering capacity,
/// and changing the entry behind a disclosure.
#[component]
fn HospitalCard(
    core: Handle,
    revision: RwSignal<u64>,
    hospital: Hospital,
    sent: usize,
) -> impl IntoView {
    let entry = hospital.entry.clone();
    let offers: Vec<String> = entry
        .departments
        .iter()
        .map(|d| label(&HOSPITALS.departments, d))
        .chain(
            entry
                .capabilities
                .iter()
                .map(|c| label(&HOSPITALS.capabilities, c)),
        )
        .collect();
    let editing = RwSignal::new(false);
    let done = Callback::new(move |()| editing.set(false));
    let name = entry.name.clone();
    let (id, title) = (entry.hospital.clone(), entry.name.clone());
    view! {
        <li class="hospital-card" data-hospital=id.clone()>
            <h3>{title}</h3>
            {(!offers.is_empty()).then(|| view! { <p>{offers.join(" · ")}</p> })}
            <p class="hospital-card__capacity">{capacity_line(&hospital)}</p>
            <p class="hospital-card__sent"><strong>{text::sent(sent)}</strong></p>
            <CapacityForm core revision hospital=hospital.clone() />
            <Disclosure
                id=format!("hospital-{id}")
                open=editing
                class="hospital-card__edit"
                button_class="button button--small button--quiet"
                panel_class="hospital-card__form"
                label=Some(Signal::derive(move || text::hospital_edit_label(&name)))
                button=Box::new(|| view! { {text::HOSPITAL_EDIT} }.into_any())
            >
                <HospitalForm core revision entry done />
            </Disclosure>
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
                        message.set(Some(text::CAPACITY_NUMBERS));
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
                <legend>{text::CAPACITY_LEGEND}</legend>
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
                    {text::CAPACITY_SAVE}
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
    let submit = {
        let existing = existing.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let entry = HospitalSet {
                hospital: existing.clone().unwrap_or_default(),
                name: name.get_untracked().trim().to_owned(),
                departments: departments
                    .with_untracked(|s| in_profile_order(&HOSPITALS.departments, s)),
                capabilities: capabilities
                    .with_untracked(|s| in_profile_order(&HOSPITALS.capabilities, s)),
            };
            if entry.check(&HOSPITALS).is_err() {
                message.set(Some(text::HOSPITAL_NAME_MISSING));
                return;
            }
            if core::set_hospital(core, revision, entry).is_ok() {
                message.set(None);
                if existing.is_none() {
                    name.set(String::new());
                    departments.set(BTreeSet::new());
                    capabilities.set(BTreeSet::new());
                }
                if let Some(done) = done {
                    done.run(());
                }
            }
        }
    };
    let button = if existing.is_some() {
        text::HOSPITAL_SAVE
    } else {
        text::HOSPITAL_ADD
    };
    view! {
        <form class="form hospital-form" novalidate on:submit=submit>
            <div class="field">
                <label for=field.clone()>{text::HOSPITAL_NAME}</label>
                <input id=field type="text" autocomplete="off" bind:value=name />
            </div>
            <CheckChips legend=text::HOSPITAL_DEPARTMENTS options=choices(&HOSPITALS.departments) value=departments />
            <CheckChips legend=text::HOSPITAL_CAPABILITIES options=choices(&HOSPITALS.capabilities) value=capabilities />
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
) -> impl IntoView {
    let request = transport.request.request.clone();
    let name = summary(core, &transport);
    let unit = RwSignal::new(String::new());
    let destination = RwSignal::new(String::new());
    let other = RwSignal::new(String::new());
    let message = RwSignal::new(None::<&'static str>);
    let (unit_id, destination_id, other_id) = (
        format!("unit-{request}"),
        format!("destination-{request}"),
        format!("other-{request}"),
    );
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let chosen = destination.get_untracked();
        // [impl->dsn~hospital-list~1]
        let (shown, hospital) = hospitals.with_untracked(|h| match h.hospitals.get(&chosen) {
            Some(h) => (h.entry.name.clone(), Some(chosen.clone())),
            None => (other.get_untracked().trim().to_owned(), None),
        });
        let free_text = chosen == OTHER || hospitals.with_untracked(|h| h.hospitals.is_empty());
        let u = unit.get_untracked();
        if u.is_empty() || shown.is_empty() || (hospital.is_none() && !free_text) {
            message.set(Some(text::TRANSPORT_ASSIGN_MISSING));
            return;
        }
        let update = TransportUpdated {
            request: request.clone(),
            status: TransportStatus::Assigned,
            unit: Some(u),
            destination: Some(shown),
            hospital,
        };
        if core::update_transport(core, revision, &update).is_ok() {
            message.set(None);
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
            {move || {
                let list: Vec<(String, String)> = hospitals.with(|h| {
                    h.by_name().iter().map(|h| (h.entry.hospital.clone(), h.entry.name.clone())).collect()
                });
                if list.is_empty() {
                    return other_field(text::TRANSPORT_DESTINATION).into_any();
                }
                let other_field = other_field.clone();
                view! {
                    <div class="field">
                        <label for=destination_id.clone()>{text::DESTINATION}</label>
                        <select id=destination_id.clone() on:change=move |ev| destination.set(event_target_value(&ev))>
                            <option value="" selected=move || destination.get().is_empty()>{text::DESTINATION_CHOOSE}</option>
                            {list
                                .into_iter()
                                .map(|(id, name)| {
                                    let chosen = id.clone();
                                    view! { <option value=id selected=move || destination.get() == chosen>{name}</option> }
                                })
                                .collect_view()}
                            <option value=OTHER selected=move || destination.get() == OTHER>{text::DESTINATION_OTHER_OPTION}</option>
                        </select>
                    </div>
                    <Show when=move || destination.get() == OTHER>
                        {other_field(text::DESTINATION_OTHER)}
                    </Show>
                }
                .into_any()
            }}
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
