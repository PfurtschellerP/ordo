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

Tags: must, med, m25

Needs: dsn

#### PAT-6 — Patient counts for command
`req~pat-6~2`

While an incident has registered patients, ORDO shall publish to the shared incident compartment, through the patient-count connector, the number of patients per triage category and location, which treatment places are taken, and how many patients have no place. The connector shall contain numbers and place IDs only.

Tags: must, med, m1

Needs: dsn

#### PAT-7 — Person search
`req~pat-7~1`

Where a role is authorised for person search (e.g. for relatives' enquiries), ORDO shall allow searching patients by name and log every search.

Tags: should, med

Needs: dsn

#### PAT-8 — Photo for identification
`req~pat-8~2`

When a medic takes or uploads a photo of a patient, ORDO shall store it in the patient's record for identification only: downscaled, without camera or location metadata, and readable only by those who may read the record. When a medic removes the photo, ORDO shall no longer show it in the record.

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

#### PAT-11 — Triage algorithms as reference
`req~pat-11~1`

When a person triages or pre-triages a patient, ORDO shall offer the triage algorithms of the active profile as references, with their publisher, edition and source, one step away from the triage input, and record which algorithm the person used; ORDO shall never propose, pre-select or check a category ([ADR-0045](../adr/0045-triage-algorithms-as-reference.md)).

Tags: should, med, m26

Needs: dsn

#### PAT-12 — Unique triage tag numbers
`req~pat-12~2`

When a person registers a triage tag number that is already registered in the incident
(letters and digits compared; case, spaces and dashes ignored), ORDO shall not register it
again, and shall say where the patient is and offer their record. When devices registered
the same number while offline, ORDO shall keep the first registration, show on the
patient's card and record that the card was registered more than once, with time, person
and place of each further registration, and ask whoever made a later registration for the
number of a new card; from then on ORDO shall show that registration, and what its device
documented under the old number since, under the new number.

Rationale: while devices are offline, no system can stop two of them from using the same
number. Pre-printed cards are unique, and scanning them avoids typing errors; the rest is
made visible, and the person who came last changes the number (product owner,
2026-11-21, [ADR-0092](../adr/0092-a-card-registered-twice-is-renumbered-by-the-later-registration.md)).

Tags: must, med, m69

Needs: dsn

### 4.10 Med package: on-site care units (UHS)

#### UHS-1 — On-site care units
`req~uhs-1~2`

ORDO shall let the incident command create on-site care units (*Unfallhilfsstelle UHS*, *Behandlungsplatz*, patient collection point), each with location, capacity and assigned staff, and change them; ORDO shall refuse creating or changing them from anyone else.

Tags: must, med, m30, m54

Needs: dsn

#### UHS-2 — UHS board
`req~uhs-2~1`

ORDO shall show a UHS board that lists every patient currently in the unit with triage category, status, waiting time and assigned staff.

Tags: must, med

Needs: dsn

#### UHS-3 — Admission and discharge
`req~uhs-3~1`

When a patient is admitted to or discharged from a UHS, ORDO shall record the time and, on discharge, the outcome: released on site, transported, refused treatment or deceased.

Tags: must, med, m17

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

#### UHS-7 — Treatment places
`req~uhs-7~1`

When an on-site care unit is set up, ORDO shall give it numbered treatment places (changeable later); when a medic assigns a patient to a place, ORDO shall record it, show it in the patient list and mark the place as taken for others, and keep which patient lies where within the clinical compartment.

Tags: must, med, m12

Needs: dsn

#### UHS-8 — Treatment places of every UHS
`req~uhs-8~3`

ORDO shall show the people of the UHS module one overview of all on-site care units they may read, per unit and in total: the free and all treatment places per triage category they are kept for, the patients without a place, open orders, and open and planned transports, so that they see which unit is full and where else a patient could go.

Tags: should, med, m22

Needs: dsn

#### UHS-9 — Working UHS
`req~uhs-9~1`

When a person opens the UHS module, ORDO shall let them choose the UHS they work at at the top of the module, register new patients there and list that UHS's patients; a person allowed at every UHS can also list all of them.

Tags: must, med, m37

Needs: dsn

#### UHS-10 — Only a UHS's staff writes its patients
`req~uhs-10~3`

ORDO shall accept registering, documenting and changing a UHS's patients only from people on its staff list and from roles allowed at every UHS, judged by the staff list valid at the time of the change, so that documentation made offline is not refused because of a later change. Where a person may only read a UHS, ORDO shall not offer them to change anything there.

Tags: must, med, m37

Needs: dsn

#### UHS-11 — UHS among the resources
`req~uhs-11~1`

ORDO shall list every on-site care unit with its staff among the resources (*Einsatzmittel*), where the incident command assigns and removes staff, without access to clinical data.

Tags: must, med, m39

Needs: dsn

#### UHS-12 — UHS count as units on scene
`req~uhs-12~1`

When ORDO shows the strength on scene, it shall count every on-site care unit as one unit,
with the organisation, service and strength the incident command gave it, or without a
strength with the people on its staff list, so that the forces on scene include the UHS
and their people.

Tags: must, med, m57

Needs: dsn

#### UHS-13 — Capacity per triage category
`req~uhs-13~1`

When the incident command gives an on-site care unit a capacity per triage category, ORDO
shall keep it, give the unit treatment places kept for each category, and show its patients
per category against that capacity.

Tags: should, med, m57

Needs: dsn

#### UHS-14 — Place plan
`req~uhs-14~2`

When a person works at one on-site care unit with treatment places, ORDO shall show a plan
of its places, grouped by the triage category they are kept for, with the patient lying on
each taken place, the free places marked, and the unit's patients without a place.
When its staff registers a patient or chooses a free place in the plan, ORDO shall let
them put a patient without a place on it.

Tags: should, med, m57

Needs: dsn

#### UHS-15 — Request a physician or a paramedic
`req~uhs-15~2`

When a person on the staff list of a UHS asks for a physician or a paramedic, ORDO shall
show the request to the incident command until it is met or withdrawn, and when the
command names the person who comes, put them on that UHS's staff list, which gives them
access to its patients, and give that person an Auftrag to go there. ORDO shall refuse
the request of anyone not on that staff list.

Tags: should, med, m62

Needs: dsn

#### UHS-16 — Free treatment places for the incident command
`req~uhs-16~2`

While an incident has on-site care units with treatment places, ORDO shall show the
incident command, per unit and in total, the free and all treatment places per triage
category they are kept for, and the number of patients without a place, from the
patient-count connector and without clinical data. Per unit, ORDO shall also show the
patients waiting for a transport, its staff by qualification, its open staff requests and
its discharged patients.

Tags: should, med, m66, m74

Needs: dsn

#### UHS-17 — Close a UHS
`req~uhs-17~1`

When the incident command closes an on-site care unit that holds no patients, ORDO shall
remove it from every list, map and count of the incident and keep its events in the
incident log.

Rationale: a UHS set up by mistake, or no longer needed, must not stay in the way; a UHS
with patients first hands them over.

Tags: must, med, m67

Needs: dsn

#### UHS-18 — Treatment place when taking a patient
`req~uhs-18~1`

When a person takes a patient handed over to a UHS that has free treatment places, ORDO
shall let them choose one of these places, or none, with the taking, and shall propose the
first free place kept for the patient's documented triage category.

Tags: should, med, m69

Needs: dsn

#### UHS-19 — Free places for crews
`req~uhs-19~1`

While a person is on the crew of a unit, ORDO shall show them the free treatment places
of every on-site care unit per triage category, from the incident's shared counts only,
so that they know where a patient can go without asking by radio.

Rationale: the product owner, 2026-11-21. Numbers only, no patient data (least knowledge).

Tags: should, med, m70

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

#### EPCR-11 — Diagnosis
`req~epcr-11~1`

When a medic documents a diagnosis, ORDO shall record it with its time, its author and
whether it is a working, suspected or confirmed diagnosis, and shall keep it in the
clinical compartment: it shall reach only those who read the patient's record and, with
the handover (EPCR-6), those who carry the patient, never a transport request or a
connector.

Tags: should, med, m58

Needs: dsn

#### EPCR-12 — Other answers as free text
`req~epcr-12~1`

Where the profile allows other answers to a choice, ORDO shall accept a short free text
besides the options and show it with them, e.g. "Sonstiges" for breathing and skin.

Tags: should, med, m58

Needs: dsn

#### EPCR-13 — Pupils as choices
`req~epcr-13~1`

When a medic documents the pupils, ORDO shall offer choices for whether they are equal,
their size and reaction to light on each side, and the consensual reaction, as the DIVI
MIND dataset records them.

Tags: should, med, m58

Needs: dsn

#### EPCR-14 — Record overview
`req~epcr-14~1`

When a person opens a patient record, ORDO shall first show an overview: the triage tag,
triage category, name and the facts the profile puts in the header (e.g. allergies),
where the patient is, the transport, the latest entry of each form the profile shows on
the overview with its time and how long ago it was, the open orders and the patient's
way so far; the timeline, the documentation and the handover shall be one step away.

Rationale: the record grew too long to see the patient's state at a glance (product
owner, 2026-11-19 and 2026-11-20).

