crew-places = Freie Behandlungsplätze
crew-not-held = Dieser Patient ist nicht (mehr) bei Ihrem Einsatzmittel. Seine Akte sieht, wer ihn jetzt versorgt.
# The UHS as a unit (UHS-12).
uhs-strength = Stärke der UHS
uhs-strength-hint = Die UHS zählt bei den Kräften vor Ort als Einsatzmittel: mit dieser Stärke, sonst mit ihrem Personal.
register-title = Patient registrieren
register-tag = Nummer der Patientenanhängekarte
# A card number already registered (PAT-12): where the patient is, and since when.
tag-in-use = Die Karte { $tag } ist schon registriert: { $holder }, seit { $time }.
# A card number registered again (PAT-12): by whom, where and when.
# Variables: $person, $holder, $time
tag-registered-again = Noch einmal registriert { $time } von { $person } ({ $holder }).
tag-twice = Karte zweimal registriert
tag-twice-hint = Zwei Geräte haben diese Kartennummer registriert, während sie offline waren, oder eine Nummer wurde falsch eingegeben. Wer später registriert hat, gibt seinem Patienten eine neue Karte und trägt deren Nummer ein; was dieses Gerät seitdem dokumentiert hat, geht mit.
retag-ask = Ihre Registrierung kam später. Bitte geben Sie Ihrem Patienten eine neue Patientenanhängekarte.
retag-number = Nummer der neuen Karte
retag-submit = Nummer ändern
retag-in-use = Die Karte { $tag } ist schon registriert. Bitte eine unbenutzte Karte nehmen.
register-category = Sichtungskategorie
register-uhs = Patient bei
register-submit = Registrieren
board-empty = Noch keine Patienten registriert.
col-retriage = Nachsichtung
not-sent = noch nicht gesendet
sent-label = gesendet
uhs-empty = Noch keine UHS angelegt. Die Einsatzleitung legt sie unter Vorbereitung – Einsatzmittel an.
uhs-name = Name der UHS
uhs-capacity = Kapazität (Patienten)
uhs-submit = UHS anlegen
uhs-new = Neue UHS
no-uhs = Noch keine UHS, in der Sie dokumentieren dürfen. Die Einsatzleitung legt UHS an und teilt ihr Personal ein.

## Patient list and record
# The workspace of one UHS in the launcher.
ws-uhs = UHS
patients-title = Patienten
all-patients = Alle Patienten
registered-word = registriert.
choose-patient = Patient in der Liste wählen, um die Akte zu öffnen.
unknown-person = Person unbekannt
patient-title = Patient { $tag }
not-found = Diesen Patienten gibt es in diesem Einsatz nicht.
sections = Abschnitte
triage-title = Sichtung
triage-history = Verlauf
retriage-to = Sichtung ändern
# Variables: $user, $time
by-at = { $time }, { $user }
photo-title = Foto
photo-none = Noch kein Foto.
photo-take = Foto aufnehmen
photo-upload = Bild hochladen
photo-busy = Foto wird verkleinert …
photo-preview-loading = Vorschau – das volle Foto wird geladen …
photo-preview-unavailable = Vorschau – das volle Foto ist noch nicht auf dem Server. Es wird später erneut geladen.
photo-purpose = Nur zur Identifikation. Das Foto wird verkleinert, ohne Kamera- und Ortsdaten gespeichert und nur an Berechtigte übertragen.
# Under the photo: "Aufgenommen 14:31 von Anna Schmidt".
photo-taken = Aufgenommen { $at } von { $by }
photo-alt = Foto von Patient { $tag }
photo-remove-short = Entfernen
photo-remove-label = Foto entfernen – Patient { $tag }
new-entry = Neuer Eintrag
correct = Korrigieren
nothing-documented = Noch nichts dokumentiert.
no-answer = keine Angabe
# The free-text answer of an open choice (EPCR-12).
other-answer = Sonstiges
now = Jetzt
empty-entry = Bitte mindestens einen Wert eintragen.
no-changes = Keine Änderungen.
check-input = Bitte die markierten Felder prüfen.
corrected = korrigiert
entry-saved = { $form } für Patient { $tag } gespeichert
new-entry-label = { new-entry }: { $form }
group-patients = Patient
action-register = Patient registrieren
action-create-uhs = UHS anlegen
action-open-record = Akte öffnen
action-take-photo = Foto aufnehmen
action-entry = { $form } eintragen

