tab-med = Medical
tab-command = Incident command
# Workspaces of the incident command, by phase (ADR-0061).
ws-lead = Command
ws-prepare = Preparation
ws-crew = Crew
crew-tasks = Tasks
crew-transports = Transports for your unit
crew-none-here = You are on no crew in this incident. The incident command adds you under Preparation – Units.
dispatch = Dispatch
people = Staff
crew-patients = Patients
tab-helpers = Helpers
tab-devices = Devices
incident-page = Incident

## The command structure and the map on one page (ADR-0095).
command-space = Command & space
# Its label in the phone's bottom bar.
command-space-short = Structure
tab-resources = Units
tab-hospitals = Hospitals
tab-free-units = Free transport units
tab-free-units-short = Free units
uhs-title = Care units
uhs-choose = Choose care unit
units-title = Units
units-empty = No units yet.
unit-call-sign = Call sign
unit-type = Type (e.g. ambulance, patient transport, medic team)
unit-organisation = Organisation
unit-submit = Add unit
unit-service = Service
unit-missing = Please give a call sign and a type.
col-status = Status
col-set-status = Set status
no-status = –
set-status-label = Set status of { $unit }
unit-status-title = Units: status and tasks
unit-status-empty = No units in the incident yet. They are added in Preparation.
# Starts with the button's visible text, so its name matches what is seen.
task-give-label = { task-give }: { $unit }
move-here = Move here
# Starts with the button's visible text, so its name matches what is seen.
move-here-label = { move-here }: { $person } to { $here }
group-units = Units
action-add-unit = Add unit

## Incidents and roles (ADR-0046)
incidents-title = Incidents
incidents-intro = Choose an incident. Roles in an incident are given by the incident command or the incident planning.
incidents-loading = Loading incidents …
incidents-offline = The list of incidents is available only with a connection to the server.
# Logging out from the incident list: the device is unpaired, so another person can log in.
log-out = Sign out
log-out-hint = Signing out unpairs this device; then another person can sign in.
incidents-empty = No incident released yet. The incident command or the incident planning gives people a role in an incident.
open = Open
plan-title = Plan incident
plan-name = Name of the incident
plan-submit = Create incident
grant = Give role
incident-state-label = Status
incident-page-title = Incident
incident-prepare = Prepare: situation map, units and staff with roles and command structure. Start the incident when it is running.

## The command's overview (ADR-0067)
open-map = Open situation map
open-dispatch = To dispatch
open-units = To the units
open-label = open
no-open-tasks = No open tasks.
persons-on-scene = Responders on scene
units-on-scene = Units on scene
quick-links = Quick access
task-for = For: { $assignee }
discharged-count = { $n } discharged
start-incident = Start incident
close-incident = Close incident
incident-paused-label = paused
pause-incident = Pause incident
resume-incident = Resume incident
incident-pause-hint = For incidents over several days, such as nights at a festival. While paused everything stays usable; the situation and the incident log go on.
incident-details = Incident details
incident-description = Description
incident-description-hint = For example place, organiser, expected visitors, special features.
incident-starts = Planned start
incident-ends = Planned end
incident-details-save = Save details

## Treatment places
places = Treatment places

## Transport requests (TRANS-1, TRANS-3)
transports = Transports
transports-empty = No open transports.

## Overview for the medical lead (UHS-8).
overview = Overview

## Units (UNIT-1…6).
strength-title = Forces on scene
strength = Strength
strength-legend-title = Strength (leaders / sub-leaders / crew)
strength-numbers = Please give the strength as three whole numbers, or leave it empty.
leaders = Leaders
sub-leaders = Sub-leaders
crew-count = Crew
by-organisation = By organisation
by-service = By service
no-service = no service
no-organisation = no organisation
crew = Crew
crew-none = No crew recorded.
crew-member = Name
crew-qualifications = Qualifications
crew-add = Add to crew
crew-remove = Remove
release = Release from the incident
back-in = Back in the incident

