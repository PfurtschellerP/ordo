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

Tags: must, shared

Needs: dsn

#### UNIT-2 — Unit attributes
`req~unit-2~1`

Each unit has a call sign (*Funkrufname*), type (e.g. HLF, RTW, KTW, NEF), organisation, service and crew strength in tactical notation (e.g. 1/8/**9**)

Tags: must, shared

Needs: dsn

#### UNIT-3 — Unit status
`req~unit-3~1`

Unit status with timestamps; status codes configurable (e.g. FMS/TMS status 1–9)

Tags: must, shared

Needs: dsn

#### UNIT-4 — Strength on scene
`req~unit-4~1`

Show total strength on scene (units, persons, by organisation and service)

Tags: must, shared

Needs: dsn

#### UNIT-5 — Crew members
`req~unit-5~1`

Record crew members of a unit, including qualifications

Tags: should, shared

Needs: dsn

#### UNIT-6 — Release units
`req~unit-6~1`

Release a unit from the incident; it stays in the history

Tags: must, shared

Needs: dsn

### 4.3 Shared: command structure

#### CMD-1 — Sectors
`req~cmd-1~1`

Create sectors (*Einsatzabschnitte*) and sub-sectors; each has a name, a leader and optionally a service

Tags: must, shared

Needs: dsn

#### CMD-2 — Assign units to sectors
`req~cmd-2~1`

Assign units to sectors by drag & drop (tablet) or two taps

Tags: must, shared

Needs: dsn

#### CMD-3 — Command structure view
`req~cmd-3~1`

Show the command structure as a tree/org chart

Tags: must, shared

Needs: dsn

#### CMD-4 — Hand over leadership
`req~cmd-4~1`

Hand over a leadership role to another person; logged

Tags: must, shared

Needs: dsn

### 4.4 Shared: tasks and orders

#### TASK-1 — Give tasks
`req~task-1~1`

Give a task to a unit or sector (free text + optional template)

Tags: must, shared

Needs: dsn

#### TASK-2 — Task status
`req~task-2~1`

Task status: open → in progress → done / cancelled

Tags: must, shared

Needs: dsn

#### TASK-3 — Open tasks always visible
`req~task-3~1`

List of open tasks always visible to the responsible leader

Tags: must, shared

Needs: dsn

#### TASK-4 — Reminders
`req~task-4~1`

Reminders / follow-ups ("ask sector 2 for status in 10 min")

Tags: should, shared

Needs: dsn

### 4.5 Shared: situation map

#### MAP-1 — Situation map
`req~map-1~1`

Map centred on the incident, with units, sectors, UHS and drawn objects

Tags: must, shared

Needs: dsn

#### MAP-2 — Offline map
`req~map-2~1`

Map works offline: regions can be downloaded in advance

Tags: must, shared

Needs: dsn

#### MAP-3 — Tactical symbols
`req~map-3~1`

Tactical symbols (*taktische Zeichen*) from the profile's symbol set

Tags: must, shared

Needs: dsn

#### MAP-4 — Drawing on the map
`req~map-4~1`

Draw points, lines and areas (e.g. danger zone, hose line, cordon, staging area)

Tags: must, shared

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

### 4.6 Shared: incident log (*Einsatztagebuch*)

#### LOG-1 — Automatic log
`req~log-1~1`

Every change is logged automatically with time, author and device

Tags: must, shared

Needs: dsn

#### LOG-2 — Manual entries and radio messages
`req~log-2~1`

Manual entries, including radio messages with sender, receiver and direction

Tags: must, shared

Needs: dsn

#### LOG-3 — Append-only log
`req~log-3~1`

The log is append-only: entries can be corrected only by a new entry that references the old one

Tags: must, shared

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

Tags: must, shared

Needs: dsn

### 4.7 Shared: organisation and master data

#### ORG-1 — Master data
`req~org-1~1`

Manage members, qualifications, vehicles and their default crew strength

Tags: must, shared

Needs: dsn

#### ORG-2 — CSV import
`req~org-2~1`

Import members and vehicles from CSV

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
