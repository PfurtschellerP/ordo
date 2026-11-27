//! *Behandlungsplätze* (UHS-8, UHS-16, ADR-0086): how full the treatment places of every
//! on-site care unit are, per triage category the places are kept for. One strip per
//! category with a box per place (filled: taken, dashed: free), the free places in large
//! figures, and the patients waiting for a place. Shown to the incident command on
//! *Führung – Übersicht* and to the UHS module on its own view; both pass the numbers in,
//! so the component knows nothing about where they come from. Numbers only; nothing is
//! ranked.

use leptos::prelude::*;
use ordo_med_patients::overview::{Occupancy, PlaceGroup};

use crate::med::text;
use crate::med::triage::TriageBadge;

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
    /// What *Führung – Lage* shows more (UHS-16); none elsewhere.
    pub lage: Option<UhsMore>,
}

// [impl->dsn~command-treatment-places~2]
/// What the tab *Behandlungsplätze* of *Lage* shows more of a UHS (UHS-16, ADR-0099), from
/// the shared compartment only.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct UhsMore {
    /// Patients waiting for a transport: open, not yet on their way.
    pub waiting: usize,
    /// Of them, those without a unit yet.
    pub waiting_no_unit: usize,
    /// Its staff by qualification (short, label, number), in the profile's order.
    pub staff: Vec<(String, String, usize)>,
    /// Everyone on its staff list.
    pub staff_total: usize,
    /// Whom it asks for in open staff requests.
    pub requests: Vec<&'static str>,
    pub discharged: u32,
    /// Whether it is on the situation map.
    pub placed: bool,
}

/// Heading levels: the tile on *Führung* sits under a tile heading (h3), the UHS module's
/// view under its page heading (h2).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Level {
    H3,
    H4,
}

// [impl->dsn~command-treatment-places~2]
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
                    return view! { <p class="hint">{text::uhs_empty()}</p> }.into_any();
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
                    <p class="hint places__legend">{text::places_legend()}</p>
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
        None => view! { <span class="triage places__any">{text::places_any()}</span> }.into_any(),
    }
}

// [impl->dsn~command-treatment-places~2]
/// How many of all are free, "2 / 4 frei"; a full category adds "voll" (hatched and
/// outlined, never colour alone).
#[component]
fn FreeOrFull(group: PlaceGroup) -> impl IntoView {
    view! {
        <span class="places__free-or-full">
            <span class="places__free">
                <strong>{group.free()}</strong> " / " {group.total} " " {text::place_free()}
            </span>
            {(group.free() == 0).then(|| view! { <span class="places__full">{text::places_full()}</span> })}
        </span>
    }
}

/// The name of a group for its text alternative.
fn group_name(group: &PlaceGroup) -> &'static str {
    group.category.map_or(text::places_any(), text::label)
}

