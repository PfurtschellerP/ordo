tab-med = Sanität
tab-command = Einsatzleitung
# Workspaces of the incident command, by phase (ADR-0061).
ws-lead = Führung
ws-prepare = Vorbereitung
ws-crew = Besatzung
crew-tasks = Aufträge
crew-transports = Transporte für Ihr Einsatzmittel
crew-none-here = Sie sind in diesem Einsatz in keiner Besatzung eingetragen. Die Einsatzleitung trägt Sie unter Vorbereitung – Einsatzmittel ein.
dispatch = Disposition
people = Personal
crew-patients = Patienten
tab-helpers = Helfende
tab-devices = Geräte
incident-page = Einsatz

## The command structure and the map on one page (ADR-0095).
command-space = Führung & Raum
# Its label in the phone's bottom bar.
command-space-short = Struktur
tab-resources = Einsatzmittel
tab-hospitals = Kliniken
tab-free-units = Freie Transportmittel
tab-free-units-short = Freie Mittel
uhs-title = Unfallhilfsstellen
uhs-choose = UHS wählen
units-title = Einsatzmittel
units-empty = Noch keine Einsatzmittel.
unit-call-sign = Funkrufname
unit-type = Typ (z. B. RTW, KTW, Sanitätstrupp)
unit-organisation = Organisation
unit-submit = Einsatzmittel hinzufügen
unit-service = Dienst
unit-missing = Bitte Funkrufname und Typ angeben.
col-status = Status
col-set-status = Status setzen
no-status = –
set-status-label = Status von { $unit } setzen
unit-status-title = Einsatzmittel: Status und Aufträge
unit-status-empty = Noch keine Einsatzmittel im Einsatz. Sie werden in der Vorbereitung hinzugefügt.
# Starts with the button's visible text, so its name matches what is seen.
task-give-label = { task-give }: { $unit }
move-here = Hierher umsetzen
# Starts with the button's visible text, so its name matches what is seen.
move-here-label = { move-here }: { $person } zu { $here }
group-units = Einsatzmittel
action-add-unit = Einsatzmittel hinzufügen

## Incidents and roles (ADR-0046)
incidents-title = Einsätze
incidents-intro = Einsatz wählen. Rollen im Einsatz vergibt die Einsatzleitung oder die Einsatzplanung.
incidents-loading = Einsätze werden geladen …
incidents-offline = Die Einsatzliste ist nur mit Verbindung zum Server verfügbar.
# Logging out from the incident list: the device is unpaired, so another person can log in.
log-out = Abmelden
log-out-hint = Abmelden entkoppelt dieses Gerät; danach kann sich eine andere Person anmelden.
incidents-empty = Noch kein Einsatz freigegeben. Die Einsatzleitung oder die Einsatzplanung gibt Personen eine Rolle im Einsatz.
open = Öffnen
plan-title = Einsatz planen
plan-name = Name des Einsatzes
plan-submit = Einsatz anlegen
grant = Rolle vergeben
incident-state-label = Status
incident-page-title = Einsatz
incident-prepare = Vorbereiten: Lagekarte, Einsatzmittel und Personal mit Rollen und Führungsstruktur. Beginnen Sie den Einsatz, wenn er läuft.

## The command's overview (ADR-0067)
open-map = Lagekarte öffnen
open-dispatch = Zur Disposition
open-units = Zu den Einsatzmitteln
open-label = offen
no-open-tasks = Keine offenen Aufträge.
persons-on-scene = Einsatzkräfte vor Ort
units-on-scene = Einsatzmittel vor Ort
quick-links = Schnellzugriff
task-for = Für: { $assignee }
discharged-count = { $n } entlassen
start-incident = Einsatz beginnen
close-incident = Einsatz abschließen
incident-paused-label = unterbrochen
pause-incident = Einsatz unterbrechen
resume-incident = Einsatz fortsetzen
incident-pause-hint = Für Einsätze über mehrere Tage, etwa nachts bei einem Festival. Unterbrochen bleibt alles nutzbar; Lage und Einsatztagebuch laufen weiter.
incident-details = Angaben zum Einsatz
incident-description = Beschreibung
incident-description-hint = Zum Beispiel Ort, Veranstalter, erwartete Besucher, Besonderheiten.
incident-starts = Geplanter Beginn
incident-ends = Geplantes Ende
incident-details-save = Angaben speichern

