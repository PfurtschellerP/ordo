# 6. Runtime view

> Part of the [ORDO architecture](README.md) (arc42).

## 6.1 Recording an event offline and syncing it later

```mermaid
sequenceDiagram
  actor Medic
  participant App as ORDO app (tablet)
  participant Store as Local store
  participant Server as Home server
  participant Other as Other devices

  Medic->>App: Register patient (tag no., SK II, UHS 1)
  App->>App: Create event, HLC timestamp, sign, seal payload (med.clinical)
  App->>Store: Append to outbox, update projection
  App-->>Medic: Patient shown on UHS board (immediately, offline)
  Note over App,Server: No network
  App->>Server: Reconnect: send outbox events
  Server->>Server: Verify signature, check permissions, assign seq, extend hash chain
  Server-->>App: Confirm seq numbers
  Server-->>Other: Push event to devices entitled to med.clinical
```

Concepts: [8.1 Events and ordering](08-concepts.md#81-events-and-ordering),
[8.2 Data compartments](08-concepts.md#82-data-compartments).

## 6.2 Patient counts reach command without patient data

```mermaid
sequenceDiagram
  participant UHS as UHS tablet (Med)
  participant Server as Home server
  participant Cmd as Command tablet (Fire)

  UHS->>UHS: Patient re-triaged (event in med.clinical)
  UHS->>UHS: Derive med.patientCounts.v1 (numbers only)
  UHS->>Server: Clinical event + connector event (incident compartment)
  Server-->>UHS: Both events (user may read both compartments)
  Server-->>Cmd: Connector event only
  Cmd->>Cmd: Show "UHS 1: SK I 3 · SK II 5 · SK III 12"
```

Concept: [8.3 Connectors](08-concepts.md#83-connectors). Requirements: PAT-6, SEC-9,
CONN-3.

## 6.3 Concurrent changes while disconnected

```mermaid
sequenceDiagram
  participant A as Tablet A
  participant B as Tablet B
  participant Server as Home server

  Note over A,B: Both offline
  A->>A: unit RTW 1 → status 4 (HLC t1)
  B->>B: unit RTW 1 → status 3 (HLC t2 > t1)
  A->>Server: event (t1)
  B->>Server: event (t2)
  Server-->>A: both events in sequence order
  Server-->>B: both events in sequence order
  A->>A: Projection: last writer by HLC wins → status 3; conflict noted in log
  B->>B: Same result
```

Concept: [8.1.1 Ordering and conflicts](08-concepts.md#811-ordering-and-conflicts).
Requirement: OFF-4.
