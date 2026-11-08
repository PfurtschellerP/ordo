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
6. Which open-source FHIR form editor fits for profile authors (licence, maintenance)?

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
- Login, step 1 (milestone `m7`, ADR-0038): every sync connection proves the device key
  (sync protocol draft 1); access by roles with Cedar policies, organisations add roles
  as policy files (ADR-0035).
- Dex in every installation; ORDO speaks only OpenID Connect, LDAP is not supported
  (ADR-0039); field-server guidance in the Compose bundle.
- Login, step 2 (milestone `m8`): OpenID Connect login (ORDO server as relying party,
  PKCE), QR pairing of shared tablets (RFC 8628), identity-provider groups as roles, paired
  devices stored; Dex in the browser tests and, optionally, in Docker Compose. The browser
  test pairs both devices for real.
- Reuse instead of own code: Leptos' router (paths, back button), Playwright with
  axe-core for the browser tests in Firefox, Chromium and WebKit; ePCR forms as an HL7
  FHIR Questionnaire (ADR-0037).

Next:

1. Login, step 3 ([ADR-0038](adr/0038-login-and-device-pairing.md)): device list and
   revocation (SEC-4), periodic re-login that never locks people out offline (due only
   when the identity provider is reachable, with a grace period), protected key storage in
   the browser (WebCrypto, non-extractable keys), an admin view for roles and for Dex's
   local accounts.
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

Later, after the features above:

- Offline device approval: an already paired person with the right role approves a new
  device directly, without any identity provider — like a coach substituting a player at
  half-time ([ADR-0039](adr/0039-dex-in-every-installation.md)).

Scaling and availability ([architecture 7.6](architecture/07-deployment-view.md#76-performance-and-availability)),
before the first real field use:

- Incremental projections and streams loaded on demand (bounded memory).
- Group commit for higher write throughput.
- PostgreSQL storage backend for several instances and high availability (OPS-9).
- Backup and restore including the master key (OPS-3).
