//! User-visible texts of the administration (German, DE profile), apart from the
//! shell's and the shared modules' texts, which they re-export. Prototype: will move to
//! Fluent files per package (I18N-1).

pub use crate::shared::text::*;

pub const TAB_ADMIN: &str = "Administration";
pub const TAB_PERSON: &str = "Person";
pub const WS_ACCESS: &str = "Zugriff";
pub const WS_INFO: &str = "Info";

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

pub fn device_name(short: &str) -> String {
    format!("Gerät …{short}")
}

/// "Firefox auf Android · …4F2A9C": the device's name and the end of its ID (ORG-8).
pub fn named_device(name: &str, short: &str) -> String {
    format!("{name} · …{short}")
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

// Licences and source code (LIC-1)
pub const NOTICES_TITLE: &str = "Lizenzen und Quellcode";
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
