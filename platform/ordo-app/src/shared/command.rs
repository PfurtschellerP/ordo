//! The command structure (CMD-1…4, ADR-0031): the view *Führung* with the tree of
//! positions, their leaders, units and UHS, and the roster below; forms to create
//! positions, hand over the lead and assign UHS.

use std::sync::LazyLock;

use leptos::prelude::*;
use ordo_command::{
    Assigned, CommandStructure, LeaderSet, Position, PositionClosed, PositionCreated,
    StructureCatalogue,
};

use crate::core::{self, Handle};
use crate::shared::situation::{MapAsk, MapAsks};
use crate::shared::text;
use crate::shell::ui::{Action, Actions, History, Suggestions};

/// Position kinds from the profile (`command/structure.toml`, checked at build time).
pub static STRUCTURE: LazyLock<StructureCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/structure.json")))
        .expect("command structure was checked at build time")
});

/// The incident's command structure, computed from the shared events.
pub fn structure(core: Handle) -> CommandStructure {
    core.with_value(|c| CommandStructure::from_events(&c.events(ordo_command::COMPARTMENT)))
}

/// "Einsatzabschnitt EA 1 Nord": kind and name of a position.
pub fn position_title(p: &Position) -> String {
    format!("{} {}", STRUCTURE.label(&p.created.kind), p.created.name)
}

/// The open positions as choices (ID, title), in tree order.
pub fn position_choices(s: &CommandStructure) -> Vec<(String, String)> {
    s.open_positions()
        .into_iter()
        .map(|p| (p.created.position.clone(), position_title(p)))
        .collect()
}

/// The map editor on the same page (*Führung & Raum*, ADR-0095), and what is on its map.
type OnMap = (MapAsks, std::collections::BTreeSet<String>);

/// One position and, nested, the positions reporting to it.
fn position_item(
    core: Handle,
    revision: RwSignal<u64>,
    s: &CommandStructure,
    id: &str,
    uhs_names: &std::collections::HashMap<String, String>,
    editable: bool,
    map: Option<&OnMap>,
) -> AnyView {
    let Some(p) = s.positions.get(id) else {
        return ().into_any();
    };
    let title = position_title(p);
    let leader = {
        let leader = p.leader().map(str::to_owned);
        crate::shell::profile::naming(move || {
            leader.as_deref().map_or_else(
                || text::no_leader().to_owned(),
                |l| text::leader(crate::shell::profile::display_text(core, l)),
            )
        })
    };
    let leaders = p.leaders.clone();
    let handovers = crate::shell::profile::naming(move || {
        leaders
            .iter()
            .map(|h| {
                (
                    core::clock(h.at.wall_ms),
                    text::handover(
                        h.from
                            .as_deref()
                            .map(|f| crate::shell::profile::display_text(core, f))
                            .as_deref(),
                        &crate::shell::profile::display_text(core, &h.to),
                    ),
                    crate::shell::profile::display_name(core, &h.by),
                )
            })
            .collect::<Vec<_>>()
    });
    // [impl->dsn~org-chart~2]
    // Units and UHS under a position are the tree's next level, below it (feedback of
    // 2026-11-14), each with a word for what it is.
    let members: Vec<AnyView> = p
        .units
        .iter()
        .map(|u| ("unit", u.clone(), text::tab_resources()))
        .chain(p.uhs.iter().map(|u| {
            (
                "uhs",
                uhs_names.get(u).cloned().unwrap_or_else(|| u.clone()),
                text::member_uhs(),
            )
        }))
        .map(|(kind, name, what)| {
            view! {
                <li class="position position--member" data-member=kind>
                    <div class="position__card position__card--member">
                        <p class="position__title"><strong>{name}</strong></p>
                        <p class="position__kind">{what}</p>
                    </div>
                </li>
            }
            .into_any()
        })
        .collect();
    let span = p
        .span_exceeded()
        .then(|| text::span_of_control(p.subordinates()));
    // [impl->dsn~command-and-space~2]
    // Put on the map, or found there, from the tree (CMD-5).
    let on_map = map.filter(|_| editable).map(|(asks, placed)| {
        let asks = *asks;
        let object = ordo_map::ObjectKind::Position.object_id(id);
        let position = id.to_owned();
        if placed.contains(&object) {
            Action {
                text: text::position_map_show(),
                label: text::position_map_show_label(&title),
                quiet: false,
                run: Callback::new(move |()| asks.ask(MapAsk::Show(object.clone()))),
            }
        } else {
            Action {
                text: text::position_map_set(),
                label: text::position_map_set_label(&title),
                quiet: false,
                run: Callback::new(move |()| asks.ask(MapAsk::Place(position.clone()))),
            }
        }
    });
    let close = editable.then(|| {
        let id = id.to_owned();
        on_map
            .into_iter()
            .chain([Action {
                text: text::position_close(),
                label: text::position_close_label(&title),
                quiet: true,
                run: Callback::new(move |()| {
                    let closed = PositionClosed {
                        position: id.clone(),
                    };
                    let _ = core::command_event(
                        core,
                        revision,
                        ordo_command::kinds::POSITION_CLOSED,
                        &closed.encode().unwrap_or_default(),
                    );
                }),
            }])
            .collect::<Vec<_>>()
    });
    let mut children: Vec<AnyView> = p
        .children
        .iter()
        .map(|child| position_item(core, revision, s, child, uhs_names, editable, map))
        .collect();
    children.extend(members);
    view! {
        <li class="position" data-position=id.to_owned()>
            <div class="position__card">
                <p class="position__title"><strong>{title.clone()}</strong></p>
                <p class="position__leader">{leader}</p>
                {span.map(|s| view! { <p class="notice position__span">{s}</p> })}
                {(!p.leaders.is_empty()).then(|| view! { <History entries=handovers /> })}
                {close.map(|actions| view! { <Actions actions /> })}
            </div>
            {(!children.is_empty()).then(|| view! { <ul class="position-tree">{children}</ul> })}
        </li>
    }
    .into_any()
}

