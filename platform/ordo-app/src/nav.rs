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

/// Where the user is: workspace, view and optionally one record (e.g. a patient). It is
/// kept in the URL fragment (`#/med.uhs/patients/DE-0042`), so the browser's and the
/// phone's back button work and a view can be reopened after a reload.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Route {
    /// Workspace ID.
    pub workspace: String,
    /// View ID.
    pub view: String,
    /// Record within the view, e.g. a triage tag number.
    pub record: Option<String>,
}

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

    /// The URL fragment, with `#`.
    pub fn to_hash(&self) -> String {
        let mut hash = format!("#/{}/{}", encode(&self.workspace), encode(&self.view));
        if let Some(record) = &self.record {
            hash.push('/');
            hash.push_str(&encode(record));
        }
        hash
    }

    /// Reads a URL fragment; `None` if it is not a route.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let path = hash.strip_prefix('#').unwrap_or(hash).strip_prefix('/')?;
        let mut parts = path.split('/');
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

/// Percent-encodes everything except unreserved characters (RFC 3986).
fn encode(text: &str) -> String {
    text.bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
                (b as char).to_string()
            } else {
                format!("%{b:02X}")
            }
        })
        .collect()
}

fn decode(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = text.get(i + 1..i + 3)?;
            out.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_survive_the_url_fragment() {
        let route = Route::record("med.uhs", "patients", "DE 0042/ä");
        assert_eq!(route.to_hash(), "#/med.uhs/patients/DE%200042%2F%C3%A4");
        assert_eq!(Route::from_hash(&route.to_hash()), Some(route));
        let view = Route::view("command.situation", "units");
        assert_eq!(Route::from_hash(&view.to_hash()), Some(view));
    }

    #[test]
    fn other_fragments_are_not_routes() {
        assert_eq!(Route::from_hash(""), None);
        assert_eq!(Route::from_hash("#main"), None);
        assert_eq!(Route::from_hash("#/med.uhs"), None);
        assert_eq!(Route::from_hash("#/a/b/c/d"), None);
        assert_eq!(Route::from_hash("#/a/b/%zz"), None);
    }

    #[test]
    fn every_workspace_has_views() {
        for w in WORKSPACES {
            assert!(!w.views.is_empty(), "{}", w.id);
        }
        assert_eq!(packages().len(), 2);
    }
}
