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

Next — backlog from the practitioners' feedback (screenshots and demo, 2026-11-08/09).

The storage work is done (milestone `m13`); next comes the rest of this backlog.

**Done already:** palette quick actions bring the opened form into view with the cursor
in its first field; the app is installable as a PWA (it lacked icons); incidents are
planned ahead and roles granted per incident (milestone `m11`, ADR-0046): after logging
in, people see the incidents they have a role in, planners plan new ones, the incident's
command manages the roster under *Einsatzleitung – Personal*; profile pictures, resolved
ORDO → identity provider → Gravatar by the server (ADR-0047); views by role: the
Einsatzleitung sees the whole incident (UHS load and counts, no clinical details), UHS crews
only their UHS work, units are changed by the incident command only; "Betroffene" as a
category of their own, transport priority from the profile, treatment places per UHS
(milestone `m12`); storage (milestone `m13`): attachments as encrypted blobs with
tickets in an object store (local directory), device data in SQLite on OPFS in a worker;
patient photos as attachments (a preview in the event, the full photo uploaded from a
queue and downloaded when shown); attachments optionally in Garage (S3-compatible) in the
Compose bundle and in CI; transport requests (milestone `m14`): the UHS requests a
transport on the record (type, priority, special needs from the profile), the incident
command assigns a unit and a destination in *Lage – Transporte* and follows the status;
the shared request carries no patient data; medical orders (milestone `m15`): physicians and
paramedics (new incident roles) give orders on the record, the crew acknowledges and
completes them, the view *UHS – Anordnungen* lists the open orders of every UHS; the
server accepts new orders only from these roles; header layout (milestone `m16`,
ADR-0049): menu, heading, search in the middle, connection and person on the right, the
person's settings in a profile menu, and a bottom bar for left-handed use; discharge
(milestone `m17`): the UHS discharges with an outcome (released on site, transported,
refused, deceased), "transported" chosen once the transport has left; readmission corrects
a mistake; the load counts only the patients in the unit, command sees how many left;
hospitals and the transport board (milestone `m18`): the command keeps a hospital list with
departments, capabilities and capacity per triage category (catalogues from the profile),
assigns transports to a hospital or a free-text destination, and sees in *Transporte* the
open requests, the free transport units and the patients sent to each hospital; the unit
status codes moved from the app into the profile (`units/status-codes.toml`); planned
transports (milestone `m19`): a request can name a later date and time, and the board
lists planned transports apart, the earliest first; person suggestions (milestone `m20`):
form fields that name a member of the crew (FHIR `reference` to `Practitioner`) suggest the
incident's people, and granting a role suggests the people the installation knows
(`listPeople`, only for those who grant roles); any other name is accepted; orders into
the record (milestone `m21`): a done medication order opens a new entry of the record's
medication form, filled with substance, dose, route and who ordered and gave it, for the
crew to check and save; the profile names the form and fields, and the build checks that
order routes and form routes match; overview for the medical lead (milestone `m22`): the
Med workspace's view *Übersicht* shows per UHS the patients by category against capacity,
free places, open orders, open and planned transports, and discharged patients; list keys
(milestone `m23`, ADR-0051): `j`/`k` and the arrow keys move through lists while they have
the focus, Ctrl+J/Ctrl+K through the palette; API versions (milestone `m24`, ADR-0052): the sync
protocol and the HTTP API each have a `major.minor` version (now `1.0`), announced in the
challenge and at `/.well-known/ordo`, asked for in `hello` and the `ORDO-API-Version`
header; no version means the oldest, one outside the range is refused with the range.

*Incident and command (Einsatzleitung)*

- **Incidents and roles, next steps** ([ADR-0046](adr/0046-roles-granted-per-incident.md)):
  templates for recurring deployments; roles from the profile instead of a fixed list;
  an admin view for organisation roles (planners); suggest registered people when
  granting roles.
