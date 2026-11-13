# Architecture Decision Records

Decisions in [MADR 4](https://adr.github.io/madr/) format, one file per decision.
A decision is never edited to mean something else: it is superseded by a new ADR.

| ADR | Decision | Status | Date |
|---|---|---|---|
| [0001](0001-local-first-event-sourcing.md) | Store data as events, local-first | proposed | 2026-11-07 |
| [0002](0002-rust-everywhere.md) | Use Rust for server and client | accepted | 2026-11-07 |
| [0003](0003-leptos-pwa.md) | Leptos as UI framework; PWA first | proposed | 2026-11-07 |
| [0004](0004-sqlite-default-postgresql-optional.md) | SQLite by default, PostgreSQL optional | proposed | 2026-11-07 |
| [0005](0005-own-sync-protocol.md) | Own sync protocol on WebSocket with CBOR | proposed | 2026-11-07 |
| [0006](0006-maplibre-pmtiles.md) | MapLibre GL JS and PMTiles for offline maps | accepted | 2026-11-10 |
| [0007](0007-licence-agpl.md) | Licence the software under AGPL-3.0-or-later | accepted | 2026-11-08 |
| [0008](0008-german-profile-first.md) | German (DE) regional profile first | accepted | 2026-11-07 |
| [0009](0009-layered-data-profiles.md) | Regional profiles as layered data packages | proposed | 2026-11-08 |
| [0010](0010-layered-modular-platform.md) | Platform, shared modules and service packages | proposed | 2026-11-08 |
| [0011](0011-workspaces-as-data.md) | Role-based workspaces defined as data | proposed | 2026-11-08 |
| [0012](0012-data-compartments.md) | Data compartments | proposed | 2026-11-08 |
| [0013](0013-connectors-as-events.md) | Connectors as versioned published events | proposed | 2026-11-08 |
| [0014](0014-clinical-trust-model.md) | Trust model for clinical data | proposed | 2026-11-08 |
| [0015](0015-cloud-ready-monolith.md) | Cloud-ready modular monolith | proposed | 2026-11-08 |
| [0016](0016-own-federation-protocol.md) | Own incident-scoped federation, inspired by Matrix | proposed | 2026-11-08 |
| [0017](0017-qualified-ids-signed-events.md) | Server-qualified IDs and signed events from day one | proposed | 2026-11-08 |
| [0018](0018-open-specifications.md) | Spec-first open specifications under permissive licences | accepted | 2026-11-08 |
| [0019](0019-med-package-first.md) | Med package first in the MVP | accepted | 2026-11-08 |
| [0020](0020-accessibility-target.md) | Accessibility target EN 301 549 / WCAG 2.2 AA | accepted | 2026-11-08 |
| [0021](0021-design-system.md) | Design system on W3C design tokens | proposed | 2026-11-08 |
| [0022](0022-open-standards-first.md) | Open standards first | accepted | 2026-11-08 |
| [0023](0023-docs-as-code.md) | Docs-as-code with arc42, MADR and OpenFastTrace | accepted | 2026-11-08 |
| [0024](0024-server-derived-connectors.md) | Connector events are derived by the server that orders the source compartment | proposed | 2026-11-08 |
| [0025](0025-workspace-navigation.md) | Navigation: workspace launcher plus view rail or bottom bar | accepted, amended by 0026 | 2026-11-08 |
| [0026](0026-navigation-at-the-bottom-on-phones.md) | Navigation at the bottom on phones, menu included | accepted | 2026-11-08 |
| [0027](0027-epcr-forms-from-the-profile.md) | Patient care record: forms from the profile, values as events | accepted, amended by 0037 | 2026-11-08 |
| [0028](0028-patient-photos-in-sealed-events.md) | Patient photos as small images inside sealed events | superseded by 0033 | 2026-11-08 |
| [0029](0029-edit-records-in-place.md) | Edit records in place, not in modal dialogs | accepted | 2026-11-08 |
| [0030](0030-command-palette.md) | Command palette and keyboard shortcuts | accepted, amended by 0036 | 2026-11-08 |
| [0031](0031-command-structure-from-doctrine.md) | Command structure as data, following the official doctrine | accepted | 2026-11-08 |
| [0032](0032-reuse-open-source-first.md) | Use existing open-source building blocks before building our own | accepted | 2026-11-08 |
| [0033](0033-attachments-in-an-object-store.md) | Attachments: encrypted, content-addressed blobs in an object store | accepted | 2026-11-09 |
| [0034](0034-device-storage-sqlite-opfs.md) | Device storage: SQLite on the browser's file system, in a worker | accepted | 2026-11-09 |
| [0035](0035-authorization-with-cedar.md) | Roles and permissions as Cedar policies | accepted | 2026-11-08 |
| [0036](0036-language-independent-shortcuts.md) | Shortcuts and command names are the same in every language | accepted | 2026-11-08 |
| [0037](0037-epcr-forms-as-fhir-questionnaire.md) | ePCR forms as an HL7 FHIR Questionnaire | accepted | 2026-11-08 |
| [0038](0038-login-and-device-pairing.md) | Login: OpenID Connect, device keys and QR pairing | accepted, amended by 0039 | 2026-11-08 |
| [0039](0039-dex-in-every-installation.md) | Dex in every installation; ORDO speaks only OpenID Connect | accepted | 2026-11-08 |
| [0040](0040-access-ends-only-on-positive-evidence.md) | Device logins expire after a timeout; unreachability alone ends nothing early | accepted | 2026-11-08 |
| [0041](0041-encryption-at-rest-on-devices.md) | Encryption at rest on devices: envelope encryption with a device-bound key | accepted | 2026-11-08 |
| [0042](0042-post-quantum-readiness.md) | Post-quantum readiness: hybrid key exchange now, replaceable algorithms throughout | accepted | 2026-11-10 |
| [0043](0043-supply-chain.md) | Supply chain: bills of materials on every build, signed releases with provenance | accepted | 2026-11-10 |
| [0044](0044-federation-across-profiles.md) | Federation across profiles: shared meanings, local labels | accepted | 2026-11-08 |
| [0045](0045-triage-algorithms-as-reference.md) | Triage algorithms are shown as reference, never computed | accepted | 2026-11-09 |
| [0046](0046-roles-granted-per-incident.md) | Roles are granted per incident in ORDO, not taken from identity provider groups | accepted | 2026-11-09 |
| [0047](0047-profile-pictures.md) | Profile pictures: ORDO, then the identity provider, then Gravatar | accepted | 2026-11-09 |
| [0048](0048-openspec-not-adopted.md) | OpenSpec is not adopted; requirements stay in OpenFastTrace | accepted | 2026-11-10 |
| [0049](0049-header-layout-and-handedness.md) | Header layout, profile menu and handedness | accepted | 2026-11-10 |
| [0050](0050-hospital-capacity-sources.md) | Hospital capacity: entered by hand now, interfaces (IVENA, EDXL-HAVE) later | proposed | 2026-11-10 |
| [0051](0051-vim-movement-keys.md) | Vim movement keys in lists and the command palette | accepted | 2026-11-10 |
| [0052](0052-api-versions.md) | API versions: `major.minor` per specification, negotiated per connection and request | accepted | 2026-11-10 |
| [0053](0053-verbal-orders.md) | Orders given by voice or radio are recorded by the crew and confirmed later | proposed | 2026-11-12 |
| [0054](0054-tactical-symbols-generated.md) | Tactical symbols: generated at build time with `@taktische-zeichen/core`, kept in the profile | accepted | 2026-11-10 |
| [0055](0055-licence-notices.md) | Licence notices: `cargo-about` for the crates, a list for the other parts | accepted | 2026-11-10 |
| [0056](0056-uhs-staff-write-guard.md) | A UHS's patients are written by its staff, checked by the server per event | accepted | 2026-11-11 |
| [0057](0057-map-region-cut-on-the-server.md) | Map regions are cut on the server from the open world map | accepted | 2026-11-11 |
| [0058](0058-drawing-at-the-map-centre.md) | Drawing on the map: corners set at the map's centre, no drawing library yet | superseded by 0064 | 2026-11-11 |
| [0059](0059-fuzzing-with-cargo-fuzz-and-clusterfuzzlite.md) | Fuzzing with cargo-fuzz, run continuously by ClusterFuzzLite | accepted | 2026-11-11 |
| [0060](0060-local-accounts-managed-in-ordo.md) | Local accounts are managed in ORDO, through Dex's API | accepted | 2026-11-11 |
| [0061](0061-workspaces-by-phase-and-administration-apart.md) | Workspaces by phase; administration apart from the incident | accepted; partly superseded by 0067 | 2026-11-11 |
| [0062](0062-map-region-per-incident.md) | A map region per incident | accepted | 2026-11-11 |
| [0063](0063-upstream-login-configured-in-ordo.md) | Upstream logins are configured in ORDO, through Dex's connector API, after a test login | accepted | 2026-11-12 |
| [0064](0064-map-editor-with-terra-draw.md) | The map is edited on its own page, with a toolbar, taps and Terra Draw | accepted; Page part superseded by 0065 | 2026-11-12 |
| [0065](0065-map-editing-in-preparation.md) | The map is edited in *Vorbereitung*, and kept offline by default | accepted; Page part superseded by 0067 | 2026-11-13 |
| [0066](0066-address-search-with-nominatim.md) | Addresses are searched with Nominatim, through the ORDO server | accepted | 2026-11-13 |
| [0067](0067-command-overview-tiles-and-map-preparation.md) | The command's overview is a grid of tiles, and *Vorbereitung* has its own *Lagekarte* | accepted | 2026-11-13 |

To add a decision, copy the newest ADR, take the next number and set `status: proposed`.
