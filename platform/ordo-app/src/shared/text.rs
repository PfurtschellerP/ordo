//! User-visible texts of the shared modules (German, DE profile), apart from the
//! shell's (`crate::text`), which they re-export: a shared module names `text::…` for
//! both. Prototype: will move to Fluent files per module (I18N-1).

pub use crate::text::*;

pub const TAB_MED: &str = "Sanität";
pub const TAB_COMMAND: &str = "Einsatzleitung";
/// Workspaces of the incident command, by phase (ADR-0061).
pub const WS_LEAD: &str = "Führung";
pub const WS_PREPARE: &str = "Vorbereitung";
pub const WS_CREW: &str = "Besatzung";
pub const CREW_TASKS: &str = "Aufträge";
pub const CREW_TRANSPORTS: &str = "Transporte für Ihr Einsatzmittel";
pub const CREW_NONE_HERE: &str = "Sie sind in diesem Einsatz in keiner Besatzung eingetragen. Die Einsatzleitung trägt Sie unter Vorbereitung – Einsatzmittel ein.";
pub const DISPATCH: &str = "Disposition";
pub const PEOPLE: &str = "Personal";
pub const CREW_PATIENTS: &str = "Patienten";
pub const TAB_HELPERS: &str = "Helfende";
pub const TAB_DEVICES: &str = "Geräte";
pub const INCIDENT_PAGE: &str = "Einsatz";
// The command structure and the map on one page (ADR-0095).
pub const COMMAND_SPACE: &str = "Führung & Raum";
/// Its label in the phone's bottom bar.
pub const COMMAND_SPACE_SHORT: &str = "Struktur";

pub const TAB_RESOURCES: &str = "Einsatzmittel";
pub const TAB_HOSPITALS: &str = "Kliniken";
pub const TAB_FREE_UNITS: &str = "Freie Transportmittel";
pub const TAB_FREE_UNITS_SHORT: &str = "Freie Mittel";

pub const UHS_TITLE: &str = "Unfallhilfsstellen";
pub const UHS_CHOOSE: &str = "UHS wählen";

pub const UNITS_TITLE: &str = "Einsatzmittel";
pub const UNITS_EMPTY: &str = "Noch keine Einsatzmittel.";
pub const UNIT_CALL_SIGN: &str = "Funkrufname";
pub const UNIT_TYPE: &str = "Typ (z. B. RTW, KTW, Sanitätstrupp)";
pub const UNIT_ORGANISATION: &str = "Organisation";
pub const UNIT_SUBMIT: &str = "Einsatzmittel hinzufügen";
pub const UNIT_SERVICE: &str = "Dienst";
pub const UNIT_MISSING: &str = "Bitte Funkrufname und Typ angeben.";
pub const COL_STATUS: &str = "Status";
pub const COL_SET_STATUS: &str = "Status setzen";
pub const NO_STATUS: &str = "–";
pub fn set_status_label(unit: &str) -> String {
    format!("Status von {unit} setzen")
}
pub const UNIT_STATUS_TITLE: &str = "Einsatzmittel: Status und Aufträge";
pub const UNIT_STATUS_EMPTY: &str =
    "Noch keine Einsatzmittel im Einsatz. Sie werden in der Vorbereitung hinzugefügt.";
/// "keine offenen Aufträge", "1 offener Auftrag", "3 offene Aufträge"
pub fn open_tasks_of(n: usize) -> String {
    match n {
        0 => "keine offenen Aufträge".to_owned(),
        1 => "1 offener Auftrag".to_owned(),
        n => format!("{n} offene Aufträge"),
    }
}
/// Starts with the button's visible text, so its name matches what is seen.
pub fn task_give_label(unit: &str) -> String {
    format!("{TASK_GIVE}: {unit}")
}
/// People on more than one crew or staff list (UNIT-8), e.g. "Ben (RTW 1, UHS Süd)".
pub fn double_booked(people: &[String]) -> String {
    format!(
        "Mehrfach eingeteilt, eventuell doppelt gezählt: {}",
        people.join("; ")
    )
}
/// The person being added is on other crews or staff lists already (UNIT-8).
pub fn assigned_elsewhere(person: &str, posts: &[String]) -> String {
    format!(
        "{person} ist schon bei {} eingeteilt. Umsetzen, damit niemand doppelt gezählt wird?",
        posts.join(", ")
    )
}
pub const MOVE_HERE: &str = "Hierher umsetzen";
/// Starts with the button's visible text, so its name matches what is seen.
pub fn move_here_label(person: &str, here: &str) -> String {
    format!("{MOVE_HERE}: {person} zu {here}")
}
pub fn lost_update(unit: &str, overwritten: &str, kept: &str) -> String {
    format!(
        "Achtung {unit}: Status {} wurde gleichzeitig durch {} ersetzt. Bitte prüfen.",
        status_label(overwritten),
        status_label(kept)
    )
}
pub const GROUP_UNITS: &str = "Einsatzmittel";
pub const ACTION_ADD_UNIT: &str = "Einsatzmittel hinzufügen";
pub fn action_status(code: &str) -> String {
    format!("Status {}", status_label(code))
}

// Incidents and roles (ADR-0046)
pub const INCIDENTS_TITLE: &str = "Einsätze";
pub const INCIDENTS_INTRO: &str =
    "Einsatz wählen. Rollen im Einsatz vergibt die Einsatzleitung oder die Einsatzplanung.";
pub const INCIDENTS_LOADING: &str = "Einsätze werden geladen …";
pub const INCIDENTS_OFFLINE: &str = "Die Einsatzliste ist nur mit Verbindung zum Server verfügbar.";
/// Logging out from the incident list: the device is unpaired, so another person can log in.
pub const LOG_OUT: &str = "Abmelden";
pub const LOG_OUT_HINT: &str =
    "Abmelden entkoppelt dieses Gerät; danach kann sich eine andere Person anmelden.";
