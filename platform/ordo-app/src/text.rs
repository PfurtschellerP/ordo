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
pub const TAB_ORGANISATION: &str = "Organisation";
pub const TAB_INCIDENT: &str = "Einsatz";

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
pub const INCIDENTS_EMPTY: &str = "Noch kein Einsatz freigegeben. Die Einsatzleitung oder die Einsatzplanung gibt Personen eine Rolle im Einsatz.";
pub const OPEN: &str = "Öffnen";
pub const MY_ROLES: &str = "Rollen";
pub const NO_ROLE: &str = "keine (Einsatzplanung)";
pub const PLAN_TITLE: &str = "Einsatz planen";
pub const PLAN_NAME: &str = "Name des Einsatzes";
pub const PLAN_SUBMIT: &str = "Einsatz anlegen";
pub const CHANGE_INCIDENT: &str = "Einsatz wechseln";
pub const ROSTER: &str = "Führung";
pub const ROSTER_TITLE: &str = "Personal und Rollen";
pub const ROSTER_EMPTY: &str = "Noch niemand hat eine Rolle in diesem Einsatz.";
pub const ROSTER_PERSON: &str = "Person (name@server)";
pub const ROSTER_ROLE: &str = "Rolle";
pub const GRANT: &str = "Rolle vergeben";
pub const WITHDRAW: &str = "Entziehen";
pub const INCIDENT_STATE: &str = "Status";
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