## Treatment places
places = Behandlungsplätze

## Transport requests (TRANS-1, TRANS-3)
transports = Transporte
transports-empty = Keine offenen Transporte.

## Overview for the medical lead (UHS-8).
overview = Übersicht

## Units (UNIT-1…6).
strength-title = Kräfte vor Ort
strength = Stärke
strength-legend-title = Stärke (Führer / Unterführer / Mannschaft)
strength-numbers = Bitte die Stärke als drei ganze Zahlen angeben oder leer lassen.
leaders = Führer
sub-leaders = Unterführer
crew-count = Mannschaft
by-organisation = Nach Organisation
by-service = Nach Dienst
no-service = ohne Dienst
no-organisation = ohne Organisation
crew = Besatzung
crew-none = Keine Besatzung erfasst.
crew-member = Name
crew-qualifications = Qualifikationen
crew-add = Zur Besatzung hinzufügen
crew-remove = Entfernen
release = Aus dem Einsatz entlassen
back-in = Wieder im Einsatz

## [impl->dsn~detailed-strength~1]
unit-leaders = Führer von Verbänden
org-chart = Führungsstruktur als Organigramm
special-functions-legend = Davon mit besonderen Funktionen
strength-detailed = Stärke ausführlich angeben: Führer von Verbänden und Zugführer getrennt, besondere Funktionen wie Ärzte und Atemschutzgeräteträger gesondert (DV 102, 2024)
# "davon 2 × Ärzte, 6 × AGT".
special-functions = davon { $listed }
unreported = ohne Stärkemeldung: { $n }
# "3 – Einsatz übernommen, seit 10:12"
status-since = { $status }, seit { $since }
crew-label = { crew } von { $unit } ({ $n })
crew-remove-label = { $name } aus der Besatzung entfernen
release-label = { $unit } aus dem Einsatz entlassen
back-in-label = { $unit } wieder im Einsatz
released-list = Entlassene Einsatzmittel ({ $n })
released-at = Entlassen um { $at }
crew-title = Besatzung { $unit }
attention-due = Rückfrage fällig: { $unit } – { $task }
attention-new-task = Neuer Auftrag: { $task }
attention-unassigned = Auftrag ohne Einsatzmittel: { $task }
case-register = Patient aufnehmen
open-in-maps-app = In der Karten-App öffnen
log-report-closed = Meldung abgeschlossen: { $report }
log-report-patient = Patient aus dem Auftrag { $task } aufgenommen von { $unit }

