---
status: proposed
date: 2026-11-07
---

# Store data as events, local-first

## Context and Problem Statement

Devices at a scene often have no network (OFF-1…5), and the incident log must be complete and tamper-evident (LOG-1, LOG-3).

## Considered Options

* Central database with a REST API; devices online only
* Local-first devices with a general-purpose CRDT library
* Local-first devices with an append-only event log per incident

## Decision Outcome

Chosen option: "Local-first devices with an append-only event log per incident", because the log comes for free, syncing means exchanging missing events, and the same events can later be shared between organisations.

### Consequences

* Good, because devices keep working offline and nothing is overwritten.
* Good, because the incident log is complete by construction.
* Bad, because current state must be computed (projections) and event schemas must be versioned carefully.

## More Information

Details: [08-concepts.md#81-events-and-ordering](../architecture/08-concepts.md#81-events-and-ordering)
