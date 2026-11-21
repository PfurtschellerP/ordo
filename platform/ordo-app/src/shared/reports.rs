//! Reports (REP-1…5, ADR-0081): what reaches the incident command is an Auftrag of its own
//! ([`crate::shared::tasks`]). This module keeps what the Aufträge and the crews share: the reports
//! of incidents recorded before ADR-0081 (module `reports`, ADR-0075), the link for the
//! device's maps app, and the patient Auftrag handed to the registration.

use leptos::prelude::*;
use ordo_reports::{Recorded, ReportKind, ReportList};

use crate::core::Handle;
use crate::shared::text;

/// The incident's reports of the module `reports` (ADR-0075), computed from the shared
/// events.
pub fn list(core: Handle) -> ReportList {
    core.with_value(|c| ReportList::from_events(&c.events(ordo_reports::COMPARTMENT)))
}

/// The kind of an Auftrag for a report's kind.
pub fn task_kind(kind: ReportKind) -> ordo_tasks::TaskKind {
    match kind {
        ReportKind::Event => ordo_tasks::TaskKind::Event,
        ReportKind::Patient => ordo_tasks::TaskKind::Patient,
    }
}

/// A report in a few words, e.g. "Patient: Bühne Nord", for lists and the map.
pub fn label(r: &Recorded) -> String {
    text::task_label(task_kind(r.kind), &r.place)
}

/// A `geo:` URI (RFC 5870) that opens the device's maps app at a point.
pub fn geo_uri([lon, lat]: [f64; 2]) -> String {
    format!("geo:{lat:.6},{lon:.6}")
}

/// An Apple Maps link to a point: iOS and iPadOS do not open `geo:` URIs, but open this
/// one in their maps app.
pub fn apple_maps_uri([lon, lat]: [f64; 2]) -> String {
    format!("https://maps.apple.com/?ll={lat:.6},{lon:.6}&q={lat:.6},{lon:.6}")
}

/// The link that opens this device's maps app at a point: Apple Maps on Apple devices
/// (an iPad also says "Mac"), `geo:` everywhere else.
pub fn maps_app_uri(at: [f64; 2]) -> String {
    let apple = web_sys::window()
        .and_then(|w| w.navigator().platform().ok())
        .is_some_and(|p| p.starts_with("Mac") || p.starts_with("iP"));
    if apple {
        apple_maps_uri(at)
    } else {
        geo_uri(at)
    }
}

/// The patient Auftrag being registered from, shared by the crew's card and the
/// registration form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    /// The Auftrag.
    pub task: String,
    /// The unit whose crew takes the patient over.
    pub unit: String,
    /// The Auftrag's place, to say which one it is.
    pub place: String,
}

/// The case the registration form registers from, if any (context of *Patienten*).
#[derive(Clone, Copy)]
pub struct ActiveCase(pub RwSignal<Option<Case>>);

thread_local! {
    /// A case chosen in the crew's workspace, for the registration in *UHS – Patienten*
    /// that opens next (ADR-0079).
    static PENDING_CASE: std::cell::RefCell<Option<Case>> = const { std::cell::RefCell::new(None) };
}

/// Hands a case to the registration that opens next.
pub fn set_pending_case(case: Case) {
    PENDING_CASE.with_borrow_mut(|p| *p = Some(case));
}

/// The case chosen for the registration that opens now, taken once.
pub fn take_pending_case() -> Option<Case> {
    PENDING_CASE.with_borrow_mut(Option::take)
}

// [impl->dsn~reports~3]
/// The UHS to offer first for a unit's patient: one under the unit's position in the command
/// structure, else under the nearest position above it.
pub fn unit_uhs(structure: &ordo_command::CommandStructure, unit: &str) -> Option<String> {
    let mut position = structure.unit_positions.get(unit)?;
    for _ in 0..structure.positions.len() {
        let p = structure.positions.get(position)?;
        if let Some(uhs) = p.uhs.first() {
            return Some(uhs.clone());
        }
        position = p.created.parent.as_ref()?;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~reports~3]
    #[test]
    fn a_geo_uri_names_latitude_first() {
        assert_eq!(geo_uri([8.2473, 49.9929]), "geo:49.992900,8.247300");
        assert_eq!(
            apple_maps_uri([8.2473, 49.9929]),
            "https://maps.apple.com/?ll=49.992900,8.247300&q=49.992900,8.247300"
        );
    }
}
