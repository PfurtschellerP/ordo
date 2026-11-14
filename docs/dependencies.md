# Dependencies

What ORDO depends on, why, and how well it is maintained. Roadmap item *Dependency
analysis*; repeated in the architecture sweep before each release. Last done 2026-11-14.

## How it is checked

* **Need:** every direct dependency has a job no smaller one or the standard library
  does; `cargo machete` in CI finds unused ones.
* **Maintenance:** the last release on crates.io or npm, and whether the repository is
  archived or quiet. A crate without a release for two years is fine when it is small and
  finished (e.g. `fnv`, `scopeguard`); it is a concern when it is large or does security
  work.
* **Advisories:** `cargo deny` in CI refuses known vulnerabilities, crates RustSec lists as
  unmaintained, yanked crates and licences that do not fit the AGPL (`deny.toml`).
* **Updates:** Dependabot proposes updates weekly, each release at least 7 days old, so a
  compromised release is more likely found and pulled first. The JavaScript libraries and
  typefaces the app vendors are not visible to Dependabot; `tools/vendored/check.sh`
  compares them with npm every week and opens an issue when one is behind.

## Size

| Part | Crates (with versions) | Of them duplicates |
|---|---|---|
| Server (`ordo-server`) | 399 | 31 names in two versions |
| Web app (`ordo-app`, WebAssembly) | 229 | 10 names, 6 of them only at build time |
| Storage worker (`ordo-store`) | 32 | 8 not in the app |

Most crates come with a few large building blocks: Leptos (app), axum, sqlx, Cedar, tonic
and object_store (server). Duplicates come from the ecosystem moving between major
versions (e.g. `base64` 0.22 in axum, sqlx and Leptos, 0.23 in ORDO); they shrink as those
crates update.

## Findings of 2026-11-14

* **Removed `console_error_panic_hook`:** its repository is archived. A few lines in the app
  do the same.
* **One `reqwest` instead of two:** openidconnect's own adapter needed reqwest 0.12, while
  object_store and pmtiles use 0.13. ORDO now hands its reqwest 0.13 to openidconnect; the
  server has seven crates fewer, among them a QUIC stack nothing used.
* **Updated `rqrr` to 0.11:** it fixes decoding of QR codes with two block sizes, as large
  codes have. It needs `g2p` and `lru`.
* **Archived, kept:** `paste`, a compile-time macro of Leptos. No known vulnerability;
  `deny.toml` lists it until Leptos replaces it.
* **Advisory, kept:** `rsa` (via openidconnect) has a timing side channel in private-key
  operations; ORDO only verifies signatures with public keys (`deny.toml`).
* **Open:** Dependabot's update of Garage (v2.1.0 to v2.5.0) waits as a pull request.
  MapLibre 6.18.0 (2026-11-11) waits for the cooldown and the weekly check.

## Direct dependencies

"Personal" means the repository belongs to one person rather than an organisation: a
single point of failure, though not a problem in itself. "If it goes" names what would
replace it.

### Server and tools