## Tasks (TASK-1…4).
tasks-heading = Offene Aufträge
tasks-empty = Keine offenen Aufträge.
task-give = Auftrag geben
task-unit = Für
task-unit-choose = Einsatzmittel oder Funktion wählen
task-template = Vorlage
task-template-none = ohne Vorlage
task-text = Auftrag
task-follow-up = Nachfragen in (Minuten, optional)
task-missing = Bitte Empfänger und Auftrag angeben.
task-new = Neuer Auftrag
task-create = Auftrag anlegen
task-kind-label = Art
task-kind-other = Sonstiger Auftrag
task-kind-missing = Bitte die Art wählen.
task-unit-later = Später zuweisen
task-unit-only = Einsatzmittel wählen
task-unit-hint = Bei Patient und Ereignis optional: das Einsatzmittel lässt sich auch später zuweisen.
task-place = Ort
task-what = Was ist los?
task-what-hint = In wenigen Worten, ohne medizinische Angaben (die gehören in die Patientenakte).
task-received-label = Eingang
task-received-hint = Leer lassen für jetzt.
task-text-missing = Bitte kurz beschreiben, worum es geht.
task-place-missing = Bitte den Ort angeben.
task-too-long = Ort oder Beschreibung sind zu lang.
task-time-invalid = Bitte eine Uhrzeit angeben, z. B. 14:05.
task-assign-to = Zuweisen an
task-assign = Zuweisen
task-unit-missing = Bitte ein Einsatzmittel wählen.
task-send-further = Weiteres Einsatzmittel schicken
task-send-to = Zusätzlich schicken
task-send = Schicken
# Opens a small map centred on the Auftrag's pin.
pin-show = Pin zeigen
pin-show-label = Pin zeigen: { $task }
my-tasks = Meine Aufträge
pin-title = Pin auf der Karte (optional)
pin-hint = Adresse suchen oder in die Karte tippen. Mit der Tastatur: Karte mit den Pfeiltasten verschieben, dann „Pin in Kartenmitte setzen“.
pin-at-centre = Pin in Kartenmitte setzen
pin-clear = Pin entfernen
pin-none = Kein Pin gesetzt.
pin-set = Pin setzen
pin-move = Pin verschieben
pin-save = Pin speichern

## Map object cards (MAP-11).
map-details = Details
map-object-gone = Nicht mehr auf der Karte.
card-assign = Einsatzmittel zuweisen
card-open-task = Auftrag öffnen
card-current-task = Auftrag
card-edit = Bearbeiten
card-patients = Patienten nach Sichtungskategorie
card-open-uhs = UHS öffnen
qualifications-short = Qualifikationen
# "Anna Schmidt entfernt."
removed = { $what } entfernt.
map-details-label = { map-details }: { $what }
card-edit-label = { card-edit }: { $unit }
# "Notfallsanitäter/in: 2"
qualification-count = { $label }: { $n }
crew-names = Namen ({ $n })
pin-at = Pin: { $at }
task-received = Eingang { $at }
task-taken = Patient aufgenommen von { $unit } ({ $at })
follow-up-later = In 10 Minuten nachfragen
due = Fällig
tasks-title = { tasks-heading } ({ $n })
# "In Arbeit: Patient: Bühne Nord"
task-step-label = { $step }: { $task }
# One unit on an Auftrag, for accessible names: "RTW 2 – Patient: Bühne Nord".
unit-in-task = { $unit } – { $task }
task-units = Einsatzmittel im Auftrag
# "Nachfragen um 10:42"
follow-up-at = Nachfragen um { $at }
# "Fällig: Nachfragen seit 10:42"
follow-up-due = { due }: nachfragen seit { $at }
# "10:12, ben@…"
task-given-by = Gegeben { $at } von { $by }

## Command structure (CMD-1…4, ADR-0031).
structure-title = Führungsstruktur
structure-short = Struktur
structure-empty = Noch keine Führungsstruktur angelegt.
position = Funktion
position-choose = Funktion wählen
position-create = Funktion anlegen
position-kind = Art
position-kind-choose = Art wählen
position-name = Bezeichnung
position-parent = Unterstellt
position-top = oberste Ebene
position-missing = Bitte Art und Bezeichnung angeben.
position-close = Auflösen
no-position = ohne Zuordnung
no-leader = Leitung: nicht besetzt
leader-label = Leitung
leader-set = Leitung festlegen oder übergeben
uhs-assign = UHS zuordnen
leader = { leader-label }: { $name }
# What a member of a position is, under its name in the org chart.
member-uhs = UHS
# Shown, never enforced (ADR-0031).
span-of-control = { $n } direkt unterstellt: mehr als 5, Führungsspanne beachten.
position-close-label = { $title } auflösen

## A position put on the map, or found there, from the tree (CMD-5).
position-map-set-label = Auf der Karte setzen: { $title }
position-map-show-label = Auf der Karte zeigen: { $title }
# The place panel's status once a position was chosen from the tree.
map-place-ready = { $label }: auf die Stelle tippen oder in der Kartenmitte platzieren.
position-of = Funktion von { $unit }

