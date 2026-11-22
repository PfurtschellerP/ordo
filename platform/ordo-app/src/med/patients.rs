//! Med workspace: patient registration, the patient list and the patient care record.
//!
//! Record editing happens in place, on the record page (master–detail on wide screens,
//! a page of its own on phones), never in a modal dialog (ADR-0029). Unsaved input is
//! kept per patient and form while the app runs, so switching views loses nothing.

use std::collections::{BTreeMap, HashMap};
use std::sync::LazyLock;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use leptos::prelude::*;
use leptos::task::spawn_local;
use ordo_contracts::TriageCategory;
use ordo_med_patients::record::{
    self, EntryRecorded, Field, FieldType, Form, FormCatalogue, FormKind, Records, Value,
};
use ordo_med_patients::transport::TransportStatus;
use ordo_med_patients::{self as med, Discharged, Holder, Outcome, Patient, PatientRegister, Stay};
use wasm_bindgen::JsCast;

use crate::attachments::Availability;
use crate::core::{self, Handle};
use crate::med::text;
use crate::med::triage::{CATEGORIES, TriageBadge};
use crate::med::uhs::uhs_list;
use crate::shell::nav::Route;
use crate::shell::ui::{
    Action, Actions, Column, Icon, IncidentPeople, Part, Parts, RadioChips, Suggestions, TabsMode,
};
use crate::shell::views::{Nav, after_render, focus};

pub(crate) const WORKSPACE: &str = "med.uhs";
/// Patient photos: longest edge and JPEG quality, tried in this order (ADR-0028).
/// Full photo: longest side and JPEG quality, tried in order until it fits.
const PHOTO_STEPS: [(f64, f64); 4] = [(1600.0, 0.8), (1280.0, 0.75), (1024.0, 0.7), (800.0, 0.6)];
/// Preview inside the event.
const PREVIEW_STEPS: [(f64, f64); 4] = [(240.0, 0.6), (200.0, 0.5), (160.0, 0.45), (120.0, 0.4)];
const VIEW: &str = "patients";

/// The ePCR forms of the profile, checked at build time (see `build.rs`).
pub static FORMS: LazyLock<FormCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/epcr-forms.json")))
        .expect("ePCR forms were checked at build time")
});

/// Transport priority of the profile: which categories, which levels (checked at build
/// time, see `build.rs`).
/// Triage algorithms of the profile, as references (checked at build time, see `build.rs`).
pub static TRIAGE: LazyLock<med::triage::TriageCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/triage-algorithms.json"
    )))
    .expect("triage algorithms were checked at build time")
});

pub static TRANSPORT_PRIORITY: LazyLock<med::TransportPriorityConfig> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(
        env!("OUT_DIR"),
        "/transport-priority.json"
    )))
    .expect("transport priority was checked at build time")
});

/// The label of a transport priority level.
fn priority_label(level: &str) -> String {
    TRANSPORT_PRIORITY
        .levels
        .iter()
        .find(|l| l.id == level)
        .map_or_else(|| level.to_owned(), |l| l.label.clone())
}

/// Route to the record of the patient with triage tag `tag`.
pub fn record_route(tag: &str) -> Route {
    Route::patient(tag)
}

/// How an editor was opened.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Mode {
    /// Edit a snapshot form.
    Edit,
    /// New entry of a series form.
    New,
    /// Correction of a series entry.
    Correct(String),
}

/// Unsaved input per patient and form, kept while the app runs (UX-5). Clinical input is
/// not written to browser storage unencrypted.
#[derive(Clone, Copy)]
pub struct Drafts(StoredValue<HashMap<String, Draft>, LocalStorage>);

/// An open editor, its values and the order it documents, if any (ORDER-7).
type Draft = (Mode, BTreeMap<String, Value>, Option<String>);

impl Default for Drafts {
    fn default() -> Self {
        Self(StoredValue::new_local(HashMap::new()))
    }
}

fn draft_key(tag: &str, form: &str) -> String {
    format!("{tag}/{form}")
}

// [impl->dsn~order-documentation~3]
/// Opens a new entry of `form` on the record of `tag`, filled with `values` (time fields
/// with `time`, else the time now), documenting `order` if given. If an entry of that form
/// is open already, it is only brought into view: the crew's input is never overwritten.
pub fn open_prefilled(
    drafts: Drafts,
    tag: &str,
    form: &'static Form,
    values: BTreeMap<String, Value>,
    time: Option<String>,
    order: Option<String>,
) {
    let key = draft_key(tag, &form.id);
    let open = drafts.0.with_value(|d| d.contains_key(&key));
    if !open {
        let start: BTreeMap<String, Value> = form
            .fields
            .iter()
            .map(|f| {
                let value = values
                    .get(&f.id)
                    .cloned()
                    .unwrap_or_else(|| match f.field_type {
                        FieldType::Choices => Value::Many(Vec::new()),
                        FieldType::Time => Value::One(time.clone().unwrap_or_else(core::now_clock)),
                        _ => Value::One(String::new()),
                    });
                (f.id.clone(), value)
            })
            .collect();
        drafts.0.update_value(|d| {
            d.insert(key, (Mode::New, start, order));
        });
        crate::shell::views::click(&format!("new-{}", form.id));
    } else {
        let heading = format!("section-{}", form.id);
        after_render(move || reveal_editor(&heading));
    }
}

/// The patients view: registration and list, and the record of the chosen patient.
#[component]
pub fn PatientsView(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    record: Signal<Option<String>>,
    /// The UHS's patients, or a crew's (ADR-0093).
    #[prop(optional)]
    scope: Scope,
) -> impl IntoView {
    provide_context(scope);
    let events = move || core.with_value(|c| c.events(med::COMPARTMENT));
    // The UHS the person works at: as chosen on this device, else the one whose staff
    // list names them, else all.
    let on_list = move || {
        core.with_value(|c| {
            let me = c.replica.user().to_string();
            med::uhs_details(&c.events(med::UHS_COMPARTMENT))
                .into_iter()
                .find(|(_, d)| d.staff.iter().any(|s| s.trim().eq_ignore_ascii_case(&me)))
                .map(|(id, _)| id)
                .unwrap_or_default()
        })
    };
    let working = WorkingUhs(RwSignal::new(
        core::preference(WORKING_UHS_KEY).unwrap_or_else(on_list),
    ));
    // [impl->dsn~uhs-staff~3]
    // Until the person chooses, it follows the staff list as it arrives: the command puts
    // people on it (ADR-0086), often before their device has the incident's events.
    Effect::new(move || {
        revision.track();
        if core::preference(WORKING_UHS_KEY).is_none() && working.0.get_untracked().is_empty() {
            let found = on_list();
            if !found.is_empty() {
                working.0.set(found);
            }
        }
    });
    provide_context(working);
    let register = Memo::new(move |_| {
        revision.track();
        PatientRegister::from_events(&events())
    });
    let records = Memo::new(move |_| {
        revision.track();
        Records::from_events(&events())
    });
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    provide_context(IncidentPeople(people.into(), core));
    // [impl->dsn~reports~3]
    // A patient case the registration fills in (REP-3).
    let pending = crate::shared::reports::take_pending_case();
    if pending.is_some() {
        request_animation_frame(|| focus("tag"));
    }
    provide_context(crate::shared::reports::ActiveCase(RwSignal::new(pending)));
    // [impl->dsn~quick-handover~1]
    // Opened from a quick handover's code: the dialog to take the patient.
    let taking = RwSignal::new(
        record
            .get_untracked()
            .zip(core::url_param("handover").filter(|h| !h.is_empty())),
    );
    // [impl->dsn~crew-patients~1]
    // A crew sees the record while its units hold the patient or are handed it; once
    // taken, it closes (least knowledge). A memo, so the record is rebuilt only then.
    let visible = Memo::new(move |_| {
        revision.track();
        scope == Scope::Uhs
            || record.get().is_some_and(|tag| {
                register.with(|r| {
                    r.patient(&tag)
                        .is_some_and(|p| p.seen_by_crew(&crew_units(core)))
                })
            })
    });
    // [impl->dsn~page-tabs~4]
    // The list, registering and staff requests are separate tasks: tabs (feedback of
    // 2026-11-22), the list first.
    let part = RwSignal::new(LIST_PART);
    let mut master_parts = vec![
        Part {
            key: LIST_PART,
            label: text::PATIENTS_TAB.to_owned(),
            short: None,
            column: Column::Start,
            content: view! {
                <PatientList core revision nav register records selected=record />
                // [impl->dsn~crew-places~1]
                {(scope == Scope::Crew).then(|| view! { <crate::med::overview::CrewPlaces core revision /> })}
            }
            .into_any(),
        },
        Part {
            key: "register",
            label: text::REGISTER_TAB.to_owned(),
            short: None,
            column: Column::Start,
            content: view! { <RegisterForm core revision nav register /> }.into_any(),
        },
    ];
    // [impl->dsn~uhs-staff-request~2]
    if scope == Scope::Uhs {
        master_parts.push(Part {
            key: "staff",
            label: text::STAFF_REQUEST_TAB.to_owned(),
            short: Some(text::STAFF_REQUEST_SHORT.to_owned()),
            column: Column::Start,
            content: view! {
                <crate::med::staffing::StaffRequestPanel core revision uhs=Signal::from(working.0) />
            }
            .into_any(),
        });
    }
    view! {
        <crate::med::handover::TakeDialog core revision nav register taking />
        <div class="master-detail" data-open=move || record.get().is_some().to_string()>
            <div class="master">
                // What needs attention stays above the tabs (chapter 8.6.5, *Tabs*).
                {(scope == Scope::Uhs).then(|| view! { <UhsChooser core revision working /> })}
                <crate::med::handover::IncomingHandovers core revision nav register />
                <crate::med::handover::ScanHandover taking />
                <Parts id="patients-parts" label=crate::text::page_tabs() parts=master_parts chosen=part mode=TabsMode::Always />
            </div>
            <div class="detail">
                {move || match (record.get(), visible.get()) {
                    (Some(tag), true) => view! { <PatientRecord core revision nav tag register records /> }.into_any(),
                    (Some(_), false) => view! { <section class="card"><p class="notice">{text::CREW_NOT_HELD}</p></section> }.into_any(),
                    (None, _) => view! { <section class="card"><p>{text::CHOOSE_PATIENT}</p></section> }.into_any(),
                }}
            </div>
        </div>
    }
}

/// The patient list's first tab.
const LIST_PART: &str = "list";

// [impl->dsn~crew-patients~1]
/// Where the patients page is shown (ADR-0093): in the UHS workspace with the UHS's
/// patients, or in the crew's with only the patients its units hold or are handed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scope {
    #[default]
    Uhs,
    Crew,
}

impl Scope {
    /// The workspace the page is in.
    pub fn workspace(self) -> &'static str {
        match self {
            Self::Uhs => WORKSPACE,
            Self::Crew => CREW_WORKSPACE,
        }
    }
}

/// The crew's workspace (shared package), where the Med package shows its patients.
pub(crate) const CREW_WORKSPACE: &str = "units.crew";

/// The call signs of the units whose crew names this person.
fn crew_units(core: Handle) -> Vec<String> {
    crate::shared::crew::my_units(core)
        .into_iter()
        .map(|u| u.unit)
        .collect()
}

/// Where the device keeps the UHS its person works at (UHS-9); empty: all.
const WORKING_UHS_KEY: &str = "ordo.working-uhs";

