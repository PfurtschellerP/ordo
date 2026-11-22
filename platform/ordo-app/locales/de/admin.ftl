tab-admin = Administration
tab-person = Person
ws-access = Zugriff
ws-info = Info
devices = Geräte
devices-intro-own = Die eigenen gekoppelten Geräte. Ein verlorenes oder gestohlenes Gerät hier sperren: Es wird sofort getrennt und löscht seine Daten, sobald es sich wieder verbindet.
devices-intro-all = Alle gekoppelten Geräte der Organisation. Ein verlorenes oder gestohlenes Gerät hier sperren: Es wird sofort getrennt und löscht seine Daten, sobald es sich wieder verbindet.
devices-offline = Die Geräteliste ist nur mit Verbindung zum Server verfügbar.
devices-loading = Geräteliste wird geladen …
devices-empty = Keine gekoppelten Geräte.
this-device = dieses Gerät
device-person = Person
device-paired = Gekoppelt
device-valid-until = Anmeldung gültig bis
device-connection = Verbindung
device-connected = verbunden
device-not-connected = nicht verbunden
# Devices used in the incident, on *Vorbereitung – Personal* (ORG-12).
incident-devices = Geräte im Einsatz
incident-devices-intro = Die Geräte aller Personen mit einer Rolle in diesem Einsatz. Ein verlorenes oder gestohlenes Gerät hier sperren: Es wird sofort getrennt und löscht alle ORDO-Daten, sobald es sich wieder verbindet, auch die anderer Einsätze.
incident-devices-empty = Noch keine Geräte: Niemand mit einer Rolle in diesem Einsatz hat ein gekoppeltes Gerät.
unknown = unbekannt
revoke = Sperren
revoke-confirm = Ja, sperren
device-name = Gerät …{ $short }
# "Firefox auf Android · …4F2A9C": the device's name and the end of its ID (ORG-8).
named-device = { $name } · …{ $short }
revoke-label = Sperren: Gerät …{ $short }

## Licences and source code (LIC-1)
notices-title = Lizenzen und Quellcode
notices-ordo-licence = ORDO ist freie Software unter der GNU Affero General Public License, Version 3 oder einer späteren Version (AGPL-3.0-or-later). Sie darf genutzt, weitergegeben und verändert werden; wer eine veränderte Fassung im Netz anbietet, muss deren Quellcode ebenso anbieten.
notices-licence-text = Lizenztext
notices-source = Quellcode dieser Fassung:
notices-parts = Weitere Bestandteile
notices-crates = Rust-Bibliotheken
notices-crates-hint = Die Lizenztexte, wie sie den Bibliotheken von Server und App beiliegen, je Text mit den Bibliotheken, für die er gilt.
notices-crates-missing = Die Lizenzhinweise der Rust-Bibliotheken sind in diesem Build nicht enthalten. Welche Bibliotheken in welcher Version genutzt werden, steht im Quellcode (Cargo.lock); offizielle Builds zeigen hier ihre Lizenzhinweise.
info-version = Version
info-app = App
info-server = Server
info-build-local = lokaler Build
info-server-offline = Der Server ist gerade nicht erreichbar.
info-loading = Wird abgefragt …
# "Protokolle: Abgleich 1.0–1.2, HTTP 1.0"
info-protocols = Protokolle: Abgleich { $sync }, HTTP { $http }
notices-loading = Lizenzhinweise werden geladen …
notices-used-map = Lagekarte
notices-used-map-font = Schrift der Kartenbeschriftung
notices-used-font = Schrift der Bedienoberfläche
notices-used-icons = Symbole der Bedienoberfläche
notices-used-symbols = Taktische Zeichen (gezeichnet beim Erstellen von ORDO)
notices-used-storage = Datenspeicher auf dem Gerät
notices-used-map-data = Kartendaten, wo eine Kartengrundlage eingerichtet ist
notices-public-domain = gemeinfrei (Public Domain)