pub const INCIDENTS_EMPTY: &str = "Noch kein Einsatz freigegeben. Die Einsatzleitung oder die Einsatzplanung gibt Personen eine Rolle im Einsatz.";
pub const OPEN: &str = "Öffnen";
pub const PLAN_TITLE: &str = "Einsatz planen";
pub const PLAN_NAME: &str = "Name des Einsatzes";
pub const PLAN_SUBMIT: &str = "Einsatz anlegen";
pub const GRANT: &str = "Rolle vergeben";
pub const INCIDENT_STATE: &str = "Status";
pub const INCIDENT_PAGE_TITLE: &str = "Einsatz";
pub const INCIDENT_PREPARE: &str = "Vorbereiten: Lagekarte, Einsatzmittel und Personal mit Rollen und Führungsstruktur. Beginnen Sie den Einsatz, wenn er läuft.";
// The command's overview (ADR-0067)
pub const OPEN_MAP: &str = "Lagekarte öffnen";
pub const OPEN_DISPATCH: &str = "Zur Disposition";
pub const OPEN_UNITS: &str = "Zu den Einsatzmitteln";
pub const OPEN_COUNTS: &str = "Patienten je UHS";
pub const OPEN_LABEL: &str = "offen";
pub const NO_OPEN_TASKS: &str = "Keine offenen Aufträge.";
pub const PERSONS_ON_SCENE: &str = "Einsatzkräfte vor Ort";
pub const UNITS_ON_SCENE: &str = "Einsatzmittel vor Ort";
pub const QUICK_LINKS: &str = "Schnellzugriff";
pub fn follow_ups_due(n: usize) -> String {
    match n {
        1 => "1 Nachfrage fällig".to_owned(),
        n => format!("{n} Nachfragen fällig"),
    }
}
pub fn task_for(assignee: &str) -> String {
    format!("Für: {assignee}")
}
pub fn unreported_strength(n: u32) -> String {
    match n {
        0 => "Alle Einsatzmittel haben ihre Stärke gemeldet.".to_owned(),
        1 => "1 Einsatzmittel ohne Stärkemeldung.".to_owned(),
        n => format!("{n} Einsatzmittel ohne Stärkemeldung."),
    }
}
pub fn transports_open(n: usize) -> String {
    match n {
        0 => "Transporte: keine offen".to_owned(),
        n => format!("Transporte: {n} offen"),
    }
}
pub fn discharged_count(n: u32) -> String {
    format!("{n} entlassen")
}

pub fn summary_map(objects: usize) -> String {
    match objects {
        0 => "Noch nichts auf der Karte.".to_owned(),
        1 => "1 Objekt auf der Karte.".to_owned(),
        n => format!("{n} Objekte auf der Karte."),
    }
}
pub const START_INCIDENT: &str = "Einsatz beginnen";
pub const CLOSE_INCIDENT: &str = "Einsatz abschließen";

/// The state as people read it, with a pause: "Laufend – unterbrochen".
pub fn incident_state(state: ordo_sync::incidents::IncidentState, paused: bool) -> String {
    if paused && state == ordo_sync::incidents::IncidentState::Active {
        format!("{} – {}", state_label(state), INCIDENT_PAUSED_LABEL)
    } else {
        state_label(state).to_owned()
    }
}
pub const INCIDENT_PAUSED_LABEL: &str = "unterbrochen";
pub const PAUSE_INCIDENT: &str = "Einsatz unterbrechen";
pub const RESUME_INCIDENT: &str = "Einsatz fortsetzen";
pub const INCIDENT_PAUSE_HINT: &str = "Für Einsätze über mehrere Tage, etwa nachts bei einem Festival. Unterbrochen bleibt alles nutzbar; Lage und Einsatztagebuch laufen weiter.";
pub const INCIDENT_DETAILS: &str = "Angaben zum Einsatz";
pub const INCIDENT_DESCRIPTION: &str = "Beschreibung";
pub const INCIDENT_DESCRIPTION_HINT: &str =
    "Zum Beispiel Ort, Veranstalter, erwartete Besucher, Besonderheiten.";
pub const INCIDENT_STARTS: &str = "Geplanter Beginn";
pub const INCIDENT_ENDS: &str = "Geplantes Ende";
pub const INCIDENT_DETAILS_SAVE: &str = "Angaben speichern";
/// "13.11.2026, 18:00 bis 15.11.2026, 23:00"
pub fn incident_times(starts: Option<&str>, ends: Option<&str>) -> Option<String> {
    match (starts, ends) {
        (Some(s), Some(e)) => Some(format!("{s} bis {e}")),
        (Some(s), None) => Some(format!("ab {s}")),
        (None, Some(e)) => Some(format!("bis {e}")),
        (None, None) => None,
    }
}

pub fn withdraw_label(user: &str, role: &str) -> String {
    format!("Entziehen: {} von {user}", role_label(role))
}

// Treatment places
pub const PLACES: &str = "Behandlungsplätze";

// Transport requests (TRANS-1, TRANS-3)
pub const TRANSPORTS: &str = "Transporte";
pub const TRANSPORTS_EMPTY: &str = "Keine offenen Transporte.";

// Overview for the medical lead (UHS-8).
pub const OVERVIEW: &str = "Übersicht";

// Units (UNIT-1…6).
pub const STRENGTH_TITLE: &str = "Kräfte vor Ort";
pub const STRENGTH: &str = "Stärke";
pub const STRENGTH_LEGEND: &str = "Stärke (Führer / Unterführer / Mannschaft)";
pub const STRENGTH_NUMBERS: &str =
    "Bitte die Stärke als drei ganze Zahlen angeben oder leer lassen.";
pub const LEADERS: &str = "Führer";
pub const SUB_LEADERS: &str = "Unterführer";
pub const CREW_COUNT: &str = "Mannschaft";
pub const BY_ORGANISATION: &str = "Nach Organisation";
pub const BY_SERVICE: &str = "Nach Dienst";
pub const NO_SERVICE: &str = "ohne Dienst";
pub const NO_ORGANISATION: &str = "ohne Organisation";
pub const CREW: &str = "Besatzung";
pub const CREW_NONE: &str = "Keine Besatzung erfasst.";
pub const CREW_MEMBER: &str = "Name";
pub const CREW_QUALIFICATIONS: &str = "Qualifikationen";
pub const CREW_ADD: &str = "Zur Besatzung hinzufügen";
pub const CREW_REMOVE: &str = "Entfernen";
pub const RELEASE: &str = "Aus dem Einsatz entlassen";
pub const BACK_IN: &str = "Wieder im Einsatz";

/// "0/1/8/9" read out: "Stärke: 0 Führer, 1 Unterführer, 8 Mannschaft, 9 gesamt".
/// The strength read out: each number with its function, the total, the special functions.
pub fn strength_label(s: &ordo_units::Strength, detailed: bool, special: Option<&str>) -> String {
    let parts = s.parts(detailed);
    let n = parts.len() - 1;
    let named: Vec<String> = parts[..n]
        .iter()
        .enumerate()
        .map(|(i, v)| {
            format!(
                "{v} {}",
                strength_part(if detailed { i } else { i + 1 }, detailed)
            )
        })
        .collect();
    let mut out = format!("{STRENGTH}: {}, {} gesamt", named.join(", "), parts[n]);
    if let Some(special) = special {
        out.push_str(&format!(", {special}"));
    }
    out
}

