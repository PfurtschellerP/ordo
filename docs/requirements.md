# ORDO — Requirements

> **O**perations, **R**esponse, **D**ispatch & **O**rganisation
> Status: **Draft v0.4** · 2026-11-08

## 1. Vision

ORDO is an open-source **platform for on-scene coordination of emergency services**.
Fire services, emergency medical services and police share one platform, but each gets
its own **service package** with dedicated views and workflows. The packages exchange
exactly the information the others need through defined **connectors**.

It gives everyone on scene a shared, live picture: which units are there, where they
are deployed, what they are doing, how many patients there are and where they are
being treated and taken, and what has happened so far.

It is built **first for volunteer organisations** (volunteer fire brigades, medical
and first-aid services, rescue and disaster-relief units) in **Germany**, but its data
model, terminology and language support are international from the start, so it can
grow to professional agencies, other countries and large multi-agency incidents.

ORDO is **composable**: a common platform, separate service packages and defined
interfaces between them, so each organisation uses exactly the parts it needs. ORDO
ships **ready-to-use packages with sensible defaults**; configuring is possible, but
never required to get started.

ORDO starts **small** (one organisation, one server) and grows: from a single binary to
containers and clusters, and from one installation to a **federation** of installations
run by different organisations, connected through **open, published interfaces**.

### Guiding principles

1. **Usable under stress.** A volunteer who has never been trained on ORDO can use the
   basics within 15 minutes, at night, in the rain, wearing gloves.
2. **Works without network.** Mobile coverage at an incident is never guaranteed.
3. **Easy to run.** A volunteer organisation with no IT staff can install and run it.
4. **Trustworthy record.** Everything that happens is logged with time and author and
   can't be silently changed afterwards.
5. **Need to know.** Everyone sees what they need for their job, and no more. Patient
   data in particular never reaches people or devices that don't need it.
6. **Modular.** Platform, shared functions and service packages are separate, so new
   services and functions can be added without changing existing ones.
7. **Open.** Open source, open formats, open interfaces; no lock-in.

## 2. Scope

### 2.1 Structure

| Layer | Content |
|---|---|
| **Platform** | Identities, organisations, roles, devices, events and sync, data compartments, profiles, module system, audit, map engine, UI shell |
| **Shared modules** | Incidents, units and resources, command structure, tasks, situation map, incident log, organisation master data |
| **Service packages** | **Fire**: breathing-apparatus monitoring, water supply, … · **Med**: patients and triage, on-site care units (UHS), patient care records (ePCR), transport planning, hospitals · **Police**: liaison views first, own modules later |
| **Connectors** | Defined, versioned exchange between service packages (e.g. Med → command: patient counts; Fire → Med: injured firefighter) |
| **Adapters** | External systems: alerting, control centres, hospitals, identity providers |

### 2.2 Planned later (see §7 Roadmap)

- Incident reports and statistics
- Federation: ORDO installations of different organisations work together on shared
  incidents (§4.15)
- Kubernetes deployment (Helm chart)
- Importing incidents from alerting systems and control centres
- Hospital pre-notification and capacity data from external systems
- Dedicated police modules (together with police partners; legal requirements differ
  per state)
- Hosted multi-tenant service

### 2.3 Explicitly out of scope (for now)

- Call-taking and dispatch in a control centre (CAD)
- Alerting members on their phones (integrate existing systems instead)
- Radio (TETRA / analogue) integration
- Clinical decision support (automatic diagnosis, treatment or triage suggestions) —
  see §6, medical-device regulation
- Billing of medical services

## 3. Users and roles

### 3.1 Shared / command

| Role | Typical person | Main needs |
|---|---|---|
| **Incident commander** (*Einsatzleiter*) | Commander of the first unit on scene | Overview at a glance; assign units and sectors fast; see what is open |
| **Command assistant** (*Führungsassistent, Melder*) | Second person in the command vehicle | Keep the log, record radio messages, update unit status |
| **Sector leader** (*Abschnittsleiter*) | Leads one sector | See own units and tasks; report back |
| **Unit leader** (*Gruppen-/Truppführer*) | Leads a vehicle crew | Receive tasks; report status |
| **Viewer** | Other agency, mayor, control-centre liaison | Read-only situation picture |
| **Administrator** | IT-savvy member | Set up organisation, members, vehicles, devices, packages |