## Organisation roles (ORG-9)
org-roles = Berechtigungen
org-roles-title = Berechtigungen in der Organisation
org-roles-intro = Wer die Organisation verwaltet oder Einsätze plant. Eine Berechtigung gilt auf allen Geräten der Person, sobald sie vergeben ist, und endet mit dem Entziehen oder am Ende des gewählten Tages. Rollen in einem Einsatz vergibt die Einsatzleitung unter „Vorbereitung – Personal und Rollen“.
org-roles-loading = Rollen werden geladen …
org-roles-empty = Noch keine Rollen vergeben.
org-roles-offline = Rollen lassen sich nur mit Verbindung zum Server sehen und ändern.
org-role-grant-title = Rolle vergeben
org-role-person = Person (Benutzerkennung)
org-role-role = Rolle
org-role-until-label = Gültig bis (optional)
org-role-until-hint = Ohne Datum gilt die Rolle, bis sie entzogen wird.
org-role-grant = Vergeben
org-role-withdraw = Entziehen
org-role-invalid = Bitte eine Benutzerkennung angeben, z. B. anna@ordo.example.
org-role-past = Das Datum liegt in der Vergangenheit.
org-role-fixed = aus der Serverkonfiguration
org-role-own-admin = die eigene Administration kann nicht entzogen werden
org-role-lapsed = abgelaufen
# Until when a role counts, e.g. "bis 30.11.2026".
org-role-until = bis { $date }

## Local accounts (ORG-10)