## Situation map (MAP-1, MAP-2).
map-title = Lagekarte
map-short = Karte
map-label = Lagekarte. Pfeiltasten verschieben, Plus und Minus zoomen.
map-loading = Karte wird geladen …
map-hint-touch = Karte mit zwei Fingern verschieben
map-hint-wheel = Zoomen: Strg und Mausrad
map-hint-wheel-mac = Zoomen: ⌘ und Mausrad
map-unavailable = Die Karte kann auf diesem Gerät nicht angezeigt werden; die Liste unten zeigt alles, was auf ihr steht.
map-from-device = Kartengrundlage: auf diesem Gerät gespeichert (auch ohne Netz).
map-from-server = Kartengrundlage: vom Server.
map-no-region = Für diesen Einsatz wurde keine Kartengrundlage vorbereitet: die Objekte stehen auf neutralem Hintergrund.
map-objects = Auf der Karte
map-locate = Zu meinem Standort
map-locating = Standort wird bestimmt …
map-located = Die Karte zeigt Ihren Standort.
map-locate-denied = Der Standort ist nicht freigegeben. Erlauben Sie ORDO den Standort in den Einstellungen des Browsers.
my-position = Ich
my-position-show = Meinen Standort zeigen
my-position-shown = Die Karte zeigt Ihren Standort (Punkt „Ich“) und den Pin. Er bleibt auf diesem Gerät.
map-locate-failed = Der Standort konnte nicht bestimmt werden.
map-objects-none = Noch nichts auf der Karte.
map-show = Zeigen
map-remove = Entfernen
map-place-hint = Auf die Stelle tippen – oder die Karte so verschieben, dass die Stelle am Fadenkreuz liegt (auch mit den Pfeiltasten), und in der Kartenmitte platzieren.
map-place-kind = Was
map-place-which = Welches
map-place-choose = Bitte wählen
map-place-label = Bezeichnung
map-place-submit = In der Kartenmitte platzieren
map-place-missing = Bitte auswählen, was platziert werden soll.
incident-location = Einsatzort
map-saving = Die Kartengrundlage wird für die Nutzung ohne Netz auf diesem Gerät gespeichert …
map-save-failed = Die Karte konnte nicht gespeichert werden.
map-kind-symbol = Taktisches Zeichen
map-symbol-label = Beschriftung (optional)
map-show-label = { $label } auf der Karte zeigen
map-remove-label = { $label } von der Karte entfernen

## The map editor (ADR-0064)
map-edit = Lagekarte bearbeiten
map-edit-hint = Einsatzstelle, Einsatzmittel, UHS und Funktionen auf der Karte setzen, Flächen und Linien zeichnen. Die Führung sieht die Karte unter Führung – Lagekarte.
map-edit-not-allowed = Die Karte bearbeitet die Einsatzleitung; hier ist sie nur zu sehen.
map-edit-object = Bearbeiten
map-tools = Werkzeug
tool-select = Auswählen
tool-place = Symbol setzen
tool-area = Fläche
tool-line = Linie
map-tap-to-choose = Auf ein Objekt der Karte tippen oder es in der Liste unten bearbeiten.
map-move-hint = Auf die neue Stelle tippen – oder die Karte so verschieben, dass sie am Fadenkreuz liegt (auch mit den Pfeiltasten), und in die Kartenmitte setzen.
map-move-centre = In die Kartenmitte setzen
map-deselect = Auswahl aufheben
reshape-hint = Ecken ziehen, um die Form zu ändern; aus der Mitte einer Seite eine neue Ecke ziehen; die Form ziehen, um sie zu verschieben. Ohne Ziehen: „Neu zeichnen“.
reshape-apply = Übernehmen
reshape-discard = Verwerfen
reshape-redraw = Neu zeichnen
map-reshape-failed = Diese Form lässt sich hier nicht ziehen; „Neu zeichnen“ geht immer.
map-edit-label = { $label } bearbeiten
map-chosen = { $label } ausgewählt.
map-moved = { $label } verschoben.
map-placed = { $label } platziert.
map-reshaped = { $label } geändert.
map-drawn = { $label } gespeichert.
map-removed = { $label } entfernt.
draw-redraw = { $label } neu zeichnen
log-hint = Jede Änderung in diesem Einsatz, die dieses Gerät kennt, mit Zeit, Person und Gerät, die neueste zuerst. Einträge werden nie geändert: Eine Korrektur ist ein neuer Eintrag, der auf den alten verweist.
log-empty = Noch keine Einträge.
log-pending = noch nicht gesendet
log-by = { $user } · Gerät { $device }
log-corrects = (korrigiert den Eintrag von { $at })
log-pause-since = Pause ab { $at }
log-address = Anschrift
log-task-given = Auftrag an { $to }: { $task }
log-position-closed = { $title } aufgelöst
log-leader = { $title }: Leitung { $leader }
log-map-placed = { $label } auf der Lagekarte platziert
log-map-removed = { $label } von der Lagekarte entfernt

