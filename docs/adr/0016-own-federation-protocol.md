---
status: proposed
date: 2026-11-08
---

# Own incident-scoped federation, inspired by Matrix

## Context and Problem Statement

Installations of different organisations should work together without a central server (FED-1…7).

## Considered Options

* Build on Matrix
* Own incident-scoped protocol, Matrix only as messaging adapter

## Decision Outcome

Chosen option: "Own incident-scoped protocol, Matrix only as messaging adapter", because emergency services need closed federation, a simple order per incident and small servers.

### Consequences

* Good, because a network split between organisations is handled like an offline tablet.
* Bad, because we design and maintain the protocol.

## More Information

Details: [08-concepts.md#84-federation](../architecture/08-concepts.md#84-federation)
