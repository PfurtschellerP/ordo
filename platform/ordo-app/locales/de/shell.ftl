app-name = ORDO
skip-link = Zum Inhalt springen
pair-title = Gerät koppeln
pair-intro = Melde dich mit dem Konto deiner Organisation an. Dieses Gerät erhält dabei einen eigenen Schlüssel, der es nie verlässt; ORDO speichert keine Passwörter.
pair-login = Anmelden
pair-qr = Mit QR-Code koppeln
pair-qr-hint = Für gemeinsam genutzte Tablets: Mit einem angemeldeten Handy den QR-Code scannen und freigeben.
pair-qr-alt = QR-Code zum Koppeln dieses Geräts
pair-code = Code: { $code }
pair-qr-link = Ohne Kamera: auf einem anderen Gerät öffnen und den Code eingeben:
pair-waiting = Warte auf Freigabe …
pair-no-login = Für diesen Server ist keine Anmeldung eingerichtet.
pair-manual = Entwicklung: Gerät manuell koppeln
pair-manual-intro = Für Testumgebungen ohne Anmeldung: Angaben aus der Seed-Datei des Servers und aus `ordo generate-device-key`.
login-failed = Anmeldung fehlgeschlagen
relogin-title = Anmeldung abgelaufen
relogin-intro = Die Anmeldung dieses Geräts ist abgelaufen. Bitte erneut anmelden – die Daten auf dem Gerät bleiben erhalten und werden danach weiter abgeglichen.
network-error = Server nicht erreichbar
pair-user = Benutzer (name@server)
pair-device = Geräte-ID
pair-key = Privater Schlüssel des Geräts
pair-incident = Einsatz-ID
pair-submit = Koppeln
unpair = Gerät entkoppeln
signed-in-as = Angemeldet als
avatar-change = Profilbild ändern
avatar-remove = Profilbild entfernen
retry = Erneut versuchen
starting = ORDO wird gestartet …
online = Verbunden
connection-label = Verbindung: { $state }. { $pending }
connecting = Verbinde …
offline = Offline – Eingaben werden gespeichert und später gesendet
my-devices = Meine Geräte
page-tabs = Teile der Seite
revoked-title = Gerät gesperrt
revoked-text = Dieses Gerät wurde gesperrt. Sein Schlüssel und alle Einsatzdaten wurden von diesem Gerät gelöscht. Um es wieder zu nutzen, das Gerät neu koppeln.
pair-again = Neu koppeln
device-name-label = Name dieses Geräts
device-name-hint = So erscheint das Gerät in der Geräteliste, z. B. „Tablet UHS Nord“.
device-revoked = Gerät …{ $short } gesperrt
retagged = Patient { $old }: neue Kartennummer { $new }
uhs-created = { $name } angelegt
unit-added = Einsatzmittel { $unit } hinzugefügt
launcher = Arbeitsbereich wechseln
views = Ansichten
incident = Einsatz
reason-no-access = Kein Zugriff in diesem Einsatz
reason-not-connected = Verfügbar nach der ersten Verbindung
no-workspace = Für dieses Gerät ist in diesem Einsatz noch kein Arbeitsbereich freigegeben.

## Navigation and header
menu = Menü
search = Suchen
storage-failed = Gerätespeicher voll oder gesperrt: Eingaben bleiben nur bis zum Senden erhalten. Nicht neu laden.

## Appearance (UX-3)
appearance = Darstellung
profile = Profil und Gerät
hand = Bedienung mit

## Vim keys (ADR-0090). Key names stay English (ADR-0036).
keymap = Tastatur
keys-title = Tastenkürzel
keys-on = Vim-Tasten sind auf diesem Gerät an. Sie wirken, solange kein Textfeld den Fokus hat, und ändern nie Daten.
keys-off = Vim-Tasten sind aus. Einschalten im Menü unter „Tastatur“.
keys-vim = Vim-Tasten
keys-turn-on = Vim-Tasten einschalten
keys-turn-off = Vim-Tasten ausschalten
keys-always = Immer
key-keys = Tasten
key-does = Wirkung
key-help = Diese Übersicht
key-palette = Befehlspalette
key-search = Suchen oder filtern
key-move = Zum nächsten Element links, darunter, darüber, rechts – auf der ganzen Seite
key-first-last = Erstes / letztes Element der Seite
key-tabs = Nächster / vorheriger Reiter
key-views = Erste bis vierte Ansicht
key-menu = Menü
key-rail = Ansichten am Rand: j/k wählen, o öffnet, l oder Esc zurück
key-open = Eintrag öffnen
key-insert = Ins erste Eingabefeld
key-scroll = Halbe Seite nach unten / oben
key-escape = Textfeld verlassen
key-in-list = In einer Liste: nächster / vorheriger Eintrag
key-close = Dialog oder Menü schließen
theme-set = Darstellung: { $label }
photo-added = Foto für Patient { $tag } gespeichert
photo-removed = Foto für Patient { $tag } entfernt
photo-restored = Foto von Patient { $tag } wiederhergestellt
photo-failed = Das Bild konnte nicht gelesen werden.
photo-too-large = Das Bild ist auch verkleinert zu groß.
edit = Bearbeiten
save = Speichern
cancel = Abbrechen