// [impl->dsn~command-structure~1]
/// The view *Führung*: the command structure as a tree (CMD-3), the forms of the command,
/// and the roster below.
#[component]
pub fn LeadershipView(core: Handle, revision: RwSignal<u64>) -> impl IntoView {
    let editable = core::permitted(core, revision, core::permission::EDIT_UNITS);
    let tree = Memo::new(move |_| {
        revision.track();
        structure(core)
    });
    let uhs_names = Memo::new(move |_| {
        revision.track();
        crate::med::uhs::uhs_list(core)
            .into_iter()
            .map(|u| (u.id, u.name))
            .collect::<std::collections::HashMap<_, _>>()
    });
    // On Führung & Raum, the map editor beside the tree (ADR-0095).
    let asks = use_context::<MapAsks>();
    let placed = Memo::new(move |_| {
        revision.track();
        asks.map(|_| crate::shared::situation::placed_objects(core))
            .unwrap_or_default()
    });
    view! {
        <section class="card" aria-labelledby="structure-title">
            <h2 id="structure-title">{text::structure_title()}</h2>
            {move || {
                let s = tree.get();
                if s.roots.is_empty() {
                    return view! { <p>{text::structure_empty()}</p> }.into_any();
                }
                let names = uhs_names.get();
                let editable = editable.get();
                let map = asks.map(|a| (a, placed.get()));
                let items: Vec<AnyView> = s
                    .roots
                    .iter()
                    .map(|id| position_item(core, revision, &s, id, &names, editable, map.as_ref()))
                    .collect();
                // [impl->dsn~org-chart~2]
                // An org chart where the card is wide enough, else an indented tree; a wide
                // chart scrolls within its own region, reachable by keyboard.
                view! {
                    <div class="org-chart" role="region" tabindex="0" aria-label=text::org_chart()>
                        <ul class="position-tree position-tree--root">{items}</ul>
                    </div>
                }
                .into_any()
            }}
            <Show when=move || editable.get()>
                <CommandForms core revision tree uhs_names />
            </Show>
        </section>
    }
}

