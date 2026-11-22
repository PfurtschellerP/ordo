//! What an object on the situation map is (MAP-11): the card that pointing at it, a tap or
//! the list opens, with details and the actions the person may take. The card's links
//! lead to the object's place in another view, which shows it once it has opened.

use std::cell::RefCell;
use std::collections::BTreeMap;

use leptos::prelude::*;
use ordo_map::{ObjectKind, SituationMap};
use ordo_tasks::Phase;
use ordo_units::{UnitCatalogue, UnitRegister};

use crate::core::{self, Handle};
use crate::shared::text;
use crate::shell::nav::Route;
use crate::shell::ui::Icon;

thread_local! {
    /// What the next view brings into view once it has opened (a CSS selector).
    static REVEAL: RefCell<Option<String>> = const { RefCell::new(None) };
    /// The unit whose "Auftrag geben" form *Disposition* opens next.
    static GIVE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Brings `selector` into view in the view that opens next.
pub fn reveal_next(selector: String) {
    REVEAL.with_borrow_mut(|r| *r = Some(selector));
}

/// Opens the form for a new Auftrag for `unit` in the next *Disposition*.
pub fn give_next(unit: String) {
    GIVE.with_borrow_mut(|g| *g = Some(unit));
}

/// The unit chosen for a new Auftrag elsewhere, taken once.
pub fn take_give() -> Option<String> {
    GIVE.with_borrow_mut(Option::take)
}

/// Brings what [`reveal_next`] named into view, once the view has rendered, and moves the
/// focus there so keyboard users continue at it.
pub fn reveal_pending() {
    let Some(selector) = REVEAL.with_borrow_mut(Option::take) else {
        return;
    };
    crate::shell::views::after_render(move || {
        let Some(element) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.query_selector(&selector).ok().flatten())
        else {
            return;
        };
        let options = web_sys::ScrollIntoViewOptions::new();
        options.set_block(web_sys::ScrollLogicalPosition::Center);
        element.scroll_into_view_with_scroll_into_view_options(&options);
        if !element.has_attribute("tabindex") {
            let _ = element.set_attribute("tabindex", "-1");
        }
        if let Ok(e) = wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlElement>(element) {
            let _ = e.focus();
        }
    });
}

/// A CSS attribute value in double quotes.
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

/// The open Aufträge a pin stands for: a linked group, or those naming an old report.
fn pinned(
    tasks: &ordo_tasks::TaskList,
    kind: ObjectKind,
    reference: &str,
) -> Vec<ordo_tasks::Task> {
    let mut list: Vec<ordo_tasks::Task> = tasks
        .tasks
        .values()
        .filter(|t| {
            t.status.is_open()
                && match kind {
                    ObjectKind::Report => t.given.report.as_deref() == Some(reference),
                    _ => t.given.report.is_none() && t.pin_task() == reference,
                }
        })
        .cloned()
        .collect();
    list.sort_by_key(ordo_tasks::Task::given_at);
    list
}

/// "Zugewiesen · SanTrp 1"
fn shown_state(core: Handle, task: &ordo_tasks::Task) -> String {
    let who = crate::shared::tasks::assignee(core, task, &crate::shared::command::structure(core));
    let phase = text::task_phase(task.phase());
    if who.is_empty() {
        phase.to_owned()
    } else {
        format!("{phase} · {who}")
    }
}

