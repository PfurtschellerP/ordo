//! The on-site care units (UHS) as the incident command sets them up and sees them
//! (UHS-1, UHS-4, UHS-11, ADR-0086): the list on *Vorbereitung – Einsatzmittel* with each
//! UHS's details panel and the form for a new one, and the counts per UHS on the situation
//! page. The UHS module itself only reads them ([`crate::med::places`]).

use leptos::prelude::*;
use ordo_contracts::TriageCategory::{self, Expectant, Immediate, Minor, Uninjured, Urgent};
use ordo_med_patients::{self as med, UhsCreated};

use crate::core::{self, Handle};
use crate::med::text;
use crate::med::triage::{CATEGORIES, TriageBadge};
use crate::shell::views::follow_route;

/// The on-site care units of the incident (from the shared compartment).
pub(crate) fn uhs_list(core: Handle) -> Vec<UhsCreated> {
    core.with_value(|c| {
        let events = c.events(med::UHS_COMPARTMENT);
        med::uhs_list(&events)
    })
}

/// What someone types into the UHS dialog: one draft for the list's one dialog, filled
/// with the UHS's values when the dialog opens, and kept while the list updates.
#[derive(Clone, Copy)]
struct UhsDraft {
    name: RwSignal<String>,
    capacity: RwSignal<String>,
    person: RwSignal<String>,
    organisation: RwSignal<String>,
    service: RwSignal<String>,
    /// Its strength as a unit.
    strength: crate::shared::units::StrengthDraft,
    /// Capacity per category, in the order of [`CAPACITY_CATEGORIES`].
    capacities: [RwSignal<String>; 5],
    /// Where the person being added to the staff is assigned already (UNIT-8).
    elsewhere: RwSignal<Vec<ordo_units::Post>>,
    /// Whether the details were saved since the last change: said after the dialog's
    /// content is rebuilt with the saved values.
    saved: RwSignal<bool>,
}

/// The triage categories a UHS can keep capacity and places for (UHS-13): all but the
/// deceased, who take no treatment place.
const CAPACITY_CATEGORIES: [TriageCategory; 5] = [Immediate, Urgent, Minor, Expectant, Uninjured];

impl UhsDraft {
    fn new() -> Self {
        Self {
            name: RwSignal::new(String::new()),
            capacity: RwSignal::new(String::new()),
            person: RwSignal::new(String::new()),
            organisation: RwSignal::new(String::new()),
            service: RwSignal::new(String::new()),
            strength: crate::shared::units::StrengthDraft::new(None, &Default::default()),
            capacities: CAPACITY_CATEGORIES.map(|_| RwSignal::new(String::new())),
            elsewhere: RwSignal::new(Vec::new()),
            saved: RwSignal::new(false),
        }
    }

    /// Fills the draft with the current values of a UHS.
    fn fill(&self, name: &str, capacity: u32, details: &med::UhsDetails) {
        self.name.set(name.to_owned());
        self.capacity.set(capacity.to_string());
        self.person.set(String::new());
        self.organisation
            .set(details.organisation.clone().unwrap_or_default());
        self.service
            .set(details.service.clone().unwrap_or_default());
        self.strength.fill(details.strength, &details.special);
        for (c, value) in CAPACITY_CATEGORIES.into_iter().zip(self.capacities) {
            value.set(
                details
                    .capacities
                    .get(&c)
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            );
        }
        self.elsewhere.set(Vec::new());
        self.saved.set(false);
    }
}

// [impl->dsn~crews-from-register~2]
/// A UHS staff entry (a user ID or name) as a person: the helper whose account it is, if
/// the register names one (HELP-2).
fn staff_member(core: Handle, entry: &str) -> crate::shared::units::Who {
    let helper = crate::shared::helpers::register(core)
        .and_then(|r| r.of_account(entry).map(|h| h.id.clone()));
    crate::shared::units::Who {
        name: entry.to_owned(),
        helper,
        account: Some(entry.to_owned()),
    }
}

