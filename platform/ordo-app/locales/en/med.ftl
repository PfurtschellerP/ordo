crew-places = Free treatment places
crew-not-held = This patient is not (or no longer) with your unit. Whoever cares for the patient now sees the record.
# The UHS as a unit (UHS-12).
uhs-strength = Strength of the care unit
uhs-strength-hint = The care unit counts as a unit among the forces on scene: with this strength, else with its staff.
register-title = Register patient
register-tag = Number of the triage tag
# A card number already registered (PAT-12): where the patient is, and since when.
tag-in-use = The tag { $tag } is already registered: { $holder }, since { $time }.
# A card number registered again (PAT-12): by whom, where and when.
# Variables: $person, $holder, $time
tag-registered-again = Registered again { $time } by { $person } ({ $holder }).
tag-twice = Tag registered twice
tag-twice-hint = Two devices registered this tag number while they were offline, or a number was mistyped. Whoever registered later gives their patient a new tag and enters its number; what this device documented since goes with it.
retag-ask = Your registration came later. Please give your patient a new triage tag.
retag-number = Number of the new tag
retag-submit = Change number
retag-in-use = The tag { $tag } is already registered. Please take an unused tag.
register-category = Triage category
register-uhs = Patient at
register-submit = Register
board-empty = No patients registered yet.
col-retriage = Re-triage
not-sent = not sent yet
sent-label = sent
uhs-empty = No care unit set up yet. The incident command sets them up under Preparation – Units.
uhs-name = Name of the care unit
uhs-capacity = Capacity (patients)
uhs-submit = Set up care unit
uhs-new = New care unit
no-uhs = No care unit yet where you may document. The incident command sets up care units and assigns their staff.

## Patient list and record
# The workspace of one UHS in the launcher.
ws-uhs = Care unit
patients-title = Patients
all-patients = All patients
registered-word = registered.
choose-patient = Choose a patient in the list to open the record.
unknown-person = Person unknown
patient-title = Patient { $tag }
not-found = This patient does not exist in this incident.
sections = Sections
triage-title = Triage
triage-history = History
retriage-to = Change triage
# Variables: $user, $time
by-at = { $time }, { $user }
photo-title = Photo
photo-none = No photo yet.
photo-take = Take photo
photo-upload = Upload picture
photo-busy = Making the photo smaller …
photo-preview-loading = Preview – loading the full photo …
photo-preview-unavailable = Preview – the full photo is not on the server yet. It is loaded again later.
photo-purpose = For identification only. The photo is made smaller, stored without camera and location data and sent only to those allowed to see it.
# Under the photo: "Taken 14:31 by Anna Schmidt".
photo-taken = Taken { $at } by { $by }
photo-alt = Photo of patient { $tag }
photo-remove-short = Remove
photo-remove-label = Remove photo – patient { $tag }
new-entry = New entry
correct = Correct
nothing-documented = Nothing documented yet.
no-answer = not given
# The free-text answer of an open choice (EPCR-12).
other-answer = Other
now = Now
empty-entry = Please enter at least one value.
no-changes = No changes.
check-input = Please check the marked fields.
corrected = corrected
entry-saved = { $form } saved for patient { $tag }
new-entry-label = { new-entry }: { $form }
group-patients = Patient
action-register = Register patient
action-create-uhs = Set up care unit
action-open-record = Open record
action-take-photo = Take photo
action-entry = Enter { $form }

## Transport priority
transport-priority = Transport priority
remove-priority = No transport priority
set-priority = Set { $label }
set-priority-label = { $tag }: set { $label }
remove-priority-label = { $tag }: no transport priority
place = Treatment place
no-place = No place

## The plan of a UHS's places (UHS-14)
place-free = free
place-fill-label = Fill
place-none-title = Without a place
register-place-hint = Suggestion: the first free place of the chosen triage category.
# "Fill place 2": the button of a free place.
place-fill = Fill { $label }

