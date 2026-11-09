# ORDO Requirements — Functional requirements: Med package

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

### 4.9 Med package: patients and triage

#### PAT-1 — Fast patient registration
`req~pat-1~1`

When a medic registers a patient, ORDO shall require only the triage tag number (scanned from the QR or barcode of the *Patientenanhängekarte*, or typed), the triage category and the location, so that registration takes at most 10 seconds. All other data can be added later.

Tags: must, med

Needs: dsn

#### PAT-2 — Triage categories from profile
`req~pat-2~1`

ORDO shall take the available triage categories from the active profile (DE: *Sichtungskategorien* SK I–IV plus deceased).

Tags: must, med

Needs: dsn

#### PAT-3 — Re-triage with history
`req~pat-3~1`

When a patient is re-triaged, ORDO shall record the new category with time and author and keep all earlier categories in the patient's history.

Tags: must, med, m1

Needs: dsn

#### PAT-4 — Unidentified patients
`req~pat-4~1`

ORDO shall allow registering patients without identity data. When identity data becomes known, ORDO shall allow adding it; when two records turn out to describe the same patient, ORDO shall allow merging them and keep both histories.

Tags: must, med

Needs: dsn

#### PAT-5 — Patient pathway
`req~pat-5~1`

ORDO shall track each patient's pathway — found → on-site care (UHS / treatment area) → transport → hospital, or released on site — with time and location of every change.

Tags: must, med

Needs: dsn

#### PAT-6 — Patient counts for command
`req~pat-6~1`

While an incident has registered patients, ORDO shall publish the number of patients per triage category and location to the shared incident compartment through the patient-count connector. The connector shall contain numbers only.

Tags: must, med, m1

Needs: dsn

#### PAT-7 — Person search
`req~pat-7~1`

