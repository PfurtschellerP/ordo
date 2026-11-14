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

pub const TAB_MED: &str = "Sanität";
pub const TAB_COMMAND: &str = "Einsatzleitung";
pub const TAB_ADMIN: &str = "Administration";
pub const TAB_PERSON: &str = "Person";
pub const MY_DEVICES: &str = "Meine Geräte";
/// Workspaces of the incident command, by phase (ADR-0061).
pub const WS_LEAD: &str = "Führung";
pub const WS_PREPARE: &str = "Vorbereitung";
pub const DISPATCH: &str = "Disposition";
pub const PEOPLE: &str = "Personal";
pub const WS_ACCESS: &str = "Zugriff";
pub const WS_INFO: &str = "Info";
/// The UHS as a unit (UHS-12).
pub const UHS_STRENGTH: &str = "Stärke der UHS";
pub const UHS_STRENGTH_HINT: &str = "Die UHS zählt bei den Kräften vor Ort als Einsatzmittel: mit dieser Stärke, sonst mit ihrem Personal.";
pub const INCIDENT_PAGE: &str = "Einsatz";

pub const DEVICES: &str = "Geräte";
pub const DEVICES_INTRO_OWN: &str = "Die eigenen gekoppelten Geräte. Ein verlorenes oder gestohlenes Gerät hier sperren: Es wird sofort getrennt und löscht seine Daten, sobald es sich wieder verbindet.";
pub const DEVICES_INTRO_ALL: &str = "Alle gekoppelten Geräte der Organisation. Ein verlorenes oder gestohlenes Gerät hier sperren: Es wird sofort getrennt und löscht seine Daten, sobald es sich wieder verbindet.";
pub const DEVICES_OFFLINE: &str = "Die Geräteliste ist nur mit Verbindung zum Server verfügbar.";
pub const DEVICES_LOADING: &str = "Geräteliste wird geladen …";
pub const DEVICES_EMPTY: &str = "Keine gekoppelten Geräte.";
pub const THIS_DEVICE: &str = "dieses Gerät";
pub const DEVICE_PERSON: &str = "Person";
pub const DEVICE_PAIRED: &str = "Gekoppelt";
pub const DEVICE_VALID_UNTIL: &str = "Anmeldung gültig bis";
pub const DEVICE_CONNECTION: &str = "Verbindung";
pub const DEVICE_CONNECTED: &str = "verbunden";
pub const DEVICE_NOT_CONNECTED: &str = "nicht verbunden";
/// Devices used in the incident, on *Vorbereitung – Personal* (ORG-12).
pub const INCIDENT_DEVICES: &str = "Geräte im Einsatz";
pub const INCIDENT_DEVICES_INTRO: &str = "Die Geräte aller Personen mit einer Rolle in diesem Einsatz. Ein verlorenes oder gestohlenes Gerät hier sperren: Es wird sofort getrennt und löscht alle ORDO-Daten, sobald es sich wieder verbindet, auch die anderer Einsätze.";
pub const INCIDENT_DEVICES_EMPTY: &str =
    "Noch keine Geräte: Niemand mit einer Rolle in diesem Einsatz hat ein gekoppeltes Gerät.";
pub const UNKNOWN: &str = "unbekannt";
pub const REVOKE: &str = "Sperren";
pub const REVOKE_CONFIRM: &str = "Ja, sperren";
pub const REVOKED_TITLE: &str = "Gerät gesperrt";
pub const REVOKED_TEXT: &str = "Dieses Gerät wurde gesperrt. Sein Schlüssel und alle Einsatzdaten wurden von diesem Gerät gelöscht. Um es wieder zu nutzen, das Gerät neu koppeln.";
pub const PAIR_AGAIN: &str = "Neu koppeln";

pub fn device_name(short: &str) -> String {
    format!("Gerät …{short}")
}

/// "Firefox auf Android · …4F2A9C": the device's name and the end of its ID (ORG-8).
pub fn named_device(name: &str, short: &str) -> String {
    format!("{name} · …{short}")
}

pub const DEVICE_NAME_LABEL: &str = "Name dieses Geräts";
pub const DEVICE_NAME_HINT: &str =
    "So erscheint das Gerät in der Geräteliste, z. B. „Tablet UHS Nord“.";

pub fn revoke_label(short: &str) -> String {
    format!("Sperren: Gerät …{short}")
}

pub fn revoke_question(short: &str, own: bool) -> String {
    if own {
        format!(
            "Dieses Gerät (…{short}) sperren? Es wird sofort getrennt, und alle Daten darauf werden gelöscht, auch noch nicht gesendete Eingaben."
        )
    } else {
        format!(
            "Gerät …{short} sperren? Es wird sofort getrennt und löscht alle Daten darauf, sobald es sich verbindet. Noch nicht gesendete Eingaben auf dem Gerät gehen verloren."
        )
    }
}

pub fn device_revoked(short: &str) -> String {
    format!("Gerät …{short} gesperrt")
}

pub const REGISTER_TITLE: &str = "Patient registrieren";
pub const REGISTER_TAG: &str = "Nummer der Patientenanhängekarte";
pub const REGISTER_CATEGORY: &str = "Sichtungskategorie";
pub const REGISTER_UHS: &str = "UHS";
pub const REGISTER_SUBMIT: &str = "Registrieren";

pub const BOARD_EMPTY: &str = "Noch keine Patienten registriert.";
pub const COL_UHS: &str = "UHS";
pub const COL_RETRIAGE: &str = "Nachsichtung";
pub const NOT_SENT: &str = "noch nicht gesendet";
pub const SENT: &str = "gesendet";

pub const COUNTS_TITLE: &str = "Patienten je UHS";
pub const COUNTS_EMPTY: &str = "Noch keine Patientenzahlen.";
pub const COL_TOTAL: &str = "Gesamt";
pub const COL_DISCHARGED: &str = "Entlassen";

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

pub const TAB_RESOURCES: &str = "Einsatzmittel";

pub const LAUNCHER: &str = "Arbeitsbereich wechseln";
pub const VIEWS: &str = "Ansichten";
pub const INCIDENT: &str = "Einsatz";
pub const REASON_NO_ACCESS: &str = "Kein Zugriff in diesem Einsatz";
pub const REASON_NOT_CONNECTED: &str = "Verfügbar nach der ersten Verbindung";
pub const NO_WORKSPACE: &str =
    "Für dieses Gerät ist in diesem Einsatz noch kein Arbeitsbereich freigegeben.";

pub const UHS_TITLE: &str = "Unfallhilfsstellen";
pub const UHS_EMPTY: &str = "Noch keine UHS angelegt.";
pub const UHS_NAME: &str = "Name der UHS";
pub const UHS_CAPACITY: &str = "Kapazität (Patienten)";
pub const UHS_SUBMIT: &str = "UHS anlegen";
pub const UHS_CHOOSE: &str = "UHS wählen";
pub const NO_UHS: &str = "Zuerst eine UHS anlegen.";
pub const COL_LOAD: &str = "Belegung";
pub fn load(patients: u32, capacity: u32) -> String {
    format!("{patients} / {capacity}")
}

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

