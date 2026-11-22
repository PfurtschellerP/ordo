//! Workspaces and views (ADR-0011): which role sees which views.
//!
//! A workspace belongs to a service package and needs read access to certain
//! compartments. Every workspace of a package the organisation uses appears in the
//! launcher; without access it stays in place, disabled, with the reason — so the
//! layout is the same for every role. Packages that are not used do not appear.
//!
//! Each package declares its workspaces as data in its entry of the registry
//! ([`crate::shell::registry`], ADR-0011); profiles may adapt them later.

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, percent_decode_str, utf8_percent_encode};

/// A text shown in the navigation: a message of the app's texts (ADR-0096), read when
/// shown, so workspaces and views can stay `const` data.
pub type Label = fn() -> &'static str;

/// A view within a workspace.
pub struct View {
    /// Stable ID, e.g. `patients`.
    pub id: &'static str,
    /// Label.
    pub label: Label,
    /// A shorter label for the phone's bottom bar, where a long one would take two lines
    /// (feedback of 2026-11-22).
    pub short: Option<Label>,
    /// English name, found by the command palette in every language (ADR-0036).
    pub name: &'static str,
    /// Icon (Lucide name), shown with the label in the rail and the bottom bar.
    pub icon: &'static str,
    /// A permission this view needs beyond its workspace's, given by the roles in this
    /// incident or by an organisation role (e.g. `manage_accounts`); without it, the view
    /// is not offered.
    pub permission: Option<&'static str>,
    /// Compartments the person must read for this view beyond its workspace's (e.g.
    /// `med.clinical` for the crew's patients).
    pub requires: &'static [&'static str],
    /// Tabs of a page with several parts (empty: one page); each section goes to the last
    /// tab whose `from` is not above its place.
    pub tabs: &'static [PageTab],
}

// [impl->dsn~page-tabs~2]
/// A tab of a page: its key, its label, and the first place of the sections it holds.
pub struct PageTab {
    /// Stable key, e.g. `helpers`.
    pub key: &'static str,
    /// Label.
    pub label: Label,
    /// The lowest section place it holds.
    pub from: u8,
}

/// A workspace: the environment of one role.
pub struct Workspace {
    /// Stable ID, e.g. `med.uhs`.
    pub id: &'static str,
    /// Service package, shown as group in the launcher.
    pub package: Label,
    /// Label within the package.
    pub label: Label,
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

pub(crate) const fn view(
    id: &'static str,
    label: Label,
    name: &'static str,
    icon: &'static str,
) -> View {
    View {
        id,
        label,
        short: None,
        name,
        icon,
        permission: None,
        requires: &[],
        tabs: &[],
    }
}

/// The workspace with `id`.
pub fn workspace(id: &str) -> Option<&'static Workspace> {
    workspaces().find(|w| w.id == id)
}

// [impl->dsn~package-registry~1]
/// Every workspace of the registered packages, in launcher order (ADR-0011, ADR-0061).
pub fn workspaces() -> impl Iterator<Item = &'static Workspace> {
    crate::shell::registry::PACKAGES
        .iter()
        .flat_map(|p| p.workspaces.iter())
}

/// Packages in launcher order, each with its workspaces.
pub fn packages() -> Vec<(&'static str, Vec<&'static Workspace>)> {
    let mut packages: Vec<(&'static str, Vec<&'static Workspace>)> = Vec::new();
    for w in workspaces().filter(|w| !w.hidden) {
        match packages.iter_mut().find(|(p, _)| *p == (w.package)()) {
            Some((_, list)) => list.push(w),
            None => packages.push(((w.package)(), vec![w])),
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

/// The first part of a patient's own address, `/patient/<tag>` (ADR-0093): the record opens
/// in the workspace the person works in, if it shows patients, else in the first one that
/// does.
pub const PATIENT: &str = "patient";

impl Route {
    // [impl->dsn~patient-address~1]
    /// A patient's own address, `/patient/<tag>`.
    pub fn patient(tag: &str) -> Self {
        Self::record(PATIENT, "patients", tag)
    }

    /// Whether this is a patient's own address, still to be placed in a workspace.
    pub fn is_patient(&self) -> bool {
        self.workspace == PATIENT
    }

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
        if let (true, Some(tag)) = (self.is_patient(), &self.record) {
            return format!("/{PATIENT}/{}", segment(tag));
        }
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
        if let Some(tag) = path.strip_prefix("/patient/") {
            let tag = decode(tag)?;
            return (!tag.is_empty() && !tag.contains('/')).then(|| Self::patient(&tag));
        }
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
        for w in workspaces() {
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

    // [utest->dsn~patient-address~1]
    #[test]
    fn a_patient_has_an_address_of_its_own() {
        let route = Route::patient("DE 0042");
        assert_eq!(route.to_path(), "/patient/DE%200042");
        assert_eq!(Route::from_path("/patient/DE%200042"), Some(route.clone()));
        assert!(route.is_patient());
        assert_eq!(route.record.as_deref(), Some("DE 0042"));
        assert_eq!(Route::from_path("/patient/"), None);
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
        for w in workspaces() {
            assert!(!w.views.is_empty(), "{}", w.id);
        }
        assert_eq!(
            packages().len(),
            3,
            "Sanität, Einsatzleitung, Administration"
        );
    }
}