/// Makes `uhs` the one this person works at on this device, e.g. opened from the map.
pub fn work_at(uhs: &str) {
    core::set_preference(WORKING_UHS_KEY, Some(uhs));
}

/// The UHS the person works at (UHS-9): its ID, or empty for all of them.
#[derive(Clone, Copy)]
pub struct WorkingUhs(pub RwSignal<String>);

// [impl->dsn~uhs-staff~3]
/// The UHS this person may write (UHS-10, ADR-0086): those whose staff list names them, or
/// all for roles allowed at every UHS. The server decides again for every event.
pub fn writable_uhs(core: Handle) -> Vec<String> {
    core.with_value(|c| {
        let me = c.replica.user().to_string();
        let everywhere = c.may(core::permission::ALL_UHS);
        let events = c.events(med::UHS_COMPARTMENT);
        let details = med::uhs_details(&events);
        med::uhs_list(&events)
            .into_iter()
            .filter(|u| {
                everywhere
                    || details
                        .get(&u.id)
                        .is_some_and(|d| d.staff.iter().any(|s| s.trim().eq_ignore_ascii_case(&me)))
            })
            .map(|u| u.id)
            .collect()
    })
}

// [impl->dsn~patient-holder~1]
/// Where this person may hold patients (ADR-0087): the UHS they may write, then the units
/// whose crew names them, with their names. The server decides again for every event.
pub fn writable_holders(core: Handle) -> Vec<(Holder, String)> {
    let writable = writable_uhs(core);
    uhs_list(core)
        .into_iter()
        .filter(|u| writable.contains(&u.id))
        .map(|u| (Holder::Uhs(u.id), u.name))
        .chain(
            crate::shared::crew::my_units(core)
                .into_iter()
                .map(|u| (Holder::Unit(u.unit.clone()), u.unit)),
        )
        .collect()
}

/// Whether this person writes the patients of `holder`.
pub fn may_write(core: Handle, holder: &Holder) -> bool {
    match holder {
        Holder::Uhs(id) => writable_uhs(core).contains(id),
        Holder::Unit(_) => {
            core.with_value(|c| c.may(core::permission::ALL_UHS))
                || writable_holders(core).iter().any(|(h, _)| h == holder)
        }
    }
}

/// A holder as the value of a choice: a UHS by its ID, a unit as `unit:` and its call sign.
pub fn holder_key(holder: &Holder) -> String {
    match holder {
        Holder::Uhs(id) => id.clone(),
        Holder::Unit(unit) => format!("unit:{unit}"),
    }
}

/// The holder of a choice's value ([`holder_key`]); none if empty.
pub fn holder_of_key(key: &str) -> Option<Holder> {
    match key.strip_prefix("unit:") {
        Some(unit) => Some(Holder::Unit(unit.to_owned())),
        None => (!key.is_empty()).then(|| Holder::Uhs(key.to_owned())),
    }
}

// [impl->dsn~uhs-staff~3]
/// "Meine UHS" at the top of the UHS module (UHS-9): the person chooses where they work;
/// registration and the patient list follow. The device remembers the choice.
#[component]
fn UhsChooser(core: Handle, revision: RwSignal<u64>, working: WorkingUhs) -> impl IntoView {
    let list = Memo::new(move |_| {
        revision.track();
        uhs_list(core)
    });
    let writable = Memo::new(move |_| {
        revision.track();
        writable_uhs(core)
    });
    view! {
        <div class="uhs-chooser">
            <label for="working-uhs"><crate::shell::ui::Icon name="tent" /> {text::WORKING_UHS}</label>
            <select id="working-uhs" on:change=move |ev| {
                let value = event_target_value(&ev);
                core::set_preference(WORKING_UHS_KEY, Some(&value));
                working.0.set(value);
            }>
                <option value="" selected=move || working.0.get().is_empty()>{text::ALL_UHS}</option>
                {move || list.get().into_iter().map(|u| {
                    let id = u.id.clone();
                    let chosen = u.id.clone();
                    let name = if writable.with(|w| w.contains(&u.id)) { u.name } else { text::read_only_uhs(&u.name) };
                    view! { <option value=id selected=move || working.0.get() == chosen>{name}</option> }
                }).collect_view()}
            </select>
        </div>
    }
}

/// The options of a choice of holders: the UHS, then the units, each in its group.
pub(crate) fn holder_options(
    list: Vec<(Holder, String)>,
    chosen: RwSignal<String>,
) -> impl IntoView {
    let (uhs, units): (Vec<_>, Vec<_>) = list.into_iter().partition(|(h, _)| h.uhs().is_some());
    let option = move |(holder, name): (Holder, String)| {
        let key = holder_key(&holder);
        let value = key.clone();
        view! { <option value=value selected=move || chosen.get() == key>{name}</option> }
    };
    view! {
        {(!uhs.is_empty()).then(|| view! {
            <optgroup label=text::HOLDERS_UHS>{uhs.into_iter().map(option).collect_view()}</optgroup>
        })}
        {(!units.is_empty()).then(|| view! {
            <optgroup label=text::HOLDERS_UNITS>{units.into_iter().map(option).collect_view()}</optgroup>
        })}
    }
}

#[component]
fn RegisterForm(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let tag = RwSignal::new(String::new());
    // Remembered between registrations: a device usually works at one UHS.
    let uhs = RwSignal::new(String::new());
    // No preselection: a forgotten choice must not become a wrong triage category.
    let category = RwSignal::new(None::<TriageCategory>);
    let error = RwSignal::new(None::<String>);
    let last = RwSignal::new(None::<String>);
    // [impl->dsn~patient-holder~1]
    // Only where this person may write (UHS-10): their UHS, then the units whose crew
    // names them (ADR-0087), starting with the working UHS (UHS-9). `uhs` holds the
    // choice's value (`holder_key`).
    // A crew registers only with its own units (ADR-0093).
    let crew = use_context::<Scope>() == Some(Scope::Crew);
    let list = Memo::new(move |_| {
        revision.track();
        writable_holders(core)
            .into_iter()
            .filter(|(h, _)| !crew || h.unit().is_some())
            .collect::<Vec<_>>()
    });
    let offers =
        move |key: &str| list.with_untracked(|l| l.iter().any(|(h, _)| holder_key(h) == key));
    if let Some(working) = use_context::<WorkingUhs>() {
        Effect::new(move || {
            let chosen = working.0.get();
            list.track();
            if offers(&chosen) {
                uhs.set(chosen);
            }
        });
    }
    // [impl->dsn~reports~3]
    // From a patient case: the crew's own unit, which then hands the patient over
    // (ADR-0087); else the unit's UHS, where the command structure names one.
    let case = use_context::<crate::shared::reports::ActiveCase>().map(|c| c.0);
    if let Some(case) = case {
        Effect::new(move || {
            let Some(c) = case.get() else { return };
            list.track();
            let own = holder_key(&Holder::Unit(c.unit.clone()));
            let offered = Some(own).filter(|k| offers(k)).or_else(|| {
                crate::shared::reports::unit_uhs(&crate::shared::command::structure(core), &c.unit)
                    .filter(|id| offers(id))
            });
            if let Some(key) = offered {
                uhs.set(key);
            }
        });
    }
    // [impl->dsn~place-plan~3]
    // The free places of the chosen UHS; a category proposes the first one kept for it.
    let place = RwSignal::new(String::new());
    let free_places = Memo::new(move |_| {
        revision.track();
        let holder = holder_of_key(&uhs.get());
        register.with(|r| free_places(core, r, holder.as_ref()))
    });
    Effect::new(move || {
        let proposed = category
            .get()
            .and_then(|c| free_places.with(|free| propose_place(free, c)));
        place.set(proposed.unwrap_or_default());
    });
    // [impl->dsn~unique-triage-tag~2]
    // A card already registered (also typed differently) is not registered again: its
    // record opens instead (PAT-12).
    let in_use = Memo::new(move |_| {
        revision.track();
        let value = tag.get();
        let value = value.trim();
        if value.is_empty() {
            return None;
        }
        register.with(|r| {
            let p = r.patient_with_card(value)?;
            let since = p.stay.first().map_or(0, |s| s.at.wall_ms);
            Some((
                p.tag.clone(),
                text::tag_in_use(
                    &p.tag,
                    &core::holder_name(core, &p.holder()),
                    &core::clock(since),
                ),
            ))
        })
    });
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if in_use.get_untracked().is_some() {
            focus("tag");
            return;
        }
        let value = tag.get().trim().to_owned();
        let Some(chosen) = category.get() else { return };
        let Some(holder) = holder_of_key(&uhs.get()) else {
            return;
        };
        if value.is_empty() {
            return;
        }
        let from = case.and_then(|c| c.get_untracked());
        let from_case = from.as_ref().map(|c| (c.task.as_str(), c.unit.as_str()));
        match core::register(core, revision, &value, chosen, &holder, from_case) {
            Ok(()) => {
                let on = place.get_untracked();
                let label = free_places
                    .with_untracked(|f| f.iter().find(|p| p.id == on).map(|p| p.label.clone()));
                if let Some(label) = label
                    && let Err(e) =
                        core::assign_place(core, revision, &value, Some(on), Some(&label))
                {
                    error.set(Some(e));
                    return;
                }
                if let Some(case) = case {
                    case.set(None);
                }
                tag.set(String::new());
                category.set(None);
                error.set(None);
                last.set(Some(value));
                focus("tag");
            }
            Err(e) => error.set(Some(e)),
        }
    };
    view! {
        <section aria-labelledby="register-title" class="card">
            <h2 id="register-title">{text::REGISTER_TITLE}</h2>
            {move || {
                if list.with(Vec::is_empty) {
                    return view! { <p class="notice">{text::NO_UHS}</p> }.into_any();
                }
                view! {
                    <form on:submit=submit class="form">
                        {move || case.and_then(|c| c.get()).map(|c| view! {
                            <div class="notice register-case" role="status">
                                <p>{text::register_from_case(&c.place, &c.unit)}</p>
                                <button type="button" class="button button--small button--quiet"
                                    on:click=move |_| if let Some(case) = case { case.set(None) }>
                                    {text::CASE_CANCEL}
                                </button>
                            </div>
                        })}
                        <label for="tag">{text::REGISTER_TAG}</label>
                        <input id="tag" type="text" inputmode="numeric" autocomplete="off" required bind:value=tag class="input--code"
                            aria-invalid=move || in_use.with(Option::is_some).then_some("true")
                            aria-describedby="tag-in-use" />
                        <p id="tag-in-use" class="error" role="status">
                            {move || in_use.get().map(|(known, words)| {
                                let route = record_route(&known);
                                view! {
                                    <Icon name="triangle-alert" />
                                    {words} " "
                                    <a href=route.to_path() on:click=move |ev| {
                                        ev.prevent_default();
                                        nav.go(route.clone());
                                        after_render(|| focus("record-title"));
                                    }>{text::ACTION_OPEN_RECORD}</a>
                                }
                            })}
                        </p>
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
                        <TriageAid id="triage-algorithm" />
                        <label for="uhs">{text::REGISTER_UHS}</label>
                        <select id="uhs" required on:change=move |ev| uhs.set(event_target_value(&ev)) prop:value=move || uhs.get()>
                            <option value="" selected=move || uhs.get().is_empty()>{text::UHS_CHOOSE}</option>
                            {holder_options(list.get(), uhs)}
                        </select>
                        <PlaceSelect id="register-place" free=free_places place hint=text::REGISTER_PLACE_HINT />
                        <p role="alert" class="error">{move || error.get()}</p>
                        <button type="submit" class="button button--primary">{text::REGISTER_SUBMIT}</button>
                    </form>
                }
                .into_any()
            }}
            {move || {
                last.get().map(|tag| {
                    let route = record_route(&tag);
                    view! {
                        <p class="notice">
                            {text::patient_title(&tag)} " " {text::REGISTERED} " "
                            <a href=route.to_path() on:click=move |ev| {
                                ev.prevent_default();
                                nav.go(route.clone());
                                after_render(|| focus("record-title"));
                            }>{text::ACTION_OPEN_RECORD}</a>
                        </p>
                    }
                })
            }}
        </section>
    }
}

