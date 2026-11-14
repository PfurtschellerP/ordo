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

1. Tactical symbols: generated per DV 102 with `@taktische-zeichen/core` (MIT) into the
   profile (decided 2026-11-10; the official BBK set of 2024 is a PDF whose terms allow
   distribution only free of charge). Open: compare the generated symbols with the BBK's
   recommendations before field use; which states have introduced FwDV/DV 102?
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
9. Triage charts: may ORDO, or an organisation, keep a local copy of the official PRIOR
   chart (BBK, DGKM) and the mSTaRT chart (Berufsfeuerwehr München, LMU) for offline use?
   Neither states a licence; ask the publishers.

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
header; no version means the oldest, one outside the range is refused with the range; MANV
patient flow (milestone `m25`): patients move between units (collection point, treatment
area, UHS) with time and person, shown as the record's *Patientenweg*; counts follow the
current unit, and the overview adds all units up; triage references (milestone `m26`,
ADR-0045): the profile names the algorithms (DE: PRIOR, mSTaRT) with publisher, edition
and source, shown as *Sichtungshilfe* next to the triage input; the algorithm used is
recorded with every triage, the category stays the person's choice; post-quantum transport
(milestone `m27`, ADR-0042): the Compose bundle's TLS endpoint offers the hybrid key
exchange X25519MLKEM768 first, checked in CI; ADR-0042 lists every algorithm and how each
is replaced; bills of materials (milestone `m28`, ADR-0043): CycloneDX for the server and the
web app on every build, checked in CI; units complete (milestone `m29`,
UNIT-1…6): units added per incident (a vehicles file of the installation was removed again,
ADR-0074), with service and strength in tactical notation, status since when, crew
with qualifications, release and return, and the strength on scene per organisation and
service; UHS details (milestone `m30`,
UHS-1): location, capacity and assigned staff per unit, also in the overview; tasks (milestone `m31`,
TASK-1…4): the command gives units tasks (free text or a template of the profile), moves
them on, and follows up; open tasks are at the top of every command view, due follow-ups
first, and on each unit's card; command structure (milestone `m32`,
CMD-1…4, ADR-0031): positions as incident data with kinds from the profile (FwDV 100 and
the medical response), leaders and logged handovers, units and UHS under positions, the
tree in the view *Führung* with the roster, span of control pointed out, tasks to
positions; situation map (milestone `m33`,
MAP-1, MAP-2, ADR-0006 accepted): MapLibre GL JS with a PMTiles region of OpenStreetMap
(`ORDO_MAP_FILE`, kept on the device for use without network), the incident location,
units, UHS and positions placed at the map centre (also by keyboard) and listed beside the
map; the command workspace's first view is now *Lage* (map and patient counts).

*Incident and command (Einsatzleitung)*

- **Incidents and roles, next steps** ([ADR-0046](adr/0046-roles-granted-per-incident.md)):
  templates for recurring deployments; roles from the profile instead of a fixed list;
  suggest registered people when granting roles. (Organisation roles are granted in the
  app since m45, ORG-9.)
- **Sectors (Einsatzabschnitte):** incident command, sectors and sub-sectors with their
  leaders (Einsatzabschnittsleiter), staff functions and liaison officers, per FwDV 100 and
  the state profile; shown as an organisation chart with the official tactical symbols.
  Medical branch first (ADR-0031).
- **Incident log (Einsatztagebuch) per DV 100:** timed entries with author, images and
  files viewable in place, immutable (events cannot change; corrections are new entries,
  hash-chained and signed). Legal requirements to be checked before building: state
  archive law, evidence value, qualified time stamps (eIDAS), retention.
- **One clinical compartment per UHS:** each UHS's patients in their own compartment, so
  devices also *receive* only their UHS's patients (need to know); the staff rule is
  enforced for writing since [ADR-0056](adr/0056-uhs-staff-write-guard.md). Needs an ADR
  for moves between UHS and the medical lead's overview.
