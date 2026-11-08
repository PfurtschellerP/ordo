# ORDO Requirements — Vision, scope and roles

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

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
8. **Open standards first.** Where an open standard exists and fits, ORDO uses it
   instead of inventing its own format — for data, interfaces, documents, tooling and
   methods. Where none fits, ORDO publishes its own format as an open specification
   (see [ADR-0022](../adr/0022-open-standards-first.md)).

## 2. Scope

### 2.1 Structure

| Layer | Content |
|---|---|
| **Platform** | Identities, organisations, roles, devices, events and sync, data compartments, profiles, module system, audit, map engine, UI shell |
| **Shared modules** | Incidents, units and resources, command structure, tasks, situation map, incident log, organisation master data |
| **Service packages** | **Fire**: breathing-apparatus monitoring, water supply, … · **Med**: patients and triage, on-site care units (UHS), patient care records (ePCR), transport planning, hospitals · **Police**: liaison views first, own modules later |
| **Connectors** | Defined, versioned exchange between service packages (e.g. Med → command: patient counts; Fire → Med: injured firefighter) |
| **Adapters** | External systems: alerting, control centres, hospitals, identity providers |

### 2.2 Planned later (see the [roadmap](../roadmap.md))

- Incident reports and statistics
- Federation: ORDO installations of different organisations work together on shared
  incidents ([§4.15](06-federation-and-interfaces.md#415-federation))
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
  see [constraints](08-constraints.md), medical-device regulation
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