## Map region from the server (MAP-8)
map-region-title = Kartengrundlage
map-region-short = Grundlage
map-region-hint = Die Kartengrundlage dieses Einsatzes für die Lagekarte: ein Umkreis aus der freien Weltkarte (OpenStreetMap über Protomaps). Nur die Geräte der Einsatzkräfte dieses Einsatzes erhalten sie. Geräte, die eine Offline-Karte gespeichert haben, speichern sie danach neu.
map-region-none = Für diesen Einsatz ist noch keine Kartengrundlage erstellt.
map-region-new = Neue Kartengrundlage
map-region-lat = Breitengrad (Mitte)
map-region-lon = Längengrad (Mitte)
map-region-radius = Umkreis (km, 1 bis 50)
map-region-from-incident = Die Mitte ist der Einsatzort.
map-region-coordinates = Mitte als Dezimalgrad, z. B. 48,37 und 10,90.
map-region-cut = Kartengrundlage erstellen
map-region-progress = Fortschritt
map-region-invalid = Bitte Breitengrad (−85 bis 85), Längengrad (−180 bis 180) und einen Umkreis von 1 bis 50 km angeben.
draw-category = Art
draw-add-corner = Ecke in der Kartenmitte setzen
draw-undo = Letzte Ecke entfernen
draw-cancel = Abbrechen
draw-finish = Fertig
draw-too-few-area = Eine Fläche braucht mindestens drei Ecken.
draw-too-few-line = Eine Linie braucht mindestens zwei Punkte.