Tags: must, med, m67

Needs: dsn

#### EPCR-15 — Timeline of the record
`req~epcr-15~1`

ORDO shall show all entries of a patient record — form entries, triage, place, way,
orders, transports and photos — in one list, newest first, with time and person,
filterable by kind, and with each correction next to the entry it corrects.

Tags: must, med, m67

Needs: dsn

#### EPCR-16 — Patient with a unit's crew
`req~epcr-16~1`

When a member of a unit's crew registers a patient, ORDO shall let them register the
patient with their unit, accept documentation of that patient from the unit's crew, judged
by the crew valid at the time, and count the patient for the incident command as with
units' crews.

Rationale: the person in the field registers the patient where they are found and hands
them over later ([ADR-0087](../adr/0087-patients-travel-between-units.md), product owner,
2026-11-19).

Tags: must, med, m68

Needs: dsn

#### EPCR-17 — Handover within the incident
`req~epcr-17~1`

When the holder of a patient hands them over to a UHS or a unit of the incident, ORDO
shall keep the patient with the holder until the receiving UHS's staff or unit's crew
takes them, show the open handover to both sides with time, person and note, and from the
taking on accept documentation only from the receiver.

Tags: must, med, m68

Needs: dsn

#### EPCR-18 — Quick handover by QR code
`req~epcr-18~1`

