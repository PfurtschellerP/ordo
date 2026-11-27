//! *Führung – Lage* (ADR-0099): the units at a glance beside the map, grouped by what the
//! incident command needs to know (UNIT-10), and the numbers the overview's tile shows. The
//! groups come from the profile's status codes; the treatment places are the Med
//! package's tab. Editing stays in *Vorbereitung*.

use std::collections::BTreeMap;

use leptos::prelude::*;
use ordo_units::{StatusGroup, Unit, UnitRegister};

use crate::core::{self, Handle};
use crate::shared::text;
use crate::shared::units::{QualificationChips, STATUS, StrengthText};
use crate::shell::ui::{Ago, Icon, RadioChips};

// [impl->dsn~unit-overview~1]
/// The icon of a group (Lucide), always shown with its word; none for units without a
/// status. Shapes stay with the triage categories.
pub fn group_icon(group: StatusGroup) -> &'static str {
    match group {
        StatusGroup::Available => "circle-check",
        StatusGroup::Task => "clipboard-list",
        StatusGroup::Transport => "ambulance",
        StatusGroup::Unavailable => "ban",
    }
}

/// The word of a group, or that a unit has no status yet.
pub fn group_word(group: Option<StatusGroup>) -> &'static str {
    match group {
        Some(StatusGroup::Available) => text::group_available(),
        Some(StatusGroup::Task) => text::group_task(),
        Some(StatusGroup::Transport) => text::group_transport(),
        Some(StatusGroup::Unavailable) => text::group_unavailable(),
        None => text::group_none(),
    }
}

/// The key of a group for styles and tests.
fn group_key(group: Option<StatusGroup>) -> &'static str {
    match group {
        Some(StatusGroup::Available) => "available",
        Some(StatusGroup::Task) => "task",
        Some(StatusGroup::Transport) => "transport",
        Some(StatusGroup::Unavailable) => "unavailable",
        None => "none",
    }
}

// [impl->dsn~unit-overview~1]
/// A group: its icon and its word, never the icon alone (gallery: *Statusgruppe*).
#[component]
pub fn GroupLabel(group: Option<StatusGroup>) -> impl IntoView {
    view! {
        <span class="status-group" data-group=group_key(group)>
            {group.map(|g| view! { <Icon name=group_icon(g) /> })}
            <span class="status-group__word">{group_word(group)}</span>
        </span>
    }
}

/// The group of a unit, from its status and the profile.
pub fn group_of(u: &Unit) -> Option<StatusGroup> {
    u.status.as_ref().and_then(|s| STATUS.group(&s.status))
}

/// How many units are in each group, without status, and per type free and in all: what the
/// overview's tile shows (UNIT-10).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UnitCounts {
    /// Per group, in [`StatusGroup::ALL`]'s order.
    pub groups: [usize; 4],
    /// Units without a status yet.
    pub no_status: usize,
    /// Per type: the type, free units, all units; types in alphabetical order.
    pub types: Vec<(String, usize, usize)>,
    /// All units.
    pub total: usize,
}

impl UnitCounts {
    // [impl->dsn~unit-overview~1]
    /// Counts the units present.
    pub fn of<'a>(units: impl IntoIterator<Item = &'a Unit>) -> Self {
        let mut counts = Self::default();
        let mut types: BTreeMap<String, (usize, usize)> = BTreeMap::new();
        for u in units {
            let group = group_of(u);
            match group {
                Some(g) => {
                    let at = StatusGroup::ALL.iter().position(|x| *x == g).unwrap_or(0);
                    counts.groups[at] += 1;
                }
                None => counts.no_status += 1,
            }
            let t = types.entry(u.unit_type.trim().to_owned()).or_default();
            t.1 += 1;
            if group == Some(StatusGroup::Available) {
                t.0 += 1;
            }
            counts.total += 1;
        }
        counts.types = types
            .into_iter()
            .filter(|(t, _)| !t.is_empty())
            .map(|(t, (free, all))| (t, free, all))
            .collect();
        counts
    }
}

/// The units present now.
pub fn present(core: Handle) -> Vec<Unit> {
    core.with_value(|c| UnitRegister::from_events(&c.events(ordo_units::COMPARTMENT)))
        .present()
        .cloned()
        .collect()
}

