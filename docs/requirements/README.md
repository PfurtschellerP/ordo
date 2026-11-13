# ORDO Requirements

What ORDO must do and how well. Every requirement is a **specification item** in the
[OpenFastTrace](https://github.com/itsallcode/openfasttrace) (OFT) Markdown format, so it
can be traced to the design that covers it and to the code and tests that implement it.

| File | Content |
|---|---|
| [01-vision-and-scope.md](01-vision-and-scope.md) | Vision, guiding principles, scope, users and roles |
| [02-shared.md](02-shared.md) | Shared modules: incidents, units, command, tasks, map, log, organisation |
| [03-fire.md](03-fire.md) | Fire package |
| [04-med.md](04-med.md) | Med package: patients and triage, UHS, ePCR, transport planning |
| [05-police-and-connectors.md](05-police-and-connectors.md) | Police package; connectors between packages |
| [06-federation-and-interfaces.md](06-federation-and-interfaces.md) | Federation; open interfaces |
| [07-quality.md](07-quality.md) | Usability, offline, performance, security, operation, internationalisation, modularity, accessibility, design system |
| [08-constraints.md](08-constraints.md) | Licence, law, regulation and other constraints |

Planning lives in the [roadmap](../roadmap.md), decisions in the [ADRs](../adr/README.md),
the solution in the [architecture](../architecture/README.md), terms in the
[glossary](../glossary.md).

## How a requirement looks

Schematic (placeholders in angle brackets; a real example is
[PAT-3](04-med.md#pat-3--re-triage-with-history)):

```markdown
#### <LABEL> — <Short title>
`req~<label in lower case>~<revision>`

<Requirement text in EARS form.>

Tags: <priority>, <area>[, <milestone>]

Needs: dsn
```

Do not write complete example IDs in documentation: OpenFastTrace would import them as
real items.

| Part | Meaning |
|---|---|
| Heading | Human-readable label (`PAT-3`) and a short title |
| ID line | `req~<name>~<revision>`: the OFT ID. The name is the label in lower case. Increase the revision when the meaning changes, so that everything that covered the old revision is flagged for review |
| Text | The requirement, written in EARS (below) |
| `Tags:` | Priority (`must`, `should`, `could`), area (`shared`, `fire`, `med`, `police`, `connectors`, `federation`, `interfaces`, `quality`) and milestone (`m0`, …) |
| `Needs:` | Which artifact types must cover it: normally `dsn` (a design item in the architecture) |

## Writing requirements: EARS

New and changed requirements use the **EARS** sentence patterns (*Easy Approach to
Requirements Syntax*). They make requirements unambiguous for people and machines.

| Pattern | Template | Example |
|---|---|---|
| Always | ORDO shall … | ORDO shall keep a list of hospitals with … |
| Event | **When** …, ORDO shall … | When a patient is re-triaged, ORDO shall … |
| State | **While** …, ORDO shall … | While an incident has registered patients, ORDO shall … |
| Unwanted | **If** …, **then** ORDO shall … | If a theme fails a contrast pair, then ORDO shall reject it |
| Optional | **Where** …, ORDO shall … | Where material tracking is switched on, ORDO shall … |

The Med requirements for the MVP (`PAT`, `UHS`, `TRANS`), the compartment rules and the
milestone-0 items are already in EARS form. Others are rephrased when they are next
changed.

## Traceability chain

```text
req  (requirements)      what is needed               Needs: dsn
 └─ dsn (architecture)   how ORDO solves it            Needs: impl, utest
     ├─ impl (code)      // [impl->dsn~<name>~<revision>]
     ├─ utest (unit tests)        // [utest->dsn~<name>~<revision>]
     └─ itest (acceptance tests)  // [itest->dsn~<name>~<revision>]
```

- Code and tests point to **design items**, not directly to requirements.
- Acceptance scenarios for key workflows are written in **Gherkin**
  ([tests/acceptance/features/](../../tests/acceptance/features/)) and become executable tests
  with the MVP.

## Milestones and CI

CI traces all items tagged with a milestone reached so far, plus everything without
tags (the coverage tags in code). Every item of the milestone must be fully covered, otherwise
the build fails.

| Milestone | Tag | Content |
|---|---|---|
| 0 — Foundation | `m0` | Globally unique IDs (FED-7), verifiable events (FED-6), design tokens and themes (DS-2, DS-3, DS-4, DS-6) |
| 1 — Sync prototype | `m1` | Compartment-scoped sync (SEC-9), sealed health data (SEC-10), automatic sync (OFF-3), no lost changes (OFF-4), triage history (PAT-3), patient counts for command (PAT-6), minimum connector data (CONN-3) |
| 2 — Server | `m2` | Configuration from the environment (OPS-7); store-before-send and wrapped keys strengthen OFF-3 and SEC-10 |
| 3 — First app | `m3` | No lost input (UX-5), colour never the only carrier of meaning (A11Y-3) |
| 4 — Several UHS | `m4` | Several on-site care units with a load overview (UHS-4) |
| 5 — Navigation | `m5` | Each role opens in its workspace (UX-8) |
| 6 — Patient record | `m6` | Patient care record from the profile (EPCR-1…4), photo for identification (PAT-8), keyboard command palette (UX-10) |
| 7 — Login, step 1 | `m7` | Connections prove the device key (SEC-11); access by roles with Cedar policies (SEC-2) |
| 8 — Login, step 2 | `m8` | Login with OpenID Connect (SEC-3), pairing by QR code (ORG-3) |
| 9 — Login, step 3 | `m9` | Device logins expire after a timeout, renewed in the background (SEC-12); lost devices are revoked and wiped (SEC-4) |
| 10 — Security hardening | `m10` | Data on devices encrypted at rest (SEC-1) |
| 11 — Incidents and roles | `m11` | Incidents planned ahead, roles granted per incident (ORG-6) |
| 12 — MANV basics | `m12` | "Betroffene" (PAT-9), transport priority (PAT-10), treatment places (UHS-7) |
| 13 — Storage | `m13` | Attachments as encrypted blobs in an object store (OFF-6); device storage as SQLite on OPFS |
| 14 — Transport requests | `m14` | Transport request from the UHS (TRANS-1); assignment of unit and destination, status with times (TRANS-3) |
| 15 — Medical orders | `m15` | Orders by physicians and paramedics, carried out by the UHS crew (ORDER-1…4) |
| 16 — Header and handedness | `m16` | Header order, profile menu, bottom bar for left-handed use (UX-11, ADR-0049) |
| 17 — Discharge | `m17` | Discharge from the UHS with outcome, readmission, load without discharged patients (UHS-3) |
| 18 — Hospitals and transport board | `m18` | Hospital list with capacity, hospitals as destinations, transport board (TRANS-2, TRANS-4) |
| 19 — Planned transports | `m19` | Transports requested for a planned time, listed apart on the board (TRANS-5) |
| 20 — Person suggestions | `m20` | Person fields suggest the incident's people; role grants suggest the installation's people (UX-13) |
| 21 — Orders into the record | `m21` | A done medication order prefills the record's medication form (ORDER-5) |
| 22 — Medical lead overview | `m22` | One overview per UHS: load, places, orders, transports, discharged (UHS-8) |
| 23 — List keys | `m23` | j/k and arrows in lists, Ctrl+J/Ctrl+K in the palette (UX-14, ADR-0051) |
| 24 — API versions | `m24` | Sync protocol and HTTP API versions announced and negotiated (API-2, API-5, ADR-0052) |
| 25 — MANV patient flow | `m25` | Patients move between units (collection point, treatment area, UHS); totals across units for the medical lead (PAT-5, UHS-8) |
| 26 — Triage references | `m26` | Triage algorithms of the profile as references; the algorithm used is recorded (PAT-11, ADR-0045) |
| 27 — Post-quantum transport | `m27` | Hybrid post-quantum TLS key exchange, checked in CI; algorithm inventory (SEC-14, ADR-0042) |
| 28 — Bills of materials | `m28` | CycloneDX SBOM of server and web app on every build, checked (SEC-15, ADR-0043) |
| 29 — Units complete | `m29` | Units from master data, service and strength, status since, strength on scene, crew, release (UNIT-1…6) |
| 30 — UHS details | `m30` | Location, capacity and assigned staff of each on-site care unit (UHS-1) |
| 31 — Tasks | `m31` | Tasks to units with status and follow-ups, open tasks always visible to the command (TASK-1…4) |
| 32 — Command structure | `m32` | Positions (medical branch first), leaders and handovers, units and UHS assigned, the tree (CMD-1…4, ADR-0031) |
| 33 — Situation map | `m33` | Map with the incident, units, UHS and positions placed; offline region (MAP-1, MAP-2, ADR-0006) |
| 34 — Tactical symbols | `m34` | Symbols generated per DV 102 into the profile, chosen by unit type, service, UHS and position kind (MAP-3) |
| 35 — Licence notices | `m35` | ORDO's licence, its source code and the notices of all shipped third-party parts, linked from every screen (LIC-1) |
| 36 — Automatic incident log | `m36` | Every change with time, author and device; corrections as new entries; each person sees only what they may read (LOG-1, LOG-3, LOG-6) |
| 37 — UHS staff | `m37` | Choose the UHS one works at; only a UHS's staff (or roles allowed everywhere) write its patients, checked by the server (UHS-9, UHS-10, ADR-0056) |
| 38 — Map region from the server | `m38` | An administrator cuts the map region (centre, radius) on the server from the open world map (MAP-8, ADR-0057) |
| 39 — UHS among the resources | `m39` | Every UHS with its staff in *Einsatzmittel*; the incident command assigns staff (UHS-11) |
| 40 — Drawing on the map | `m40` | Areas and lines (incident area, hazard zone, cordon, …) drawn corner by corner at the map's centre, also by keyboard (MAP-4, ADR-0058, superseded by ADR-0064) |
| 41 — Multi-architecture image | `m41` | The server image for linux/amd64 and linux/arm64, each built natively and smoke-tested, signed as one (OPS-5) |
| 42 — Fuzzing | `m42` | The decoders of everything from the network fuzzed continuously with cargo-fuzz and ClusterFuzzLite (SEC-17, ADR-0059) |
| 43 — Device names | `m43` | Each device named at pairing (suggested from the browser), shown in the device list (ORG-8) |
| 44 — Display names | `m44` | People shown by name (from the identity provider, changeable) instead of their user ID (ORG-7) |
| 45 — Organisation roles | `m45` | Administrators grant and withdraw organisation roles, optionally until a date, in the app (ORG-9) |
| 46 — Local accounts | `m46` | Administrators create, reset and delete Dex's local accounts in the app (ORG-10, ADR-0060) |
| 47 — Field-near test | `m47` | The Compose bundle with two tablets (Android, iPad): propagation within 2 s, network cut, server restart, download and memory budgets (PERF-2, PERF-5) |
| 48 — Large incident | `m48` | The field-near test with an incident of 100 units, 50 sectors, 500 patients and 10,000 log entries, written by a load device: joining, incident log, propagation and memory on two tablets (PERF-3) |
| 49 — Workspaces by phase | `m49` | The incident command's pages grouped by phase with an overview first; administration apart from the incident (ADR-0061, UX-1) |
| 50 — Spoken orders | `m50` | Orders given by voice, radio or phone recorded by the crew and confirmed by their giver; who ordered a documented medication only from an order (ADR-0053, ORDER-6, ORDER-7) |
| 51 — Map region per incident | `m51` | Each incident has its own map region, cut by its incident command and read only by its devices with a ticket (ADR-0062, MAP-8) |
| 52 — Upstream login providers | `m52` | Administrators connect the organisation's OpenID Connect providers to Dex in the app, after a test login (ADR-0063, ORG-11) |
| 53 — Devices in the incident | `m53` | The incident command sees the devices of the people in its incident and blocks a lost one, without the organisation's device list (ADR-0061, ORG-12) |
| 54 — Map editor | `m54` | The map is edited on its own page with a toolbar: symbols placed and corners set by a tap or at the map's centre, drawings reshaped by dragging; a UHS's place is its symbol on the map (MAP-1, MAP-4, UHS-1, ADR-0064) |
| 55 — Feedback of 2026-11-12 | `m55` | Header menus close on a click outside; licences and versions under *Administration – Info* instead of a footer; the map edited in *Vorbereitung* and kept offline by default; the app shell cached per version; address search on the map |
| 56 — Command overview and preparation | `m56` | *Führung – Übersicht* as a grid of tiles; *Vorbereitung – Lagekarte* with the map region and the editor on one page; incident description, planned times and pause |
| 57 — UHS and units | `m57` | Units planned per incident, without organisation-wide master data (ADR-0070); status and orders in *Führung*, one post per person (UNIT-8); the UHS as a unit, its capacity per triage category and its places; strength after DV 102; the command structure as an org chart (roadmap item 6) |
| 58 — Patient record fields | `m58` | A diagnosis with its certainty, kept in the record (EPCR-11); other answers as free text (EPCR-12); the pupils as choices (EPCR-13) (roadmap item 8) |
| 59 — Helper register | `m59` | Helpers of an incident with personal data in the compartment `people`, read by the incident command (HELP-1), linked to accounts (HELP-2); crews from the register (HELP-3) (roadmap item 9, ADR-0071) |

To add a requirement to the current milestone, add the milestone tag to it **and** to
the design items that cover it. Requirements and design items must always carry at
least a priority tag, so that items outside the milestone are not traced by accident.

Run locally (needs Java 17 or newer):

```sh
java -jar openfasttrace-4.11.0.jar trace -t _,m0,m1,m2,m3,m4,m5,m6,m7,m8,m9,m10,m11,m12,m13,m14,m15,m16,m17,m18,m19,m20,m21,m22,m23,m24,m25,m26,m27,m28,m29,m30,m31,m32,m33,m34,m35,m36,m37,m38,m39,m40,m41,m42,m43,m44,m45,m46,m47,m48,m49,m50,m51,m52,m53,m54,m55,m56,m57,m58,m59 docs platform modules packages tests
```
