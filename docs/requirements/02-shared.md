# ORDO Requirements — Functional requirements: shared modules

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

### 4.1 Shared: incidents

#### INC-1 — Create an incident quickly
`req~inc-1~1`

Create an incident in ≤ 3 taps with type, keyword (*Einsatzstichwort*) and location; everything else optional

Tags: must, shared

Needs: dsn

#### INC-2 — Locate the incident
`req~inc-2~1`

Location by address search, GPS of the device, or tap on map; works offline with cached data

Tags: must, shared

Needs: dsn

#### INC-3 — Incident lifecycle
`req~inc-3~1`

Incident lifecycle: planned → active → closed → archived; closed incidents are read-only

Tags: must, shared

Needs: dsn

#### INC-4 — Several active incidents
`req~inc-4~1`

Several incidents can be active at the same time

Tags: must, shared

Needs: dsn

#### INC-5 — Configurable keywords and types
`req~inc-5~1`

Incident keywords and types are configurable per organisation/region

Tags: must, shared

Needs: dsn

#### INC-6 — Planned operations
`req~inc-6~1`

**Planned operations**: set up an incident in advance for a planned event (e.g. *Sanitätswachdienst* at a concert or festival) with sectors, units, UHS and shifts

Tags: must, shared

Needs: dsn

#### INC-7 — Several services in one incident
`req~inc-7~1`

Several services (fire, med, police) take part in one incident, each with its own package views

Tags: must, shared

Needs: dsn

#### INC-8 — Incident from external alert
`req~inc-8~1`

Create an incident from an external alert (e-mail, API, alerting system)

Tags: could, shared

Needs: dsn

### 4.2 Shared: units and resources

#### UNIT-1 — Add units
`req~unit-1~1`

Add units to an incident from master data or ad hoc (neighbouring organisation, other service)

Tags: must, shared, m29

Needs: dsn

#### UNIT-2 — Unit attributes
`req~unit-2~1`