/// One unit as the list shows it.
#[derive(Clone, Debug, PartialEq)]
struct UnitEntry {
    unit: Unit,
    group: Option<StatusGroup>,
    /// Its transport or Auftrag in words, without patient data.
    doing: Option<String>,
    chips: Vec<(String, String, usize)>,
    placed: bool,
}

/// One UHS as the list shows it.
#[derive(Clone, Debug, PartialEq)]
struct UhsEntry {
    id: String,
    name: String,
    places: String,
    staff: usize,
    placed: bool,
}

fn entries(core: Handle) -> (Vec<UnitEntry>, Vec<UhsEntry>) {
    let placed = crate::shared::situation::placed_objects(core);
    let tasks = crate::shared::tasks::task_list(core);
    let transports = crate::med::transports::unit_transports(core);
    let mut units: Vec<UnitEntry> = present(core)
        .into_iter()
        .map(|u| {
            let doing = transports.get(&u.unit).cloned().or_else(|| {
                tasks.of_unit(&u.unit).first().map(|t| {
                    format!(
                        "{}: {}",
                        text::card_current_task(),
                        crate::shared::tasks::label(t)
                    )
                })
            });
            let chips = crate::shared::units::qualification_counts(
                u.crew
                    .iter()
                    .flat_map(|m| m.qualifications.iter().map(String::as_str)),
            );
            UnitEntry {
                group: group_of(&u),
                placed: placed.contains(&ordo_map::ObjectKind::Unit.object_id(&u.unit)),
                doing,
                chips,
                unit: u,
            }
        })
        .collect();
    units.sort_by(|a, b| a.unit.unit.cmp(&b.unit.unit));
    let uhs = crate::med::places::uhs_glance(core)
        .into_iter()
        .map(|(id, name, places, staff)| UhsEntry {
            placed: placed.contains(&ordo_map::ObjectKind::Uhs.object_id(&id)),
            id,
            name,
            places,
            staff,
        })
        .collect();
    (units, uhs)
}

/// The type filter's choice for all units, and for the UHS.
const ALL: &str = "all";
const UHS: &str = "uhs";

/// Whether a unit passes the filters.
fn shown(e: &UnitEntry, kind: &str, only_available: bool) -> bool {
    (kind == ALL || kind.strip_prefix("t:") == Some(e.unit.unit_type.trim()))
        && (!only_available || e.group == Some(StatusGroup::Available))
}

// [impl->dsn~unit-overview~1]
// [impl->dsn~command-lage~1]
/// The tab *Einsatzmittel* of *Lage*: every unit and UHS, by group, with filters by type.
#[component]
pub fn UnitsAtGlance(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let list = Memo::new(move |_| {
        revision.track();
        entries(core)
    });
    let editable = core::permitted(core, revision, core::permission::EDIT_UNITS);
    let kind = RwSignal::new(ALL.to_owned());
    let only_available = RwSignal::new(false);
    // The unit whose details are open.
    let open = RwSignal::new(None::<String>);
    let filters = move || {
        let (units, uhs) = list.get();
        let counts = UnitCounts::of(units.iter().map(|e| &e.unit));
        let mut options = vec![(
            ALL.to_owned(),
            format!("{} {}", text::filter_all(), units.len()),
        )];
        options.extend(
            counts
                .types
                .iter()
                .map(|(t, _, all)| (format!("t:{t}"), format!("{t} {all}"))),
        );
        if !uhs.is_empty() {
            options.push((UHS.to_owned(), format!("UHS {}", uhs.len())));
        }
        view! { <RadioChips legend=text::units_filter() name="lage-units-type" options value=kind /> }
    };
    view! {
        <section class="card lage-units" aria-labelledby="lage-units-title">
            <h2 id="lage-units-title">{text::units_title()}</h2>
            <div class="lage-units__filters">
                {filters}
                <label class="choice">
                    <input id="lage-units-available" type="checkbox" bind:checked=only_available />
                    {text::units_only_available()}
                </label>
            </div>
            {move || {
                let (units, uhs) = list.get();
                let kind = kind.get();
                let only = only_available.get();
                let editable = editable.get();
                let mut groups: Vec<(Option<StatusGroup>, Vec<UnitEntry>)> = StatusGroup::ALL
                    .iter()
                    .map(|g| Some(*g))
                    .chain(std::iter::once(None))
                    .map(|g| {
                        let members = units
                            .iter()
                            .filter(|e| e.group == g && shown(e, &kind, only))
                            .cloned()
                            .collect();
                        (g, members)
                    })
                    .collect();
                groups.retain(|(_, m): &(_, Vec<UnitEntry>)| !m.is_empty());
                let uhs: Vec<UhsEntry> = if (kind == ALL || kind == UHS) && !only { uhs } else { Vec::new() };
                if groups.is_empty() && uhs.is_empty() {
                    return view! { <p class="hint">{text::units_glance_empty()}</p> }.into_any();
                }
                view! {
                    {groups.into_iter().map(|(group, members)| view! {
                        <UnitGroup core revision group members editable open />
                    }).collect_view()}
                    {(!uhs.is_empty()).then(|| view! { <UhsGroup list=uhs /> })}
                }
                .into_any()
            }}
        </section>
    }
}

