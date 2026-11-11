# AGENTS.md — instructions for AI coding agents

ORDO is an open-source platform for on-scene coordination of emergency services (fire,
medical, police). Offline-first, need-to-know, accessible, federated. Status: pre-alpha
(phase 0). Licence: AGPL-3.0-or-later. Humans: see [README.md](README.md).

## Where things are

| Path | Content |
|---|---|
| `docs/requirements/` | Requirements as OpenFastTrace items (`req`), EARS wording — read the README first |
| `docs/architecture/` | arc42 chapters; design items (`dsn`) live in chapter 8 |
| `docs/adr/` | Architecture decisions (MADR 4), one per file |
| `docs/glossary.md` | Domain terms, English ↔ German. Use the English term in code |
| `docs/standards.md` | Standards, doctrine and conventions ORDO follows, with editions and links |
| `profiles/` | Regional profiles as data (e.g. `de/med/epcr-forms.json`, a FHIR Questionnaire), one per German state (`de-rp`, …) with their sources; no regional data in code |
| `docs/roadmap.md` | Phases, open questions, next steps |
| `spec/` | Open interface specifications (CDDL, later OpenAPI/AsyncAPI); permissive licences |
| `design/` | Design tokens (W3C DTCG), themes, contrast pairs |
| `platform/` | Platform crates: `ordo-core`, `ordo-crypto`, `ordo-sync`, `ordo-contracts`, `ordo-tokens`, `ordo-profiles` |
| `modules/`, `packages/` | Shared modules (`units`) and service packages (`med/patients`) |
| `tests/acceptance/` | Cross-crate acceptance tests (simulated network) and Gherkin `features/` |
| `tests/e2e/` | Browser end-to-end scenario and rendered-page audit (Playwright, axe-core; Firefox, Chromium, WebKit) |
| `platform/ordo-app/` | Web app (Leptos → WebAssembly, Trunk); user-visible texts in `src/text.rs` |

Read only the files you need; each document is split into small parts on purpose.

## Commands

```sh
cargo fmt --all --check                                   # formatting
cargo clippy --workspace --all-targets -- -D warnings      # lints, no warnings allowed
cargo test --workspace                                    # all tests
cargo build --target wasm32-unknown-unknown -p ordo-core -p ordo-crypto -p ordo-sync \
  -p ordo-contracts -p ordo-units -p ordo-med-patients
cargo run -p ordo-tokens -- check                         # design tokens + WCAG contrast
cargo run -p ordo-profiles -- check --register docs/standards-de-states.md --parent de  # profiles
cargo deny check                                          # dependency licences, advisories
reuse lint                                                # licence information per file
java -jar openfasttrace-4.11.0.jar trace -t _,m0,m1,m2,m3,m4,m5,m6,m7,m8,m9,m10,m11,m12,m13,m14,m15,m16,m17,m18,m19,m20,m21,m22,m23,m24,m25,m26,m27,m28,m29,m30,m31,m32,m33,m34,m35,m36,m37,m38,m39,m40 docs platform modules packages tests
```

All of these run in CI (`.github/workflows/ci.yml`) and must pass. Server and image:

```sh
cargo run -p ordo-server -- serve                          # needs ORDO_* variables, see --help
docker build -f deploy/Containerfile -t ordo:local .         # container image
(cd platform/ordo-app && trunk build --release)             # web app (needs trunk, wasm-bindgen-cli)
(cd tests/e2e && npm ci && npx playwright test)              # UI test: Playwright + axe-core
```

## Rules that must never be broken

1. **No clinical decision support.** ORDO documents and organises; it never diagnoses,
   recommends treatment or decides triage (EU MDR). Sums and unit conversions of
   documented values are fine.
2. **Need to know.** Every event belongs to one compartment. Clinical data stays in
   `med.clinical`; a device only receives compartments its user may read. Connectors
   carry the minimum (counts, statuses), never clinical details.
3. **Events are immutable.** Never edit or delete stored events; corrections are new
   events. Body bytes are forwarded unchanged. Envelope or payload changes need a new
   version.
4. **Accessibility: EN 301 549 / WCAG 2.2 AA.** Never use colour alone. Use design tokens,
   never hard-coded colours or sizes. Semantic HTML first, ARIA only where needed.
5. **Open standards and open source first** ([ADR-0022](docs/adr/0022-open-standards-first.md),
   [ADR-0032](docs/adr/0032-reuse-open-source-first.md)). Before adding a format, protocol,
   component or tool, check whether an open standard and a maintained open-source project
   exist and fit. Don't re-implement what they cover. Record the choice in an ADR.
6. **No regional code.** Nothing country- or organisation-specific in `platform/` or
   modules; it belongs in profiles.
7. **Dependencies point downwards.** Platform ← shared modules ← service packages.
   Packages talk to each other only through connectors.
8. **No `unsafe`** in ORDO's own code. Dependencies must have AGPL-compatible licences
   (see `deny.toml`).
9. **Accepted ADRs stand.** Don't work against an accepted decision; propose a new ADR
   that supersedes it instead.

## Requirements and traceability

- Requirements (`req`) are covered by design items (`dsn`), which are covered by code
  (`impl`) and tests (`utest`). Add coverage tags as plain comments:
  `// [impl->dsn~<name>~<revision>]` above the implementing item,
  `// [utest->dsn~<name>~<revision>]` above the test.
- CI traces items tagged with the milestones so far (`m0` … `m40`). Adding a requirement to
  a milestone means tagging it **and** its design items. Unit tests use `utest`,
  acceptance and end-to-end tests in `tests/` use `itest`. See `tests/README.md`.
- New or changed requirements use EARS ("When …, ORDO shall …"). Bump the revision when
  the meaning changes.
- Spec first: a larger change starts with its requirement and design item (or an ADR),
  then code and tests ([ADR-0048](docs/adr/0048-openspec-not-adopted.md)).
- Never write complete example IDs in Markdown; OpenFastTrace would import them.

## Style

- Code, identifiers, commit messages and documentation in English; German only as user
  labels in profiles and in the glossary.
- Short, plain sentences in documentation. Link to requirement IDs, ADRs and chapters
  instead of repeating them.
- Commit messages: imperative subject line, body explaining why.
- Keep CI green: format, lint, test, tokens, REUSE, cargo-deny, tracing.
