//! User-visible texts of the Med package (ADR-0096). The messages are in
//! `locales/<language>/med.ftl`; the build writes one function per message into this
//! module. What maps ORDO's values (triage categories, statuses, needs) to messages, and
//! joins lists, stays here. It re-exports the shared modules' texts, so the package writes
//! `text::…` for all it may use.

pub use crate::shared::text::*;
use ordo_contracts::TriageCategory;

// [impl->dsn~ui-texts~1]
include!(concat!(env!("OUT_DIR"), "/text_med.rs"));

/// Spoken description, so screen readers convey the meaning, not only the label.
pub fn description(category: TriageCategory) -> &'static str {
    match category {
        TriageCategory::Immediate => triage_description_immediate(),
        TriageCategory::Urgent => triage_description_urgent(),
        TriageCategory::Minor => triage_description_minor(),
        TriageCategory::Expectant => triage_description_expectant(),
        TriageCategory::Deceased => triage_description_deceased(),
        TriageCategory::Uninjured => triage_description_uninjured(),
    }
}

pub fn retriage_label(tag: &str, category: TriageCategory) -> String {
    retriage_of(tag, label(category))
}

pub fn edit_label(form: &str) -> String {
    edit_form(form, edit().to_lowercase())
}

pub fn value_error(error: &ordo_med_patients::record::ValueError) -> String {
    use ordo_med_patients::record::ValueError as E;
    match error {
        E::NotANumber => value_not_a_number().into(),
        E::OutOfRange { min, max } => match (min, max) {
            (Some(min), Some(max)) => value_between(min.to_string(), max.to_string()),
            (Some(min), None) => value_at_least(min.to_string()),
            (None, Some(max)) => value_at_most(max.to_string()),
            (None, None) => value_impossible().into(),
        },
        E::NotADate => value_not_a_date().into(),
        E::NotATime => value_not_a_time().into(),
        E::TooLong => value_too_long().into(),
        E::UnknownField | E::UnknownOption(_) | E::WrongKind => value_impossible().into(),
    }
}

pub fn action_retriage(category: TriageCategory) -> String {
    action_retriage_to(label(category))
}

/// "7 von 10 belegt · 3 frei", or "Patienten" without a capacity.
pub fn uhs_load(patients: u32, capacity: u32) -> String {
    if capacity == 0 {
        uhs_load_patients(patients)
    } else {
        uhs_load_of(capacity, capacity.saturating_sub(patients))
    }
}

/// How a UHS's capacity is split, as typed.
pub fn uhs_places_split(kept: u32, total: u32) -> String {
    if kept > total {
        uhs_places_over(kept, total)
    } else {
        uhs_places_within(kept, total - kept)
    }
}

/// The text of a category's strip of places, e.g. "SK II: 4 von 4 belegt, voll".
pub fn place_strip(category: &str, taken: usize, total: usize) -> String {
    let free = total.saturating_sub(taken);
    if free == 0 {
        place_strip_full(category, taken, total)
    } else {
        place_strip_free(category, taken, total, free)
    }
}

/// Button text that moves a transport to `status`.
pub fn transport_step(status: ordo_med_patients::transport::TransportStatus) -> &'static str {
    use ordo_med_patients::transport::TransportStatus as S;
    match status {
        S::Requested => transport_step_requested(),
        S::Assigned => transport_step_assign(),
        S::Loading => transport_step_loading(),
        S::Departed => transport_step_departed(),
        S::Arrived => transport_step_arrived(),
        S::HandedOver => transport_step_handed_over(),
        S::Cancelled => transport_step_cancel(),
    }
}

/// Button text that moves an order to `status`.
pub fn order_step(status: ordo_med_patients::orders::OrderStatus) -> &'static str {
    use ordo_med_patients::orders::OrderStatus as S;
    match status {
        S::Given => order_step_give(),
        S::Acknowledged => order_step_acknowledge(),
        S::Done => order_step_done(),
        S::Withdrawn => order_step_withdraw(),
    }
}

pub fn order_channel(channel: ordo_med_patients::orders::Channel) -> &'static str {
    use ordo_med_patients::orders::Channel as C;
    match channel {
        C::InPerson => order_channel_in_person(),
        C::Radio => order_channel_radio(),
        C::Phone => order_channel_phone(),
        C::Telemedicine => order_channel_telemedicine(),
    }
}

/// "Mündlich angeordnet von Petra Schulz, per Funk, wiederholt und bestätigt"
pub fn order_spoken(giver: &str, channel: &str, read_back: bool) -> String {
    if read_back {
        order_spoken_read_back(giver, channel)
    } else {
        order_spoken_not_read_back(giver, channel)
    }
}

pub fn readmit_label(tag: &str) -> String {
    readmit_of(tag, readmit().to_lowercase())
}

