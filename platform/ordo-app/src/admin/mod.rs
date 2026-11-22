//! Administration beyond one incident: accounts, organisation roles, devices, login
//! providers, and the licence notices.

pub(crate) mod accounts;
pub(crate) mod devices;
pub(crate) mod login_providers;
pub(crate) mod notices;
pub(crate) mod roles;
pub(crate) mod text;

use crate::shell::nav;

/// The workspaces of this part, in launcher order (ADR-0011, ADR-0061).
pub(crate) const WORKSPACES: &[nav::Workspace] = &[
    // Administration beyond one incident: only organisation roles open it, never a role in
    // an incident; each view needs its own permission.
    nav::Workspace {
        id: "admin.access",
        package: || crate::admin::text::tab_admin(),
        label: || crate::admin::text::ws_access(),
        requires: &[],
        permission: None,
        or_organisation: &["manage_accounts", "manage_roles", "manage_devices"],
        views: &[
            // [impl->dsn~local-accounts~1]
            nav::View {
                permission: Some("manage_accounts"),
                ..nav::view(
                    "accounts",
                    crate::admin::text::accounts,
                    "accounts login passwords users",
                    "users",
                )
            },
            // [impl->dsn~organisation-roles~1]
            nav::View {
                permission: Some("manage_roles"),
                ..nav::view(
                    "permissions",
                    crate::admin::text::org_roles,
                    "permissions roles administrators planners organisation",
                    "key-round",
                )
            },
            nav::View {
                permission: Some("manage_devices"),
                ..nav::view(
                    "devices",
                    crate::admin::text::devices,
                    "devices",
                    "tablet-smartphone",
                )
            },
            // [impl->dsn~login-providers~1]
            nav::View {
                permission: Some("manage_accounts"),
                ..nav::view(
                    "login",
                    crate::admin::text::login_providers,
                    "login providers single sign-on oidc entra keycloak",
                    "log-in",
                )
            },
        ],
        hidden: false,
    },
    // [impl->dsn~licence-notices~2]
    // Version, licences and source code, for everyone (LIC-1): no permission needed.
    nav::Workspace {
        id: "admin.info",
        package: || crate::admin::text::tab_admin(),
        label: || crate::admin::text::ws_info(),
        requires: &[],
        permission: None,
        or_organisation: &[],
        views: &[nav::view(
            "info",
            crate::admin::text::ws_info,
            "info about version licences licenses source code notices",
            "info",
        )],
        hidden: false,
    },
    // The person's own devices, for everyone (SEC-4): opened from the person's menu.
    nav::Workspace {
        id: "person.devices",
        package: || crate::admin::text::tab_person(),
        label: crate::admin::text::my_devices,
        requires: &[],
        permission: None,
        or_organisation: &[],
        views: &[nav::view(
            "devices",
            crate::admin::text::my_devices,
            "my devices",
            "tablet-smartphone",
        )],
        hidden: true,
    },
];

use crate::shell::registry::{Package, PageArgs, Section};
use leptos::prelude::*;

// [impl->dsn~package-registry~1]
/// The administration in the app's registry.
pub(crate) const PACKAGE: Package = Package {
    workspaces: WORKSPACES,
    provide: || {},
    sections,
    bell: |_| Vec::new(),
    palette: crate::shell::palette::PalettePart::NONE,
    log: crate::shared::log::LogPart::NONE,
    gallery: &[],
};

fn sections(a: &PageArgs, workspace: &str, view: &str) -> Vec<Section> {
    let PageArgs { core, revision, .. } = *a;
    let one = |v: AnyView| vec![(0, v)];
    match (workspace, view) {
        // The devices used in the incident, last on the page of its people (ORG-12).
        ("command.prepare", "people") => {
            vec![(
                30,
                view! { <devices::IncidentDevicesView core revision /> }.into_any(),
            )]
        }
        ("admin.access", "permissions") => {
            one(view! { <roles::RolesView core revision /> }.into_any())
        }
        ("admin.access", "devices") => {
            one(view! { <devices::DevicesView core revision /> }.into_any())
        }
        ("admin.access", "login") => {
            one(view! { <login_providers::LoginProvidersView core revision /> }.into_any())
        }
        ("admin.access", _) => one(view! { <accounts::AccountsView core revision /> }.into_any()),
        ("admin.info", _) => one(view! { <notices::InfoView /> }.into_any()),
        ("person.devices", _) => {
            one(view! { <devices::DevicesView core revision mine_only=true /> }.into_any())
        }
        _ => Vec::new(),
    }
}
