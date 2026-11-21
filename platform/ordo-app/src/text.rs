//! User-visible texts (German, DE profile). Prototype: will move to Fluent translation
//! files (I18N-1); code and identifiers stay English.

use ordo_contracts::TriageCategory;

pub const APP_NAME: &str = "ORDO";
pub const SKIP_LINK: &str = "Zum Inhalt springen";

pub const PAIR_TITLE: &str = "Gerät koppeln";
pub const PAIR_INTRO: &str = "Melde dich mit dem Konto deiner Organisation an. Dieses Gerät erhält dabei einen eigenen Schlüssel, der es nie verlässt; ORDO speichert keine Passwörter.";
pub const PAIR_LOGIN: &str = "Anmelden";
pub const PAIR_QR: &str = "Mit QR-Code koppeln";
pub const PAIR_QR_HINT: &str = "Für gemeinsam genutzte Tablets: Mit einem angemeldeten Handy den QR-Code scannen und freigeben.";
pub const PAIR_QR_ALT: &str = "QR-Code zum Koppeln dieses Geräts";
pub fn pair_code(code: &str) -> String {
    format!("Code: {code}")
}
pub const PAIR_QR_LINK: &str = "Ohne Kamera: auf einem anderen Gerät öffnen und den Code eingeben:";
pub const PAIR_WAITING: &str = "Warte auf Freigabe …";
pub const PAIR_NO_LOGIN: &str = "Für diesen Server ist keine Anmeldung eingerichtet.";
pub const PAIR_MANUAL: &str = "Entwicklung: Gerät manuell koppeln";
pub const PAIR_MANUAL_INTRO: &str = "Für Testumgebungen ohne Anmeldung: Angaben aus der Seed-Datei des Servers und aus `ordo generate-device-key`.";
pub const LOGIN_FAILED: &str = "Anmeldung fehlgeschlagen";
pub const RELOGIN_TITLE: &str = "Anmeldung abgelaufen";
pub const RELOGIN_INTRO: &str = "Die Anmeldung dieses Geräts ist abgelaufen. Bitte erneut anmelden – die Daten auf dem Gerät bleiben erhalten und werden danach weiter abgeglichen.";
pub const NETWORK_ERROR: &str = "Server nicht erreichbar";
pub const PAIR_USER: &str = "Benutzer (name@server)";
pub const PAIR_DEVICE: &str = "Geräte-ID";
pub const PAIR_KEY: &str = "Privater Schlüssel des Geräts";
pub const PAIR_INCIDENT: &str = "Einsatz-ID";
pub const PAIR_SUBMIT: &str = "Koppeln";
pub const UNPAIR: &str = "Gerät entkoppeln";
pub const SIGNED_IN_AS: &str = "Angemeldet als";
pub const AVATAR_CHANGE: &str = "Profilbild ändern";
pub const AVATAR_REMOVE: &str = "Profilbild entfernen";
pub const RETRY: &str = "Erneut versuchen";
pub const STARTING: &str = "ORDO wird gestartet …";

pub const ONLINE: &str = "Verbunden";
pub fn connection_label(state: &str, pending: &str) -> String {
    format!("Verbindung: {state}. {pending}")
}
pub const CONNECTING: &str = "Verbinde …";
pub const OFFLINE: &str = "Offline – Eingaben werden gespeichert und später gesendet";
pub fn pending(n: usize) -> String {
    match n {
        0 => "Alles gesendet".into(),
        1 => "1 Eingabe noch nicht gesendet".into(),
        n => format!("{n} Eingaben noch nicht gesendet"),
    }
}
pub const MY_DEVICES: &str = "Meine Geräte";
pub const PAGE_TABS: &str = "Teile der Seite";
pub const REVOKED_TITLE: &str = "Gerät gesperrt";
pub const REVOKED_TEXT: &str = "Dieses Gerät wurde gesperrt. Sein Schlüssel und alle Einsatzdaten wurden von diesem Gerät gelöscht. Um es wieder zu nutzen, das Gerät neu koppeln.";
pub const PAIR_AGAIN: &str = "Neu koppeln";