pub fn status_label(code: &str) -> String {
    crate::units::STATUS
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

/// Spoken description, so screen readers convey the meaning, not only the label.
pub fn description(category: TriageCategory) -> &'static str {
    match category {
        TriageCategory::Immediate => "Sichtungskategorie I, rot, sofortige Behandlung",
        TriageCategory::Urgent => "Sichtungskategorie II, gelb, dringlich",
        TriageCategory::Minor => "Sichtungskategorie III, grün, leicht verletzt",
        TriageCategory::Expectant => "Sichtungskategorie IV, blau, abwartende Behandlung",
        TriageCategory::Deceased => "Verstorben",
        TriageCategory::Uninjured => "Betroffene Person, unverletzt, ohne Behandlungsbedarf",
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
pub fn theme_set(label: &str) -> String {
    format!("Darstellung: {label}")
}

// Patient list and record
pub const PATIENTS_TITLE: &str = "Patienten";
pub const ALL_PATIENTS: &str = "Alle Patienten";
pub const REGISTERED: &str = "registriert.";
pub const CHOOSE_PATIENT: &str = "Patient in der Liste wählen, um die Akte zu öffnen.";
pub const UNKNOWN_PERSON: &str = "Person unbekannt";
pub fn patient_title(tag: &str) -> String {
    format!("Patient {tag}")
}
pub fn triage_count(n: usize) -> String {
    match n {
        1 => "1 Sichtung".into(),
        n => format!("{n} Sichtungen"),
    }
}
pub const NOT_FOUND: &str = "Diesen Patienten gibt es in diesem Einsatz nicht.";
pub const SECTIONS: &str = "Abschnitte";
pub const TRIAGE_TITLE: &str = "Sichtung";
pub const TRIAGE_HISTORY: &str = "Verlauf";
pub const RETRIAGE_TO: &str = "Sichtung ändern";
pub fn retriage_label(tag: &str, category: TriageCategory) -> String {
    format!("{COL_RETRIAGE} {tag}: {}", label(category))
}
pub fn by_at(user: &str, time: &str) -> String {
    format!("{time}, {user}")
}

pub const PHOTO_TITLE: &str = "Foto";
pub const PHOTO_NONE: &str = "Noch kein Foto.";
pub const PHOTO_TAKE: &str = "Foto aufnehmen";
pub const PHOTO_UPLOAD: &str = "Bild hochladen";
pub const PHOTO_BUSY: &str = "Foto wird verkleinert …";
pub const PHOTO_PREVIEW_LOADING: &str = "Vorschau – das volle Foto wird geladen …";
pub const PHOTO_PREVIEW_UNAVAILABLE: &str =
    "Vorschau – das volle Foto ist noch nicht auf dem Server. Es wird später erneut geladen.";
pub const PHOTO_PURPOSE: &str = "Nur zur Identifikation. Das Foto wird verkleinert, ohne Kamera- und Ortsdaten gespeichert und nur an Berechtigte übertragen.";
pub fn photo_alt(tag: &str) -> String {
    format!("Foto von Patient {tag}")
}
pub fn photo_added(tag: &str) -> String {
    format!("Foto für Patient {tag} gespeichert")
}
pub const PHOTO_FAILED: &str = "Das Bild konnte nicht gelesen werden.";
pub const PHOTO_TOO_LARGE: &str = "Das Bild ist auch verkleinert zu groß.";

pub const EDIT: &str = "Bearbeiten";
pub const SAVE: &str = "Speichern";
pub const CANCEL: &str = "Abbrechen";
pub const NEW_ENTRY: &str = "Neuer Eintrag";
pub const CORRECT: &str = "Korrigieren";
pub const NOTHING_DOCUMENTED: &str = "Noch nichts dokumentiert.";
pub const NO_ANSWER: &str = "keine Angabe";
/// The free-text answer of an open choice (EPCR-12).
pub const OTHER_ANSWER: &str = "Sonstiges";
pub const NOW: &str = "Jetzt";
pub const EMPTY_ENTRY: &str = "Bitte mindestens einen Wert eintragen.";
pub const NO_CHANGES: &str = "Keine Änderungen.";
pub const CHECK_INPUT: &str = "Bitte die markierten Felder prüfen.";
pub fn corrected_entries(n: usize) -> String {
    match n {
        1 => "1 korrigierter Eintrag".into(),
        n => format!("{n} korrigierte Einträge"),
    }
}
pub const CORRECTED: &str = "korrigiert";
pub fn entry_saved(form: &str, tag: &str) -> String {
    format!("{form} für Patient {tag} gespeichert")
}
pub fn new_entry_label(form: &str) -> String {
    format!("{NEW_ENTRY}: {form}")
}
pub fn edit_label(form: &str) -> String {
    format!("{form} {}", EDIT.to_lowercase())
}
pub fn value_error(error: &ordo_med_patients::record::ValueError) -> String {
    use ordo_med_patients::record::ValueError as E;
    match error {
        E::NotANumber => "Bitte eine Zahl eingeben.".into(),
        E::OutOfRange { min, max } => match (min, max) {
            (Some(min), Some(max)) => format!("Bitte einen Wert von {min} bis {max} eingeben."),
            (Some(min), None) => format!("Bitte mindestens {min} eingeben."),
            (None, Some(max)) => format!("Bitte höchstens {max} eingeben."),
            (None, None) => "Wert nicht möglich.".into(),
        },
        E::NotADate => "Bitte ein Datum eingeben.".into(),
        E::NotATime => "Bitte eine Uhrzeit (hh:mm) eingeben.".into(),
        E::TooLong => "Der Text ist zu lang.".into(),
        E::UnknownField | E::UnknownOption(_) | E::WrongKind => "Wert nicht möglich.".into(),
    }
}

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
pub const GROUP_PATIENTS: &str = "Patient";
pub const GROUP_UNITS: &str = "Einsatzmittel";
pub const ACTION_REGISTER: &str = "Patient registrieren";
pub const ACTION_CREATE_UHS: &str = "UHS anlegen";
pub const ACTION_ADD_UNIT: &str = "Einsatzmittel hinzufügen";
pub const ACTION_OPEN_RECORD: &str = "Akte öffnen";
pub const ACTION_TAKE_PHOTO: &str = "Foto aufnehmen";
pub fn action_entry(form: &str) -> String {
    format!("{form} eintragen")
}
pub fn action_retriage(category: TriageCategory) -> String {
    format!("Sichtung ändern: {}", label(category))
}
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
pub const MY_ROLES: &str = "Rollen";
pub const NO_ROLE: &str = "keine (Einsatzplanung)";
pub const PLAN_TITLE: &str = "Einsatz planen";
pub const PLAN_NAME: &str = "Name des Einsatzes";
pub const PLAN_SUBMIT: &str = "Einsatz anlegen";
pub const CHANGE_INCIDENT: &str = "Einsatz wechseln";
pub const GRANT: &str = "Rolle vergeben";
pub const WITHDRAW: &str = "Entziehen";
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
pub const UHS_TILE: &str = "UHS";
pub const QUICK_LINKS: &str = "Schnellzugriff";
pub const BY_CATEGORY: &str = "Nach Sichtungskategorie";
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
/// "7 von 10 belegt · 3 frei", or "Patienten" without a capacity.
pub fn uhs_load(patients: u32, capacity: u32) -> String {
    if capacity == 0 {
        if patients == 1 {
            "Patient".to_owned()
        } else {
            "Patienten".to_owned()
        }
    } else {
        format!(
            "von {capacity} belegt · {} frei",
            capacity.saturating_sub(patients)
        )
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
/// The open tasks in short (the strip above the command's views).
pub fn tasks_strip(open: usize, due: usize) -> String {
    match (open, due) {
        (0, _) => "Aufträge: keine offenen".to_owned(),
        (n, 0) => format!("Aufträge: {n} offen"),
        (n, d) => format!("Aufträge: {n} offen, davon {d} fällig"),
    }
}
pub const START_INCIDENT: &str = "Einsatz beginnen";
pub const CLOSE_INCIDENT: &str = "Einsatz abschließen";

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
pub fn incident_paused(paused: bool) -> &'static str {
    if paused {
        "Einsatz unterbrochen"
    } else {
        "Einsatz fortgesetzt"
    }
}
pub const INCIDENT_DETAILS: &str = "Angaben zum Einsatz";
pub const INCIDENT_DESCRIPTION: &str = "Beschreibung";
pub const INCIDENT_DESCRIPTION_HINT: &str =
    "Zum Beispiel Ort, Veranstalter, erwartete Besucher, Besonderheiten.";
pub const INCIDENT_STARTS: &str = "Geplanter Beginn";
pub const INCIDENT_ENDS: &str = "Geplantes Ende";
pub const INCIDENT_DETAILS_SAVE: &str = "Angaben speichern";
pub const INCIDENT_DETAILS_SAVED: &str = "Angaben zum Einsatz gespeichert.";
pub const INCIDENT_TIMES_INVALID: &str =
    "Das Ende liegt vor dem Beginn, oder ein Datum ist unvollständig.";
/// "13.11.2026, 18:00 bis 15.11.2026, 23:00"
pub fn incident_times(starts: Option<&str>, ends: Option<&str>) -> Option<String> {
    match (starts, ends) {
        (Some(s), Some(e)) => Some(format!("{s} bis {e}")),
        (Some(s), None) => Some(format!("ab {s}")),
        (None, Some(e)) => Some(format!("bis {e}")),
        (None, None) => None,
    }
}

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

pub fn withdraw_label(user: &str, role: &str) -> String {
    format!("Entziehen: {} von {user}", role_label(role))
}

pub fn incident_state_changed(state: ordo_sync::incidents::IncidentState) -> String {
    format!("Einsatz: {}", state_label(state))
}

// Transport priority
pub const TRANSPORT_PRIORITY: &str = "Transportpriorität";
pub const REMOVE_PRIORITY: &str = "Keine Transportpriorität";

pub fn set_priority(label: &str) -> String {
    format!("{label} setzen")
}

pub fn set_priority_label(tag: &str, label: &str) -> String {
    format!("{tag}: {label} setzen")
}

pub fn remove_priority_label(tag: &str) -> String {
    format!("{tag}: keine Transportpriorität")
}

pub fn priority_changed(tag: &str, set: bool) -> String {
    if set {
        format!("{tag}: Transportpriorität gesetzt")
    } else {
        format!("{tag}: Transportpriorität entfernt")
    }
}

// Treatment places
pub const PLACES: &str = "Behandlungsplätze";
pub const PLACE: &str = "Behandlungsplatz";
pub const NO_PLACE: &str = "Kein Platz";
pub const PLACES_SET: &str = "Übernehmen";

pub fn place_label(n: u32) -> String {
    format!("Platz {n}")
}

/// A place kept for a triage category, e.g. "SK I – Platz 2" (UHS-13).
pub fn category_place_label(category: TriageCategory, n: u32) -> String {
    format!("{} – Platz {n}", label(category))
}

// The plan of a UHS's places (UHS-14)
pub const PLACE_FREE: &str = "frei";

pub fn place_plan_title(uhs: &str) -> String {
    format!("Platzplan – {uhs}")
}

// Capacity per triage category (UHS-13)
pub const UHS_CAPACITIES_LEGEND: &str = "Kapazität nach Sichtungskategorie (optional)";
pub const UHS_CAPACITIES_HINT: &str = "Mit Zahlen hier ist die Kapazität ihre Summe, und die UHS bekommt ihre Behandlungsplätze je Sichtungskategorie.";

pub fn places_set(n: u32) -> String {
    format!("{n} Behandlungsplätze")
}

pub fn place_assigned(tag: &str, label: Option<&str>) -> String {
    match label {
        Some(l) => format!("{tag} auf {l}"),
        None => format!("{tag}: kein Behandlungsplatz"),
    }
}

pub fn place_taken(label: &str, by: &str) -> String {
    format!("{label} (belegt: {by})")
}

pub fn places_of(name: &str) -> String {
    format!("Behandlungsplätze {name}")
}

pub fn places_free(free: usize, total: usize) -> String {
    format!("{free} von {total} frei")
}

// Transport requests (TRANS-1, TRANS-3)
pub const TRANSPORTS: &str = "Transporte";
pub const TRANSPORTS_TITLE: &str = "Offene Transporte";
pub const TRANSPORTS_EMPTY: &str = "Keine offenen Transporte.";
pub const TRANSPORT_TITLE: &str = "Transport";
pub const TRANSPORT_TYPE: &str = "Transportmittel";
pub const TRANSPORT_PRIORITY_OF_REQUEST: &str = "Dringlichkeit";
pub const TRANSPORT_NEEDS: &str = "Besonderheiten";
pub const TRANSPORT_REQUEST: &str = "Transport anfordern";
pub const TRANSPORT_CHOOSE: &str = "Bitte Transportmittel und Dringlichkeit wählen.";
pub const TRANSPORT_NONE: &str = "Kein Transport angefordert.";
pub const TRANSPORT_HISTORY: &str = "Verlauf";
pub const TRANSPORT_UNIT: &str = "Einsatzmittel";
pub const TRANSPORT_DESTINATION: &str = "Ziel (z. B. Klinik)";
pub const TRANSPORT_ASSIGN: &str = "Zuweisen";
pub const TRANSPORT_ASSIGN_MISSING: &str = "Bitte Einsatzmittel und Ziel angeben.";
pub const TRANSPORT_NO_UNITS: &str = "Noch keine Einsatzmittel im Einsatz.";
pub const TRANSPORT_CHOOSE_UNIT: &str = "Einsatzmittel wählen";

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

/// Button text that moves a transport to `status`.
pub fn transport_step(status: ordo_med_patients::transport::TransportStatus) -> &'static str {
    use ordo_med_patients::transport::TransportStatus as S;
    match status {
        S::Requested => "Angefordert",
        S::Assigned => "Zuweisen",
        S::Loading => "Verladen",
        S::Departed => "Abgefahren",
        S::Arrived => "Am Ziel",
        S::HandedOver => "Patient am Ziel übergeben",
        S::Cancelled => "Stornieren",
    }
}

/// Accessible name of a step button: the step and which transport.
pub fn transport_step_label(step: &str, transport: &str) -> String {
    format!("{step}: {transport}")
}

/// Short description of a transport for lists and button names.
pub fn transport_summary(transport_type: &str, uhs: &str, at: &str) -> String {
    format!("{transport_type} für {uhs}, {at}")
}

pub fn transport_unit(unit: &str, destination: &str) -> String {
    format!("{unit} → {destination}")
}

pub fn transport_requested(tag: &str) -> String {
    format!("{tag}: Transport angefordert")
}

pub fn transport_status_set(status: ordo_med_patients::transport::TransportStatus) -> String {
    format!("Transport: {}", transport_status(status))
}

// Medical orders (ORDER-1…4)
pub const ORDERS: &str = "Anordnungen";
pub const ORDERS_TITLE: &str = "Offene Anordnungen";
pub const ORDERS_EMPTY: &str = "Keine offenen Anordnungen.";
pub const ORDERS_NONE: &str = "Keine Anordnungen.";
pub const ORDER_GIVE: &str = "Anordnen";
pub const ORDER_KIND: &str = "Art";
pub const ORDER_MEDICATION: &str = "Medikament";
pub const ORDER_MEASURE: &str = "Maßnahme";
pub const ORDER_SUBSTANCE: &str = "Wirkstoff";
pub const ORDER_DOSE: &str = "Dosis mit Einheit";
pub const ORDER_ROUTE: &str = "Applikationsweg";
pub const ORDER_ROUTE_CHOOSE: &str = "Weg wählen";
pub const ORDER_MEASURE_TEXT: &str = "Maßnahme";
pub const ORDER_NOTE: &str = "Hinweis (optional)";
pub const ORDER_INCOMPLETE: &str = "Bitte die Anordnung vollständig angeben.";
pub const ORDER_RECORDS_ONLY: &str =
    "ORDO dokumentiert Anordnungen; Dosis und Indikation prüft die anordnende Person.";

pub fn order_status(status: ordo_med_patients::orders::OrderStatus) -> &'static str {
    use ordo_med_patients::orders::OrderStatus as S;
    match status {
        S::Given => "Angeordnet",
        S::Acknowledged => "Übernommen",
        S::Done => "Durchgeführt",
        S::Withdrawn => "Zurückgenommen",
    }
}

/// Button text that moves an order to `status`.
pub fn order_step(status: ordo_med_patients::orders::OrderStatus) -> &'static str {
    use ordo_med_patients::orders::OrderStatus as S;
    match status {
        S::Given => "Anordnen",
        S::Acknowledged => "Übernehmen",
        S::Done => "Durchgeführt",
        S::Withdrawn => "Zurücknehmen",
    }
}

