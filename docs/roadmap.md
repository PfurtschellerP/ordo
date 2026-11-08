# ORDO Roadmap and open questions

> Moved from the requirements. Phases are planning, not commitments.

## Roadmap (draft)

| Phase | Content |
|---|---|
| **0 — Foundation** | Repository, CI, container image, Docker Compose, architecture decisions, platform core (events, sync, compartments, modules, profiles, global IDs, signed events), sync prototype, first spec drafts, design tokens and themes, accessibility testing in CI |
| **1 — MVP (Med)** | Shared modules (incidents, units, command, tasks, map, log, org) + **Med package: patients and triage, UHS, transport planning, hospitals** (no ePCR yet); planned operations; offline; one organisation; field tests at planned events |
| **2 — Fire** | Fire package; first connectors between Fire and Med; field tests with fire brigades |
| **3 — Patient records** | ePCR with clinical compartment, PDF handover; data-protection review |
| **4 — Connected** | Federation between installations, alert import, local server in the command vehicle, breathing-apparatus monitoring, conformance tests |
| **5 — Professional** | Single sign-on, Helm chart, hosted service, control-centre interfaces (EMSI), hospital interfaces (FHIR, IVENA), Matrix adapter, police modules with partners |

## Open questions

1. Which open tactical-symbol set (DV 102) can be used, and under which licence?
2. Which organisations in Germany could be early test partners — first medical services
   for planned events (e.g. DRK, ASB, JUH, MHD), then fire brigades?
3. Which DIVI MIND version should the ePCR forms align with field by field (7.0?), and is
   its use licence-free?
4. Who can review data protection and medical-device questions before field use?
5. Should larger tablets in portrait also use the bottom bar (breakpoint now 48 rem)?
6. Should the ePCR forms move to the HL7 FHIR Questionnaire format, so existing form
   editors can be used ([ADR-0032](adr/0032-reuse-open-source-first.md))?

## Next steps

Done in phase 0:

- Workspace, CI, specs (event envelope, sync protocol draft 0), design tokens and themes.
- Sync logic with compartments, sealed payloads, units and patients modules, server-derived
  patient counts; acceptance tests on a simulated network (milestone `m1`).
- Server `ordo serve`: WebSocket, SQLite, health/readiness/metrics, wrapped keys, restart
  without data loss; container image and minimal Docker Compose setup (milestone `m2`).

- First web app (milestone `m3`): pairing, Med workspace (registration, UHS board,
  re-triage), command workspace (patient counts), offline with reload, served by
  `ordo serve`; UI scenario in headless Firefox in CI.

- Several UHS with load overview (milestone `m4`); resources (vehicles, teams) with
  status in their own workspace; audit of the rendered app (contrast, layout, touch
  targets, reflow) in all themes.

- Navigation: workspace launcher with role-based availability, view rail or bottom bar
  (milestone `m5`, ADR-0025); test directories moved to conventional places.

- Patient care record (milestone `m6`): forms from the DE profile (identity, encounter,
  history, vital signs, findings, measures, medication, disposition) with corrections,
  photo for identification, master–detail editing without modal dialogs, routes with
  back-button support; Ctrl+K command palette with quick actions; menu in the bottom bar
  on phones; patient and unit lists as cards; [standards register](standards.md) and a
  proposal for the command structure (ADR-0031).
- Decisions: command structure follows the doctrine, medical branch first and compatible
  with FwDV 100; access follows roles, never people (ADR-0031); roles and permissions as
  Cedar policies (ADR-0035); shortcuts and command names the same in every language
  (ADR-0036); open source first (ADR-0032).
- Reuse instead of own code: Leptos' router (paths, back button), Playwright with
  axe-core for the browser tests in Firefox, Chromium and WebKit.

Next:

1. Login and roles: OpenID Connect with an existing identity provider; pairing by QR code
   with the OAuth 2.0 Device Authorization Grant (RFC 8628) instead of the seed file
   (ORG-3); authentication of the connection (sync protocol draft 1); roles and
   permissions as Cedar policies (ADR-0035); protected key storage in the app. The login
   ADR also covers working offline (cached tokens, field server without internet).
2. Storage: device storage as SQLite on OPFS in a worker, after a short spike
   ([ADR-0034](adr/0034-device-storage-sqlite-opfs.md)); attachments as encrypted,
   content-addressed blobs in an object store ([ADR-0033](adr/0033-attachments-in-an-object-store.md)).
3. Field-near test: two tablets and a server via Docker Compose, network on/off; measure
   bundle size and memory.
4. Complete resources and UHS (UNIT-1…6, UHS-1): units from master data, crew strength,
   status codes from the DE profile, releasing units, UHS staff and location.
5. Design the main tablet screens for the Med MVP with volunteers (the patient list is
   now a list of cards; check forms and the record page with medics).
6. Command structure as data, medical branch first (ADR-0031).

Scaling and availability ([architecture 7.6](architecture/07-deployment-view.md#76-performance-and-availability)),
before the first real field use:

- Incremental projections and streams loaded on demand (bounded memory).
- Group commit for higher write throughput.
- PostgreSQL storage backend for several instances and high availability (OPS-9).
- Backup and restore including the master key (OPS-3).