pub const DEVICE_NAME_LABEL: &str = "Name dieses Geräts";
pub const DEVICE_NAME_HINT: &str =
    "So erscheint das Gerät in der Geräteliste, z. B. „Tablet UHS Nord“.";

pub fn device_revoked(short: &str) -> String {
    format!("Gerät …{short} gesperrt")
}
pub fn retagged(old: &str, new: &str) -> String {
    format!("Patient {old}: neue Kartennummer {new}")
}

pub fn registered(tag: &str, category: TriageCategory, uhs: &str) -> String {
    format!("Patient {tag} registriert: {}, {uhs}", label(category))
}

pub fn uhs_created(name: &str) -> String {
    format!("{name} angelegt")
}

pub fn unit_added(unit: &str) -> String {
    format!("Einsatzmittel {unit} hinzugefügt")
}

pub fn status_set(unit: &str, status: &str) -> String {
    format!("{unit}: Status {}", status_label(status))
}

pub const LAUNCHER: &str = "Arbeitsbereich wechseln";
pub const VIEWS: &str = "Ansichten";
pub const INCIDENT: &str = "Einsatz";
pub const REASON_NO_ACCESS: &str = "Kein Zugriff in diesem Einsatz";
pub const REASON_NOT_CONNECTED: &str = "Verfügbar nach der ersten Verbindung";
pub const NO_WORKSPACE: &str =
    "Für dieses Gerät ist in diesem Einsatz noch kein Arbeitsbereich freigegeben.";

pub fn status_label(code: &str) -> String {
    crate::shared::units::STATUS
        .get(code)
        .map_or_else(|| code.to_owned(), |c| format!("{} – {}", c.code, c.label))
}

pub fn retriaged(tag: &str, category: TriageCategory) -> String {
    format!("Patient {tag} nachgesichtet: {}", label(category))
}

/// Label of a triage category in the DE profile.
pub fn label(category: TriageCategory) -> &'static str {
    match category {
        TriageCategory::Immediate => "SK I",
        TriageCategory::Urgent => "SK II",
        TriageCategory::Minor => "SK III",
        TriageCategory::Expectant => "SK IV",
        TriageCategory::Deceased => "Verstorben",
        TriageCategory::Uninjured => "Betroffene",
    }
}

// Navigation and header
pub const MENU: &str = "Menü";
pub const SEARCH: &str = "Suchen";
pub const STORAGE_FAILED: &str = "Gerätespeicher voll oder gesperrt: Eingaben bleiben nur bis zum Senden erhalten. Nicht neu laden.";

// Appearance (UX-3)
pub const APPEARANCE: &str = "Darstellung";
pub const THEMES: [(&str, &str); 4] = [
    ("", "Automatisch"),
    ("day", "Tag"),
    ("night", "Nacht"),
    ("high-contrast", "Hoher Kontrast"),
];
pub const PROFILE: &str = "Profil und Gerät";
pub const HAND: &str = "Bedienung mit";
pub const HANDS: [(&str, &str); 2] = [("", "rechter Hand"), ("left", "linker Hand")];
// Vim keys (ADR-0090). Key names stay English (ADR-0036).
pub const KEYMAP: &str = "Tastatur";
pub const KEYMAPS: [(&str, &str); 2] = [("", "Standard"), ("vim", "Vim-Tasten")];
pub const KEYS_TITLE: &str = "Tastenkürzel";
pub const KEYS_ON: &str = "Vim-Tasten sind auf diesem Gerät an. Sie wirken, solange kein Textfeld den Fokus hat, und ändern nie Daten.";
pub const KEYS_OFF: &str = "Vim-Tasten sind aus. Einschalten im Menü unter „Tastatur“.";
pub const KEYS_VIM: &str = "Vim-Tasten";
pub const KEYS_TURN_ON: &str = "Vim-Tasten einschalten";
pub const KEYS_TURN_OFF: &str = "Vim-Tasten ausschalten";
pub const KEYS_ALWAYS: &str = "Immer";
pub const KEY_KEYS: &str = "Tasten";
pub const KEY_DOES: &str = "Wirkung";
pub const KEY_HELP: &str = "Diese Übersicht";
pub const KEY_PALETTE: &str = "Befehlspalette";
pub const KEY_SEARCH: &str = "Suchen oder filtern";
pub const KEY_MOVE: &str =
    "Zum nächsten Element links, darunter, darüber, rechts – auf der ganzen Seite";