// [impl->dsn~detailed-strength~1]
/// The name of a strength's part: 0 leaders of formations, 1 leaders, 2 sub-leaders,
/// 3 crew; the detailed notation (UNIT-7) says Zugführer and Einsatzkräfte.
pub fn strength_part(part: usize, detailed: bool) -> &'static str {
    match (part, detailed) {
        (0, _) => UNIT_LEADERS,
        (1, true) => "Zugführer",
        (1, false) => LEADERS,
        (2, _) => SUB_LEADERS,
        (_, true) => "Einsatzkräfte",
        (_, false) => CREW_COUNT,
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
    format!("{title} ({})", parts.join(" / "))
}

pub const UNIT_LEADERS: &str = "Führer von Verbänden";
pub const ORG_CHART: &str = "Führungsstruktur als Organigramm";
pub const SPECIAL_FUNCTIONS_LEGEND: &str = "Davon mit besonderen Funktionen";
pub const STRENGTH_DETAILED: &str = "Stärke ausführlich angeben: Führer von Verbänden und Zugführer getrennt, besondere Funktionen wie Ärzte und Atemschutzgeräteträger gesondert (DV 102, 2024)";

/// "davon 2 × Ärzte, 6 × AGT".
pub fn special_functions(listed: &str) -> String {
    format!("davon {listed}")
}

pub fn units_count(n: u32) -> String {
    if n == 1 {
        "1 Einsatzmittel".to_owned()
    } else {
        format!("{n} Einsatzmittel")
    }
}

pub fn unreported(n: u32) -> String {
    format!("ohne Stärkemeldung: {n}")
}

/// "3 – Einsatz übernommen, seit 10:12"
pub fn status_since(status: &str, since: &str) -> String {
    format!("{status}, seit {since}")
}

pub fn crew_label(unit: &str, n: usize) -> String {
    format!("{CREW} von {unit} ({n})")
}

pub fn crew_remove_label(name: &str) -> String {
    format!("{name} aus der Besatzung entfernen")
}

pub fn release_label(unit: &str) -> String {
    format!("{unit} aus dem Einsatz entlassen")
}

pub fn back_in_label(unit: &str) -> String {
    format!("{unit} wieder im Einsatz")
}

pub fn released_list(n: usize) -> String {
    format!("Entlassene Einsatzmittel ({n})")
}

pub fn released_at(at: &str) -> String {
    format!("Entlassen um {at}")
}

pub fn crew_title(unit: &str) -> String {
    format!("Besatzung {unit}")
}
pub fn attention_due(unit: &str, task: &str) -> String {
    format!("Rückfrage fällig: {unit} – {task}")
}
pub fn attention_new_task(task: &str) -> String {
    format!("Neuer Auftrag: {task}")
}
pub fn attention_unassigned(task: &str) -> String {
    format!("Auftrag ohne Einsatzmittel: {task}")
}
pub const CASE_REGISTER: &str = "Patient aufnehmen";
pub const OPEN_IN_MAPS_APP: &str = "In der Karten-App öffnen";

pub fn log_report_recorded(r: &ordo_reports::Recorded) -> String {
    format!(
        "Meldung aufgenommen: {}, {} – {} – {}",
        task_kind(crate::shared::reports::task_kind(r.kind)),
        task_received(&crate::core::clock(r.received_at)),
        r.place,
        r.text
    )
}

pub fn log_report_closed(report: &str) -> String {
    format!("Meldung abgeschlossen: {report}")
}

pub fn log_report_patient(task: &str, unit: &str) -> String {
    format!("Patient aus dem Auftrag {task} aufgenommen von {unit}")
}

// Tasks (TASK-1…4).
pub const TASKS_TITLE: &str = "Offene Aufträge";
pub const TASKS_EMPTY: &str = "Keine offenen Aufträge.";
pub const TASK_GIVE: &str = "Auftrag geben";
pub const TASK_UNIT: &str = "Für";
pub const TASK_UNIT_CHOOSE: &str = "Einsatzmittel oder Funktion wählen";
pub const TASK_TEMPLATE: &str = "Vorlage";
pub const TASK_TEMPLATE_NONE: &str = "ohne Vorlage";
pub const TASK_TEXT: &str = "Auftrag";
pub const TASK_FOLLOW_UP: &str = "Nachfragen in (Minuten, optional)";
pub const TASK_MISSING: &str = "Bitte Empfänger und Auftrag angeben.";
pub const TASK_NEW: &str = "Neuer Auftrag";
pub const TASK_CREATE: &str = "Auftrag anlegen";
pub const TASK_KIND: &str = "Art";
pub const TASK_KIND_OTHER: &str = "Sonstiger Auftrag";
pub const TASK_KIND_MISSING: &str = "Bitte die Art wählen.";
pub const TASK_UNIT_LATER: &str = "Später zuweisen";
pub const TASK_UNIT_ONLY: &str = "Einsatzmittel wählen";
pub const TASK_UNIT_HINT: &str =
    "Bei Patient und Ereignis optional: das Einsatzmittel lässt sich auch später zuweisen.";
pub const TASK_PLACE: &str = "Ort";
pub const TASK_WHAT: &str = "Was ist los?";
pub const TASK_WHAT_HINT: &str =
    "In wenigen Worten, ohne medizinische Angaben (die gehören in die Patientenakte).";
pub const TASK_RECEIVED: &str = "Eingang";
pub const TASK_RECEIVED_HINT: &str = "Leer lassen für jetzt.";
pub const TASK_TEXT_MISSING: &str = "Bitte kurz beschreiben, worum es geht.";
pub const TASK_PLACE_MISSING: &str = "Bitte den Ort angeben.";
pub const TASK_TOO_LONG: &str = "Ort oder Beschreibung sind zu lang.";
pub const TASK_TIME_INVALID: &str = "Bitte eine Uhrzeit angeben, z. B. 14:05.";
pub const TASK_ASSIGN_TO: &str = "Zuweisen an";
pub const TASK_ASSIGN: &str = "Zuweisen";
pub const TASK_UNIT_MISSING: &str = "Bitte ein Einsatzmittel wählen.";
pub const TASK_SEND_FURTHER: &str = "Weiteres Einsatzmittel schicken";
pub const TASK_SEND_TO: &str = "Zusätzlich schicken";
pub const TASK_SEND: &str = "Schicken";
pub const TASK_ON_MAP: &str = "Pin auf der Lagekarte";
pub const MY_TASKS: &str = "Meine Aufträge";
pub const PIN_TITLE: &str = "Pin auf der Karte (optional)";
pub const PIN_HINT: &str = "Adresse suchen oder in die Karte tippen. Mit der Tastatur: Karte mit den Pfeiltasten verschieben, dann „Pin in Kartenmitte setzen“.";
pub const PIN_AT_CENTRE: &str = "Pin in Kartenmitte setzen";
pub const PIN_CLEAR: &str = "Pin entfernen";
pub const PIN_NONE: &str = "Kein Pin gesetzt.";
pub const PIN_SET: &str = "Pin setzen";
pub const PIN_MOVE: &str = "Pin verschieben";
pub const PIN_SAVE: &str = "Pin speichern";

