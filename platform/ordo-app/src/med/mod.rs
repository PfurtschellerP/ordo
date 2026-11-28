//! The Med package's views (ADR-0019): patients and their records, orders, transports,
//! treatment places, staff requests, and the UHS the incident command sets up.

pub(crate) mod commands;
pub(crate) mod document;
pub(crate) mod flow;
pub(crate) mod gallery;
pub(crate) mod handover;
pub(crate) mod log;
pub(crate) mod orders;
pub(crate) mod overview;
pub(crate) mod patients;
pub(crate) mod places;
pub(crate) mod record;
pub(crate) mod staffing;
pub(crate) mod text;
pub(crate) mod transports;
pub(crate) mod triage;
pub(crate) mod uhs;

use crate::shell::nav;

/// The workspaces of this part, in launcher order (ADR-0011, ADR-0061).
pub(crate) const WORKSPACES: &[nav::Workspace] = &[nav::Workspace {
    id: "med.uhs",
    package: || crate::med::text::tab_med(),
    label: crate::med::text::ws_uhs,
    requires: &["incident", "med.clinical"],
    permission: None,
    or_organisation: &[],
    views: &[
        nav::view(
            "patients",
            crate::med::text::patients_title,
            "patients",
            "users",
        ),
        nav::view(
            "orders",
            crate::med::text::orders,
            "orders",
            "clipboard-list",
        ),
        // The UHS are set up by the incident command (ADR-0086); here, their places.
        nav::view(
            "places",
            crate::med::text::places,
            "treatment places overview uhs free full",
            "tent",
        ),
    ],
    hidden: false,
}];

use crate::shell::registry::{Package, PageArgs, Section};
use leptos::prelude::*;

// [impl->dsn~package-registry~1]
/// The Med package in the app's registry.
pub(crate) const PACKAGE: Package = Package {
    workspaces: WORKSPACES,
    provide: || {
        provide_context(patients::Drafts::default());
        provide_context(orders::Performers::default());
    },
    sections,
    bell: |core| {
        let mut items = handover::bell(core);
        items.extend(staffing::bell(core));
        items.extend(transports::bell(core));
        items
    },
    palette: commands::PALETTE,
    log: log::PART,
    gallery: gallery::PARTS,
};

fn sections(a: &PageArgs, workspace: &str, view: &str) -> Vec<Section> {
    let PageArgs {
        core,
        revision,
        nav,
        record,
    } = *a;
    match (workspace, view) {
        ("med.uhs", "orders") => vec![(
            0,
            view! { <orders::OrdersView core revision nav /> }.into_any(),
        )],
        // [impl->dsn~uhs-overview~3]
        // The places of every UHS; *UHS verwalten* is gone (ADR-0086), its old links land
        // here.
        ("med.uhs", "places" | "overview" | "uhs") => {
            vec![(
                0,
                view! { <overview::OverviewView core revision /> }.into_any(),
            )]
        }
        // [impl->dsn~crew-patients~1]
        // The crew's patients in its own workspace (ADR-0093).
        ("units.crew", "patients") => vec![(
            0,
            view! { <patients::PatientsView core revision nav record=Signal::from(record) scope=patients::Scope::Crew /> }
                .into_any(),
        )],
        ("med.uhs", _) => vec![(
            0,
            view! { <patients::PatientsView core revision nav record=Signal::from(record) /> }
                .into_any(),
        )],
        // In the incident command's views: the treatment places beside the map, the staff
        // requests above the Aufträge, and the transport board.
        // [impl->dsn~command-lage~1]
        // [impl->dsn~command-treatment-places~2]
        ("command.lead", "map") => {
            crate::shared::situation::map_asks();
            vec![(
                30,
                view! { <places::PlacesAtGlance core revision /> }.into_any(),
            )]
        }
        // [impl->dsn~reports~4]
        ("command.lead", "dispatch") => {
            vec![(
                30,
                view! { <staffing::StaffRequestsBoard core revision /> }.into_any(),
            )]
        }
        // [impl->dsn~hospital-list~2]
        // The hospital list is kept with the units, in its own tab (place 40).
        ("command.prepare", "units") => {
            vec![(
                40,
                view! { <transports::HospitalsSetup core revision /> }.into_any(),
            )]
        }
        // [impl->dsn~page-tabs~5]
        ("command.lead", "transports") => transports::transport_sections(core, revision),
        // [impl->dsn~profile-page~1]
        // The triage aid of this device, for those who triage (PAT-11, ADR-0100).
        ("person.devices", "profile") if crate::core::granted(core, ordo_med_patients::COMPARTMENT) => {
            vec![(
                30,
                view! {
                    <section class="card profile-part" aria-labelledby="profile-triage-title">
                        <h2 id="profile-triage-title">{text::profile_triage()}</h2>
                        <patients::TriageAid id="profile-triage-algorithm" />
                    </section>
                }
                .into_any(),
            )]
        }
        _ => Vec::new(),
    }
}
