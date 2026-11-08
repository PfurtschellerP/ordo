---
status: proposed
date: 2026-11-08
---

# Connector events are derived by the server that orders the source compartment

## Context and Problem Statement

Connector events (e.g. `med.patientCounts.v1`) summarise data of a protected
compartment. [ADR-0013](0013-connectors-as-events.md) defines them as published events;
it left open who derives them. The first idea was "on the device where the change
happens". The sync prototype showed two problems: devices never exchange data directly
(only through a server), so device-side derivation brings no offline benefit; and several
medical devices with partly outdated views would publish contradicting counts.

## Considered Options

* Derive on the device where the change happens
* Derive on a designated device (e.g. the UHS lead's tablet)
* Derive on the server that orders the source compartment (home server; later also the edge server)

## Decision Outcome

Chosen option: "Derive on the server that orders the source compartment", because it has
the complete, ordered view and is the only source, so counts are always consistent.

### Consequences

* Good, because there is exactly one producer per connector and incident.
* Good, because offline operation at a scene is provided by the edge server, which runs
  the same derivation.
* Bad, because the server must be able to decrypt the source compartment; this fits the
  server-trusted model of [ADR-0014](0014-clinical-trust-model.md). With end-to-end
  encryption later, derivation moves to a designated device.

## More Information

Details: [8.3 Connectors](../architecture/08-concepts.md#83-connectors), test
`command_sees_counts_but_never_patient_data` in `acceptance/tests/sync_prototype.rs`.
