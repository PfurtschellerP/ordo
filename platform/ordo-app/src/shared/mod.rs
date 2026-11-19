//! The shared modules' views (ADR-0010): incidents, people, units and crews, the command
//! structure, Aufträge and reports, the situation map, the overview and the incident log.

pub(crate) mod command;
pub(crate) mod commands;
pub(crate) mod crew;
pub(crate) mod helpers;
pub(crate) mod incidents;
pub(crate) mod log;
pub(crate) mod map_region;
pub(crate) mod object_card;
pub(crate) mod reports;
pub(crate) mod situation;
pub(crate) mod summary;
pub(crate) mod tasks;
pub(crate) mod units;

use crate::shell::nav;

/// The workspaces of this part, in launcher order (ADR-0011, ADR-0061).
pub(crate) const WORKSPACES: &[nav::Workspace] = &[
    // [impl->dsn~crew-workspace~3]
    // A unit's crew: its status, tasks, patient cases and transports (UNIT-9, ADR-0079).
    nav::Workspace {
        id: "units.crew",
        // Under Sanität: medical units' crews (product owner, 2026-11-17).
        package: crate::text::TAB_MED,
        label: crate::text::WS_CREW,
        requires: &["incident"],
        permission: Some(crate::core::permission::CREW_WORK),
        or_organisation: &[],
        views: &[
            nav::view(
                "tasks",
                crate::text::CREW_TASKS,
                "crew tasks status patient cases",
                "clipboard-list",
            ),
            nav::view(
                "transports",
                crate::text::TRANSPORTS,
                "crew transports",
                "ambulance",
            ),
        ],
        hidden: false,
    },
    // The incident command (ADR-0061): what is set up before and adjusted during it …
    nav::Workspace {
        id: "command.prepare",
        package: crate::text::TAB_COMMAND,
        label: crate::text::WS_PREPARE,
        requires: &["incident"],
        permission: Some(crate::core::permission::VIEW_SITUATION),
        or_organisation: &[],
        views: &[
            nav::view(
                "incident",
                crate::text::INCIDENT_PAGE,
                "incident description date time start pause resume close",
                "flag",
            ),
            // [impl->dsn~workspaces-by-phase~4]
            // The map prepared on one page: its region and its objects (ADR-0067).
            nav::view(
                "map",
                crate::text::MAP_TITLE,
                "situation map region edit draw place units address",
                "map",
            ),
            nav::view("units", crate::text::TAB_RESOURCES, "units", "truck"),
            nav::view(
                "people",
                crate::text::PEOPLE,
                "people helpers roles roster devices command structure",
                "id-card",
            ),
        ],
        hidden: false,
    },
    // … and leading it while it runs (the menu follows the order of the work).
    nav::Workspace {
        id: "command.lead",
        package: crate::text::TAB_COMMAND,
        label: crate::text::WS_LEAD,
        requires: &["incident"],
        // The incident command's views; UHS crews do not see them.
        permission: Some(crate::core::permission::VIEW_SITUATION),
        or_organisation: &[],
        views: &[
            nav::view(
                "overview",
                crate::text::OVERVIEW,
                "overview summary",
                "layout-grid",
            ),
            nav::view("map", crate::text::MAP_TITLE, "situation map", "map"),
            nav::view(
                "dispatch",
                crate::text::DISPATCH,
                "dispatch tasks orders",
                "clipboard-list",
            ),
            nav::view(
                "transports",
                crate::text::TRANSPORTS,
                "transports",
                "ambulance",
            ),
        ],
        hidden: false,
    },
    // [impl->dsn~incident-log~1]
    // Everyone in the incident; each device shows only what its user may read (LOG-6).
    nav::Workspace {
        id: "incident.log",
        package: crate::text::TAB_COMMAND,
        label: crate::text::LOG_TITLE,
        requires: &["incident"],
        permission: None,
        or_organisation: &[],
        views: &[nav::view(
            "log",
            crate::text::LOG_TITLE,
            "incident log diary",
            "notebook-text",
        )],
        hidden: false,
    },
];

use crate::shell::registry::{Package, PageArgs, Section};
use leptos::prelude::*;

// [impl->dsn~package-registry~1]
/// The shared modules in the app's registry.
pub(crate) const PACKAGE: Package = Package {
    workspaces: WORKSPACES,
    provide: || {},
    sections,
    palette: commands::PALETTE,
};

fn sections(a: &PageArgs, workspace: &str, view: &str) -> Vec<Section> {
    let PageArgs {
        core,
        revision,
        record,
        ..
    } = *a;
    let one = |v: AnyView| vec![(20, v)];
    match (workspace, view) {
        ("units.crew", "transports") => one(view! { <crew::CrewTransports core revision /> }.into_any()),
        ("units.crew", _) => one(view! { <crew::CrewTasks core revision /> }.into_any()),
        // [impl->dsn~workspaces-by-phase~4]
        // [impl->dsn~map-drawing~2]
        ("command.lead", "map") => vec![(10, view! { <situation::SituationView core revision /> }.into_any())],
        // [impl->dsn~reports~3]
        ("command.lead", "dispatch") => one(view! { <tasks::OpenTasks core revision /> }.into_any()),
        ("command.lead", "transports") => Vec::new(),
        ("command.lead", _) => one(view! { <summary::CommandOverview core revision /> }.into_any()),
        ("command.prepare", "units") => one(view! { <units::ResourcesView core revision /> }.into_any()),
        // The command structure is prepared with the people (ADR-0067); old links to its
        // own view land here. Who is there and taking people in first (feedback of
        // 2026-11-14), the devices after them (administration).
        ("command.prepare", "people" | "structure") => vec![
            (10, view! { <helpers::HelpersView core revision /> }.into_any()),
            (20, view! { <command::LeadershipView core revision /> }.into_any()),
        ],
        // The situation map prepared on one page: its region, then the editor, also opened
        // to place one object (`place:<kind>:<ID>`).
        ("command.prepare", "map") => one(view! {
            <situation::MapPreparation core revision preset=record.get_untracked().and_then(|r| situation::preset(&r)) />
        }.into_any()),
        ("command.prepare", _) => one(view! { <incidents::IncidentPage core revision /> }.into_any()),
        ("incident.log", _) => one(view! { <log::LogView core revision /> }.into_any()),
        _ => Vec::new(),
    }
}