pub const KEY_FIRST_LAST: &str = "Erstes / letztes Element der Seite";
pub const KEY_TABS: &str = "Nächster / vorheriger Reiter";
pub const KEY_VIEWS: &str = "Erste bis vierte Ansicht";
pub const KEY_MENU: &str = "Menü";
pub const KEY_OPEN: &str = "Eintrag öffnen";
pub const KEY_INSERT: &str = "Ins erste Eingabefeld";
pub const KEY_SCROLL: &str = "Halbe Seite nach unten / oben";
pub const KEY_ESCAPE: &str = "Textfeld verlassen";
pub const KEY_IN_LIST: &str = "In einer Liste: nächster / vorheriger Eintrag";
pub const KEY_CLOSE: &str = "Dialog oder Menü schließen";
pub fn theme_set(label: &str) -> String {
    format!("Darstellung: {label}")
}
pub fn photo_added(tag: &str) -> String {
    format!("Foto für Patient {tag} gespeichert")
}

pub fn photo_removed(tag: &str) -> String {
    format!("Foto für Patient {tag} entfernt")
}

pub fn photo_restored(tag: &str) -> String {
    format!("Foto von Patient {tag} wiederhergestellt")
}
pub const PHOTO_FAILED: &str = "Das Bild konnte nicht gelesen werden.";
pub const PHOTO_TOO_LARGE: &str = "Das Bild ist auch verkleinert zu groß.";

pub const EDIT: &str = "Bearbeiten";
pub const SAVE: &str = "Speichern";
pub const CANCEL: &str = "Abbrechen";

// Command palette (UX-10)
pub const PALETTE_TITLE: &str = "Suchen und Befehle";
pub const PALETTE_PLACEHOLDER: &str = "Patient, Einsatzmittel, Ansicht oder Befehl …";
pub const PALETTE_RESULTS: &str = "Ergebnisse";
pub const PALETTE_NONE: &str = "Nichts gefunden.";
pub const PALETTE_KEYS: &str = "wählen";
pub const PALETTE_RUN: &str = "ausführen";
pub const PALETTE_CLOSE: &str = "schließen";
pub const PALETTE_BACK: &str = "zurück";
pub fn palette_count(n: usize) -> String {
    match n {
        0 => PALETTE_NONE.into(),
        1 => "1 Ergebnis".into(),
        n => format!("{n} Ergebnisse"),
    }
}
pub const GROUP_VIEWS: &str = "Ansicht";
pub const GROUP_ACTIONS: &str = "Befehl";
pub const MY_ROLES: &str = "Rollen";
pub const NO_ROLE: &str = "keine (Einsatzplanung)";
pub const CHANGE_INCIDENT: &str = "Einsatz wechseln";

/// Roles offered when granting (prototype; later from the profile).
pub const ROLES: [(&str, &str); 4] = [
    ("commander", "Einsatzleitung"),
    ("medic", "Sanitätsdienst"),
    ("physician", "Arzt/Ärztin"),
    ("paramedic", "Notfallsanitäter/in"),
];

pub fn role_label(role: &str) -> String {
    ROLES
        .iter()
        .find(|(id, _)| *id == role)
        .map_or_else(|| role.to_owned(), |(_, label)| (*label).to_owned())
}
pub fn incident_paused(paused: bool) -> &'static str {
    if paused {
        "Einsatz unterbrochen"
    } else {
        "Einsatz fortgesetzt"
    }
}
pub const INCIDENT_DETAILS_SAVED: &str = "Angaben zum Einsatz gespeichert.";
pub const INCIDENT_TIMES_INVALID: &str =
    "Das Ende liegt vor dem Beginn, oder ein Datum ist unvollständig.";

