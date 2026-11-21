//! The app's package registry (ADR-0010, ADR-0011): every service package, and the shared
//! modules and the administration alike, describe themselves in one [`Package`]. The shell
//! builds the launcher, the navigation and each view's page from them and names no package
//! anywhere else. A new package adds its entry to [`PACKAGES`] and changes no shared file.

use leptos::prelude::*;

use crate::core::Handle;
use crate::shell::nav::Workspace;
use crate::shell::views::Nav;

/// What a section of a page gets: the device's state, the navigation and the record in the
/// URL, if any (e.g. a patient's tag).
#[derive(Clone, Copy)]
pub struct PageArgs {
    pub core: Handle,
    pub revision: RwSignal<u64>,
    pub nav: Nav,
    pub record: Memo<Option<String>>,
}

/// One part of a page: its place (lower first) and its view. Several packages may add to
/// the same page, e.g. the staff requests of the Med package above the shared Aufträge.
pub type Section = (u8, AnyView);

// [impl->dsn~package-registry~1]
/// One package as the app shows it.
pub struct Package {
    /// Its workspaces, in launcher order.
    pub workspaces: &'static [Workspace],
    /// Gives the workspace what the package keeps across views (e.g. unsaved drafts).
    pub provide: fn(),
    /// Its sections of the page of `view` in `workspace` (any package's).
    pub sections: fn(&PageArgs, &str, &str) -> Vec<Section>,
    /// What needs the person's attention now, for the bell: the text and where it is dealt
    /// with.
    pub bell: fn(Handle) -> Vec<(String, crate::shell::nav::Route)>,
    /// Its entries in the command palette.
    pub palette: crate::shell::palette::PalettePart,
    /// Its events in the incident log.
    pub log: crate::shared::log::LogPart,
    /// Its building blocks in the component gallery (ADR-0094).
    pub gallery: &'static [crate::shell::gallery::Part],
}

/// The packages the app is built with: the one place that names them. Their order is the
/// launcher's.
pub const PACKAGES: &[&Package] = &[
    &crate::med::PACKAGE,
    &crate::shared::PACKAGE,
    &crate::admin::PACKAGE,
];

/// Gives the workspace what every package keeps across views.
pub fn provide() {
    for package in PACKAGES {
        (package.provide)();
    }
}

/// The page of `view` in `workspace`: every package's sections, in their places.
pub fn page(args: &PageArgs, workspace: &str, view: &str) -> Vec<Section> {
    let mut sections: Vec<Section> = PACKAGES
        .iter()
        .flat_map(|p| (p.sections)(args, workspace, view))
        .collect();
    sections.sort_by_key(|(place, _)| *place);
    sections
}

// [impl->dsn~page-tabs~1]
/// The tab of `tabs` a section at `place` belongs to: the last one starting at or before.
pub fn tab_of(tabs: &[crate::shell::nav::PageTab], place: u8) -> Option<&'static str> {
    tabs.iter()
        .take_while(|t| t.from <= place)
        .last()
        .or(tabs.first())
        .map(|t| t.key)
}