// [impl->dsn~uhs-details~4]
/// Name, capacity, place on the situation map and assigned staff of one UHS (UHS-1), in
/// the list's edit dialog.
#[component]
fn UhsDetailsPanel(
    core: Handle,
    revision: RwSignal<u64>,
    id: String,
    name: String,
    /// Its capacity now: a change sets its places again (ADR-0086).
    capacity: u32,
    details: med::UhsDetails,
    draft: UhsDraft,
    /// Patients in it now, from the counts: a UHS with patients is not closed (UHS-17).
    #[prop(into)]
    patients: Signal<u32>,
    /// Closes the dialog.
    on_done: Callback<()>,
) -> impl IntoView {
    let UhsDraft {
        name: new_name,
        capacity: cap,
        person,
        organisation,
        service,
        strength,
        capacities,
        elsewhere,
        saved,
    } = draft;
    let slug = core::slug(&id);
    let staff = details.staff.clone();
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    let error = RwSignal::new(None::<String>);
    let current_capacity = capacity;
    // How the capacity is split, as typed: places by category and the rest for any.
    let split = Memo::new(move |_| {
        let total = cap.get().trim().parse::<u32>().ok()?;
        let kept: u32 = capacities
            .iter()
            .filter_map(|c| c.get().trim().parse::<u32>().ok())
            .sum();
        Some((kept, total))
    });
    // Changed since opened or last saved: closing the dialog saves it (dsn edit-dialog).
    let changed_here = RwSignal::new(false);
    // [impl->dsn~uhs-capacity-by-category~2]
    // More places by category than the capacity: those fields are invalid while typed.
    let exceeded = Memo::new(move |_| split.get().is_some_and(|(kept, total)| kept > total));
    let save = {
        let (id, current_name) = (id.clone(), name.clone());
        Callback::new(move |()| -> bool {
            // [impl->dsn~uhs-capacity-by-category~2]
            // The capacity is the whole; places per category, if given, fit in it.
            let capacity = match cap.get_untracked().trim().parse::<u32>() {
                Ok(n) if n > 0 => n,
                _ => {
                    error.set(Some(text::UHS_CAPACITY_NUMBER.to_owned()));
                    return false;
                }
            };
            let mut by_category = std::collections::BTreeMap::new();
            for (category, value) in CAPACITY_CATEGORIES.into_iter().zip(capacities) {
                let value = value.get_untracked();
                if value.trim().is_empty() {
                    continue;
                }
                match value.trim().parse::<u32>() {
                    Ok(0) => {}
                    Ok(n) => {
                        by_category.insert(category, n);
                    }
                    Err(_) => {
                        error.set(Some(text::UHS_CAPACITY_NUMBER.to_owned()));
                        return false;
                    }
                }
            }
            let kept = by_category.values().sum::<u32>();
            if kept > capacity {
                error.set(Some(text::uhs_places_exceed(kept, capacity)));
                return false;
            }
            let chosen = new_name.get_untracked().trim().to_owned();
            if chosen.is_empty() {
                error.set(Some(text::UHS_NAME_NEEDED.to_owned()));
                return false;
            }
            // [impl->dsn~uhs-as-unit~1]
            // Its strength as a unit, or none, and its special functions.
            let (reported, special) = match strength.read() {
                Ok(read) => read,
                Err(e) => {
                    error.set(Some(e.to_owned()));
                    return false;
                }
            };
            let was = (
                details.organisation.clone().unwrap_or_default(),
                details.service.clone().unwrap_or_default(),
                details.strength,
            );
            let now = (
                organisation.get_untracked().trim().to_owned(),
                service.get_untracked(),
                reported,
            );
            let update = med::UhsUpdated {
                id: id.clone(),
                // Only a changed name is sent: the log then shows the renaming.
                name: (chosen != current_name).then_some(chosen),
                // The place is the UHS on the situation map (ADR-0064); a location text
                // written before stays as it is.
                location: None,
                capacity: Some(capacity),
                staff: None,
                // Only what changed is sent, as for the name.
                organisation: (now.0 != was.0).then(|| now.0.clone()),
                service: (now.1 != was.1).then(|| now.1.clone()),
                strength: (now.2 != was.2)
                    .then_some(now.2.map(|s| [s.leaders, s.sub_leaders, s.crew])),
                // [impl->dsn~detailed-strength~1]
                unit_leaders: (now.2.map(|s| s.unit_leaders) != was.2.map(|s| s.unit_leaders))
                    .then(|| now.2.map_or(0, |s| s.unit_leaders)),
                special: (special != details.special).then(|| special.clone()),
                capacities: (by_category != details.capacities).then(|| by_category.clone()),
            };
            if let Err(e) = core::update_uhs(core, revision, &update) {
                error.set(Some(e));
                return false;
            }
            // [impl->dsn~treatment-places~3]
            // The places follow the capacity: per category and the rest for any, or one per
            // patient.
            let changed = update.capacities.is_some() || capacity != current_capacity;
            let placed = match (changed, by_category.is_empty()) {
                (false, _) => Ok(()),
                (true, false) => {
                    core::set_category_places(core, revision, &id, &by_category, capacity)
                }
                (true, true) => core::set_places(core, revision, &id, capacity),
            };
            match placed {
                Ok(()) => {
                    error.set(None);
                    changed_here.set(false);
                    saved.set(true);
                    true
                }
                Err(e) => {
                    error.set(Some(e));
                    false
                }
            }
        })
    };
    // [impl->dsn~edit-dialog~3]
    crate::shell::ui::PendingSaves::register(Callback::new(move |()| {
        !changed_here.get_untracked() || save.run(())
    }));
    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        save.run(());
    };
    let edited_now = move |_| {
        saved.set(false);
        changed_here.set(true);
    };
    let commit = {
        let (id, staff) = (id.clone(), staff.clone());
        Callback::new(move |who: String| {
            let mut list = staff.clone();
            // Moved here while on the list already: kept once.
            list.retain(|s| !ordo_units::same_person(s, &who));
            list.push(who);
            let update = med::UhsUpdated {
                id: id.clone(),
                staff: Some(list),
                ..Default::default()
            };
            if core::update_uhs(core, revision, &update).is_ok() {
                person.set(String::new());
                elsewhere.set(Vec::new());
            }
        })
    };
    let here = med::uhs_post(&id, &name);
    // [impl->dsn~double-booking~1]
    // Someone on a crew or another UHS is moved here, not counted twice (UNIT-8).
    // Adds the person typed; false while it waits for the question whether to move them.
    let add_person = {
        let here = here.clone();
        Callback::new(move |()| -> bool {
            let who = person.get_untracked().trim().to_owned();
            if who.is_empty() {
                return true;
            }
            let away = staff_member(core, &who).elsewhere(core, &here);
            if away.is_empty() {
                commit.run(who);
                true
            } else {
                elsewhere.set(away);
                false
            }
        })
    };
    // [impl->dsn~edit-dialog~3]
    // A person typed and not yet added is added when the dialog closes.
    crate::shell::ui::PendingSaves::register(add_person);
    let add = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        add_person.run(());
    };
    let on_move = Callback::new(move |()| {
        let who = person.get_untracked().trim().to_owned();
        crate::shared::units::move_away(
            core,
            revision,
            &staff_member(core, &who),
            &elsewhere.get_untracked(),
        );
        commit.run(who);
    });
    let members = staff
        .iter()
        .enumerate()
        .map(|(i, member)| {
            let (id, staff) = (id.clone(), staff.clone());
            let shown = crate::shell::profile::display_text(core, member);
            // [impl->dsn~icon-rules~1]
            let remove = Callback::new(move |()| {
                let mut list = staff.clone();
                list.remove(i);
                let update = med::UhsUpdated {
                    id: id.clone(),
                    staff: Some(list),
                    ..Default::default()
                };
                if core::update_uhs(core, revision, &update).is_ok() {
                    let back = med::UhsUpdated {
                        id: id.clone(),
                        staff: Some(staff.clone()),
                        ..Default::default()
                    };
                    crate::shell::ui::offer_undo(text::removed(&shown), move || {
                        let _ = core::update_uhs(core, revision, &back);
                    });
                }
            });
            let name = {
                let member = member.clone();
                crate::shell::profile::naming(move || {
                    crate::shell::profile::display_text(core, &member)
                })
            };
            view! {
                <li class="staff-member">
                    <strong>{name}</strong>
                    <crate::shell::ui::IconButton icon="trash-2" word=text::CREW_REMOVE quiet=true
                        label=text::staff_remove_label(member) on_click=remove />
                </li>
            }
        })
        .collect_view();
    let part = |key: &str| format!("uhs-part-{key}-{slug}");
    view! {
        <div class="edit-dialog__columns">
            <section class="edit-dialog__part" aria-labelledby=part("data")>
                <h3 id=part("data")>{text::UHS_DATA}</h3>
                    <form class="form" novalidate on:submit=submit on:input=edited_now on:change=edited_now>
                        <div class="field">
                            <label for=format!("uhs-name-{slug}")>{text::UHS_NAME}</label>
                            <input id=format!("uhs-name-{slug}") type="text" autocomplete="off" bind:value=new_name />
                        </div>
                        <div class="field field--small">
                            <label for=format!("uhs-capacity-{slug}")>{text::UHS_CAPACITY}</label>
                            <input id=format!("uhs-capacity-{slug}") type="text" inputmode="numeric" autocomplete="off" bind:value=cap />
                        </div>
                        <div class="field">
                            <label for=format!("uhs-organisation-{slug}")>{text::UNIT_ORGANISATION}</label>
                            <input id=format!("uhs-organisation-{slug}") type="text" autocomplete="off" bind:value=organisation />
                        </div>
                        <div class="field">
                            <label for=format!("uhs-service-{slug}")>{text::UNIT_SERVICE}</label>
                            <select id=format!("uhs-service-{slug}") prop:value=move || service.get() on:change=move |ev| service.set(event_target_value(&ev))>
                                <option value="">{text::NO_SERVICE}</option>
                                {crate::shared::units::CATALOGUE.services.iter().map(|s| view! { <option value=s.id.clone()>{s.label.clone()}</option> }).collect_view()}
                            </select>
                        </div>
                        <crate::shared::units::StrengthFields draft=strength prefix=format!("uhs-{slug}") title=text::UHS_STRENGTH
                            detailed=Signal::derive(move || { revision.track(); crate::shared::units::detailed(core) }) />
                        <p id=format!("uhs-strength-hint-{slug}") class="hint">{text::UHS_STRENGTH_HINT}</p>
                        <fieldset class="choices strength-input" aria-describedby=format!("uhs-capacities-hint-{slug}")>
                            <legend>{text::UHS_CAPACITIES_LEGEND}</legend>
                            {CAPACITY_CATEGORIES
                                .into_iter()
                                .zip(capacities)
                                .map(|(category, value)| {
                                    let id = format!("uhs-capacity-{}-{slug}", core::slug(text::label(category)));
                                    // Only a field with places marks the excess.
                                    let invalid = move || exceeded.get() && !value.with(|v| v.trim().is_empty() || v.trim() == "0");
                                    view! {
                                        <div class="field field--small" class:field--invalid=invalid>
                                            <label for=id.clone()>{text::label(category)}</label>
                                            <input id=id type="text" inputmode="numeric" autocomplete="off" bind:value=value
                                                title=text::description(category)
                                                aria-invalid=move || invalid().then_some("true") />
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </fieldset>
                        <p id=format!("uhs-capacities-hint-{slug}") class="hint">
                            {text::UHS_CAPACITIES_HINT} " "
                            {move || split.get().map(|(kept, total)| text::uhs_places_split(kept, total))}
                        </p>
                        <div class="actions">
                            <button type="submit" class="button">{text::UHS_SAVE}</button>
                            <p role="status" class="hint">{move || saved.get().then_some(text::SAVED)}</p>
                        </div>
                    </form>
                <p role="alert" class="error">{move || error.get()}</p>
            </section>
            <section class="edit-dialog__part" aria-labelledby=part("place")>
                <h3 id=part("place")>{text::UHS_ON_MAP}</h3>
                <UhsOnMap core revision id=id.clone() name=name.clone() written=details.location.clone() />
            </section>
            <section class="edit-dialog__part" aria-labelledby=part("staff")>
                <h3 id=part("staff")>{text::UHS_STAFF}</h3>
                {if staff.is_empty() {
                    view! { <p class="hint">{text::UHS_STAFF_NONE}</p> }.into_any()
                } else {
                    view! { <ul class="crew-list">{members}</ul> }.into_any()
                }}
                <form class="form form--inline" novalidate on:submit=add>
                    <div class="field">
                        <label for=format!("uhs-staff-{slug}")>{text::CREW_MEMBER}</label>
                        <input id=format!("uhs-staff-{slug}") type="text" autocomplete="off"
                            list=format!("uhs-staff-{slug}-people") bind:value=person />
                        <crate::shell::ui::Suggestions id=format!("uhs-staff-{slug}-people") values=people core />
                    </div>
                    <button type="submit" class="button button--small">{text::UHS_STAFF_ADD}</button>
                </form>
                <crate::shared::units::AssignedElsewhere person elsewhere here=here.label.clone() on_move />
            </section>
        </div>
        <CloseUhs core revision id=id.clone() name=name.clone() patients on_done />
    }
}

// [impl->dsn~uhs-closing~1]
/// Closes a UHS that holds no patients (UHS-17), after asking once more. With patients in
/// it, the button is off and says why.
#[component]
fn CloseUhs(
    core: Handle,
    revision: RwSignal<u64>,
    id: String,
    name: String,
    patients: Signal<u32>,
    /// Closes the dialog it sits in, with the UHS.
    on_done: Callback<()>,
) -> impl IntoView {
    let asking = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);
    let slug = core::slug(&id);
    let hint = format!("uhs-close-hint-{slug}");
    let confirm = {
        let id = id.clone();
        move |_| match core::close_uhs(core, revision, &id) {
            Ok(()) => {
                asking.set(false);
                on_done.run(());
            }
            Err(e) => error.set(Some(e)),
        }
    };
    view! {
        <div class="danger-zone">
            <h3>{text::UHS_CLOSE_TITLE}</h3>
            <p id=hint.clone() class="hint">{move || match patients.get() {
                0 => text::UHS_CLOSE_HINT.to_owned(),
                n => text::uhs_close_blocked(n),
            }}</p>
            <Show when=move || !asking.get()>
                <button type="button" class="button button--small button--danger" disabled={move || patients.get() > 0}
                    aria-describedby=hint.clone() on:click=move |_| asking.set(true)>
                    {text::UHS_CLOSE}
                </button>
            </Show>
            <Show when=move || asking.get()>
                <p role="alert">{text::uhs_close_question(&name)}</p>
                <div class="actions">
                    <button type="button" class="button button--small button--danger" on:click=confirm.clone()>{text::UHS_CLOSE_CONFIRM}</button>
                    <button type="button" class="button button--small button--quiet" on:click=move |_| asking.set(false)>{text::CANCEL}</button>
                </div>
            </Show>
            <p role="alert" class="error">{move || error.get()}</p>
        </div>
    }
}

// [impl->dsn~uhs-details~4]
/// Where a UHS is: its place on the situation map, and a location text written before
/// (kept, events are never changed). The incident command sets the place on the map.
#[component]
fn UhsOnMap(
    core: Handle,
    revision: RwSignal<u64>,
    id: String,
    name: String,
    written: Option<String>,
) -> impl IntoView {
    let object = ordo_map::ObjectKind::Uhs.object_id(&id);
    let at = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            ordo_map::SituationMap::from_events(&c.events(ordo_map::COMPARTMENT))
                .objects
                .get(&object)
                .and_then(|o| o.placed.geometry.anchor())
        })
    });
    let may_place = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.may(core::permission::VIEW_SITUATION) && c.may(core::permission::EDIT_UNITS)
        })
    });
    let place = crate::shared::situation::editor_route(Some(
        &crate::shared::situation::place_record(ordo_map::ObjectKind::Uhs, &id),
    ));
    let name = StoredValue::new(name);
    let visible = move || {
        if at.get().is_some() {
            text::UHS_MOVE_ON_MAP
        } else {
            text::UHS_PLACE_ON_MAP
        }
    };
    view! {
        <p class="uhs-on-map">
            {text::UHS_ON_MAP} ": "
            <span class="uhs-on-map__at">{move || at.get().map_or_else(
                || text::UHS_NOT_ON_MAP.to_owned(),
                |[lon, lat]| format!("{lat:.5}, {lon:.5}"),
            )}</span>
        </p>
        {written.filter(|w| !w.is_empty()).map(|w| view! {
            <p class="hint">{text::UHS_LOCATION} ": " {w}</p>
        })}
        <Show when=move || may_place.get()>
            <a class="button button--small" href=place.to_path() on:click=follow_route(place.clone())
                aria-label=move || name.with_value(|n| text::uhs_place_label(visible(), n))>
                {visible}
            </a>
        </Show>
    }
}