// Map object cards (MAP-11).
pub const MAP_DETAILS: &str = "Details";
pub const MAP_OBJECT_GONE: &str = "Nicht mehr auf der Karte.";
pub const CARD_ASSIGN: &str = "Einsatzmittel zuweisen";
pub const CARD_OPEN_TASK: &str = "Auftrag öffnen";
pub const CARD_CURRENT_TASK: &str = "Auftrag";
pub const CARD_EDIT: &str = "Bearbeiten";
pub const CARD_PATIENTS: &str = "Patienten nach Sichtungskategorie";
pub const CARD_OPEN_UHS: &str = "UHS öffnen";
pub const QUALIFICATIONS_SHORT: &str = "Qualifikationen";

/// "Anna Schmidt entfernt."
pub fn removed(what: &str) -> String {
    format!("{what} entfernt.")
}

/// "Rolle Arzt/Ärztin von Anna Schmidt entzogen."
pub fn role_removed(role: &str, who: &str) -> String {
    format!("Rolle {} von {who} entzogen.", role_label(role))
}

pub fn map_details_label(what: &str) -> String {
    format!("{MAP_DETAILS}: {what}")
}

pub fn card_edit_label(unit: &str) -> String {
    format!("{CARD_EDIT}: {unit}")
}

/// "Notfallsanitäter/in: 2"
pub fn qualification_count(label: &str, n: usize) -> String {
    format!("{label}: {n}")
}

pub fn crew_names(n: usize) -> String {
    format!("Namen ({n})")
}

pub fn pin_at(at: &str) -> String {
    format!("Pin: {at}")
}

pub fn pin_edit_label(placed: bool, task: &str) -> String {
    format!("{}: {task}", if placed { PIN_MOVE } else { PIN_SET })
}

pub fn task_kind(kind: ordo_tasks::TaskKind) -> &'static str {
    match kind {
        ordo_tasks::TaskKind::Event => "Ereignis",
        ordo_tasks::TaskKind::Patient => "Patient",
    }
}

/// "Patient: Bühne Nord"
pub fn task_label(kind: ordo_tasks::TaskKind, place: &str) -> String {
    format!("{}: {place}", task_kind(kind))
}

pub fn task_received(at: &str) -> String {
    format!("Eingang {at}")
}

pub fn task_taken(unit: &str, at: &str) -> String {
    format!("Patient aufgenommen von {unit} ({at})")
}

/// How far an Auftrag is (TASK-2).
pub fn task_phase(phase: ordo_tasks::Phase) -> &'static str {
    use ordo_tasks::Phase as P;
    match phase {
        P::Received => "Eingegangen",
        P::Assigned => "Zugewiesen",
        P::InProgress => "In Arbeit",
        P::Done => "Erledigt",
        P::Cancelled => "Abgebrochen",
    }
}

pub fn log_task_received(
    kind: ordo_tasks::TaskKind,
    at: &str,
    place: &str,
    task: &str,
    to: &str,
) -> String {
    let line = format!(
        "Auftrag eingegangen: {}, {} – {place} – {task}",
        task_kind(kind),
        task_received(at)
    );
    if to.is_empty() {
        line
    } else {
        format!("{line} – an {to}")
    }
}

pub fn log_task_assigned(task: &str, unit: &str) -> String {
    format!("Auftrag „{task}“ an {unit} zugewiesen")
}
pub const TASK_FOLLOW_UP_NUMBER: &str = "Bitte die Minuten als ganze Zahl angeben.";
pub const FOLLOW_UP_LATER: &str = "In 10 Minuten nachfragen";
pub const DUE: &str = "Fällig";

pub fn tasks_title(n: usize) -> String {
    format!("{TASKS_TITLE} ({n})")
}

pub fn task_step(status: ordo_tasks::TaskStatus) -> &'static str {
    use ordo_tasks::TaskStatus as S;
    match status {
        S::Open => "Öffnen",
        S::InProgress => "In Arbeit",
        S::Done => "Erledigt",
        S::Cancelled => "Abbrechen",
    }
}

/// "In Arbeit: Patient: Bühne Nord"
pub fn task_step_label(step: &str, task: &str) -> String {
    format!("{step}: {task}")
}

/// One unit on an Auftrag, for accessible names: "RTW 2 – Patient: Bühne Nord".
pub fn unit_in_task(unit: &str, task: &str) -> String {
    format!("{unit} – {task}")
}

pub const TASK_UNITS: &str = "Einsatzmittel im Auftrag";

/// "Nachfragen um 10:42"
pub fn follow_up_at(at: &str) -> String {
    format!("Nachfragen um {at}")
}

/// Time left until a follow-up, e.g. "noch 7:05 min"; "jetzt" at zero.
pub fn countdown(left_ms: u64) -> String {
    let secs = left_ms.div_ceil(1000);
    if secs == 0 {
        return "jetzt".to_owned();
    }
    format!("noch {}:{:02} min", secs / 60, secs % 60)
}

/// "Fällig: Nachfragen seit 10:42"
pub fn follow_up_due(at: &str) -> String {
    format!("{DUE}: nachfragen seit {at}")
}

/// "10:12, ben@…"
pub fn task_given_by(at: &str, by: &str) -> String {
    format!("Gegeben {at} von {by}")
}