## Transport priority
transport-priority = Transportpriorität
remove-priority = Keine Transportpriorität
set-priority = { $label } setzen
set-priority-label = { $tag }: { $label } setzen
remove-priority-label = { $tag }: keine Transportpriorität
place = Behandlungsplatz
no-place = Kein Platz

## The plan of a UHS's places (UHS-14)
place-free = frei
place-fill-label = Belegen
place-none-title = Ohne Platz
register-place-hint = Vorschlag: der erste freie Platz der gewählten Sichtungskategorie.
# "Platz 2 belegen": the button of a free place.
place-fill = { $label } belegen

## Capacity per triage category (UHS-13)
uhs-capacities-legend = Kapazität nach Sichtungskategorie (optional)
uhs-capacities-hint = Behandlungsplätze, die für eine Sichtungskategorie freigehalten werden; zusammen höchstens die Kapazität. Die übrigen Plätze sind ohne Kategorie.
# The places per category do not fit in the capacity.
uhs-places-exceed = Die Plätze nach Kategorie ({ $kept }) sind mehr als die Kapazität ({ $total }). Bitte die Kapazität erhöhen oder weniger Plätze freihalten.
place-taken = { $label } (belegt: { $by })
places-free = { $free } von { $total } frei

## Free treatment places per category (UHS-8, UHS-16, ADR-0086)
places-all = Alle UHS
places-any = ohne Kategorie
places-full = voll
places-none = Keine Behandlungsplätze angelegt.
places-legend = Ein Kästchen je Platz: gefüllt belegt, gestrichelt frei. Volle Kategorien sind schraffiert und mit „voll“ benannt.
# "frei von 15", after the large number of free places.
free-of = frei von { $total }
# "2 ohne Platz": patients waiting for a place.
without-place = { $n } ohne Platz
transports-title = Offene Transporte
transport-title = Transport
transport-type = Transportmittel
transport-priority-of-request = Dringlichkeit
transport-needs = Besonderheiten
transport-request = Transport anfordern
transport-choose = Bitte Transportmittel und Dringlichkeit wählen.
transport-none = Kein Transport angefordert.
transport-unit-label = Einsatzmittel
transport-destination = Ziel (z. B. Klinik)
transport-assign = Zuweisen

## Transports waiting for a unit, in the bell and on the overview (TASK-3).
transport-waiting = Transport ohne Einsatzmittel: { $summary } · { $priority }
transports-title-short = Transporte
open-transports = Zu den Transporten
transports-waiting = ohne Einsatzmittel
no-transports-waiting = Keine Transportanforderung wartet.
transports-under-way = { $n } unterwegs
transport-assign-missing = Bitte ein Einsatzmittel wählen; das Ziel kann später folgen.
transport-destination-missing = Bitte ein Ziel wählen.
# A transport sent before its destination was decided (ADR-0098).
destination-open = Ziel offen
destination-set-label = Ziel festlegen
# A destination set before changes on the way, e.g. a hospital stops admitting.
destination-change = Ziel ändern
destination-set = Ziel: { $destination }
transport-no-units = Noch keine Einsatzmittel im Einsatz.
transport-choose-unit = Einsatzmittel wählen
# Accessible name of a step button: the step and which transport.
transport-step-label = { $step }: { $transport }
# Short description of a transport for lists and button names.
transport-summary = { $transport-type } für { $uhs }, { $at }
transport-unit = { $unit } → { $destination }

