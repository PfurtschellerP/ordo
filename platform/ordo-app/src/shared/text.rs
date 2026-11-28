//! User-visible texts of the shared modules (ADR-0096). The messages are in
//! `locales/<language>/shared.ftl`; the build writes one function per message into this
//! module. What maps ORDO's values (states, strengths, kinds) to messages, and joins lists,
//! stays here. It re-exports the shell's texts, so a module writes `text::…` for all it
//! may use.

pub use crate::text::*;

// [impl->dsn~ui-texts~3]
include!(concat!(env!("OUT_DIR"), "/text_shared.rs"));

/// People on more than one crew or staff list (UNIT-8), e.g. "Ben (RTW 1, UHS Süd)".
pub fn double_booked(people: &[String]) -> String {
    double_booked_people(people.join("; "))
}

/// The person being added is on other crews or staff lists already (UNIT-8).
pub fn assigned_elsewhere(person: &str, posts: &[String]) -> String {
    assigned_elsewhere_to(person, posts.join(", "))
}

pub fn lost_update(unit: &str, overwritten: &str, kept: &str) -> String {
    lost_update_of(unit, status_label(overwritten), status_label(kept))
}

pub fn action_status(code: &str) -> String {
    action_status_of(status_label(code))
}

/// The state as people read it, with a pause: "Laufend – unterbrochen".
pub fn incident_state(state: ordo_sync::incidents::IncidentState, paused: bool) -> String {
    if paused && state == ordo_sync::incidents::IncidentState::Active {
        incident_state_paused(state_label(state))
    } else {
        state_label(state).to_owned()
    }
}

/// "13.11.2026, 18:00 bis 15.11.2026, 23:00"
pub fn incident_times(starts: Option<&str>, ends: Option<&str>) -> Option<String> {
    match (starts, ends) {
        (Some(s), Some(e)) => Some(incident_from_to(s, e)),
        (Some(s), None) => Some(incident_from(s)),
        (None, Some(e)) => Some(incident_to(e)),
        (None, None) => None,
    }
}

pub fn withdraw_label(user: &str, role: &str) -> String {
    withdraw_role(user, role_label(role))
}

/// "0/1/8/9" read out: "Stärke: 0 Führer, 1 Unterführer, 8 Mannschaft, 9 gesamt".
/// The strength read out: each number with its function, the total, the special functions.
pub fn strength_label(s: &ordo_units::Strength, detailed: bool, special: Option<&str>) -> String {
    let parts = s.parts(detailed);
    let n = parts.len() - 1;
    let named: Vec<String> = parts[..n]
        .iter()
        .enumerate()
        .map(|(i, v)| {
            strength_count(
                *v,
                strength_part(if detailed { i } else { i + 1 }, detailed),
            )
        })
        .collect();
    let read = strength_read(named.join(", "), parts[n]);
    match special {
        Some(special) => strength_special(read, special),
        None => read,
    }
}

/// The name of a strength's part: 0 leaders of formations, 1 leaders, 2 sub-leaders,
/// 3 crew; the detailed notation (UNIT-7) says Zugführer and Einsatzkräfte.
pub fn strength_part(part: usize, detailed: bool) -> &'static str {
    match (part, detailed) {
        (0, _) => unit_leaders(),
        (1, true) => strength_platoon_leaders(),
        (1, false) => leaders(),
        (2, _) => sub_leaders(),
        (_, true) => strength_forces(),
        (_, false) => crew_count(),
    }
}

/// A strength's legend, e.g. "Stärke (Führer / Unterführer / Mannschaft)".
pub fn strength_legend(title: &str, detailed: bool) -> String {
    let parts: Vec<&str> = if detailed {
        (0..4).collect::<Vec<_>>()
    } else {
        (1..4).collect()
    }
    .into_iter()
    .map(|p| strength_part(p, detailed))
    .collect();
    strength_legend_of(title, parts.join(" / "))
}

