---
status: proposed
date: 2026-11-08
---

# Regional profiles as layered data packages

## Context and Problem Statement

Countries, states and organisations differ in terminology, codes and forms (I18N-2…8).

## Considered Options

* Country-specific code
* Configuration in the database
* Layered profile packages (TOML, CSV, SVG, Fluent) mapped to core meanings

## Decision Outcome

Chosen option: "Layered profile packages mapped to core meanings", because the code works with core meanings while users see regional labels; new regions need no recompiling.

### Consequences

* Good, because non-programmers can write profiles.
* Bad, because every catalogue type needs a schema and validation.

## More Information

Details: [08-concepts.md#88-regional-profiles](../architecture/08-concepts.md#88-regional-profiles)