- **Situation map, next steps:** symbols for drawn objects and staff functions, unit sizes
  and capacities on symbols (open points of the BBK comparison); **map editor with
  geofencing**: sectors, hazard and restricted areas, collection points as shapes (GeoJSON, RFC 7946; drawing with an open-source plug-in such as Terra Draw);
  a geofence reports when a unit or person enters or leaves. Every change is an event.
- **Positions** of units and people with call signs on the map. Browsers report positions
  only while the app is open (iOS: foreground only); continuous tracking needs a native
  shell or the BOS digital radios' positions. Only during the deployment, visible to the
  person, limited retention.
- **Register personnel by scanning the Meldekarte** ([meldekarte.de](https://meldekarte.de),
  run by volunteers of the BRK Kreisverband Augsburg-Land): done from a photo (HELP-5,
  ADR-0072), with the format read from a sample card. Open: confirm the format with the
  operators (how a `;` inside a field is written, whether it changes), scanning several
  cards in a row with live video.
- **Interfaces:** Rescuetrack (vehicle positions and status), DIVERA 24/7 (forces on foot,
  availability), and the status from BOS digital radio, so a status set on the radio
  appears in ORDO (via the control centre's interfaces or the organisation's own radios;
  to be clarified).

*Medical (MANV, UHS)*

- **MANV, next steps:** pre-triage apart from triage (with the algorithm used, see
  triage reference cards), a registration flow for the collection point (scan, category,
  next), transport organisation in the MANV with the hospital list; the Med workspace
  serves the MANV (collection point, treatment area and UHS are kinds of units), so no
  workspace of its own unless practitioners need one.
- **Transports, next steps:** later, not a priority
  ([ADR-0050](adr/0050-hospital-capacity-sources.md), proposed): capacity from IVENA
  eHealth once an organisation's agreement covers it; a hospital list kept by the
  organisation and imported (InEK site ID; HL7 FHIR or EDXL-HAVE) instead of entered per
  incident.
- **Orders, next steps:** substances from the medication catalogue (below). Spoken
  orders ([ADR-0053](adr/0053-verbal-orders.md)) are accepted: only physicians give
  and confirm orders, and ORDO keeps no SOP catalogue (2026-11-13). Open is a check of the
  fields and of the first users' state law with practitioners and a medical director
  (ÄLRD).
- **Triage references, next step:** a local copy of the official charts for offline use,
  once the rights are clear (open question 9).
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

Now — from the user test with two testers (2026-11-11), plan approved the same day:

1. **Quick fixes:** log out (unpair) from the incident list too, not only inside an
   incident; names of local accounts known before their first login (from Dex); rename a
   UHS; icons instead of the group words in the search; organisation chosen from a list
   when adding a unit, and the service "Zivilschutz" in the DE profile; the situation map
   opens on the incident's area; the current location by GPS.
2. **Propagation in a large incident** is done: the replica decodes and decrypts each
   event once and shares the list with the views (dsn~decoded-events~1). In the large
   incident a change now reaches the other tablet in 160–350 ms (before 1.5–2.1 s), a
   fresh tablet shows all 500 patients in 0.7 s (before 2.4 s). Next: hold each stream's
   decoded events once instead of twice (WebAssembly memory rose from 50 to 60–73 MB),
   then projections updated per event instead of recomputed.
3. **Menu structure** ([ADR-0061](adr/0061-workspaces-by-phase-and-administration-apart.md)):
   *Einsatzleitung – Führung* for the running incident (overview with a card per page and
   quick actions, situation map, dispatch with the orders, transports), *Vorbereitung*
   for what is set up before and adjusted during it (incident with location, map region
   and start; units; people; command structure), and the incident log, open to everyone.
   *Administration* replaces "Organisation": access (accounts, devices, organisation
   roles renamed "Berechtigungen"), later master data. The incident command does not see
   Administration; every device used in an incident can be blocked from the incident's
   views, the full device list stays with Administration. The menu structure is done
   (m49); **devices in the incident** are done (m53, ORG-12): the incident command sees
   them on *Vorbereitung – Personal*, connected or not, and blocks a lost one. **A map region per incident** is done (m51, ADR-0062): the incident command
   cuts it on the incident's page, and only the devices of that incident read it, with a
   ticket from their sync connection.
4. **Upstream login** configured in ORDO is done (m52,
   [ADR-0063](adr/0063-upstream-login-configured-in-ordo.md)): administrators add
   OpenID Connect providers in *Administration – Zugriff – Anmeldung*, after a test login.
5. **Situation map** is done (m54, m55): edited on its own page with a toolbar
   ([ADR-0064](adr/0064-map-editor-with-terra-draw.md)), opened from *Vorbereitung –
   Einsatz* and kept offline by itself
   ([ADR-0065](adr/0065-map-editing-in-preparation.md)); a UHS's place is its symbol; an
   address search through the server's Nominatim
   ([ADR-0066](adr/0066-address-search-with-nominatim.md)). Other map types (basemap.de,
   aerial images per state) are deferred by the product owner (2026-11-12).
   Since m56 ([ADR-0067](adr/0067-command-overview-tiles-and-map-preparation.md)): the
   map is prepared on *Vorbereitung – Lagekarte* (region and editor on one page), the
   *Führung* overview is a grid of tiles, and an incident has a description, planned times
   and a pause.
6. **UHS and units** is done (m57): units are incident data, planned per incident in
   *Vorbereitung – Einsatzmittel* with their crews, without organisation-wide master data
   (the product owner, 2026-11-13); their status is set and their orders given in *Führung – Disposition*
   ([ADR-0070](adr/0070-units-planned-in-preparation-led-in-command.md)); a person added
   to a second crew or UHS is moved, and a double booking made offline is marked (UNIT-8); a UHS counts as a unit in the
   forces on scene, with organisation, service and strength (UHS-12,
   [ADR-0069](adr/0069-uhs-counts-as-a-unit.md)); a UHS's capacity per triage category with
   places kept for each, and a plan of its places with the patient on each (UHS-13,
   UHS-14); the detailed strength after the 2024 BBK recommendation for DV 102 as an option
   per incident (UNIT-7: leaders of formations and platoon leaders apart, special
   functions such as physicians and respirator wearers after it); the command structure
   drawn as an org chart (CMD-3).

From a practitioner's test of the same version (2026-11-11), before the map editor:

7. **Orders only by those who give them** is built (m50; ADR-0053 accepted on 2026-11-13:
   only physicians give and confirm orders, no SOP catalogue): the crew records an order
   given by voice, radio or phone
   with who gave it, how and whether it was read back;
   its giver confirms it later, and the UHS overview counts the unconfirmed ones. A
   documented medication keeps its order, and "Angeordnet durch" is filled only from it.
8. **Patient record (profile)** is done (m58): a working, suspected or confirmed
   diagnosis in its own form, kept in the record and never in the transport request
   (EPCR-11; it reaches the crew that carries the patient with the handover, EPCR-6, and
   later the vehicles' workspace of item 11); "Sonstiges" with free text for breathing and
   skin as FHIR `open-choice` (EPCR-12); the pupils as choices after DIVI MIND: equal or
   not, size and reaction to light per side, consensual reaction (EPCR-13).
9. **Helper register** (*Vorbereitung – Personal*,
   [ADR-0071](adr/0071-helper-register-in-its-own-compartment.md)): parts 1 and 2 are
   done (m59), and part 3 too: helpers with name, organisation, phone, address, date of birth,
   qualifications, diet (for catering), duty and ORDO account in the sealed compartment `people`, which only the
   incident command reads (HELP-1, HELP-2); crews taken from the register, double bookings
   matched by helper and account (HELP-3); the helpers' data deleted on the server and on
   every device 90 days after the incident is closed (HELP-4, confirmed by the product
   owner), the first deletion by retention in ORDO; helpers registered from a photo of
   their Meldekarte, read on the device (HELP-5,
   [ADR-0072](adr/0072-registration-cards-read-with-rqrr.md); format from a sample card;
   read in attempts since photos of a screen failed, diet kept,
   [ADR-0073](adr/0073-registration-cards-read-in-attempts-diet-kept.md)). A card is a
   convenience: one that does not fit is not taken, and the helper is typed in.
10. **Dispatch** (*Führung – Disposition*): the incident command records reported events
    and patient cases (place, short description, no clinical data) and assigns them to a
    unit, which turns them into a patient (ADR).
11. **Map:** every tactical symbol of the profile placeable, chosen from a menu grouped by
    topic. **UHS:** request a physician or paramedic, who then gets access to the UHS.
    **Vehicles and medical teams** (*Sanitätstrupp*): a workspace for crews to see,
    confirm and carry out their transports and outstanding orders and to set their own
    status, built on the units board of *Führung – Disposition* (ADR-0070).
12. **Hospital capacities from the state's register** (e.g. ZLB) once its interface and an
    agreement are available (ADR-0050, regional profile).

Blocked or later: units and status from Rescuetrack and BOS digital radio (vendor interfaces, see below).

Then:

1. **Dependency analysis** (the product owner, 2026-11-13; before the first release, with
   the architecture review): for every dependency of the server, the app and the build,
   whether ORDO needs it (fewer is better: smaller app, smaller attack surface), how
   actively it is maintained (releases, open advisories, number of maintainers, response
   to issues) and what replaces it if it is abandoned. `cargo deny` already refuses known
   advisories and unmaintained crates; this adds the judgement before they get there
   (e.g. rxing was not taken for registration cards because it pulls in the unmaintained
   `encoding`, ADR-0073). Recorded per dependency, reviewed each release.
2. Security hardening (ASVS 5.0 level 2, selected level 3 requirements for keys and
   clinical data): hybrid post-quantum key grants and signatures as planned in ADR-0042;
   the first release (signed image, provenance, bills of materials: the release workflow
   is ready, ADR-0043; images for amd64 and arm64 since 2026-11-11); later the device lock with
   WebAuthn PRF or PIN (SEC-13).
3. Login, step 3 ([ADR-0038](adr/0038-login-and-device-pairing.md)) is done: device
   names (ORG-8), display names (ORG-7), organisation roles with "valid until" (ORG-9)
   and Dex's local accounts managed in ORDO (ORG-10, ADR-0060). Later: people change
   their own password (needs the link from an ORDO user to their Dex account; Dex can check the old password with `VerifyPassword`).
4. Field-near test is done (m47, `tests/e2e/field/`, weekly in CI): the Compose bundle
   with an Android tablet and an iPad, network cut and server restart, budgets for download
   and memory. It found that the image lacked the web app, that the bundle did not start
   with current Compose, and that a vanished network went unnoticed (now: ping and
   reconnect). The large incident (PERF-3, m48) runs in the same setup: a load device
   writes 10,000 events; fresh tablets show all 500 patients and the log within 2.5 s.
   It found that a change then takes 1.5–2.1 s to reach the other tablet (80 ms in a
   small incident) and that the app's memory grows to about 50 MB: next are incremental
   projections on the device (see "Scaling" below). Later: the test on real tablets.
5. Upstream login configured in ORDO is done (m52, ADR-0063). Later: LDAP and SAML
   providers in the app; a permission of its own instead of `manage_accounts`.
6. Master data in an admin view instead of a file (ORG-1); no organisation-wide vehicle
   master data (product owner, 2026-11-13, ADR-0070).
7. Design the main tablet screens for the Med MVP with volunteers (the patient list is
   now a list of cards; check forms and the record page with medics).
8. Command structure, next steps (ADR-0031): positions give roles and so access (a
   treatment-area lead reads its UHS), one tree per organisation linked by joint command,
   drag and drop on tablets, sectors on the map.

Later, after the features above:

- Incident templates: start an incident from an earlier one or a template with its map,
  units, command structure and roles, e.g. for a city festival held every year (product
  owner, 2026-11-13).

- Offline device approval: an already paired person with the right role approves a new
  device directly, without any identity provider — like a coach substituting a player at
  half-time ([ADR-0039](adr/0039-dex-in-every-installation.md)).

Scaling and availability ([architecture 7.6](architecture/07-deployment-view.md#76-performance-and-availability)),
before the first real field use:

- Incremental projections and streams loaded on demand (bounded memory).
- Group commit for higher write throughput.
- PostgreSQL storage backend for several instances and high availability (OPS-9).
- Backup and restore including the master key (OPS-3).
