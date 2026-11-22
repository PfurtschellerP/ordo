//! User-visible texts of the shell (ADR-0096). The messages are in
//! `locales/<language>/shell.ftl`; the build writes one function per message into this
//! module. What maps ORDO's values (triage categories, states, durations) to messages stays
//! here. Code and identifiers stay English.

use std::sync::LazyLock;

use ordo_contracts::TriageCategory;

// [impl->dsn~ui-texts~1]
include!(concat!(env!("OUT_DIR"), "/text_shell.rs"));

pub fn registered(tag: &str, category: TriageCategory, uhs: &str) -> String {
    patient_registered(tag, label(category), uhs)
}

pub fn status_set(unit: &str, status: &str) -> String {
    unit_status_set(unit, status_label(status))
}

pub fn status_label(code: &str) -> String {
    crate::shared::units::STATUS.get(code).map_or_else(
        || code.to_owned(),
        |c| status_with_code(c.code.as_str(), c.label.as_str()),
    )
}

pub fn retriaged(tag: &str, category: TriageCategory) -> String {
    patient_retriaged(tag, label(category))
}

/// Label of a triage category in the DE profile.
pub fn label(category: TriageCategory) -> &'static str {
    match category {
        TriageCategory::Immediate => triage_immediate(),
        TriageCategory::Urgent => triage_urgent(),
        TriageCategory::Minor => triage_minor(),
        TriageCategory::Expectant => triage_expectant(),
        TriageCategory::Deceased => triage_deceased(),
        TriageCategory::Uninjured => triage_uninjured(),
    }
}

/// The themes: (stored value, label).
pub static THEMES: LazyLock<[(&str, &str); 4]> = LazyLock::new(|| {
    [
        ("", theme_auto()),
        ("day", theme_day()),
        ("night", theme_night()),
        ("high-contrast", theme_high_contrast()),
    ]
});
pub static HANDS: LazyLock<[(&str, &str); 2]> =
    LazyLock::new(|| [("", hand_right()), ("left", hand_left())]);
pub static KEYMAPS: LazyLock<[(&str, &str); 2]> =
    LazyLock::new(|| [("", keymap_standard()), ("vim", keymap_vim())]);

/// Roles offered when granting (prototype; later from the profile).
pub static ROLES: LazyLock<[(&str, &str); 4]> = LazyLock::new(|| {
    [
        ("commander", role_commander()),
        ("medic", role_medic()),
        ("physician", role_physician()),
        ("paramedic", role_paramedic()),
    ]
});

pub fn role_label(role: &str) -> String {
    ROLES
        .iter()
        .find(|(id, _)| *id == role)
        .map_or_else(|| role.to_owned(), |(_, label)| (*label).to_owned())
}

pub fn incident_paused(paused: bool) -> &'static str {
    if paused {
        incident_now_paused()
    } else {
        incident_now_resumed()
    }
}

pub fn state_label(state: ordo_sync::incidents::IncidentState) -> &'static str {
    use ordo_sync::incidents::IncidentState::*;
    match state {
        Planned => incident_planned(),
        Active => incident_active(),
        Closed => incident_closed(),
    }
}

pub fn role_granted(user: &str, role: &str) -> String {
    role_granted_to(user, role_label(role))
}

pub fn role_withdrawn(user: &str, role: &str) -> String {
    role_withdrawn_from(user, role_label(role))
}

pub fn incident_state_changed(state: ordo_sync::incidents::IncidentState) -> String {
    incident_state_changed_to(state_label(state))
}

pub fn priority_changed(tag: &str, set: bool) -> String {
    if set {
        priority_set(tag)
    } else {
        priority_removed(tag)
    }
}

/// A place kept for a triage category, e.g. "SK I – Platz 2" (UHS-13).
pub fn category_place_label(category: TriageCategory, n: u32) -> String {
    category_place(label(category), n)
}

pub fn place_assigned(tag: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => place_assigned_to(tag, l),
        None => place_none(tag),
    }
}

