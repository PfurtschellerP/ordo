//! *Behandlungsplätze* (UHS-8, UHS-16, ADR-0086): how full the treatment places of every
//! on-site care unit are, per triage category the places are kept for. One strip per
//! category with a box per place (filled: taken, dashed: free), the free places in large
//! figures, and the patients waiting for a place. Shown to the incident command on
//! *Führung – Übersicht* and to the UHS module on its own view; both pass the numbers in,
//! so the component knows nothing about where they come from. Numbers only; nothing is
//! ranked.

use leptos::prelude::*;
use ordo_med_patients::overview::{Occupancy, PlaceGroup};

use crate::text;
use crate::triage::TriageBadge;

/// One on-site care unit as the overview shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaceRow {
    /// Its ID, for tests and links.
    pub id: String,
    pub name: String,
    /// Its places per category; none: it has no places.
    pub occupancy: Option<Occupancy>,
    /// Patients in it and its capacity, shown where it has no places.
    pub load: (u32, u32),
    /// A further line, e.g. its open orders and transports.
    pub more: Option<String>,
}

/// Heading levels: the tile on *Führung* sits under a tile heading (h3), the UHS module's
/// view under its page heading (h2).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    H3,
    H4,
}

// [impl->dsn~command-treatment-places~1]
// [impl->dsn~uhs-overview~3]
/// All units added up first (when there are several), then one row per unit.
#[component]
pub fn TreatmentPlaces(
    #[prop(into)] rows: Signal<Vec<PlaceRow>>,
    /// The heading level of a unit's name.
    level: Level,
) -> impl IntoView {
    view! {
        <div class="places">
            {move || {
                let rows = rows.get();
                if rows.is_empty() {
                    return view! { <p class="hint">{text::UHS_EMPTY}</p> }.into_any();
                }
                let all = (rows.len() > 1).then(|| {
                    let mut sum = Occupancy::default();
                    let mut any = false;
                    for o in rows.iter().filter_map(|r| r.occupancy.as_ref()) {
                        sum.add(o);
                        any = true;
                    }
                    any.then_some(sum)
                }).flatten();
                view! {
                    {all.map(|o| view! { <AllUnits occupancy=o level /> })}
                    <ul class="places__list">
                        {rows.into_iter().map(|row| view! { <UnitRow row level /> }).collect_view()}
                    </ul>
                    <p class="hint places__legend">{text::PLACES_LEGEND}</p>
                }
                .into_any()
            }}
        </div>
    }
}

/// The badge of a group: the triage category's, or a neutral one for places kept for any.
#[component]
fn GroupBadge(group: PlaceGroup) -> impl IntoView {
    match group.category {
        Some(category) => view! { <TriageBadge category /> }.into_any(),
        None => view! { <span class="triage places__any">{text::PLACES_ANY}</span> }.into_any(),
    }
}

/// How many are free, or "voll" (hatched and outlined, never colour alone).
#[component]
fn FreeOrFull(group: PlaceGroup) -> impl IntoView {
    if group.free() == 0 {
        view! { <span class="places__full">{text::PLACES_FULL}</span> }.into_any()
    } else {
        view! {
            <span class="places__free"><strong>{group.free()}</strong> " " {text::PLACE_FREE}</span>
        }
        .into_any()
    }
}

/// The name of a group for its text alternative.
fn group_name(group: &PlaceGroup) -> &'static str {
    group.category.map_or(text::PLACES_ANY, text::label)
}

/// All units added up: the free places in large figures and a bar per category.
#[component]
fn AllUnits(occupancy: Occupancy, level: Level) -> impl IntoView {
    let title = match level {
        Level::H3 => {
            view! { <h3 id="places-all-title" class="places__name">{text::PLACES_ALL}</h3> }
                .into_any()
        }
        Level::H4 => {
            view! { <h4 id="places-all-title" class="places__name">{text::PLACES_ALL}</h4> }
                .into_any()
        }
    };
    view! {
        <section class="places__all" aria-labelledby="places-all-title">
            <div class="places__head">
                {title}
                <p class="places__figure">
                    <span class="places__number">{occupancy.free()}</span>
                    " " {text::free_of(occupancy.total())}
                </p>
                <p class="places__waiting">{text::without_place(occupancy.without_place)}</p>
            </div>
            <ul class="places__sums">
                {occupancy.groups.iter().map(|g| {
                    let share = (g.taken * 100).checked_div(g.total).unwrap_or(0).min(100);
                    let class = format!(
                        "places__bar-fill places__box--{}",
                        g.category.map_or("any", crate::triage::css_key)
                    );
                    view! {
                        <li class="places__sum" class:places__sum--full=g.free() == 0>
                            <GroupBadge group=*g />
                            <span class="places__sum-text">
                                <FreeOrFull group=*g />
                                <span class="visually-hidden">{format!(", {}", text::place_strip(group_name(g), g.taken, g.total))}</span>
                            </span>
                            <span class="places__bar" aria-hidden="true">
                                <span class=class style:inline-size=format!("{share}%")></span>
                            </span>
                        </li>
                    }
                }).collect_view()}
            </ul>
        </section>
    }
}

/// One unit: name and free places on the left, a strip per category on the right.
#[component]
fn UnitRow(row: PlaceRow, level: Level) -> impl IntoView {
    let name = match level {
        Level::H3 => view! { <h3 class="places__name">{row.name.clone()}</h3> }.into_any(),
        Level::H4 => view! { <h4 class="places__name">{row.name.clone()}</h4> }.into_any(),
    };
    let head = match &row.occupancy {
        Some(o) => view! {
            <p class="places__figure">
                <span class="places__number">{o.free()}</span> " " {text::free_of(o.total())}
            </p>
            {(o.without_place > 0).then(|| view! {
                <p class="places__waiting">{text::without_place(o.without_place)}</p>
            })}
        }
        .into_any(),
        None => view! {
            <p class="places__figure">{text::uhs_load(row.load.0, row.load.1)}</p>
        }
        .into_any(),
    };
    let strips = match row.occupancy {
        Some(o) => view! {
            <div class="places__strips">
                {o.groups.into_iter().map(|g| view! { <Strip group=g /> }).collect_view()}
            </div>
        }
        .into_any(),
        None => view! { <p class="hint places__strips">{text::PLACES_NONE}</p> }.into_any(),
    };
    view! {
        <li class="places__unit" data-uhs=row.id>
            <div class="places__head">
                {name}
                {head}
                {row.more.map(|m| view! { <p class="hint places__more">{m}</p> })}
            </div>
            {strips}
        </li>
    }
}

/// One category's places: its badge, a box per place, and how many are free.
#[component]
fn Strip(group: PlaceGroup) -> impl IntoView {
    let kind = group.category.map_or("any", crate::triage::css_key);
    let boxes = (0..group.total)
        .map(|i| {
            let class = if i < group.taken {
                format!("places__box places__box--taken places__box--{kind}")
            } else {
                "places__box places__box--free".to_owned()
            };
            view! { <span class=class></span> }
        })
        .collect_view();
    view! {
        <div class="places__strip" class:places__strip--full=group.free() == 0>
            <GroupBadge group />
            <span class="places__boxes" role="img"
                aria-label=text::place_strip(group_name(&group), group.taken, group.total)>
                {boxes}
            </span>
            <FreeOrFull group />
        </div>
    }
}
