//! User-visible texts of the Med package (German, DE profile), apart from the shell's
//! and the shared modules' texts, which they re-export: the package names `text::…`
//! for all of them. Prototype: will move to Fluent files per package (I18N-1).

pub use crate::shared::text::*;
use ordo_contracts::TriageCategory;

pub const CREW_PLACES: &str = "Freie Behandlungsplätze";
pub const CREW_NOT_HELD: &str = "Dieser Patient ist nicht (mehr) bei Ihrem Einsatzmittel. Seine Akte sieht, wer ihn jetzt versorgt.";
/// The UHS as a unit (UHS-12).
pub const UHS_STRENGTH: &str = "Stärke der UHS";
pub const UHS_STRENGTH_HINT: &str = "Die UHS zählt bei den Kräften vor Ort als Einsatzmittel: mit dieser Stärke, sonst mit ihrem Personal.";

pub const REGISTER_TITLE: &str = "Patient registrieren";
pub const REGISTER_TAG: &str = "Nummer der Patientenanhängekarte";
/// A card number already registered (PAT-12): where the patient is, and since when.
pub fn tag_in_use(tag: &str, holder: &str, time: &str) -> String {
    format!("Die Karte {tag} ist schon registriert: {holder}, seit {time}.")
}
/// A card number registered again (PAT-12): by whom, where and when.
pub fn tag_registered_again(person: &str, holder: &str, time: &str) -> String {
    format!("Noch einmal registriert {time} von {person} ({holder}).")
}
pub const TAG_TWICE: &str = "Karte zweimal registriert";
pub const TAG_TWICE_HINT: &str = "Zwei Geräte haben diese Kartennummer registriert, während sie offline waren, oder eine Nummer wurde falsch eingegeben. Wer später registriert hat, gibt seinem Patienten eine neue Karte und trägt deren Nummer ein; was dieses Gerät seitdem dokumentiert hat, geht mit.";
pub const RETAG_ASK: &str = "Ihre Registrierung kam später. Bitte geben Sie Ihrem Patienten eine neue Patientenanhängekarte.";
pub const RETAG_NUMBER: &str = "Nummer der neuen Karte";
pub const RETAG_SUBMIT: &str = "Nummer ändern";
pub fn retag_in_use(tag: &str) -> String {
    format!("Die Karte {tag} ist schon registriert. Bitte eine unbenutzte Karte nehmen.")
}
pub const REGISTER_CATEGORY: &str = "Sichtungskategorie";
pub const REGISTER_UHS: &str = "Patient bei";
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
pub const UHS_EMPTY: &str =
    "Noch keine UHS angelegt. Die Einsatzleitung legt sie unter Vorbereitung – Einsatzmittel an.";