// [impl->dsn~place-plan~3]
/// The free treatment places of `holder`, if it is a UHS: its places no present patient
/// holds.
pub(crate) fn free_places(
    core: Handle,
    register: &PatientRegister,
    holder: Option<&Holder>,
) -> Vec<med::Place> {
    let Some(Holder::Uhs(uhs)) = holder else {
        return Vec::new();
    };
    let places = core
        .with_value(|c| med::places(&c.events(med::UHS_COMPARTMENT)))
        .remove(uhs)
        .unwrap_or_default();
    let taken: Vec<&String> = register
        .patients()
        .filter(|p| &p.uhs == uhs && p.is_present())
        .filter_map(|p| p.place.as_ref())
        .collect();
    places
        .into_iter()
        .filter(|p| !taken.contains(&&p.id))
        .collect()
}

/// The place proposed for a patient of `category`: the first free one kept for it, else
/// the first one kept for any category. The person can always choose another or none.
pub(crate) fn propose_place(free: &[med::Place], category: TriageCategory) -> Option<String> {
    free.iter()
        .find(|p| p.category == Some(category))
        .or_else(|| free.iter().find(|p| p.category.is_none()))
        .map(|p| p.id.clone())
}

/// Assigns the place `id` among `free` to the patient `tag`; nothing for none.
pub(crate) fn assign_chosen_place(
    core: Handle,
    revision: RwSignal<u64>,
    tag: &str,
    free: &[med::Place],
    id: &str,
) -> Result<(), String> {
    match free.iter().find(|p| p.id == id) {
        Some(p) => core::assign_place(core, revision, tag, Some(p.id.clone()), Some(&p.label)),
        None => Ok(()),
    }
}

// [impl->dsn~place-plan~3]
/// A choice among free treatment places, by the category they are kept for, with "Kein
/// Platz"; hidden while there are none.
#[component]
pub(crate) fn PlaceSelect(
    #[prop(into)] id: String,
    #[prop(into)] free: Signal<Vec<med::Place>>,
    place: RwSignal<String>,
    hint: &'static str,
) -> impl IntoView {
    let hint_id = format!("{id}-hint");
    view! {
        <Show when=move || free.with(|f| !f.is_empty())>
            <div class="field">
                <label for=id.clone()>{text::PLACE}</label>
                <select id=id.clone() aria-describedby=hint_id.clone()
                    on:change=move |ev| place.set(event_target_value(&ev)) prop:value=move || place.get()>
                    <option value="" selected=move || place.get().is_empty()>{text::NO_PLACE}</option>
                    {move || {
                        let places = free.get().into_iter().map(|p| (p.category, p.id, p.label, None)).collect();
                        by_category(places).into_iter().map(|(category, places)| view! {
                            <optgroup label=category.map_or(text::PLACES_ANY, text::label)>
                                {places.into_iter().map(|(id, label, _)| {
                                    let chosen = id.clone();
                                    view! { <option value=id selected=move || place.get() == chosen>{label}</option> }
                                }).collect_view()}
                            </optgroup>
                        }).collect_view()
                    }}
                </select>
                <p id=hint_id.clone() class="hint">{hint}</p>
            </div>
        </Show>
    }
}

/// Display name from the identity form, if documented.
fn person_name(records: &Records, tag: &str) -> Option<String> {
    let record = records.get(tag)?;
    let value = |field: &str| match record.current("identity", field).map(|s| &s.value) {
        Some(Value::One(text)) => Some(text.clone()),
        _ => None,
    };
    match (value("family_name"), value("given_name")) {
        (Some(family), Some(given)) => Some(format!("{family}, {given}")),
        (Some(name), None) | (None, Some(name)) => Some(name),
        (None, None) => None,
    }
}

/// What a card of the patient list shows.
#[derive(Clone, PartialEq)]
struct Card {
    tag: String,
    category: TriageCategory,
    uhs: String,
    triages: usize,
    name: Option<String>,
    pending: bool,
    priority: Option<String>,
    place: Option<String>,
    /// The place's ID, for the places of the working UHS.
    place_id: Option<String>,
    discharged: Option<Outcome>,
    /// An open handover, in words (ADR-0087).
    handover: Option<String>,
    /// The card was registered more than once (PAT-12).
    twice: bool,
}

/// Sample cards for the component gallery: one of each kind, the second open.
pub(crate) fn gallery_cards() -> AnyView {
    let card = |tag: &str, category, name: Option<&str>| Card {
        tag: tag.to_owned(),
        category,
        uhs: "UHS Bühne Nord".to_owned(),
        triages: 1,
        name: name.map(str::to_owned),
        pending: false,
        priority: None,
        place: None,
        place_id: None,
        discharged: None,
        handover: None,
        twice: false,
    };
    let cards = vec![
        card("DE-0042", TriageCategory::Immediate, None),
        Card {
            priority: Some("Transportpriorität".to_owned()),
            place: Some("Platz 2".to_owned()),
            triages: 2,
            ..card("DE-0044", TriageCategory::Urgent, Some("Mustermann, Max"))
        },
        Card {
            handover: Some("Übergabe an UHS Eingang offen".to_owned()),
            pending: true,
            ..card("DE-0045", TriageCategory::Minor, None)
        },
        Card {
            twice: true,
            ..card("DE-0046", TriageCategory::Minor, None)
        },
        Card {
            discharged: Some(Outcome::Transported),
            ..card(
                "DE-0047",
                TriageCategory::Expectant,
                Some("Musterfrau, Erika"),
            )
        },
    ];
    let nav = Nav::inert();
    let selected = Signal::stored(Some("DE-0044".to_owned()));
    view! {
        <ul class="patient-list" data-keys="list">
            {cards.into_iter().map(|card| view! { <PatientCard card nav selected /> }).collect_view()}
        </ul>
    }
    .into_any()
}

/// One patient in the list: a link to the record.
#[component]
fn PatientCard(card: Card, nav: Nav, selected: Signal<Option<String>>) -> impl IntoView {
    view! { <li><PatientLink card nav selected /></li> }
}

/// A patient's card: a link to the record, in the list or on their place.
#[component]
fn PatientLink(card: Card, nav: Nav, selected: Signal<Option<String>>) -> impl IntoView {
    let route = record_route(&card.tag);
    let current = {
        let tag = card.tag.clone();
        move || (selected.get().as_deref() == Some(tag.as_str())).then_some("page")
    };
    // Discharged: the outcome instead of the place.
    let place = match card.discharged {
        Some(outcome) => Some(text::outcome(outcome).to_owned()),
        None => card.place,
    };
    view! {
            <a
                class="patient-card"
                data-tag=card.tag.clone()
                href=route.to_path()
                aria-current=current
                on:click=move |ev| {
                    ev.prevent_default();
                    nav.go(route.clone());
                    after_render(|| focus("record-title"));
                }
            >
                <span class="patient-card__head">
                    <span class="patient-card__tag code">{card.tag.clone()}</span>
                    <TriageBadge category=card.category />
                    {card.priority.map(|label| view! { <PriorityBadge label /> })}
                </span>
                <span class="patient-card__name">{card.name.unwrap_or_else(|| text::UNKNOWN_PERSON.to_owned())}</span>
                {card.handover.map(|h| view! {
                    <span class="patient-card__handover"><crate::shell::ui::Icon name="arrow-right-left" />{h}</span>
                })}
                {card.twice.then(|| view! {
                    <span class="patient-card__warning"><Icon name="triangle-alert" />{text::TAG_TWICE}</span>
                })}
                <span class="patient-card__meta">
                    {place.map(|p| view! { <span class="patient-card__place">{p}</span> " · " })}
                    {card.uhs} " · " {text::triage_count(card.triages)} " · "
                    <span class="patient-card__state">{if card.pending { text::NOT_SENT } else { text::SENT }}</span>
                </span>
            </a>
    }
}