pub fn state_label(state: ordo_sync::incidents::IncidentState) -> &'static str {
    use ordo_sync::incidents::IncidentState::*;
    match state {
        Planned => "Geplant",
        Active => "Laufend",
        Closed => "Abgeschlossen",
    }
}

pub fn role_granted(user: &str, role: &str) -> String {
    format!("Rolle {} an {user} vergeben", role_label(role))
}

pub fn role_withdrawn(user: &str, role: &str) -> String {
    format!("Rolle {} von {user} entzogen", role_label(role))
}

pub fn incident_state_changed(state: ordo_sync::incidents::IncidentState) -> String {
    format!("Einsatz: {}", state_label(state))
}

pub fn priority_changed(tag: &str, set: bool) -> String {
    if set {
        format!("{tag}: Transportpriorität gesetzt")
    } else {
        format!("{tag}: Transportpriorität entfernt")
    }
}

pub fn place_label(n: u32) -> String {
    format!("Platz {n}")
}

/// A place kept for a triage category, e.g. "SK I – Platz 2" (UHS-13).
pub fn category_place_label(category: TriageCategory, n: u32) -> String {
    format!("{} – Platz {n}", label(category))
}

pub fn places_set(n: u32) -> String {
    format!("{n} Behandlungsplätze")
}

pub fn place_assigned(tag: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => format!("{tag} auf {l}"),
        None => format!("{tag}: kein Behandlungsplatz"),
    }
}
pub const TRANSPORT_HISTORY: &str = "Verlauf";

pub fn transport_status(status: ordo_med_patients::transport::TransportStatus) -> &'static str {
    use ordo_med_patients::transport::TransportStatus as S;
    match status {
        S::Requested => "Angefordert",
        S::Assigned => "Zugewiesen",
        S::Loading => "Wird verladen",
        S::Departed => "Abgefahren",
        S::Arrived => "Am Ziel",
        S::HandedOver => "Patient am Ziel übergeben",
        S::Cancelled => "Storniert",
    }
}

pub fn transport_requested(tag: &str) -> String {
    format!("{tag}: Transport angefordert")
}

pub fn transport_status_set(status: ordo_med_patients::transport::TransportStatus) -> String {
    format!("Transport: {}", transport_status(status))
}

pub fn order_status(status: ordo_med_patients::orders::OrderStatus) -> &'static str {
    use ordo_med_patients::orders::OrderStatus as S;
    match status {
        S::Given => "Angeordnet",
        S::Acknowledged => "Übernommen",
        S::Done => "Durchgeführt",
        S::Withdrawn => "Zurückgenommen",
    }
}

pub fn order_given(tag: &str) -> String {
    format!("{tag}: Anordnung erfasst")
}

pub fn order_status_set(status: ordo_med_patients::orders::OrderStatus) -> String {
    format!("Anordnung: {}", order_status(status))
}
pub const ORDER_CONFIRMED_SAID: &str = "Anordnung bestätigt";

pub fn order_recorded(tag: &str) -> String {
    format!("{tag}: Mündliche Anordnung erfasst")
}
pub const SECTION_CLOSE: &str = "Schließen";
/// Closes a dialog.
pub const CLOSE: &str = "Schließen";
pub const DONE: &str = "Fertig";
pub const DISCHARGE: &str = "Entlassen";

pub fn outcome(outcome: ordo_med_patients::Outcome) -> &'static str {
    use ordo_med_patients::Outcome as O;
    match outcome {
        O::ReleasedOnSite => "Vor Ort entlassen",
        O::Transported => "Transportiert",
        O::Refused => "Behandlung oder Transport verweigert",
        O::Deceased => "Verstorben",
    }
}