## [impl->dsn~detailed-strength~1]
unit-leaders = Leaders of formations
org-chart = Command structure as an organisation chart
special-functions-legend = Of these with special functions
strength-detailed = Give the strength in detail: leaders of formations and platoon leaders separately, special functions such as physicians and breathing-apparatus wearers apart (DV 102, 2024)
# "of these 2 × physicians, 6 × BA wearers".
special-functions = of these { $listed }
unreported = without strength report: { $n }
# "3 – Task accepted, since 10:12"
status-since = { $status }, since { $since }
crew-label = { crew } of { $unit } ({ $n })
crew-remove-label = Remove { $name } from the crew
release-label = Release { $unit } from the incident
back-in-label = { $unit } back in the incident
released-list = Released units ({ $n })
released-at = Released at { $at }
crew-title = Crew { $unit }
attention-due = Follow-up due: { $unit } – { $task }
attention-new-task = New task: { $task }
attention-unassigned = Task without a unit: { $task }
case-register = Admit patient
open-in-maps-app = Open in the maps app
log-report-closed = Report closed: { $report }
log-report-patient = Patient from the task { $task } admitted by { $unit }

## Tasks (TASK-1…4).
tasks-heading = Open tasks
tasks-empty = No open tasks.
task-give = Give task
task-unit = For
task-unit-choose = Choose unit or position
task-template = Template
task-template-none = no template
task-text = Task
task-follow-up = Follow up in (minutes, optional)
task-missing = Please give a recipient and a task.
task-new = New task
task-create = Create task
task-kind-label = Kind
task-kind-other = Other task
task-kind-missing = Please choose the kind.
task-unit-later = Assign later
task-unit-only = Choose unit
task-unit-hint = Optional for a patient or an event: the unit can also be assigned later.
task-place = Place
task-what = What is going on?
task-what-hint = In a few words, without medical details (they belong in the patient record).
task-received-label = Received
task-received-hint = Leave empty for now.
task-text-missing = Please describe briefly what it is about.
task-place-missing = Please give the place.
task-too-long = Place or description is too long.
task-time-invalid = Please give a time, e.g. 14:05.
task-assign-to = Assign to
task-assign = Assign
task-unit-missing = Please choose a unit.
task-send-further = Send another unit
task-send-to = Also send
task-send = Send
# Opens a small map centred on the Auftrag's pin.
pin-show = Show pin
pin-show-label = Show pin: { $task }
my-tasks = My tasks
pin-title = Pin on the map (optional)
pin-hint = Search an address or tap into the map. With the keyboard: move the map with the arrow keys, then “Set pin at map centre”.
pin-at-centre = Set pin at map centre
pin-clear = Remove pin
pin-none = No pin set.
pin-set = Set pin
pin-move = Move pin
pin-save = Save pin

## Map object cards (MAP-11).
map-details = Details
map-object-gone = No longer on the map.
card-assign = Assign unit
card-open-task = Open task
card-current-task = Task
card-edit = Edit
card-patients = Patients by triage category
card-open-uhs = Open care unit
qualifications-short = Qualifications
# "Anna Schmidt removed."
removed = { $what } removed.
map-details-label = { map-details }: { $what }
card-edit-label = { card-edit }: { $unit }
# "Paramedic: 2"
qualification-count = { $label }: { $n }
crew-names = Names ({ $n })
pin-at = Pin: { $at }
task-received = Received { $at }
task-taken = Patient admitted by { $unit } ({ $at })
follow-up-later = Follow up in 10 minutes
due = Due
tasks-title = { tasks-heading } ({ $n })
# "In progress: Patient: Stage north"
task-step-label = { $step }: { $task }
# One unit on a task, for accessible names: "RTW 2 – Patient: Stage north".
unit-in-task = { $unit } – { $task }
task-units = Units on the task
# "Follow up at 10:42"
follow-up-at = Follow up at { $at }
# "Due: follow up since 10:42"
follow-up-due = { due }: follow up since { $at }
# "10:12, ben@…"
task-given-by = Given { $at } by { $by }