/// Creating positions (CMD-1), naming or handing over the lead (CMD-4), and putting a UHS
/// under a position (CMD-2). Outside the tree, so updates never wipe what is typed.
#[component]
fn CommandForms(
    core: Handle,
    revision: RwSignal<u64>,
    tree: Memo<CommandStructure>,
    uhs_names: Memo<std::collections::HashMap<String, String>>,
) -> impl IntoView {
    let choices = Memo::new(move |_| tree.with(position_choices));
    let people = Memo::new(move |_| {
        revision.track();
        core::incident_people(core)
    });
    // New position.
    let kind = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let parent = RwSignal::new(String::new());
    let created_error = RwSignal::new(None::<&'static str>);
    let create = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let parent_id = parent.get_untracked();
        let position = PositionCreated {
            position: String::new(),
            kind: kind.get_untracked(),
            name: name.get_untracked().trim().to_owned(),
            parent: (!parent_id.is_empty()).then_some(parent_id),
            service: None,
            organisation: None,
        };
        if position.check().is_err() {
            created_error.set(Some(text::position_missing()));
            return;
        }
        if core::create_position(core, revision, position).is_ok() {
            name.set(String::new());
            created_error.set(None);
        }
    };
    // Leader.
    let lead_position = RwSignal::new(String::new());
    let leader = RwSignal::new(String::new());
    let lead = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let (position, who) = (
            lead_position.get_untracked(),
            leader.get_untracked().trim().to_owned(),
        );
        if position.is_empty() || who.is_empty() {
            return;
        }
        let set = LeaderSet {
            position,
            leader: who,
        };
        if core::command_event(
            core,
            revision,
            ordo_command::kinds::LEADER_SET,
            &set.encode().unwrap_or_default(),
        )
        .is_ok()
        {
            leader.set(String::new());
        }
    };
    // UHS under a position.
    let uhs = RwSignal::new(String::new());
    let uhs_position = RwSignal::new(String::new());
    let assign = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let id = uhs.get_untracked();
        if id.is_empty() {
            return;
        }
        let position = uhs_position.get_untracked();
        let a = Assigned {
            unit: None,
            uhs: Some(id),
            position: (!position.is_empty()).then_some(position),
        };
        let _ = core::command_event(
            core,
            revision,
            ordo_command::kinds::ASSIGNED,
            &a.encode().unwrap_or_default(),
        );
    };
    let position_options = move |none: &'static str| {
        view! {
            <option value="">{none}</option>
            {move || choices.get().into_iter().map(|(id, title)| view! { <option value=id>{title}</option> }).collect_view()}
        }
    };
    view! {
        <form class="form command-form" novalidate on:submit=create>
            <h3>{text::position_create()}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="position-kind">{text::position_kind()}</label>
                    <select id="position-kind" prop:value=move || kind.get() on:change=move |ev| kind.set(event_target_value(&ev))>
                        <option value="">{text::position_kind_choose()}</option>
                        {STRUCTURE.kinds.iter().map(|k| view! { <option value=k.id.clone()>{k.label.clone()}</option> }).collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label for="position-name">{text::position_name()}</label>
                    <input id="position-name" type="text" autocomplete="off" bind:value=name />
                </div>
                <div class="field">
                    <label for="position-parent">{text::position_parent()}</label>
                    <select id="position-parent" prop:value=move || parent.get() on:change=move |ev| parent.set(event_target_value(&ev))>
                        {position_options(text::position_top())}
                    </select>
                </div>
            </div>
            <p role="alert" class="error">{move || created_error.get()}</p>
            <div class="row"><button type="submit" class="button">{text::position_create()}</button></div>
        </form>
        <form class="form command-form" novalidate on:submit=lead>
            <h3>{text::leader_set()}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="lead-position">{text::position()}</label>
                    <select id="lead-position" prop:value=move || lead_position.get() on:change=move |ev| lead_position.set(event_target_value(&ev))>
                        {position_options(text::position_choose())}
                    </select>
                </div>
                <div class="field">
                    <label for="lead-person">{text::leader_label()}</label>
                    <input id="lead-person" type="text" autocomplete="off" list="lead-people" bind:value=leader />
                    <Suggestions id="lead-people" values=people core />
                </div>
            </div>
            <div class="row"><button type="submit" class="button">{text::leader_set()}</button></div>
        </form>
        <form class="form command-form" novalidate on:submit=assign>
            <h3>{text::uhs_assign()}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="assign-uhs">{text::uhs_title()}</label>
                    <select id="assign-uhs" prop:value=move || uhs.get() on:change=move |ev| uhs.set(event_target_value(&ev))>
                        <option value="">{text::uhs_choose()}</option>
                        {move || uhs_names.get().into_iter().map(|(id, name)| view! { <option value=id>{name}</option> }).collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label for="assign-position">{text::position()}</label>
                    <select id="assign-position" prop:value=move || uhs_position.get() on:change=move |ev| uhs_position.set(event_target_value(&ev))>
                        {position_options(text::no_position())}
                    </select>
                </div>
            </div>
            <div class="row"><button type="submit" class="button">{text::uhs_assign()}</button></div>
        </form>
    }
}