// [impl->dsn~map-object-cards~4]
/// The card of one object (MAP-11): its details by kind and the actions this person may
/// take. `on_close` closes it; the close button names the object.
#[component]
pub fn ObjectCard(
    core: Handle,
    revision: RwSignal<u64>,
    object: String,
    on_close: Callback<()>,
) -> impl IntoView {
    let editable = core::permitted(core, revision, core::permission::EDIT_UNITS);
    let id = format!("card-{}", core::slug(&object));
    let title = format!("{id}-title");
    let labelled = title.clone();
    let object = StoredValue::new(object);
    let body = move || {
        revision.track();
        let map = core.with_value(|c| SituationMap::from_events(&c.events(ordo_map::COMPARTMENT)));
        let Some(o) = map.objects.get(&object.get_value()).cloned() else {
            return view! { <p>{text::map_object_gone()}</p> }.into_any();
        };
        let reference = o.placed.reference.clone().unwrap_or_default();
        match o.placed.kind {
            ObjectKind::Task | ObjectKind::Report => view! {
                <TaskPart core kind=o.placed.kind reference editable=editable.get() />
            }
            .into_any(),
            ObjectKind::Unit => {
                view! { <UnitPart core revision unit=reference editable=editable.get() /> }
                    .into_any()
            }
            ObjectKind::Uhs => {
                view! { <UhsPart core uhs=reference editable=editable.get() /> }.into_any()
            }
            kind => view! { <p>{text::map_kind(kind)}</p> }.into_any(),
        }
    };
    let label = move || {
        core.with_value(|c| SituationMap::from_events(&c.events(ordo_map::COMPARTMENT)))
            .objects
            .get(&object.get_value())
            .map(|o| o.placed.label.clone())
            .unwrap_or_default()
    };
    view! {
        <section id=id class="object-card" aria-labelledby=labelled data-object=object.get_value()
            on:keydown=move |ev| if ev.key() == "Escape" { on_close.run(()) }>
            <div class="object-card__head">
                <h3 id=title>{label}</h3>
                <button type="button" class="button button--small button--icon object-card__close"
                    aria-label=move || text::close_label(label()) title=text::section_close()
                    on:click=move |_| on_close.run(())>
                    <Icon name="x" />
                </button>
            </div>
            {body}
        </section>
    }
}

/// An Auftrag's pin: each open Auftrag with its state and unit, and the way to it.
#[component]
fn TaskPart(core: Handle, kind: ObjectKind, reference: String, editable: bool) -> impl IntoView {
    let list = pinned(&crate::shared::tasks::task_list(core), kind, &reference);
    if list.is_empty() {
        return view! { <p>{text::tasks_empty()}</p> }.into_any();
    }
    view! {
        <ul class="object-card__list">
            {list.into_iter().map(|t| {
                let received = t.phase() == Phase::Received;
                let selector = format!(".task-card[data-task={}]", quoted(&t.given.task));
                let route = Route::view("command.lead", "dispatch");
                let follow = crate::shell::views::follow_route(route.clone());
                let open = move |ev| {
                    reveal_next(selector.clone());
                    follow(ev);
                };
                view! {
                    <li>
                        <p>{t.given.text.clone()}</p>
                        <p><strong>{shown_state(core, &t)}</strong></p>
                        {editable.then(|| view! {
                            <p><a class="button button--small" href=route.to_path() on:click=open>
                                {if received { text::card_assign() } else { text::card_open_task() }}
                            </a></p>
                        })}
                    </li>
                }
            }).collect_view()}
        </ul>
    }
    .into_any()
}