## Command palette (UX-10)
palette-title = Suchen und Befehle
palette-placeholder = Patient, Einsatzmittel, Ansicht oder Befehl …
palette-results = Ergebnisse
palette-none = Nichts gefunden.
palette-keys = wählen
palette-run = ausführen
palette-close = schließen
palette-back = zurück
group-views = Ansicht
group-actions = Befehl
my-roles = Rollen
no-role = keine (Einsatzplanung)
change-incident = Einsatz wechseln
incident-details-saved = Angaben zum Einsatz gespeichert.
incident-times-invalid = Das Ende liegt vor dem Beginn, oder ein Datum ist unvollständig.
place-label = Platz { $n }
places-set = { $n } Behandlungsplätze
transport-history = Verlauf
transport-requested = { $tag }: Transport angefordert
order-given = { $tag }: Anordnung erfasst
order-confirmed-said = Anordnung bestätigt
order-recorded = { $tag }: Mündliche Anordnung erfasst
section-close = Schließen
# Closes a dialog.
close = Schließen
done = Fertig
discharge = Entlassen
readmitted = { $tag } wieder aufgenommen
hospital-saved = Klinik gespeichert: { $name }
capacity-saved = Aufnahme gespeichert: { $name }

## API versions (ADR-0052).
version-unsupported = Diese App und der Server haben keine gemeinsame Version. Bitte die Seite neu laden.
crew-set = Besatzung von { $unit } gespeichert
unit-released = { $unit } aus dem Einsatz entlassen
# The patient counts of patients with units' crews.
with-units-label = Bei Einsatzmitteln
handed-over = Patient { $tag } an { $to } übergeben, Übernahme offen
handed-over-quick = Schnellübergabe für Patient { $tag } gestartet
handed-outside = Patient { $tag } außerhalb von ORDO übergeben an { $receiver }
taken = Patient { $tag } übernommen von { $by }
uhs-closed = { $name } aufgelöst
uhs-updated = { $name } gespeichert

## What needs the command's attention (TASK-3): the bell in the header.
attention = Hinweise
attention-none = Nichts, das gerade Aufmerksamkeit braucht.

## Reports as Aufträge (REP-1…5, ADR-0081); events of ADR-0075 still read.
report-closed = Meldung abgeschlossen
task-given-label = Auftrag gegeben
task-patient-taken = Patient aus dem Auftrag aufgenommen
undo = Rückgängig
close-label = { section-close }: { $what }
task-recorded = Auftrag angelegt: { $place }
follow-up-later-done = Nachfrage verschoben
task-given = Auftrag an { $unit } gegeben
structure-saved = Führungsstruktur gespeichert
position-created = { $name } angelegt
map-saved-label = Lagekarte gespeichert

## Address search (ADR-0066)
address-label = Adresse suchen
address-submit = Suchen
address-searching = Suche läuft …
address-empty = Bitte eine Adresse eingeben.
address-too-long = Bitte höchstens 200 Zeichen eingeben.
address-offline = Die Adresssuche braucht eine Verbindung zum Server.
address-not-found = Nichts gefunden. Bitte anders schreiben, zum Beispiel mit dem Ort.
address-off = Auf diesem Server ist keine Adresssuche eingerichtet.
address-refused = Die Adresssuche ist für die Einsatzleitung.
address-failed = Die Adresssuche antwortet gerade nicht. Bitte später erneut versuchen.
address-places = Gefundene Orte
address-attribution = Adressen aus OpenStreetMap, © OpenStreetMap-Mitwirkende (ODbL).
address-chosen = Gewählt: { $label }
notices-back = Zurück

## Stored data that cannot be used (local persistence)
stored-state-unusable = Die Daten auf diesem Gerät können von dieser Version nicht gelesen werden ({ $reason }). Sie bleiben unverändert erhalten; bitte die App neu laden oder die Administration verständigen.

## Display names (ORG-7)
display-name = Anzeigename
display-name-hint = So sehen andere Sie in ORDO. Ohne eigenen Namen gilt der Name aus der Anmeldung.
display-name-save = Namen speichern
display-name-reset = Namen aus der Anmeldung verwenden
guest-pair-title = Zugang für Helfende
guest-ended-title = Zugang beendet
guest-ended = Ihr Zugang zu diesem Einsatz ist mit Ihrem Dienst beendet. Für einen neuen Zugang bitte die Einsatzleitung fragen.
guest-pairing = Dieses Gerät wird mit dem QR-Code gekoppelt …
guest-code-invalid = Der QR-Code gilt nicht mehr (schon benutzt, abgelaufen oder Dienst beendet). Bitte die Einsatzleitung um einen neuen.
helper-saved = Angaben zu { $name } gespeichert.