pub fn order_given(tag: &str) -> String {
    format!("{tag}: Anordnung erfasst")
}

pub fn order_status_set(status: ordo_med_patients::orders::OrderStatus) -> String {
    format!("Anordnung: {}", order_status(status))
}

/// "Angeordnet 10:12 von petra@…"
pub fn order_by(at: &str, by: &str) -> String {
    format!("{at} von {by}")
}

pub const ORDER_PERFORMED_BY: &str = "Durchgeführt von";

pub fn order_performed_by(person: &str) -> String {
    format!("{ORDER_PERFORMED_BY} {person}")
}

/// "Durchgeführt (durch paul@…)", in the history: who carried it out, apart from who
/// reported it.
pub fn order_status_performed(status: &str, person: &str) -> String {
    format!("{status} (durch {person})")
}

// Spoken orders, recorded by the crew and confirmed by their giver (ORDER-6), and who
// ordered a documented medication (ORDER-7).
pub const ORDER_WHO: &str = "Wer ordnet an?";
pub const ORDER_MINE: &str = "Ich ordne an";
pub const ORDER_SPOKEN: &str = "Mündlich, per Funk oder Telefon angeordnet";
pub const ORDER_ORDERED_BY: &str = "Angeordnet von";
pub const ORDER_CHANNEL: &str = "Übermittelt";
pub const ORDER_READ_BACK: &str = "Wiederholt und bestätigt (Closed Loop)";
pub const ORDER_RECORD: &str = "Anordnung erfassen";
pub const ORDER_SPOKEN_HINT: &str =
    "Wer angeordnet hat, bestätigt die Anordnung später in ORDO, sofern er oder sie ORDO nutzt.";
