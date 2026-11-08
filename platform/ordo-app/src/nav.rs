//! Workspaces and views (ADR-0011): which role sees which views.
//!
//! A workspace belongs to a service package and needs read access to certain
//! compartments. Every workspace of a package the organisation uses appears in the
//! launcher; without access it stays in place, disabled, with the reason — so the
//! layout is the same for every role. Packages that are not used do not appear.
//!
//! Prototype: the definitions are Rust constants; later they come from package and
//! profile data, so organisations can adapt them without code.

use crate::text;

/// A view within a workspace.
pub struct View {
    /// Stable ID, e.g. `patients`.
    pub id: &'static str,
    /// Label.
    pub label: &'static str,
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
    /// Views, in navigation order. The first is the start view.
    pub views: &'static [View],
}

/// All workspaces of the packages in use, in launcher order.
pub const WORKSPACES: &[Workspace] = &[
    Workspace {
        id: "med.uhs",
        package: text::TAB_MED,
        label: "UHS",
        requires: &["incident", "med.clinical"],
        views: &[
            View {
                id: "patients",
                label: "Patienten",
            },
            View {
                id: "uhs",
                label: "UHS verwalten",
            },
        ],
    },
    Workspace {
        id: "command.situation",
        package: text::TAB_COMMAND,
        label: "Lage",
        requires: &["incident"],
        views: &[
            View {
                id: "counts",
                label: "Patientenzahlen",
            },
            View {
                id: "units",
                label: text::TAB_RESOURCES,
            },
        ],
    },
];

/// The workspace with `id`.
pub fn workspace(id: &str) -> Option<&'static Workspace> {
    WORKSPACES.iter().find(|w| w.id == id)
}

/// Packages in launcher order, each with its workspaces.
pub fn packages() -> Vec<(&'static str, Vec<&'static Workspace>)> {
    let mut packages: Vec<(&'static str, Vec<&'static Workspace>)> = Vec::new();
    for w in WORKSPACES {
        match packages.iter_mut().find(|(p, _)| *p == w.package) {
            Some((_, list)) => list.push(w),
            None => packages.push((w.package, vec![w])),
        }
    }
    packages
}