## Medical orders (ORDER-1…4)
orders = Anordnungen
orders-title = Offene Anordnungen
orders-empty = Keine offenen Anordnungen.
orders-none = Keine Anordnungen.
order-give = Anordnen
order-kind = Art
order-medication = Medikament
order-measure = Maßnahme
order-substance = Wirkstoff
order-dose = Dosis mit Einheit
order-route = Applikationsweg
order-route-choose = Weg wählen
order-measure-text = Maßnahme
order-note = Hinweis (optional)
order-incomplete = Bitte die Anordnung vollständig angeben.
order-records-only = ORDO dokumentiert Anordnungen; Dosis und Indikation prüft die anordnende Person.
# "Angeordnet 10:12 von petra@…"
order-by = { $at } von { $by }
order-performed-by-label = Durchgeführt von
order-performed-by = { order-performed-by-label } { $person }
# "Durchgeführt (durch paul@…)", in the history: who carried it out, apart from who
# reported it.
order-status-performed = { $status } (durch { $person })

## Spoken orders, recorded by the crew and confirmed by their giver (ORDER-6), and who

## ordered a documented medication (ORDER-7).
order-who = Wer ordnet an?
order-mine = Ich ordne an
order-spoken-label = Mündlich, per Funk oder Telefon angeordnet
order-ordered-by = Angeordnet von
order-channel-label = Übermittelt
order-read-back = Wiederholt und bestätigt (Closed Loop)
order-record = Anordnung erfassen
order-spoken-hint = Wer angeordnet hat, bestätigt die Anordnung später in ORDO, sofern er oder sie ORDO nutzt.
order-unconfirmed = Noch nicht bestätigt
order-confirm = Bestätigen
ordered-by-from-order = Wird aus der Anordnung übernommen. Mündliche Anordnungen zuerst unter „Anordnungen“ erfassen.
ordered-by-no-order = ohne Anordnung in ORDO
# "Erfasst 10:12 von Anna Becker"
order-recorded-by = Erfasst { $at } von { $by }
# "Bestätigt 10:20 von Petra Schulz"
order-confirmed = Bestätigt { $at } von { $by }
order-confirm-label = Anordnung bestätigen: { $what }
log-order-confirmed = { $tag }: Mündliche Anordnung bestätigt
# Who ordered a documented medication, taken from its order: "Petra Schulz (Anordnung 10:12)"
ordered-by-order = { $giver } (Anordnung { $at })

## Admission and discharge (UHS-3).
stay-title = Patientenweg
stay-open = Entlassen …
# The stay section on the tab *Übergabe*: admissions and discharges; moving is a handover.
discharge-title = Aufnahme und Entlassung
triage-open = Sichtung ändern
orders-open = Neue Anordnung
discharge-outcome = Entlassen mit Ergebnis
discharge-choose = Bitte das Ergebnis der Entlassung wählen.
readmit = Wieder aufnehmen
# "Entlassene Patienten (2)"
discharged-list = Entlassene Patienten ({ $n })
# "Mit dem Transport: RTW → Klinikum Mitte"
discharge-transport = Mit dem Transport: { $line }

## Hospitals and the transport board (TRANS-2, TRANS-4).
destination = Ziel
destination-choose = Ziel wählen
destination-other-option = Anderes Ziel (Freitext)
destination-other = Anderes Ziel
free-units-title = Freie Transportmittel
free-units-empty = Keine freien Transportmittel.
hospitals-title = Kliniken
hospitals-empty = Noch keine Kliniken erfasst.
# Where the list is kept, on the transport board while it is empty.
hospitals-where = Kliniken werden unter Vorbereitung – Einsatzmittel, Reiter „Kliniken“, erfasst.
hospital-name = Name der Klinik
hospital-departments = Fachabteilungen
hospital-capabilities = Versorgungsangebote
hospital-add = Klinik hinzufügen
hospital-save = Klinik speichern
hospital-edit = Bearbeiten
hospital-name-missing = Bitte den Namen der Klinik angeben.
capacity-legend = Freie Aufnahme (Patienten)
capacity-save = Aufnahme speichern
capacity-none = Aufnahme: nicht gemeldet
capacity-numbers = Bitte die Aufnahme als ganze Zahlen angeben.
sent = Transportiert: { $n }
hospital-edit-label = Bearbeiten: { $name }
hospital-edit-title = { $name } bearbeiten
capacity-save-label = { capacity-save }: { $name }