## Command structure (CMD-1…4, ADR-0031).
structure-title = Command structure
structure-short = Structure
structure-empty = No command structure set up yet.
position = Position
position-choose = Choose position
position-create = Create position
position-kind = Kind
position-kind-choose = Choose kind
position-name = Name
position-parent = Reports to
position-top = top level
position-missing = Please give a kind and a name.
position-close = Dissolve
no-position = not assigned
no-leader = Lead: not filled
leader-label = Lead
leader-set = Set or hand over the lead
uhs-assign = Assign care unit
leader = { leader-label }: { $name }
# What a member of a position is, under its name in the org chart.
member-uhs = Care unit
# Shown, never enforced (ADR-0031).
span-of-control = { $n } reporting directly: more than 5, mind the span of control.
position-close-label = Dissolve { $title }

## A position put on the map, or found there, from the tree (CMD-5).
position-map-set-label = Set on the map: { $title }
position-map-show-label = Show on the map: { $title }
# The place panel's status once a position was chosen from the tree.
map-place-ready = { $label }: tap the spot or place it at the map centre.
position-of = Position of { $unit }

## Situation map (MAP-1, MAP-2).
map-title = Situation map
map-short = Map
map-label = Situation map. Arrow keys move, plus and minus zoom.
map-loading = Loading map …
map-hint-touch = Move the map with two fingers
map-hint-wheel = Zoom: Ctrl and mouse wheel
map-hint-wheel-mac = Zoom: ⌘ and mouse wheel
map-unavailable = The map cannot be shown on this device; the list below shows everything on it.
map-from-device = Base map: saved on this device (also without network).
map-from-server = Base map: from the server.
map-no-region = No base map was prepared for this incident: the objects stand on a neutral background.
map-objects = On the map
map-locate = To my location
map-locating = Finding the location …
map-located = The map shows your location.
map-locate-denied = The location is not allowed. Allow ORDO the location in the browser's settings.
my-position = Me
my-position-show = Show my location
my-position-shown = The map shows your location (point “Me”) and the pin. It stays on this device.
map-locate-failed = The location could not be found.
map-objects-none = Nothing on the map yet.
map-show = Show
map-remove = Remove
map-place-hint = Tap the spot – or move the map so the spot lies under the crosshair (also with the arrow keys), and place it at the map centre.
map-place-kind = What
map-place-which = Which
map-place-choose = Please choose
map-place-label = Label
map-place-submit = Place at map centre
map-place-missing = Please choose what to place.
incident-location = Incident location
map-saving = Saving the base map on this device for use without network …
map-save-failed = The map could not be saved.
map-kind-symbol = Tactical symbol
map-symbol-label = Label (optional)
map-show-label = Show { $label } on the map
map-remove-label = Remove { $label } from the map

## The map editor (ADR-0064)
map-edit = Edit situation map
map-edit-hint = Set the incident location, units, care units and positions on the map, draw areas and lines. Command sees the map under Command – Situation map.
map-edit-not-allowed = The incident command edits the map; here it can only be viewed.
map-edit-object = Edit
map-tools = Tool
tool-select = Select
tool-place = Set symbol
tool-area = Area
tool-line = Line
map-tap-to-choose = Tap an object on the map, or edit it in the list below.
map-move-hint = Tap the new spot – or move the map so it lies under the crosshair (also with the arrow keys), and set it at the map centre.
map-move-centre = Set at map centre
map-deselect = Clear selection
reshape-hint = Drag corners to change the shape; drag a new corner from the middle of a side; drag the shape to move it. Without dragging: “Redraw”.
reshape-apply = Apply
reshape-discard = Discard
reshape-redraw = Redraw
map-reshape-failed = This shape cannot be dragged here; “Redraw” always works.
map-edit-label = Edit { $label }
map-chosen = { $label } selected.
map-moved = { $label } moved.
map-placed = { $label } placed.
map-reshaped = { $label } changed.
map-drawn = { $label } saved.
map-removed = { $label } removed.
draw-redraw = Redraw { $label }
log-hint = Every change in this incident that this device knows of, with time, person and device, newest first. Entries are never changed: a correction is a new entry that refers to the old one.
log-empty = No entries yet.
log-pending = not sent yet
log-by = { $user } · device { $device }
log-corrects = (corrects the entry of { $at })
log-pause-since = Pause from { $at }
log-address = Address
log-task-given = Task for { $to }: { $task }
log-position-closed = { $title } dissolved
log-leader = { $title }: lead { $leader }
log-map-placed = { $label } placed on the situation map
log-map-removed = { $label } removed from the situation map