pub const UHS_NAME: &str = "Name der UHS";
pub const UHS_CAPACITY: &str = "Kapazität (Patienten)";
pub const UHS_SUBMIT: &str = "UHS anlegen";
pub const UHS_NEW: &str = "Neue UHS";
pub const NO_UHS: &str = "Noch keine UHS, in der Sie dokumentieren dürfen. Die Einsatzleitung legt UHS an und teilt ihr Personal ein.";
pub const COL_LOAD: &str = "Belegung";
pub fn load(patients: u32, capacity: u32) -> String {
    format!("{patients} / {capacity}")
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
/// Under the photo: "Aufgenommen 14:31 von Anna Schmidt".
pub fn photo_taken(at: &str, by: &str) -> String {
    format!("Aufgenommen {at} von {by}")
}
pub fn photo_alt(tag: &str) -> String {
    format!("Foto von Patient {tag}")
}

pub const PHOTO_REMOVE_SHORT: &str = "Entfernen";

pub fn photo_remove_label(tag: &str) -> String {
    format!("Foto entfernen – Patient {tag}")
}
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
    format!("{form} {}", edit().to_lowercase())
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
pub const GROUP_PATIENTS: &str = "Patient";
pub const ACTION_REGISTER: &str = "Patient registrieren";
pub const ACTION_CREATE_UHS: &str = "UHS anlegen";
pub const ACTION_OPEN_RECORD: &str = "Akte öffnen";
pub const ACTION_TAKE_PHOTO: &str = "Foto aufnehmen";
pub fn action_entry(form: &str) -> String {
    format!("{form} eintragen")
}
pub fn action_retriage(category: TriageCategory) -> String {
    format!("Sichtung ändern: {}", label(category))
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
pub const PLACE: &str = "Behandlungsplatz";
pub const NO_PLACE: &str = "Kein Platz";

// The plan of a UHS's places (UHS-14)
pub const PLACE_FREE: &str = "frei";
pub const PLACE_FILL: &str = "Belegen";
pub const PLACE_NONE_TITLE: &str = "Ohne Platz";
pub const REGISTER_PLACE_HINT: &str =
    "Vorschlag: der erste freie Platz der gewählten Sichtungskategorie.";

/// "Platz 2 belegen": the button of a free place.
pub fn place_fill(label: &str) -> String {
    format!("{label} belegen")
}

/// "1 Patient", "3 Patienten".
pub fn patient_count(n: usize) -> String {
    if n == 1 {
        "1 Patient".to_owned()
    } else {
        format!("{n} Patienten")
    }
}

// Capacity per triage category (UHS-13)
pub const UHS_CAPACITIES_LEGEND: &str = "Kapazität nach Sichtungskategorie (optional)";
pub const UHS_CAPACITIES_HINT: &str = "Behandlungsplätze, die für eine Sichtungskategorie freigehalten werden; zusammen höchstens die Kapazität. Die übrigen Plätze sind ohne Kategorie.";

/// How a UHS's capacity is split, as typed.
pub fn uhs_places_split(kept: u32, total: u32) -> String {
    if kept > total {
        format!("Zurzeit {kept} Plätze nach Kategorie: mehr als die Kapazität von {total}.")
    } else {
        format!(
            "Zurzeit {kept} nach Kategorie, {} ohne Kategorie.",
            total - kept
        )
    }
}

/// The places per category do not fit in the capacity.
pub fn uhs_places_exceed(kept: u32, total: u32) -> String {
    format!(
        "Die Plätze nach Kategorie ({kept}) sind mehr als die Kapazität ({total}). Bitte die Kapazität erhöhen oder weniger Plätze freihalten."
    )
}

pub fn place_taken(label: &str, by: &str) -> String {
    format!("{label} (belegt: {by})")
}

pub fn places_free(free: usize, total: usize) -> String {
    format!("{free} von {total} frei")
}

// Free treatment places per category (UHS-8, UHS-16, ADR-0086)
pub const PLACES_ALL: &str = "Alle UHS";
pub const PLACES_ANY: &str = "ohne Kategorie";
pub const PLACES_FULL: &str = "voll";
pub const PLACES_NONE: &str = "Keine Behandlungsplätze angelegt.";
pub const PLACES_LEGEND: &str = "Ein Kästchen je Platz: gefüllt belegt, gestrichelt frei. Volle Kategorien sind schraffiert und mit „voll“ benannt.";

/// "frei von 15", after the large number of free places.
pub fn free_of(total: usize) -> String {
    format!("frei von {total}")
}

/// "2 ohne Platz": patients waiting for a place.
pub fn without_place(n: u32) -> String {
    format!("{n} ohne Platz")
}

/// The text of a category's strip of places, e.g. "SK II: 4 von 4 belegt, voll".
pub fn place_strip(category: &str, taken: usize, total: usize) -> String {
    let free = total.saturating_sub(taken);
    if free == 0 {
        format!("{category}: {taken} von {total} belegt, voll")
    } else {
        format!("{category}: {taken} von {total} belegt, {free} frei")
    }
}
pub const TRANSPORTS_TITLE: &str = "Offene Transporte";
pub const TRANSPORT_TITLE: &str = "Transport";
pub const TRANSPORT_TYPE: &str = "Transportmittel";
pub const TRANSPORT_PRIORITY_OF_REQUEST: &str = "Dringlichkeit";
pub const TRANSPORT_NEEDS: &str = "Besonderheiten";
pub const TRANSPORT_REQUEST: &str = "Transport anfordern";
pub const TRANSPORT_CHOOSE: &str = "Bitte Transportmittel und Dringlichkeit wählen.";
pub const TRANSPORT_NONE: &str = "Kein Transport angefordert.";
pub const TRANSPORT_UNIT: &str = "Einsatzmittel";
pub const TRANSPORT_DESTINATION: &str = "Ziel (z. B. Klinik)";
pub const TRANSPORT_ASSIGN: &str = "Zuweisen";
pub const TRANSPORT_ASSIGN_MISSING: &str = "Bitte Einsatzmittel und Ziel angeben.";
pub const TRANSPORT_NO_UNITS: &str = "Noch keine Einsatzmittel im Einsatz.";
pub const TRANSPORT_CHOOSE_UNIT: &str = "Einsatzmittel wählen";

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
pub const STAY_OPEN: &str = "Entlassen …";
/// The stay section on the tab *Übergabe*: admissions and discharges; moving is a handover.
pub const DISCHARGE_TITLE: &str = "Aufnahme und Entlassung";
pub const TRIAGE_OPEN: &str = "Sichtung ändern";
pub const ORDERS_OPEN: &str = "Neue Anordnung";
pub const DISCHARGE_OUTCOME: &str = "Entlassen mit Ergebnis";
pub const DISCHARGE_CHOOSE: &str = "Bitte das Ergebnis der Entlassung wählen.";
pub const READMIT: &str = "Wieder aufnehmen";

/// "Entlassene Patienten (2)"
pub fn discharged_list(n: usize) -> String {
    format!("Entlassene Patienten ({n})")
}

/// "Mit dem Transport: RTW → Klinikum Mitte"
pub fn discharge_transport(line: &str) -> String {
    format!("Mit dem Transport: {line}")
}

pub fn readmit_label(tag: &str) -> String {
    format!("{tag} {}", READMIT.to_lowercase())
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
    format!("Bearbeiten: {name}")
}
pub fn hospital_edit_title(name: &str) -> String {
    format!("{name} bearbeiten")
}

pub fn capacity_save_label(name: &str) -> String {
    format!("{CAPACITY_SAVE}: {name}")
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

// Patient pathway (PAT-5).

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

// UHS details (UHS-1).
pub const UHS_DETAILS: &str = "Ort und Personal";
pub const UHS_DATA: &str = "Angaben";
pub const SAVED: &str = "Gespeichert.";
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

// The record page (EPCR-14, EPCR-15).
pub const RECORD_TABS: &str = "Bereiche der Akte";
pub const RECORD_OVERVIEW: &str = "Übersicht";
pub const RECORD_TIMELINE: &str = "Verlauf";
pub const RECORD_DOCUMENTATION: &str = "Dokumentation";
pub const RECORD_DOCUMENTATION_SHORT: &str = "Doku";
pub const RECORD_HANDOVER: &str = "Übergabe";
pub const RECORD_AT: &str = "Bei";
pub const RECORD_TRIAGED: &str = "Gesichtet";
pub const RECORD_SINCE: &str = "Seit";
pub const QUICK_ACTIONS: &str = "Schnell erfassen";
pub const ALL: &str = "Alle";
pub const ALL_KINDS: &str = "Alles";
pub const LATEST: &str = "zuletzt";
pub const ORDER: &str = "Anordnung";
pub const ORDERS_OPEN_TITLE: &str = "Offene Anordnungen";
pub const ORDERS_ALL: &str = "Alle Anordnungen in der Dokumentation";
pub const ORDERS_NONE_OPEN: &str = "Keine offenen Anordnungen.";
// Handover (ADR-0087).
pub const HOLDERS_UHS: &str = "UHS";
pub const HOLDERS_UNITS: &str = "Einsatzmittel";
pub fn handover_to(to: &str) -> String {
    format!("Übergabe an {to}")
}
pub const HANDOVER_QUICK_STEP: &str = "Schnellübergabe per QR-Code";
pub fn handover_taken_by(by: &str) -> String {
    format!("Übernommen: {by}")
}
pub fn handover_taken_back(by: &str) -> String {
    format!("Übergabe zurückgenommen: {by}")
}
pub fn handed_outside_step(receiver: &str) -> String {
    format!("Übergeben an {receiver} (außerhalb von ORDO)")
}
pub fn handover_open_to(to: &str) -> String {
    format!("Übergabe an {to} offen")
}
pub const HANDOVER_OPEN_QUICK: &str = "Schnellübergabe offen";
/// "Noch bei SanTrp 1 · übergeben 14:31 von Max Muster".
pub fn handover_open_since(holder: &str, at: &str, by: &str) -> String {
    format!("Noch bei {holder} · übergeben {at} von {by}")
}
pub const HANDOVER_OPEN_HINT: &str = "Bis zur Übernahme dokumentiert die bisherige Stelle weiter; die übernehmende Stelle bestätigt mit „Übernehmen“.";
pub const HANDOVER_OPEN_QUICK_HINT: &str = "Die übernehmende Stelle scannt den QR-Code mit ORDO oder der Kamera ihres Geräts und wählt, wohin der Patient kommt.";
pub const HANDOVER_NOTE_SHORT: &str = "Notiz:";
pub fn handover_take(to: &str) -> String {
    format!("Übernehmen für {to}")
}
pub const HANDOVER_TAKE: &str = "Übernehmen";
/// The take button's accessible name: its words and the patient.
pub fn handover_take_label(words: &str, tag: &str) -> String {
    format!("{words}: Patient {tag}")
}
pub const HANDOVER_TAKE_TO: &str = "Übernehmen für";
pub const TAKE_PLACE_HINT: &str =
    "Vorschlag: der erste freie Platz der Sichtungskategorie des Patienten.";
pub const HANDOVER_TAKE_BACK: &str = "Übergabe zurücknehmen";
pub const HANDOVER_FAILED: &str = "Die Übergabe konnte nicht gespeichert werden.";
pub const HANDOVER_WAY: &str = "Weg des Patienten";
// The handover document (ADR-0089).
pub const DOCUMENT_TITLE: &str = "Übergabedokument";
pub const DOCUMENT_HINT: &str = "PDF für die übernehmende Stelle. Alle Daten liegen ihm zusätzlich maschinenlesbar bei, als HL7-FHIR-Bundle (Anhang der PDF-Datei, in PDF-Programmen unter „Anhänge“). Der ORDO-Server erstellt es; dafür braucht das Gerät eine Verbindung.";
pub const DOCUMENT_BUTTON: &str = "PDF herunterladen";
pub fn document_label(tag: &str) -> String {
    format!("PDF herunterladen: Übergabedokument Patient {tag}")
}
/// The file name of the saved document.
pub fn document_file(tag: &str) -> String {
    format!("Uebergabe-{tag}.pdf")
}
pub const DOCUMENT_ASKING: &str = "Wird auf dem Server erstellt …";
pub const DOCUMENT_SAVED: &str = "Heruntergeladen.";
pub const DOCUMENT_OFFLINE: &str = "Keine Verbindung zum Server. Das Dokument wird dort erstellt; bitte mit Verbindung erneut versuchen.";
pub const DOCUMENT_REFUSED: &str =
    "Sie dürfen das Übergabedokument dieses Patienten nicht abrufen.";
pub const DOCUMENT_FAILED: &str =
    "Das Dokument konnte nicht erstellt werden. Bitte erneut versuchen.";
// The way as a flow chart (dsn patient-flow).
pub fn flow_registered(time: &str) -> String {
    format!("Erfasst {time}")
}
pub fn flow_taken(time: &str) -> String {
    format!("Übernommen {time}")
}
pub fn flow_taken_back(time: &str) -> String {
    format!("Übergabe zurückgenommen {time}")
}
/// "Transport: RTW 2 → Klinikum Mitte · Angefordert"
pub fn flow_transport(line: &str, status: &str) -> String {
    format!("Transport: {line} · {status}")
}
pub const FLOW_HANDOVER: &str = "Übergabe";
pub const FLOW_HANDOVER_OPEN: &str = "Übergabe offen";
pub const FLOW_MOVED: &str = "Verlegt";
pub const FLOW_OUTSIDE: &str = "Außerhalb von ORDO";
pub const FLOW_QUICK_RECEIVER: &str = "Wer den QR-Code scannt";
pub const FLOW_NOT_TAKEN: &str = "Noch nicht übernommen";
pub const FLOW_NEXT_EDGE: &str = "Nächste Übergabe";
pub const HANDOVER_NEXT: &str = "an wen übergeben";
pub const HANDOVER_TITLE: &str = "Patient übergeben";
pub const HANDOVER_TO: &str = "Übergeben an";
pub const HANDOVER_TO_UHS: &str = "UHS oder Behandlungsplatz";
pub const HANDOVER_TO_UHS_HINT: &str =
    "Im Einsatz. Die Akte geht mit; das Personal der UHS dokumentiert weiter.";
pub const HANDOVER_TO_UNIT: &str = "Einsatzmittel";
pub const HANDOVER_TO_UNIT_HINT: &str =
    "Im Einsatz, z. B. ein RTW. Seine Besatzung dokumentiert weiter.";
pub const HANDOVER_TO_TRANSPORT: &str = "Transport";
pub const HANDOVER_TO_TRANSPORT_HINT: &str =
    "Das Einsatzmittel eines angeforderten Transports übernimmt; das Ziel folgt aus dem Transport.";
pub const HANDOVER_TO_OUTSIDE: &str = "Außerhalb von ORDO";
pub const HANDOVER_TO_OUTSIDE_HINT: &str =
    "Z. B. Klinik, anderer Rettungsdienst, Polizei. Der Patient verlässt den Einsatz.";
pub fn handover_target_label(kind: &str) -> String {
    format!("Welche Stelle: {kind}")
}
pub fn handover_no_target(transport: bool) -> &'static str {
    if transport {
        "Kein Transport mit zugewiesenem Einsatzmittel. Transport im Abschnitt „Transport“ anfordern."
    } else {
        "Keine andere Stelle im Einsatz."
    }
}
pub const HANDOVER_RECEIVER: &str = "An wen";
pub const HANDOVER_RECEIVER_HINT: &str =
    "Z. B. „Klinikum Mitte, Notaufnahme“ oder „RTW Florian Kreis 3/83-1“.";
pub const HANDOVER_RECEIVER_MISSING: &str = "Bitte angeben, an wen der Patient übergeben wird.";
pub const HANDOVER_TARGET_MISSING: &str = "Bitte wählen, an wen der Patient übergeben wird.";
pub const HANDOVER_NOTE: &str = "Übergabe-Notiz (optional)";
pub const HANDOVER_NOTE_EXAMPLE: &str = "z. B. Verlauf auf dem Weg";
pub const HANDOVER_RECEIVER_ANY: &str = "die übernehmende Stelle";
pub fn handover_gets(to: &str) -> String {
    format!("Was {to} bekommt")
}
pub const HANDOVER_GETS_RECORD: &str = "die ganze Akte mit Verlauf, Fotos und Anordnungen";
pub const HANDOVER_GETS_NOTE: &str = "wer übergeben hat, wann, und die Übergabe-Notiz";
pub const HANDOVER_GETS_OUTSIDE: &str =
    "die Übergabe mit Empfänger, Zeit und Notiz im Verlauf; ein Übergabedokument (PDF) folgt";
pub const HANDOVER_AFTER: &str = "Danach";
pub const HANDOVER_AFTER_INSIDE: &str = "Bis zur Übernahme dokumentieren Sie weiter, und beide Seiten sehen „Übergabe offen“. Nach der Übernahme lesen Sie die Akte weiter, schreiben aber nicht mehr.";
pub const HANDOVER_AFTER_OUTSIDE: &str = "Der Patient verlässt den Einsatz sofort und zählt als entlassen. Kommt er zurück, „Wieder aufnehmen“.";
pub const HANDOVER_BUTTON: &str = "Übergeben";
pub fn handover_button(to: &str) -> String {
    format!("An {to} übergeben")
}
pub const QUICK_HANDOVER: &str = "Schnellübergabe";
pub const QUICK_HANDOVER_HINT: &str = "Wenn noch offen ist, wer übernimmt: ORDO zeigt einen QR-Code. Wer ihn scannt, wählt, wohin der Patient kommt. Der Code enthält keine medizinischen Daten.";
pub const QUICK_HANDOVER_START: &str = "QR-Code zeigen";
pub const QUICK_CODE_ALT: &str = "QR-Code der Schnellübergabe";
pub const QUICK_CODE_HINT: &str =
    "Mit ORDO („Übergabe scannen“) oder der Kamera des übernehmenden Geräts scannen.";
pub const SCAN_HANDOVER: &str = "Übergabe scannen";
pub const SCAN_HANDOVER_HINT: &str = "Foto vom QR-Code einer Schnellübergabe.";
pub const SCAN_READING: &str = "Der QR-Code wird gelesen …";
pub const SCAN_NO_HANDOVER: &str = "Auf dem Foto ist kein QR-Code einer Übergabe zu erkennen.";
pub fn take_title(tag: &str) -> String {
    format!("Patient {tag} übernehmen")
}
pub fn take_from(holder: &str) -> String {
    format!("Bisher bei {holder}. Wählen Sie, wo der Patient bei Ihnen ist.")
}
pub const TAKE_NOT_HERE: &str = "Die Akte ist noch nicht auf diesem Gerät. Sobald es verbunden ist, kommt sie an; dieses Fenster bleibt offen.";
pub const TAKE_NOT_ARRIVED: &str = "Die Übergabe selbst ist noch nicht auf diesem Gerät. Die Übernahme gilt, sobald beide Geräte wieder verbunden sind.";
pub const TAKE_GONE: &str =
    "Der Patient ist nicht mehr im Einsatz (entlassen oder außerhalb übergeben).";
pub const TAKE_TAKEN: &str = "Diese Übergabe ist schon übernommen.";
pub const TAKE_NOWHERE: &str = "Sie stehen auf keiner Personalliste einer UHS und in keiner Besatzung. Die Einsatzleitung teilt Sie ein.";
pub const INCOMING_TITLE: &str = "Übergaben an Sie";
pub fn incoming_line(from: &str, to: &str) -> String {
    format!("von {from} an {to}")
}
pub const RECORD_OPEN: &str = "Akte";
pub fn handover_bell(tag: &str, to: &str) -> String {
    format!("Übergabe an {to}: Patient {tag}")
}

pub const READMITTED_STEP: &str = "Wieder aufgenommen";
pub const CLEARED: &str = "gelöscht";
pub const PHOTO_ADDED: &str = "Foto hinzugefügt";
pub const TIMELINE_FILTER: &str = "Verlauf filtern";

pub fn way_registered(uhs: &str) -> String {
    format!("Erfasst · {uhs}")
}

pub fn registered_as(category: &str) -> String {
    format!("Erfasst: {category}")
}

pub fn entries_all(n: usize) -> String {
    format!("Alle {n}")
}

pub fn entries_all_label(form: &str, n: usize) -> String {
    format!("{form}: alle {n} Einträge im Verlauf")
}

pub fn corrects_entry(time: &str) -> String {
    format!("korrigiert den Eintrag von {time}")
}

pub fn uhs_edit_title(name: &str) -> String {
    format!("{name} bearbeiten")
}

pub const UHS_CLOSE_TITLE: &str = "UHS auflösen";
pub const UHS_CLOSE: &str = "UHS auflösen …";
pub const UHS_CLOSE_CONFIRM: &str = "Ja, auflösen";
pub const UHS_CLOSE_HINT: &str =
    "Die UHS verschwindet aus Listen, Karte und Zahlen. Das Einsatztagebuch behält sie.";

pub fn uhs_close_blocked(patients: u32) -> String {
    let who = if patients == 1 {
        "1 Patient ist".to_owned()
    } else {
        format!("{patients} Patienten sind")
    };
    format!("{who} noch hier. Erst übergeben oder entlassen, dann auflösen.")
}

pub fn uhs_close_question(name: &str) -> String {
    format!("{name} wirklich auflösen?")
}

pub fn uhs_details_label(name: &str) -> String {
    format!("{UHS_DETAILS} von {name}")
}

pub fn staff_remove_label(name: &str) -> String {
    format!("{name} aus dem Personal entfernen")
}

pub fn uhs_renamed(name: &str) -> String {
    format!("UHS umbenannt in {name}")
}

// A UHS asks for staff (UHS-15).
pub const STAFF_REQUESTS_TITLE: &str = "Personal anfordern";
pub const STAFF_REQUEST_CHOOSE_UHS: &str =
    "Wählen Sie oben unter „Meine UHS“ Ihre UHS, um für sie anzufordern.";

pub fn staff_request_for(uhs: &str) -> String {
    format!("Für {uhs}: die Einsatzleitung sieht die Anforderung und schickt jemanden.")
}
pub const STAFF_REQUEST_OPEN: &str = "offen";
pub const STAFF_REQUEST_WITHDRAWN: &str = "zurückgenommen";
pub const STAFF_REQUEST_CLOSED: &str = "Anforderung abgeschlossen";
pub const STAFF_BOARD_TITLE: &str = "Anforderungen der UHS";
pub const STAFF_SEND: &str = "Wer kommt?";
pub const STAFF_ASSIGN: &str = "Zuweisen";
pub const STAFF_WITHDRAW: &str = "Zurücknehmen";
pub fn staff_need(need: ordo_med_patients::staffing::Need) -> &'static str {
    match need {
        ordo_med_patients::staffing::Need::Physician => "Arzt/Ärztin",
        ordo_med_patients::staffing::Need::Paramedic => "Notfallsanitäter/in",
    }
}
pub fn staff_request_button(need: ordo_med_patients::staffing::Need) -> String {
    format!("{} anfordern", staff_need(need))
}
pub fn staff_requested(uhs: &str, need: ordo_med_patients::staffing::Need) -> String {
    format!("{uhs} fordert {} an", staff_need(need))
}
pub fn staff_request_label(uhs: &str, need: ordo_med_patients::staffing::Need) -> String {
    staff_requested(uhs, need)
}
pub fn staff_request_met(person: &str) -> String {
    format!("{person} kommt")
}
pub fn staff_withdraw_label(request: &str) -> String {
    format!("{STAFF_WITHDRAW}: {request}")
}
pub const CASE_CANCEL: &str = "Ohne Auftrag aufnehmen";

pub fn register_from_case(place: &str, unit: &str) -> String {
    format!("Aus dem Auftrag „{place}“ für {unit}")
}

/// The Auftrag of the person sent after a staff request (UHS-15).
pub fn staff_task(uhs: &str, need: ordo_med_patients::staffing::Need) -> String {
    format!("Als {} zu {uhs}", staff_need(need))
}

pub fn staff_request_not_staff(uhs: &str) -> String {
    format!("Anfordern kann nur, wer auf der Personalliste von {uhs} steht.")
}

pub fn log_counts(uhs: &str, counts: &[(&str, u32)], unplaced: u32, discharged: u32) -> String {
    let total: u32 = counts.iter().map(|(_, n)| n).sum();
    let mut out = format!("Patientenzahlen {uhs}: {total}");
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
        out.push_str(&format!(", {discharged} entlassen"));
    }
    out
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

/// Above a record the person may only read (UHS-10).
pub fn record_read_only(uhs: &str) -> String {
    format!(
        "Nur lesen: Sie stehen nicht auf der Personalliste von {uhs}. Die Einsatzleitung teilt Personal zu."
    )
}