- **Sectors (Einsatzabschnitte):** incident command, sectors and sub-sectors with their
  leaders (Einsatzabschnittsleiter), staff functions and liaison officers, per FwDV 100 and
  the state profile; shown as an organisation chart with the official tactical symbols.
  Medical branch first (ADR-0031).
- **Incident log (Einsatztagebuch) per DV 100:** timed entries with author, images and
  files viewable in place, immutable (events cannot change; corrections are new entries,
  hash-chained and signed). Legal requirements to be checked before building: state
  archive law, evidence value, qualified time stamps (eIDAS), retention.
- **Situation map (Lagekarte)** with the official symbols (ADR-0006: MapLibre, PMTiles);
  **map editor with geofencing**: sectors, hazard and restricted areas, collection points
  as shapes (GeoJSON, RFC 7946; drawing with an open-source plug-in such as Terra Draw);
  a geofence reports when a unit or person enters or leaves. Every change is an event.
- **Positions** of units and people with call signs on the map. Browsers report positions
  only while the app is open (iOS: foreground only); continuous tracking needs a native
  shell or the BOS digital radios' positions. Only during the deployment, visible to the
  person, limited retention.
- **Register personnel by scanning the Meldekarte** ([meldekarte.de](https://meldekarte.de),
  run by volunteers of the BRK Kreisverband Augsburg-Land): the card's QR code carries
  what is printed on it; scanning it adds the person to the incident's roster with arrival
  time and formation. The QR format is not published — get a sample card or the format
  from the operators. Decoding with an open-source library in the app (e.g. rxing,
  Apache-2.0), camera via the browser.
- **Interfaces:** Rescuetrack (vehicle positions and status), DIVERA 24/7 (forces on foot,
  availability), and the status from BOS digital radio, so a status set on the radio
  appears in ORDO (via the control centre's interfaces or the organisation's own radios;
  to be clarified).

*Medical (MANV, UHS)*

- **MANV workspace:** pre-triage and triage, patient collection point, treatment area,
  transport organisation, overview for the medical command (LNA, OrgL).
- **Transports, next steps:** later, not a priority
  ([ADR-0050](adr/0050-hospital-capacity-sources.md), proposed): capacity from IVENA
  eHealth once an organisation's agreement covers it; a hospital list kept by the
  organisation and imported (InEK site ID; HL7 FHIR or EDXL-HAVE) instead of entered per
  incident.
- **Orders, next steps:** substances from the medication catalogue (below); orders given
  by voice or radio recorded with the person who gave them.
- **Triage algorithms** (PRIOR, mSTaRT, …) as reference cards from the profile; the person
  chooses the category, the algorithm used is recorded ([ADR-0045](adr/0045-triage-algorithms-as-reference.md)).
- **Medication catalogue:** fields for substances suggest from a catalogue of substances
  used in emergency medicine (open source and licence to be found), free text allowed.
- **Vital signs from patient monitors** (corpuls C3 first): interface to be clarified with
  the manufacturer; prefer IEEE 11073 SDC, HL7 FHIR. Values are documented, never
  interpreted (no alarms).

*App*

- **Display names** (ORG-7): people are shown by name instead of their user ID (roster,
  histories, orders, person fields, suggestions); the name comes from the identity
  provider at login (OIDC `name`) and can be changed by the person. Suggestions then show
  the name and keep the user ID as the value.
- **PDF/A exports with Typst** (Apache-2.0, embeddable, PDF/A): incident log, patient
  record for the handover (layout after the DIVI emergency protocol where no open
  standard exists), other printable views.

Then:

1. Security hardening (ASVS 5.0 level 2, selected level 3 requirements for keys and
   clinical data): post-quantum plan with crypto agility and hybrid TLS check (ADR-0042);
   supply chain with SBOM (CycloneDX), build provenance, signed images and OpenSSF
   Scorecard (ADR-0043); later the device lock with WebAuthn PRF or PIN (SEC-13).
2. Login, step 3, rest ([ADR-0038](adr/0038-login-and-device-pairing.md)): device names
   in the device list; an admin view for roles, "valid until" and Dex's local accounts.
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