### 3.2 Med

| Role | Typical person | Main needs |
|---|---|---|
| **Medical incident lead** (*Leitender Notarzt LNA*, *Organisatorischer Leiter OrgL*) | Leads medical response in a larger incident | Patient numbers by category, UHS capacity, transport status |
| **UHS lead** (*Leiter UHS / Behandlungsplatz*) | Runs an on-site care unit | Who is in the UHS, in which state, waiting how long, treated by whom |
| **Medic** (*Sanitäter, Rettungssanitäter/-assistent, Notfallsanitäter*) | Treats patients | Fast documentation on the patient record; minimal typing |
| **Physician** (*Arzt, Notarzt*) | Treats patients, decides on transport | Full patient record; sign off; transport decision |
| **Transport officer** (*Transportorganisation*) | Plans patient transports | Open transport requests, free transport units, hospital capacity |

### 3.3 Fire and police

Fire-specific roles (e.g. *Atemschutzüberwachung*) come with the Fire package. Police
roles start as **liaison** (*Verbindungsbeamter*): sees the shared situation picture
and the information the connectors share with police.

Roles are **per incident**: the same person can be commander in one incident and
medic in another. Roles can be handed over, and the handover is logged.

## 4. Functional requirements

Priorities: **M** = must for the package's first release, **S** = should, **C** =
could (later).

### 4.1 Shared: incidents

| ID | Requirement | Prio |
|---|---|---|
| INC-1 | Create an incident in ≤ 3 taps with type, keyword (*Einsatzstichwort*) and location; everything else optional | M |
| INC-2 | Location by address search, GPS of the device, or tap on map; works offline with cached data | M |
| INC-3 | Incident lifecycle: planned → active → closed → archived; closed incidents are read-only | M |
| INC-4 | Several incidents can be active at the same time | M |
| INC-5 | Incident keywords and types are configurable per organisation/region | M |
| INC-6 | **Planned operations**: set up an incident in advance for a planned event (e.g. *Sanitätswachdienst* at a concert or festival) with sectors, units, UHS and shifts | M |
| INC-7 | Several services (fire, med, police) take part in one incident, each with its own package views | M |
| INC-8 | Create an incident from an external alert (e-mail, API, alerting system) | C |

### 4.2 Shared: units and resources