When the holder of a patient chooses a quick handover, ORDO shall show a QR code that
names only the patient's record and the handover, and shall let a person who opens or
scans it take the patient for a UHS or unit they document for, also while their devices
are offline.

Tags: must, med, m68

Needs: dsn

#### EPCR-19 — Handover outside ORDO
`req~epcr-19~1`

When the holder of a patient hands them over to someone outside ORDO, ORDO shall record
who took them, when and the note, and take the patient out of the incident as at a
discharge.

Rationale: the receiver gets the handover document of EPCR-20.

Tags: must, med, m68

Needs: dsn

#### EPCR-20 — Handover document
`req~epcr-20~1`

When a person who may read a patient's record asks for the patient's handover document
while their device is connected, ORDO shall give them a PDF/A-3 file laid out by the
active profile's template, with the record's data, the triage history, the way, the orders
and the handover, and with the same data attached as an HL7 FHIR bundle.

Rationale: the hospital, the control centre or another service reads the PDF and can
import the bundle; the layout belongs to the profile, so each state and organisation can
adapt it ([ADR-0089](../adr/0089-handover-document-from-typst-templates.md)).

Tags: must, med, m69

Needs: dsn

#### EPCR-21 — Patients in the crew's workspace
`req~epcr-21~1`

While a person is on the crew of a unit and may read clinical data, ORDO shall let them
register patients with their unit and document them in the crew's own workspace, showing
only the patients their units hold or are handed and not yet taken, and shall open every
patient's record at an address of its own in the workspace the person works in
([ADR-0093](../adr/0093-patients-apart-from-the-uhs.md)).