/// "Entlassen: Transportiert"
pub fn discharged_as(outcome: ordo_med_patients::Outcome) -> String {
    format!("{DISCHARGE}: {}", self::outcome(outcome))
}

pub fn discharged(tag: &str, outcome: ordo_med_patients::Outcome) -> String {
    format!("{tag}: {}", discharged_as(outcome))
}

pub fn readmitted(tag: &str) -> String {
    format!("{tag} wieder aufgenommen")
}

pub fn hospital_saved(name: &str) -> String {
    format!("Klinik gespeichert: {name}")
}

pub fn capacity_saved(name: &str) -> String {
    format!("Aufnahme gespeichert: {name}")
}

// API versions (ADR-0052).
pub const VERSION_UNSUPPORTED: &str =
    "Diese App und der Server haben keine gemeinsame Version. Bitte die Seite neu laden.";

pub fn crew_set(unit: &str) -> String {
    format!("Besatzung von {unit} gespeichert")
}

pub fn unit_released(unit: &str) -> String {
    format!("{unit} aus dem Einsatz entlassen")
}

/// How long ago, from milliseconds: "gerade eben", "vor 5 Min.", "vor 2 Std. 5 Min.",
/// "vor 3 Tagen".
pub fn ago(ms: u64) -> String {
    let minutes = ms / 60_000;
    match minutes {
        0 => "gerade eben".to_owned(),
        1..=59 => format!("vor {minutes} Min."),
        60..=1439 => match minutes % 60 {
            0 => format!("vor {} Std.", minutes / 60),
            m => format!("vor {} Std. {m} Min.", minutes / 60),
        },
        _ => match minutes / 1440 {
            1 => "vor 1 Tag".to_owned(),
            d => format!("vor {d} Tagen"),
        },
    }
}
/// The patient counts of patients with units' crews.
pub const WITH_UNITS: &str = "Bei Einsatzmitteln";
pub fn handed_over(tag: &str, to: &str) -> String {
    format!("Patient {tag} an {to} übergeben, Übernahme offen")
}
pub fn handed_over_quick(tag: &str) -> String {
    format!("Schnellübergabe für Patient {tag} gestartet")
}
pub fn handed_outside(tag: &str, receiver: &str) -> String {
    format!("Patient {tag} außerhalb von ORDO übergeben an {receiver}")
}
pub fn taken(tag: &str, by: &str) -> String {
    format!("Patient {tag} übernommen von {by}")
}

pub fn uhs_closed(name: &str) -> String {
    format!("{name} aufgelöst")
}

pub fn uhs_updated(name: &str) -> String {
    format!("{name} gespeichert")
}

// What needs the command's attention (TASK-3): the bell in the header.
pub const ATTENTION: &str = "Hinweise";
pub const ATTENTION_NONE: &str = "Nichts, das gerade Aufmerksamkeit braucht.";
pub fn attention_label(n: usize) -> String {
    match n {
        0 => format!("{ATTENTION}: keine"),
        n => format!("{ATTENTION}: {n}"),
    }
}

// Reports as Aufträge (REP-1…5, ADR-0081); events of ADR-0075 still read.
pub const REPORT_CLOSED: &str = "Meldung abgeschlossen";
pub const TASK_GIVEN: &str = "Auftrag gegeben";
pub const TASK_PATIENT_TAKEN: &str = "Patient aus dem Auftrag aufgenommen";
pub const UNDO: &str = "Rückgängig";

pub fn close_label(what: &str) -> String {
    format!("{SECTION_CLOSE}: {what}")
}

pub fn task_recorded(place: &str) -> String {
    format!("Auftrag angelegt: {place}")
}
pub const FOLLOW_UP_LATER_DONE: &str = "Nachfrage verschoben";

pub fn task_status(status: ordo_tasks::TaskStatus) -> &'static str {
    use ordo_tasks::TaskStatus as S;
    match status {
        S::Open => "Offen",
        S::InProgress => "In Arbeit",
        S::Done => "Erledigt",
        S::Cancelled => "Abgebrochen",
    }
}