/// A unit: status, its Auftrag, strength and the crew's qualifications in short, the names
/// folded away; editing, status and a new Auftrag for those who may.
#[component]
fn UnitPart(core: Handle, revision: RwSignal<u64>, unit: String, editable: bool) -> impl IntoView {
    let register =
        core.with_value(|c| UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)));
    let Some(u) = register.units.get(&unit).cloned() else {
        return view! { <p>{text::map_kind(ObjectKind::Unit)}</p> }.into_any();
    };
    let tasks = crate::shared::tasks::task_list(core);
    let current = tasks
        .of_unit(&unit)
        .first()
        .map(|t| crate::shared::tasks::label(t));
    let detailed = crate::shared::units::detailed(core);
    let special = u.special.clone();
    // Each qualification once, with how many of the crew have it, in the profile's order.
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for m in &u.crew {
        for q in &m.qualifications {
            *counts.entry(q.as_str()).or_default() += 1;
        }
    }
    let catalogue = &crate::shared::units::CATALOGUE.qualifications;
    let chips: Vec<(String, String, usize)> = catalogue
        .iter()
        .filter_map(|q| {
            counts.get(q.id.as_str()).map(|n| {
                (
                    UnitCatalogue::short(catalogue, &q.id).to_owned(),
                    q.label.clone(),
                    *n,
                )
            })
        })
        .collect();
    let names: Vec<String> = u
        .crew
        .iter()
        .map(|m| crate::shell::profile::display_text(core, m.account.as_deref().unwrap_or(&m.name)))
        .collect();
    let status = crate::shared::units::status_shown(&u);
    let code = u
        .status
        .as_ref()
        .map(|s| s.status.clone())
        .unwrap_or_default();
    let edit_selector = format!(".unit-card[data-unit={}]", quoted(&unit));
    let edit_route = Route::view("command.prepare", "units");
    let follow_edit = crate::shell::views::follow_route(edit_route.clone());
    let dispatch = Route::view("command.lead", "dispatch");
    let follow_give = crate::shell::views::follow_route(dispatch.clone());
    let give_unit = unit.clone();
    view! {
        <p class="unit-card__details">{crate::shared::units::details(&u)}</p>
        <p>{text::col_status()} ": " <strong>{status}</strong></p>
        {current.map(|c| view! { <p>{text::card_current_task()} ": " {c}</p> })}
        {u.strength.map(|s| view! { <p>{text::strength()} ": " <crate::shared::units::StrengthText strength=s special detailed /></p> })}
        {(!chips.is_empty()).then(|| view! {
            <ul class="object-card__chips" aria-label=text::qualifications_short()>
                {chips.into_iter().map(|(short, label, n)| view! {
                    <li class="tag">
                        <abbr title=label.clone() aria-hidden="true">{short} " " {n}</abbr>
                        <span class="visually-hidden">{text::qualification_count(&label, n)}</span>
                    </li>
                }).collect_view()}
            </ul>
        })}
        {(!names.is_empty()).then(|| view! {
            <details class="object-card__names">
                <summary>{text::crew_names(names.len())}</summary>
                <p>{names.join(", ")}</p>
            </details>
        })}
        {editable.then(|| view! {
            <crate::shared::units::StatusSelect core revision unit=unit.clone() current=code />
            <div class="row">
                <a class="button button--small" href=edit_route.to_path()
                    aria-label=text::card_edit_label(&unit) title=text::card_edit()
                    on:click=move |ev| { reveal_next(edit_selector.clone()); follow_edit(ev) }>
                    <Icon name="pencil" /><span class="object-card__word">{text::card_edit()}</span>
                </a>
                <a class="button button--small" href=dispatch.to_path()
                    on:click=move |ev| { give_next(give_unit.clone()); follow_give(ev) }>
                    {text::task_give()}
                </a>
            </div>
        })}
    }
    .into_any()
}

/// A UHS: its patients per triage category, in words and numbers, the way there for those
/// who work with patients, and to its place and staff for the incident command.
#[component]
fn UhsPart(core: Handle, uhs: String, editable: bool) -> impl IntoView {
    let counts = core
        .with_value(|c| {
            ordo_med_patients::latest_counts(&c.events(ordo_med_patients::UHS_COMPARTMENT))
        })
        .remove(&uhs)
        .unwrap_or_default();
    let clinical = core::granted(core, ordo_med_patients::COMPARTMENT);
    let route = Route::view("med.uhs", "patients");
    let follow = crate::shell::views::follow_route(route.clone());
    let edit_selector = format!(".uhs-card[data-uhs={}]", quoted(&uhs));
    let edit_route = Route::view("command.prepare", "units");
    let follow_edit = crate::shell::views::follow_route(edit_route.clone());
    let name = core::uhs_name(core, &uhs);
    view! {
        <ul class="object-card__chips" aria-label=text::card_patients()>
            {crate::med::triage::CATEGORIES.into_iter().map(|k| view! {
                <li><crate::med::triage::TriageBadge category=k /> " " {counts.get(k)}</li>
            }).collect_view()}
        </ul>
        <div class="row">
            {editable.then(|| view! {
                <a class="button button--small" href=edit_route.to_path()
                    aria-label=text::card_edit_label(&name) title=text::card_edit()
                    on:click=move |ev| { reveal_next(edit_selector.clone()); follow_edit(ev) }>
                    <Icon name="pencil" /><span class="object-card__word">{text::card_edit()}</span>
                </a>
            })}
            {clinical.then(|| view! {
                <a class="button button--small" href=route.to_path()
                    on:click=move |ev| { crate::med::patients::work_at(&uhs); follow(ev) }>{text::card_open_uhs()}</a>
            })}
        </div>
    }
}