## Counts
pending = { $n ->
    [0] Alles gesendet
    [one] 1 Eingabe noch nicht gesendet
   *[other] { $n } Eingaben noch nicht gesendet
}
palette-count = { $n ->
    [0] { palette-none }
    [one] 1 Ergebnis
   *[other] { $n } Ergebnisse
}
attention-label = { $n ->
    [0] { attention }: keine
   *[other] { attention }: { $n }
}
address-found = { $n ->
    [one] 1 Ort gefunden. Bitte wählen.
   *[other] { $n } Orte gefunden. Bitte wählen.
}

## How long ago, from minutes: "gerade eben", "vor 5 Min.", "vor 2 Std. 5 Min.", "vor 3 Tagen"
ago-now = gerade eben
ago-minutes = vor { $minutes } Min.
ago-hours = vor { $hours } Std.
# Variables: $hours, $minutes
ago-hours-minutes = vor { $hours } Std. { $minutes } Min.
ago-days = { $days ->
    [one] vor 1 Tag
   *[other] vor { $days } Tagen
}

## Triage categories in the DE profile
triage-immediate = SK I
triage-urgent = SK II
triage-minor = SK III
triage-expectant = SK IV
triage-deceased = Verstorben
triage-uninjured = Betroffene
# A place kept for a triage category, e.g. "SK I – Platz 2" (UHS-13).
category-place = { $category } – Platz { $n }

## Log lines with values of ORDO's types
patient-registered = Patient { $tag } registriert: { $category }, { $uhs }
patient-retriaged = Patient { $tag } nachgesichtet: { $category }
unit-status-set = { $unit }: Status { $status }
# A unit status with its code, e.g. "3 – Einsatz übernommen".
status-with-code = { $code } – { $label }
# Variables: $user, $role
role-granted-to = Rolle { $role } an { $user } vergeben
# Variables: $user, $role
role-withdrawn-from = Rolle { $role } von { $user } entzogen
incident-state-changed-to = Einsatz: { $state }
priority-set = { $tag }: Transportpriorität gesetzt
priority-removed = { $tag }: Transportpriorität entfernt
place-assigned-to = { $tag } auf { $place }
place-none = { $tag }: kein Behandlungsplatz
transport-status-set-to = Transport: { $status }
order-status-set-to = Anordnung: { $status }
task-status-set-to = Auftrag: { $status }
# "Entlassen: Transportiert"
discharged-as-outcome = { discharge }: { $outcome }
discharged-with = { $tag }: { $outcome }

## Themes, hands and keyboards (UX-3, ADR-0049, ADR-0090)
theme-auto = Automatisch
theme-day = Tag
theme-night = Nacht
theme-high-contrast = Hoher Kontrast
hand-right = rechter Hand
hand-left = linker Hand
keymap-standard = Standard
keymap-vim = Vim-Tasten

## Roles offered when granting (prototype; later from the profile)
role-commander = Einsatzleitung
role-medic = Sanitätsdienst
role-physician = Arzt/Ärztin
role-paramedic = Notfallsanitäter/in

## States
incident-now-paused = Einsatz unterbrochen
incident-now-resumed = Einsatz fortgesetzt
incident-planned = Geplant
incident-active = Laufend
incident-closed = Abgeschlossen
transport-status-requested = Angefordert
transport-status-assigned = Zugewiesen
transport-status-loading = Wird verladen
transport-status-departed = Abgefahren
transport-status-arrived = Am Ziel
transport-status-handed-over = Patient am Ziel übergeben
transport-status-cancelled = Storniert
order-status-given = Angeordnet
order-status-acknowledged = Übernommen
order-status-done = Durchgeführt
order-status-withdrawn = Zurückgenommen
outcome-released-on-site = Vor Ort entlassen
outcome-transported = Transportiert
outcome-refused = Behandlung oder Transport verweigert
outcome-deceased = Verstorben
task-status-open = Offen
task-status-in-progress = In Arbeit
task-status-done = Erledigt
task-status-cancelled = Abgebrochen

## Dates and numbers, as each language writes them
# Day and month with two digits. Variables: $day, $month, $year
date-of = { $day }.{ $month }.{ $year }
# Variables: $date, $time
date-time-of = { $date }, { $time }
# The decimal separator, as in "1,5 MB".
decimal-separator = ,

## Errors of the device's own workings (pairing, storage, randomness)
no-random = Keine Zufallszahlen verfügbar
no-incident-chosen = Kein Einsatz gewählt
device-id-invalid = Geräte-ID: { $error }
private-key-format = Privater Schlüssel: 64 Hex-Zeichen erwartet
photo-not-found = Foto nicht gefunden
photo-other-version = Foto in einer anderen Version
app-state-unavailable = App-Zustand nicht verfügbar
no-secure-context = Sichere Speicherung nicht verfügbar: Die App muss über HTTPS geöffnet werden.
no-device-storage = Kein Gerätespeicher
# A device's suggested name from its browser and system (ORG-8): "Firefox auf Android".
device-name-on = { $browser } auf { $system }

## Language of this device (I18N-1)
# Each language names itself in its own file.
language-name = Deutsch
language = Sprache
language-auto = Automatisch (wie der Browser)
language-after-reload = Die Sprache gilt nach dem nächsten Neuladen. Bitte erst neu laden, wenn alles gesendet ist.