// [impl->dsn~uhs-discharge~1]
// [impl->dsn~place-plan~3]
/// *Patienten* (UHS-2, UHS-14): for one UHS with places, its patients on their places, by
/// the triage category the places are kept for, the free places with "Belegen", and the
/// patients without a place below, so nobody is lost; otherwise the patients in their units.
/// Discharged patients apart, collapsed.
#[component]
fn PatientList(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
    selected: Signal<Option<String>>,
) -> impl IntoView {
    let working = use_context::<WorkingUhs>();
    let cards = Memo::new(move |_| {
        revision.track();
        let names: HashMap<String, String> =
            uhs_list(core).into_iter().map(|u| (u.id, u.name)).collect();
        let places = core.with_value(|c| med::places(&c.events(med::UHS_COMPARTMENT)));
        // Patients with changes that are still in the outbox.
        let pending: Vec<String> = core.with_value(|c| {
            c.events(med::COMPARTMENT)
                .iter()
                .filter(|e| e.seq.is_none())
                .filter_map(tag_of)
                .collect()
        });
        let working = working.map(|w| w.0.get()).unwrap_or_default();
        let here = Holder::Uhs(working.clone());
        // [impl->dsn~crew-patients~1]
        let crew = (use_context::<Scope>() == Some(Scope::Crew)).then(|| crew_units(core));
        register.with(|r| {
            records.with(|records| {
                r.patients()
                    // A crew: only what its units hold or are handed (least knowledge).
                    .filter(|p| crew.as_ref().is_none_or(|units| p.seen_by_crew(units)))
                    // At one UHS: its patients and those handed over to it.
                    .filter(|p| {
                        crew.is_some()
                            || working.is_empty()
                            || p.uhs == working
                            || p.handover
                                .as_ref()
                                .is_some_and(|h| h.to.as_ref() == Some(&here))
                    })
                    .map(|p| Card {
                        tag: p.tag.clone(),
                        category: p.category(),
                        uhs: match &p.unit {
                            Some(unit) => unit.clone(),
                            None => names.get(&p.uhs).cloned().unwrap_or_else(|| p.uhs.clone()),
                        },
                        handover: p.handover.as_ref().map(|h| match &h.to {
                            Some(to) => text::handover_open_to(&core::holder_name(core, to)),
                            None => text::HANDOVER_OPEN_QUICK.to_owned(),
                        }),
                        twice: !p.registered_again.is_empty(),
                        triages: p.history.len(),
                        name: person_name(records, &p.tag),
                        pending: pending.contains(&p.tag),
                        priority: p
                            .transport_priority(&TRANSPORT_PRIORITY)
                            .map(priority_label),
                        place: p.place.as_ref().and_then(|id| {
                            places
                                .get(&p.uhs)?
                                .iter()
                                .find(|pl| &pl.id == id)
                                .map(|pl| pl.label.clone())
                        }),
                        place_id: p.place.clone(),
                        discharged: discharged_outcome(p),
                    })
                    .collect::<Vec<_>>()
            })
        })
    });
    let present = Memo::new(move |_| {
        cards.with(|c| {
            c.iter()
                .filter(|c| c.discharged.is_none())
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    let discharged = Memo::new(move |_| {
        cards.with(|c| {
            c.iter()
                .filter(|c| c.discharged.is_some())
                .cloned()
                .collect::<Vec<_>>()
        })
    });
    // The places of the working UHS, if it has any, with the patient on each.
    let board = Memo::new(move |_| {
        revision.track();
        let uhs = working.map(|w| w.0.get()).unwrap_or_default();
        if uhs.is_empty() {
            return None;
        }
        let places = core
            .with_value(|c| med::places(&c.events(med::UHS_COMPARTMENT)))
            .remove(&uhs)
            .filter(|p| !p.is_empty())?;
        let present = present.get();
        let mut zones: Vec<Zone> = Vec::new();
        for place in places {
            let card = present
                .iter()
                .find(|c| c.place_id.as_deref() == Some(place.id.as_str()))
                .cloned();
            match zones.iter_mut().find(|(c, _)| *c == place.category) {
                Some((_, list)) => list.push((place, card)),
                None => zones.push((place.category, vec![(place, card)])),
            }
        }
        zones.sort_by_key(|(c, _)| (c.is_none(), *c));
        let placed: Vec<&str> = zones
            .iter()
            .flat_map(|(_, l)| {
                l.iter()
                    .filter_map(|(_, c)| c.as_ref().map(|c| c.tag.as_str()))
            })
            .collect();
        let mut waiting: Vec<Card> = present
            .iter()
            .filter(|c| !placed.contains(&c.tag.as_str()))
            .cloned()
            .collect();
        waiting.sort_by(|a, b| (a.category, &a.tag).cmp(&(b.category, &b.tag)));
        Some((zones, waiting, writable_uhs(core).contains(&uhs)))
    });
    // The free place whose list of patients without a place is open.
    let filling = RwSignal::new(None::<String>);
    let list = move |cards: Vec<Card>| {
        view! {
            <ul class="patient-list" data-keys="list">
                {cards.into_iter().map(|card| view! { <PatientCard card nav selected /> }).collect_view()}
            </ul>
        }
    };
    view! {
        <section aria-labelledby="board-title" class="card">
            <h2 id="board-title">{text::PATIENTS_TITLE}</h2>
            {move || {
                if cards.with(Vec::is_empty) {
                    return view! { <p>{text::BOARD_EMPTY}</p> }.into_any();
                }
                if let Some((zones, waiting, writable)) = board.get() {
                    return view! { <PlacesBoard core revision nav selected zones waiting writable filling /> }.into_any();
                }
                let present = present.get();
                (!present.is_empty()).then(|| list(present)).into_any()
            }}
            // Created once, so it stays open while events arrive.
            <details class="discharged-list" hidden=move || discharged.with(Vec::is_empty)>
                <summary>{move || text::discharged_list(discharged.with(Vec::len))}</summary>
                {move || list(discharged.get())}
            </details>
        </section>
    }
}

/// The places of one triage category (none: any), each with the patient lying there.
type Zone = (Option<TriageCategory>, Vec<(med::Place, Option<Card>)>);

/// The patients of one UHS on its places (UHS-14): a zone per triage category the places
/// are kept for, a place with its patient's card or "frei" (a dashed outline, never colour
/// alone) and, for those who write the UHS, "Belegen" with the patients without a place;
/// then those patients' cards under *Ohne Platz*.
#[component]
fn PlacesBoard(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    selected: Signal<Option<String>>,
    zones: Vec<Zone>,
    waiting: Vec<Card>,
    writable: bool,
    filling: RwSignal<Option<String>>,
) -> impl IntoView {
    let free: usize = zones
        .iter()
        .map(|(_, l)| l.iter().filter(|(_, c)| c.is_none()).count())
        .sum();
    let total: usize = zones.iter().map(|(_, l)| l.len()).sum();
    let fillable = writable && !waiting.is_empty();
    let choices: Vec<(String, TriageCategory)> = waiting
        .iter()
        .map(|c| (c.tag.clone(), c.category))
        .collect();
    let waiting_count = waiting.len();
    view! {
        <div class="place-plan">
            <p class="place-plan__summary">
                {text::places_free(free, total)}
                {(waiting_count > 0).then(|| format!(" · {}", text::without_place(u32::try_from(waiting_count).unwrap_or(u32::MAX))))}
            </p>
            {zones
                .into_iter()
                .map(|(category, places)| {
                    let free = places.iter().filter(|(_, c)| c.is_none()).count();
                    view! {
                        <div class="place-plan__zone">
                            <h3 class="place-plan__zone-title">
                                {match category {
                                    Some(c) => view! { <TriageBadge category=c /> }.into_any(),
                                    None => view! { <span>{text::PLACES}</span> }.into_any(),
                                }}
                                <span class="place-plan__free">{text::places_free(free, places.len())}</span>
                            </h3>
                            <ul class="place-plan__places" data-keys="list">
                                {places
                                    .into_iter()
                                    .map(|(place, card)| match card {
                                        Some(card) => view! {
                                            <li class="place-tile place-tile--taken" data-place=place.id.clone()>
                                                <PatientLink card nav selected />
                                            </li>
                                        }
                                        .into_any(),
                                        None => {
                                            let id = place.id.clone();
                                            let label = place.label.clone();
                                            let open = {
                                                let id = id.clone();
                                                move || filling.with(|f| f.as_deref() == Some(id.as_str()))
                                            };
                                            let list_id = format!("fill-{id}");
                                            let fill = fillable.then(|| {
                                                let choices = choices.clone();
                                                let (toggle_id, open_too) = (id.clone(), open.clone());
                                                view! {
                                                    <button type="button" class="button button--small place-tile__fill"
                                                        aria-expanded=move || open().to_string() aria-controls=list_id.clone()
                                                        aria-label=text::place_fill(&label)
                                                        on:click=move |_| filling.update(|f| {
                                                            *f = if f.as_deref() == Some(toggle_id.as_str()) { None } else { Some(toggle_id.clone()) };
                                                        })>
                                                        <crate::shell::ui::Icon name="plus" />{text::PLACE_FILL}
                                                    </button>
                                                    <ul id=list_id.clone() class="place-tile__choices" aria-label=text::place_fill(&label)
                                                        hidden=move || !open_too()>
                                                        {choices.into_iter().map(|(tag, category)| {
                                                            let (id, label, chosen) = (id.clone(), label.clone(), tag.clone());
                                                            view! {
                                                                <li>
                                                                    <button type="button" class="button button--small place-tile__choice"
                                                                        on:click=move |_| {
                                                                            let _ = core::assign_place(core, revision, &chosen, Some(id.clone()), Some(&label));
                                                                            filling.set(None);
                                                                        }>
                                                                        <span class="code">{tag.clone()}</span>
                                                                        <TriageBadge category />
                                                                    </button>
                                                                </li>
                                                            }
                                                        }).collect_view()}
                                                    </ul>
                                                }
                                            });
                                            view! {
                                                <li class="place-tile place-tile--free" data-place=place.id.clone()>
                                                    <span class="place-tile__label">{place.label.clone()}</span>
                                                    <span class="place-tile__patient">{text::PLACE_FREE}</span>
                                                    {fill}
                                                </li>
                                            }
                                            .into_any()
                                        }
                                    })
                                    .collect_view()}
                            </ul>
                        </div>
                    }
                })
                .collect_view()}
            // The patients waiting for a place: nobody is lost.
            {(waiting_count > 0).then(|| view! {
                <div class="place-plan__zone place-plan__waiting">
                    <h3 class="place-plan__zone-title">
                        <span>{text::PLACE_NONE_TITLE}</span>
                        <span class="place-plan__free">{text::patient_count(waiting_count)}</span>
                    </h3>
                    <ul class="patient-list" data-keys="list">
                        {waiting.into_iter().map(|card| view! { <PatientCard card nav selected /> }).collect_view()}
                    </ul>
                </div>
            })}
        </div>
    }
}

/// The triage tag number of a patient event (all patient event types carry `tag`).
fn tag_of(event: &ordo_sync::AppliedEvent) -> Option<String> {
    #[derive(serde::Deserialize)]
    struct Tag {
        tag: String,
    }
    ordo_core::codec::from_cbor::<Tag>(&event.payload)
        .ok()
        .map(|t| t.tag)
}

// [impl->dsn~patient-care-record~1]
/// The care record of one patient: photo, triage, and the forms of the profile.
#[component]
fn PatientRecord(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    tag: String,
    register: Memo<PatientRegister>,
    records: Memo<Records>,
) -> impl IntoView {
    let scope = use_context::<Scope>().unwrap_or_default();
    // [impl->dsn~crew-patients~1]
    // A crew opens only the records of patients its units hold or are handed.
    let exists = {
        let tag = tag.clone();
        move || {
            register.with(|r| {
                r.patient(&tag)
                    .is_some_and(|p| scope == Scope::Uhs || p.seen_by_crew(&crew_units(core)))
            })
        }
    };
    let back = Route::view(scope.workspace(), VIEW);
    let back_link = view! {
        <a class="back-link" href=back.to_path() on:click=move |ev| {
            ev.prevent_default();
            nav.go(back.clone());
            after_render(|| focus("board-title"));
        }>
            <span aria-hidden="true">"←"</span>{text::ALL_PATIENTS}
        </a>
    };
    if !exists() {
        return view! {
            <section class="card record">
                {back_link}
                <p class="notice">{text::NOT_FOUND}</p>
            </section>
        }
        .into_any();
    }
    // The holder's name when this person may only read the patient's record (UHS-10,
    // ADR-0087).
    let read_only = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            let holder = register.with(|r| r.patient(&tag).map(Patient::holder))?;
            (!may_write(core, &holder)).then(|| core::holder_name(core, &holder))
        })
    };
    let sections: Vec<(String, String)> = [
        ("section-photo".to_owned(), text::PHOTO_TITLE.to_owned()),
        ("section-triage".to_owned(), text::TRIAGE_TITLE.to_owned()),
        ("section-orders".to_owned(), text::ORDERS.to_owned()),
    ]
    .into_iter()
    .chain(
        FORMS
            .forms
            .iter()
            .map(|f| (format!("section-{}", f.id), f.label.clone())),
    )
    .collect();
    // [impl->dsn~record-overview~3]
    // The page opens on the overview (EPCR-14); the tabs keep their input when switched.
    let chosen = RwSignal::new(crate::med::record::OVERVIEW);
    let filter = RwSignal::new(String::new());
    let writable = move || read_only.get().is_none();
    use crate::med::record::{DOCUMENTATION, HANDOVER, OVERVIEW, TIMELINE};
    use crate::shell::ui::{TabList, TabPanel};
    // Each tab's content takes its own copy.
    let (t_header, t_notice, t_overview, t_timeline, t_doc, t_handover) = (
        tag.clone(),
        tag.clone(),
        tag.clone(),
        tag.clone(),
        tag.clone(),
        tag.clone(),
    );
    view! {
        <article class="card record" aria-labelledby="record-title">
            <header class="record__header">
                {back_link}
                <crate::med::record::RecordHeader core revision tag=t_header register records />
                {move || read_only.get().map(|uhs| view! {
                    <p class="notice record__read-only" role="note">{text::record_read_only(&uhs)}</p>
                })}
                <crate::med::handover::HandoverNotice core revision tag=t_notice register />
                <TabList id="record" label=text::RECORD_TABS tabs=crate::med::record::tabs() chosen />
            </header>
            <Show when=writable>
                <crate::med::record::QuickActions chosen />
            </Show>
            <TabPanel id="record" key=OVERVIEW chosen>
                <crate::med::record::OverviewTiles core revision tag=t_overview register records chosen filter />
            </TabPanel>
            <TabPanel id="record" key=TIMELINE chosen>
                <crate::med::record::Timeline core revision tag=t_timeline register records filter />
            </TabPanel>
            <TabPanel id="record" key=DOCUMENTATION chosen>
                // The way to each section, folded: the quick actions and the overview's tiles
                // lead to the sections too, so this is not seen first (calmer record).
                <details class="record__index">
                    <summary>{text::SECTIONS}</summary>
                    <nav aria-label=text::SECTIONS>
                        {sections
                            .into_iter()
                            .map(|(id, label)| {
                                view! {
                                    <button type="button" class="chip" on:click=move |_| focus(&id)>{label}</button>
                                }
                            })
                            .collect_view()}
                    </nav>
                </details>
                // [impl->dsn~uhs-staff~3]
                // Where the person may not write (UHS-10), nothing offers it: the disabled
                // fieldset turns off every control, and the style hides them.
                <fieldset class="record__body" disabled=move || read_only.get().is_some()>
                    // Where the patient lies is seen and changed first, not inside a section.
                    <PlaceChoice core revision tag=t_doc.clone() />
                    <PhotoSection core revision tag=t_doc.clone() records />
                    <TriageSection core revision tag=t_doc.clone() register />
                    <crate::med::orders::OrdersSection core revision tag=t_doc.clone() />
                    {FORMS
                        .forms
                        .iter()
                        .map(|form| view! { <FormSection core revision tag=t_doc.clone() form records /> })
                        .collect_view()}
                </fieldset>
            </TabPanel>
            <TabPanel id="record" key=HANDOVER chosen>
                <crate::med::handover::HandoverPanel core revision tag=t_handover.clone() register />
                <fieldset class="record__body" disabled=move || read_only.get().is_some()>
                    <StaySection core revision tag=t_handover.clone() register />
                    <crate::med::transports::TransportSection core revision tag=t_handover />
                </fieldset>
            </TabPanel>
        </article>
    }
    .into_any()
}

/// The outcome, while the patient is discharged.
fn discharged_outcome(patient: &Patient) -> Option<Outcome> {
    match patient.discharge()?.change {
        Stay::Discharged { outcome, .. } => Some(outcome),
        // Handed over outside ORDO: as transported, for the lists.
        Stay::HandedOutside { .. } => Some(Outcome::Transported),
        _ => None,
    }
}

// [impl->dsn~uhs-discharge~1]
/// Admission and discharge (UHS-3): every admission and discharge with time and person;
/// discharging with an outcome, or readmitting. Once the patient's transport has left,
/// "transported" is chosen already (the crew can still choose otherwise).
#[component]
fn StaySection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let patient = {
        let tag = tag.clone();
        Memo::new(move |_| register.with(|r| r.patient(&tag).cloned()))
    };
    let present = Memo::new(move |_| patient.with(|p| p.as_ref().is_none_or(Patient::is_present)));
    // The transport the patient leaves with: the latest one not cancelled.
    let transport = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            crate::med::transports::patient_transports(core, &tag)
                .into_iter()
                .rev()
                .find(|t| t.status != TransportStatus::Cancelled)
        })
    };
    let outcome = RwSignal::new(String::new());
    Effect::new(move || {
        let left = transport.with(|t| t.as_ref().is_some_and(|t| t.status.has_left()));
        if left && present.get() && outcome.get_untracked().is_empty() {
            outcome.set(Outcome::Transported.id().to_owned());
        }
    });
    let message = RwSignal::new(None::<&'static str>);
    let submit = {
        let tag = tag.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let Some(chosen) = Outcome::from_id(&outcome.get_untracked()) else {
                message.set(Some(text::DISCHARGE_CHOOSE));
                return;
            };
            let discharge = Discharged {
                tag: tag.clone(),
                outcome: chosen,
                transport: (chosen == Outcome::Transported)
                    .then(|| transport.get_untracked().map(|t| t.request.request))
                    .flatten(),
            };
            if core::discharge(core, revision, &discharge).is_ok() {
                outcome.set(String::new());
                message.set(None);
            }
        }
    };
    let readmit = {
        let tag = tag.clone();
        move || {
            let tag = tag.clone();
            vec![Action {
                text: text::READMIT,
                label: text::readmit_label(&tag),
                quiet: false,
                run: Callback::new(move |()| {
                    let _ = core::readmit(core, revision, &tag);
                }),
            }]
        }
    };
    let options: Vec<(String, String)> = Outcome::ALL
        .into_iter()
        .map(|o| (o.id().to_owned(), text::outcome(o).to_owned()))
        .collect();
    // Copy handles: the inputs are built again each time they show.
    let (submit, options) = (StoredValue::new(submit), StoredValue::new(options));
    view! {
        <section class="record__section" aria-labelledby="section-stay">
            <h3 id="section-stay" tabindex="-1">{text::DISCHARGE_TITLE}</h3>
            <ol class="history">
                {move || {
                    // Also when names arrive (ORG-7).
                    revision.track();
                    patient.with(|p| {
                        p.iter()
                            .flat_map(|p| p.stay.iter().enumerate().map(move |(i, e)| (p, i, e)))
                            .map(|(p, i, e)| {
                                let what = crate::med::record::stay_words(core, p, i, e);
                                let when = text::by_at(
                                    &crate::shell::profile::display_name(core, &e.by),
                                    &core::clock(e.at.wall_ms),
                                );
                                view! { <li>{what} " · " {when}</li> }
                            })
                            .collect_view()
                    })
                }}
            </ol>
            <Show
                when=move || present.get()
                fallback=move || view! { <Actions actions=readmit() /> }
            >
                <SectionInputs id="section-stay" open_text=text::STAY_OPEN>
                <form class="form stay-form" novalidate on:submit=submit.get_value()>
                    <RadioChips legend=text::DISCHARGE_OUTCOME name="discharge-outcome" options=options.get_value() value=outcome />
                    {move || {
                        (outcome.get() == Outcome::Transported.id())
                            .then(|| transport.get())
                            .flatten()
                            .map(|t| view! {
                                <p class="hint">{text::discharge_transport(&crate::med::transports::transport_line(&t))}</p>
                            })
                    }}
                    <p class="error" role="alert">{move || message.get()}</p>
                    <div class="row">
                        <button type="submit" class="button">{text::discharge()}</button>
                    </div>
                </form>
                </SectionInputs>
            </Show>
        </section>
    }
}