## Map region from the server (MAP-8)
map-region-title = Base map
map-region-short = Base map
map-region-hint = This incident's base map for the situation map: a radius from the free world map (OpenStreetMap via Protomaps). Only the devices of this incident's responders receive it. Devices that saved an offline map save it again afterwards.
map-region-none = No base map has been made for this incident yet.
map-region-new = New base map
map-region-lat = Latitude (centre)
map-region-lon = Longitude (centre)
map-region-radius = Radius (km, 1 to 50)
map-region-from-incident = The centre is the incident location.
map-region-coordinates = Centre in decimal degrees, e.g. 48.37 and 10.90.
map-region-cut = Make base map
map-region-progress = Progress
map-region-invalid = Please give a latitude (−85 to 85), a longitude (−180 to 180) and a radius of 1 to 50 km.
draw-category = Kind
draw-add-corner = Set corner at map centre
draw-undo = Remove last corner
draw-cancel = Cancel
draw-finish = Done
draw-too-few-area = An area needs at least three corners.
draw-too-few-line = A line needs at least two points.

## Helper register (HELP-1, HELP-2, ADR-0071)
helpers-title = Helpers
helpers-intro = Personal details of this incident's helpers. Only the incident command sees them.
helpers-intro-roles = Who has which role in this incident. Only the incident command sees personal details.
helpers-empty = Nobody recorded yet.
helper-add-title = Add person
helper-add = Add
role-add-title = Give role
helper-complete = Complete details
helper-complete-label = { helper-complete }: { $name }
helper-complete-title = Complete the details of { $name }
helper-name = Name
helper-phone = Phone
helper-phone-hint = With the area code, e.g. 0171 1234567 or +43 664 1234567.
helper-phone-invalid = The phone number is not a valid number with an area code, e.g. 0171 1234567.
helper-street = Street and number
helper-postcode = Postcode
helper-postcode-invalid = The postcode does not have the country's form, e.g. 55116.
helper-town = Town
helper-role = Role in the incident
helper-roles = Roles in the incident
or = or
guest-duty-ended = The end of duty has passed. For access without an account, start the duty again or change the end of duty.
guest-until-closed = Guest access until the end of the incident
guest-no-command = The incident command needs an ORDO account.
guest-access = Access by QR code
guest-code-title = QR code for access
guest-code-refused = The access could not be created.
guest-code-alt = QR code that pairs this helper's device with the incident
guest-link = Or open this link on the device:
guest-code-close = Done
guest-code-for = For { $name }: scan with the camera of every device { $name } uses. Valid until { $expires }.
guest-code-until-closed = For { $name }: scan with the camera of every device { $name } uses. Valid until the end of the incident.
guest-access-label = { guest-access }: { $name }
guest-until = Guest access until { $until }
role-missing = Please choose a role.
helper-account-hint = Choose the organisation's account; its name is taken over. Leave empty for helpers without an account.
diet-other = other …
diet-other-label = Other diet
helper-birth-date-label = Date of birth
helper-diet = Diet
diet-none = not given
helper-duty-end = End of duty
helper-account-label = ORDO account
helper-name-missing = Please give a name.
helper-too-long = Each detail may be at most 200 characters long.
helper-birth-date = Please give the date of birth as a date.
helper-account = The ORDO account is not a valid identifier, e.g. anna@ordo.example.org.
helper-time = Please give the start and end of duty as a date with a time.
helper-not-saved = The details could not be saved.
duty-start = Start duty
duty-end = End duty
# Starts with the button's visible text, so its name matches what is seen.
duty-start-label = { duty-start }: { $name }
# Starts with the button's visible text, so its name matches what is seen.
duty-end-label = { duty-end }: { $name }
helper-edit-title = Edit { $name }
duty-pause = Pause duty
duty-resume = Resume duty
duty-pause-label = { duty-pause }: { $name }
duty-resume-label = { duty-resume }: { $name }
# "on a break since 14:02"
paused-since = on a break since { $at }
on-duty-since = on duty since { $at }
duty-from-to = Duty { $from } – { $to }
duty-from = Duty from { $from }
duty-to = Duty until { $to }
# The person typed for a crew is not in the helper register (HELP-3).
not-registered = { $name } is not in the helper register. Record with the name and add?
register-and-add = Record and add
# Starts with the button's visible text, so its name matches what is seen.
register-and-add-label = { register-and-add }: { $name }
# Registration cards (HELP-5, ADR-0072).
# "Photograph Meldekarte", with the profile's label of the card ("card" for several).
card-generic = card
card-photo = Photograph { $label }
card-reading = Reading the card …
card-not-recognised = No card recognised. Please photograph it again: card flat, QR code fully in the picture and sharp.
card-read = { $card } of { $name } read. Please check and record.