/// One group: its label and number, then its units.
#[component]
fn UnitGroup(
    core: Handle,
    revision: RwSignal<u64>,
    group: Option<StatusGroup>,
    members: Vec<UnitEntry>,
    editable: bool,
    open: RwSignal<Option<String>>,
) -> impl IntoView {
    let title = format!("lage-group-{}", group_key(group));
    view! {
        <section class="unit-group" aria-labelledby=title.clone() data-group=group_key(group)>
            <h3 id=title.clone() class="unit-group__title">
                <GroupLabel group /> " " <span class="unit-group__n">{members.len()}</span>
            </h3>
            <ul class="unit-glance" data-keys="list">
                {members.into_iter().map(|e| view! { <UnitItem core revision entry=e editable open /> }).collect_view()}
            </ul>
        </section>
    }
}

/// One unit: name and kind, status since when, its transport or Auftrag, strength and
/// qualifications; its details with the actions of leading.
#[component]
fn UnitItem(
    core: Handle,
    revision: RwSignal<u64>,
    entry: UnitEntry,
    editable: bool,
    open: RwSignal<Option<String>>,
) -> impl IntoView {
    let UnitEntry {
        unit: u,
        doing,
        chips,
        placed,
        ..
    } = entry;
    let name = u.unit.clone();
    let is_open = Signal::derive({
        let name = name.clone();
        move || open.with(|o| o.as_deref() == Some(name.as_str()))
    });
    let toggle = {
        let name = name.clone();
        Callback::new(move |()| {
            open.update(|o| *o = (o.as_deref() != Some(name.as_str())).then(|| name.clone()))
        })
    };
    let since = u.status.as_ref().map(|s| s.since.wall_ms);
    let current = u
        .status
        .as_ref()
        .map(|s| s.status.clone())
        .unwrap_or_default();
    let detailed = crate::shared::units::detailed(core);
    let names: Vec<String> = u
        .crew
        .iter()
        .map(|m| crate::shell::profile::display_text(core, m.account.as_deref().unwrap_or(&m.name)))
        .collect();
    let dispatch = crate::shell::nav::Route::view("command.lead", "dispatch");
    let follow_give = crate::shell::views::follow_route(dispatch.clone());
    let details = {
        let name = name.clone();
        move || {
            is_open.get().then(|| {
                let (unit, give) = (name.clone(), name.clone());
                let (current, names) = (current.clone(), names.clone());
                let (dispatch, follow_give) = (dispatch.clone(), follow_give.clone());
                view! {
                    <div class="unit-glance__details">
                        {editable.then(|| view! {
                            <crate::shared::units::StatusSelect core revision unit=unit.clone() current />
                            <a class="button button--small" href=dispatch.to_path()
                                aria-label=text::task_give_label(&unit)
                                on:click=move |ev| {
                                    crate::shared::object_card::give_next(give.clone());
                                    follow_give(ev)
                                }>
                                {text::task_give()}
                            </a>
                        })}
                        {if names.is_empty() {
                            view! { <p class="hint">{text::crew_none()}</p> }.into_any()
                        } else {
                            view! { <p>{text::crew_names(names.len())} ": " {names.join(", ")}</p> }.into_any()
                        }}
                    </div>
                }
            })
        }
    };
    view! {
        <li class="unit-glance__item" data-unit=name.clone()>
            <div class="unit-glance__row">
                <span class="unit-glance__name">
                    <strong>{name.clone()}</strong>
                    " · " {crate::shared::units::details(&u)}
                </span>
                <div class="unit-glance__actions">
                    {placed.then(|| view! {
                        <crate::shared::situation::ShowOnMap object=ordo_map::ObjectKind::Unit.object_id(&name) name=name.clone() />
                    })}
                    <crate::shell::ui::IconButton icon="info" word=text::map_details()
                        label=text::map_details_label(&name) expanded=is_open on_click=toggle />
                </div>
            </div>
            <p class="unit-glance__status">
                {crate::shared::units::status_shown(&u)}
                {since.map(|at| view! { " " <Ago at /> })}
            </p>
            {doing.map(|d| view! { <p class="unit-glance__doing">{d}</p> })}
            {(u.strength.is_some() || !chips.is_empty()).then(|| view! {
                <div class="unit-glance__crew">
                    {u.strength.map(|s| view! {
                        <span>{text::strength()} " " <StrengthText strength=s special=u.special.clone() detailed /></span>
                    })}
                    <QualificationChips chips />
                </div>
            })}
            {details}
        </li>
    }
}