// [impl->dsn~triage-reference~1]
/// "Sichtung nach": the algorithm used, kept on the device and recorded with every triage;
/// and the *Sichtungshilfe*: the profile's algorithms with their sources (ADR-0045). The
/// category is always the person's choice; nothing here proposes or checks one.
#[component]
fn TriageAid(id: &'static str) -> impl IntoView {
    if TRIAGE.algorithms.is_empty() {
        return ().into_any();
    }
    let chosen = RwSignal::new(core::triage_algorithm().unwrap_or_default());
    let open = RwSignal::new(false);
    view! {
        <div class="triage-aid">
            <div class="field">
                <label for=id>{text::TRIAGE_WITH}</label>
                <select id=id prop:value=move || chosen.get() on:change=move |ev| {
                    let value = event_target_value(&ev);
                    core::set_triage_algorithm(&value);
                    chosen.set(value);
                }>
                    <option value="">{text::TRIAGE_WITH_NONE}</option>
                    {TRIAGE
                        .algorithms
                        .iter()
                        .map(|a| view! { <option value=a.id.clone()>{format!("{} ({})", a.name, text::triage_stage(a.stage))}</option> })
                        .collect_view()}
                </select>
            </div>
            <crate::shell::ui::Disclosure
                id=format!("{id}-help")
                open
                class="triage-aid__help"
                button_class="button button--small button--quiet"
                panel_class="triage-aid__panel"
                button=Box::new(|| view! { {text::TRIAGE_AID} }.into_any())
            >
                <ul class="triage-aid__list">
                    {TRIAGE
                        .algorithms
                        .iter()
                        .map(|a| view! {
                            <li>
                                <strong>{a.name.clone()}</strong>
                                {a.description.clone().map(|d| format!(" – {d}"))}
                                <br />{text::triage_stage(a.stage)} " · " {a.publisher.clone()} " · " {a.edition.clone()}
                                <br /><a href=a.url.clone() target="_blank" rel="noopener noreferrer">{text::TRIAGE_SOURCE}</a>
                            </li>
                        })
                        .collect_view()}
                </ul>
                <p class="hint">{text::TRIAGE_AID_NOTE}</p>
            </crate::shell::ui::Disclosure>
        </div>
    }
    .into_any()
}

/// Which image of a photo is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PhotoKind {
    /// The full photo (or the photo of an older event, which had no preview).
    Full,
    /// The preview, while the full photo is loaded.
    Loading,
    /// The preview; the full photo is not on the server yet.
    Unavailable,
}