// Command structure (CMD-1…4, ADR-0031).
pub const STRUCTURE_TITLE: &str = "Führungsstruktur";
pub const STRUCTURE_SHORT: &str = "Struktur";
pub const STRUCTURE_EMPTY: &str = "Noch keine Führungsstruktur angelegt.";
pub const POSITION: &str = "Funktion";
pub const POSITION_CHOOSE: &str = "Funktion wählen";
pub const POSITION_CREATE: &str = "Funktion anlegen";
pub const POSITION_KIND: &str = "Art";
pub const POSITION_KIND_CHOOSE: &str = "Art wählen";
pub const POSITION_NAME: &str = "Bezeichnung";
pub const POSITION_PARENT: &str = "Unterstellt";
pub const POSITION_TOP: &str = "oberste Ebene";
pub const POSITION_MISSING: &str = "Bitte Art und Bezeichnung angeben.";
pub const POSITION_CLOSE: &str = "Auflösen";
pub const NO_POSITION: &str = "ohne Zuordnung";
pub const NO_LEADER: &str = "Leitung: nicht besetzt";
pub const LEADER: &str = "Leitung";
pub const LEADER_SET: &str = "Leitung festlegen oder übergeben";
pub const UHS_ASSIGN: &str = "UHS zuordnen";

pub fn leader(name: &str) -> String {
    format!("{LEADER}: {name}")
}

/// "Übergabe: Dr. M. → Dr. K." or "Leitung: Dr. M."
pub fn handover(from: Option<&str>, to: &str) -> String {
    match from {
        Some(from) => format!("Übergabe: {from} → {to}"),
        None => leader(to),
    }
}

/// What a member of a position is, under its name in the org chart.
pub const MEMBER_UHS: &str = "UHS";

/// Shown, never enforced (ADR-0031).
pub fn span_of_control(n: usize) -> String {
    format!("{n} direkt unterstellt: mehr als 5, Führungsspanne beachten.")
}

pub fn position_close_label(title: &str) -> String {
    format!("{title} auflösen")
}

// A position put on the map, or found there, from the tree (CMD-5).
pub const POSITION_MAP_SET: &str = "Auf der Karte setzen";
pub const POSITION_MAP_SHOW: &str = "Auf der Karte zeigen";

pub fn position_map_set_label(title: &str) -> String {
    format!("Auf der Karte setzen: {title}")
}

pub fn position_map_show_label(title: &str) -> String {
    format!("Auf der Karte zeigen: {title}")
}

/// The place panel's status once a position was chosen from the tree.
pub fn map_place_ready(label: &str) -> String {
    format!("{label}: auf die Stelle tippen oder in der Kartenmitte platzieren.")
}

pub fn position_of(unit: &str) -> String {
    format!("Funktion von {unit}")
}

// Situation map (MAP-1, MAP-2).
pub const MAP_TITLE: &str = "Lagekarte";
pub const MAP_SHORT: &str = "Karte";
pub const MAP_LABEL: &str = "Lagekarte. Pfeiltasten verschieben, Plus und Minus zoomen.";
pub const MAP_LOADING: &str = "Karte wird geladen …";
pub const MAP_HINT_TOUCH: &str = "Karte mit zwei Fingern verschieben";
pub const MAP_HINT_WHEEL: &str = "Zoomen: Strg und Mausrad";
pub const MAP_HINT_WHEEL_MAC: &str = "Zoomen: ⌘ und Mausrad";
pub const MAP_UNAVAILABLE: &str = "Die Karte kann auf diesem Gerät nicht angezeigt werden; die Liste unten zeigt alles, was auf ihr steht.";
pub const MAP_FROM_DEVICE: &str = "Kartengrundlage: auf diesem Gerät gespeichert (auch ohne Netz).";
pub const MAP_FROM_SERVER: &str = "Kartengrundlage: vom Server.";
pub const MAP_NO_REGION: &str = "Für diesen Einsatz wurde keine Kartengrundlage vorbereitet: die Objekte stehen auf neutralem Hintergrund.";
pub const MAP_OBJECTS: &str = "Auf der Karte";
pub const MAP_LOCATE: &str = "Zu meinem Standort";
pub const MAP_LOCATING: &str = "Standort wird bestimmt …";
pub const MAP_LOCATED: &str = "Die Karte zeigt Ihren Standort.";
pub const MAP_LOCATE_DENIED: &str = "Der Standort ist nicht freigegeben. Erlauben Sie ORDO den Standort in den Einstellungen des Browsers.";
pub const MY_POSITION: &str = "Ich";
pub const MY_POSITION_SHOW: &str = "Meinen Standort zeigen";
pub const MY_POSITION_SHOWN: &str =
    "Die Karte zeigt Ihren Standort (Punkt „Ich“) und den Pin. Er bleibt auf diesem Gerät.";
pub const MAP_LOCATE_FAILED: &str = "Der Standort konnte nicht bestimmt werden.";
pub const MAP_OBJECTS_NONE: &str = "Noch nichts auf der Karte.";
pub const MAP_SHOW: &str = "Zeigen";
pub const MAP_REMOVE: &str = "Entfernen";
pub const MAP_PLACE_HINT: &str = "Auf die Stelle tippen – oder die Karte so verschieben, dass die Stelle am Fadenkreuz liegt (auch mit den Pfeiltasten), und in der Kartenmitte platzieren.";
pub const MAP_PLACE_KIND: &str = "Was";
pub const MAP_PLACE_WHICH: &str = "Welches";
pub const MAP_PLACE_CHOOSE: &str = "Bitte wählen";
pub const MAP_PLACE_LABEL: &str = "Bezeichnung";
pub const MAP_PLACE_SUBMIT: &str = "In der Kartenmitte platzieren";
pub const MAP_PLACE_MISSING: &str = "Bitte auswählen, was platziert werden soll.";
pub const INCIDENT_LOCATION: &str = "Einsatzort";
pub const MAP_SAVING: &str =
    "Die Kartengrundlage wird für die Nutzung ohne Netz auf diesem Gerät gespeichert …";
pub const MAP_SAVE_FAILED: &str = "Die Karte konnte nicht gespeichert werden.";
pub const MAP_KIND_SYMBOL: &str = "Taktisches Zeichen";
pub const MAP_SYMBOL_LABEL: &str = "Beschriftung (optional)";

pub fn map_kind(kind: ordo_map::ObjectKind) -> &'static str {
    use ordo_map::ObjectKind as K;
    match kind {
        K::Incident => INCIDENT_LOCATION,
        K::Unit => TAB_RESOURCES,
        K::Uhs => "UHS",
        K::Position => POSITION,
        K::Drawing => "Zeichnung",
        K::Report => "Meldung",
        K::Task => TASK_TEXT,
        K::Symbol => MAP_KIND_SYMBOL,
    }
}

pub fn map_show_label(label: &str) -> String {
    format!("{label} auf der Karte zeigen")
}

