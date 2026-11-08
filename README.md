# ORDO

**O**perations, **R**esponse, **D**ispatch & **O**rganisation

ORDO is an open-source platform for on-scene coordination of emergency services.
Fire services, emergency medical services and police share one platform, and each
gets its own package with dedicated views, connected to the others through defined
interfaces.

- **Works offline**: every device keeps working without network and syncs later.
- **Need to know**: patient data stays with medical staff and never reaches devices
  that don't need it.
- **Easy to run**: one binary or one `docker compose up`, from a Raspberry Pi to a
  cluster.
- **Open**: open source, open specifications, federation between organisations.

Built first for volunteer organisations in Germany, designed to be international.

> **Status: planning (pre-alpha).** There is no usable software yet.

## Documentation

- [Requirements](docs/requirements/README.md) — traceable items (OpenFastTrace), EARS wording
- [Architecture](docs/architecture/README.md) — arc42 chapters
- [Decisions](docs/adr/README.md) — architecture decision records (MADR)
- [Glossary](docs/glossary.md) — domain terms, English ↔ German
- [Roadmap](docs/roadmap.md) — phases, open questions, next steps
- [Specifications](spec/README.md) — open interface specifications
- [Design system](design/README.md) — design tokens, themes, contrast
- [AGENTS.md](AGENTS.md) — instructions for AI coding agents

ORDO uses **open standards wherever they exist and fit**
([ADR-0022](docs/adr/0022-open-standards-first.md)); the list of standards in use is in
[architecture chapter 4.3](docs/architecture/04-solution-strategy.md#43-open-standards-in-use).

## Repository map

```text
docs/            requirements (OpenFastTrace), architecture (arc42), decisions (MADR), glossary, roadmap
spec/            open interface specifications (CDDL, later OpenAPI/AsyncAPI)
design/          design tokens and themes (W3C DTCG), WCAG contrast pairs
platform/        platform crates: core, crypto, sync, contracts, tokens, server, web app
modules/         shared functional modules (e.g. units)
packages/        service packages (e.g. med/patients)
tests/           cross-crate acceptance tests, Gherkin features, browser end-to-end tests
deploy/          container image, Docker Compose
```

Start with [AGENTS.md](AGENTS.md) for commands and rules, and with
[tests/README.md](tests/README.md) for the kinds of tests.

## Licence

- ORDO software: [GNU Affero General Public License v3.0 or later](LICENSE)
- Documentation: [CC BY 4.0](LICENSES/CC-BY-4.0.txt)
- Specifications: [CC BY 4.0](LICENSES/CC-BY-4.0.txt) (texts) and
  [Apache-2.0](LICENSES/Apache-2.0.txt) (schemas, conformance tests)

Details per file are in [`REUSE.toml`](REUSE.toml).
