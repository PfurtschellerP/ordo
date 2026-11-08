//! User-visible texts (German, DE profile). Prototype: will move to Fluent translation
//! files (I18N-1); code and identifiers stay English.

use ordo_contracts::TriageCategory;

pub const APP_NAME: &str = "ORDO";
pub const SKIP_LINK: &str = "Zum Inhalt springen";

pub const PAIR_TITLE: &str = "Gerät koppeln";
pub const PAIR_INTRO: &str = "Prototyp: Die Angaben stammen aus der Seed-Datei des Servers und aus `ordo generate-device-key`. Später ersetzt ein QR-Code diesen Schritt.";
pub const PAIR_USER: &str = "Benutzer (name@server)";
pub const PAIR_DEVICE: &str = "Geräte-ID";
pub const PAIR_KEY: &str = "Privater Schlüssel des Geräts";
pub const PAIR_INCIDENT: &str = "Einsatz-ID";
pub const PAIR_NEW_INCIDENT: &str = "Neuen Einsatz anlegen";
pub const PAIR_SUBMIT: &str = "Koppeln";
pub const UNPAIR: &str = "Gerät entkoppeln";

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

pub const WORKSPACES: &str = "Arbeitsbereiche";
pub const TAB_MED: &str = "Sanität";
pub const TAB_COMMAND: &str = "Einsatzleitung";

pub const REGISTER_TITLE: &str = "Patient registrieren";
pub const REGISTER_TAG: &str = "Nummer der Patientenanhängekarte";
pub const REGISTER_CATEGORY: &str = "Sichtungskategorie";
pub const REGISTER_UHS: &str = "UHS";
pub const REGISTER_SUBMIT: &str = "Registrieren";
pub const NO_CLINICAL_ACCESS: &str =
    "Kein Zugriff auf Patientendaten in diesem Einsatz – oder das Gerät war noch nie verbunden.";

pub const BOARD_TITLE: &str = "UHS-Übersicht";
pub const BOARD_EMPTY: &str = "Noch keine Patienten registriert.";
pub const COL_TAG: &str = "PAK-Nr.";
pub const COL_CATEGORY: &str = "Kategorie";
pub const COL_UHS: &str = "UHS";
pub const COL_HISTORY: &str = "Sichtungen";
pub const COL_RETRIAGE: &str = "Nachsichtung";
pub const COL_STATE: &str = "Status";
pub const NOT_SENT: &str = "noch nicht gesendet";
pub const SENT: &str = "gesendet";

pub const COUNTS_TITLE: &str = "Patienten je UHS";
pub const COUNTS_EMPTY: &str = "Noch keine Patientenzahlen.";
pub const COL_TOTAL: &str = "Gesamt";

pub fn registered(tag: &str, category: TriageCategory) -> String {
    format!("Patient {tag} registriert: {}", label(category))
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
