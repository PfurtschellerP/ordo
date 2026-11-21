---
status: proposed
date: 2026-11-21
---

# Patients apart from the UHS: the record opens where the person works

## Context and Problem Statement

Since [ADR-0087](0087-patients-travel-between-units.md) a unit's crew registers patients
and holds them until a handover. The registration and the record still live in the
workspace *Sanität – UHS*: a crew member who presses "Patient aufnehmen" on an Auftrag in
*Besatzung* lands in the UHS workspace, with its views (Patienten, Anordnungen,
Behandlungsplätze) that are not theirs. The product owner found this suboptimal on
2026-11-21 and asked for an idea.

## Considered Options

1. **Keep it:** the crew works in the UHS workspace for patients.
2. **A view "Patienten" in *Besatzung*:** the Med package adds it to the crew's workspace
   (as it adds its sections to other packages' pages through the package registry): the patients its
   units hold, registering with the own unit chosen, and the record.
3. **The record at an address of its own** (`/patient/<tag>`), opened from any workspace
   (UHS list, crew view, Aufträge, bell, palette, a quick handover's QR code), with a back
   link to where it was opened from; the workspace stays the one the person works in.
4. **A workspace "Patienten"** next to *UHS* and *Besatzung*, for everyone who documents.

## Decision Outcome

Proposed: options 2 and 3 together.

* *Besatzung* gets a third view, *Patienten*: the patients held by the units whose crew
  names the person (EPCR-16), each with its open handover, and the register form with the
  own unit chosen. "Patient aufnehmen" on an Auftrag opens it there, with the Auftrag
  named. The UHS workspace keeps its list, per UHS, and its other views.
* The record is one component at one address, `/patient/<tag>`, inside whichever
  workspace the person is in; old addresses (`/med.uhs/patients/<tag>`, the QR codes
  already shown) lead there. Who may write it stays with the holder (ADR-0087).
* The four-view limit holds: *Besatzung* has *Aufträge*, *Transporte*, *Patienten*.

### Consequences

* Good, because a crew stays in its workspace from the Auftrag to the handover.
* Good, because the record is the same everywhere, and every link to a patient works
  from every workspace.
* Bad, because the crew's view and the UHS list are two lists of patients; both use one
  list component, filtered by holder.

### Open question for the product owner

Should the crew's *Patienten* also show patients the unit handed over, for the rest of
the incident (read only), or only those it holds?