pub fn log_report_recorded(r: &ordo_reports::Recorded) -> String {
    log_report_line(
        task_kind(crate::shared::reports::task_kind(r.kind)),
        task_received(crate::core::clock(r.received_at)),
        r.place.as_str(),
        r.text.as_str(),
    )
}

/// "Rolle Arzt/Ärztin von Anna Schmidt entzogen."
pub fn role_removed(role: &str, who: &str) -> String {
    role_removed_from(role_label(role), who)
}

pub fn task_kind(kind: ordo_tasks::TaskKind) -> &'static str {
    match kind {
        ordo_tasks::TaskKind::Event => task_kind_event(),
        ordo_tasks::TaskKind::Patient => task_kind_patient(),
    }
}

/// "Patient: Bühne Nord"
pub fn task_label(kind: ordo_tasks::TaskKind, place: &str) -> String {
    task_label_of(task_kind(kind), place)
}

/// How far an Auftrag is (TASK-2).
pub fn task_phase(phase: ordo_tasks::Phase) -> &'static str {
    use ordo_tasks::Phase as P;
    match phase {
        P::Received => task_phase_received(),
        P::Assigned => task_phase_assigned(),
        P::InProgress => task_phase_in_progress(),
        P::Done => task_phase_done(),
        P::Cancelled => task_phase_cancelled(),
    }
}

pub fn log_task_received(
    kind: ordo_tasks::TaskKind,
    at: &str,
    place: &str,
    task: &str,
    to: &str,
) -> String {
    let line = log_task_received_line(task_kind(kind), task_received(at), place, task);
    if to.is_empty() {
        line
    } else {
        log_task_received_to(line, to)
    }
}

pub fn task_step(status: ordo_tasks::TaskStatus) -> &'static str {
    use ordo_tasks::TaskStatus as S;
    match status {
        S::Open => task_step_open(),
        S::InProgress => task_step_in_progress(),
        S::Done => task_step_done(),
        S::Cancelled => task_step_cancel(),
    }
}

/// Time left until a follow-up, e.g. "noch 7:05 min"; "jetzt" at zero.
pub fn countdown(left_ms: u64) -> String {
    let secs = left_ms.div_ceil(1000);
    if secs == 0 {
        return countdown_now().to_owned();
    }
    countdown_left(secs / 60, format!("{:02}", secs % 60))
}

/// "Übergabe: Dr. M. → Dr. K." or "Leitung: Dr. M."
pub fn handover(from: Option<&str>, to: &str) -> String {
    match from {
        Some(from) => handover_from_to(from, to),
        None => leader(to),
    }
}

pub fn map_kind(kind: ordo_map::ObjectKind) -> &'static str {
    use ordo_map::ObjectKind as K;
    match kind {
        K::Incident => incident_location(),
        K::Unit => tab_resources(),
        K::Uhs => map_kind_uhs(),
        K::Position => position(),
        K::Drawing => map_kind_drawing(),
        K::Report => map_kind_report(),
        K::Task => task_text(),
        K::Symbol => map_kind_symbol(),
    }
}

/// A number with the language's decimal separator ("1,5" in German).
fn decimal(number: String) -> String {
    number.replace('.', decimal_separator())
}

/// Megabytes with one decimal ("1,5" in German).
fn megabytes(bytes: f64) -> String {
    decimal(format!("{:.1}", bytes / 1_000_000.0))
}

pub fn map_saved(bytes: f64) -> String {
    map_saved_size(megabytes(bytes))
}

/// The patient counts with units' crews: "Bei Einsatzmitteln: 2 (SK I 1 · SK II 1)".
pub fn with_units(total: u32, counts: &[(&str, u32)]) -> String {
    let parts: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(label, n)| format!("{label} {n}"))
        .collect();
    with_units_line(with_units_label(), total, parts.join(" · "))
}

pub fn log_unit_added(unit: &str, unit_type: &str, organisation: &str) -> String {
    log_unit_added_line(unit_added(unit), unit_type, organisation)
}