pub fn task_given(unit: &str) -> String {
    format!("Auftrag an {unit} gegeben")
}

pub fn task_status_set(status: ordo_tasks::TaskStatus) -> String {
    format!("Auftrag: {}", task_status(status))
}
pub const STRUCTURE_SAVED: &str = "Führungsstruktur gespeichert";

pub fn position_created(name: &str) -> String {
    format!("{name} angelegt")
}
pub const MAP_SAVED: &str = "Lagekarte gespeichert";

// Address search (ADR-0066)
pub const ADDRESS_LABEL: &str = "Adresse suchen";
pub const ADDRESS_SUBMIT: &str = "Suchen";
pub const ADDRESS_SEARCHING: &str = "Suche läuft …";
pub const ADDRESS_EMPTY: &str = "Bitte eine Adresse eingeben.";
pub const ADDRESS_TOO_LONG: &str = "Bitte höchstens 200 Zeichen eingeben.";
pub const ADDRESS_OFFLINE: &str = "Die Adresssuche braucht eine Verbindung zum Server.";
pub const ADDRESS_NOT_FOUND: &str =
    "Nichts gefunden. Bitte anders schreiben, zum Beispiel mit dem Ort.";
pub const ADDRESS_OFF: &str = "Auf diesem Server ist keine Adresssuche eingerichtet.";
pub const ADDRESS_REFUSED: &str = "Die Adresssuche ist für die Einsatzleitung.";
pub const ADDRESS_FAILED: &str =
    "Die Adresssuche antwortet gerade nicht. Bitte später erneut versuchen.";
pub const ADDRESS_PLACES: &str = "Gefundene Orte";
pub const ADDRESS_ATTRIBUTION: &str =
    "Adressen aus OpenStreetMap, © OpenStreetMap-Mitwirkende (ODbL).";
pub fn address_found(n: usize) -> String {
    match n {
        1 => "1 Ort gefunden. Bitte wählen.".to_owned(),
        n => format!("{n} Orte gefunden. Bitte wählen."),
    }
}
pub fn address_chosen(label: &str) -> String {
    format!("Gewählt: {label}")
}
pub const NOTICES_BACK: &str = "Zurück";

// Stored data that cannot be used (local persistence)
pub fn stored_state_unusable(reason: &str) -> String {
    format!(
        "Die Daten auf diesem Gerät können von dieser Version nicht gelesen werden ({reason}). Sie bleiben unverändert erhalten; bitte die App neu laden oder die Administration verständigen."
    )
}

// Display names (ORG-7)
pub const DISPLAY_NAME: &str = "Anzeigename";
pub const DISPLAY_NAME_HINT: &str =
    "So sehen andere Sie in ORDO. Ohne eigenen Namen gilt der Name aus der Anmeldung.";
pub const DISPLAY_NAME_SAVE: &str = "Namen speichern";
pub const DISPLAY_NAME_RESET: &str = "Namen aus der Anmeldung verwenden";
pub const GUEST_PAIR_TITLE: &str = "Zugang für Helfende";
pub const GUEST_ENDED_TITLE: &str = "Zugang beendet";
pub const GUEST_ENDED: &str = "Ihr Zugang zu diesem Einsatz ist mit Ihrem Dienst beendet. Für einen neuen Zugang bitte die Einsatzleitung fragen.";
pub const GUEST_PAIRING: &str = "Dieses Gerät wird mit dem QR-Code gekoppelt …";
pub const GUEST_CODE_INVALID: &str = "Der QR-Code gilt nicht mehr (schon benutzt, abgelaufen oder Dienst beendet). Bitte die Einsatzleitung um einen neuen.";
pub fn helper_saved(name: &str) -> String {
    format!("Angaben zu {name} gespeichert.")
}

#[cfg(test)]
mod tests {
    // [utest->dsn~record-overview~2]
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
}