| ID | Requirement | Prio |
|---|---|---|
| UNIT-1 | Add units to an incident from master data or ad hoc (neighbouring organisation, other service) | M |
| UNIT-2 | Each unit has a call sign (*Funkrufname*), type (e.g. HLF, RTW, KTW, NEF), organisation, service and crew strength in tactical notation (e.g. 1/8/**9**) | M |
| UNIT-3 | Unit status with timestamps; status codes configurable (e.g. FMS/TMS status 1–9) | M |
| UNIT-4 | Show total strength on scene (units, persons, by organisation and service) | M |
| UNIT-5 | Record crew members of a unit, including qualifications | S |
| UNIT-6 | Release a unit from the incident; it stays in the history | M |

### 4.3 Shared: command structure

| ID | Requirement | Prio |
|---|---|---|
| CMD-1 | Create sectors (*Einsatzabschnitte*) and sub-sectors; each has a name, a leader and optionally a service | M |
| CMD-2 | Assign units to sectors by drag & drop (tablet) or two taps | M |
| CMD-3 | Show the command structure as a tree/org chart | M |
| CMD-4 | Hand over a leadership role to another person; logged | M |

### 4.4 Shared: tasks and orders

| ID | Requirement | Prio |
|---|---|---|
| TASK-1 | Give a task to a unit or sector (free text + optional template) | M |
| TASK-2 | Task status: open → in progress → done / cancelled | M |
| TASK-3 | List of open tasks always visible to the responsible leader | M |
| TASK-4 | Reminders / follow-ups ("ask sector 2 for status in 10 min") | S |

### 4.5 Shared: situation map

| ID | Requirement | Prio |
|---|---|---|
| MAP-1 | Map centred on the incident, with units, sectors, UHS and drawn objects | M |
| MAP-2 | Map works offline: regions can be downloaded in advance | M |
| MAP-3 | Tactical symbols (*taktische Zeichen*) from the profile's symbol set | M |
| MAP-4 | Draw points, lines and areas (e.g. danger zone, hose line, cordon, staging area) | M |
| MAP-5 | Map layers per service package (e.g. hydrants for fire, hospitals for med); each user chooses which to see | M |
| MAP-6 | Show live device positions of units (opt-in) | C |
| MAP-7 | Building plans / fire-brigade plans (*Feuerwehrpläne*) as overlay | C |

### 4.6 Shared: incident log (*Einsatztagebuch*)

| ID | Requirement | Prio |
|---|---|---|
| LOG-1 | Every change is logged automatically with time, author and device | M |
| LOG-2 | Manual entries, including radio messages with sender, receiver and direction | M |
| LOG-3 | The log is append-only: entries can be corrected only by a new entry that references the old one | M |
| LOG-4 | Filter and search the log (by unit, sector, service, type) | S |
| LOG-5 | Export the log as PDF and in a machine-readable format | M |
| LOG-6 | Each user sees only log entries from compartments they may read (e.g. no clinical details in the command log) | M |

### 4.7 Shared: organisation and master data

| ID | Requirement | Prio |
|---|---|---|
| ORG-1 | Manage members, qualifications, vehicles and their default crew strength | M |
| ORG-2 | Import members and vehicles from CSV | S |
| ORG-3 | Pair a device via QR code; a paired device can be revoked | M |
| ORG-4 | Join an incident as a guest (e.g. neighbouring organisation) via QR code / link, with limited rights | S |
| ORG-5 | Choose which service packages an organisation uses (e.g. a medical service uses only Med) | M |

### 4.8 Fire package

| ID | Requirement | Prio |
|---|---|---|
| FIRE-1 | Fire command workspace: units, sectors, tasks, map, log with fire wording and symbols | M |
| FIRE-2 | **Breathing-apparatus monitoring** (*Atemschutzüberwachung*): register teams, start time, pressure readings, alarms at configurable thresholds, retreat reminders. Safety-critical: own detailed requirements and tests before release | S |
| FIRE-3 | Water supply: hydrants and water sources on the map, hose lines | S |
| FIRE-4 | Injured or missing firefighter: one action raises a request to Med and marks the event in the log (via connector) | S |
| FIRE-5 | Hazardous materials support (substance lookup, danger zones) | C |

### 4.9 Med package: patients and triage

| ID | Requirement | Prio |
|---|---|---|
| PAT-1 | Register a patient in ≤ 10 s with minimal data: triage tag number (*Patientenanhängekarte*, scan QR/barcode or type), triage category, location; everything else later | M |
| PAT-2 | Triage categories come from the profile (DE: *Sichtungskategorien* SK I–IV, plus deceased) | M |
| PAT-3 | Re-triage at any time; the history is kept | M |
| PAT-4 | Patients may be unidentified; identity data can be added later; duplicates can be merged | M |
| PAT-5 | Patient pathway: found → UHS / treatment area → transport → hospital (or released on site) | M |
| PAT-6 | Live counts per category and location, shared with command via connector (numbers only) | M |
| PAT-7 | Search for a person (e.g. for relatives' enquiries), restricted to authorised roles and logged | S |

### 4.10 Med package: on-site care units (UHS)

| ID | Requirement | Prio |
|---|---|---|
| UHS-1 | Create on-site care units (*Unfallhilfsstelle UHS*, *Behandlungsplatz*, patient collection point) with location, capacity and staff | M |
| UHS-2 | UHS board: all patients in the UHS with category, status, waiting time and assigned staff | M |
| UHS-3 | Admission and discharge with outcome: released on site, transported, refused treatment, deceased | M |
| UHS-4 | Several UHS per incident; overview of load across all UHS | M |
| UHS-5 | Shift planning for staff at planned operations | S |
| UHS-6 | Material and consumables tracking | C |

### 4.11 Med package: electronic patient care record (ePCR)

| ID | Requirement | Prio |
|---|---|---|
| EPCR-1 | One patient care record per patient, linked to the triage tag and the patient pathway | M |
| EPCR-2 | Record structure based on the German emergency dataset (DIVI **MIND**); forms are defined in the profile, not in code | M |
| EPCR-3 | Quick entry on tablet: vital signs, findings, measures, medication, each with time; large controls, minimal typing | M |
| EPCR-4 | Append-only: corrections are new entries that reference the old one; the responsible person signs off | M |
| EPCR-5 | Document refusal of treatment or transport, with signature | M |
| EPCR-6 | Handover to transport crew or hospital: printable/PDF summary; later structured exchange (HL7 FHIR) | M (PDF) / C (FHIR) |
| EPCR-7 | Only assigned medical staff can read a record; every access is logged | M |
| EPCR-8 | Retention as required by law (DE: at least 10 years), separately from incident data | M |
| EPCR-9 | Calculations that only sum or convert documented values (e.g. GCS total) are allowed; no automatic diagnosis, treatment or triage suggestions (§6) | M |
| EPCR-10 | Works fully offline; records are encrypted on the device | M |

### 4.12 Med package: transport planning

| ID | Requirement | Prio |
|---|---|---|
| TRANS-1 | Create a transport request for a patient: priority, required transport type (e.g. RTW, KTW, with physician), special needs (child, intensive care, infectious) | M |
| TRANS-2 | Hospital list with departments/specialties and current capacity; capacity entered manually first | M |
| TRANS-3 | Assign a transport unit and destination; track status (requested → assigned → loading → departed → arrived → handed over) | M |
| TRANS-4 | Transport board: open requests, available transport units, patients per hospital | M |
| TRANS-5 | Planned transports (e.g. scheduled transports at planned operations or patient relocation) | S |
| TRANS-6 | Distribution overview to avoid overloading the nearest hospital; suggestions only, the decision stays with the person | C |
| TRANS-7 | Hospital capacity and pre-notification from external systems via adapter (e.g. IVENA eHealth) | C |

### 4.13 Police package

| ID | Requirement | Prio |
|---|---|---|
| POL-1 | Liaison view: shared situation picture, cordons, access and exit routes, staging areas | S |
| POL-2 | Receive the information other packages share with police via connectors (e.g. number of injured, scene access) | S |
| POL-3 | Dedicated police modules only together with police partners | C |

### 4.14 Connectors between service packages

| ID | Requirement | Prio |
|---|---|---|
| CONN-1 | Service packages exchange information **only** through defined, versioned connectors; never by reading each other's internal data | M |
| CONN-2 | Each connector states what data it shares, in which direction, and why; this list is visible to administrators | M |
| CONN-3 | Connectors share the minimum necessary; clinical details never leave the Med package | M |
| CONN-4 | Requests across services (e.g. fire asks for medical care; med asks for technical rescue) are tasks with status visible to both sides | M |
| CONN-5 | First connectors: patient counts and transport status (Med → command); responder injured (Fire → Med); sectors, danger zones, staging areas (command → all); cordons and routes (command/police → all) | M |

### 4.15 Federation

| ID | Requirement | Prio |
|---|---|---|
| FED-1 | ORDO installations of different organisations can work together on one incident without a central server | C |
| FED-2 | Federation is **closed by default**: an installation only exchanges data with partners it has explicitly agreed with | C |
| FED-3 | A federation agreement states which compartments and roles may cross to the partner | C |
| FED-4 | Each incident has a lead installation (*home*); the lead can be handed over (e.g. when a higher command level takes over); the handover is logged | C |
| FED-5 | If the connection between installations breaks, each side keeps working and they catch up automatically afterwards | C |
| FED-6 | Origin and integrity of every event can be verified by every installation that receives it | C |
| FED-7 | Identifiers (users, organisations, incidents, events) are globally unique from the first release, so installations can federate later without data migration | M |

### 4.16 Open interfaces

| ID | Requirement | Prio |
|---|---|---|
| API-1 | All external interfaces (client–server, sync protocol, federation, connector contracts, profile format) are published as open, versioned specifications | M |
| API-2 | Specifications are versioned independently of the software; servers announce which versions they support | M |
| API-3 | Specifications are under a permissive licence — CC BY 4.0 for texts, Apache-2.0 for schemas and the conformance suite — so that other systems, including commercial ones, can implement them | M |
| API-4 | A conformance test suite lets other implementations check that they follow the specification | S |

## 5. Non-functional requirements

### 5.1 Usability

| ID | Requirement |
|---|---|
| UX-1 | Main functions reachable in ≤ 3 taps from the workspace overview |
| UX-2 | Touch targets ≥ 12 mm; usable with gloves on a 10" tablet |
| UX-3 | Day mode with high contrast for sunlight; night mode that doesn't dazzle |
| UX-4 | A new user can do the core task of their role without training |
| UX-5 | No data entry is lost when the app is closed, the device sleeps or the network drops |
| UX-6 | Works on tablet (main device), laptop and phone (reduced view) |
| UX-7 | Meets WCAG 2.2 AA where it doesn't conflict with the above |
| UX-8 | Each role opens in its **workspace**: only the views and actions for that role, in that service's wording |
| UX-9 | Ready-made packages work with defaults; no configuration needed to start |

### 5.2 Offline and reliability

| ID | Requirement |
|---|---|
| OFF-1 | All functions needed on scene work without any network connection |
| OFF-2 | Devices at the same scene can share data over a local network without internet |
| OFF-3 | When the connection returns, changes sync automatically without user action |
| OFF-4 | Concurrent changes never lose data; conflicts are resolved automatically and visibly |
| OFF-5 | The device shows clearly whether it is in sync, and since when it isn't |

### 5.3 Performance

| ID | Requirement |
|---|---|
| PERF-1 | Runs smoothly on a mid-range Android tablet that is 5 years old |
| PERF-2 | Changes appear on other connected devices within 2 s |
| PERF-3 | Handles an incident with 100 units, 50 sectors, 500 patients and 10,000 log entries |
| PERF-4 | A small installation runs on a Raspberry Pi 5 or a mini PC |
| PERF-5 | Resource-efficient by design: low memory and CPU use, small downloads, minimal battery drain |
| PERF-6 | App starts and shows the active incident in < 2 s on the reference tablet, also offline |

### 5.4 Security and data protection

| ID | Requirement |
|---|---|
| SEC-1 | All network traffic encrypted (TLS); data on devices encrypted at rest |
| SEC-2 | Role-based access per organisation, per incident and per compartment |
| SEC-3 | Login with local accounts; single sign-on via OpenID Connect for larger organisations |
| SEC-4 | Lost or stolen devices can be revoked remotely; devices with clinical data lock automatically and require a PIN or biometrics |
| SEC-5 | GDPR: data minimisation, configurable retention and deletion, export of personal data |
| SEC-6 | Audit trail of all access to and changes of incident data |
| SEC-7 | Regular dependency updates and a documented security-reporting process |
| SEC-8 | **Data compartments**: data is separated into compartments (e.g. shared incident data, clinical data, police-restricted data), each with its own access rules, encryption key, sync scope and retention |
| SEC-9 | A device only receives data from compartments its user may read; e.g. a fire tablet never stores patient records |
| SEC-10 | Health data is treated as special-category data (GDPR Art. 9): encrypted per compartment on server and devices, access logged per record |

### 5.5 Operation

| ID | Requirement |
|---|---|
| OPS-1 | Install a self-hosted server with one command (single binary or container image) and a setup wizard |
| OPS-2 | Updates without data loss; automatic database migrations |
| OPS-3 | Built-in backup and restore, including encrypted compartments |
| OPS-4 | The same software can later run as a hosted multi-tenant service |
| OPS-5 | An official container image (multi-architecture: amd64, arm64) is published with every release |
| OPS-6 | Ready-made Docker Compose setups: minimal (ORDO + automatic TLS) and full (ORDO + PostgreSQL + TLS) |
| OPS-7 | All configuration via environment variables or a config file; nothing baked into the image |
| OPS-8 | Health and readiness endpoints, structured logs to stdout, metrics (Prometheus) and optional tracing (OpenTelemetry) |
| OPS-9 | With PostgreSQL, several server instances can run in parallel behind a load balancer |
| OPS-10 | Graceful shutdown: connected devices reconnect to another instance without losing data |
| OPS-11 | A Helm chart for Kubernetes (later) |

### 5.6 Internationalisation

| ID | Requirement |
|---|---|
| I18N-1 | User interface in German and English at launch; all texts translatable |
| I18N-2 | Regional profiles bundle terminology, unit types, status codes, tactical symbols, incident keywords, triage categories and patient-record forms. **The first profile is Germany (DE)**: FwDV 100, DV 102, DIN/EN vehicle types, status 1–9, *Sichtungskategorien*, DIVI MIND. Austria and Switzerland follow. |
| I18N-3 | Times stored in UTC and shown in local time; units of measurement configurable |
| I18N-4 | The core contains **no country-specific code or data**; everything regional comes from profiles |
| I18N-5 | A new country, state or organisation profile can be added without changing or recompiling ORDO |
| I18N-6 | Profiles are layered: a profile can extend another and add, override or hide entries (e.g. DE → Bavaria → own organisation) |
| I18N-7 | Profiles can be written and validated by non-programmers (documented text format + validation tool) |
| I18N-8 | Each profile declares which version of the profile format it uses; ORDO rejects incompatible profiles with a clear message |

### 5.7 Modularity

| ID | Requirement |
|---|---|
| MOD-1 | ORDO is layered: platform → shared modules → service packages, with connectors between packages and adapters to external systems |
| MOD-2 | Every functional area is a module with its own events, state, permissions, views and settings |
| MOD-3 | A new module or service package can be added without changing existing ones |
| MOD-4 | Service packages and modules can be switched on or off per organisation |
| MOD-5 | Modules depend only on the platform and on other modules' published interfaces, never on their internals |
| MOD-6 | Interfaces to external systems are adapters behind a common interface |
| MOD-7 | Map symbol sets, forms, report templates and translations are replaceable data, not code |

## 6. Constraints

- Licence: **AGPL-3.0-or-later**. Modified versions must stay open, including when they
  are offered as a hosted service.
- Implemented in Rust; client runs as WebAssembly (see architecture).
- Only open-licensed map data, symbols and fonts (e.g. OpenStreetMap).
- No dependency on a single commercial cloud provider.
- ORDO is a support tool. Responsibility stays with the people in charge, and
  procedures must work without it (fallback: paper, e.g. paper triage tags).
- **Medical-device regulation (EU MDR):** ORDO documents and organises; it does not
  diagnose, recommend treatment or make triage decisions. Any feature that would do so
  needs a regulatory assessment first, because it could make ORDO a medical device.
- **Health data:** GDPR Art. 9 and national rules on medical documentation (DE: e.g.
  § 630f BGB retention, professional confidentiality) apply to the ePCR. Requires review
  by a data-protection expert before the first field use.

## 7. Roadmap (draft)

| Phase | Content |
|---|---|
| **0 — Foundation** | Repository, CI, container image, Docker Compose, architecture decisions, platform core (events, sync, compartments, modules, profiles, global IDs, signed events), sync prototype, first spec drafts |
| **1 — MVP (Med)** | Shared modules (incidents, units, command, tasks, map, log, org) + **Med package: patients and triage, UHS, transport planning, hospitals** (no ePCR yet); planned operations; offline; one organisation; field tests at planned events |
| **2 — Fire** | Fire package; first connectors between Fire and Med; field tests with fire brigades |
| **3 — Patient records** | ePCR with clinical compartment, PDF handover; data-protection review |
| **4 — Connected** | Federation between installations, alert import, local server in the command vehicle, breathing-apparatus monitoring, conformance tests |
| **5 — Professional** | Single sign-on, Helm chart, hosted service, control-centre interfaces (EMSI), hospital interfaces (FHIR, IVENA), Matrix adapter, police modules with partners |

## 8. Open questions

1. Which open tactical-symbol set (DV 102) can be used, and under which licence?
2. Which organisations in Germany could be early test partners — first medical services
   for planned events (e.g. DRK, ASB, JUH, MHD), then fire brigades?
3. Which DIVI MIND version should the ePCR be based on, and is its use licence-free?
4. Who can review data protection and medical-device questions before field use?