## Upstream login providers (ORG-11, ADR-0063).
login-providers = Anmeldung
providers-title = Anmeldung über die Organisation
providers-intro = Verbinden Sie den Anmeldedienst Ihrer Organisation (OpenID Connect, z. B. Microsoft Entra ID, Keycloak, Authentik, Google). Er erscheint dann als eigene Schaltfläche auf der Anmeldeseite. Vor dem Speichern melden Sie sich einmal testweise darüber an.
providers-loading = Anbieter werden geladen …
providers-not-managed = Dieser Server verwaltet keine Anmeldeanbieter (kein Zugang zur Schnittstelle des Anmeldedienstes). Anbieter stehen dann in dessen Konfiguration.
provider-local = Lokale Konten dieses Servers
provider-change = Ändern
provider-delete = Entfernen
provider-delete-confirm = Ja, entfernen
provider-new = Anbieter hinzufügen
provider-name = Name auf der Anmeldeseite
provider-name-hint = z. B. „DRK-Konto“
provider-issuer = Aussteller-URL (Issuer)
provider-issuer-hint = z. B. https://login.microsoftonline.com/<Mandant>/v2.0
provider-client-id = Client-ID
provider-secret = Client-Secret
provider-secret-hint = Wird an den Anmeldedienst übergeben und in ORDO nie wieder angezeigt.
provider-secret-keep = Leer lassen, um das bisherige Secret zu behalten.
provider-skip-email = E-Mail-Adressen auch ohne Bestätigungsvermerk annehmen (z. B. bei Microsoft Entra ID)
provider-redirect = Beim Anbieter als Weiterleitungs-URI eintragen:
provider-invalid = Bitte Name, Aussteller-URL (https://…), Client-ID und Client-Secret angeben.
provider-test = Testanmeldung vorbereiten
provider-test-title = Testanmeldung
provider-test-open = Testanmeldung öffnen
provider-test-hint = Danach hierher zurückkehren: Das Ergebnis erscheint hier.
provider-save = Anbieter speichern
provider-saved = Anbieter gespeichert.
provider-removed = Anbieter entfernt.
provider-kind = Aus der Konfiguration ({ $kind })
provider-change-label = { provider-change }: { $name }
provider-delete-label = { provider-delete }: { $name }
provider-delete-question = „{ $name }“ entfernen? Neue Anmeldungen darüber sind dann nicht mehr möglich.
provider-change-title = „{ $name }“ ändern
provider-test-waiting = Melden Sie sich in einem neuen Tab über „{ $name }“ an. Die Testanmeldung gilt 10 Minuten; so lange steht auf der Anmeldeseite zusätzlich „Test: { $name }“.
provider-confirm = Ich bestätige: Anmeldungen über „{ $name }“ werden für diesen Server angenommen.
provider-test-failed = Die Testanmeldung ist fehlgeschlagen: { $error }. Bitte die Angaben prüfen und erneut testen.
accounts = Konten
accounts-title = Anmeldekonten
accounts-intro = Konten für die Anmeldung an diesem Server, z. B. für Helferinnen und Helfer ohne Konto der Organisation. Passwörter speichert nur der Anmeldedienst; ORDO kennt sie nicht.
accounts-loading = Konten werden geladen …
accounts-empty = Noch keine Konten.
accounts-offline = Konten lassen sich nur mit Verbindung zum Server sehen und ändern.
accounts-not-managed = Dieser Server verwaltet keine Anmeldekonten (kein Zugang zur Schnittstelle des Anmeldedienstes). Konten kommen dann aus der Konfiguration oder dem Konto-System der Organisation.
account-create-title = Konto anlegen
account-email = E-Mail-Adresse (Anmeldename)
account-name = Name
account-password = Erstes Passwort
account-new-password = Neues Passwort
account-password-hint = Mindestens 12 Zeichen. Der Person persönlich mitteilen.
account-create = Konto anlegen
account-set-password = Passwort neu setzen
account-save-password = Passwort speichern
account-delete = Konto löschen
account-delete-confirm = Ja, Konto löschen
account-done = Gespeichert.
account-invalid = Bitte eine E-Mail-Adresse und einen Namen angeben.
account-password-length = Das Passwort muss mindestens 12 Zeichen haben (höchstens 72 Bytes).
account-exists = Ein Konto mit dieser E-Mail-Adresse gibt es schon.
account-not-found = Dieses Konto gibt es nicht mehr.
account-refused = Der Anmeldedienst hat die Änderung abgelehnt. Konten aus seiner Konfiguration lassen sich nur dort ändern.
account-unreachable = Der Anmeldedienst ist nicht erreichbar. Bitte später erneut versuchen.
# The question before deleting an account.
account-delete-question = Konto von { $name } ({ $email }) löschen? Die Person kann sich nicht mehr anmelden, und ihre Geräte werden gesperrt.
# The accessible names of an account's buttons.
account-set-password-label = Passwort neu setzen: { $email }
account-delete-label = Konto löschen: { $email }

## Converted by hand: forms and values of ORDO's types
revoke-own-question = Dieses Gerät (…{ $short }) sperren? Es wird sofort getrennt, und alle Daten darauf werden gelöscht, auch noch nicht gesendete Eingaben.
revoke-other-question = Gerät …{ $short } sperren? Es wird sofort getrennt und löscht alle Daten darauf, sobald es sich verbindet. Noch nicht gesendete Eingaben auf dem Gerät gehen verloren.
# "0.0.0, Build 1a2b3c4"
info-build-commit = { $version }, Build { $commit }
info-build-local-of = { $version }, { info-build-local }
org-role-admin = Administration
org-role-planner = Einsatzplanung (alle Einsätze)
# The accessible name of the withdraw button. Variables: $role, $person
org-role-withdraw-of = { $role } entziehen: { $person }
provider-test-passed-as = Die Testanmeldung hat geklappt. ORDO würde sie als { $user } erkennen.
# Variables: $user, $name, $email
provider-test-passed-name-email = Die Testanmeldung hat geklappt. ORDO würde sie als { $user } erkennen (Name: { $name }, E-Mail: { $email }).
provider-test-passed-name = Die Testanmeldung hat geklappt. ORDO würde sie als { $user } erkennen (Name: { $name }).
provider-test-passed-email = Die Testanmeldung hat geklappt. ORDO würde sie als { $user } erkennen (E-Mail: { $email }).
provider-not-found = Diesen Anbieter gibt es nicht mehr, oder er lässt sich nur in der Konfiguration ändern.
provider-not-tested = Bitte zuerst erfolgreich testen; die Testanmeldung gilt 10 Minuten.
provider-refused = Der Anmeldedienst hat die Änderung abgelehnt. Ist seine Schnittstelle für Anbieter freigeschaltet (DEX_API_CONNECTORS_CRUD)?
