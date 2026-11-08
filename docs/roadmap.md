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

1. Tactical symbols: decided — ORDO ships the official BBK symbols (2024) with the source
   named. Open: which states have introduced FwDV/DV 102?
2. Which organisations in Germany could be early test partners — first medical services
   for planned events (e.g. DRK, ASB, JUH, MHD), then fire brigades?
3. Which DIVI MIND version should the ePCR forms align with field by field (7.0?), and is
   its use licence-free?
4. Who can review data protection and medical-device questions before field use?
5. Should larger tablets in portrait also use the bottom bar (breakpoint now 48 rem)?
6. Which open-source FHIR form editor fits for profile authors (licence, maintenance)?
7. Governance: should a legal entity (foundation, association) support ORDO, and thereby
   become its open-source steward under the Cyber Resilience Act? A lawyer should review
   [ORDO's CRA role](standards.md#ordos-role-under-the-cyber-resilience-act) before the
   first release.
8. State profiles: find the missing sources (Berlin, Hamburg, Thuringia; status codes,
   MANV concepts and medical command structure per state), see
   [German state sources](standards-de-states.md).

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
- Access timeout (milestone `m9`, ADR-0040): a device login ends 48 hours (1–30 days)
  after the last verification; the server renews it in the background with the identity
  provider's renewal token, ends it at once on a definitive refusal, refuses and cuts
  expired devices; the device locks itself, also offline, and its person logs in again
  with the same key.
- Device revocation (milestone `m9`, SEC-4): everyone sees and revokes their own devices,
  admins (Cedar permission `manage_devices`) all devices; a revoked device is cut off at
  once, refused from then on, and deletes its key and data when it connects (remote
  wipe). Unpairing revokes, too.
- Encryption at rest on devices (milestone `m10`, ADR-0041): pairing, device key and
  replica are stored encrypted with a storage key wrapped by a non-extractable WebCrypto
  key; unpairing and revocation destroy it (crypto-shredding).
- Login, step 2 (milestone `m8`): OpenID Connect login (ORDO server as relying party,
  PKCE), QR pairing of shared tablets (RFC 8628), identity-provider groups as roles, paired
  devices stored; Dex in the browser tests and, optionally, in Docker Compose. The browser
  test pairs both devices for real.
- Reuse instead of own code: Leptos' router (paths, back button), Playwright with
  axe-core for the browser tests in Firefox, Chromium and WebKit; ePCR forms as an HL7
  FHIR Questionnaire (ADR-0037).

Next — feedback of practitioners on the screenshots (2026-11-08), Med first:

- MANV workspace: Vorsichtung and Sichtung, patient collection point, treatment area,
  transport organisation, overview for the medical command (LNA/OrgL).
- "Betroffene" (unharmed, affected people) as a category of their own.
- Transport priority for SK I and SK II, documented separately from the triage category
  (Bavaria, NRW and Austria decouple them); which categories may get it, and its levels,
  come from the state or country profile.
- Treatment places within a UHS (Behandlungsplätze); patients are assigned to places.
- Triage catalogues and algorithms (PRIOR, mSTaRT, …): constraint "no clinical decision
  support" (EU MDR) — see the open decision in the reply of 2026-11-08.
- Situation map (Lagekarte) with the official tactical symbols (ADR-0006: MapLibre,
  PMTiles); positions of units and people with call signs. Browsers report positions only
  while the app is open (iOS: foreground only); continuous tracking needs a native shell
  or the positions of the BOS digital radios. Privacy: only during the deployment, visible
  to the person, limited retention.
- Vital signs from patient monitors (corpuls C3 first): interface to be clarified with
  the manufacturer; open standards to prefer: IEEE 11073 SDC, HL7 FHIR. Values are
  documented, never interpreted (no alarms).

Then:

1. Security hardening (ASVS 5.0 level 2, selected level 3 requirements for keys and
   clinical data): post-quantum plan with crypto agility and hybrid TLS check (ADR-0042);
   supply chain with SBOM (CycloneDX), build provenance, signed images and OpenSSF
   Scorecard (ADR-0043); later the device lock with WebAuthn PRF or PIN (SEC-13).
2. Login, step 3, rest ([ADR-0038](adr/0038-login-and-device-pairing.md)): device names
   in the device list; an admin view for roles, "valid until" and Dex's local accounts.
3. Storage: device storage as SQLite on OPFS in a worker, after a short spike
   ([ADR-0034](adr/0034-device-storage-sqlite-opfs.md)); attachments as encrypted,
   content-addressed blobs in an object store ([ADR-0033](adr/0033-attachments-in-an-object-store.md)).
4. Field-near test: two tablets and a server via Docker Compose, network on/off; measure
   bundle size and memory.
5. Complete resources and UHS (UNIT-1…6, UHS-1): units from master data, crew strength,
   status codes from the DE profile, releasing units, UHS staff and location.
6. Design the main tablet screens for the Med MVP with volunteers (the patient list is
   now a list of cards; check forms and the record page with medics).
7. Command structure as data, medical branch first (ADR-0031).

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
