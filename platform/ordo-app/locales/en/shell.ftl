app-name = ORDO
skip-link = Skip to content
pair-title = Pair device
pair-intro = Sign in with your organisation's account. This device gets a key of its own that never leaves it; ORDO stores no passwords.
pair-login = Sign in
pair-qr = Pair with a QR code
pair-qr-hint = For shared tablets: scan the QR code with a signed-in phone and approve.
pair-qr-alt = QR code to pair this device
pair-code = Code: { $code }
pair-qr-link = Without a camera: open on another device and enter the code:
pair-waiting = Waiting for approval …
pair-no-login = No sign-in is set up for this server.
pair-manual = Development: pair device by hand
pair-manual-intro = For test setups without sign-in: values from the server's seed file and from `ordo generate-device-key`.
login-failed = Sign-in failed
relogin-title = Sign-in expired
relogin-intro = This device's sign-in has expired. Please sign in again – the data on the device is kept and synchronised afterwards.
network-error = Server not reachable
pair-user = User (name@server)
pair-device = Device ID
pair-key = Device's private key
pair-incident = Incident ID
pair-submit = Pair
unpair = Unpair device
signed-in-as = Signed in as
avatar-change = Change profile picture
avatar-remove = Remove profile picture
retry = Try again
starting = ORDO is starting …
online = Connected
connection-label = Connection: { $state }. { $pending }
connecting = Connecting …
offline = Offline – entries are saved and sent later
my-devices = My devices
page-tabs = Parts of the page
revoked-title = Device blocked
revoked-text = This device has been blocked. Its key and all incident data have been deleted from it. To use it again, pair the device anew.
pair-again = Pair again
device-name-label = Name of this device
device-name-hint = How the device appears in the device list, e.g. “Tablet care unit north”.
device-revoked = Device …{ $short } blocked
retagged = Patient { $old }: new tag number { $new }
uhs-created = { $name } set up
unit-added = Unit { $unit } added
launcher = Switch workspace
views = Views
incident = Incident
reason-no-access = No access in this incident
reason-not-connected = Available after the first connection
no-workspace = No workspace has been released for this device in this incident yet.

## Navigation and header
menu = Menu
search = Search
storage-failed = Device storage full or blocked: entries are kept only until they are sent. Do not reload.

## Appearance (UX-3)
appearance = Appearance
profile = Profile and device
hand = Operated with

## Vim keys (ADR-0090). Key names stay English (ADR-0036).
keymap = Keyboard
keys-title = Keyboard shortcuts
keys-on = Vim keys are on for this device. They work while no text field has the focus, and never change data.
keys-off = Vim keys are off. Turn them on in the menu under “Keyboard”.
keys-vim = Vim keys
keys-turn-on = Turn on Vim keys
keys-turn-off = Turn off Vim keys
keys-always = Always
key-keys = Keys
key-does = Effect
key-help = This overview
key-palette = Command palette
key-search = Search or filter
key-move = To the next element left, below, above, right – across the whole page
key-first-last = First / last element of the page
key-tabs = Next / previous tab
key-views = First to fourth view
key-menu = Menu
key-rail = Views at the edge: j/k to choose, o opens, l or Esc to go back
key-open = Open entry
key-insert = Into the first input field
key-scroll = Half a page down / up
key-escape = Leave text field
key-in-list = In a list: next / previous entry
key-close = Close dialog or menu
theme-set = Appearance: { $label }
photo-added = Photo saved for patient { $tag }
photo-removed = Photo removed for patient { $tag }
photo-restored = Photo of patient { $tag } restored
photo-failed = The picture could not be read.
photo-too-large = The picture is too large, even made smaller.
edit = Edit
save = Save
cancel = Cancel

## Command palette (UX-10)
palette-title = Search and commands
palette-placeholder = Patient, unit, view or command …
palette-results = Results
palette-none = Nothing found.
palette-keys = choose
palette-run = run
palette-close = close
palette-back = back
group-views = View
group-actions = Command
my-roles = Roles
no-role = none (incident planning)
change-incident = Change incident
incident-details-saved = Incident details saved.
incident-times-invalid = The end is before the start, or a date is incomplete.
place-label = Place { $n }
places-set = { $n } treatment places
transport-history = History
transport-requested = { $tag }: transport requested
order-given = { $tag }: order recorded
order-confirmed-said = Order confirmed
order-recorded = { $tag }: verbal order recorded
section-close = Close
# Closes a dialog.
close = Close
done = Done
discharge = Discharge
readmitted = { $tag } readmitted
hospital-saved = Hospital saved: { $name }
capacity-saved = Admission saved: { $name }

## API versions (ADR-0052).
version-unsupported = This app and the server have no version in common. Please reload the page.
crew-set = Crew of { $unit } saved
unit-released = { $unit } released from the incident
# The patient counts of patients with units' crews.
with-units-label = With units
handed-over = Patient { $tag } handed over to { $to }, takeover pending
handed-over-quick = Quick handover started for patient { $tag }
handed-outside = Patient { $tag } handed over outside ORDO to { $receiver }
taken = Patient { $tag } taken over by { $by }
uhs-closed = { $name } closed
uhs-updated = { $name } saved

## What needs the command's attention (TASK-3): the bell in the header.
attention = Notices
attention-none = Nothing that needs attention right now.

## Reports as Aufträge (REP-1…5, ADR-0081); events of ADR-0075 still read.
report-closed = Report closed
task-given-label = Task given
task-patient-taken = Patient admitted from the task
undo = Undo
close-label = { section-close }: { $what }
task-recorded = Task created: { $place }
follow-up-later-done = Follow-up postponed
task-given = Task given to { $unit }
structure-saved = Command structure saved
position-created = { $name } created
map-saved-label = Situation map saved

