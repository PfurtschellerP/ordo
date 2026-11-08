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
pub const PAIR_NEW_INCIDENT: &str = "Neuen Einsatz anlegen";
pub const PAIR_SUBMIT: &str = "Koppeln";
pub const UNPAIR: &str = "Gerät entkoppeln";
pub const SIGNED_IN_AS: &str = "Angemeldet als";
pub const STARTING: &str = "ORDO wird gestartet …";

pub const ONLINE: &str = "Verbunden";
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
pub const TAB_ORGANISATION: &str = "Organisation";

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
pub const UNKNOWN: &str = "unbekannt";
pub const REVOKE: &str = "Sperren";
pub const REVOKE_CONFIRM: &str = "Ja, sperren";
pub const REVOKED_TITLE: &str = "Gerät gesperrt";
pub const REVOKED_TEXT: &str = "Dieses Gerät wurde gesperrt. Sein Schlüssel und alle Einsatzdaten wurden von diesem Gerät gelöscht. Um es wieder zu nutzen, das Gerät neu koppeln.";
pub const PAIR_AGAIN: &str = "Neu koppeln";

pub fn device_name(short: &str) -> String {
    format!("Gerät …{short}")
}

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
pub const DEVICE: &str = "Gerät";

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
pub const COL_STATUS: &str = "Status";
pub const COL_SET_STATUS: &str = "Status setzen";
pub const NO_STATUS: &str = "–";
pub fn set_status_label(unit: &str) -> String {
    format!("Status von {unit} setzen")
}
pub fn lost_update(unit: &str, overwritten: &str, kept: &str) -> String {
    format!(
        "Achtung {unit}: Status {} wurde gleichzeitig durch {} ersetzt. Bitte prüfen.",
        status_label(overwritten),
        status_label(kept)
    )
}

/// Status codes of the DE profile (FMS/TMS), prototype subset.
pub const STATUS_CODES: [(&str, &str); 7] = [
    ("1", "Einsatzbereit über Funk"),
    ("2", "Einsatzbereit auf Wache"),
    ("3", "Einsatz übernommen"),
    ("4", "Am Einsatzort"),
    ("6", "Nicht einsatzbereit"),
    ("7", "Patient aufgenommen"),
    ("8", "Am Transportziel"),
];

pub fn status_label(code: &str) -> String {
    STATUS_CODES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or_else(|| code.to_owned(), |(c, label)| format!("{c} – {label}"))
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