pub const ORDER_UNCONFIRMED: &str = "Noch nicht bestätigt";
pub const ORDER_CONFIRM: &str = "Bestätigen";
pub const ORDER_CONFIRMED_SAID: &str = "Anordnung bestätigt";
pub const ORDERED_BY_FROM_ORDER: &str =
    "Wird aus der Anordnung übernommen. Mündliche Anordnungen zuerst unter „Anordnungen“ erfassen.";
pub const ORDERED_BY_NO_ORDER: &str = "ohne Anordnung in ORDO";

pub fn order_channel(channel: ordo_med_patients::orders::Channel) -> &'static str {
    use ordo_med_patients::orders::Channel as C;
    match channel {
        C::InPerson => "persönlich",
        C::Radio => "per Funk",
        C::Phone => "per Telefon",
        C::Telemedicine => "per Telemedizin",
    }
}

pub fn order_recorded(tag: &str) -> String {
    format!("{tag}: Mündliche Anordnung erfasst")
}

/// "Mündlich angeordnet von Petra Schulz, per Funk, wiederholt und bestätigt"
pub fn order_spoken(giver: &str, channel: &str, read_back: bool) -> String {
    let read_back = if read_back {
        "wiederholt und bestätigt"
    } else {
        "nicht wiederholt"
    };
    format!("Mündlich angeordnet von {giver}, {channel}, {read_back}")
}

/// "Erfasst 10:12 von Anna Becker"
pub fn order_recorded_by(at: &str, by: &str) -> String {
    format!("Erfasst {at} von {by}")
}

/// "Bestätigt 10:20 von Petra Schulz"
pub fn order_confirmed(at: &str, by: &str) -> String {
    format!("Bestätigt {at} von {by}")
}

pub fn order_confirm_label(what: &str) -> String {
    format!("Anordnung bestätigen: {what}")
}

pub fn log_order_confirmed(tag: &str) -> String {
    format!("{tag}: Mündliche Anordnung bestätigt")
}

/// Who ordered a documented medication, taken from its order: "Petra Schulz (Anordnung 10:12)"
pub fn ordered_by_order(giver: &str, at: &str) -> String {
    format!("{giver} (Anordnung {at})")
}

// Admission and discharge (UHS-3).
pub const STAY_TITLE: &str = "Patientenweg";
pub const DISCHARGE_OUTCOME: &str = "Entlassen mit Ergebnis";
pub const DISCHARGE: &str = "Entlassen";
pub const DISCHARGE_CHOOSE: &str = "Bitte das Ergebnis der Entlassung wählen.";
pub const READMIT: &str = "Wieder aufnehmen";
pub const ADMITTED: &str = "Aufgenommen";

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

/// "Entlassene Patienten (2)"
pub fn discharged_list(n: usize) -> String {
    format!("Entlassene Patienten ({n})")
}

/// "Mit dem Transport: RTW → Klinikum Mitte"
pub fn discharge_transport(line: &str) -> String {
    format!("Mit dem Transport: {line}")
}

pub fn discharged(tag: &str, outcome: ordo_med_patients::Outcome) -> String {
    format!("{tag}: {}", discharged_as(outcome))
}

pub fn readmit_label(tag: &str) -> String {
    format!("{tag} {}", READMIT.to_lowercase())
}

pub fn readmitted(tag: &str) -> String {
    format!("{tag} wieder aufgenommen")
}

// Hospitals and the transport board (TRANS-2, TRANS-4).
pub const DESTINATION: &str = "Ziel";
pub const DESTINATION_CHOOSE: &str = "Ziel wählen";
pub const DESTINATION_OTHER_OPTION: &str = "Anderes Ziel (Freitext)";
pub const DESTINATION_OTHER: &str = "Anderes Ziel";
pub const FREE_UNITS_TITLE: &str = "Freie Transportmittel";
pub const FREE_UNITS_EMPTY: &str = "Keine freien Transportmittel.";
pub const HOSPITALS_TITLE: &str = "Kliniken";
pub const HOSPITALS_EMPTY: &str = "Noch keine Kliniken erfasst.";
pub const HOSPITAL_NAME: &str = "Name der Klinik";
pub const HOSPITAL_DEPARTMENTS: &str = "Fachabteilungen";
pub const HOSPITAL_CAPABILITIES: &str = "Versorgungsangebote";
pub const HOSPITAL_ADD: &str = "Klinik hinzufügen";
pub const HOSPITAL_SAVE: &str = "Klinik speichern";
pub const HOSPITAL_EDIT: &str = "Bearbeiten";
pub const HOSPITAL_NAME_MISSING: &str = "Bitte den Namen der Klinik angeben.";
pub const CAPACITY_LEGEND: &str = "Freie Aufnahme (Patienten)";
pub const CAPACITY_SAVE: &str = "Aufnahme speichern";
pub const CAPACITY_NONE: &str = "Aufnahme: nicht gemeldet";
pub const CAPACITY_NUMBERS: &str = "Bitte die Aufnahme als ganze Zahlen angeben.";

/// "Aufnahme: SK I 2 · SK II 4 · SK III – (Stand 10:12, ben@…)"
pub fn capacity_line(parts: &[String], at: &str, by: &str) -> String {
    format!("Aufnahme: {} (Stand {at}, {by})", parts.join(" · "))
}

pub fn sent(n: usize) -> String {
    format!("Transportiert: {n}")
}

pub fn hospital_edit_label(name: &str) -> String {
    format!("{name} bearbeiten")
}

pub fn capacity_save_label(name: &str) -> String {
    format!("{CAPACITY_SAVE}: {name}")
}

pub fn hospital_saved(name: &str) -> String {
    format!("Klinik gespeichert: {name}")
}

pub fn capacity_saved(name: &str) -> String {
    format!("Aufnahme gespeichert: {name}")
}

// Planned transports (TRANS-5).
pub const TRANSPORT_PLANNED: &str = "Geplant für (leer: sofort)";
pub const TRANSPORT_PLANNED_INVALID: &str = "Bitte Datum und Uhrzeit vollständig angeben.";
pub const PLANNED_TITLE: &str = "Geplante Transporte";
pub const PLANNED_EMPTY: &str = "Keine geplanten Transporte.";