Where a role is authorised for person search (e.g. for relatives' enquiries), ORDO shall allow searching patients by name and log every search.

Tags: should, med

Needs: dsn

#### PAT-8 — Photo for identification
`req~pat-8~1`

When a medic takes or uploads a photo of a patient, ORDO shall store it in the patient's record for identification only: downscaled, without camera or location metadata, and readable only by those who may read the record.

Tags: should, med, m6

Needs: dsn

#### PAT-9 — Unharmed affected people
`req~pat-9~1`

When people are affected but unharmed ("Betroffene"), ORDO shall register them with a category of their own, apart from the triage categories, and count them apart.

Tags: must, med, m12

Needs: dsn

#### PAT-10 — Transport priority
`req~pat-10~1`

When a medic gives a patient a transport priority, ORDO shall record it apart from the triage category, only for the categories and with the levels the regional profile defines (DE: SK I and SK II), and show it wherever the patient is listed; a re-triage to a category without transport priority shall end it.

Tags: must, med, m12

Needs: dsn

### 4.10 Med package: on-site care units (UHS)

#### UHS-1 — On-site care units
`req~uhs-1~1`

ORDO shall allow creating on-site care units (*Unfallhilfsstelle UHS*, *Behandlungsplatz*, patient collection point), each with location, capacity and assigned staff.

Tags: must, med

Needs: dsn

#### UHS-2 — UHS board
`req~uhs-2~1`

ORDO shall show a UHS board that lists every patient currently in the unit with triage category, status, waiting time and assigned staff.

Tags: must, med

Needs: dsn

#### UHS-3 — Admission and discharge
`req~uhs-3~1`

When a patient is admitted to or discharged from a UHS, ORDO shall record the time and, on discharge, the outcome: released on site, transported, refused treatment or deceased.

Tags: must, med

Needs: dsn

#### UHS-4 — Several UHS
`req~uhs-4~1`

ORDO shall support several UHS per incident and show the load of all of them (patients versus capacity) in one overview.

Tags: must, med, m4

Needs: dsn

#### UHS-5 — Shift planning
`req~uhs-5~1`

Where an incident is a planned operation, ORDO shall allow planning staff shifts per UHS.

Tags: should, med

Needs: dsn

#### UHS-6 — Material tracking
`req~uhs-6~1`

Where material tracking is switched on, ORDO shall track material and consumables per UHS.

Tags: could, med

Needs: dsn

### 4.11 Med package: electronic patient care record (ePCR)

#### EPCR-1 — One record per patient
`req~epcr-1~1`

ORDO shall keep one patient care record per patient, linked to the patient's triage tag number and pathway.

Tags: must, med, m6

Needs: dsn

#### EPCR-2 — Record structure from profile
`req~epcr-2~1`

ORDO shall take the structure of the patient care record (forms, fields, units, choices) from the active profile, not from code. The DE profile follows the German emergency dataset (DIVI **MIND**).

Tags: must, med, m6

Needs: dsn

#### EPCR-3 — Quick entry
`req~epcr-3~1`

When a medic documents vital signs, findings, measures or medication, ORDO shall record each entry with its time and author, with large controls and as little typing as possible.

Tags: must, med, m6

Needs: dsn

#### EPCR-4 — Append-only record
`req~epcr-4~1`

When an entry of the patient care record is corrected, ORDO shall record the correction as a new entry that references the old one and keep the old entry visible. (Sign-off by the responsible person follows later.)

Tags: must, med, m6

Needs: dsn

#### EPCR-5 — Refusal of treatment
`req~epcr-5~1`

Document refusal of treatment or transport, with signature

Tags: must, med

Needs: dsn

#### EPCR-6 — Handover summary
`req~epcr-6~1`

Handover to transport crew or hospital: printable/PDF summary; later structured exchange (HL7 FHIR)

Tags: must, med

Needs: dsn

#### EPCR-7 — Need-to-know access
`req~epcr-7~1`

Only assigned medical staff can read a record; every access is logged

Tags: must, med

Needs: dsn

#### EPCR-8 — Medical retention
`req~epcr-8~1`

Retention as required by law (DE: at least 10 years), separately from incident data

Tags: must, med

Needs: dsn

#### EPCR-9 — No clinical decision support
`req~epcr-9~1`

Calculations that only sum or convert documented values (e.g. GCS total) are allowed; no automatic diagnosis, treatment or triage suggestions (see [constraints](08-constraints.md))

Tags: must, med

Needs: dsn

#### EPCR-10 — Offline and encrypted
`req~epcr-10~1`

Works fully offline; records are encrypted on the device

Tags: must, med

Needs: dsn

### 4.12 Med package: transport planning

#### TRANS-1 — Transport request
`req~trans-1~1`

When a patient needs transport, ORDO shall allow creating a transport request with priority, required transport type (e.g. RTW, KTW, with physician) and special needs (child, intensive care, infectious).

Tags: must, med

Needs: dsn

#### TRANS-2 — Hospital list
`req~trans-2~1`

ORDO shall keep a list of hospitals with departments, specialties and current capacity; capacity can be entered manually.

Tags: must, med

Needs: dsn

#### TRANS-3 — Transport assignment and status
`req~trans-3~1`

When a transport request is assigned, ORDO shall record the transport unit and destination and track the status requested → assigned → loading → departed → arrived → handed over, with the time of every change.

Tags: must, med

Needs: dsn

#### TRANS-4 — Transport board
`req~trans-4~1`

ORDO shall show a transport board with open transport requests, available transport units and the number of patients sent to each hospital.

Tags: must, med

Needs: dsn

#### TRANS-5 — Planned transports
`req~trans-5~1`

ORDO shall allow planning transports in advance (e.g. scheduled transports at planned operations, patient relocation).

Tags: should, med

Needs: dsn

#### TRANS-6 — Distribution overview
`req~trans-6~1`

Where distribution support is switched on, ORDO shall show how patients are distributed across hospitals and may suggest destinations. ORDO shall never assign a destination without a person's decision.

Tags: could, med

Needs: dsn

#### TRANS-7 — External hospital data
`req~trans-7~1`

Where a hospital-capacity adapter (e.g. IVENA eHealth) is configured, ORDO shall import capacity data and may send pre-notifications.

Tags: could, med

Needs: dsn