pub fn map_remove_label(label: &str) -> String {
    format!("{label} von der Karte entfernen")
}

// The map editor (ADR-0064)
pub const MAP_EDIT: &str = "Lagekarte bearbeiten";
pub const MAP_EDIT_HINT: &str = "Einsatzstelle, Einsatzmittel, UHS und Funktionen auf der Karte setzen, Flächen und Linien zeichnen. Die Führung sieht die Karte unter Führung – Lagekarte.";
pub const MAP_EDIT_NOT_ALLOWED: &str =
    "Die Karte bearbeitet die Einsatzleitung; hier ist sie nur zu sehen.";
pub const MAP_EDIT_OBJECT: &str = "Bearbeiten";
pub const MAP_TOOLS: &str = "Werkzeug";
pub const TOOL_SELECT: &str = "Auswählen";
pub const TOOL_PLACE: &str = "Symbol setzen";
pub const TOOL_AREA: &str = "Fläche";
pub const TOOL_LINE: &str = "Linie";
pub const MAP_TAP_TO_CHOOSE: &str =
    "Auf ein Objekt der Karte tippen oder es in der Liste unten bearbeiten.";
pub const MAP_MOVE_HINT: &str = "Auf die neue Stelle tippen – oder die Karte so verschieben, dass sie am Fadenkreuz liegt (auch mit den Pfeiltasten), und in die Kartenmitte setzen.";
pub const MAP_MOVE_CENTRE: &str = "In die Kartenmitte setzen";
pub const MAP_DESELECT: &str = "Auswahl aufheben";
pub const RESHAPE_HINT: &str = "Ecken ziehen, um die Form zu ändern; aus der Mitte einer Seite eine neue Ecke ziehen; die Form ziehen, um sie zu verschieben. Ohne Ziehen: „Neu zeichnen“.";
pub const RESHAPE_APPLY: &str = "Übernehmen";
pub const RESHAPE_DISCARD: &str = "Verwerfen";
pub const RESHAPE_REDRAW: &str = "Neu zeichnen";
pub const MAP_RESHAPE_FAILED: &str =
    "Diese Form lässt sich hier nicht ziehen; „Neu zeichnen“ geht immer.";

pub fn map_edit_label(label: &str) -> String {
    format!("{label} bearbeiten")
}

pub fn map_chosen(label: &str) -> String {
    format!("{label} ausgewählt.")
}

pub fn map_moved(label: &str) -> String {
    format!("{label} verschoben.")
}

pub fn map_placed(label: &str) -> String {
    format!("{label} platziert.")
}

pub fn map_reshaped(label: &str) -> String {
    format!("{label} geändert.")
}

pub fn map_drawn(label: &str) -> String {
    format!("{label} gespeichert.")
}

pub fn map_removed(label: &str) -> String {
    format!("{label} entfernt.")
}

pub fn draw_redraw(label: &str) -> String {
    format!("{label} neu zeichnen")
}

fn megabytes(bytes: f64) -> String {
    format!("{:.1}", bytes / 1_000_000.0).replace('.', ",")
}

pub fn map_saved(bytes: f64) -> String {
    format!(
        "Kartengrundlage auf diesem Gerät gespeichert ({} MB), auch ohne Netz nutzbar.",
        megabytes(bytes)
    )
}

// Incident log (LOG-1, LOG-3, LOG-6)
pub const LOG_TITLE: &str = "Einsatztagebuch";
pub const LOG_HINT: &str = "Jede Änderung in diesem Einsatz, die dieses Gerät kennt, mit Zeit, Person und Gerät, die neueste zuerst. Einträge werden nie geändert: Eine Korrektur ist ein neuer Eintrag, der auf den alten verweist.";
pub const LOG_EMPTY: &str = "Noch keine Einträge.";
pub const LOG_PENDING: &str = "noch nicht gesendet";

pub fn log_by(user: &str, device: &str) -> String {
    format!("{user} · Gerät {device}")
}

pub fn log_corrects(at: &str) -> String {
    format!("(korrigiert den Eintrag von {at})")
}

/// The patient counts of a UHS as the log shows them (derived by the server, ADR-0024):
/// "Patientenzahlen UHS Eingang: 5 (SK I 1 · SK III 4), 1 ohne Platz, 2 entlassen".
/// "Bei Einsatzmitteln: 2 (SK I 1 · SK II 1)".
pub fn with_units(total: u32, counts: &[(&str, u32)]) -> String {
    let parts: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(label, n)| format!("{label} {n}"))
        .collect();
    format!("{}: {total} ({})", with_units_label(), parts.join(" · "))
}

pub fn log_unknown(module: &str, event_type: &str) -> String {
    format!("{module}: {event_type}")
}

pub fn log_planned(name: &str) -> String {
    format!("Einsatz „{name}“ angelegt")
}

pub fn log_unit_added(unit: &str, unit_type: &str, organisation: &str) -> String {
    format!("{} ({unit_type}, {organisation})", unit_added(unit))
}

pub fn log_helper_registered(name: &str) -> String {
    format!("{name} aufgenommen")
}
/// "Anna Schmidt: Dienstbeginn 14:00 · Angaben geändert: Telefon".
pub fn log_helper_updated(name: &str, changes: &[String]) -> String {
    if changes.is_empty() {
        return format!("Angaben zu {name} gespeichert");
    }
    format!("{name}: {}", changes.join(" · "))
}
pub fn log_cleared(label: &str) -> String {
    format!("{label} gelöscht")
}
pub const LOG_PAUSE_ENDED: &str = "Pause beendet";
pub fn log_pause_since(at: &str) -> String {
    format!("Pause ab {at}")
}
pub const LOG_ADDRESS: &str = "Anschrift";
pub fn log_fields_changed(fields: &[&str]) -> String {
    format!("Angaben geändert: {}", fields.join(", "))
}

pub fn log_crew(unit: &str, n: usize) -> String {
    match n {
        0 => format!("Besatzung von {unit}: niemand"),
        1 => format!("Besatzung von {unit}: 1 Person"),
        n => format!("Besatzung von {unit}: {n} Personen"),
    }
}

pub fn log_task_given(to: &str, task: &str) -> String {
    format!("Auftrag an {to}: {task}")
}

/// A unit's step on an Auftrag (ADR-0085): "Auftrag „…“: RTW 2 In Arbeit".
pub fn log_unit_step(task: &str, unit: &str, status: Option<&str>) -> String {
    match status {
        Some(s) => format!("Auftrag „{task}“: {unit} {s}"),
        None => log_task_updated(task, None),
    }
}