## Planned transports (TRANS-5).
transport-planned = Geplant für (leer: sofort)
transport-planned-invalid = Bitte Datum und Uhrzeit vollständig angeben.
planned-title = Geplante Transporte
planned-empty = Keine geplanten Transporte.
planned-for = Geplant für { $at }

## Documenting a done order (ORDER-5).
document-order = In Dokumentation übernehmen
document-order-label = { document-order }: { $what }
transports-count = Transporte: { $open } offen, { $planned } geplant

## Patient pathway (PAT-5).
# "Verlegt: Patientenablage Nord → Behandlungsplatz"
moved = Verlegt: { $from } → { $to }
patient-moved = { $tag } verlegt nach { $to }

## Triage references (PAT-11, ADR-0045).
triage-with-label = Sichtung nach
triage-with-none = ohne Angabe
triage-aid = Sichtungshilfe
triage-source = Offizielle Quelle öffnen (neues Fenster)
triage-aid-note = Die Kategorie entscheidet die sichtende Person; ORDO schlägt keine vor und prüft keine.
# "nach PRIOR"
triage-with = nach { $name }

## UHS details (UHS-1).
uhs-details = Ort und Personal
uhs-data = Angaben
saved = Gespeichert.
uhs-name-needed = Die UHS braucht einen Namen.
uhs-location = Ort
uhs-on-map = Auf der Lagekarte
uhs-not-on-map = noch nicht gesetzt
uhs-place-on-map = Auf der Karte setzen
uhs-move-on-map = Auf der Karte versetzen
# The link's visible text, then the unit: the name contains what is seen.
uhs-place-label = { $visible }: { $name }
uhs-staff = Personal
uhs-staff-none = Kein Personal eingetragen.
uhs-resources-title = UHS und Personal
uhs-staff-add = Zum Personal hinzufügen
uhs-save = Speichern
uhs-capacity-number = Bitte die Kapazität als ganze Zahl angeben.

## The record page (EPCR-14, EPCR-15).
record-tabs = Bereiche der Akte
record-overview = Übersicht
record-timeline = Verlauf
record-documentation = Dokumentation
record-documentation-short = Doku
record-handover = Übergabe
record-at = Bei
record-triaged = Gesichtet
record-since = Seit

## The patient list's tabs (chapter 8.6.5, *Tabs*).
patients-tab = Patienten
register-tab = Registrieren
staff-request-tab = Personal anfordern
staff-request-short = Personal
quick-actions = Schnell erfassen
# The floating button that opens the quick actions on phones.
quick-capture = Erfassen
all = Alle
all-kinds = Alles
latest = zuletzt
order = Anordnung
orders-open-title = Offene Anordnungen
orders-all = Alle Anordnungen in der Dokumentation
orders-none-open = Keine offenen Anordnungen.