| Crate | For | Last release | Maintained by | If it goes |
|---|---|---|---|---|
| axum | HTTP and WebSocket server | 2026-04 | tokio-rs | – (the Rust standard) |
| sqlx | Server database, migrations (SQLite) | 2026-05 | launchbadge | rusqlite with refinery |
| tokio | Async runtime | 2026-10 | tokio-rs | – |
| tower-http | Static files, headers | 2026-08 | tower-rs | own handlers |
| cedar-policy | Authorisation policies (ADR-0035) | 2026-11 | AWS, cedar-policy | – (the policy language is Cedar) |
| openidconnect | Login with OpenID Connect | 2026-10 | personal (ramosbugs) | `openid` crate; the protocol is standard |
| reqwest | HTTP client (login, map regions, address search, S3) | 2026-09 | personal (seanmonstar) | hyper directly |
| rustls | TLS, with ring | 2026-11 | rustls | – |
| object_store | Attachments: local directory or S3 | 2026-09 | Apache Arrow | `rust-s3` |
| pmtiles | Cutting map regions (ADR-0057) | 2026-11 | Stadia Maps | own reader of the documented format |
| tonic, tonic-prost, prost | Dex's gRPC API for local accounts (ADR-0060) | 2026-10/11 | grpc, tokio-rs | – |
| bcrypt | Password hashes for Dex | 2026-07 | personal (Keats) | another bcrypt crate (Dex needs bcrypt) |
| rcgen | Certificates of the Dex API | 2026-08 | rustls | – |
| qrcode | Pairing QR code as SVG | 2024-07 | personal (kennytm) | `fast_qr`; watch |
| time | Dates of certificates and map builds | 2026-08 | time-rs | `jiff` |
| tracing, tracing-subscriber | Logs (JSON) | 2025-12, 2026-03 | tokio-rs | – |
| getrandom | Randomness from the system | 2026-06 | rust-random | – |
| base64, sha2 | Encodings, Gravatar hash, profile checksums | 2026-08, 2026-10 | personal (marshallpierce), RustCrypto | – |
| futures-util | Stream helpers for WebSockets | 2026-08 | rust-lang | – |
| toml, toml_edit | Configuration, profiles | 2026-11 | toml-rs | – |
| tempfile (tests), tokio-tungstenite (tests, load device) | | 2026-03, 2026-07 | personal (Stebalien), snapview | – |

### Shared by server and app

| Crate | For | Last release | Maintained by | If it goes |
|---|---|---|---|---|
| serde, serde_json, serde_bytes | Serialisation | 2025-09 – 2026-10 | serde-rs | – |
| ciborium | CBOR of events and messages (ADR-0005) | 2024-01 | enarx (repository active) | `minicbor` with serde; **watch**: the core format depends on it |
| ed25519-dalek | Event signatures | 2026-07 | dalek-cryptography | – |
| chacha20poly1305, blake3, zeroize | Encryption, hashes, wiping keys | 2026-06 – 2026-10 | RustCrypto, BLAKE3 team | – |
| thiserror | Error types | 2026-09 | personal (dtolnay) | – |
| percent-encoding | URLs | 2025-08 | servo | – |

### Web app and storage worker

| Crate | For | Last release | Maintained by | If it goes |
|---|---|---|---|---|
| leptos, leptos_router | User interface (ADR-0003) | 2026-10 | leptos-rs | – (a framework change) |
| wasm-bindgen, js-sys, web-sys, wasm-bindgen-futures | Browser APIs | 2026-10 | wasm-bindgen | – |
| idb | IndexedDB for the storage key (ADR-0041) | 2025-12 | personal (devashishdxt) | web-sys directly, about 80 lines; watch |
| rqrr | Reading Meldekarten (ADR-0072, ADR-0073) | 2026-09 | personal (WanzenBug) | rxing once it drops `encoding` |
| rusqlite, sqlite-wasm-rs, sqlite-wasm-vfs | SQLite in the browser's private file system (ADR-0034) | 2026-09 – 2026-11 | rusqlite; personal (Spxg) | – |

## Vendored JavaScript and typefaces

Taken from npm with the recorded integrity, licences next to them
(`platform/ordo-app/vendor-map.sh`, `vendor-fonts.sh`).

| Package | Vendored | For | Maintained by |
|---|---|---|---|
| maplibre-gl | 6.17.0 | Maps (ADR-0006) | MapLibre organisation |
| pmtiles | 4.5.0 | Reading map regions | Protomaps |
| @protomaps/basemaps | 5.7.2 | Map style | Protomaps |
| terra-draw, its MapLibre adapter | 1.39.0, 1.5.0 | Drawing on the map (ADR-0064) | personal (JamesLMilner), active |
| Atkinson Hyperlegible Next and Mono (Fontsource) | 5.3.0 | Typefaces of the design tokens | Fontsource |

## Container images

Pinned by digest, updated by Dependabot: `rust` (build), `distroless/static` (the ORDO
image), Dex, Caddy and Garage (Compose bundle). All maintained; Garage is developed at
git.deuxfleurs.fr, GitHub holds a mirror.
