---
status: accepted
date: 2026-11-21
---

# A card registered twice is renumbered by the later registration

## Context and Problem Statement

Two devices that are offline can register the same triage tag number (PAT-12): the same
pre-printed card twice by mistake, or a number typed wrong. When their events meet, ORDO
keeps the first registration and shows the duplicate. The product owner decided on
2026-11-21 how it is resolved: the person whose patient "came last" changes the number.
Events are immutable (AGENTS.md rule 3), and everything about a patient (entries, orders,
photos, transports) names the patient by the number.

## Considered Options

* Rewrite: copy the second patient's events with the new number and mark the old ones void
* Reread: one event names the change; readers apply it when they read
* Leave the duplicate; people merge by talking

## Decision Outcome

Chosen option: "Reread".

* `patients.retagged` (clinical compartment, schema 1) names the old number, the later
  registration's event ID and the new number. Only its author, or the people of the holder
  it registered the patient with, may record it (Med write guard).
* Readers of the clinical compartment pass the events through `retag::read` first: the
  registration, and every event the same device recorded about the old number from the
  registration until the change, are read with the new number. The device is what tells
  the two patients apart: until the two devices met, each documented its own patient.
* Stored events and their bytes stay as they were; the change is an event like any other,
  in the log as "Patient DE-0042: neue Kartennummer DE-0099".
* The first registration keeps the number. A new number already in use is refused on the
  device.

### Consequences

* Good, because nothing is copied or deleted, and the counts for the command are right
  again at once.
* Good, because the person who knows which patient is theirs makes the change.
* Bad, because what that device recorded about the old number after the two devices met,
  but before the change, moves too; the record says so before the number is changed.
* Bad, because every clinical reader has to pass through `retag::read`; it costs nothing
  while there is no change.
