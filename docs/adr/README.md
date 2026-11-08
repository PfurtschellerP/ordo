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
| [0006](0006-maplibre-pmtiles.md) | MapLibre GL JS and PMTiles for offline maps | proposed | 2026-11-07 |
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
| [0027](0027-epcr-forms-from-the-profile.md) | Patient care record: forms from the profile, values as events | accepted | 2026-11-08 |
| [0028](0028-patient-photos-in-sealed-events.md) | Patient photos as small images inside sealed events | proposed | 2026-11-08 |
| [0029](0029-edit-records-in-place.md) | Edit records in place, not in modal dialogs | accepted | 2026-11-08 |
| [0030](0030-command-palette.md) | Command palette and keyboard shortcuts | accepted | 2026-11-08 |
| [0031](0031-command-structure-from-doctrine.md) | Command structure as data, following the official doctrine | proposed | 2026-11-08 |

To add a decision, copy the newest ADR, take the next number and set `status: proposed`.