/// "Aufnahme: SK I 2 · SK II 4 · SK III – (Stand 10:12, ben@…)"
pub fn capacity_line(parts: &[String], at: &str, by: &str) -> String {
    capacity_of(parts.join(" · "), at, by)
}

/// "Offene Anordnungen: 3 (1 nicht bestätigt)"
pub fn open_orders(n: usize, unconfirmed: usize) -> String {
    if unconfirmed == 0 {
        open_orders_count(n)
    } else {
        open_orders_unconfirmed(n, unconfirmed)
    }
}

pub fn triage_stage(stage: ordo_med_patients::triage::Stage) -> &'static str {
    match stage {
        ordo_med_patients::triage::Stage::Pretriage => triage_stage_pretriage(),
        ordo_med_patients::triage::Stage::Triage => triage_stage_triage(),
    }
}

pub fn handover_no_target(transport: bool) -> &'static str {
    if transport {
        handover_no_transport()
    } else {
        handover_no_other()
    }
}

pub fn staff_need(need: ordo_med_patients::staffing::Need) -> &'static str {
    match need {
        ordo_med_patients::staffing::Need::Physician => staff_need_physician(),
        ordo_med_patients::staffing::Need::Paramedic => staff_need_paramedic(),
    }
}

pub fn staff_request_button(need: ordo_med_patients::staffing::Need) -> String {
    staff_request_of(staff_need(need))
}

pub fn staff_requested(uhs: &str, need: ordo_med_patients::staffing::Need) -> String {
    staff_requested_of(uhs, staff_need(need))
}

pub fn staff_request_label(uhs: &str, need: ordo_med_patients::staffing::Need) -> String {
    staff_requested(uhs, need)
}

/// The Auftrag of the person sent after a staff request (UHS-15).
pub fn staff_task(uhs: &str, need: ordo_med_patients::staffing::Need) -> String {
    staff_task_of(uhs, staff_need(need))
}

/// "Patientenzahlen UHS Eingang: 5 (SK I 1 · SK III 4), 1 ohne Platz, 2 entlassen".
pub fn log_counts(uhs: &str, counts: &[(&str, u32)], unplaced: u32, discharged: u32) -> String {
    let total: u32 = counts.iter().map(|(_, n)| n).sum();
    let mut out = log_counts_total(uhs, total);
    let parts: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(label, n)| format!("{label} {n}"))
        .collect();
    if !parts.is_empty() {
        out.push_str(&format!(" ({})", parts.join(" · ")));
    }
    if unplaced > 0 {
        out.push_str(&format!(", {}", without_place(unplaced)));
    }
    if discharged > 0 {
        out.push_str(&format!(", {}", log_counts_discharged(discharged)));
    }
    out
}

pub fn log_transport_updated(transport: &str, status: &str, unit: Option<&str>) -> String {
    match unit {
        Some(u) => log_transport_updated_by(transport, status, u),
        None => log_transport_updated_to(transport, status),
    }
}

#[cfg(test)]
mod tests {
    // [utest->dsn~ui-texts~1]
    #[test]
    fn texts_by_hand_read_as_before() {
        use super::*;
        assert_eq!(triage_count(1usize), "1 Sichtung");
        assert_eq!(triage_count(3usize), "3 Sichtungen");
        assert_eq!(uhs_load(1, 0), "Patient");
        assert_eq!(uhs_load(7, 10), "von 10 belegt · 3 frei");
        assert_eq!(
            uhs_places_split(11, 5),
            "Zurzeit 11 Plätze nach Kategorie: mehr als die Kapazität von 5."
        );
        assert_eq!(
            uhs_places_split(4, 5),
            "Zurzeit 4 nach Kategorie, 1 ohne Kategorie."
        );
        assert_eq!(place_strip("SK II", 4, 4), "SK II: 4 von 4 belegt, voll");
        assert_eq!(
            open_orders(3, 1),
            "Offene Anordnungen: 3 (1 nicht bestätigt)"
        );
        assert_eq!(uhs_staff_count(0usize), "Kein Personal eingetragen.");
        assert_eq!(
            uhs_close_blocked(1u32),
            "1 Patient ist noch hier. Erst übergeben oder entlassen, dann auflösen."
        );
        assert_eq!(
            log_counts("UHS Eingang", &[("SK I", 1), ("SK III", 4)], 1, 2),
            "Patientenzahlen UHS Eingang: 5 (SK I 1 · SK III 4), 1 ohne Platz, 2 entlassen"
        );
        assert_eq!(
            staff_task("UHS Eingang", ordo_med_patients::staffing::Need::Paramedic),
            "Als Notfallsanitäter/in zu UHS Eingang"
        );
        assert_eq!(
            order_spoken("Petra Schulz", "per Funk", true),
            "Mündlich angeordnet von Petra Schulz, per Funk, wiederholt und bestätigt"
        );
    }
}
