//! The shared modules' views (ADR-0010): incidents, people, units and crews, the command
//! structure, Aufträge and reports, the situation map, the overview and the incident log.

pub(crate) mod command;
pub(crate) mod commands;
pub(crate) mod crew;
pub(crate) mod gallery;
pub(crate) mod helpers;
pub(crate) mod incidents;
pub(crate) mod lage;
pub(crate) mod log;
pub(crate) mod map_region;
pub(crate) mod object_card;
pub(crate) mod reports;
pub(crate) mod situation;
pub(crate) mod summary;
pub(crate) mod tasks;
pub(crate) mod text;
pub(crate) mod units;

use crate::shell::nav;

/// The workspaces of this part, in launcher order (ADR-0011, ADR-0061).
pub(crate) const WORKSPACES: &[nav::Workspace] = &[
    // [impl->dsn~crew-workspace~3]
    // A unit's crew: its status, tasks, patient cases and transports (UNIT-9, ADR-0079).
    nav::Workspace {
        id: "units.crew",
        // Under Sanität: medical units' crews (product owner, 2026-11-17).
        package: || crate::shared::text::tab_med(),
        label: || crate::shared::text::ws_crew(),
        requires: &["incident"],
        permission: Some(crate::core::permission::CREW_WORK),
        or_organisation: &[],
        views: &[
            nav::view(
                "tasks",
                crate::shared::text::crew_tasks,
                "crew tasks status patient cases",
                "clipboard-list",
            ),
            nav::view(
                "transports",
                crate::shared::text::transports,
                "crew transports",
                "ambulance",
            ),
            // [impl->dsn~crew-patients~1]
            // Filled by the Med package (ADR-0093), for those who read clinical data.
            nav::View {
                requires: &["med.clinical"],
                ..nav::view(
                    "patients",
                    crate::shared::text::crew_patients,
                    "crew patients register",
                    "users",
                )
            },
        ],
        hidden: false,
    },
    // The incident command (ADR-0061): what is set up before and adjusted during it …
    nav::Workspace {
        id: "command.prepare",
        package: || crate::shared::text::tab_command(),
        label: || crate::shared::text::ws_prepare(),
        requires: &["incident"],
        permission: Some(crate::core::permission::VIEW_SITUATION),
        or_organisation: &[],
        views: &[
            nav::view(
                "incident",
                crate::shared::text::incident_page,
                "incident description date time start pause resume close",
                "flag",
            ),
            // [impl->dsn~page-tabs~5]
            // Two parts in tabs (feedback of 2026-11-21): the helpers and taking people in,
            // the devices (admin package, place 30). The command structure is on
            // Führung & Raum (ADR-0095).
            nav::View {
                tabs: &[
                    nav::tab("helpers", crate::shared::text::tab_helpers, 0),
                    nav::tab("devices", crate::shared::text::tab_devices, 30),
                ],
                ..nav::view(
                    "people",
                    crate::shared::text::people,
                    "people helpers roles roster devices",
                    "id-card",
                )
            },
            // [impl->dsn~page-tabs~5]
            // Two parts in tabs (feedback of 2026-11-22): the units and UHS, and the hospital
            // list the Med package keeps there (place 40).
            nav::View {
                tabs: &[
                    nav::tab("units", crate::shared::text::tab_resources, 0),
                    nav::tab("hospitals", crate::shared::text::tab_hospitals, 40),
                ],
                ..nav::view(
                    "units",
                    crate::shared::text::tab_resources,
                    "units hospitals",
                    "truck",
                )
            },
            // [impl->dsn~workspaces-by-phase~5]
            // [impl->dsn~command-and-space~2]
            // Führung & Raum (ADR-0095): the command structure and the map prepared on one
            // page, its region last.
            // "Struktur" in the phone's bar, where the name takes two lines (product owner,
            // 2026-11-22; the name is to be revisited).
            // [impl->dsn~page-tabs~5]
            // The structure beside the map where both fit, tabs where narrower (ADR-0097).
            nav::View {
                short: Some(|| crate::shared::text::command_space_short()),
                tabs: &[
                    nav::PageTab {
                        short: Some(|| crate::shared::text::structure_short()),
                        ..nav::tab("structure", crate::shared::text::structure_title, 10)
                    },
                    nav::PageTab {
                        short: Some(|| crate::shared::text::map_short()),
                        column: crate::shell::ui::Column::End,
                        ..nav::tab("map", crate::shared::text::map_title, 20)
                    },
                    nav::PageTab {
                        short: Some(|| crate::shared::text::map_region_short()),
                        column: crate::shell::ui::Column::Below,
                        ..nav::tab("region", crate::shared::text::map_region_title, 30)
                    },
                ],
                columns: crate::shell::ui::TabsMode::ColumnsFrom84,
                ..nav::view(
                    "map",
                    crate::shared::text::command_space,
                    "command structure org chart positions leader situation map region edit draw place units address",
                    "network",
                )
            },
        ],
        hidden: false,
    },
    // … and leading it while it runs (the menu follows the order of the work).
    nav::Workspace {
        id: "command.lead",
        package: || crate::shared::text::tab_command(),
        label: || crate::shared::text::ws_lead(),
        requires: &["incident"],
        // The incident command's views; UHS crews do not see them.
        permission: Some(crate::core::permission::VIEW_SITUATION),
        or_organisation: &[],
        views: &[
            nav::view(
                "overview",
                crate::shared::text::overview,
                "overview summary",
                "layout-grid",
            ),
            // [impl->dsn~command-lage~1]
            // [impl->dsn~page-tabs~5]
            // *Lage* (ADR-0099): the map beside the units or the treatment places (Med
            // package, place 30) where both fit; tabs where narrower. Its address stays.
            nav::View {
                tabs: &[
                    nav::PageTab {
                        column: crate::shell::ui::Column::End,
                        ..nav::tab("map", crate::shared::text::map_short, 10)
                    },
                    nav::tab("units", crate::shared::text::tab_resources, 20),
                    nav::tab("places", crate::shared::text::places, 30),
                ],
                columns: crate::shell::ui::TabsMode::BesideFrom84,
                ..nav::view(
                    "map",
                    crate::shared::text::lage,
                    "situation map units treatment places",
                    "map",
                )
            },
            nav::view(
                "dispatch",
                crate::shared::text::dispatch,
                "dispatch tasks orders",
                "clipboard-list",
            ),
            // [impl->dsn~page-tabs~5]
            // The transports beside the free units and hospitals they are assigned to, where
            // both fit; tabs where narrower (feedback of 2026-11-22). Filled by the Med
            // package.
            nav::View {
                tabs: &[
                    nav::tab("requests", crate::shared::text::transports, 10),
                    nav::PageTab {
                        short: Some(|| crate::shared::text::tab_free_units_short()),
                        column: crate::shell::ui::Column::End,
                        ..nav::tab("free", crate::shared::text::tab_free_units, 20)
                    },
                    nav::PageTab {
                        column: crate::shell::ui::Column::End,
                        ..nav::tab("hospitals", crate::shared::text::tab_hospitals, 30)
                    },
                ],
                columns: crate::shell::ui::TabsMode::ColumnsFrom60,
                ..nav::view(
                    "transports",
                    crate::shared::text::transports,
                    "transports hospitals",
                    "ambulance",
                )
            },
        ],
        hidden: false,
    },
    // [impl->dsn~incident-log~1]
    // Everyone in the incident; each device shows only what its user may read (LOG-6).
    nav::Workspace {
        id: "incident.log",
        package: || crate::shared::text::tab_command(),
        label: || crate::shared::text::log_title(),
        requires: &["incident"],
        permission: None,
        or_organisation: &[],
        views: &[nav::view(
            "log",
            crate::shared::text::log_title,
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
    bell: tasks::bell,
    palette: commands::PALETTE,
    log: log::PART,
    gallery: gallery::PARTS,
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
        ("units.crew", "transports") => {
            one(view! { <crew::CrewTransports core revision /> }.into_any())
        }
        // The crew's patients come from the Med package (ADR-0093).
        ("units.crew", "patients") => Vec::new(),
        ("units.crew", _) => one(view! { <crew::CrewTasks core revision /> }.into_any()),
        // [impl->dsn~workspaces-by-phase~5]
        // [impl->dsn~map-drawing~2]
        // [impl->dsn~command-lage~1]
        // *Lage*: the map, and the units (ADR-0099).
        ("command.lead", "map") => {
            situation::map_asks();
            vec![
                (
                    10,
                    view! { <situation::SituationView core revision /> }.into_any(),
                ),
                (
                    20,
                    view! { <lage::UnitsAtGlance core revision /> }.into_any(),
                ),
            ]
        }
        // [impl->dsn~reports~3]
        ("command.lead", "dispatch") => {
            one(view! { <tasks::OpenTasks core revision /> }.into_any())
        }
        ("command.lead", "transports") => Vec::new(),
        ("command.lead", _) => one(view! { <summary::CommandOverview core revision /> }.into_any()),
        ("command.prepare", "units") => {
            one(view! { <units::ResourcesView core revision /> }.into_any())
        }
        // Who is there and taking people in first (feedback of 2026-11-14), the devices
        // after them (administration).
        ("command.prepare", "people") => vec![(
            10,
            view! { <helpers::HelpersView core revision /> }.into_any(),
        )],
        // [impl->dsn~command-and-space~2]
        // Führung & Raum (ADR-0095, ADR-0097), also for old links to the command structure;
        // opened to place one object with `place:<kind>:<ID>`, on the map's part.
        ("command.prepare", "map" | "structure") => situation::preparation_sections(
            core,
            revision,
            record.get_untracked().and_then(|r| situation::preset(&r)),
        ),
        ("command.prepare", _) => {
            one(view! { <incidents::IncidentPage core revision /> }.into_any())
        }
        ("incident.log", _) => one(view! { <log::LogView core revision /> }.into_any()),
        _ => Vec::new(),
    }
}
