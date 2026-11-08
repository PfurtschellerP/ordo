---
status: proposed
date: 2026-11-07
---

# Own sync protocol on WebSocket with CBOR

## Context and Problem Statement

Events must sync between devices and servers, scoped by compartment, offline-tolerant, and later between servers.

## Considered Options

* A general CRDT library (e.g. Automerge, Yjs)
* MQTT
* Own small protocol over WebSocket, CBOR-encoded, published as an open specification

## Decision Outcome

Chosen option: "Own small protocol over WebSocket, CBOR-encoded, published as an open specification", because incident data is small, ordering per incident is simple, and compartment scoping is a core need that generic libraries don't provide.

### Consequences

* Good, because the protocol is exactly as complex as needed.
* Bad, because we maintain it ourselves; mitigated by an open spec and conformance tests.

## More Information

Details: [event.md](../../spec/sync/event.md)
