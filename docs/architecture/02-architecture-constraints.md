# 2. Architecture constraints

> Part of the [ORDO architecture](README.md) (arc42).

The full list is in [requirements/08-constraints.md](../requirements/08-constraints.md).

| Constraint | Consequence for the architecture | Decision |
|---|---|---|
| Licence AGPL-3.0-or-later; specifications CC BY 4.0 / Apache-2.0 | All dependencies must be AGPL-compatible (checked by cargo-deny) | [ADR-0007](../adr/0007-licence-agpl.md), [ADR-0018](../adr/0018-open-specifications.md) |
| Rust on server and client (WebAssembly) | Shared crates; DOM-based UI | [ADR-0002](../adr/0002-rust-everywhere.md) |
| Open standards first | New formats and protocols are checked against existing standards | [ADR-0022](../adr/0022-open-standards-first.md) |
| GDPR, health data (Art. 9), DE medical documentation rules | Data compartments, per-compartment retention and keys | [ADR-0012](../adr/0012-data-compartments.md) |
| EU Medical Device Regulation | No clinical decision support | [requirements/08](../requirements/08-constraints.md) |
| EN 301 549 / WCAG 2.2 AA | Accessible design system, DOM UI, map alternative | [ADR-0020](../adr/0020-accessibility-target.md) |
| Germany first | DE profile, but no regional code in the core | [ADR-0008](../adr/0008-german-profile-first.md) |
| Volunteer project, limited time | Small MVP; modular monolith; field tests at planned operations | [ADR-0015](../adr/0015-cloud-ready-monolith.md), [ADR-0019](../adr/0019-med-package-first.md) |