/// The UHS, which count as units (UHS-12): free places, staff, the way to the map.
#[component]
fn UhsGroup(list: Vec<UhsEntry>) -> impl IntoView {
    view! {
        <section class="unit-group" aria-labelledby="lage-group-uhs" data-group="uhs">
            <h3 id="lage-group-uhs" class="unit-group__title">
                <span class="status-group"><Icon name="tent" /><span class="status-group__word">{text::uhs_title()}</span></span>
                " " <span class="unit-group__n">{list.len()}</span>
            </h3>
            <ul class="unit-glance" data-keys="list">
                {list.into_iter().map(|u| view! {
                    <li class="unit-glance__item" data-uhs=u.id.clone()>
                        <div class="unit-glance__row">
                            <span class="unit-glance__name"><strong>{u.name.clone()}</strong></span>
                            <div class="unit-glance__actions">
                                {u.placed.then(|| view! {
                                    <crate::shared::situation::ShowOnMap object=ordo_map::ObjectKind::Uhs.object_id(&u.id) name=u.name.clone() />
                                })}
                            </div>
                        </div>
                        <p class="unit-glance__status">{u.places} " · " {crate::med::text::uhs_staff_count(u.staff)}</p>
                    </li>
                }).collect_view()}
            </ul>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit(name: &str, kind: &str, status: Option<&str>) -> Unit {
        let node = ordo_core::DeviceId(ordo_core::Ulid::from_parts(1, 1).expect("valid"));
        Unit {
            unit: name.into(),
            unit_type: kind.into(),
            organisation: "DRK".into(),
            service: None,
            strength: None,
            special: Default::default(),
            status: status.map(|code| ordo_units::UnitStatus {
                status: code.into(),
                since: ordo_core::Hlc {
                    wall_ms: 1,
                    counter: 0,
                    node,
                },
                by: "ben@ordo.example.org".parse().expect("valid user"),
            }),
            crew: Vec::new(),
            presence: Vec::new(),
        }
    }

    // [utest->dsn~unit-overview~1]
    #[test]
    fn units_count_by_group_and_type_as_the_profile_groups_their_status() {
        let units = [
            unit("RTW 1", "RTW", Some("2")),
            unit("RTW 2", "RTW", Some("7")),
            unit("RTW 3", "RTW", Some("4")),
            unit("KTW 1", "KTW", Some("6")),
            unit("SanTrp 1", "SanTrp", Some("1")),
            unit("SanTrp 2", "SanTrp", None),
        ];
        let counts = UnitCounts::of(&units);
        assert_eq!(counts.groups, [2, 1, 1, 1]);
        assert_eq!(counts.no_status, 1);
        assert_eq!(counts.total, 6);
        assert_eq!(
            counts.types,
            [
                ("KTW".to_owned(), 0, 1),
                ("RTW".to_owned(), 1, 3),
                ("SanTrp".to_owned(), 1, 2)
            ]
        );
    }
}