Each unit has a call sign (*Funkrufname*), type (e.g. HLF, RTW, KTW, NEF), organisation, service and crew strength in tactical notation (e.g. 1/8/**9**)

Tags: must, shared, m29

Needs: dsn

#### UNIT-3 — Unit status
`req~unit-3~1`

Unit status with timestamps; status codes configurable (e.g. FMS/TMS status 1–9)

Tags: must, shared, m29

Needs: dsn

#### UNIT-4 — Strength on scene
`req~unit-4~1`

Show total strength on scene (units, persons, by organisation and service)

Tags: must, shared, m29

Needs: dsn

#### UNIT-5 — Crew members
`req~unit-5~1`

Record crew members of a unit, including qualifications

Tags: should, shared, m29

Needs: dsn

#### UNIT-7 — Detailed strength as an option
`req~unit-7~1`

Where the incident command chooses the detailed notation for its incident, ORDO shall
give strengths in five numbers (leaders of formations, platoon leaders, sub-leaders,
other forces, total) and name the special functions of the people (e.g. physicians,
respirator wearers) after it, as in the 2024 recommendation for DV 102; otherwise ORDO
shall keep the four-number notation.

Tags: should, shared, m57

Needs: dsn

#### UNIT-8 — One post per person
`req~unit-8~1`

When someone adds a person to the crew of a unit or the staff list of a UHS while that
person is on another crew or staff list of the incident, ORDO shall say where they are and
offer to move them there from the other lists. Where a person is on more than one crew or
staff list anyway, e.g. after changes made offline, ORDO shall mark the strength on scene
and name the person and the lists, because they may be counted twice.

Tags: should, shared, m57

Needs: dsn

#### UNIT-6 — Release units
`req~unit-6~1`

Release a unit from the incident; it stays in the history

Tags: must, shared, m29

Needs: dsn

### 4.3 Shared: command structure

#### CMD-1 — Sectors
`req~cmd-1~1`

Create sectors (*Einsatzabschnitte*) and sub-sectors; each has a name, a leader and optionally a service

Tags: must, shared, m32

Needs: dsn

#### CMD-2 — Assign units to sectors
`req~cmd-2~1`

Assign units to sectors by drag & drop (tablet) or two taps

Tags: must, shared, m32

Needs: dsn

#### CMD-3 — Command structure view
`req~cmd-3~1`

Show the command structure as a tree/org chart

Tags: must, shared, m32

Needs: dsn

#### CMD-4 — Hand over leadership
`req~cmd-4~1`

Hand over a leadership role to another person; logged

Tags: must, shared, m32

Needs: dsn

### 4.4 Shared: tasks and orders

#### TASK-1 — Give tasks
`req~task-1~1`

Give a task to a unit or sector (free text + optional template)

Tags: must, shared, m31

Needs: dsn

#### TASK-2 — Task status
`req~task-2~1`

Task status: open → in progress → done / cancelled

Tags: must, shared, m31

Needs: dsn

#### TASK-3 — Open tasks always visible
`req~task-3~1`

List of open tasks always visible to the responsible leader

Tags: must, shared, m31

Needs: dsn

#### TASK-4 — Reminders
`req~task-4~1`

Reminders / follow-ups ("ask sector 2 for status in 10 min")

Tags: should, shared, m31

Needs: dsn

### 4.5 Shared: situation map

#### MAP-1 — Situation map
`req~map-1~1`

Map centred on the incident, with units, sectors, UHS and drawn objects

Tags: must, shared, m33, m54, m55, m56

Needs: dsn

#### MAP-2 — Offline map
`req~map-2~2`

When a device opens the situation map of an incident whose map region is prepared and
not yet kept on the device, ORDO shall store the whole region on the device without being
asked, so the map works offline.

Tags: must, shared, m33, m55, m56

Needs: dsn

#### MAP-3 — Tactical symbols
`req~map-3~1`

Tactical symbols (*taktische Zeichen*) from the profile's symbol set

Tags: must, shared, m34

Needs: dsn

#### MAP-4 — Drawing on the map
`req~map-4~1`

Draw points, lines and areas (e.g. danger zone, hose line, cordon, staging area)

Tags: must, shared, m40, m54

Needs: dsn

#### MAP-5 — Map layers per package
`req~map-5~1`

Map layers per service package (e.g. hydrants for fire, hospitals for med); each user chooses which to see

Tags: must, shared

Needs: dsn

#### MAP-6 — Live unit positions
`req~map-6~1`

Show live device positions of units (opt-in)

Tags: could, shared

Needs: dsn

#### MAP-7 — Building plans
`req~map-7~1`

Building plans / fire-brigade plans (*Feuerwehrpläne*) as overlay

Tags: could, shared

Needs: dsn

#### MAP-8 — Map region from the server
`req~map-8~2`

When the incident command chooses a centre and a radius for an incident, ORDO shall cut that region from an open world map on the server, show the progress, and offer the region only to the devices of the people in that incident, for offline use as well.

Tags: must, shared, m38, m51

Needs: dsn

#### MAP-9 — Address search
`req~map-9~1`

When the incident command enters an address and asks to search it, ORDO shall show the
places an open geocoder finds for it and, for the place chosen, use its coordinates as
the centre of the map region or move the map there.

Tags: should, shared, m55

Needs: dsn

### 4.6 Shared: incident log (*Einsatztagebuch*)

#### LOG-1 — Automatic log
`req~log-1~1`

Every change is logged automatically with time, author and device

Tags: must, shared, m36

Needs: dsn

#### LOG-2 — Manual entries and radio messages
`req~log-2~1`

Manual entries, including radio messages with sender, receiver and direction

Tags: must, shared

Needs: dsn

#### LOG-3 — Append-only log
`req~log-3~1`

The log is append-only: entries can be corrected only by a new entry that references the old one

Tags: must, shared, m36

Needs: dsn

#### LOG-4 — Filter and search the log
`req~log-4~1`

Filter and search the log (by unit, sector, service, type)

Tags: should, shared

Needs: dsn

#### LOG-5 — Log export
`req~log-5~1`

Export the log as PDF and in a machine-readable format

Tags: must, shared

Needs: dsn

#### LOG-6 — Log respects compartments
`req~log-6~1`

Each user sees only log entries from compartments they may read (e.g. no clinical details in the command log)

Tags: must, shared, m36

Needs: dsn

### 4.6a Shared: helpers in an incident

#### HELP-1 — Helper register
`req~help-1~1`

When the incident command registers a helper of its incident, ORDO shall record the name,
and where given the date of birth, address, phone, organisation, qualifications from the
profile and the start and end of duty, in a compartment of their own that only the
incident command reads.

Tags: should, shared, m59

Needs: dsn

#### HELP-2 — Helpers linked to accounts
`req~help-2~1`

Where a helper has an ORDO account, ORDO shall allow linking the helper to it, so the same
person is recognised in the roster, crews and staff lists.

Tags: should, shared, m59

Needs: dsn

#### HELP-3 — Crews from the register
`req~help-3~1`

When someone adds a person to the crew of a unit, ORDO shall take the person from the
helper register, where a new helper can be registered with a name alone, and shall pass
on only the name and the link, never the other personal data.

Tags: should, shared, m59

Needs: dsn

#### HELP-4 — Helpers' data deleted after the incident
`req~help-4~1`

When the retention period for helpers' data has passed after the incident was closed
(90 days unless the installation sets another), ORDO shall delete the helpers' personal
data on the server and on every device; the crews keep the names.

Tags: should, shared, m59

Needs: dsn

#### HELP-5 — Registration card read by camera
`req~help-5~1`

When the incident command photographs a helper's registration card whose format the
profile describes (DE: the *Meldekarte* of meldekarte.de), ORDO shall read its QR code on
the device and fill the helper form with what the register keeps, for the command to check
and save; ORDO shall take no other data from the card and send the photo nowhere.

Tags: should, shared, m59

Needs: dsn

### 4.7 Shared: organisation and master data

#### ORG-1 — Master data
`req~org-1~2`

Manage members and qualifications. Vehicles are no master data: an incident's units are
incident data, planned per incident (ADR-0070).

Tags: must, shared

Needs: dsn

#### ORG-2 — CSV import
`req~org-2~2`

Import members from CSV

Tags: should, shared

Needs: dsn

#### ORG-3 — Device pairing
`req~org-3~2`

When a logged-in person approves a device by scanning the QR code it shows, ORDO shall pair that device with the person, without typing passwords on the device. (Revoking devices: SEC-4.)

Tags: must, shared, m8

Needs: dsn

#### ORG-4 — Guest access
`req~org-4~1`

Join an incident as a guest (e.g. neighbouring organisation) via QR code / link, with limited rights

Tags: should, shared

Needs: dsn

#### ORG-5 — Choose service packages
`req~org-5~1`

Choose which service packages an organisation uses (e.g. a medical service uses only Med)

Tags: must, shared

Needs: dsn

#### ORG-6 — Incidents planned ahead, roles per incident
`req~org-6~1`

ORDO shall let planners prepare incidents ahead with the people taking part and their
roles. When a planner or the incident's command grants a person a role in an incident,
ORDO shall give that person's devices what the role permits in that incident; when the
role is withdrawn, ORDO shall take it away and delete that data from the devices. Roles
shall not come from groups of the identity provider.

Tags: must, shared, m11, m56

Needs: dsn

#### ORG-7 — Display names
`req~org-7~1`

ORDO shall show people by a display name (e.g. "Anna Schmidt") wherever it names them —
roster, history entries, orders, person fields, suggestions — and keep the user ID only
where it must be exact. The display name shall come from the identity provider at login
(OpenID Connect `name`), and a person may change their own. Where no display name is
known, ORDO shall show the user ID.

Tags: should, shared, m44

Needs: dsn

#### ORG-8 — Device names
`req~org-8~1`

When a device is paired, ORDO shall give it a name, suggested from the browser and operating system (e.g. "Firefox auf Android") and changeable by the person, and shall show that name in the device list, so that a lost or retired device is recognised before it is revoked.

Tags: should, shared, m43

Needs: dsn

#### ORG-9 — Organisation roles
`req~org-9~1`

When an administrator grants a person an organisation role (administrator, planner), optionally until a date, ORDO shall give that person's devices what the role permits at once, and shall take it away when the administrator withdraws the role or the date has passed. Roles from the server's configuration shall be shown but not changed in ORDO.

Tags: should, shared, m45

Needs: dsn

#### ORG-10 — Local accounts
`req~org-10~1`

When an administrator creates, resets or deletes a local login account (e-mail address, name, password) in ORDO, ORDO shall make the change in the installation's identity provider (Dex) without a restart; when an account is deleted, ORDO shall end that person's access on their devices. ORDO shall not store the passwords.

Tags: should, shared, m46

Needs: dsn

#### ORG-11 — Upstream login providers
`req~org-11~1`

When an administrator adds or changes an OpenID Connect provider of the organisation in ORDO, ORDO shall offer saving it only after a successful test login through it that shows the identity ORDO would take, shall make the change in the installation's identity provider (Dex) without a restart, shall never show the client secret again, and shall log who changed which provider.

Tags: should, shared, m52

Needs: dsn

#### ORG-12 — Devices in the incident
`req~org-12~1`

When the incident command opens the people of its incident, ORDO shall list the devices of everyone with a role in the incident, with their person and whether they are connected, and shall let it block (revoke) any of them; it shall not show the organisation's other devices.

Tags: should, shared, m53

Needs: dsn

#### ORG-13 — Incident details and pause
`req~org-13~1`

When the incident command describes an incident, ORDO shall keep its description and
planned start and end with the incident, and while an incident runs, ORDO shall let the
command pause and resume it, e.g. overnight at a festival over several days.

Tags: should, shared, m56

Needs: dsn
