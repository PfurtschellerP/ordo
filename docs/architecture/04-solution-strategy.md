# 4. Solution strategy

> Part of the [ORDO architecture](README.md) (arc42).

## 4.1 Key ideas

| Goal | Approach | Details |
|---|---|---|
| Works offline | Local-first devices; every change is an immutable event; sync exchanges missing events | [8.1](08-concepts.md#81-events-and-ordering), [ADR-0001](../adr/0001-local-first-event-sourcing.md) |
| Need to know | Data compartments decide what a device receives | [8.2](08-concepts.md#82-data-compartments) |
| Several services, one platform | Layers: platform → shared modules → service packages; connectors between packages | [5.1](05-building-block-view.md#51-level-1-layers), [8.3](08-concepts.md#83-connectors) |
| Trustworthy record | Signed events, hash chain per incident compartment | [8.1](08-concepts.md#81-events-and-ordering) |
| Resource-efficient | Rust everywhere, WebAssembly client, compact CBOR, one binary | [4.2](#42-technology-choices) |
| Accessible | DOM-based UI, design tokens with automatic contrast checks | [8.6](08-concepts.md#86-design-system-and-accessibility) |
| Federation later | Server-qualified IDs and signed events from day one; same protocol for devices and servers | [8.4](08-concepts.md#84-federation) |
| No lock-in | Open standards first; own formats published as open specifications | [4.3](#43-open-standards-in-use) |

## 4.2 Technology choices

| Area | Proposal | Reasoning | Alternatives |
|---|---|---|---|
| Language | **Rust** everywhere | Fast, low memory, no GC; one codebase for client and server | — (decided) |
| Client UI | **Leptos** (client-side rendered, WASM) | Fine-grained reactivity, no virtual DOM | Dioxus; Yew |
| Client delivery | **PWA** | One codebase for tablet/laptop/phone; no app store | — |
| Native shell | **Tauri 2** (later) | Rust-native; Android/iOS/desktop; secure key storage, native SQLite | Capacitor |
| Client storage | **SQLite in WASM** (OPFS), encrypted payloads | Transactional; same SQL as server | IndexedDB |
| Map | **MapLibre GL JS** + **PMTiles** (OpenStreetMap) via `wasm-bindgen` | GPU vector maps; offline regions | maplibre-rs (experimental) |
| Server | **axum** on **tokio** | Small, fast, WebSocket built in | actix-web |
| Database | **sqlx**: SQLite default, PostgreSQL optional | One crate for both | rusqlite + tokio-postgres |
| Serialisation | **serde** + **CBOR**; JSON for export | Compact and evolvable | postcard |
| Cryptography | **RustCrypto** / **ring**: XChaCha20-Poly1305, X25519, BLAKE3, Argon2id | Audited, pure Rust or well-reviewed | libsodium bindings |
| Auth | Local accounts + **OpenID Connect** | Simple for volunteers, SSO for agencies | — |
| PDF | **Typst ≥ 0.14** as a library | Pure Rust; templates as data; tagged PDFs and PDF/UA-1 for accessibility | HTML-to-PDF |
| Health data exchange | **HL7 FHIR** (later, adapter) | Standard for hospital interfaces | — |
| Packaging | **Single static binary** (musl, x86_64/aarch64); container optional | One-command install | — |

## 4.3 Open standards in use

ORDO uses open standards wherever they exist and fit ([ADR-0022](../adr/0022-open-standards-first.md)).
Where nothing fits, ORDO's own formats are published as open specifications in
[spec/](../../spec/README.md).

| Area | Standard / open specification |
|---|---|
| Data encoding | CBOR (RFC 8949), CDDL (RFC 8610), JSON, TOML, CSV |
| Identifiers and time | ULID (open specification), DNS names, UTC |
| Cryptography | Ed25519 (RFC 8032), XChaCha20-Poly1305, X25519 (RFC 7748), BLAKE3 (open specification), TLS 1.3 |
| Interfaces | OpenAPI 3.1, AsyncAPI 3, WebSocket (RFC 6455), OpenID Connect |
| Emergency and health data | EMSI (ISO/TR 22351), CAP (OASIS), HL7 FHIR, DIVI MIND (dataset), DV 102 tactical symbols |
| Maps | OpenStreetMap data, PMTiles, MapLibre style specification, WMS/WMTS (OGC) |
| User interface | HTML, CSS, WAI-ARIA, Web App Manifest (PWA), Project Fluent (translations), SVG |
| Design | W3C Design Tokens format (DTCG 2025.10) |
| Accessibility | EN 301 549, WCAG 2.2, PDF/UA-1 |
| Operation | OCI container images, Prometheus/OpenMetrics, OpenTelemetry, twelve-factor app |
| Licensing and supply chain | SPDX identifiers, REUSE, software bill of materials |
| Documentation and method | arc42, C4 model, MADR, EARS, Gherkin, OpenFastTrace, AGENTS.md, Markdown, Mermaid |