// [impl->dsn~patient-photo~3]
#[component]
fn PhotoSection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    records: Memo<Records>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let error = RwSignal::new(None::<&'static str>);
    let latest = {
        let tag = tag.clone();
        move || {
            records.with(|r| {
                r.get(&tag).and_then(|r| r.photo()).map(|p| {
                    (
                        p.id.clone(),
                        p.media_type.clone(),
                        p.data.clone(),
                        p.full.clone(),
                    )
                })
            })
        }
    };
    // What is shown: (event ID, kind, data URL). The data URL is rebuilt only when another
    // photo becomes the latest or its full image arrives.
    let source = Memo::new(
        move |previous: Option<&Option<(String, PhotoKind, String)>>| {
            revision.track();
            let (id, media_type, preview, full) = latest()?;
            let (kind, media_type, data) = match &full {
                Some(full) => {
                    match crate::attachments::get(core, revision, med::COMPARTMENT, full) {
                        Availability::Ready(data) => {
                            (PhotoKind::Full, full.media_type.clone(), Some(data))
                        }
                        Availability::Loading => (PhotoKind::Loading, media_type, None),
                        Availability::Unavailable => (PhotoKind::Unavailable, media_type, None),
                    }
                }
                None => (PhotoKind::Full, media_type, None),
            };
            if let Some(Some((prev_id, prev_kind, src))) = previous
                && *prev_id == id
                && (*prev_kind == kind
                    || (*prev_kind != PhotoKind::Full && kind != PhotoKind::Full))
            {
                return Some((id, kind, src.clone()));
            }
            let data = data.unwrap_or(preview);
            Some((
                id,
                kind,
                format!("data:{media_type};base64,{}", BASE64.encode(data)),
            ))
        },
    );
    let on_file = {
        let tag = tag.clone();
        move |ev: leptos::ev::Event| {
            let input: web_sys::HtmlInputElement = event_target(&ev);
            let Some(file) = input.files().and_then(|files| files.get(0)) else {
                return;
            };
            input.set_value("");
            busy.set(true);
            error.set(None);
            let tag = tag.clone();
            spawn_local(async move {
                let images = async {
                    let full = crate::shell::images::downscale(
                        &file,
                        &PHOTO_STEPS,
                        record::MAX_FULL_PHOTO_BYTES,
                    )
                    .await?;
                    let preview = crate::shell::images::downscale(
                        &file,
                        &PREVIEW_STEPS,
                        record::MAX_PREVIEW_BYTES,
                    )
                    .await?;
                    Ok::<_, &'static str>((full, preview))
                };
                match images.await {
                    Ok((full, preview)) => {
                        if core::add_photo(core, revision, &tag, &full, preview).is_err() {
                            error.set(Some(text::photo_too_large()));
                        }
                    }
                    Err(message) => error.set(Some(message)),
                }
                busy.set(false);
            });
        }
    };
    let alt = text::photo_alt(&tag);
    let caption = {
        let tag = tag.clone();
        move || {
            records.with(|r| {
                r.get(&tag).and_then(|r| r.photo()).map(|p| {
                    text::photo_taken(
                        &core::clock(p.at.wall_ms),
                        &crate::shell::profile::display_name(core, &p.by),
                    )
                })
            })
        }
    };
    let remove_tag = tag.clone();
    view! {
        <section class="record__section" aria-labelledby="section-photo">
            <h3 id="section-photo" tabindex="-1">{text::PHOTO_TITLE}</h3>
            {move || match source.get() {
                Some((_, kind, src)) => view! {
                    <img class="photo" src=src alt=alt.clone()
                        data-photo=if kind == PhotoKind::Full { "full" } else { "preview" } />
                    <p class="photo__caption">{caption.clone()}</p>
                    {match kind {
                        PhotoKind::Full => None,
                        PhotoKind::Loading => Some(view! { <p class="hint">{text::PHOTO_PREVIEW_LOADING}</p> }),
                        PhotoKind::Unavailable => Some(view! { <p class="hint">{text::PHOTO_PREVIEW_UNAVAILABLE}</p> }),
                    }}
                }
                .into_any(),
                None => view! { <p>{text::PHOTO_NONE}</p> }.into_any(),
            }}
            <p class="hint" id="photo-purpose">{text::PHOTO_PURPOSE}</p>
            <div class="row">
                <input id="photo-camera" class="file-input visually-hidden" type="file" accept="image/*" capture="environment"
                    aria-describedby="photo-purpose" on:change=on_file.clone() />
                <label for="photo-camera" class="button button--with-icon"><Icon name="camera" />{text::PHOTO_TAKE}</label>
                <input id="photo-upload" class="file-input visually-hidden" type="file" accept="image/*"
                    aria-describedby="photo-purpose" on:change=on_file />
                <label for="photo-upload" class="button button--with-icon"><Icon name="upload" />{text::PHOTO_UPLOAD}</label>
                // [impl->dsn~patient-photo~3]
                // [impl->dsn~icon-rules~1]
                // A trash icon; removed at once, with undo instead of a question.
                {move || source.with(|s| s.as_ref().map(|(id, ..)| id.clone())).map(|id| {
                    let tag = remove_tag.clone();
                    let label = text::photo_remove_label(&tag);
                    view! {
                        <button type="button" class="button button--quiet icon-button" id="photo-remove"
                            aria-label=label.clone() data-tip=label
                            on:click=move |_| {
                                if core::remove_photo(core, revision, &tag, &id).is_ok() {
                                    let (tag, id) = (tag.clone(), id.clone());
                                    crate::shell::ui::offer_undo(text::photo_removed(&tag), move || {
                                        let _ = core::restore_photo(core, revision, &tag, &id);
                                    });
                                }
                            }>
                            <Icon name="trash-2" /><span class="icon-button__word" aria-hidden="true">{text::PHOTO_REMOVE_SHORT}</span>
                        </button>
                    }
                })}
            </div>
            <p role="status" class="hint">{move || busy.get().then_some(text::PHOTO_BUSY)}</p>
            <p role="alert" class="error">{move || error.get()}</p>
        </section>
    }
}

#[component]
fn TriageSection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    register: Memo<PatientRegister>,
) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        move || {
            // Also when names arrive (ORG-7).
            revision.track();
            register.with(|r| {
                r.patient(&tag).map(|p| {
                    (
                        p.category(),
                        p.history
                            .iter()
                            .rev()
                            .map(|h| {
                                (
                                    h.category,
                                    core::clock(h.at.wall_ms),
                                    crate::shell::profile::display_name(core, &h.by),
                                    h.algorithm.clone(),
                                )
                            })
                            .collect::<Vec<_>>(),
                    )
                })
            })
        }
    };
    let current = {
        let state = state.clone();
        move || state().map(|(c, _)| c)
    };
    view! {
        <section class="record__section" aria-labelledby="section-triage">
            <h3 id="section-triage" tabindex="-1">{text::TRIAGE_TITLE}</h3>
            {
                let current = current.clone();
                move || current().map(|category| view! { <p><TriageBadge category /></p> })
            }
            <SectionInputs id="section-triage" open_text=text::TRIAGE_OPEN>
            <TriageAid id="retriage-algorithm" />
            <fieldset class="choices">
                <legend>{text::RETRIAGE_TO}</legend>
                {CATEGORIES
                    .into_iter()
                    .map(|c| {
                        let tag = tag.clone();
                        let current = current.clone();
                        view! {
                            <button
                                type="button"
                                class="button button--small"
                                aria-label=text::retriage_label(&tag, c)
                                hidden=move || current() == Some(c)
                                on:click=move |_| {
                                    let _ = core::retriage(core, revision, &tag, c);
                                }
                            >
                                {text::label(c)}
                            </button>
                        }
                    })
                    .collect_view()}
            </fieldset>
            <TransportPriority core revision tag=tag.clone() />
            </SectionInputs>
            <h4>{text::TRIAGE_HISTORY}</h4>
            <ol class="history">
                {move || {
                    state()
                        .map(|(_, history)| {
                            history
                                .into_iter()
                                .map(|(category, time, by, algorithm)| {
                                    // [impl->dsn~triage-reference~1]
                                    let used = algorithm.map(|a| {
                                        let name = TRIAGE.get(&a).map_or(a.clone(), |x| x.name.clone());
                                        format!(" {}", text::triage_with(&name))
                                    });
                                    view! { <li><TriageBadge category />{used} " " {text::by_at(&by, &time)}</li> }
                                })
                                .collect_view()
                        })
                }}
            </ol>
        </section>
    }
}

// [impl->dsn~treatment-places~3]
/// The patient's treatment place within their unit; places taken by others are marked.
#[component]
fn PlaceChoice(core: Handle, revision: RwSignal<u64>, tag: String) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            core.with_value(|c| {
                let register = PatientRegister::from_events(&c.events(med::COMPARTMENT));
                let patient = register.patient(&tag)?;
                let places = med::places(&c.events(med::UHS_COMPARTMENT))
                    .remove(&patient.uhs)
                    .unwrap_or_default();
                let options: Vec<PlaceOption> = places
                    .into_iter()
                    .map(|p| {
                        let taken = register
                            .patients()
                            .find(|o| {
                                o.tag != tag
                                    && o.uhs == patient.uhs
                                    && o.place.as_deref() == Some(&p.id)
                            })
                            .map(|o| o.tag.clone());
                        (p.category, p.id, p.label, taken)
                    })
                    .collect();
                Some((patient.place.clone(), options))
            })
        })
    };
    move || {
        let Some((current, options)) = state.get() else {
            return ().into_any();
        };
        if options.is_empty() {
            return ().into_any();
        }
        let tag = tag.clone();
        let labels: Vec<(String, String)> = options
            .iter()
            .map(|(_, id, label, _)| (id.clone(), label.clone()))
            .collect();
        let change = move |ev: leptos::ev::Event| {
            let value = event_target_value(&ev);
            let place = (!value.is_empty()).then_some(value);
            let label = place.as_ref().and_then(|p| {
                labels
                    .iter()
                    .find(|(id, _)| id == p)
                    .map(|(_, l)| l.clone())
            });
            let _ = core::assign_place(core, revision, &tag, place, label.as_deref());
        };
        view! {
            <div class="field place-choice">
                <label for="place">{text::PLACE}</label>
                <select id="place" on:change=change>
                    <option value="" selected=current.is_none()>{text::NO_PLACE}</option>
                    // The triage category first, its places under it (feedback of
                    // 2026-11-19); places kept for no category last.
                    {by_category(options)
                        .into_iter()
                        .map(|(category, places)| {
                            let group = category.map_or(text::PLACES_ANY, text::label);
                            let current = current.clone();
                            view! {
                                <optgroup label=group>
                                    {places.into_iter().map(|(id, label, taken)| {
                                        let selected = current.as_deref() == Some(id.as_str());
                                        let shown = match &taken {
                                            Some(by) => text::place_taken(&label, by),
                                            None => label,
                                        };
                                        view! { <option value=id selected=selected disabled=taken.is_some()>{shown}</option> }
                                    }).collect_view()}
                                </optgroup>
                            }
                        })
                        .collect_view()}
                </select>
            </div>
        }
        .into_any()
    }
}

/// A place to choose: its category, ID, label and who lies there, if anyone else does.
type PlaceOption = (Option<TriageCategory>, String, String, Option<String>);

/// The places of one category: ID, label and who lies there, if anyone else does.
type PlaceGroup = (
    Option<TriageCategory>,
    Vec<(String, String, Option<String>)>,
);

/// Places grouped by the triage category they are kept for, in the order of the categories,
/// places kept for no category last.
fn by_category(options: Vec<PlaceOption>) -> Vec<PlaceGroup> {
    let mut groups: Vec<PlaceGroup> = Vec::new();
    for (category, id, label, taken) in options {
        match groups.iter_mut().find(|(c, _)| *c == category) {
            Some((_, list)) => list.push((id, label, taken)),
            None => groups.push((category, vec![(id, label, taken)])),
        }
    }
    groups.sort_by_key(|(c, _)| (c.is_none(), *c));
    groups
}

/// Transport priority as a badge: an arrow and the label, never colour alone.
#[component]
fn PriorityBadge(label: String) -> impl IntoView {
    view! {
        <span class="priority-badge">
            <span class="priority-badge__icon" aria-hidden="true">"▲"</span>
            {label}
        </span>
    }
}

// [impl->dsn~transport-priority~1]
/// Transport priority of a patient, for the categories the profile allows; the person
/// decides, ORDO records it.
#[component]
fn TransportPriority(core: Handle, revision: RwSignal<u64>, tag: String) -> impl IntoView {
    let state = {
        let tag = tag.clone();
        Memo::new(move |_| {
            revision.track();
            core.with_value(|c| {
                let register = PatientRegister::from_events(&c.events(med::COMPARTMENT));
                register.patient(&tag).map(|p| {
                    (
                        TRANSPORT_PRIORITY.allows(p.category()),
                        p.transport_priority(&TRANSPORT_PRIORITY).map(str::to_owned),
                    )
                })
            })
        })
    };
    let set = move |tag: String, level: Option<String>| {
        let _ = core::set_transport_priority(core, revision, &tag, level);
    };
    move || {
        let Some((true, current)) = state.get() else {
            return ().into_any();
        };
        let tag = tag.clone();
        view! {
            <fieldset class="choices transport-priority">
                <legend>{text::TRANSPORT_PRIORITY}</legend>
                {current.clone().map(|level| view! { <p><PriorityBadge label=priority_label(&level) /></p> })}
                {TRANSPORT_PRIORITY
                    .levels
                    .iter()
                    .filter(|l| current.as_deref() != Some(l.id.as_str()))
                    .map(|l| {
                        let (tag, id) = (tag.clone(), l.id.clone());
                        view! {
                            <button type="button" class="button button--small"
                                aria-label=text::set_priority_label(&tag, &l.label)
                                on:click=move |_| set(tag.clone(), Some(id.clone()))>
                                {text::set_priority(&l.label)}
                            </button>
                        }
                    })
                    .collect_view()}
                {current.is_some().then(|| {
                    let tag = tag.clone();
                    view! {
                        <button type="button" class="button button--small"
                            aria-label=text::remove_priority_label(&tag)
                            on:click=move |_| set(tag.clone(), None)>
                            {text::REMOVE_PRIORITY}
                        </button>
                    }
                })}
            </fieldset>
        }
        .into_any()
    }
}

