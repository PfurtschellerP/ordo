//! The organisation's vehicles as master data (ORG-14, ADR-0068), kept on the page where
//! units are planned, *Vorbereitung – Einsatzmittel* (ADR-0070). Those who keep master data
//! add, change and remove them, each with its usual strength and no crew; they are offered
//! when units are added to an incident. Vehicles from the server's configuration are shown,
//! not changed.

use leptos::prelude::*;
use ordo_sync::message::{Vehicle, VehicleChange};
use ordo_units::{Strength, UnitCatalogue};

use crate::core::{self, Handle};
use crate::text;
use crate::units::{CATALOGUE, StrengthText};

// [impl->dsn~vehicle-master-data~2]
/// The organisation's vehicles, sorted by organisation and call sign, and the form.
#[component]
pub fn VehiclesView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let ready = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.replica.is_ready())
    });
    let welcomes = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| c.welcomes)
    });
    // Ask after every new connection (an answer may be lost when it is cut).
    Effect::new(move |_| {
        if ready.get() && welcomes.get() > 0 {
            core::list_vehicles(core);
        }
    });
    let vehicles = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.vehicles_listed.then(|| {
                let mut list = c.vehicles.clone();
                list.sort_by(|a, b| (&a.organisation, &a.unit).cmp(&(&b.organisation, &b.unit)));
                list
            })
        })
    });
    // The vehicle being changed, if any; the form then edits it.
    let editing = RwSignal::new(None::<Vehicle>);
    view! {
        <section aria-labelledby="vehicles-title" class="card">
            <h2 id="vehicles-title" tabindex="-1">{text::VEHICLES_TITLE}</h2>
            <p>{text::VEHICLES_INTRO}</p>
            {move || {
                if !ready.get() {
                    return view! { <p class="notice">{text::VEHICLES_OFFLINE}</p> }.into_any();
                }
                match vehicles.get() {
                    None => view! { <p>{text::VEHICLES_LOADING}</p> }.into_any(),
                    Some(list) if list.is_empty() => view! { <p>{text::VEHICLES_EMPTY}</p> }.into_any(),
                    Some(list) => view! { <VehicleList core list editing /> }.into_any(),
                }
            }}
            {move || {
                // A new form for each vehicle edited, filled with it.
                let vehicle = editing.get();
                view! { <VehicleForm core vehicles editing vehicle /> }
            }}
        </section>
    }
}

/// Type, organisation and service of a vehicle as one line.
fn details(v: &Vehicle) -> String {
    let service = v
        .service
        .as_deref()
        .map(|s| UnitCatalogue::label(&CATALOGUE.services, s).to_owned());
    [
        Some(v.unit_type.clone()),
        Some(v.organisation.clone()),
        service,
    ]
    .into_iter()
    .flatten()
    .filter(|t| !t.is_empty())
    .collect::<Vec<_>>()
    .join(" · ")
}