## Capacity per triage category (UHS-13)
uhs-capacities-legend = Capacity by triage category (optional)
uhs-capacities-hint = Treatment places kept free for a triage category; together at most the capacity. The other places have no category.
# The places per category do not fit in the capacity.
uhs-places-exceed = The places by category ({ $kept }) are more than the capacity ({ $total }). Please raise the capacity or keep fewer places free.
place-taken = { $label } (taken: { $by })
places-free = { $free } of { $total } free

## Free treatment places per category (UHS-8, UHS-16, ADR-0086)
places-all = All care units
places-any = no category
places-full = full
places-none = No treatment places set up.
places-legend = One box per place: filled is taken, dashed is free. Full categories are hatched and labelled “full”.
# "free of 15", after the large number of free places.
free-of = free of { $total }
# "2 without a place": patients waiting for a place.
without-place = { $n } without a place
transports-title = Open transports
transport-title = Transport
transport-type = Transport type
transport-priority-of-request = Urgency
transport-needs = Special needs
transport-request = Request transport
transport-choose = Please choose a transport type and an urgency.
transport-none = No transport requested.
transport-unit-label = Unit
transport-destination = Destination (e.g. hospital)
transport-assign = Assign

## Transports waiting for a unit, in the bell and on the overview (TASK-3).
transport-waiting = Transport without a unit: { $summary } · { $priority }
transports-title-short = Transports
open-transports = To the transports
transports-waiting = without a unit
no-transports-waiting = No transport request is waiting.
transports-under-way = { $n } under way
transport-assign-missing = Please choose a unit; the destination can follow later.
transport-destination-missing = Please choose a destination.
# A transport sent before its destination was decided (ADR-0098).
destination-open = Destination open
destination-set-label = Set destination
# A destination set before changes on the way, e.g. a hospital stops admitting.
destination-change = Change destination
destination-set = Destination: { $destination }
transport-no-units = No units in the incident yet.
transport-choose-unit = Choose unit
# Accessible name of a step button: the step and which transport.
transport-step-label = { $step }: { $transport }
# Short description of a transport for lists and button names.
transport-summary = { $transport-type } for { $uhs }, { $at }
transport-unit = { $unit } → { $destination }

## Medical orders (ORDER-1…4)
orders = Orders
orders-title = Open orders
orders-empty = No open orders.
orders-none = No orders.
order-give = Give order
order-kind = Kind
order-medication = Medication
order-measure = Measure
order-substance = Active substance
order-dose = Dose with unit
order-route = Route
order-route-choose = Choose route
order-measure-text = Measure
order-note = Note (optional)
order-incomplete = Please give the order in full.
order-records-only = ORDO documents orders; the person giving the order checks dose and indication.
# "Given 10:12 by petra@…"
order-by = { $at } by { $by }
order-performed-by-label = Carried out by
order-performed-by = { order-performed-by-label } { $person }
# "Done (by paul@…)", in the history: who carried it out, apart from who
# reported it.
order-status-performed = { $status } (by { $person })

## Spoken orders, recorded by the crew and confirmed by their giver (ORDER-6), and who

## ordered a documented medication (ORDER-7).
order-who = Who gives the order?
order-mine = I give the order
order-spoken-label = Ordered verbally, by radio or by phone
order-ordered-by = Ordered by
order-channel-label = Passed on
order-read-back = Read back and confirmed (closed loop)
order-record = Record order
order-spoken-hint = Whoever gave the order confirms it later in ORDO, if they use ORDO.
order-unconfirmed = Not confirmed yet
order-confirm = Confirm
ordered-by-from-order = Taken from the order. Record verbal orders first under “Orders”.
ordered-by-no-order = without an order in ORDO
# "Recorded 10:12 by Anna Becker"
order-recorded-by = Recorded { $at } by { $by }
# "Confirmed 10:20 by Petra Schulz"
order-confirmed = Confirmed { $at } by { $by }
order-confirm-label = Confirm order: { $what }
log-order-confirmed = { $tag }: verbal order confirmed
# Who ordered a documented medication, taken from its order: "Petra Schulz (order 10:12)"
ordered-by-order = { $giver } (order { $at })