## Handover (ADR-0087).
holders-uhs = UHS
holders-units = Einsatzmittel
handover-to = Übergabe an { $to }
handover-quick-step = Schnellübergabe per QR-Code
handover-taken-by = Übernommen: { $by }
handover-taken-back = Übergabe zurückgenommen: { $by }
handed-outside-step = Übergeben an { $receiver } (außerhalb von ORDO)
handover-open-to = Übergabe an { $to } offen
handover-open-quick = Schnellübergabe offen
# "Noch bei SanTrp 1 · übergeben 14:31 von Max Muster".
handover-open-since = Noch bei { $holder } · übergeben { $at } von { $by }
handover-open-hint = Bis zur Übernahme dokumentiert die bisherige Stelle weiter; die übernehmende Stelle bestätigt mit „Übernehmen“.
handover-open-quick-hint = Die übernehmende Stelle scannt den QR-Code mit ORDO oder der Kamera ihres Geräts und wählt, wohin der Patient kommt.
handover-note-short = Notiz:
handover-take = Übernehmen für { $to }
handover-take-word = Übernehmen
# The take button's accessible name: its words and the patient.
handover-take-label = { $words }: Patient { $tag }
handover-take-to = Übernehmen für
take-place-hint = Vorschlag: der erste freie Platz der Sichtungskategorie des Patienten.
handover-take-back = Übergabe zurücknehmen
handover-failed = Die Übergabe konnte nicht gespeichert werden.
handover-way = Weg des Patienten

## The handover document (ADR-0089).
document-title = Übergabedokument
document-hint = PDF für die übernehmende Stelle. Alle Daten liegen ihm zusätzlich maschinenlesbar bei, als HL7-FHIR-Bundle (Anhang der PDF-Datei, in PDF-Programmen unter „Anhänge“). Der ORDO-Server erstellt es; dafür braucht das Gerät eine Verbindung.
document-button = PDF herunterladen
document-label = PDF herunterladen: Übergabedokument Patient { $tag }
# The file name of the saved document.
document-file = Uebergabe-{ $tag }.pdf
document-asking = Wird auf dem Server erstellt …
document-saved = Heruntergeladen.
document-offline = Keine Verbindung zum Server. Das Dokument wird dort erstellt; bitte mit Verbindung erneut versuchen.
document-refused = Sie dürfen das Übergabedokument dieses Patienten nicht abrufen.
document-failed = Das Dokument konnte nicht erstellt werden. Bitte erneut versuchen.