/// "Anna Schmidt: Dienstbeginn 14:00 · Angaben geändert: Telefon".
pub fn log_helper_updated(name: &str, changes: &[String]) -> String {
    if changes.is_empty() {
        return log_helper_saved(name);
    }
    log_helper_changes(name, changes.join(" · "))
}

pub fn log_fields_changed(fields: &[&str]) -> String {
    log_fields_changed_list(fields.join(", "))
}

/// A unit's step on an Auftrag (ADR-0085): "Auftrag „…“: RTW 2 In Arbeit".
pub fn log_unit_step(task: &str, unit: &str, status: Option<&str>) -> String {
    match status {
        Some(s) => log_unit_step_of(task, unit, s),
        None => log_task_updated(task, None),
    }
}

pub fn log_task_updated(task: &str, status: Option<&str>) -> String {
    match status {
        Some(s) => log_task_updated_to(task, s),
        None => log_task_follow_up_changed(task),
    }
}

pub fn log_assigned(what: &str, position: Option<&str>) -> String {
    match position {
        Some(p) => log_assigned_to(what, p),
        None => log_assigned_none(what),
    }
}

pub fn map_region_running(done: u64, total: u64) -> String {
    if total == 0 {
        map_region_preparing().to_owned()
    } else {
        map_region_tiles(done, total)
    }
}

pub fn map_region_ready(area: Option<(f64, f64, f64)>, bytes: f64, at: Option<String>) -> String {
    let mut out = map_region_ready_size(megabytes(bytes));
    if let Some((lat, lon, km)) = area {
        out.push(' ');
        out.push_str(&map_region_area(
            decimal(format!("{km}")),
            format!("{lat:.5}"),
            format!("{lon:.5}"),
        ));
    }
    if let Some(at) = at {
        out.push(' ');
        out.push_str(&map_region_created(at));
    }
    out
}

/// The diets of the helpers still expected or on duty, for catering: "Ernährung (ohne
/// beendeten Dienst): 2 × vegetarisch · 5 × ohne Angabe".
pub fn diet_counts(counts: &[(String, usize)]) -> String {
    let list = counts
        .iter()
        .map(|(diet, n)| diet_count(*n, diet))
        .collect::<Vec<_>>()
        .join(" · ");
    diet_counts_line(list)
}

/// Starts with the button's visible text, so its name matches what is seen.
pub fn helper_edit_label(name: &str) -> String {
    helper_edit_of(edit(), name)
}

#[cfg(test)]
mod tests {
    // [utest->dsn~ui-texts~3]
    #[test]
    fn texts_by_hand_read_as_before() {
        use super::*;
        assert_eq!(units_count(1usize), "1 Einsatzmittel");
        assert_eq!(units_count(3usize), "3 Einsatzmittel");
        assert_eq!(follow_ups_due(2usize), "2 Nachfragen fällig");
        assert_eq!(
            unreported_strength(0u32),
            "Alle Einsatzmittel haben ihre Stärke gemeldet."
        );
        assert_eq!(countdown(0), "jetzt");
        assert_eq!(countdown(425_000), "noch 7:05 min");
        assert_eq!(
            map_region_ready(
                Some((48.37, 10.9, 10.0)),
                0.0,
                Some("22.11.2026, 09:36".into())
            ),
            "Kartengrundlage bereit (0,0 MB). Umkreis 10 km um 48.37000, 10.90000. Erstellt 22.11.2026, 09:36."
        );
        assert_eq!(incident_times(Some("1"), None).as_deref(), Some("ab 1"));
        assert_eq!(
            strength_legend("Stärke", false),
            "Stärke (Führer / Unterführer / Mannschaft)"
        );
        assert_eq!(log_crew("RTW 1", 1usize), "Besatzung von RTW 1: 1 Person");
        assert_eq!(
            diet_counts(&[("vegetarisch".into(), 2)]),
            "Ernährung (ohne beendeten Dienst): 2 × vegetarisch"
        );
    }
}