pub fn planned_for(at: &str) -> String {
    format!("Geplant für {at}")
}

// Documenting a done order (ORDER-5).
pub const DOCUMENT_ORDER: &str = "In Dokumentation übernehmen";

pub fn document_order_label(what: &str) -> String {
    format!("{DOCUMENT_ORDER}: {what}")
}

// Overview for the medical lead (UHS-8).
pub const OVERVIEW: &str = "Übersicht";
pub const OVERVIEW_TITLE: &str = "Übersicht der UHS";

/// "Offene Anordnungen: 3 (1 nicht bestätigt)"
pub fn open_orders(n: usize, unconfirmed: usize) -> String {
    if unconfirmed == 0 {
        format!("Offene Anordnungen: {n}")
    } else {
        format!("Offene Anordnungen: {n} ({unconfirmed} nicht bestätigt)")
    }
}

pub fn transports_count(open: usize, planned: usize) -> String {
    format!("Transporte: {open} offen, {planned} geplant")
}

// API versions (ADR-0052).
pub const VERSION_UNSUPPORTED: &str =
    "Diese App und der Server haben keine gemeinsame Version. Bitte die Seite neu laden.";

// Patient pathway (PAT-5).
pub const MOVE_TO: &str = "Verlegen nach";
pub const MOVE_CHOOSE: &str = "Einheit wählen";
pub const MOVE: &str = "Verlegen";
pub const TOTAL_TITLE: &str = "Alle Einheiten";

/// "Verlegt: Patientenablage Nord → Behandlungsplatz"
pub fn moved(from: &str, to: &str) -> String {
    format!("Verlegt: {from} → {to}")
}

pub fn patient_moved(tag: &str, to: &str) -> String {
    format!("{tag} verlegt nach {to}")
}

// Triage references (PAT-11, ADR-0045).
pub const TRIAGE_WITH: &str = "Sichtung nach";
pub const TRIAGE_WITH_NONE: &str = "ohne Angabe";
pub const TRIAGE_AID: &str = "Sichtungshilfe";
pub const TRIAGE_SOURCE: &str = "Offizielle Quelle öffnen (neues Fenster)";
pub const TRIAGE_AID_NOTE: &str =
    "Die Kategorie entscheidet die sichtende Person; ORDO schlägt keine vor und prüft keine.";

pub fn triage_stage(stage: ordo_med_patients::triage::Stage) -> &'static str {
    match stage {
        ordo_med_patients::triage::Stage::Pretriage => "Vorsichtung",
        ordo_med_patients::triage::Stage::Triage => "Sichtung",
    }
}

/// "nach PRIOR"
pub fn triage_with(name: &str) -> String {
    format!("nach {name}")
}

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

pub fn crew_set(unit: &str) -> String {
    format!("Besatzung von {unit} gespeichert")
}

pub fn release_label(unit: &str) -> String {
    format!("{unit} aus dem Einsatz entlassen")
}

pub fn unit_released(unit: &str) -> String {
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

// UHS details (UHS-1).
pub const UHS_DETAILS: &str = "Ort und Personal";
pub const UHS_NAME_NEEDED: &str = "Die UHS braucht einen Namen.";
pub const UHS_LOCATION: &str = "Ort";
pub const UHS_ON_MAP: &str = "Auf der Lagekarte";
pub const UHS_NOT_ON_MAP: &str = "noch nicht gesetzt";
pub const UHS_PLACE_ON_MAP: &str = "Auf der Karte setzen";
pub const UHS_MOVE_ON_MAP: &str = "Auf der Karte versetzen";

/// The link's visible text, then the unit: the name contains what is seen.
pub fn uhs_place_label(visible: &str, name: &str) -> String {
    format!("{visible}: {name}")
}
pub const UHS_STAFF: &str = "Personal";
pub const UHS_STAFF_NONE: &str = "Kein Personal eingetragen.";
pub const UHS_RESOURCES_TITLE: &str = "UHS und Personal";

pub fn uhs_staff_count(n: usize) -> String {
    match n {
        0 => UHS_STAFF_NONE.to_owned(),
        1 => "Personal: 1 Person".to_owned(),
        n => format!("Personal: {n} Personen"),
    }
}
pub const UHS_STAFF_ADD: &str = "Zum Personal hinzufügen";
pub const UHS_SAVE: &str = "Speichern";
pub const UHS_CAPACITY_NUMBER: &str = "Bitte die Kapazität als ganze Zahl angeben.";

pub fn uhs_details_label(name: &str) -> String {
    format!("{UHS_DETAILS} von {name}")
}

pub fn staff_remove_label(name: &str) -> String {
    format!("{name} aus dem Personal entfernen")
}

pub fn uhs_updated(name: &str) -> String {
    format!("{name} gespeichert")
}

pub fn uhs_renamed(name: &str) -> String {
    format!("UHS umbenannt in {name}")
}

// Reports (REP-1…5, ADR-0075).
pub const REPORT: &str = "Meldung";
pub const REPORTS_TITLE: &str = "Offene Meldungen";
pub const REPORTS_EMPTY: &str = "Keine offenen Meldungen.";
pub const REPORT_RECORD: &str = "Meldung aufnehmen";
pub const REPORT_KIND: &str = "Art";
pub const REPORT_PLACE: &str = "Ort";
pub const REPORT_TEXT: &str = "Kurzbeschreibung";
pub const REPORT_TEXT_HINT: &str = "Was gemeldet wurde, in wenigen Worten, ohne medizinische Angaben (die gehören in die Patientenakte).";
pub const REPORT_RECEIVED: &str = "Eingang";
pub const REPORT_RECEIVED_HINT: &str = "Leer lassen für jetzt.";
pub const REPORT_KIND_MISSING: &str = "Bitte die Art der Meldung wählen.";
pub const REPORT_PLACE_MISSING: &str = "Bitte den Ort angeben.";
pub const REPORT_TEXT_MISSING: &str = "Bitte kurz beschreiben, was gemeldet wurde.";
pub const REPORT_TOO_LONG: &str = "Ort oder Kurzbeschreibung sind zu lang.";
pub const REPORT_TIME_INVALID: &str = "Bitte eine Uhrzeit angeben, z. B. 14:05.";
pub const REPORT_ASSIGN_TO: &str = "Zuweisen an";
pub const REPORT_ASSIGN: &str = "Zuweisen";
pub const REPORT_UNIT_MISSING: &str = "Bitte ein Einsatzmittel wählen.";
pub const REPORT_PLACE_ON_MAP: &str = "Auf der Karte setzen";
pub const REPORT_MOVE_ON_MAP: &str = "Auf der Karte verschieben";
pub const REPORT_ON_MAP: &str = "Auf der Lagekarte gesetzt";
pub const REPORT_CLOSE: &str = "Abschließen";
pub const REPORT_CLOSED: &str = "Meldung abgeschlossen";
pub const REPORT_PATIENT_TAKEN: &str = "Patient aus der Meldung aufgenommen";
pub const ASSIGNED_CASES: &str = "Gemeldete Patienten für Ihr Einsatzmittel";
pub const CASE_REGISTER: &str = "Patient aufnehmen";
pub const CASE_CANCEL: &str = "Ohne Meldung aufnehmen";
pub const OPEN_IN_MAPS_APP: &str = "In der Karten-App öffnen";

pub fn report_kind(kind: ordo_reports::ReportKind) -> &'static str {
    match kind {
        ordo_reports::ReportKind::Event => "Ereignis",
        ordo_reports::ReportKind::Patient => "Patient",
    }
}

pub fn reports_title(n: usize) -> String {
    format!("{REPORTS_TITLE} ({n})")
}

pub fn report_label(kind: ordo_reports::ReportKind, place: &str) -> String {
    format!("{}: {place}", report_kind(kind))
}

pub fn report_received(at: &str) -> String {
    format!("Eingang {at}")
}