## Address search (ADR-0066)
address-label = Search address
address-submit = Search
address-searching = Searching …
address-empty = Please enter an address.
address-too-long = Please enter at most 200 characters.
address-offline = The address search needs a connection to the server.
address-not-found = Nothing found. Please write it differently, for example with the town.
address-off = No address search is set up on this server.
address-refused = The address search is for the incident command.
address-failed = The address search is not answering right now. Please try again later.
address-places = Places found
address-attribution = Addresses from OpenStreetMap, © OpenStreetMap contributors (ODbL).
address-chosen = Chosen: { $label }
notices-back = Back

## Stored data that cannot be used (local persistence)
stored-state-unusable = This version cannot read the data on this device ({ $reason }). It is kept unchanged; please reload the app or tell the administration.

## Display names (ORG-7)
display-name = Display name
display-name-hint = How others see you in ORDO. Without a name of your own, the name from the sign-in applies.
display-name-save = Save name
display-name-reset = Use the name from the sign-in
guest-pair-title = Access for helpers
guest-ended-title = Access ended
guest-ended = Your access to this incident ended with your duty. For new access, please ask the incident command.
guest-pairing = Pairing this device with the QR code …
guest-code-invalid = The QR code is no longer valid (already used, expired or duty ended). Please ask the incident command for a new one.
helper-saved = Details of { $name } saved.

## Counts
pending = { $n ->
    [0] Everything sent
    [one] 1 entry not sent yet
   *[other] { $n } entries not sent yet
}
palette-count = { $n ->
    [0] { palette-none }
    [one] 1 result
   *[other] { $n } results
}
attention-label = { $n ->
    [0] { attention }: none
   *[other] { attention }: { $n }
}
address-found = { $n ->
    [one] 1 place found. Please choose.
   *[other] { $n } places found. Please choose.
}

## How long ago, from minutes: "just now", "5 min ago", "2 h 5 min ago", "3 days ago"
ago-now = just now
ago-minutes = { $minutes } min ago
ago-hours = { $hours } h ago
# Variables: $hours, $minutes
ago-hours-minutes = { $hours } h { $minutes } min ago
ago-days = { $days ->
    [one] 1 day ago
   *[other] { $days } days ago
}

## Triage categories in the DE profile: the codes on the triage tags stay.
triage-immediate = SK I
triage-urgent = SK II
triage-minor = SK III
triage-expectant = SK IV
triage-deceased = Deceased
triage-uninjured = Unharmed
# A place kept for a triage category, e.g. "SK I – Place 2" (UHS-13).
category-place = { $category } – place { $n }

## Log lines with values of ORDO's types
patient-registered = Patient { $tag } registered: { $category }, { $uhs }
patient-retriaged = Patient { $tag } re-triaged: { $category }
unit-status-set = { $unit }: status { $status }
# A unit status with its code, e.g. "3 – Task accepted".
status-with-code = { $code } – { $label }
# Variables: $user, $role
role-granted-to = Role { $role } given to { $user }
# Variables: $user, $role
role-withdrawn-from = Role { $role } withdrawn from { $user }
incident-state-changed-to = Incident: { $state }
priority-set = { $tag }: transport priority set
priority-removed = { $tag }: transport priority removed
place-assigned-to = { $tag } on { $place }
place-none = { $tag }: no treatment place
transport-status-set-to = Transport: { $status }
order-status-set-to = Order: { $status }
task-status-set-to = Task: { $status }
# "Discharge: Transported"
discharged-as-outcome = { discharge }: { $outcome }
discharged-with = { $tag }: { $outcome }

## Themes, hands and keyboards (UX-3, ADR-0049, ADR-0090)
theme-auto = Automatic
theme-day = Day
theme-night = Night
theme-high-contrast = High contrast
hand-right = right hand
hand-left = left hand
keymap-standard = Standard
keymap-vim = Vim keys

## Roles offered when granting (prototype; later from the profile)
role-commander = Incident command
role-medic = Medical service
role-physician = Physician
role-paramedic = Paramedic

## States
incident-now-paused = Incident paused
incident-now-resumed = Incident resumed
incident-planned = Planned
incident-active = Running
incident-closed = Closed
transport-status-requested = Requested
transport-status-assigned = Assigned
transport-status-loading = Loading
transport-status-departed = Departed
transport-status-arrived = At destination
transport-status-handed-over = Patient handed over at destination
transport-status-cancelled = Cancelled
order-status-given = Given
order-status-acknowledged = Acknowledged
order-status-done = Done
order-status-withdrawn = Withdrawn
outcome-released-on-site = Released on site
outcome-transported = Transported
outcome-refused = Refused treatment or transport
outcome-deceased = Deceased
task-status-open = Open
task-status-in-progress = In progress
task-status-done = Done
task-status-cancelled = Cancelled

## Dates and numbers, as each language writes them
# Year first, so no reader takes the day for the month.
# Day and month with two digits. Variables: $day, $month, $year
date-of = { $year }-{ $month }-{ $day }
# Variables: $date, $time
date-time-of = { $date } { $time }
# The decimal separator, as in "1.5 MB".
decimal-separator = .

## Errors of the device's own workings (pairing, storage, randomness)
no-random = No random numbers available
no-incident-chosen = No incident chosen
device-id-invalid = Device ID: { $error }
private-key-format = Private key: 64 hex characters expected
photo-not-found = Photo not found
photo-other-version = Photo in another version
app-state-unavailable = App state not available
no-secure-context = Secure storage not available: the app must be opened over HTTPS.
no-device-storage = No device storage
# A device's suggested name from its browser and system (ORG-8): "Firefox on Android".
device-name-on = { $browser } on { $system }