/// One form of the record: its documented values and, on demand, an editor in place.
#[component]
fn FormSection(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    form: &'static Form,
    records: Memo<Records>,
) -> impl IntoView {
    let drafts = use_context::<Drafts>().unwrap_or_default();
    let key = draft_key(&tag, &form.id);
    let editing = RwSignal::new(
        drafts
            .0
            .with_value(|d| d.get(&key).map(|(m, ..)| m.clone())),
    );
    let heading = format!("section-{}", form.id);
    let opener = match form.kind {
        FormKind::Snapshot => format!("edit-{}", form.id),
        FormKind::Series => format!("new-{}", form.id),
    };
    let content = {
        let tag = tag.clone();
        move || {
            records.with(|records| {
                let record = records.get(&tag);
                match form.kind {
                    FormKind::Snapshot => {
                        let facts: Vec<(String, String)> = form
                            .fields
                            .iter()
                            .filter_map(|f| {
                                let value = record?.current(&form.id, &f.id)?;
                                Some((f.label.clone(), f.display(&value.value)))
                            })
                            .collect();
                        if facts.is_empty() {
                            return view! { <p class="hint">{text::NOTHING_DOCUMENTED}</p> }.into_any();
                        }
                        view! { <Facts facts /> }.into_any()
                    }
                    FormKind::Series => {
                        let entries: Vec<record::Entry> = record
                            .map(|r| r.entries.get(&form.id).cloned().unwrap_or_default())
                            .unwrap_or_default();
                        if entries.is_empty() {
                            return view! { <p class="hint">{text::NOTHING_DOCUMENTED}</p> }.into_any();
                        }
                        let corrected = entries.iter().filter(|e| e.replaced_by.is_some()).count();
                        view! {
                            <ol class="entries" reversed>
                                {entries
                                    .iter()
                                    .rev()
                                    .filter(|e| e.replaced_by.is_none())
                                    .map(|e| view! { <EntryView core form entry=e.clone() editing /> })
                                    .collect_view()}
                            </ol>
                            {(corrected > 0).then(|| view! {
                                <details class="corrected">
                                    <summary>{text::corrected_entries(corrected)}</summary>
                                    <ol class="entries">
                                        {entries
                                            .iter()
                                            .filter(|e| e.replaced_by.is_some())
                                            .map(|e| view! { <EntryView core form entry=e.clone() editing=RwSignal::new(None) /> })
                                            .collect_view()}
                                    </ol>
                                </details>
                            })}
                        }
                        .into_any()
                    }
                }
            })
        }
    };
    let open_label = match form.kind {
        FormKind::Snapshot => text::edit_label(&form.label),
        FormKind::Series => text::new_entry_label(&form.label),
    };
    let (open_icon, open_text) = match form.kind {
        FormKind::Snapshot => ("pencil", text::edit()),
        FormKind::Series => ("plus", text::NEW_ENTRY),
    };
    let opener_id = opener.clone();
    view! {
        <section class="record__section" aria-labelledby=heading.clone()>
            <div class="record__section-head">
                <h3 id=heading.clone() tabindex="-1">{form.label.clone()}</h3>
                // [impl->dsn~icon-rules~1]
                <button
                    id=opener.clone()
                    type="button"
                    class="button button--small icon-button"
                    aria-label=open_label.clone()
                    data-tip=open_label
                    hidden=move || editing.get().is_some()
                    on:click={
                        let heading = heading.clone();
                        move |_| {
                            editing.set(Some(match form.kind {
                                FormKind::Snapshot => Mode::Edit,
                                FormKind::Series => Mode::New,
                            }));
                            let heading = heading.clone();
                            after_render(move || reveal_editor(&heading));
                        }
                    }
                >
                    <crate::shell::ui::Icon name=open_icon /><span class="icon-button__word" aria-hidden="true">{open_text}</span>
                </button>
            </div>
            {move || {
                editing.get().map(|mode| {
                    let opener_id = opener_id.clone();
                    view! {
                        <EntryEditor core revision tag=tag.clone() form mode records
                            on_close=Callback::new(move |_| {
                                editing.set(None);
                                focus(&opener_id);
                            }) />
                    }
                })
            }}
            {content}
        </section>
    }
}

/// The inputs of a section of the record, collapsed until opened, so the page first shows
/// the patient's state (product owner, 2026-11-17). Panel ID: `{id}-inputs`.
#[component]
pub(crate) fn SectionInputs(
    id: &'static str,
    /// What the button opens, e.g. "Sichtung ändern".
    open_text: &'static str,
    children: ChildrenFn,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let panel = format!("{id}-inputs");
    view! {
        <crate::shell::ui::DisclosureButton id=format!("{id}-open") controls=panel.clone() open
            class="button button--small record__open">
            {move || if open.get() { text::section_close() } else { open_text }}
        </crate::shell::ui::DisclosureButton>
        <div id=panel class="record__inputs" hidden=move || !open.get()>
            {children()}
        </div>
    }
}

/// Brings an opened editor into view and focuses its first field, also when it was
/// opened from the command palette (UX-10).
fn reveal_editor(heading: &str) {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    // Runs a moment after opening: if the person already moved into the editor, the
    // focus stays where they are, or their first input would land in another field.
    let editor = format!("[aria-labelledby='{heading}'] .editor");
    if document
        .active_element()
        .and_then(|e| e.closest(&editor).ok().flatten())
        .is_some()
    {
        return;
    }
    if let Some(section_head) = document.get_element_by_id(heading) {
        let options = web_sys::ScrollIntoViewOptions::new();
        options.set_block(web_sys::ScrollLogicalPosition::Start);
        section_head.scroll_into_view_with_scroll_into_view_options(&options);
    }
    let selector = format!("{editor} :is(input, select, textarea):not([type=hidden])");
    if let Some(field) = document
        .query_selector(&selector)
        .ok()
        .flatten()
        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = field.focus();
    }
}

#[component]
fn Facts(facts: Vec<(String, String)>) -> impl IntoView {
    view! {
        <dl class="facts">
            {facts
                .into_iter()
                .map(|(label, value)| view! { <div><dt>{label}</dt><dd>{value}</dd></div> })
                .collect_view()}
        </dl>
    }
}

#[component]
fn EntryView(
    core: Handle,
    form: &'static Form,
    entry: record::Entry,
    editing: RwSignal<Option<Mode>>,
) -> impl IntoView {
    let time = match entry.values.get("time") {
        Some(Value::One(t)) if !t.is_empty() => t.clone(),
        _ => core::clock(entry.at.wall_ms),
    };
    // [impl->dsn~order-documentation~3]
    // Who ordered it: from the order the entry documents (its author checked by the
    // server, or the giver named in a recorded order); a name without an order is marked.
    let from_order = entry.order.as_ref().and_then(|id| {
        core.with_value(|c| {
            ordo_med_patients::orders::OrderBook::from_events(
                &c.events(ordo_med_patients::orders::COMPARTMENT),
            )
            .orders
            .get(id)
            .map(|o| (o.giver(), core::clock(o.at().wall_ms)))
        })
    });
    let mut facts: Vec<(String, String)> = form
        .fields
        .iter()
        .filter(|f| f.id != "time")
        .filter_map(|f| {
            let value = entry.values.get(&f.id).filter(|v| !v.is_empty());
            if crate::med::orders::is_ordered_by(&form.id, &f.id) {
                let shown = match (&from_order, value) {
                    (Some((giver, at)), _) => text::ordered_by_order(
                        &crate::shell::profile::display_text(core, giver),
                        at,
                    ),
                    (None, Some(v)) => format!(
                        "{} – {}",
                        crate::shell::profile::display_text(core, &f.display(v)),
                        text::ORDERED_BY_NO_ORDER
                    ),
                    (None, None) => return None,
                };
                return Some((f.label.clone(), shown));
            }
            Some((f.label.clone(), f.display(value?)))
        })
        .collect();
    facts.extend(
        form.sums(&entry.values)
            .into_iter()
            .map(|(sum, total)| (sum.label.clone(), total)),
    );
    let corrected = entry.replaced_by.is_some();
    let id = entry.id.clone();
    view! {
        <li class="entry" class:entry--corrected=corrected>
            <p class="entry__head">
                <strong>{time.clone()}</strong> " · " {crate::shell::profile::naming({
                    let by = entry.by.clone();
                    move || crate::shell::profile::display_name(core, &by)
                })}
                {corrected.then(|| view! { " · " <em>{text::CORRECTED}</em> })}
            </p>
            <Facts facts />
            {(!corrected).then(|| view! {
                <crate::shell::ui::IconButton icon="pencil" word=text::CORRECT
                    label=format!("{}: {} {}", text::CORRECT, form.label, time)
                    on_click=Callback::new(move |()| editing.set(Some(Mode::Correct(id.clone())))) />
            })}
        </li>
    }
}

/// Initial editor values: the draft, else the current values (edit), the corrected
/// entry (correction), or empty with the current time (new entry).
fn initial_values(
    form: &Form,
    mode: &Mode,
    record: Option<&record::Record>,
) -> BTreeMap<String, Value> {
    let empty = |f: &Field| match f.field_type {
        FieldType::Choices => Value::Many(Vec::new()),
        _ => Value::One(String::new()),
    };
    form.fields
        .iter()
        .map(|f| {
            let value = match mode {
                Mode::Edit => record
                    .and_then(|r| r.current(&form.id, &f.id))
                    .map(|s| s.value.clone()),
                Mode::Correct(id) => record
                    .and_then(|r| r.entries.get(&form.id))
                    .and_then(|list| list.iter().find(|e| &e.id == id))
                    .and_then(|e| e.values.get(&f.id).cloned()),
                Mode::New if f.field_type == FieldType::Time => Some(Value::One(core::now_clock())),
                Mode::New => None,
            };
            (f.id.clone(), value.unwrap_or_else(|| empty(f)))
        })
        .collect()
}