pub fn log_task_updated(task: &str, status: Option<&str>) -> String {
    match status {
        Some(s) => format!("Auftrag „{task}“: {s}"),
        None => format!("Auftrag „{task}“: Nachfrage geändert"),
    }
}

pub fn log_position_closed(title: &str) -> String {
    format!("{title} aufgelöst")
}

pub fn log_leader(title: &str, leader: &str) -> String {
    format!("{title}: Leitung {leader}")
}

pub fn log_assigned(what: &str, position: Option<&str>) -> String {
    match position {
        Some(p) => format!("{what} unterstellt: {p}"),
        None => format!("{what}: Unterstellung aufgehoben"),
    }
}

pub fn log_map_placed(label: &str) -> String {
    format!("{label} auf der Lagekarte platziert")
}

pub fn log_map_removed(label: &str) -> String {
    format!("{label} von der Lagekarte entfernt")
}

// Map region from the server (MAP-8)
pub const MAP_REGION_TITLE: &str = "Kartengrundlage";
pub const MAP_REGION_SHORT: &str = "Grundlage";
pub const MAP_REGION_HINT: &str = "Die Kartengrundlage dieses Einsatzes für die Lagekarte: ein Umkreis aus der freien Weltkarte (OpenStreetMap über Protomaps). Nur die Geräte der Einsatzkräfte dieses Einsatzes erhalten sie. Geräte, die eine Offline-Karte gespeichert haben, speichern sie danach neu.";
pub const MAP_REGION_NONE: &str = "Für diesen Einsatz ist noch keine Kartengrundlage erstellt.";
pub const MAP_REGION_NEW: &str = "Neue Kartengrundlage";
pub const MAP_REGION_LAT: &str = "Breitengrad (Mitte)";
pub const MAP_REGION_LON: &str = "Längengrad (Mitte)";
pub const MAP_REGION_RADIUS: &str = "Umkreis (km, 1 bis 50)";
pub const MAP_REGION_FROM_INCIDENT: &str = "Die Mitte ist der Einsatzort.";
pub const MAP_REGION_COORDINATES: &str = "Mitte als Dezimalgrad, z. B. 48,37 und 10,90.";
pub const MAP_REGION_CUT: &str = "Kartengrundlage erstellen";
pub const MAP_REGION_PROGRESS: &str = "Fortschritt";
pub const MAP_REGION_INVALID: &str = "Bitte Breitengrad (−85 bis 85), Längengrad (−180 bis 180) und einen Umkreis von 1 bis 50 km angeben.";

pub fn map_region_running(done: u64, total: u64) -> String {
    if total == 0 {
        "Kartengrundlage wird vorbereitet …".to_owned()
    } else {
        format!("Kartengrundlage wird erstellt: {done} von {total} Kacheln …")
    }
}

pub fn map_region_ready(area: Option<(f64, f64, f64)>, bytes: f64, at: Option<String>) -> String {
    let what = area.map_or_else(String::new, |(lat, lon, km)| {
        format!(
            " Umkreis {} km um {lat:.5}, {lon:.5}.",
            format!("{km}").replace('.', ",")
        )
    });
    let when = at.map_or_else(String::new, |at| format!(" Erstellt {at}."));
    format!(
        "Kartengrundlage bereit ({} MB).{what}{when}",
        megabytes(bytes)
    )
}

pub fn map_region_failed(error: &str) -> String {
    format!("Die Kartengrundlage konnte nicht erstellt werden: {error}")
}

// Drawing on the map (MAP-4)
pub const DRAW_HINT: &str = "Auf jede Ecke tippen – oder die Karte so verschieben, dass die Ecke am Fadenkreuz liegt (auch mit den Pfeiltasten), und sie setzen. Dann „Fertig“.";
pub const DRAW_CATEGORY: &str = "Art";
pub const DRAW_ADD_CORNER: &str = "Ecke in der Kartenmitte setzen";
pub const DRAW_UNDO: &str = "Letzte Ecke entfernen";
pub const DRAW_CANCEL: &str = "Abbrechen";
pub const DRAW_FINISH: &str = "Fertig";
pub const DRAW_TOO_FEW_AREA: &str = "Eine Fläche braucht mindestens drei Ecken.";
pub const DRAW_TOO_FEW_LINE: &str = "Eine Linie braucht mindestens zwei Punkte.";

pub fn draw_corners(n: usize) -> String {
    match n {
        0 => "Noch keine Ecke gesetzt.".to_owned(),
        1 => "1 Ecke gesetzt.".to_owned(),
        n => format!("{n} Ecken gesetzt."),
    }
}

// Helper register (HELP-1, HELP-2, ADR-0071)
pub const HELPERS_TITLE: &str = "Helferinnen und Helfer";
pub const HELPERS_INTRO: &str =
    "Persönliche Angaben der Helfenden dieses Einsatzes. Nur die Einsatzleitung sieht sie.";
pub const HELPERS_INTRO_ROLES: &str =
    "Wer in diesem Einsatz welche Rolle hat. Persönliche Angaben sieht nur die Einsatzleitung.";
pub const HELPERS_EMPTY: &str = "Noch niemand erfasst.";
pub const HELPER_ADD_TITLE: &str = "Person aufnehmen";
pub const HELPER_ADD: &str = "Aufnehmen";
pub const ROLE_ADD_TITLE: &str = "Rolle vergeben";
pub const HELPER_COMPLETE: &str = "Angaben ergänzen";
pub fn helper_complete_label(name: &str) -> String {
    format!("{HELPER_COMPLETE}: {name}")
}
pub fn helper_complete_title(name: &str) -> String {
    format!("Angaben zu {name} ergänzen")
}
pub const HELPER_NAME: &str = "Name";
pub const HELPER_PHONE: &str = "Telefon";
pub const HELPER_PHONE_HINT: &str = "Mit Vorwahl, z. B. 0171 1234567 oder +43 664 1234567.";
pub const HELPER_PHONE_INVALID: &str =
    "Die Telefonnummer ist keine gültige Nummer mit Vorwahl, z. B. 0171 1234567.";
pub const HELPER_STREET: &str = "Straße und Hausnummer";
pub const HELPER_POSTCODE: &str = "PLZ";
pub const HELPER_POSTCODE_INVALID: &str =
    "Die Postleitzahl hat nicht die Form des Landes, z. B. 55116.";