## Admission and discharge (UHS-3).
stay-title = Patient pathway
stay-open = Discharge …
# The stay section on the tab *Übergabe*: admissions and discharges; moving is a handover.
discharge-title = Admission and discharge
triage-open = Change triage
orders-open = New order
discharge-outcome = Discharge with outcome
discharge-choose = Please choose the outcome of the discharge.
readmit = Readmit
# "Discharged patients (2)"
discharged-list = Discharged patients ({ $n })
# "With the transport: RTW → City hospital"
discharge-transport = With the transport: { $line }

## Hospitals and the transport board (TRANS-2, TRANS-4).
destination = Destination
destination-choose = Choose destination
destination-other-option = Other destination (free text)
destination-other = Other destination
free-units-title = Free transport units
free-units-empty = No free transport units.
hospitals-title = Hospitals
hospitals-empty = No hospitals recorded yet.
# Where the list is kept, on the transport board while it is empty.
hospitals-where = Hospitals are recorded under Preparation – Units, tab “Hospitals”.
hospital-name = Name of the hospital
hospital-departments = Departments
hospital-capabilities = Services offered
hospital-add = Add hospital
hospital-save = Save hospital
hospital-edit = Edit
hospital-name-missing = Please give the name of the hospital.
capacity-legend = Free admission (patients)
capacity-save = Save admission
capacity-none = Admission: not reported
capacity-numbers = Please give the admission as whole numbers.
sent = Transported: { $n }
hospital-edit-label = Edit: { $name }
hospital-edit-title = Edit { $name }
capacity-save-label = { capacity-save }: { $name }

## Planned transports (TRANS-5).
transport-planned = Planned for (empty: at once)
transport-planned-invalid = Please give the date and the time in full.
planned-title = Planned transports
planned-empty = No planned transports.
planned-for = Planned for { $at }

## Documenting a done order (ORDER-5).
document-order = Take into documentation
document-order-label = { document-order }: { $what }
transports-count = Transports: { $open } open, { $planned } planned

## Patient pathway (PAT-5).
# "Moved: Patient collection point north → Treatment area"
moved = Moved: { $from } → { $to }
patient-moved = { $tag } moved to { $to }

## Triage references (PAT-11, ADR-0045).
triage-with-label = Triage by
triage-with-none = not given
triage-aid = Triage aid
triage-source = Open the official source (new window)
triage-aid-note = The person triaging decides the category; ORDO suggests none and checks none.
# "by PRIOR"
triage-with = by { $name }

## UHS details (UHS-1).
uhs-details = Location and staff
uhs-data = Details
saved = Saved.
uhs-name-needed = The care unit needs a name.
uhs-location = Location
uhs-on-map = On the situation map
uhs-not-on-map = not set yet
uhs-place-on-map = Set on the map
uhs-move-on-map = Move on the map
# The link's visible text, then the unit: the name contains what is seen.
uhs-place-label = { $visible }: { $name }
uhs-staff = Staff
uhs-staff-none = No staff entered.
uhs-resources-title = Care units and staff
uhs-staff-add = Add to staff
uhs-save = Save
uhs-capacity-number = Please give the capacity as a whole number.

## The record page (EPCR-14, EPCR-15).
record-tabs = Areas of the record
record-overview = Overview
record-timeline = History
record-documentation = Documentation
record-documentation-short = Docs
record-handover = Handover
record-at = At
record-triaged = Triaged
record-since = Since

## The patient list's tabs (chapter 8.6.5, *Tabs*).
patients-tab = Patients
register-tab = Register
staff-request-tab = Request staff
staff-request-short = Staff
quick-actions = Quick entry
# The floating button that opens the quick actions on phones.
quick-capture = Enter
all = All
all-kinds = Everything
latest = latest
order = Order
orders-open-title = Open orders
orders-all = All orders in the documentation
orders-none-open = No open orders.