pub fn report_recorded(place: &str) -> String {
    format!("Meldung aufgenommen: {place}")
}

pub fn report_taken(unit: &str, at: &str) -> String {
    format!("Patient aufgenommen von {unit} ({at})")
}

pub fn report_place_label(placed: bool, report: &str) -> String {
    let action = if placed {
        REPORT_MOVE_ON_MAP
    } else {
        REPORT_PLACE_ON_MAP
    };
    format!("{action}: {report}")
}

pub fn report_close_label(report: &str) -> String {
    format!("{REPORT_CLOSE}: {report}")
}

/// The text of the task a report gives a unit.
pub fn report_task(r: &ordo_reports::Recorded) -> String {
    format!("{}: {} – {}", report_kind(r.kind), r.place, r.text)
}

pub fn case_for(unit: &str) -> String {
    format!("Zugewiesen an {unit}")
}

pub fn log_report_recorded(r: &ordo_reports::Recorded) -> String {
    format!(
        "Meldung aufgenommen: {}, {} – {} – {}",
        report_kind(r.kind),
        report_received(&crate::core::clock(r.received_at)),
        r.place,
        r.text
    )
}

pub fn log_report_closed(report: &str) -> String {
    format!("Meldung abgeschlossen: {report}")
}

pub fn log_report_patient(report: &str, unit: &str) -> String {
    format!("Patient aus der Meldung {report} aufgenommen von {unit}")
}

