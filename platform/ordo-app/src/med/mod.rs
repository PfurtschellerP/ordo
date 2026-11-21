//! The Med package's views (ADR-0019): patients and their records, orders, transports,
//! treatment places, staff requests, and the UHS the incident command sets up.

pub(crate) mod commands;
pub(crate) mod flow;
pub(crate) mod handover;
pub(crate) mod log;
pub(crate) mod orders;
pub(crate) mod overview;
pub(crate) mod patients;
pub(crate) mod places;
pub(crate) mod record;
pub(crate) mod staffing;
pub(crate) mod transports;
pub(crate) mod triage;
pub(crate) mod uhs;

use crate::shell::nav;

/// The workspaces of this part, in launcher order (ADR-0011, ADR-0061).
pub(crate) const WORKSPACES: &[nav::Workspace] = &[nav::Workspace {
    id: "med.uhs",
    package: crate::text::TAB_MED,
    label: "UHS",
    requires: &["incident", "med.clinical"],
    permission: None,
    or_organisation: &[],
    views: &[
        nav::view("patients", "Patienten", "patients", "users"),
        nav::view("orders", crate::text::ORDERS, "orders", "clipboard-list"),
        // The UHS are set up by the incident command (ADR-0086); here, their places.
        nav::view(
            "places",
            crate::text::PLACES,
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
        items
    },
    palette: commands::PALETTE,
    log: log::PART,
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
        ("med.uhs", _) => vec![(
            0,
            view! { <patients::PatientsView core revision nav record=Signal::from(record) /> }
                .into_any(),
        )],
        // In the incident command's views: the counts per UHS under the map, the staff
        // requests above the Aufträge, and the transport board.
        ("command.lead", "map") => {
            vec![(20, view! { <uhs::UhsCounts core revision /> }.into_any())]
        }
        // [impl->dsn~reports~3]
        ("command.lead", "dispatch") => {
            vec![(
                10,
                view! { <staffing::StaffRequestsBoard core revision /> }.into_any(),
            )]
        }
        ("command.lead", "transports") => {
            vec![(
                10,
                view! { <transports::TransportsView core revision /> }.into_any(),
            )]
        }
        _ => Vec::new(),
    }
}
