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
use crate::text;
use crate::ui::{Action, Actions, History, Suggestions};

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

/// One position and, nested, the positions reporting to it.
fn position_item(
    core: Handle,
    revision: RwSignal<u64>,
    s: &CommandStructure,
    id: &str,
    uhs_names: &std::collections::HashMap<String, String>,
    editable: bool,
) -> AnyView {
    let Some(p) = s.positions.get(id) else {
        return ().into_any();
    };
    let title = position_title(p);
    let leader = p
        .leader()
        .map_or_else(|| text::NO_LEADER.to_owned(), text::leader);
    let handovers: Vec<(String, String, String)> = p
        .leaders
        .iter()
        .map(|h| {
            (
                core::clock(h.at.wall_ms),
                text::handover(h.from.as_deref(), &h.to),
                h.by.to_string(),
            )
        })
        .collect();
    let units = (!p.units.is_empty()).then(|| text::assigned_units(&p.units.join(", ")));
    let uhs: Vec<String> = p
        .uhs
        .iter()
        .map(|u| uhs_names.get(u).cloned().unwrap_or_else(|| u.clone()))
        .collect();
    let uhs = (!uhs.is_empty()).then(|| text::assigned_uhs(&uhs.join(", ")));
    let span = p
        .span_exceeded()
        .then(|| text::span_of_control(p.subordinates()));
    let close = editable.then(|| {
        let id = id.to_owned();
        vec![Action {
            text: text::POSITION_CLOSE,
            label: text::position_close_label(&title),
            quiet: true,
            run: Callback::new(move |()| {
                let closed = PositionClosed {
                    position: id.clone(),
                };
                let _ = core::command_event(
                    core,
                    revision,
                    ordo_command::POSITION_CLOSED,
                    &closed.encode().unwrap_or_default(),
                );
            }),
        }]
    });
    let children: Vec<AnyView> = p
        .children
        .iter()
        .map(|child| position_item(core, revision, s, child, uhs_names, editable))
        .collect();
    view! {
        <li class="position" data-position=id.to_owned()>
            <p class="position__title"><strong>{title.clone()}</strong></p>
            <p class="position__leader">{leader}</p>
            {units.map(|u| view! { <p class="position__units">{u}</p> })}
            {uhs.map(|u| view! { <p class="position__uhs">{u}</p> })}
            {span.map(|s| view! { <p class="notice position__span">{s}</p> })}
            {(!handovers.is_empty()).then(|| view! { <History entries=handovers /> })}
            {close.map(|actions| view! { <Actions actions /> })}
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
    let editable = Memo::new(move |_| {
        revision.track();
        core.with_value(|c| {
            c.incident
                .as_ref()
                .is_some_and(|i| c.replica.may(i, "edit_units"))
        })
    });
    let tree = Memo::new(move |_| {
        revision.track();
        structure(core)
    });
    let uhs_names = Memo::new(move |_| {
        revision.track();
        crate::views::uhs_list(core)
            .into_iter()
            .map(|u| (u.id, u.name))
            .collect::<std::collections::HashMap<_, _>>()
    });
    view! {
        <section class="card" aria-labelledby="structure-title">
            <h2 id="structure-title">{text::STRUCTURE_TITLE}</h2>
            {move || {
                let s = tree.get();
                if s.roots.is_empty() {
                    return view! { <p>{text::STRUCTURE_EMPTY}</p> }.into_any();
                }
                let names = uhs_names.get();
                let editable = editable.get();
                let items: Vec<AnyView> = s
                    .roots
                    .iter()
                    .map(|id| position_item(core, revision, &s, id, &names, editable))
                    .collect();
                view! { <ul class="position-tree position-tree--root">{items}</ul> }.into_any()
            }}
            <Show when=move || editable.get()>
                <CommandForms core revision tree uhs_names />
            </Show>
        </section>
        <crate::incidents::RosterView core revision />
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
            created_error.set(Some(text::POSITION_MISSING));
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
            ordo_command::LEADER_SET,
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
            ordo_command::ASSIGNED,
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
            <h3>{text::POSITION_CREATE}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="position-kind">{text::POSITION_KIND}</label>
                    <select id="position-kind" prop:value=move || kind.get() on:change=move |ev| kind.set(event_target_value(&ev))>
                        <option value="">{text::POSITION_KIND_CHOOSE}</option>
                        {STRUCTURE.kinds.iter().map(|k| view! { <option value=k.id.clone()>{k.label.clone()}</option> }).collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label for="position-name">{text::POSITION_NAME}</label>
                    <input id="position-name" type="text" autocomplete="off" bind:value=name />
                </div>
                <div class="field">
                    <label for="position-parent">{text::POSITION_PARENT}</label>
                    <select id="position-parent" prop:value=move || parent.get() on:change=move |ev| parent.set(event_target_value(&ev))>
                        {position_options(text::POSITION_TOP)}
                    </select>
                </div>
            </div>
            <p role="alert" class="error">{move || created_error.get()}</p>
            <div class="row"><button type="submit" class="button">{text::POSITION_CREATE}</button></div>
        </form>
        <form class="form command-form" novalidate on:submit=lead>
            <h3>{text::LEADER_SET}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="lead-position">{text::POSITION}</label>
                    <select id="lead-position" prop:value=move || lead_position.get() on:change=move |ev| lead_position.set(event_target_value(&ev))>
                        {position_options(text::POSITION_CHOOSE)}
                    </select>
                </div>
                <div class="field">
                    <label for="lead-person">{text::LEADER}</label>
                    <input id="lead-person" type="text" autocomplete="off" list="lead-people" bind:value=leader />
                    <Suggestions id="lead-people" values=people />
                </div>
            </div>
            <div class="row"><button type="submit" class="button">{text::LEADER_SET}</button></div>
        </form>
        <form class="form command-form" novalidate on:submit=assign>
            <h3>{text::UHS_ASSIGN}</h3>
            <div class="form--inline">
                <div class="field">
                    <label for="assign-uhs">{text::UHS_TITLE}</label>
                    <select id="assign-uhs" prop:value=move || uhs.get() on:change=move |ev| uhs.set(event_target_value(&ev))>
                        <option value="">{text::UHS_CHOOSE}</option>
                        {move || uhs_names.get().into_iter().map(|(id, name)| view! { <option value=id>{name}</option> }).collect_view()}
                    </select>
                </div>
                <div class="field">
                    <label for="assign-position">{text::POSITION}</label>
                    <select id="assign-position" prop:value=move || uhs_position.get() on:change=move |ev| uhs_position.set(event_target_value(&ev))>
                        {position_options(text::NO_POSITION)}
                    </select>
                </div>
            </div>
            <div class="row"><button type="submit" class="button">{text::UHS_ASSIGN}</button></div>
        </form>
    }
}