## The way as a flow chart (dsn patient-flow).
flow-registered = Erfasst { $time }
flow-taken = Übernommen { $time }
flow-taken-back = Übergabe zurückgenommen { $time }
# "Transport: RTW 2 → Klinikum Mitte · Angefordert"
flow-transport = Transport: { $line } · { $status }
flow-handover = Übergabe
flow-handover-open = Übergabe offen
flow-moved = Verlegt
flow-outside = Außerhalb von ORDO
flow-quick-receiver = Wer den QR-Code scannt
flow-not-taken = Noch nicht übernommen
flow-next-edge = Nächste Übergabe
handover-next = an wen übergeben
handover-title = Patient übergeben
handover-to-label = Übergeben an
handover-to-uhs = UHS oder Behandlungsplatz
handover-to-uhs-hint = Im Einsatz. Die Akte geht mit; das Personal der UHS dokumentiert weiter.
handover-to-unit = Einsatzmittel
handover-to-unit-hint = Im Einsatz, z. B. ein RTW. Seine Besatzung dokumentiert weiter.
handover-to-transport = Transport
handover-to-transport-hint = Das Einsatzmittel eines angeforderten Transports übernimmt; das Ziel folgt aus dem Transport.
handover-to-outside = Außerhalb von ORDO
handover-to-outside-hint = Z. B. Klinik, anderer Rettungsdienst, Polizei. Der Patient verlässt den Einsatz.
handover-target-label = Welche Stelle: { $kind }
handover-receiver = An wen
handover-receiver-hint = Z. B. „Klinikum Mitte, Notaufnahme“ oder „RTW Florian Kreis 3/83-1“.
handover-receiver-missing = Bitte angeben, an wen der Patient übergeben wird.
handover-target-missing = Bitte wählen, an wen der Patient übergeben wird.
handover-note = Übergabe-Notiz (optional)
handover-note-example = z. B. Verlauf auf dem Weg
handover-receiver-any = die übernehmende Stelle
handover-gets = Was { $to } bekommt
handover-gets-record = die ganze Akte mit Verlauf, Fotos und Anordnungen
handover-gets-note = wer übergeben hat, wann, und die Übergabe-Notiz
handover-gets-outside = die Übergabe mit Empfänger, Zeit und Notiz im Verlauf; ein Übergabedokument (PDF) folgt
handover-after = Danach
handover-after-inside = Bis zur Übernahme dokumentieren Sie weiter, und beide Seiten sehen „Übergabe offen“. Nach der Übernahme lesen Sie die Akte weiter, schreiben aber nicht mehr.
handover-after-outside = Der Patient verlässt den Einsatz sofort und zählt als entlassen. Kommt er zurück, „Wieder aufnehmen“.
handover-button-label = Übergeben
handover-button = An { $to } übergeben
quick-handover = Schnellübergabe
quick-handover-hint = Wenn noch offen ist, wer übernimmt: ORDO zeigt einen QR-Code. Wer ihn scannt, wählt, wohin der Patient kommt. Der Code enthält keine medizinischen Daten.
quick-handover-start = QR-Code zeigen
quick-code-alt = QR-Code der Schnellübergabe
quick-code-hint = Mit ORDO („Übergabe scannen“) oder der Kamera des übernehmenden Geräts scannen.
scan-handover = Übergabe scannen
scan-handover-hint = Foto vom QR-Code einer Schnellübergabe.
scan-reading = Der QR-Code wird gelesen …
scan-no-handover = Auf dem Foto ist kein QR-Code einer Übergabe zu erkennen.
take-title = Patient { $tag } übernehmen
take-from = Bisher bei { $holder }. Wählen Sie, wo der Patient bei Ihnen ist.
take-not-here = Die Akte ist noch nicht auf diesem Gerät. Sobald es verbunden ist, kommt sie an; dieses Fenster bleibt offen.
take-not-arrived = Die Übergabe selbst ist noch nicht auf diesem Gerät. Die Übernahme gilt, sobald beide Geräte wieder verbunden sind.
take-gone = Der Patient ist nicht mehr im Einsatz (entlassen oder außerhalb übergeben).
take-taken = Diese Übergabe ist schon übernommen.
take-nowhere = Sie stehen auf keiner Personalliste einer UHS und in keiner Besatzung. Die Einsatzleitung teilt Sie ein.
incoming-title = Übergaben an Sie
incoming-line = von { $from } an { $to }
record-open = Akte
# Variables: $tag, $to
handover-bell = Übergabe an { $to }: Patient { $tag }
readmitted-step = Wieder aufgenommen
cleared = gelöscht
photo-added-word = Foto hinzugefügt
timeline-filter = Verlauf filtern
way-registered = Erfasst · { $uhs }
registered-as = Erfasst: { $category }
entries-all = Alle { $n }
entries-all-label = { $form }: alle { $n } Einträge im Verlauf
corrects-entry = korrigiert den Eintrag von { $time }
uhs-edit-title = { $name } bearbeiten
uhs-close-title = UHS auflösen
uhs-close = UHS auflösen …
uhs-close-confirm = Ja, auflösen
uhs-close-hint = Die UHS verschwindet aus Listen, Karte und Zahlen. Das Einsatztagebuch behält sie.
uhs-close-question = { $name } wirklich auflösen?
uhs-details-label = { uhs-details } von { $name }
staff-remove-label = { $name } aus dem Personal entfernen
uhs-renamed = UHS umbenannt in { $name }