/// The vehicles, each with its way to change or remove it.
#[component]
fn VehicleList(
    core: Handle,
    list: Vec<Vehicle>,
    editing: RwSignal<Option<Vehicle>>,
) -> impl IntoView {
    view! {
        <ul class="roster vehicle-list" data-keys="list">
            {list
                .into_iter()
                .map(|v| {
                    let strength = v.strength.map(Strength::from);
                    let unit = v.unit.clone();
                    let remove = VehicleChange { replaces: Some(unit.clone()), vehicle: None };
                    let edit = v.clone();
                    view! {
                        <li class="roster__entry" data-unit=unit.clone()>
                            <span class="roster__person">{unit.clone()}</span>
                            <span class="roster__id">{details(&v)}</span>
                            {strength.map(|s| view! { <StrengthText strength=s /> })}
                            {v.fixed.then(|| view! { <span class="roster__id">{text::ORG_ROLE_FIXED}</span> })}
                            {(!v.fixed).then(|| view! {
                                <span class="row vehicle-list__actions">
                                    <button type="button" class="button button--small" aria-label=text::vehicle_edit_label(&unit)
                                        on:click=move |_| {
                                            editing.set(Some(edit.clone()));
                                            crate::views::focus("vehicle-call-sign");
                                        }>
                                        {text::EDIT}
                                    </button>
                                    <button type="button" class="button button--small" aria-label=text::vehicle_remove_label(&unit)
                                        on:click=move |_| core::change_vehicle(core, remove.clone())>
                                        {text::VEHICLE_REMOVE}
                                    </button>
                                </span>
                            })}
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}

/// Adds a vehicle, or changes the one in `vehicle`.
#[component]
fn VehicleForm(
    core: Handle,
    vehicles: Memo<Option<Vec<Vehicle>>>,
    editing: RwSignal<Option<Vehicle>>,
    vehicle: Option<Vehicle>,
) -> impl IntoView {
    let replaces = vehicle.as_ref().map(|v| v.unit.clone());
    let v = vehicle.unwrap_or(Vehicle {
        unit: String::new(),
        unit_type: String::new(),
        organisation: String::new(),
        service: None,
        strength: None,
        fixed: false,
    });
    let call_sign = RwSignal::new(v.unit);
    let unit_type = RwSignal::new(v.unit_type);
    let organisation = RwSignal::new(v.organisation);
    let service = RwSignal::new(v.service.unwrap_or_default());
    let part = |i: usize| RwSignal::new(v.strength.map(|s| s[i].to_string()).unwrap_or_default());
    let (leaders, sub_leaders, crew) = (part(0), part(1), part(2));
    let error = RwSignal::new(None::<&'static str>);
    let title = match &replaces {
        Some(unit) => text::vehicle_edit_title(unit),
        None => text::VEHICLE_ADD_TITLE.to_owned(),
    };
    let editing_one = replaces.is_some();
    let submit = {
        let replaces = replaces.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let unit = call_sign.get_untracked().trim().to_owned();
            let kind = unit_type.get_untracked().trim().to_owned();
            if unit.is_empty() || kind.is_empty() {
                error.set(Some(text::UNIT_MISSING));
                crate::views::focus(if unit.is_empty() {
                    "vehicle-call-sign"
                } else {
                    "vehicle-type"
                });
                return;
            }
            let taken = vehicles.with_untracked(|list| {
                list.iter()
                    .flatten()
                    .any(|other| other.unit == unit && Some(&other.unit) != replaces.as_ref())
            });
            if taken {
                error.set(Some(text::VEHICLE_TAKEN));
                crate::views::focus("vehicle-call-sign");
                return;
            }
            let number = |s: RwSignal<String>| s.get_untracked().trim().parse::<u32>().ok();
            let given = [leaders, sub_leaders, crew]
                .iter()
                .any(|s| !s.get_untracked().trim().is_empty());
            let strength = match [number(leaders), number(sub_leaders), number(crew)] {
                [Some(a), Some(b), Some(c)] if a < 1000 && b < 1000 && c < 1000 => Some([a, b, c]),
                _ if !given => None,
                _ => {
                    error.set(Some(text::STRENGTH_NUMBERS));
                    crate::views::focus("vehicle-leaders");
                    return;
                }
            };
            let service = service.get_untracked();
            let vehicle = Vehicle {
                unit,
                unit_type: kind,
                organisation: organisation.get_untracked().trim().to_owned(),
                service: (!service.is_empty()).then_some(service),
                strength,
                fixed: false,
            };
            if !vehicle.check() {
                error.set(Some(text::VEHICLE_TOO_LONG));
                return;
            }
            error.set(None);
            core::change_vehicle(
                core,
                VehicleChange {
                    replaces: replaces.clone(),
                    vehicle: Some(vehicle),
                },
            );
            if editing_one {
                editing.set(None); // a fresh form for the next one
            } else {
                for s in [call_sign, unit_type, leaders, sub_leaders, crew] {
                    s.set(String::new());
                }
            }
        }
    };
    view! {
        <form class="form vehicle-form" novalidate on:submit=submit aria-labelledby="vehicle-form-title">
            <h3 id="vehicle-form-title">{title}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="vehicle-call-sign">{text::UNIT_CALL_SIGN}</label>
                    <input id="vehicle-call-sign" type="text" required autocomplete="off" bind:value=call_sign />
                </div>
                <div class="field">
                    <label for="vehicle-type">{text::UNIT_TYPE}</label>
                    <input id="vehicle-type" type="text" required autocomplete="off" bind:value=unit_type />
                </div>
                <div class="field">
                    <label for="vehicle-organisation">{text::UNIT_ORGANISATION}</label>
                    <input id="vehicle-organisation" type="text" autocomplete="off" bind:value=organisation />
                </div>
                <div class="field">
                    <label for="vehicle-service">{text::UNIT_SERVICE}</label>
                    <select id="vehicle-service" prop:value=move || service.get() on:change=move |ev| service.set(event_target_value(&ev))>
                        <option value="">{text::NO_SERVICE}</option>
                        {CATALOGUE.services.iter().map(|s| view! { <option value=s.id.clone()>{s.label.clone()}</option> }).collect_view()}
                    </select>
                </div>
            </div>
            <fieldset class="choices strength-input">
                <legend>{text::VEHICLE_STRENGTH_LEGEND}</legend>
                {[(leaders, "vehicle-leaders", text::LEADERS), (sub_leaders, "vehicle-sub-leaders", text::SUB_LEADERS), (crew, "vehicle-crew", text::CREW_COUNT)]
                    .into_iter()
                    .map(|(value, id, label)| view! {
                        <div class="field field--small">
                            <label for=id>{label}</label>
                            <input id=id type="text" inputmode="numeric" autocomplete="off" bind:value=value />
                        </div>
                    })
                    .collect_view()}
            </fieldset>
            <p role="alert" class="error">{move || error.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary">
                    {if editing_one { text::SAVE } else { text::VEHICLE_ADD }}
                </button>
                {editing_one.then(|| view! {
                    <button type="button" class="button" on:click=move |_| editing.set(None)>{text::CANCEL}</button>
                })}
            </div>
        </form>
    }
}
