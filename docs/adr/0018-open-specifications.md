---
status: accepted
date: 2026-11-08
---

# Spec-first open specifications under permissive licences

## Context and Problem Statement

Other systems, including commercial ones, should be able to talk to ORDO (API-1…4).

## Considered Options

* Code as the only documentation
* Open specifications under the software licence
* Spec-first open specifications under CC BY 4.0 (texts) and Apache-2.0 (schemas, conformance)

## Decision Outcome

Chosen option: "Spec-first open specifications under CC BY 4.0 and Apache-2.0", because anyone may implement the interfaces while ORDO's code stays AGPL.

### Consequences

* Good, because adoption and interoperability become easier.
* Bad, because specs must be maintained alongside the code.

## More Information

Details: [README.md](../../spec/README.md)