pub fn register_from_case(place: &str, unit: &str) -> String {
    format!("Aus der Meldung „{place}“ für {unit}")
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
pub const TASK_FOLLOW_UP_NUMBER: &str = "Bitte die Minuten als ganze Zahl angeben.";
pub const FOLLOW_UP_LATER: &str = "In 10 Minuten nachfragen";
pub const FOLLOW_UP_LATER_DONE: &str = "Nachfrage verschoben";
pub const DUE: &str = "Fällig";

pub fn tasks_title(n: usize) -> String {
    format!("{TASKS_TITLE} ({n})")
}

pub fn task_status(status: ordo_tasks::TaskStatus) -> &'static str {
    use ordo_tasks::TaskStatus as S;
    match status {
        S::Open => "Offen",
        S::InProgress => "In Arbeit",
        S::Done => "Erledigt",
        S::Cancelled => "Abgebrochen",
    }
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

/// "In Arbeit: RTW 83/1 – Lagemeldung abgeben"
pub fn task_step_label(step: &str, unit: &str, text: &str) -> String {
    format!("{step}: {unit} – {text}")
}

pub fn task_given(unit: &str) -> String {
    format!("Auftrag an {unit} gegeben")
}

pub fn task_status_set(status: ordo_tasks::TaskStatus) -> String {
    format!("Auftrag: {}", task_status(status))
}

/// "Nachfragen um 10:42"
pub fn follow_up_at(at: &str) -> String {
    format!("Nachfragen um {at}")
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
pub const STRUCTURE_EMPTY: &str = "Noch keine Führungsstruktur angelegt.";
pub const STRUCTURE_SAVED: &str = "Führungsstruktur gespeichert";
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

pub fn assigned_units(units: &str) -> String {
    format!("Einsatzmittel: {units}")
}

pub fn assigned_uhs(uhs: &str) -> String {
    format!("UHS: {uhs}")
}

/// Shown, never enforced (ADR-0031).
pub fn span_of_control(n: usize) -> String {
    format!("{n} direkt unterstellt: mehr als 5, Führungsspanne beachten.")
}

pub fn position_close_label(title: &str) -> String {
    format!("{title} auflösen")
}

pub fn position_created(name: &str) -> String {
    format!("{name} angelegt")
}

pub fn position_of(unit: &str) -> String {
    format!("Funktion von {unit}")
}

// Situation map (MAP-1, MAP-2).
pub const MAP_TITLE: &str = "Lagekarte";
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
pub const MAP_LOCATE_FAILED: &str = "Der Standort konnte nicht bestimmt werden.";
pub const MAP_OBJECTS_NONE: &str = "Noch nichts auf der Karte.";
pub const MAP_SHOW: &str = "Zeigen";
pub const MAP_REMOVE: &str = "Entfernen";
pub const MAP_SAVED: &str = "Lagekarte gespeichert";
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

pub fn map_kind(kind: ordo_map::ObjectKind) -> &'static str {
    use ordo_map::ObjectKind as K;
    match kind {
        K::Incident => INCIDENT_LOCATION,
        K::Unit => TAB_RESOURCES,
        K::Uhs => "UHS",
        K::Position => POSITION,
        K::Drawing => "Zeichnung",
        K::Report => REPORT,
    }
}

pub fn map_show_label(label: &str) -> String {
    format!("{label} auf der Karte zeigen")
}

pub fn map_remove_label(label: &str) -> String {
    format!("{label} von der Karte entfernen")
}

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

// Licences and source code (LIC-1)
pub const NOTICES_TITLE: &str = "Lizenzen und Quellcode";
pub const NOTICES_BACK: &str = "Zurück";
pub const NOTICES_ORDO_LICENCE: &str = "ORDO ist freie Software unter der GNU Affero General Public License, Version 3 oder einer späteren Version (AGPL-3.0-or-later). Sie darf genutzt, weitergegeben und verändert werden; wer eine veränderte Fassung im Netz anbietet, muss deren Quellcode ebenso anbieten.";
pub const NOTICES_LICENCE_TEXT: &str = "Lizenztext";
pub const NOTICES_SOURCE: &str = "Quellcode dieser Fassung:";
pub const NOTICES_PARTS: &str = "Weitere Bestandteile";
pub const NOTICES_CRATES: &str = "Rust-Bibliotheken";
pub const NOTICES_CRATES_HINT: &str = "Die Lizenztexte, wie sie den Bibliotheken von Server und App beiliegen, je Text mit den Bibliotheken, für die er gilt.";
pub const NOTICES_CRATES_MISSING: &str = "Die Lizenzhinweise der Rust-Bibliotheken sind in diesem Build nicht enthalten. Welche Bibliotheken in welcher Version genutzt werden, steht im Quellcode (Cargo.lock); offizielle Builds zeigen hier ihre Lizenzhinweise.";
pub const INFO_VERSION: &str = "Version";
pub const INFO_APP: &str = "App";
pub const INFO_SERVER: &str = "Server";
pub const INFO_BUILD_LOCAL: &str = "lokaler Build";
pub const INFO_SERVER_OFFLINE: &str = "Der Server ist gerade nicht erreichbar.";
pub const INFO_LOADING: &str = "Wird abgefragt …";
/// "0.0.0, Build 1a2b3c4"
pub fn info_build(version: &str, commit: Option<&str>) -> String {
    match commit {
        Some(c) => format!("{version}, Build {}", &c[..c.len().min(7)]),
        None => format!("{version}, {INFO_BUILD_LOCAL}"),
    }
}
/// "Protokolle: Abgleich 1.0–1.2, HTTP 1.0"
pub fn info_protocols(sync: &str, http: &str) -> String {
    format!("Protokolle: Abgleich {sync}, HTTP {http}")
}
pub const NOTICES_LOADING: &str = "Lizenzhinweise werden geladen …";
pub const NOTICES_USED_MAP: &str = "Lagekarte";
pub const NOTICES_USED_MAP_FONT: &str = "Schrift der Kartenbeschriftung";
pub const NOTICES_USED_FONT: &str = "Schrift der Bedienoberfläche";
pub const NOTICES_USED_ICONS: &str = "Symbole der Bedienoberfläche";
pub const NOTICES_USED_SYMBOLS: &str = "Taktische Zeichen (gezeichnet beim Erstellen von ORDO)";
pub const NOTICES_USED_STORAGE: &str = "Datenspeicher auf dem Gerät";
pub const NOTICES_USED_MAP_DATA: &str = "Kartendaten, wo eine Kartengrundlage eingerichtet ist";
pub const NOTICES_PUBLIC_DOMAIN: &str = "gemeinfrei (Public Domain)";

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

pub fn log_unknown(module: &str, event_type: &str) -> String {
    format!("{module}: {event_type}")
}

pub fn log_planned(name: &str) -> String {
    format!("Einsatz „{name}“ angelegt")
}

pub fn log_unit_added(unit: &str, unit_type: &str, organisation: &str) -> String {
    format!("{} ({unit_type}, {organisation})", unit_added(unit))
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

pub fn log_places(uhs: &str, n: usize) -> String {
    format!("{uhs}: {n} Behandlungsplätze")
}

pub fn log_transport(kind: &str, uhs: &str) -> String {
    format!("{kind} ab {uhs}")
}

pub fn log_transport_requested(transport: &str) -> String {
    format!("Transport angefordert: {transport}")
}

pub fn log_transport_updated(transport: &str, status: &str, unit: Option<&str>) -> String {
    match unit {
        Some(u) => format!("Transport {transport}: {status} ({u})"),
        None => format!("Transport {transport}: {status}"),
    }
}

pub fn log_order_updated(tag: &str, status: &str) -> String {
    format!("{tag}: Anordnung {status}")
}

// Working UHS (UHS-9, UHS-10)
pub const WORKING_UHS: &str = "Meine UHS";
pub const ALL_UHS: &str = "Alle UHS";

pub fn read_only_uhs(name: &str) -> String {
    format!("{name} (nur lesen)")
}

// Map region from the server (MAP-8)
pub const MAP_REGION_TITLE: &str = "Kartengrundlage";
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

// Organisation roles (ORG-9)
pub const ORG_ROLES: &str = "Berechtigungen";
pub const ORG_ROLES_TITLE: &str = "Berechtigungen in der Organisation";
pub const ORG_ROLES_INTRO: &str = "Wer die Organisation verwaltet oder Einsätze plant. Eine Berechtigung gilt auf allen Geräten der Person, sobald sie vergeben ist, und endet mit dem Entziehen oder am Ende des gewählten Tages. Rollen in einem Einsatz vergibt die Einsatzleitung unter „Vorbereitung – Personal und Rollen“.";
pub const ORG_ROLES_LOADING: &str = "Rollen werden geladen …";
pub const ORG_ROLES_EMPTY: &str = "Noch keine Rollen vergeben.";
pub const ORG_ROLES_OFFLINE: &str =
    "Rollen lassen sich nur mit Verbindung zum Server sehen und ändern.";
pub const ORG_ROLE_GRANT_TITLE: &str = "Rolle vergeben";
pub const ORG_ROLE_PERSON: &str = "Person (Benutzerkennung)";
pub const ORG_ROLE_ROLE: &str = "Rolle";
pub const ORG_ROLE_UNTIL: &str = "Gültig bis (optional)";
pub const ORG_ROLE_UNTIL_HINT: &str = "Ohne Datum gilt die Rolle, bis sie entzogen wird.";
pub const ORG_ROLE_GRANT: &str = "Vergeben";
pub const ORG_ROLE_WITHDRAW: &str = "Entziehen";
pub const ORG_ROLE_INVALID: &str = "Bitte eine Benutzerkennung angeben, z. B. anna@ordo.example.";
pub const ORG_ROLE_PAST: &str = "Das Datum liegt in der Vergangenheit.";
pub const ORG_ROLE_FIXED: &str = "aus der Serverkonfiguration";
pub const ORG_ROLE_OWN_ADMIN: &str = "die eigene Administration kann nicht entzogen werden";
pub const ORG_ROLE_LAPSED: &str = "abgelaufen";

/// The label of an organisation role.
pub fn org_role(role: &str) -> &str {
    match role {
        "admin" => "Administration",
        "planner" => "Einsatzplanung (alle Einsätze)",
        other => other,
    }
}

/// Until when a role counts, e.g. "bis 30.11.2026".
pub fn org_role_until(date: &str) -> String {
    format!("bis {date}")
}

/// The accessible name of the withdraw button.
pub fn org_role_withdraw_label(role: &str, person: &str) -> String {
    format!("{} entziehen: {person}", org_role(role))
}

// Local accounts (ORG-10)
// Upstream login providers (ORG-11, ADR-0063).
pub const LOGIN_PROVIDERS: &str = "Anmeldung";
pub const PROVIDERS_TITLE: &str = "Anmeldung über die Organisation";
pub const PROVIDERS_INTRO: &str = "Verbinden Sie den Anmeldedienst Ihrer Organisation (OpenID Connect, z. B. Microsoft Entra ID, Keycloak, Authentik, Google). Er erscheint dann als eigene Schaltfläche auf der Anmeldeseite. Vor dem Speichern melden Sie sich einmal testweise darüber an.";
pub const PROVIDERS_LOADING: &str = "Anbieter werden geladen …";
pub const PROVIDERS_NOT_MANAGED: &str = "Dieser Server verwaltet keine Anmeldeanbieter (kein Zugang zur Schnittstelle des Anmeldedienstes). Anbieter stehen dann in dessen Konfiguration.";
pub const PROVIDER_LOCAL: &str = "Lokale Konten dieses Servers";
pub const PROVIDER_CHANGE: &str = "Ändern";
pub const PROVIDER_DELETE: &str = "Entfernen";
pub const PROVIDER_DELETE_CONFIRM: &str = "Ja, entfernen";
pub const PROVIDER_NEW: &str = "Anbieter hinzufügen";
pub const PROVIDER_NAME: &str = "Name auf der Anmeldeseite";
pub const PROVIDER_NAME_HINT: &str = "z. B. „DRK-Konto“";
pub const PROVIDER_ISSUER: &str = "Aussteller-URL (Issuer)";
pub const PROVIDER_ISSUER_HINT: &str = "z. B. https://login.microsoftonline.com/<Mandant>/v2.0";
pub const PROVIDER_CLIENT_ID: &str = "Client-ID";
pub const PROVIDER_SECRET: &str = "Client-Secret";
pub const PROVIDER_SECRET_HINT: &str =
    "Wird an den Anmeldedienst übergeben und in ORDO nie wieder angezeigt.";
pub const PROVIDER_SECRET_KEEP: &str = "Leer lassen, um das bisherige Secret zu behalten.";
pub const PROVIDER_SKIP_EMAIL: &str =
    "E-Mail-Adressen auch ohne Bestätigungsvermerk annehmen (z. B. bei Microsoft Entra ID)";
pub const PROVIDER_REDIRECT: &str = "Beim Anbieter als Weiterleitungs-URI eintragen:";
pub const PROVIDER_INVALID: &str =
    "Bitte Name, Aussteller-URL (https://…), Client-ID und Client-Secret angeben.";
pub const PROVIDER_TEST: &str = "Testanmeldung vorbereiten";
pub const PROVIDER_TEST_TITLE: &str = "Testanmeldung";
pub const PROVIDER_TEST_OPEN: &str = "Testanmeldung öffnen";
pub const PROVIDER_TEST_HINT: &str = "Danach hierher zurückkehren: Das Ergebnis erscheint hier.";
pub const PROVIDER_SAVE: &str = "Anbieter speichern";
pub const PROVIDER_SAVED: &str = "Anbieter gespeichert.";
pub const PROVIDER_REMOVED: &str = "Anbieter entfernt.";

pub fn provider_kind(kind: &str) -> String {
    format!("Aus der Konfiguration ({kind})")
}

pub fn provider_change_label(name: &str) -> String {
    format!("{PROVIDER_CHANGE}: {name}")
}

pub fn provider_delete_label(name: &str) -> String {
    format!("{PROVIDER_DELETE}: {name}")
}

pub fn provider_delete_question(name: &str) -> String {
    format!("„{name}“ entfernen? Neue Anmeldungen darüber sind dann nicht mehr möglich.")
}

pub fn provider_change_title(name: &str) -> String {
    format!("„{name}“ ändern")
}

pub fn provider_test_waiting(name: &str) -> String {
    format!(
        "Melden Sie sich in einem neuen Tab über „{name}“ an. Die Testanmeldung gilt 10 Minuten; so lange steht auf der Anmeldeseite zusätzlich „Test: {name}“."
    )
}

pub fn provider_test_passed(user: &str, name: Option<&str>, email: Option<&str>) -> String {
    let mut text = format!("Die Testanmeldung hat geklappt. ORDO würde sie als {user} erkennen");
    match (name, email) {
        (Some(n), Some(e)) => text.push_str(&format!(" (Name: {n}, E-Mail: {e})")),
        (Some(n), None) => text.push_str(&format!(" (Name: {n})")),
        (None, Some(e)) => text.push_str(&format!(" (E-Mail: {e})")),
        (None, None) => {}
    }
    text.push('.');
    text
}

pub fn provider_confirm(name: &str) -> String {
    format!("Ich bestätige: Anmeldungen über „{name}“ werden für diesen Server angenommen.")
}

pub fn provider_test_failed(error: &str) -> String {
    format!(
        "Die Testanmeldung ist fehlgeschlagen: {error}. Bitte die Angaben prüfen und erneut testen."
    )
}

pub fn provider_error(error: ordo_sync::message::LoginProviderError) -> &'static str {
    use ordo_sync::message::LoginProviderError as E;
    match error {
        E::Invalid => PROVIDER_INVALID,
        E::NotFound => {
            "Diesen Anbieter gibt es nicht mehr, oder er lässt sich nur in der Konfiguration ändern."
        }
        E::NotTested => "Bitte zuerst erfolgreich testen; die Testanmeldung gilt 10 Minuten.",
        E::Refused => {
            "Der Anmeldedienst hat die Änderung abgelehnt. Ist seine Schnittstelle für Anbieter freigeschaltet (DEX_API_CONNECTORS_CRUD)?"
        }
        E::Unreachable => ACCOUNT_UNREACHABLE,
    }
}

pub const ACCOUNTS: &str = "Konten";
pub const ACCOUNTS_TITLE: &str = "Anmeldekonten";
pub const ACCOUNTS_INTRO: &str = "Konten für die Anmeldung an diesem Server, z. B. für Helferinnen und Helfer ohne Konto der Organisation. Passwörter speichert nur der Anmeldedienst; ORDO kennt sie nicht.";
pub const ACCOUNTS_LOADING: &str = "Konten werden geladen …";
pub const ACCOUNTS_EMPTY: &str = "Noch keine Konten.";
pub const ACCOUNTS_OFFLINE: &str =
    "Konten lassen sich nur mit Verbindung zum Server sehen und ändern.";
pub const ACCOUNTS_NOT_MANAGED: &str = "Dieser Server verwaltet keine Anmeldekonten (kein Zugang zur Schnittstelle des Anmeldedienstes). Konten kommen dann aus der Konfiguration oder dem Konto-System der Organisation.";
pub const ACCOUNT_CREATE_TITLE: &str = "Konto anlegen";
pub const ACCOUNT_EMAIL: &str = "E-Mail-Adresse (Anmeldename)";
pub const ACCOUNT_NAME: &str = "Name";
pub const ACCOUNT_PASSWORD: &str = "Erstes Passwort";
pub const ACCOUNT_NEW_PASSWORD: &str = "Neues Passwort";
pub const ACCOUNT_PASSWORD_HINT: &str = "Mindestens 12 Zeichen. Der Person persönlich mitteilen.";
pub const ACCOUNT_CREATE: &str = "Konto anlegen";
pub const ACCOUNT_SET_PASSWORD: &str = "Passwort neu setzen";
pub const ACCOUNT_SAVE_PASSWORD: &str = "Passwort speichern";
pub const ACCOUNT_DELETE: &str = "Konto löschen";
pub const ACCOUNT_DELETE_CONFIRM: &str = "Ja, Konto löschen";
pub const ACCOUNT_DONE: &str = "Gespeichert.";
pub const ACCOUNT_INVALID: &str = "Bitte eine E-Mail-Adresse und einen Namen angeben.";
pub const ACCOUNT_PASSWORD_LENGTH: &str =
    "Das Passwort muss mindestens 12 Zeichen haben (höchstens 72 Bytes).";
pub const ACCOUNT_EXISTS: &str = "Ein Konto mit dieser E-Mail-Adresse gibt es schon.";
pub const ACCOUNT_NOT_FOUND: &str = "Dieses Konto gibt es nicht mehr.";
pub const ACCOUNT_REFUSED: &str = "Der Anmeldedienst hat die Änderung abgelehnt. Konten aus seiner Konfiguration lassen sich nur dort ändern.";
pub const ACCOUNT_UNREACHABLE: &str =
    "Der Anmeldedienst ist nicht erreichbar. Bitte später erneut versuchen.";

/// Why a change to an account was not made.
pub fn account_error(error: ordo_sync::message::AccountError) -> &'static str {
    use ordo_sync::message::AccountError;
    match error {
        AccountError::Invalid => ACCOUNT_INVALID,
        AccountError::PasswordLength => ACCOUNT_PASSWORD_LENGTH,
        AccountError::Exists => ACCOUNT_EXISTS,
        AccountError::NotFound => ACCOUNT_NOT_FOUND,
        AccountError::Refused => ACCOUNT_REFUSED,
        AccountError::Unreachable => ACCOUNT_UNREACHABLE,
    }
}

