---
status: proposed
date: 2026-11-08
---

# Connectors as versioned published events

## Context and Problem Statement

Packages must exchange data without reading each other's internals (CONN-1…5).

## Considered Options

* Packages call each other's APIs
* Shared database tables
* Versioned published events in the shared incident compartment

## Decision Outcome

Chosen option: "Versioned published events in the shared incident compartment", because connectors then sync, log and work offline like everything else.

### Consequences

* Good, because contracts are explicit and minimal.
* Bad, because derived data must be kept consistent with its source.

## More Information

Details: [08-concepts.md#83-connectors](../architecture/08-concepts.md#83-connectors)