/// All units added up: the free places in large figures and a bar per category.
#[component]
fn AllUnits(occupancy: Occupancy, level: Level) -> impl IntoView {
    let title = match level {
        Level::H3 => {
            view! { <h3 id="places-all-title" class="places__name">{text::places_all()}</h3> }
                .into_any()
        }
        Level::H4 => {
            view! { <h4 id="places-all-title" class="places__name">{text::places_all()}</h4> }
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
                        g.category.map_or("any", crate::med::triage::css_key)
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
        None => view! { <p class="hint places__strips">{text::places_none()}</p> }.into_any(),
    };
    let lage = row
        .lage
        .map(|more| view! { <UhsMoreView id=row.id.clone() name=row.name.clone() more /> });
    view! {
        <li class="places__unit" data-uhs=row.id>
            <div class="places__head">
                {name}
                {head}
                {row.more.map(|m| view! { <p class="hint places__more">{m}</p> })}
            </div>
            {strips}
            {lage}
        </li>
    }
}

// [impl->dsn~command-treatment-places~2]
/// The more of a UHS on *Lage*: open staff requests, those waiting for a transport, its
/// staff by qualification, its discharged patients, and the ways on.
#[component]
fn UhsMoreView(id: String, name: String, more: UhsMore) -> impl IntoView {
    let transports = crate::shell::nav::Route::view("command.lead", "transports");
    let dispatch = crate::shell::nav::Route::view("command.lead", "dispatch");
    let requests = (!more.requests.is_empty()).then(|| {
        view! {
            <p class="places__requests">
                {more.requests.iter().map(|need| view! {
                    <span class="tag tag--warning">{text::staff_requested_tag(*need)}</span>
                }).collect_view()}
            </p>
        }
    });
    let line = [
        (more.waiting > 0).then(|| text::waiting_transport(more.waiting, more.waiting_no_unit)),
        Some(crate::shared::text::discharged_count(more.discharged)),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" · ");
    view! {
        <div class="places__lage">
            {requests}
            <p class="places__more">{line}</p>
            <div class="places__staff">
                <span>{text::uhs_staff_count(more.staff_total)}</span>
                <crate::shared::units::QualificationChips chips=more.staff />
            </div>
            <div class="row places__actions">
                {more.placed.then(|| view! {
                    <crate::shared::situation::ShowOnMap object=ordo_map::ObjectKind::Uhs.object_id(&id) name=name.clone() />
                })}
                <a class="button button--small" href=transports.to_path()
                    on:click=crate::shell::views::follow_route(transports.clone())>{crate::shared::text::transports()}</a>
                {(!more.requests.is_empty()).then(|| view! {
                    <a class="button button--small" href=dispatch.to_path()
                        aria-label=text::staff_go_send_label(&name)
                        on:click=crate::shell::views::follow_route(dispatch.clone())>{text::staff_go_send()}</a>
                })}
            </div>
        </div>
    }
}

// [impl->dsn~command-treatment-places~2]
/// The UHS as the command sees them, from the shared compartment only: places, counts, the
/// open transports from each, and, for *Lage*, the rest of [`UhsMore`].
pub fn place_rows(core: crate::core::Handle, lage: bool) -> Vec<PlaceRow> {
    use ordo_med_patients as med;
    use ordo_med_patients::transport::{TransportBoard, TransportStatus};
    let (counts, mut occupancies, details, requests, board) = core.with_value(|c| {
        let incident = c.events(med::UHS_COMPARTMENT);
        (
            med::latest_counts(&incident),
            med::overview::occupancies(&incident),
            med::uhs_details(&incident),
            med::staffing::StaffRequests::from_events(&c.events(med::staffing::COMPARTMENT)),
            TransportBoard::from_events(&c.events(med::transport::COMPARTMENT)),
        )
    });
    let placed = if lage {
        crate::shared::situation::placed_objects(core)
    } else {
        Default::default()
    };
    let helpers = lage
        .then(|| crate::shared::helpers::register(core))
        .flatten();
    crate::med::uhs::uhs_list(core)
        .into_iter()
        .map(|u| {
            let c = counts.get(&u.id).cloned().unwrap_or_default();
            let from_here: Vec<_> = board
                .transports
                .values()
                .filter(|t| t.status.is_open() && t.request.uhs == u.id)
                .collect();
            let more = if lage {
                None
            } else {
                let mut more = crate::shared::text::transports_open(from_here.len());
                if c.discharged > 0 {
                    more = format!(
                        "{more} · {}",
                        crate::shared::text::discharged_count(c.discharged)
                    );
                }
                Some(more)
            };
            let extra = lage.then(|| {
                let waiting: Vec<_> = from_here
                    .iter()
                    .filter(|t| t.status <= TransportStatus::Loading && t.request.planned.is_none())
                    .collect();
                let staff = details
                    .get(&u.id)
                    .map(|d| d.staff.clone())
                    .unwrap_or_default();
                let qualifications: Vec<String> = staff
                    .iter()
                    .filter_map(|entry| helpers.as_ref()?.of_account(entry))
                    .flat_map(|h| h.fields.qualifications.clone().unwrap_or_default())
                    .collect();
                UhsMore {
                    waiting: waiting.len(),
                    waiting_no_unit: waiting
                        .iter()
                        .filter(|t| t.status == TransportStatus::Requested)
                        .count(),
                    staff: crate::shared::units::qualification_counts(
                        qualifications.iter().map(String::as_str),
                    ),
                    staff_total: staff.len(),
                    requests: requests
                        .open()
                        .into_iter()
                        .filter(|r| r.requested.uhs == u.id)
                        .map(|r| text::staff_need(r.requested.need))
                        .collect(),
                    discharged: c.discharged,
                    placed: placed.contains(&ordo_map::ObjectKind::Uhs.object_id(&u.id)),
                }
            });
            PlaceRow {
                occupancy: occupancies.remove(&u.id),
                load: (
                    crate::med::triage::CATEGORIES
                        .iter()
                        .map(|k| c.get(*k))
                        .sum(),
                    u.capacity,
                ),
                more,
                lage: extra,
                id: u.id,
                name: u.name,
            }
        })
        .collect()
}

// [impl->dsn~unit-overview~1]
/// The UHS for the tab *Einsatzmittel* of *Lage*: ID, name, free places of all (or the
/// load where it has no places), and how many are on its staff list.
pub fn uhs_glance(core: crate::core::Handle) -> Vec<(String, String, String, usize)> {
    place_rows(core, true)
        .into_iter()
        .map(|r| {
            let places = match &r.occupancy {
                Some(o) => text::places_free(o.free(), o.total()),
                None => text::uhs_load(r.load.0, r.load.1),
            };
            let staff = r.lage.map_or(0, |m| m.staff_total);
            (r.id, r.name, places, staff)
        })
        .collect()
}

// [impl->dsn~command-treatment-places~2]
// [impl->dsn~command-lage~1]
/// The tab *Behandlungsplätze* of *Führung – Lage* (ADR-0099): the free places of all UHS,
/// those waiting for a place or a transport and those registered, then one card per UHS.
/// It replaces the table *Patienten je UHS*.
#[component]
pub fn PlacesAtGlance(core: crate::core::Handle, revision: RwSignal<u64>) -> impl IntoView {
    let rows = Memo::new(move |_| {
        revision.track();
        place_rows(core, true)
    });
    let totals = move || {
        rows.with(|rows| {
            let waiting: usize = rows
                .iter()
                .filter_map(|r| r.lage.as_ref())
                .map(|m| m.waiting)
                .sum();
            let here: u32 = rows.iter().map(|r| r.load.0).sum();
            let discharged: u32 = rows
                .iter()
                .filter_map(|r| r.lage.as_ref())
                .map(|m| m.discharged)
                .sum();
            let without: u32 = rows
                .iter()
                .filter_map(|r| r.occupancy.as_ref())
                .map(|o| o.without_place)
                .sum();
            (!rows.is_empty()).then(|| view! {
                <ul class="places__totals">
                    <li>{text::without_place(without)}</li>
                    <li>{text::waiting_transport_total(waiting)}</li>
                    <li>{text::registered_here_discharged(here + discharged, here, discharged)}</li>
                </ul>
            })
        })
    };
    view! {
        <section class="card lage-places" aria-labelledby="lage-places-title">
            <h2 id="lage-places-title">{crate::shared::text::places()}</h2>
            {totals}
            <TreatmentPlaces rows=Signal::from(rows) level=Level::H3 />
        </section>
    }
}

/// One category's places: its badge, a box per place, and how many are free.
#[component]
fn Strip(group: PlaceGroup) -> impl IntoView {
    let kind = group.category.map_or("any", crate::med::triage::css_key);
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