// [impl->dsn~uhs-resources~2]
// [impl->dsn~uhs-registry~2]
/// The UHS among the resources (UHS-1, UHS-11, ADR-0086): each with its staff, which the
/// incident command assigns there; location, capacity and places in the same panel; and the
/// form that sets up a new one. The only place where a UHS is set up or changed.
#[component]
pub(crate) fn UhsResources(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            let events = c.events(med::UHS_COMPARTMENT);
            let details = med::uhs_details(&events);
            let counts = med::latest_counts(&events);
            med::uhs_list(&events)
                .into_iter()
                .map(|u| {
                    let d = details.get(&u.id).cloned().unwrap_or_default();
                    let held: u32 = counts
                        .get(&u.id)
                        .map_or(0, |c| CATEGORIES.iter().map(|k| c.get(*k)).sum());
                    (u.id, u.name, u.capacity, d, held)
                })
                .collect::<Vec<_>>()
        })
    });
    let editable = core::permitted(core, revision, core::permission::EDIT_UNITS);
    // [impl->dsn~edit-dialog~3]
    // One dialog for the list: the UHS being edited, as the dialog shows it, changes only
    // when that UHS changes; the list may update under it.
    let draft = UhsDraft::new();
    let editing = RwSignal::new(None::<String>);
    let back = RwSignal::new(String::new());
    let edited = Memo::new(move |_| {
        let id = editing.get()?;
        rows.with(|r| {
            r.iter()
                .find(|row| row.0 == id)
                .map(|(id, name, capacity, details, _)| {
                    (id.clone(), name.clone(), *capacity, details.clone())
                })
        })
    });
    let held = Memo::new(move |_| {
        editing
            .get()
            .and_then(|id| rows.with(|r| r.iter().find(|row| row.0 == id).map(|row| row.4)))
            .unwrap_or(0)
    });
    let close = Callback::new(move |()| editing.set(None));
    view! {
        <section aria-labelledby="uhs-resources-title" class="card">
            <h2 id="uhs-resources-title">{text::UHS_RESOURCES_TITLE}</h2>
            {move || {
                let rows = rows.get();
                crate::shell::views::keep_focus();
                if rows.is_empty() {
                    return view! { <p>{text::UHS_EMPTY}</p> }.into_any();
                }
                let editable = editable.get();
                view! {
                    <ul class="unit-list">
                        {rows
                            .into_iter()
                            .map(|(id, name, capacity, details, _)| {
                                let staff = text::uhs_staff_count(details.staff.len());
                                let button = format!("uhs-details-{}-button", core::slug(&id));
                                view! {
                                    <li class="unit-card uhs-card" data-uhs=id.clone()>
                                        <div class="card-head">
                                        <h3 class="uhs-card__name">{name.clone()}</h3>
                                        {editable.then({
                                            let id = id.clone();
                                            move || view! {
                                            <crate::shell::ui::EditButton id=button.clone() icon="pencil" word=text::UHS_DETAILS
                                                label=text::uhs_details_label(&name)
                                                on_click=Callback::new(move |()| {
                                                    draft.fill(&name, capacity, &details);
                                                    back.set(button.clone());
                                                    editing.set(Some(id.clone()));
                                                }) />
                                        }})}
                                        </div>
                                        <p class="uhs-card__staff">{staff}</p>
                                    </li>
                                }
                            })
                            .collect_view()}
                    </ul>
                }
                .into_any()
            }}
            <crate::shell::ui::EditDialog id="uhs-details" open=Signal::derive(move || edited.with(Option::is_some))
                on_close=close back
                title=Signal::derive(move || edited.with(|e| e.as_ref().map(|e| text::uhs_edit_title(&e.1)).unwrap_or_default()))>
                {move || edited.get().map(|(id, name, capacity, details)| {
                    crate::shell::views::keep_focus();
                    view! { <UhsDetailsPanel core revision id name capacity details draft patients=held on_done=close /> }
                })}
            </crate::shell::ui::EditDialog>
            <Show when=move || editable.get()>
                <NewUhsForm core revision />
            </Show>
        </section>
    }
}