pub const HELPER_TOWN: &str = "Ort";
pub const HELPER_ROLE: &str = "Rolle im Einsatz";
pub const HELPER_ROLES: &str = "Rollen im Einsatz";
pub const OR: &str = "oder";
pub const GUEST_DUTY_ENDED: &str = "Das Dienstende ist vorbei. Für einen Zugang ohne Konto den Dienst neu beginnen oder das Dienstende ändern.";
pub const GUEST_UNTIL_CLOSED: &str = "Gast-Zugang bis zum Einsatzende";
pub const GUEST_NO_COMMAND: &str = "Die Einsatzleitung braucht ein ORDO-Konto.";
pub const GUEST_ACCESS: &str = "Zugang per QR-Code";
pub const GUEST_CODE_TITLE: &str = "QR-Code für den Zugang";
pub const GUEST_CODE_REFUSED: &str = "Der Zugang konnte nicht erstellt werden.";
pub const GUEST_CODE_ALT: &str = "QR-Code, der dieses Gerät des Helfers mit dem Einsatz koppelt";
pub const GUEST_LINK: &str = "Oder diesen Link auf dem Gerät öffnen:";
pub const GUEST_CODE_CLOSE: &str = "Fertig";
pub fn guest_code_for(name: &str, expires: &str) -> String {
    format!(
        "Für {name}: mit der Kamera jedes Geräts scannen, das {name} nutzt. Gilt bis {expires}."
    )
}
pub fn guest_code_until_closed(name: &str) -> String {
    format!(
        "Für {name}: mit der Kamera jedes Geräts scannen, das {name} nutzt. Gilt bis zum Einsatzende."
    )
}
pub fn guest_access_label(name: &str) -> String {
    format!("{GUEST_ACCESS}: {name}")
}
pub fn guest_until(until: &str) -> String {
    format!("Gast-Zugang bis {until}")
}
pub const ROLE_MISSING: &str = "Bitte eine Rolle wählen.";
pub const HELPER_ACCOUNT_HINT: &str =
    "Konto der Organisation wählen; der Name wird übernommen. Leer lassen für Helfende ohne Konto.";
pub const DIET_OTHER: &str = "andere …";
pub const DIET_OTHER_LABEL: &str = "Andere Ernährung";
pub const HELPER_BIRTH_DATE_LABEL: &str = "Geburtsdatum";
pub const HELPER_DIET: &str = "Ernährung";
pub const DIET_NONE: &str = "ohne Angabe";
/// The diets of the helpers still expected or on duty, for catering: "Ernährung (ohne
/// beendeten Dienst): 2 × vegetarisch · 5 × ohne Angabe".
pub fn diet_counts(counts: &[(String, usize)]) -> String {
    let list = counts
        .iter()
        .map(|(diet, n)| format!("{n} × {diet}"))
        .collect::<Vec<_>>()
        .join(" · ");
    format!("Ernährung (ohne beendeten Dienst): {list}")
}
pub const HELPER_DUTY_START: &str = "Dienstbeginn";
pub const HELPER_DUTY_END: &str = "Dienstende";
pub const HELPER_ACCOUNT_LABEL: &str = "ORDO-Konto";
pub const HELPER_NAME_MISSING: &str = "Bitte einen Namen angeben.";
pub const HELPER_TOO_LONG: &str = "Jede Angabe darf höchstens 200 Zeichen lang sein.";
pub const HELPER_BIRTH_DATE: &str = "Bitte das Geburtsdatum als Datum angeben.";
pub const HELPER_ACCOUNT: &str =
    "Das ORDO-Konto ist keine gültige Kennung, z. B. anna@ordo.example.org.";
pub const HELPER_TIME: &str = "Bitte Dienstbeginn und -ende als Datum mit Uhrzeit angeben.";
pub const HELPER_NOT_SAVED: &str = "Die Angaben konnten nicht gespeichert werden.";
pub const DUTY_START: &str = "Dienst beginnen";
pub const DUTY_END: &str = "Dienst beenden";
/// Starts with the button's visible text, so its name matches what is seen.
pub fn duty_start_label(name: &str) -> String {
    format!("{DUTY_START}: {name}")
}
/// Starts with the button's visible text, so its name matches what is seen.
pub fn duty_end_label(name: &str) -> String {
    format!("{DUTY_END}: {name}")
}
/// Starts with the button's visible text, so its name matches what is seen.
pub fn helper_edit_label(name: &str) -> String {
    format!("{}: {name}", edit())
}
pub fn helper_edit_title(name: &str) -> String {
    format!("{name} bearbeiten")
}
pub const DUTY_PAUSE: &str = "Dienst pausieren";
pub const DUTY_RESUME: &str = "Dienst fortsetzen";
pub fn duty_pause_label(name: &str) -> String {
    format!("{DUTY_PAUSE}: {name}")
}
pub fn duty_resume_label(name: &str) -> String {
    format!("{DUTY_RESUME}: {name}")
}
/// "in Pause seit 14:02"
pub fn paused_since(at: &str) -> String {
    format!("in Pause seit {at}")
}
pub fn on_duty_since(at: &str) -> String {
    format!("im Dienst seit {at}")
}
pub fn duty_from_to(from: &str, to: &str) -> String {
    format!("Dienst {from} – {to}")
}
pub fn duty_from(from: &str) -> String {
    format!("Dienst ab {from}")
}
pub fn duty_to(to: &str) -> String {
    format!("Dienst bis {to}")
}
/// The person typed for a crew is not in the helper register (HELP-3).
pub fn not_registered(name: &str) -> String {
    format!("{name} ist nicht im Helferregister. Mit dem Namen erfassen und hinzufügen?")
}
pub const REGISTER_AND_ADD: &str = "Erfassen und hinzufügen";
/// Starts with the button's visible text, so its name matches what is seen.
pub fn register_and_add_label(name: &str) -> String {
    format!("{REGISTER_AND_ADD}: {name}")
}
/// Registration cards (HELP-5, ADR-0072).
/// "Meldekarte fotografieren", with the profile's label of the card ("Karte" for several).
pub const CARD_GENERIC: &str = "Karte";
pub fn card_photo(label: &str) -> String {
    format!("{label} fotografieren")
}
pub const CARD_READING: &str = "Karte wird gelesen …";
pub const CARD_NOT_RECOGNISED: &str = "Keine Karte erkannt. Bitte noch einmal fotografieren: Karte flach, QR-Code ganz im Bild und scharf.";
pub fn card_read(card: &str, name: &str) -> String {
    format!("{card} von {name} gelesen. Bitte prüfen und erfassen.")
}