## Helper register (HELP-1, HELP-2, ADR-0071)
helpers-title = Helferinnen und Helfer
helpers-intro = Persönliche Angaben der Helfenden dieses Einsatzes. Nur die Einsatzleitung sieht sie.
helpers-intro-roles = Wer in diesem Einsatz welche Rolle hat. Persönliche Angaben sieht nur die Einsatzleitung.
helpers-empty = Noch niemand erfasst.
helper-add-title = Person aufnehmen
helper-add = Aufnehmen
role-add-title = Rolle vergeben
helper-complete = Angaben ergänzen
helper-complete-label = { helper-complete }: { $name }
helper-complete-title = Angaben zu { $name } ergänzen
helper-name = Name
helper-phone = Telefon
helper-phone-hint = Mit Vorwahl, z. B. 0171 1234567 oder +43 664 1234567.
helper-phone-invalid = Die Telefonnummer ist keine gültige Nummer mit Vorwahl, z. B. 0171 1234567.
helper-street = Straße und Hausnummer
helper-postcode = PLZ
helper-postcode-invalid = Die Postleitzahl hat nicht die Form des Landes, z. B. 55116.
helper-town = Ort
helper-role = Rolle im Einsatz
helper-roles = Rollen im Einsatz
or = oder
guest-duty-ended = Das Dienstende ist vorbei. Für einen Zugang ohne Konto den Dienst neu beginnen oder das Dienstende ändern.
guest-until-closed = Gast-Zugang bis zum Einsatzende
guest-no-command = Die Einsatzleitung braucht ein ORDO-Konto.
guest-access = Zugang per QR-Code
guest-code-title = QR-Code für den Zugang
guest-code-refused = Der Zugang konnte nicht erstellt werden.
guest-code-alt = QR-Code, der dieses Gerät des Helfers mit dem Einsatz koppelt
guest-link = Oder diesen Link auf dem Gerät öffnen:
guest-code-close = Fertig
guest-code-for = Für { $name }: mit der Kamera jedes Geräts scannen, das { $name } nutzt. Gilt bis { $expires }.
guest-code-until-closed = Für { $name }: mit der Kamera jedes Geräts scannen, das { $name } nutzt. Gilt bis zum Einsatzende.
guest-access-label = { guest-access }: { $name }
guest-until = Gast-Zugang bis { $until }
role-missing = Bitte eine Rolle wählen.
helper-account-hint = Konto der Organisation wählen; der Name wird übernommen. Leer lassen für Helfende ohne Konto.
diet-other = andere …
diet-other-label = Andere Ernährung
helper-birth-date-label = Geburtsdatum
helper-diet = Ernährung
diet-none = ohne Angabe
helper-duty-end = Dienstende
helper-account-label = ORDO-Konto
helper-name-missing = Bitte einen Namen angeben.
helper-too-long = Jede Angabe darf höchstens 200 Zeichen lang sein.
helper-birth-date = Bitte das Geburtsdatum als Datum angeben.
helper-account = Das ORDO-Konto ist keine gültige Kennung, z. B. anna@ordo.example.org.
helper-time = Bitte Dienstbeginn und -ende als Datum mit Uhrzeit angeben.
helper-not-saved = Die Angaben konnten nicht gespeichert werden.
duty-start = Dienst beginnen
duty-end = Dienst beenden
# Starts with the button's visible text, so its name matches what is seen.
duty-start-label = { duty-start }: { $name }
# Starts with the button's visible text, so its name matches what is seen.
duty-end-label = { duty-end }: { $name }
helper-edit-title = { $name } bearbeiten
duty-pause = Dienst pausieren
duty-resume = Dienst fortsetzen
duty-pause-label = { duty-pause }: { $name }
duty-resume-label = { duty-resume }: { $name }
# "in Pause seit 14:02"
paused-since = in Pause seit { $at }
on-duty-since = im Dienst seit { $at }
duty-from-to = Dienst { $from } – { $to }
duty-from = Dienst ab { $from }
duty-to = Dienst bis { $to }
# The person typed for a crew is not in the helper register (HELP-3).
not-registered = { $name } ist nicht im Helferregister. Mit dem Namen erfassen und hinzufügen?
register-and-add = Erfassen und hinzufügen
# Starts with the button's visible text, so its name matches what is seen.
register-and-add-label = { register-and-add }: { $name }
# Registration cards (HELP-5, ADR-0072).
# "Meldekarte fotografieren", with the profile's label of the card ("Karte" for several).
card-generic = Karte
card-photo = { $label } fotografieren
card-reading = Karte wird gelesen …
card-not-recognised = Keine Karte erkannt. Bitte noch einmal fotografieren: Karte flach, QR-Code ganz im Bild und scharf.
card-read = { $card } von { $name } gelesen. Bitte prüfen und erfassen.