/// Sets up a UHS with its name and capacity, which also gives it its places (UHS-1, UHS-7).
#[component]
fn NewUhsForm(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
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
    view! {
        <h3>{text::UHS_NEW}</h3>
        <form on:submit=submit class="form form--inline new-uhs">
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
    }
}

/// One on-site care unit's patients per triage category, their total against its
/// capacity, and how many were discharged; `children` add more lines (UHS-8).
#[component]
pub fn CountCard(
    name: String,
    capacity: u32,
    counts: ordo_contracts::PatientCountsV1,
    /// Capacity per triage category, shown next to the count (UHS-13).
    #[prop(optional)]
    capacities: std::collections::BTreeMap<TriageCategory, u32>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let c = counts;
    let total: u32 = CATEGORIES.iter().map(|k| c.get(*k)).sum();
    view! {
        <li class="count-card">
            <h3>{name}</h3>
            <ul class="count-card__counts">
                {CATEGORIES
                    .into_iter()
                    .map(|k| {
                        // [impl->dsn~uhs-capacity-by-category~2]
                        let of = capacities.get(&k).map(|n| format!(" / {n}"));
                        view! { <li><TriageBadge category=k /> <span class="count-card__n">{c.get(k)}{of}</span></li> }
                    })
                    .collect_view()}
            </ul>
            <p>
                {text::COL_TOTAL} ": " <strong>{total}</strong> " · " {text::COL_LOAD} ": " {text::load(total, capacity)}
                " · " {text::COL_DISCHARGED} ": " {c.discharged}
            </p>
            {children.map(|c| c())}
        </li>
    }
}