pub fn transport_status(status: ordo_med_patients::transport::TransportStatus) -> &'static str {
    use ordo_med_patients::transport::TransportStatus as S;
    match status {
        S::Requested => transport_status_requested(),
        S::Assigned => transport_status_assigned(),
        S::Loading => transport_status_loading(),
        S::Departed => transport_status_departed(),
        S::Arrived => transport_status_arrived(),
        S::HandedOver => transport_status_handed_over(),
        S::Cancelled => transport_status_cancelled(),
    }
}

pub fn transport_status_set(status: ordo_med_patients::transport::TransportStatus) -> String {
    transport_status_set_to(transport_status(status))
}

pub fn order_status(status: ordo_med_patients::orders::OrderStatus) -> &'static str {
    use ordo_med_patients::orders::OrderStatus as S;
    match status {
        S::Given => order_status_given(),
        S::Acknowledged => order_status_acknowledged(),
        S::Done => order_status_done(),
        S::Withdrawn => order_status_withdrawn(),
    }
}

pub fn order_status_set(status: ordo_med_patients::orders::OrderStatus) -> String {
    order_status_set_to(order_status(status))
}

pub fn outcome(outcome: ordo_med_patients::Outcome) -> &'static str {
    use ordo_med_patients::Outcome as O;
    match outcome {
        O::ReleasedOnSite => outcome_released_on_site(),
        O::Transported => outcome_transported(),
        O::Refused => outcome_refused(),
        O::Deceased => outcome_deceased(),
    }
}

/// "Entlassen: Transportiert"
pub fn discharged_as(outcome: ordo_med_patients::Outcome) -> String {
    discharged_as_outcome(self::outcome(outcome))
}

pub fn discharged(tag: &str, outcome: ordo_med_patients::Outcome) -> String {
    discharged_with(tag, discharged_as(outcome))
}

/// How long ago, from milliseconds: "gerade eben", "vor 5 Min.", "vor 2 Std. 5 Min.",
/// "vor 3 Tagen".
pub fn ago(ms: u64) -> String {
    let minutes = ms / 60_000;
    match minutes {
        0 => ago_now().to_owned(),
        1..=59 => ago_minutes(minutes),
        60..=1439 => match minutes % 60 {
            0 => ago_hours(minutes / 60),
            m => ago_hours_minutes(minutes / 60, m),
        },
        _ => ago_days(minutes / 1440),
    }
}

pub fn task_status(status: ordo_tasks::TaskStatus) -> &'static str {
    use ordo_tasks::TaskStatus as S;
    match status {
        S::Open => task_status_open(),
        S::InProgress => task_status_in_progress(),
        S::Done => task_status_done(),
        S::Cancelled => task_status_cancelled(),
    }
}

pub fn task_status_set(status: ordo_tasks::TaskStatus) -> String {
    task_status_set_to(task_status(status))
}

#[cfg(test)]
mod tests {
    // [utest->dsn~record-overview~3]
    #[test]
    fn ages_read_at_a_glance() {
        let min = 60_000;
        assert_eq!(super::ago(30_000), "gerade eben");
        assert_eq!(super::ago(59 * min), "vor 59 Min.");
        assert_eq!(super::ago(60 * min), "vor 1 Std.");
        assert_eq!(super::ago(125 * min), "vor 2 Std. 5 Min.");
        assert_eq!(super::ago(1440 * min), "vor 1 Tag");
        assert_eq!(super::ago(3 * 1440 * min), "vor 3 Tagen");
    }

    // [utest->dsn~ui-texts~1]
    #[test]
    fn counts_choose_their_form() {
        assert_eq!(super::pending(0usize), "Alles gesendet");
        assert_eq!(super::pending(1usize), "1 Eingabe noch nicht gesendet");
        assert_eq!(super::pending(3usize), "3 Eingaben noch nicht gesendet");
        assert_eq!(super::attention_label(0usize), "Hinweise: keine");
        assert_eq!(super::close_label("Verlauf"), "Schließen: Verlauf");
    }
}