## Converted by hand: counts, lists and values of ORDO's types
log-title = Einsatztagebuch
task-follow-up-number = Bitte die Minuten als ganze Zahl angeben.
log-pause-ended = Pause beendet
draw-hint = Auf jede Ecke tippen – oder die Karte so verschieben, dass die Ecke am Fadenkreuz liegt (auch mit den Pfeiltasten), und sie setzen. Dann „Fertig“.
helper-duty-start = Dienstbeginn
# "keine offenen Aufträge", "1 offener Auftrag", "3 offene Aufträge"
open-tasks-of = { $n ->
    [0] keine offenen Aufträge
    [one] 1 offener Auftrag
   *[other] { $n } offene Aufträge
}
# People on more than one crew or staff list (UNIT-8), e.g. "Ben (RTW 1, UHS Süd)".
double-booked-people = Mehrfach eingeteilt, eventuell doppelt gezählt: { $people }
# The person being added is on other crews or staff lists already (UNIT-8).
assigned-elsewhere-to = { $person } ist schon bei { $posts } eingeteilt. Umsetzen, damit niemand doppelt gezählt wird?
# Variables: $unit, $overwritten, $kept
lost-update-of = Achtung { $unit }: Status { $overwritten } wurde gleichzeitig durch { $kept } ersetzt. Bitte prüfen.
action-status-of = Status { $status }
follow-ups-due = { $n ->
    [one] 1 Nachfrage fällig
   *[other] { $n } Nachfragen fällig
}
unreported-strength = { $n ->
    [0] Alle Einsatzmittel haben ihre Stärke gemeldet.
    [one] 1 Einsatzmittel ohne Stärkemeldung.
   *[other] { $n } Einsatzmittel ohne Stärkemeldung.
}
transports-open = { $n ->
    [0] Transporte: keine offen
   *[other] Transporte: { $n } offen
}
summary-map = { $objects ->
    [0] Noch nichts auf der Karte.
    [one] 1 Objekt auf der Karte.
   *[other] { $objects } Objekte auf der Karte.
}
# The state as people read it, with a pause: "Laufend – unterbrochen".
incident-state-paused = { $state } – { incident-paused-label }
# "13.11.2026, 18:00 bis 15.11.2026, 23:00"
incident-from-to = { $starts } bis { $ends }
incident-from = ab { $starts }
incident-to = bis { $ends }
# Variables: $user, $role
withdraw-role = Entziehen: { $role } von { $user }
# The strength read out: "Stärke: 0 Führer, 1 Unterführer, 8 Mannschaft, 9 gesamt".
strength-count = { $count } { $part }
strength-read = { strength }: { $parts }, { $total } gesamt
strength-special = { $strength }, { $special }
strength-platoon-leaders = Zugführer
strength-forces = Einsatzkräfte
# A strength's legend, e.g. "Stärke (Führer / Unterführer / Mannschaft)".
strength-legend-of = { $title } ({ $parts })
units-count = { $n ->
    [one] 1 Einsatzmittel
   *[other] { $n } Einsatzmittel
}
# Variables: $kind, $received, $place, $text
log-report-line = Meldung aufgenommen: { $kind }, { $received } – { $place } – { $text }
# "Rolle Arzt/Ärztin von Anna Schmidt entzogen."
role-removed-from = Rolle { $role } von { $who } entzogen.
# Variables: $action, $task
pin-edit-of = { $action }: { $task }
task-kind-event = Ereignis
task-kind-patient = Patient
# "Patient: Bühne Nord"
task-label-of = { $kind }: { $place }
# How far an Auftrag is (TASK-2).
task-phase-received = Eingegangen
task-phase-assigned = Zugewiesen
task-phase-in-progress = In Arbeit
task-phase-done = Erledigt
task-phase-cancelled = Abgebrochen
# Variables: $kind, $received, $place, $task
log-task-received-line = Auftrag eingegangen: { $kind }, { $received } – { $place } – { $task }
log-task-received-to = { $line } – an { $to }
log-task-assigned = Auftrag „{ $task }“ an { $unit } zugewiesen
task-step-open = Öffnen
task-step-in-progress = In Arbeit
task-step-done = Erledigt
task-step-cancel = Abbrechen
# Time left until a follow-up, e.g. "noch 7:05 min"; "jetzt" at zero.
countdown-now = jetzt
# Variables: $minutes, $seconds
countdown-left = noch { $minutes }:{ $seconds } min
# "Übergabe: Dr. M. → Dr. K."
handover-from-to = Übergabe: { $from } → { $to }
map-kind-uhs = UHS
map-kind-drawing = Zeichnung
map-kind-report = Meldung
map-saved-size = Kartengrundlage auf diesem Gerät gespeichert ({ $megabytes } MB), auch ohne Netz nutzbar.
# "Bei Einsatzmitteln: 2 (SK I 1 · SK II 1)"
with-units-line = { $label }: { $total } ({ $parts })
# Variables: $module, $event-type
log-unknown = { $module }: { $event-type }
log-planned = Einsatz „{ $name }“ angelegt
# Variables: $added, $unit-type, $organisation
log-unit-added-line = { $added } ({ $unit-type }, { $organisation })
log-helper-registered = { $name } aufgenommen
log-helper-saved = Angaben zu { $name } gespeichert
log-helper-changes = { $name }: { $changes }
log-cleared = { $label } gelöscht
log-fields-changed-list = Angaben geändert: { $fields }
# Variables: $unit, $n
log-crew = { $n ->
    [0] Besatzung von { $unit }: niemand
    [one] Besatzung von { $unit }: 1 Person
   *[other] Besatzung von { $unit }: { $n } Personen
}
# A unit's step on an Auftrag (ADR-0085): "Auftrag „…“: RTW 2 In Arbeit".
log-unit-step-of = Auftrag „{ $task }“: { $unit } { $status }
log-task-updated-to = Auftrag „{ $task }“: { $status }
log-task-follow-up-changed = Auftrag „{ $task }“: Nachfrage geändert
log-assigned-to = { $what } unterstellt: { $position }
log-assigned-none = { $what }: Unterstellung aufgehoben
map-region-preparing = Kartengrundlage wird vorbereitet …
map-region-tiles = Kartengrundlage wird erstellt: { $done } von { $total } Kacheln …
map-region-ready-size = Kartengrundlage bereit ({ $megabytes } MB).
# Variables: $km, $lat, $lon
map-region-area = Umkreis { $km } km um { $lat }, { $lon }.
map-region-created = Erstellt { $at }.
map-region-failed = Die Kartengrundlage konnte nicht erstellt werden: { $error }
draw-corners = { $n ->
    [0] Noch keine Ecke gesetzt.
    [one] 1 Ecke gesetzt.
   *[other] { $n } Ecken gesetzt.
}
# The diets of the helpers still expected or on duty, for catering.
diet-counts-line = Ernährung (ohne beendeten Dienst): { $list }
diet-count = { $n } × { $diet }
# Starts with the button's visible text, so its name matches what is seen.
helper-edit-of = { $edit }: { $name }

## Führung – Lage (ADR-0099): the map beside the units and the treatment places.
lage = Lage
# The groups the command sees its units in (UNIT-10); the profile gives each status its group.
group-available = Einsatzbereit
group-task = Im Auftrag
group-transport = Transport
group-unavailable = Nicht einsatzbereit
group-none = Ohne Status
# The first of the type filter's chips: "Alle 14".
filter-all = Alle
units-filter = Typ
units-only-available = Nur einsatzbereite
units-glance-empty = Kein Einsatzmittel passt zur Auswahl.
open-lage = Zur Lage
# "4 von 14 einsatzbereit", the first number large.
units-available-of = von { $total } einsatzbereit
col-type = Typ
col-free = frei
col-all = gesamt

## Führung & Raum: actions at each position (ADR-0102), forms in a dialog.
position-new = Funktion
lead-word = Leitung
sub-position-word = Unterfunktion
uhs-word = UHS
map-set-word = Setzen
leader-set-label = Leitung festlegen: { $title }
position-sub-label = Unterfunktion anlegen: { $title }
uhs-assign-label = UHS zuordnen: { $title }