## Handover (ADR-0087).
holders-uhs = Care units
holders-units = Units
handover-to = Handover to { $to }
handover-quick-step = Quick handover by QR code
handover-taken-by = Taken over: { $by }
handover-taken-back = Handover taken back: { $by }
handed-outside-step = Handed over to { $receiver } (outside ORDO)
handover-open-to = Handover to { $to } pending
handover-open-quick = Quick handover pending
# "Still with SanTrp 1 · handed over 14:31 by Max Muster".
handover-open-since = Still with { $holder } · handed over { $at } by { $by }
handover-open-hint = Until the takeover, the previous post keeps documenting; the taking post confirms with “Take over”.
handover-open-quick-hint = The taking post scans the QR code with ORDO or its device's camera and chooses where the patient goes.
handover-note-short = Note:
handover-take = Take over for { $to }
handover-take-word = Take over
# The take button's accessible name: its words and the patient.
handover-take-label = { $words }: patient { $tag }
handover-take-to = Take over for
take-place-hint = Suggestion: the first free place of the patient's triage category.
handover-take-back = Take handover back
handover-failed = The handover could not be saved.
handover-way = The patient's way

## The handover document (ADR-0089).
document-title = Handover document
document-hint = PDF for the taking post. All data is also attached to it in machine-readable form, as an HL7 FHIR bundle (an attachment of the PDF file, under “Attachments” in PDF programs). The ORDO server makes it; for that the device needs a connection.
document-button = Download PDF
document-label = Download PDF: handover document patient { $tag }
# The file name of the saved document.
document-file = Handover-{ $tag }.pdf
document-asking = Being made on the server …
document-saved = Downloaded.
document-offline = No connection to the server. The document is made there; please try again with a connection.
document-refused = You may not fetch this patient's handover document.
document-failed = The document could not be made. Please try again.

## The way as a flow chart (dsn patient-flow).
flow-registered = Recorded { $time }
flow-taken = Taken over { $time }
flow-taken-back = Handover taken back { $time }
# "Transport: RTW 2 → City hospital · Requested"
flow-transport = Transport: { $line } · { $status }
flow-handover = Handover
flow-handover-open = Handover pending
flow-moved = Moved
flow-outside = Outside ORDO
flow-quick-receiver = Whoever scans the QR code
flow-not-taken = Not taken over yet
flow-next-edge = Next handover
handover-next = whom to hand over to
handover-title = Hand over patient
handover-to-label = Hand over to
handover-to-uhs = Care unit or treatment place
handover-to-uhs-hint = In the incident. The record goes with the patient; the care unit's staff keeps documenting.
handover-to-unit = Unit
handover-to-unit-hint = In the incident, e.g. an ambulance. Its crew keeps documenting.
handover-to-transport = Transport
handover-to-transport-hint = The unit of a requested transport takes over; the destination follows from the transport.
handover-to-outside = Outside ORDO
handover-to-outside-hint = E.g. a hospital, another ambulance service, the police. The patient leaves the incident.
handover-target-label = Which post: { $kind }
handover-receiver = To whom
handover-receiver-hint = E.g. “City hospital, emergency department” or “Ambulance Florian Kreis 3/83-1”.
handover-receiver-missing = Please give to whom the patient is handed over.
handover-target-missing = Please choose to whom the patient is handed over.
handover-note = Handover note (optional)
handover-note-example = e.g. what happened on the way
handover-receiver-any = the taking post
handover-gets = What { $to } gets
handover-gets-record = the whole record with history, photos and orders
handover-gets-note = who handed over, when, and the handover note
handover-gets-outside = the handover with receiver, time and note in the history; a handover document (PDF) follows
handover-after = Afterwards
handover-after-inside = Until the takeover you keep documenting, and both sides see “Handover pending”. After the takeover you can still read the record, but no longer write.
handover-after-outside = The patient leaves the incident at once and counts as discharged. If the patient comes back, “Readmit”.
handover-button-label = Hand over
handover-button = Hand over to { $to }
quick-handover = Quick handover
quick-handover-hint = When it is still open who takes over: ORDO shows a QR code. Whoever scans it chooses where the patient goes. The code holds no medical data.
quick-handover-start = Show QR code
quick-code-alt = QR code of the quick handover
quick-code-hint = Scan with ORDO (“Scan handover”) or the camera of the taking device.
scan-handover = Scan handover
scan-handover-hint = Photo of a quick handover's QR code.
scan-reading = Reading the QR code …
scan-no-handover = No handover QR code can be seen in the photo.
take-title = Take over patient { $tag }
take-from = So far with { $holder }. Choose where the patient is with you.
take-not-here = The record is not on this device yet. It arrives as soon as the device is connected; this window stays open.
take-not-arrived = The handover itself is not on this device yet. The takeover counts as soon as both devices are connected again.
take-gone = The patient is no longer in the incident (discharged or handed over outside).
take-taken = This handover has already been taken over.
take-nowhere = You are on no care unit's staff list and on no crew. The incident command assigns you.
incoming-title = Handovers to you
incoming-line = from { $from } to { $to }
record-open = Record
# Variables: $tag, $to
handover-bell = Handover to { $to }: patient { $tag }
readmitted-step = Readmitted
cleared = cleared
photo-added-word = Photo added
timeline-filter = Filter history
way-registered = Recorded · { $uhs }
registered-as = Recorded: { $category }
entries-all = All { $n }
entries-all-label = { $form }: all { $n } entries in the history
corrects-entry = corrects the entry of { $time }
uhs-edit-title = Edit { $name }
uhs-close-title = Close care unit
uhs-close = Close care unit …
uhs-close-confirm = Yes, close
uhs-close-hint = The care unit disappears from lists, map and counts. The incident log keeps it.
uhs-close-question = Really close { $name }?
uhs-details-label = { uhs-details } of { $name }
staff-remove-label = Remove { $name } from the staff
uhs-renamed = Care unit renamed to { $name }

