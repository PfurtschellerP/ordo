# ORDO Specifications

This folder holds the open specifications of ORDO's interfaces. They are versioned
independently of the ORDO software, so that any system — open source or commercial —
can implement them and work together with ORDO.

| Folder | Specification | Format | Status |
|---|---|---|---|
| `client-server/` | Client–server API (login, master data, admin) | OpenAPI 3.1 | planned |
| `sync/` | [Event envelope](sync/event.md) and [sync protocol](sync/protocol.md) | CDDL (RFC 8610); AsyncAPI 3 later | draft 0 |
| `federation/` | Server discovery, keys, server-to-server sync | OpenAPI 3.1 + AsyncAPI 3 | planned |
| `connectors/` | Contracts for data crossing service-package borders | CDDL | planned |
| `profiles/` | Format of regional profiles; [ePCR forms as FHIR Questionnaire](profiles/questionnaire-subset.md) | FHIR R4 subset; JSON Schema later | draft 0 |
| `conformance/` | Test suite for implementations | — | planned |

## Licence

The specifications are deliberately **more permissive** than the ORDO software
(AGPL-3.0-or-later), so that anyone can implement them:

- **Texts** (descriptions, explanations): [CC BY 4.0](../LICENSES/CC-BY-4.0.txt)
- **Schemas** (`.yaml`, `.yml`, `.json`, `.cddl`) and the **conformance test suite**:
  [Apache-2.0](../LICENSES/Apache-2.0.txt)

The exact mapping is in [`REUSE.toml`](../REUSE.toml).