/// Editor for one form, in place on the record page.
#[component]
fn EntryEditor(
    core: Handle,
    revision: RwSignal<u64>,
    tag: String,
    form: &'static Form,
    mode: Mode,
    records: Memo<Records>,
    on_close: Callback<()>,
) -> impl IntoView {
    let drafts = use_context::<Drafts>().unwrap_or_default();
    let key = draft_key(&tag, &form.id);
    let draft = drafts.0.with_value(|d| {
        d.get(&key)
            .filter(|(m, ..)| *m == mode)
            .map(|(_, v, o)| (v.clone(), o.clone()))
    });
    // [impl->dsn~order-documentation~3]
    // The order it documents: from the draft, or the one of the entry it corrects.
    let order = match &draft {
        Some((_, order)) => order.clone(),
        None => match &mode {
            Mode::Correct(id) => records.with_untracked(|r| {
                r.get(&tag)
                    .and_then(|r| r.entries.get(&form.id))
                    .and_then(|list| list.iter().find(|e| &e.id == id))
                    .and_then(|e| e.order.clone())
            }),
            _ => None,
        },
    };
    let start = draft.map_or_else(
        || records.with_untracked(|r| initial_values(form, &mode, r.get(&tag))),
        |(v, _)| v,
    );
    let values: Vec<(&'static Field, RwSignal<Value>)> = form
        .fields
        .iter()
        .map(|f| {
            (
                f,
                RwSignal::new(
                    start
                        .get(&f.id)
                        .cloned()
                        .unwrap_or(Value::One(String::new())),
                ),
            )
        })
        .collect();
    let errors = RwSignal::new(BTreeMap::<String, String>::new());
    let message = RwSignal::new(None::<&'static str>);
    {
        // Keep unsaved input while the app runs.
        let values = values.clone();
        let (key, mode, order) = (key.clone(), mode.clone(), order.clone());
        Effect::new(move |_| {
            let snapshot: BTreeMap<String, Value> = values
                .iter()
                .map(|(f, v)| (f.id.clone(), v.get()))
                .collect();
            drafts.0.update_value(|d| {
                d.insert(key.clone(), (mode.clone(), snapshot, order.clone()));
            });
        });
    }
    let close = {
        let key = key.clone();
        move || {
            drafts.0.update_value(|d| {
                d.remove(&key);
            });
            on_close.run(());
        }
    };
    let submit = {
        let values = values.clone();
        let close = close.clone();
        let mode = mode.clone();
        let order = order.clone();
        move |ev: leptos::ev::SubmitEvent| {
            ev.prevent_default();
            let current =
                records.with_untracked(|r| initial_values(form, &Mode::Edit, r.get(&tag)));
            let chosen: BTreeMap<String, Value> = values
                .iter()
                .map(|(f, v)| (f.id.clone(), v.get_untracked()))
                .filter(|(id, v)| match mode {
                    // Snapshot: only what changed. Series: what was filled in.
                    Mode::Edit => {
                        let field = form.field(id).expect("own field");
                        field.normalise(v).ok() != field.normalise(&current[id]).ok()
                    }
                    Mode::New | Mode::Correct(_) => !v.is_empty(),
                })
                .collect();
            match form.normalise(&chosen) {
                Err(problems) => {
                    errors.set(
                        problems
                            .iter()
                            .map(|(id, e)| (id.clone(), text::value_error(e)))
                            .collect(),
                    );
                    message.set(Some(text::CHECK_INPUT));
                    if let Some((id, _)) = problems.first() {
                        focus(&format!("f-{}-{id}", form.id));
                    }
                }
                Ok(clean) if clean.is_empty() => {
                    errors.set(BTreeMap::new());
                    message.set(Some(match mode {
                        Mode::Edit => text::NO_CHANGES,
                        _ => text::EMPTY_ENTRY,
                    }));
                }
                Ok(clean) => {
                    let entry = EntryRecorded {
                        tag: tag.clone(),
                        form: form.id.clone(),
                        values: clean,
                        replaces: match &mode {
                            Mode::Correct(id) => Some(id.clone()),
                            _ => None,
                        },
                        order: order.clone(),
                    };
                    match core::record_entry(
                        core,
                        revision,
                        &entry,
                        text::entry_saved(&form.label, &tag),
                    ) {
                        Ok(()) => close(),
                        Err(_) => message.set(Some(text::CHECK_INPUT)),
                    }
                }
            }
        }
    };
    let title_id = format!("editor-{}", form.id);
    let labelled_by = title_id.clone();
    view! {
        <form class="form editor" on:submit=submit aria-labelledby=labelled_by novalidate>
            <p id=title_id class="visually-hidden">{form.label.clone()}</p>
            {values
                .into_iter()
                .map(|(field, value)| {
                    let error = Signal::derive(move || errors.with(|e| e.get(&field.id).cloned()));
                    view! { <FieldInput form field value error /> }
                })
                .collect_view()}
            <p role="alert" class="error">{move || message.get()}</p>
            <div class="row">
                <button type="submit" class="button button--primary">{text::save()}</button>
                <button type="button" class="button" on:click=move |_| close()>{text::cancel()}</button>
            </div>
        </form>
    }
}

/// One field of an editor, by type. Labels, hints and errors are linked for assistive
/// technology; numbers keep the decimal keyboard on phones.
#[component]
fn FieldInput(
    form: &'static Form,
    field: &'static Field,
    value: RwSignal<Value>,
    error: Signal<Option<String>>,
) -> impl IntoView {
    let id = format!("f-{}-{}", form.id, field.id);
    let hint_id = format!("{id}-hint");
    let error_id = format!("{id}-error");
    let described = {
        let mut ids = Vec::new();
        if field.hint.is_some() || field.unit.is_some() {
            ids.push(hint_id.clone());
        }
        ids.push(error_id.clone());
        ids.join(" ")
    };
    let hint = match (&field.unit, &field.hint) {
        (Some(unit), Some(hint)) => Some(format!("{unit} · {hint}")),
        (Some(unit), None) => Some(unit.clone()),
        (None, hint) => hint.clone(),
    };
    let text_value = move || match value.get() {
        Value::One(t) => t,
        Value::Many(_) => String::new(),
    };
    let set_text = move |ev: leptos::ev::Event| value.set(Value::One(event_target_value(&ev)));
    let invalid = move || error.get().is_some().then_some("true");
    let input = match field.field_type {
        FieldType::Text | FieldType::Number | FieldType::Date | FieldType::Time => {
            let (input_type, mode) = match field.field_type {
                FieldType::Number => ("text", Some("decimal")),
                FieldType::Date => ("date", None),
                FieldType::Time => ("time", None),
                _ => ("text", None),
            };
            let now = (field.field_type == FieldType::Time).then(|| {
                view! {
                    <button type="button" class="button button--small" on:click=move |_| value.set(Value::One(core::now_clock()))>
                        {text::NOW}
                    </button>
                }
            });
            view! {
                <label for=id.clone()>{field.label.clone()}</label>
                <div class="row row--tight">
                    <input
                        id=id.clone()
                        type=input_type
                        inputmode=mode
                        autocomplete="off"
                        aria-describedby=described.clone()
                        aria-invalid=invalid
                        prop:value=text_value
                        on:input=set_text
                    />
                    {now}
                </div>
            }
            .into_any()
        }
        // [impl->dsn~order-documentation~3]
        // Who ordered a medication comes only from an order (ORDER-7).
        FieldType::Person if crate::med::orders::is_ordered_by(&form.id, &field.id) => {
            let note_id = format!("{id}-from-order");
            view! {
                <label for=id.clone()>{field.label.clone()}</label>
                <input
                    id=id.clone()
                    type="text"
                    readonly
                    aria-describedby=format!("{note_id} {described}")
                    prop:value=text_value
                />
                <p id=note_id class="hint">{text::ORDERED_BY_FROM_ORDER}</p>
            }
            .into_any()
        }
        // [impl->dsn~person-suggestions~1]
        FieldType::Person => {
            let people =
                use_context::<IncidentPeople>().map_or_else(|| Signal::stored(Vec::new()), |p| p.0);
            let list = format!("{id}-people");
            view! {
                <label for=id.clone()>{field.label.clone()}</label>
                <input
                    id=id.clone()
                    type="text"
                    list=list.clone()
                    autocomplete="off"
                    aria-describedby=described.clone()
                    aria-invalid=invalid
                    prop:value=text_value
                    on:input=set_text
                />
                {match use_context::<IncidentPeople>().map(|p| p.1) {
                    Some(core) => view! { <Suggestions id=list.clone() values=people core /> }.into_any(),
                    None => view! { <Suggestions id=list.clone() values=people /> }.into_any(),
                }}
            }
            .into_any()
        }
        FieldType::Longtext => view! {
            <label for=id.clone()>{field.label.clone()}</label>
            <textarea
                id=id.clone()
                rows="3"
                aria-describedby=described.clone()
                aria-invalid=invalid
                prop:value=text_value
                on:input=set_text
            ></textarea>
        }
        .into_any(),
        FieldType::Choice => {
            let options = std::iter::once((String::new(), text::NO_ANSWER.to_owned())).chain(
                field
                    .options
                    .iter()
                    .map(|o| (o.id.clone(), o.label.clone())),
            );
            view! {
                <fieldset class="choices" id=id.clone() tabindex="-1" aria-describedby=described.clone()>
                    <legend>{field.label.clone()}</legend>
                    {options
                        .map(|(option, label)| {
                            let checked = {
                                let option = option.clone();
                                move || value.get() == Value::One(option.clone())
                            };
                            view! {
                                <label class="choice chip">
                                    <input
                                        type="radio"
                                        name=id.clone()
                                        prop:checked=checked
                                        on:change=move |_| value.set(Value::One(option.clone()))
                                    />
                                    {label}
                                </label>
                            }
                        })
                        .collect_view()}
                </fieldset>
            }
            .into_any()
        }
        FieldType::Choices => {
            // [impl->dsn~open-choice~1]
            // An open choice takes one other answer as free text, kept after the options.
            let other = field.open.then(|| {
                let codes: Vec<String> = field.options.iter().map(|o| o.id.clone()).collect();
                let other_id = format!("{id}-other");
                let current = {
                    let codes = codes.clone();
                    move || match value.get() {
                        Value::Many(list) => list.into_iter().find(|x| !codes.contains(x)).unwrap_or_default(),
                        Value::One(_) => String::new(),
                    }
                };
                let set = move |ev: leptos::ev::Event| {
                    let typed = event_target_value(&ev);
                    value.update(|v| {
                        let mut list = match v {
                            Value::Many(list) => std::mem::take(list),
                            Value::One(_) => Vec::new(),
                        };
                        list.retain(|x| codes.contains(x));
                        if !typed.is_empty() {
                            list.push(typed);
                        }
                        *v = Value::Many(list);
                    });
                };
                view! {
                    <div class="field choices__other">
                        <label for=other_id.clone()>{text::OTHER_ANSWER}</label>
                        <input id=other_id type="text" autocomplete="off" maxlength=ordo_med_patients::record::MAX_OTHER_CHARS.to_string() prop:value=current on:input=set />
                    </div>
                }
            });
            view! {
            <fieldset class="choices" id=id.clone() tabindex="-1" aria-describedby=described.clone()>
                <legend>{field.label.clone()}</legend>
                {field
                    .options
                    .iter()
                    .map(|o| {
                        let option = o.id.clone();
                        let checked = {
                            let option = option.clone();
                            move || matches!(value.get(), Value::Many(list) if list.contains(&option))
                        };
                        view! {
                            <label class="choice chip">
                                <input
                                    type="checkbox"
                                    prop:checked=checked
                                    on:change=move |ev| {
                                        let on = event_target_checked(&ev);
                                        value.update(|v| {
                                            let mut list = match v {
                                                Value::Many(list) => std::mem::take(list),
                                                Value::One(_) => Vec::new(),
                                            };
                                            list.retain(|x| x != &option);
                                            if on {
                                                list.push(option.clone());
                                            }
                                            *v = Value::Many(list);
                                        });
                                    }
                                />
                                {o.label.clone()}
                            </label>
                        }
                    })
                    .collect_view()}
                {other}
            </fieldset>
            }
            .into_any()
        }
    };
    view! {
        <div class="field" class:field--invalid=move || error.get().is_some()>
            {input}
            {hint.map(|h| view! { <p id=hint_id class="hint">{h}</p> })}
            <p id=error_id class="field-error">{move || error.get()}</p>
        </div>
    }
}