/// The question before deleting an account.
pub fn account_delete_question(name: &str, email: &str) -> String {
    format!(
        "Konto von {name} ({email}) löschen? Die Person kann sich nicht mehr anmelden, und ihre Geräte werden gesperrt."
    )
}

/// The accessible names of an account's buttons.
pub fn account_set_password_label(email: &str) -> String {
    format!("Passwort neu setzen: {email}")
}
pub fn account_delete_label(email: &str) -> String {
    format!("Konto löschen: {email}")
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
pub const ROLE_NONE: &str = "keine Rolle";
pub const GUEST_NEEDS_DUTY_END: &str =
    "Für den Zugang ohne Konto bitte das Dienstende angeben: Der Zugang endet mit ihm.";
pub const GUEST_NO_COMMAND: &str = "Die Einsatzleitung braucht ein ORDO-Konto.";
pub const GUEST_PAIR_TITLE: &str = "Zugang für Helfende";
pub const GUEST_ENDED_TITLE: &str = "Zugang beendet";
pub const GUEST_ENDED: &str = "Ihr Zugang zu diesem Einsatz ist mit Ihrem Dienst beendet. Für einen neuen Zugang bitte die Einsatzleitung fragen.";
pub const GUEST_PAIRING: &str = "Dieses Gerät wird mit dem QR-Code gekoppelt …";
pub const GUEST_CODE_INVALID: &str = "Der QR-Code gilt nicht mehr (schon benutzt, abgelaufen oder Dienst beendet). Bitte die Einsatzleitung um einen neuen.";
pub const GUEST_ACCESS: &str = "Zugang per QR-Code";
pub const GUEST_CODE_TITLE: &str = "QR-Code für den Zugang";
pub const GUEST_CODE_REFUSED: &str = "Der Zugang konnte nicht erstellt werden.";
pub const GUEST_CODE_ALT: &str = "QR-Code, der dieses Gerät des Helfers mit dem Einsatz koppelt";
pub const GUEST_LINK: &str = "Oder diesen Link auf dem Gerät öffnen:";
pub const GUEST_CODE_CLOSE: &str = "Fertig";
pub fn guest_code_for(name: &str, expires: &str) -> String {
    format!("Für {name}: mit der Kamera des Handys scannen. Gilt einmal, bis {expires} Uhr.")
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
    format!("{EDIT}: {name}")
}
pub fn helper_edit_title(name: &str) -> String {
    format!("{name} bearbeiten")
}
pub fn helper_saved(name: &str) -> String {
    format!("Angaben zu {name} gespeichert.")
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