Tags: must, med, m70

Needs: dsn

### 4.11.1 Med package: medical orders

Orders (*Anordnungen*) are given by physicians and carried out by the crew
of the on-site care unit. ORDO records them; it never proposes, checks or calculates a
medication, dose or measure ([EPCR-9](#epcr-9--no-clinical-decision-support)).

#### ORDER-1 — Give orders
`req~order-1~1`

When a person whose roles permit giving orders ([ORDER-3](#order-3--only-permitted-roles-give-orders)) gives an order for a patient — a medication with substance, dose and route, or a measure — ORDO shall record it in the patient record with the person and the time, and show it to the crew of the patient's on-site care unit as an open order.

Tags: must, med, m15

Needs: dsn

#### ORDER-2 — Carry out orders
`req~order-2~2`

When a crew member acknowledges an order, ORDO shall record who did it and when. When a crew member reports an order done, ORDO shall record who carried it out (which may be another person than the one reporting it), who reported it, and when. When an order is withdrawn, ORDO shall record who withdrew it and when. Orders and their status changes are never deleted.

Tags: must, med, m15

Needs: dsn

#### ORDER-3 — Only permitted roles give orders
`req~order-3~2`

When a person gives a new order or confirms a recorded one, ORDO shall accept it only if the person's roles in the incident permit giving orders (by default physicians only, [ADR-0053](../adr/0053-verbal-orders.md)); the server checks this for every order.

Tags: must, med, m15

Needs: dsn

#### ORDER-4 — Open orders across all UHS
`req~order-4~1`

ORDO shall list the open orders of every on-site care unit the person may read, oldest first, with the patient and the unit, so physicians and crews see what is pending.

Tags: must, med, m15

Needs: dsn

#### ORDER-5 — Document a done medication order
`req~order-5~2`

When a medication order is done, ORDO shall offer to document the administration in the patient care record with the ordered substance, dose, route, the person who ordered it, the person who carried it out and the time it was reported done filled in, for the crew to check, complete and save; ORDO shall save nothing without the crew's confirmation.

Tags: should, med, m21

Needs: dsn

#### ORDER-6 — Record spoken orders
`req~order-6~1`

When a crew member records an order given by voice, radio, phone or telemedicine, ORDO shall record who gave it, by which channel and whether it was read back, mark it until the person who gave it confirms it, and accept the confirmation only from roles that give orders.

Tags: must, med, m50

Needs: dsn

#### ORDER-7 — Who ordered comes from the order
`req~order-7~1`

When an administration is documented in the patient care record, ORDO shall take who ordered it only from an order in ORDO (given, or recorded as spoken), and shall not let it be typed.

Tags: must, med, m50

Needs: dsn

### 4.12 Med package: transport planning

#### TRANS-1 — Transport request
`req~trans-1~1`

When a patient needs transport, ORDO shall allow creating a transport request with priority, required transport type (e.g. RTW, KTW, with physician) and special needs (child, intensive care, infectious).

Tags: must, med, m14

Needs: dsn

#### TRANS-2 — Hospital list
`req~trans-2~1`

ORDO shall keep a list of hospitals with departments, specialties and current capacity; capacity can be entered manually.

Tags: must, med, m18

Needs: dsn

#### TRANS-3 — Transport assignment and status
`req~trans-3~3`

When a transport request is assigned, ORDO shall record the transport unit, and the destination as soon as it is known, shall let the destination be changed until the unit reaches it, and shall track the transport by the unit's status codes the profile names for it (DE: 3, 4, 7, 8, then 1 or 2), with the time of every change.

Tags: must, med, m14

Needs: dsn

#### TRANS-4 — Transport board
`req~trans-4~1`

ORDO shall show a transport board with open transport requests, available transport units and the number of patients sent to each hospital.

Tags: must, med, m18

Needs: dsn

#### TRANS-5 — Planned transports
`req~trans-5~1`

ORDO shall allow planning transports in advance (e.g. scheduled transports at planned operations, patient relocation).

Tags: should, med, m19

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
