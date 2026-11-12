//! Workspaces and views (ADR-0011): which role sees which views.
//!
//! A workspace belongs to a service package and needs read access to certain
//! compartments. Every workspace of a package the organisation uses appears in the
//! launcher; without access it stays in place, disabled, with the reason — so the
//! layout is the same for every role. Packages that are not used do not appear.
//!
//! Prototype: the definitions are Rust constants; later they come from package and
//! profile data, so organisations can adapt them without code.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};

use crate::text;

/// A view within a workspace.
pub struct View {
    /// Stable ID, e.g. `patients`.
    pub id: &'static str,
    /// Label.
    pub label: &'static str,
    /// English name, found by the command palette in every language (ADR-0036).
    pub name: &'static str,
    /// Icon (Lucide name), shown with the label in the rail and the bottom bar.
    pub icon: &'static str,
    /// A permission this view needs beyond its workspace's, given by the roles in this
    /// incident or by an organisation role (e.g. `manage_accounts`); without it, the view
    /// is not offered.
    pub permission: Option<&'static str>,
}

/// A workspace: the environment of one role.
pub struct Workspace {
    /// Stable ID, e.g. `med.uhs`.
    pub id: &'static str,
    /// Service package, shown as group in the launcher.
    pub package: &'static str,
    /// Label within the package.
    pub label: &'static str,
    /// Compartments the user must be able to read.
    pub requires: &'static [&'static str],
    /// A permission the user's roles must give in this incident (e.g. `view_situation`),
    /// decided by the server's policies.
    pub permission: Option<&'static str>,
    /// Organisation permissions, any of which opens the workspace (e.g. `manage_devices`).
    /// A workspace with these and no incident permission is for administration only.
    pub or_organisation: &'static [&'static str],
    /// Views, in navigation order. The first is the start view.
    pub views: &'static [View],
    /// Not in the launcher: opened from elsewhere (e.g. the person's menu).
    pub hidden: bool,
}

const fn view(
    id: &'static str,
    label: &'static str,
    name: &'static str,
    icon: &'static str,
) -> View {
    View {
        id,
        label,
        name,
        icon,
        permission: None,
    }
}

// [impl->dsn~workspaces-by-phase~1]
/// All workspaces of the packages in use, in launcher order (ADR-0061).
pub const WORKSPACES: &[Workspace] = &[
    Workspace {
        id: "med.uhs",
        package: text::TAB_MED,
        label: "UHS",
        requires: &["incident", "med.clinical"],
        permission: None,
        or_organisation: &[],
        views: &[
            view("patients", "Patienten", "patients", "users"),
            view("orders", text::ORDERS, "orders", "clipboard-list"),
            view("uhs", "UHS verwalten", "manage uhs", "tent"),
            view("overview", text::OVERVIEW, "overview", "chart-column"),
        ],
        hidden: false,
    },
    // The incident command (ADR-0061): leading the running incident …
    Workspace {
        id: "command.lead",
        package: text::TAB_COMMAND,
        label: text::WS_LEAD,
        requires: &["incident"],
        // The incident command's views; UHS crews do not see them.
        permission: Some("view_situation"),
        or_organisation: &[],
        views: &[
            view(
                "overview",
                text::OVERVIEW,
                "overview summary",
                "layout-grid",
            ),
            view("map", text::MAP_TITLE, "situation map", "map"),
            view(
                "dispatch",
                text::DISPATCH,
                "dispatch tasks orders",
                "clipboard-list",
            ),
            view("transports", text::TRANSPORTS, "transports", "ambulance"),
        ],
        hidden: false,
    },
    // … and what is set up before and adjusted during it.
    Workspace {
        id: "command.prepare",
        package: text::TAB_COMMAND,
        label: text::WS_PREPARE,
        requires: &["incident"],
        permission: Some("view_situation"),
        or_organisation: &[],
        views: &[
            view(
                "incident",
                text::INCIDENT_PAGE,
                "incident start close map region",
                "flag",
            ),
            view("units", text::TAB_RESOURCES, "units", "truck"),
            view(
                "people",
                text::PEOPLE,
                "people helpers roles roster",
                "id-card",
            ),
            view(
                "structure",
                text::STRUCTURE_TITLE,
                "command structure",
                "network",
            ),
        ],
        hidden: false,
    },
    // [impl->dsn~incident-log~1]
    // Everyone in the incident; each device shows only what its user may read (LOG-6).
    Workspace {
        id: "incident.log",
        package: text::TAB_COMMAND,
        label: text::LOG_TITLE,
        requires: &["incident"],
        permission: None,
        or_organisation: &[],
        views: &[view(
            "log",
            text::LOG_TITLE,
            "incident log diary",
            "notebook-text",
        )],
        hidden: false,
    },
    // Administration beyond one incident: only organisation roles open it, never a role in
    // an incident; each view needs its own permission.
    Workspace {
        id: "admin.access",
        package: text::TAB_ADMIN,
        label: text::WS_ACCESS,
        requires: &[],
        permission: None,
        or_organisation: &["manage_accounts", "manage_roles", "manage_devices"],
        views: &[
            // [impl->dsn~local-accounts~1]
            View {
                permission: Some("manage_accounts"),
                ..view(
                    "accounts",
                    text::ACCOUNTS,
                    "accounts login passwords users",
                    "users",
                )
            },
            // [impl->dsn~organisation-roles~1]
            View {
                permission: Some("manage_roles"),
                ..view(
                    "permissions",
                    text::ORG_ROLES,
                    "permissions roles administrators planners organisation",
                    "key-round",
                )
            },
            View {
                permission: Some("manage_devices"),
                ..view("devices", text::DEVICES, "devices", "tablet-smartphone")
            },
            // [impl->dsn~login-providers~1]
            View {
                permission: Some("manage_accounts"),
                ..view(
                    "login",
                    text::LOGIN_PROVIDERS,
                    "login providers single sign-on oidc entra keycloak",
                    "log-in",
                )
            },
        ],
        hidden: false,
    },
    // The person's own devices, for everyone (SEC-4): opened from the person's menu.
    Workspace {
        id: "person.devices",
        package: text::TAB_PERSON,
        label: text::MY_DEVICES,
        requires: &[],
        permission: None,
        or_organisation: &[],
        views: &[view(
            "devices",
            text::MY_DEVICES,
            "my devices",
            "tablet-smartphone",
        )],
        hidden: true,
    },
];