#[component]
pub(crate) fn UhsCounts(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        let list = uhs_list(core);
        let counts = core.with_value(|c| {
            let events = c.events("incident");
            med::latest_counts(&events)
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
    });
    view! {
        <section aria-labelledby="counts-title" class="card">
            <h2 id="counts-title">{text::COUNTS_TITLE}</h2>
            {move || {
                let rows = rows.get();
                if rows.is_empty() {
                    return view! { <p>{text::COUNTS_EMPTY}</p> }.into_any();
                }
                // Narrow screens: one block per UHS instead of a wide table (reflow).
                let list = rows
                    .iter()
                    .map(|(name, capacity, c)| view! { <CountCard name=name.clone() capacity=*capacity counts=c.clone() /> })
                    .collect_view();
                view! {
                    <ul class="count-list only-narrow">{list}</ul>
                    <table class="table table--counts only-wide">
                        <thead>
                            <tr>
                                <th scope="col">{text::COL_UHS}</th>
                                {CATEGORIES.into_iter().map(|c| view! { <th scope="col" class="number"><TriageBadge category=c /></th> }).collect_view()}
                                <th scope="col" class="number">{text::COL_TOTAL}</th>
                                <th scope="col" class="number">{text::COL_LOAD}</th>
                                <th scope="col" class="number">{text::COL_DISCHARGED}</th>
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
                                            <td class="number">{c.discharged}</td>
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
