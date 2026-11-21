//! Triage categories as the app shows them (A11Y-3): colour, label and shape, never colour
//! alone.

use leptos::prelude::*;
use ordo_contracts::TriageCategory::{
    self, Deceased, Expectant, Immediate, Minor, Uninjured, Urgent,
};

use crate::med::text;

/// Triage categories in display order.
pub(crate) const CATEGORIES: [TriageCategory; 6] =
    [Immediate, Urgent, Minor, Expectant, Deceased, Uninjured];

fn css_class(category: TriageCategory) -> &'static str {
    match category {
        Immediate => "triage triage--immediate",
        Urgent => "triage triage--urgent",
        Minor => "triage triage--minor",
        Expectant => "triage triage--expectant",
        Deceased => "triage triage--deceased",
        Uninjured => "triage triage--uninjured",
    }
}

/// The category's key in class names, e.g. `immediate`.
pub(crate) fn css_key(category: TriageCategory) -> &'static str {
    match category {
        Immediate => "immediate",
        Urgent => "urgent",
        Minor => "minor",
        Expectant => "expectant",
        Deceased => "deceased",
        Uninjured => "uninjured",
    }
}

// [impl->dsn~triage-badge~1]
/// Colour plus label plus shape, never colour alone (A11Y-3).
#[component]
pub(crate) fn TriageBadge(category: TriageCategory) -> impl IntoView {
    view! {
        <span class=css_class(category) title=text::description(category)>
            <span class="triage__shape" aria-hidden="true"></span>
            <span aria-hidden="true">{text::label(category)}</span>
            <span class="visually-hidden">{text::description(category)}</span>
        </span>
    }
}