## A UHS asks for staff (UHS-15).
staff-requests-title = Request staff
staff-request-choose-uhs = Choose your care unit above under “My care unit” to request for it.
staff-request-for = For { $uhs }: the incident command sees the request and sends someone.
staff-request-open = open
staff-request-withdrawn = withdrawn
staff-request-closed = Request closed
staff-board-title = Requests of the care units
staff-send = Who is coming?
staff-assign = Assign
staff-withdraw = Withdraw
staff-request-met = { $person } is coming
staff-withdraw-label = { staff-withdraw }: { $request }
case-cancel = Admit without a task
register-from-case = From the task “{ $place }” for { $unit }
staff-request-not-staff = Only someone on the staff list of { $uhs } can request.
all-uhs = All care units
read-only-uhs = { $name } (read only)
# Above a record the person may only read (UHS-10).
record-read-only = Read only: you are not on the staff list of { $uhs }. The incident command assigns staff.

## Converted by hand: counts, forms and values of ORDO's types
working-uhs = My care unit
# Spoken description of a triage category, so screen readers convey its meaning.
triage-description-immediate = Triage category I, red, immediate treatment
triage-description-urgent = Triage category II, yellow, urgent
triage-description-minor = Triage category III, green, minor injuries
triage-description-expectant = Triage category IV, blue, expectant treatment
triage-description-deceased = Deceased
triage-description-uninjured = Person affected, unharmed, no need for treatment
triage-count = { $n ->
    [one] 1 triage
   *[other] { $n } triages
}
# Variables: $tag, $category
retriage-of = { col-retriage } { $tag }: { $category }
corrected-entries = { $n ->
    [one] 1 corrected entry
   *[other] { $n } corrected entries
}
# "Edit Vitalwerte": the button's name with its form.
edit-form = Edit { $form }
value-not-a-number = Please enter a number.
value-between = Please enter a value from { $min } to { $max }.
value-at-least = Please enter at least { $min }.
value-at-most = Please enter at most { $max }.
value-impossible = Value not possible.
value-not-a-date = Please enter a date.
value-not-a-time = Please enter a time (hh:mm).
value-too-long = The text is too long.
action-retriage-to = Change triage: { $category }
# "7 of 10 taken · 3 free", or "Patients" without a capacity.
uhs-load-of = of { $capacity } taken · { $free } free
uhs-load-patients = { $n ->
    [one] Patient
   *[other] Patients
}
patient-count = { $n ->
    [one] 1 patient
   *[other] { $n } patients
}
# How a UHS's capacity is split, as typed. Variables: $kept, $total
uhs-places-over = At present { $kept } places by category: more than the capacity of { $total }.
# Variables: $kept, $rest
uhs-places-within = At present { $kept } by category, { $rest } without a category.
# A category's strip of places, e.g. "SK II: 4 of 4 taken, full".
place-strip-full = { $category }: { $taken } of { $total } taken, full
# Variables: $category, $taken, $total, $free
place-strip-free = { $category }: { $taken } of { $total } taken, { $free } free
transport-step-requested = Requested
transport-step-assign = Assign
transport-step-loading = Load
transport-step-departed = Departed
transport-step-arrived = At destination
transport-step-handed-over = Patient handed over at destination
transport-step-cancel = Cancel transport
order-step-give = Give order
order-step-acknowledge = Acknowledge
order-step-done = Done
order-step-withdraw = Withdraw
order-channel-in-person = in person
order-channel-radio = by radio
order-channel-phone = by phone
order-channel-telemedicine = by telemedicine
# "Ordered verbally by Petra Schulz, by radio, read back and confirmed"
order-spoken-read-back = Ordered verbally by { $giver }, { $channel }, read back and confirmed
order-spoken-not-read-back = Ordered verbally by { $giver }, { $channel }, not read back
# "Readmit DE-0042": the button's name with its patient.
readmit-of = Readmit { $tag }
# "Admission: SK I 2 · SK II 4 · SK III – (as of 10:12, ben@…)"
capacity-of = Admission: { $parts } (as of { $at }, { $by })
# "Open orders: 3 (1 not confirmed)"
open-orders-count = Open orders: { $n }
# Variables: $n, $unconfirmed
open-orders-unconfirmed = Open orders: { $n } ({ $unconfirmed } not confirmed)
triage-stage-pretriage = Pre-triage
triage-stage-triage = Triage
uhs-staff-count = { $n ->
    [0] { uhs-staff-none }
    [one] Staff: 1 person
   *[other] Staff: { $n } people
}
handover-no-transport = No transport with an assigned unit. Request a transport in the section “Transport”.
handover-no-other = No other post in the incident.
uhs-close-blocked = { $patients ->
    [one] 1 patient is still here. Hand over or discharge first, then close.
   *[other] { $patients } patients are still here. Hand over or discharge first, then close.
}
staff-need-physician = Physician
staff-need-paramedic = Paramedic
staff-request-of = Request { $need }
# Variables: $uhs, $need
staff-requested-of = { $uhs } requests { $need }
# The Auftrag of the person sent after a staff request (UHS-15). Variables: $uhs, $need
staff-task-of = As { $need } to { $uhs }
# "Patient counts care unit entrance: 5 (SK I 1 · SK III 4), 1 without a place, 2 discharged"
log-counts-total = Patient counts { $uhs }: { $total }
log-counts-discharged = { $n } discharged
log-places = { $uhs }: { $n } treatment places
log-transport = { $kind } from { $uhs }
log-transport-requested = Transport requested: { $transport }
# Variables: $transport, $status, $unit
log-transport-updated-by = Transport { $transport }: { $status } ({ $unit })
log-transport-updated-to = Transport { $transport }: { $status }
log-order-updated = { $tag }: order { $status }

## Führung – Lage (ADR-0099): more of each UHS, without patient data.
# Variables: $uhs, $destination
transport-of-unit = Transport from { $uhs } → { $destination }
staff-requested-tag = Staff requested: { $need }
waiting-transport-n = { $n ->
    [one] 1 waiting for transport
   *[other] { $n } waiting for transport
}
waiting-no-unit = { $n } without a unit
waiting-transport-total = Waiting for transport: { $n }
# Variables: $all, $here, $discharged
registered-here-discharged = Registered { $all } · here now { $here } · discharged { $discharged }
staff-go-send = Send staff
staff-go-send-label = { staff-go-send }: { $uhs }
# The triage aid on the profile page (PAT-11, ADR-0100).
profile-triage = Triage on this device
# The "+" button that opens the form to add one (ADR-0102).
uhs-new-word = Care unit