## A UHS asks for staff (UHS-15).
staff-requests-title = Personal anfordern
staff-request-choose-uhs = Wählen Sie oben unter „Meine UHS“ Ihre UHS, um für sie anzufordern.
staff-request-for = Für { $uhs }: die Einsatzleitung sieht die Anforderung und schickt jemanden.
staff-request-open = offen
staff-request-withdrawn = zurückgenommen
staff-request-closed = Anforderung abgeschlossen
staff-board-title = Anforderungen der UHS
staff-send = Wer kommt?
staff-assign = Zuweisen
staff-withdraw = Zurücknehmen
staff-request-met = { $person } kommt
staff-withdraw-label = { staff-withdraw }: { $request }
case-cancel = Ohne Auftrag aufnehmen
register-from-case = Aus dem Auftrag „{ $place }“ für { $unit }
staff-request-not-staff = Anfordern kann nur, wer auf der Personalliste von { $uhs } steht.
all-uhs = Alle UHS
read-only-uhs = { $name } (nur lesen)
# Above a record the person may only read (UHS-10).
record-read-only = Nur lesen: Sie stehen nicht auf der Personalliste von { $uhs }. Die Einsatzleitung teilt Personal zu.

## Converted by hand: counts, forms and values of ORDO's types
working-uhs = Meine UHS
# Spoken description of a triage category, so screen readers convey its meaning.
triage-description-immediate = Sichtungskategorie I, rot, sofortige Behandlung
triage-description-urgent = Sichtungskategorie II, gelb, dringlich
triage-description-minor = Sichtungskategorie III, grün, leicht verletzt
triage-description-expectant = Sichtungskategorie IV, blau, abwartende Behandlung
triage-description-deceased = Verstorben
triage-description-uninjured = Betroffene Person, unverletzt, ohne Behandlungsbedarf
triage-count = { $n ->
    [one] 1 Sichtung
   *[other] { $n } Sichtungen
}
# Variables: $tag, $category
retriage-of = { col-retriage } { $tag }: { $category }
corrected-entries = { $n ->
    [one] 1 korrigierter Eintrag
   *[other] { $n } korrigierte Einträge
}
# "Vitalwerte bearbeiten": the button's name with its form.
edit-form = { $form } bearbeiten
value-not-a-number = Bitte eine Zahl eingeben.
value-between = Bitte einen Wert von { $min } bis { $max } eingeben.
value-at-least = Bitte mindestens { $min } eingeben.
value-at-most = Bitte höchstens { $max } eingeben.
value-impossible = Wert nicht möglich.
value-not-a-date = Bitte ein Datum eingeben.
value-not-a-time = Bitte eine Uhrzeit (hh:mm) eingeben.
value-too-long = Der Text ist zu lang.
action-retriage-to = Sichtung ändern: { $category }
# "7 von 10 belegt · 3 frei", or "Patienten" without a capacity.
uhs-load-of = von { $capacity } belegt · { $free } frei
uhs-load-patients = { $n ->
    [one] Patient
   *[other] Patienten
}
patient-count = { $n ->
    [one] 1 Patient
   *[other] { $n } Patienten
}
# How a UHS's capacity is split, as typed. Variables: $kept, $total
uhs-places-over = Zurzeit { $kept } Plätze nach Kategorie: mehr als die Kapazität von { $total }.
# Variables: $kept, $rest
uhs-places-within = Zurzeit { $kept } nach Kategorie, { $rest } ohne Kategorie.
# A category's strip of places, e.g. "SK II: 4 von 4 belegt, voll".
place-strip-full = { $category }: { $taken } von { $total } belegt, voll
# Variables: $category, $taken, $total, $free
place-strip-free = { $category }: { $taken } von { $total } belegt, { $free } frei
transport-step-requested = Angefordert
transport-step-assign = Zuweisen
transport-step-loading = Verladen
transport-step-departed = Abgefahren
transport-step-arrived = Am Ziel
transport-step-handed-over = Patient am Ziel übergeben
transport-step-cancel = Stornieren
order-step-give = Anordnen
order-step-acknowledge = Übernehmen
order-step-done = Durchgeführt
order-step-withdraw = Zurücknehmen
order-channel-in-person = persönlich
order-channel-radio = per Funk
order-channel-phone = per Telefon
order-channel-telemedicine = per Telemedizin
# "Mündlich angeordnet von Petra Schulz, per Funk, wiederholt und bestätigt"
order-spoken-read-back = Mündlich angeordnet von { $giver }, { $channel }, wiederholt und bestätigt
order-spoken-not-read-back = Mündlich angeordnet von { $giver }, { $channel }, nicht wiederholt
# "DE-0042 wieder aufnehmen": the button's name with its patient.
readmit-of = { $tag } wieder aufnehmen
# "Aufnahme: SK I 2 · SK II 4 · SK III – (Stand 10:12, ben@…)"
capacity-of = Aufnahme: { $parts } (Stand { $at }, { $by })
# "Offene Anordnungen: 3 (1 nicht bestätigt)"
open-orders-count = Offene Anordnungen: { $n }
# Variables: $n, $unconfirmed
open-orders-unconfirmed = Offene Anordnungen: { $n } ({ $unconfirmed } nicht bestätigt)
triage-stage-pretriage = Vorsichtung
triage-stage-triage = Sichtung
uhs-staff-count = { $n ->
    [0] { uhs-staff-none }
    [one] Personal: 1 Person
   *[other] Personal: { $n } Personen
}
handover-no-transport = Kein Transport mit zugewiesenem Einsatzmittel. Transport im Abschnitt „Transport“ anfordern.
handover-no-other = Keine andere Stelle im Einsatz.
uhs-close-blocked = { $patients ->
    [one] 1 Patient ist noch hier. Erst übergeben oder entlassen, dann auflösen.
   *[other] { $patients } Patienten sind noch hier. Erst übergeben oder entlassen, dann auflösen.
}
staff-need-physician = Arzt/Ärztin
staff-need-paramedic = Notfallsanitäter/in
staff-request-of = { $need } anfordern
# Variables: $uhs, $need
staff-requested-of = { $uhs } fordert { $need } an
# The Auftrag of the person sent after a staff request (UHS-15). Variables: $uhs, $need
staff-task-of = Als { $need } zu { $uhs }
# "Patientenzahlen UHS Eingang: 5 (SK I 1 · SK III 4), 1 ohne Platz, 2 entlassen"
log-counts-total = Patientenzahlen { $uhs }: { $total }
log-counts-discharged = { $n } entlassen
log-places = { $uhs }: { $n } Behandlungsplätze
log-transport = { $kind } ab { $uhs }
log-transport-requested = Transport angefordert: { $transport }
# Variables: $transport, $status, $unit
log-transport-updated-by = Transport { $transport }: { $status } ({ $unit })
log-transport-updated-to = Transport { $transport }: { $status }
log-order-updated = { $tag }: Anordnung { $status }

## Führung – Lage (ADR-0099): more of each UHS, without patient data.
# Variables: $uhs, $destination
transport-of-unit = Transport von { $uhs } → { $destination }
staff-requested-tag = Personal angefordert: { $need }
waiting-transport-n = { $n ->
    [one] 1 wartet auf Transport
   *[other] { $n } warten auf Transport
}
waiting-no-unit = { $n } ohne Einsatzmittel
waiting-transport-total = Warten auf Transport: { $n }
# Variables: $all, $here, $discharged
registered-here-discharged = Registriert { $all } · hier jetzt { $here } · entlassen { $discharged }
staff-go-send = Personal schicken
staff-go-send-label = { staff-go-send }: { $uhs }
# The triage aid on the profile page (PAT-11, ADR-0100).
profile-triage = Sichtung auf diesem Gerät
# The "+" button that opens the form to add one (ADR-0102).
uhs-new-word = UHS
# The "⋯" menu of a transport's further steps (ADR-0102).
transport-more-label = Weitere Schritte: { $transport }
