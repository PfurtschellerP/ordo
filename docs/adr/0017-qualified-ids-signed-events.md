---
status: proposed
date: 2026-11-08
---

# Server-qualified IDs and signed events from day one

## Context and Problem Statement

Federation later must not require migrating data (FED-6, FED-7).

## Considered Options

* Local IDs, migrate later
* Globally unique, server-qualified IDs and signed events now

## Decision Outcome

Chosen option: "Globally unique, server-qualified IDs and signed events now", because it is cheap now and expensive later.

### Consequences

* Good, because every event's origin and integrity can be verified.
* Bad, because events are slightly larger.

## More Information

Details: [event.md](../../spec/sync/event.md)
