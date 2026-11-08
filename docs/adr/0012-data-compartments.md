---
status: proposed
date: 2026-11-08
---

# Data compartments

## Context and Problem Statement

Patient data is special-category data (GDPR Art. 9) and must never reach devices that don't need it (SEC-8…10).

## Considered Options

* Role checks only on the server
* Compartments with own access rules, keys, sync scope and retention

## Decision Outcome

Chosen option: "Compartments with own access rules, keys, sync scope and retention", because need-to-know is enforced by what a device receives, not only by what it shows.

### Consequences

* Good, because a lost fire tablet holds no patient data.
* Bad, because sync and key management are more complex.

## More Information

Details: [08-concepts.md#82-data-compartments](../architecture/08-concepts.md#82-data-compartments)