## Converted by hand: counts, lists and values of ORDO's types
log-title = Incident log
task-follow-up-number = Please give the minutes as a whole number.
log-pause-ended = Pause ended
draw-hint = Tap each corner – or move the map so the corner lies under the crosshair (also with the arrow keys), and set it. Then “Done”.
helper-duty-start = Start of duty
# "no open tasks", "1 open task", "3 open tasks"
open-tasks-of = { $n ->
    [0] no open tasks
    [one] 1 open task
   *[other] { $n } open tasks
}
# People on more than one crew or staff list (UNIT-8), e.g. "Ben (RTW 1, care unit south)".
double-booked-people = Assigned more than once, perhaps counted twice: { $people }
# The person being added is on other crews or staff lists already (UNIT-8).
assigned-elsewhere-to = { $person } is already assigned to { $posts }. Move, so that nobody is counted twice?
# Variables: $unit, $overwritten, $kept
lost-update-of = Attention { $unit }: status { $overwritten } was replaced by { $kept } at the same time. Please check.
action-status-of = Status { $status }
follow-ups-due = { $n ->
    [one] 1 follow-up due
   *[other] { $n } follow-ups due
}
unreported-strength = { $n ->
    [0] All units have reported their strength.
    [one] 1 unit without strength report.
   *[other] { $n } units without strength report.
}
transports-open = { $n ->
    [0] Transports: none open
   *[other] Transports: { $n } open
}
summary-map = { $objects ->
    [0] Nothing on the map yet.
    [one] 1 object on the map.
   *[other] { $objects } objects on the map.
}
# The state as people read it, with a pause: "Running – paused".
incident-state-paused = { $state } – { incident-paused-label }
# "13.11.2026, 18:00 to 15.11.2026, 23:00"
incident-from-to = { $starts } to { $ends }
incident-from = from { $starts }
incident-to = until { $ends }
# Variables: $user, $role
withdraw-role = Withdraw: { $role } from { $user }
# The strength read out: "Strength: 0 leaders, 1 sub-leader, 8 crew, 9 in total".
strength-count = { $count } { $part }
strength-read = { strength }: { $parts }, { $total } in total
strength-special = { $strength }, { $special }
strength-platoon-leaders = Platoon leaders
strength-forces = Responders
# A strength's legend, e.g. "Strength (leaders / sub-leaders / crew)".
strength-legend-of = { $title } ({ $parts })
units-count = { $n ->
    [one] 1 unit
   *[other] { $n } units
}
# Variables: $kind, $received, $place, $text
log-report-line = Report taken: { $kind }, { $received } – { $place } – { $text }
# "Role Physician withdrawn from Anna Schmidt."
role-removed-from = Role { $role } withdrawn from { $who }.
# Variables: $action, $task
pin-edit-of = { $action }: { $task }
task-kind-event = Event
task-kind-patient = Patient
# "Patient: Stage north"
task-label-of = { $kind }: { $place }
# How far a task is (TASK-2).
task-phase-received = Received
task-phase-assigned = Assigned
task-phase-in-progress = In progress
task-phase-done = Done
task-phase-cancelled = Cancelled
# Variables: $kind, $received, $place, $task
log-task-received-line = Task received: { $kind }, { $received } – { $place } – { $task }
log-task-received-to = { $line } – to { $to }
log-task-assigned = Task “{ $task }” assigned to { $unit }
task-step-open = Open
task-step-in-progress = In progress
task-step-done = Done
task-step-cancel = Cancel
# Time left until a follow-up, e.g. "7:05 min left"; "now" at zero.
countdown-now = now
# Variables: $minutes, $seconds
countdown-left = { $minutes }:{ $seconds } min left
# "Handover: Dr M. → Dr K."
handover-from-to = Handover: { $from } → { $to }
map-kind-uhs = Care unit
map-kind-drawing = Drawing
map-kind-report = Report
map-saved-size = Base map saved on this device ({ $megabytes } MB), usable without network.
# "With units: 2 (SK I 1 · SK II 1)"
with-units-line = { $label }: { $total } ({ $parts })
# Variables: $module, $event-type
log-unknown = { $module }: { $event-type }
log-planned = Incident “{ $name }” created
# Variables: $added, $unit-type, $organisation
log-unit-added-line = { $added } ({ $unit-type }, { $organisation })
log-helper-registered = { $name } added
log-helper-saved = Details of { $name } saved
log-helper-changes = { $name }: { $changes }
log-cleared = { $label } cleared
log-fields-changed-list = Details changed: { $fields }
# Variables: $unit, $n
log-crew = { $n ->
    [0] Crew of { $unit }: nobody
    [one] Crew of { $unit }: 1 person
   *[other] Crew of { $unit }: { $n } people
}
# A unit's step on a task (ADR-0085): "Task “…”: RTW 2 In progress".
log-unit-step-of = Task “{ $task }”: { $unit } { $status }
log-task-updated-to = Task “{ $task }”: { $status }
log-task-follow-up-changed = Task “{ $task }”: follow-up changed
log-assigned-to = { $what } reports to: { $position }
log-assigned-none = { $what }: no longer reports to a position
map-region-preparing = Preparing the base map …
map-region-tiles = Making the base map: { $done } of { $total } tiles …
map-region-ready-size = Base map ready ({ $megabytes } MB).
# Variables: $km, $lat, $lon
map-region-area = Radius { $km } km around { $lat }, { $lon }.
map-region-created = Made { $at }.
map-region-failed = The base map could not be made: { $error }
draw-corners = { $n ->
    [0] No corner set yet.
    [one] 1 corner set.
   *[other] { $n } corners set.
}
# The diets of the helpers still expected or on duty, for catering.
diet-counts-line = Diet (without ended duty): { $list }
diet-count = { $n } × { $diet }
# Starts with the button's visible text, so its name matches what is seen.
helper-edit-of = { $edit }: { $name }

## Führung – Lage (ADR-0099): the map beside the units and the treatment places.
lage = Situation
# The groups the command sees its units in (UNIT-10); the profile gives each status its group.
group-available = Available
group-task = On a task
group-transport = Transporting
group-unavailable = Not available
group-none = No status
# The first of the type filter's chips: "Alle 14".
filter-all = All
units-filter = Type
units-only-available = Available only
units-glance-empty = No unit matches the choice.
open-lage = To the situation
# "4 von 14 einsatzbereit", the first number large.
units-available-of = of { $total } available
col-type = Type
col-free = free
col-all = all

## Führung & Raum: actions at each position (ADR-0102), forms in a dialog.
position-new = Position
lead-word = Lead
sub-position-word = Sub-position
uhs-word = Care unit
map-set-word = Place
leader-set-label = Set the lead: { $title }
position-sub-label = Add a sub-position: { $title }
uhs-assign-label = Assign a care unit: { $title }
