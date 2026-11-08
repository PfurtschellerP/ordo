---
status: accepted
date: 2026-11-08
---

# Use existing open-source building blocks before building our own

## Context and Problem Statement

ORDO is a volunteer project with limited time. Every component we build ourselves must
be designed, tested, documented and maintained for years. Good open-source platforms,
tools and frameworks already exist for many needs. Early prototypes nevertheless
contained home-grown parts (a browser test driver, hash routing, accessibility checks).

## Considered Options

* Build what we need, use libraries only for hard problems
* Reuse first: use a maintained open-source project wherever it fits; build only what nothing covers

## Decision Outcome

Chosen option: "Reuse first", together with [ADR-0022](0022-open-standards-first.md)
(open standards first).

Rule: before building a component, tool or format, look for an existing open-source
project that fits the use case. Use it when it

* covers the need without bending ORDO's architecture (offline-first, need-to-know,
  compartments, accessibility),
* is actively maintained, with releases and more than one maintainer or a backing
  foundation or company,
* has an AGPL-compatible licence (checked by cargo-deny for Rust; reviewed for other
  ecosystems), and
* fits the resource budget (single binary, small devices, WebAssembly bundle size).

Build our own only when nothing fits, and record why in an ADR. Keep own components
small and replaceable behind clear interfaces.

### Consequences

* Good, because effort goes into what makes ORDO different: compartments, offline sync,
  emergency workflows.
* Good, because contributors meet familiar tools.
* Bad, because dependencies need watching (Dependabot, cargo-deny, licence review).

## More Information

Review of existing home-grown parts (November 2026):

| Part | Existing open-source option | Decision |
|---|---|---|
| Browser test driver (own Marionette client) | Playwright (Apache-2.0) | Replaced, as planned in architecture 8.6.2; also adds Chromium and WebKit |
| Contrast and touch-target checks | axe-core (MPL-2.0) | Replaced; only checks axe does not have stay (44 px targets, reflow, bars covering content) |
| Hash routing in the app | leptos_router (MIT), part of Leptos | Replaced |
| Percent-encoding | `percent-encoding` (MIT/Apache-2.0) | Replaced |
| Policy enforcement (planned) | Cedar (Apache-2.0, CNCF) | Use ([ADR-0035](0035-authorization-with-cedar.md)) |
| Device storage (`localStorage` snapshot) | SQLite in WebAssembly on OPFS; IndexedDB | Replace ([ADR-0034](0034-device-storage-sqlite-opfs.md)) |
| Binary attachments | `object_store` (Apache-2.0); S3-compatible stores such as Garage | Use ([ADR-0033](0033-attachments-in-an-object-store.md)) |
| ePCR form format (own TOML) | HL7 FHIR Questionnaire | Evaluate next; would also allow existing form editors |
| Design-token build (`ordo-tokens`) | Style Dictionary, Terrazzo | Keep for now: small, Rust-only build, standard format, swappable |
| Sync protocol | CRDT libraries (Automerge, Loro), Matrix, Postgres sync engines | Keep own protocol: none covers server-ordered, sealed, need-to-know compartments ([ADR-0005](0005-own-sync-protocol.md), architecture 8.4.1); reuse their building blocks (CBOR, Ed25519, …) |
| Fuzzy matching in the command palette | `nucleo`, `fuzzy-matcher` | Keep: about 40 lines; a library would grow the WebAssembly bundle |