pub fn open_orders(n: usize) -> String {
    format!("Offene Anordnungen: {n}")
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
pub fn strength_label(leaders: u32, sub_leaders: u32, crew: u32, total: u32) -> String {
    format!(
        "{STRENGTH}: {leaders} {LEADERS}, {sub_leaders} {SUB_LEADERS}, {crew} {CREW_COUNT}, {total} gesamt"
    )
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
pub const UHS_LOCATION: &str = "Ort";
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
pub const TASKS_OF_UNIT: &str = "Aufträge";

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
pub const MAP_UNAVAILABLE: &str = "Die Karte kann auf diesem Gerät nicht angezeigt werden; die Liste unten zeigt alles, was auf ihr steht.";
pub const MAP_FROM_DEVICE: &str = "Kartengrundlage: auf diesem Gerät gespeichert (auch ohne Netz).";
pub const MAP_FROM_SERVER: &str = "Kartengrundlage: vom Server.";
pub const MAP_NO_REGION: &str =
    "Keine Kartengrundlage eingerichtet: die Objekte stehen auf neutralem Hintergrund.";
pub const MAP_OBJECTS: &str = "Auf der Karte";
pub const MAP_OBJECTS_NONE: &str = "Noch nichts auf der Karte.";
pub const MAP_SHOW: &str = "Zeigen";
pub const MAP_REMOVE: &str = "Entfernen";
pub const MAP_SAVED: &str = "Lagekarte gespeichert";
pub const MAP_PLACE: &str = "Auf der Karte platzieren";
pub const MAP_PLACE_HINT: &str = "Karte so verschieben, dass die Stelle in der Mitte liegt (auch mit den Pfeiltasten), dann platzieren.";
pub const MAP_PLACE_KIND: &str = "Was";
pub const MAP_PLACE_WHICH: &str = "Welches";
pub const MAP_PLACE_CHOOSE: &str = "Bitte wählen";
pub const MAP_PLACE_LABEL: &str = "Bezeichnung";
pub const MAP_PLACE_SUBMIT: &str = "In der Kartenmitte platzieren";
pub const MAP_PLACE_MISSING: &str = "Bitte auswählen, was platziert werden soll.";
pub const INCIDENT_LOCATION: &str = "Einsatzort";
pub const MAP_OFFLINE: &str = "Karte ohne Netz";
pub const MAP_NOT_ON_DEVICE: &str = "Die Kartengrundlage ist nicht auf diesem Gerät gespeichert.";
pub const MAP_SAVE: &str = "Karte offline speichern";
pub const MAP_SAVING: &str = "Karte wird gespeichert …";
pub const MAP_SAVE_FAILED: &str = "Die Karte konnte nicht gespeichert werden.";
pub const MAP_REMOVE_OFFLINE: &str = "Offline-Karte löschen";
pub const MAP_REMOVED: &str = "Offline-Karte gelöscht.";

pub fn map_kind(kind: ordo_map::ObjectKind) -> &'static str {
    use ordo_map::ObjectKind as K;
    match kind {
        K::Incident => INCIDENT_LOCATION,
        K::Unit => TAB_RESOURCES,
        K::Uhs => "UHS",
        K::Position => POSITION,
        K::Drawing => "Zeichnung",
    }
}

pub fn map_show_label(label: &str) -> String {
    format!("{label} auf der Karte zeigen")
}

pub fn map_remove_label(label: &str) -> String {
    format!("{label} von der Karte entfernen")
}

fn megabytes(bytes: f64) -> String {
    format!("{:.1}", bytes / 1_000_000.0).replace('.', ",")
}

pub fn map_saved(bytes: f64) -> String {
    format!("Karte gespeichert ({} MB).", megabytes(bytes))
}

pub fn map_on_device(bytes: f64) -> String {
    format!(
        "Auf diesem Gerät gespeichert: {} MB, auch ohne Netz nutzbar.",
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
pub const MAP_REGION_HINT: &str = "Die Kartengrundlage, die alle Geräte für die Lagekarte nutzen: ein Umkreis aus der freien Weltkarte (OpenStreetMap über Protomaps). Geräte, die eine Offline-Karte gespeichert haben, speichern sie danach neu.";
pub const MAP_REGION_NONE: &str = "Noch keine Kartengrundlage auf dem Server.";
pub const MAP_REGION_NEW: &str = "Neue Kartengrundlage";
pub const MAP_REGION_LAT: &str = "Breitengrad (Mitte)";
pub const MAP_REGION_LON: &str = "Längengrad (Mitte)";
pub const MAP_REGION_RADIUS: &str = "Umkreis (km, 1 bis 50)";
pub const MAP_REGION_FROM_INCIDENT: &str = "Die Mitte ist der Einsatzort des geöffneten Einsatzes.";
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
pub const DRAW_TITLE: &str = "Auf der Karte zeichnen";
pub const DRAW_HINT: &str = "Karte so verschieben, dass das Fadenkreuz auf einem Eckpunkt liegt (auch mit den Pfeiltasten), dann den Eckpunkt setzen; so weiter bis zum letzten.";
pub const DRAW_CATEGORY: &str = "Art";
pub const DRAW_ADD_CORNER: &str = "Eckpunkt setzen";
pub const DRAW_UNDO: &str = "Letzten Eckpunkt entfernen";
pub const DRAW_CANCEL: &str = "Verwerfen";
pub const DRAW_FINISH: &str = "Zeichnung speichern";
pub const DRAW_TOO_FEW_AREA: &str = "Eine Fläche braucht mindestens drei Eckpunkte.";
pub const DRAW_TOO_FEW_LINE: &str = "Eine Linie braucht mindestens zwei Punkte.";

pub fn draw_corners(n: usize) -> String {
    match n {
        0 => "Noch kein Eckpunkt gesetzt.".to_owned(),
        1 => "1 Eckpunkt gesetzt.".to_owned(),
        n => format!("{n} Eckpunkte gesetzt."),
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
pub const ORG_ROLES: &str = "Rollen";
pub const ORG_ROLES_TITLE: &str = "Rollen in der Organisation";
pub const ORG_ROLES_INTRO: &str = "Wer die Organisation verwaltet oder Einsätze plant. Eine Rolle gilt auf allen Geräten der Person, sobald sie vergeben ist, und endet mit dem Entziehen oder am Ende des gewählten Tages. Rollen in einem Einsatz werden im Einsatz vergeben.";
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
        "planner" => "Einsatzplanung",
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