/// The workspace with `id`.
pub fn workspace(id: &str) -> Option<&'static Workspace> {
    WORKSPACES.iter().find(|w| w.id == id)
}

/// Packages in launcher order, each with its workspaces.
pub fn packages() -> Vec<(&'static str, Vec<&'static Workspace>)> {
    let mut packages: Vec<(&'static str, Vec<&'static Workspace>)> = Vec::new();
    for w in WORKSPACES.iter().filter(|w| !w.hidden) {
        match packages.iter_mut().find(|(p, _)| *p == w.package) {
            Some((_, list)) => list.push(w),
            None => packages.push((w.package, vec![w])),
        }
    }
    packages
}

/// Where the user is: workspace, view and optionally one record (e.g. a patient). It is
/// the URL path (`/med.uhs/patients/DE-0042`), handled by `leptos_router`, so the
/// browser's and the phone's back button work and a view can be reopened after a reload.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Route {
    /// Workspace ID.
    pub workspace: String,
    /// View ID.
    pub view: String,
    /// Record within the view, e.g. a triage tag number.
    pub record: Option<String>,
}

/// Characters escaped in a path segment: everything except unreserved ones (RFC 3986).
const SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

impl Route {
    /// A route to a view.
    pub fn view(workspace: &str, view: &str) -> Self {
        Self {
            workspace: workspace.to_owned(),
            view: view.to_owned(),
            record: None,
        }
    }

    /// A route to a record.
    pub fn record(workspace: &str, view: &str, record: &str) -> Self {
        Self {
            record: Some(record.to_owned()),
            ..Self::view(workspace, view)
        }
    }

    /// The URL path.
    pub fn to_path(&self) -> String {
        let segment = |s: &str| utf8_percent_encode(s, SEGMENT).to_string();
        let mut path = format!("/{}/{}", segment(&self.workspace), segment(&self.view));
        if let Some(record) = &self.record {
            path.push('/');
            path.push_str(&segment(record));
        }
        path
    }

    /// Reads a URL path; `None` if it is not a route (e.g. `/`).
    pub fn from_path(path: &str) -> Option<Self> {
        let decode = |s: &str| {
            percent_decode_str(s)
                .decode_utf8()
                .ok()
                .map(|c| c.into_owned())
        };
        let mut parts = path.strip_prefix('/')?.split('/');
        let workspace = decode(parts.next()?)?;
        let view = decode(parts.next()?)?;
        let record = match parts.next() {
            Some(part) => Some(decode(part)?),
            None => None,
        };
        if parts.next().is_some() || workspace.is_empty() || view.is_empty() {
            return None;
        }
        Some(Self {
            workspace,
            view,
            record: record.filter(|r| !r.is_empty()),
        })
    }
}

#[cfg(test)]
mod tests {

    /// Most views a workspace may have: with the menu they fill the five places of the
    /// bottom bar on phones (ADR-0049).
    const MAX_VIEWS: usize = 4;

    // [utest->dsn~compact-navigation~2]
    #[test]
    fn every_workspace_fits_the_bottom_bar_and_every_view_has_an_icon() {
        for w in WORKSPACES {
            assert!(
                w.views.len() <= MAX_VIEWS,
                "{}: {} views",
                w.id,
                w.views.len()
            );
            for v in w.views {
                assert!(!v.icon.is_empty(), "{}/{} has no icon", w.id, v.id);
            }
        }
    }
    use super::*;

    #[test]
    fn routes_survive_the_url_path() {
        let route = Route::record("med.uhs", "patients", "DE 0042/ä");
        assert_eq!(route.to_path(), "/med.uhs/patients/DE%200042%2F%C3%A4");
        assert_eq!(Route::from_path(&route.to_path()), Some(route));
        let view = Route::view("command.prepare", "units");
        assert_eq!(Route::from_path(&view.to_path()), Some(view));
    }

    #[test]
    fn other_paths_are_not_routes() {
        assert_eq!(Route::from_path(""), None);
        assert_eq!(Route::from_path("/"), None);
        assert_eq!(Route::from_path("/med.uhs"), None);
        assert_eq!(Route::from_path("/a/b/c/d"), None);
        assert_eq!(Route::from_path("/a/b/%FF"), None);
    }

    #[test]
    fn every_workspace_has_views() {
        for w in WORKSPACES {
            assert!(!w.views.is_empty(), "{}", w.id);
        }
        assert_eq!(packages().len(), 3);
    }
}
