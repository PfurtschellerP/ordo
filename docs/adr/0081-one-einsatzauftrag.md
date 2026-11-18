---
status: accepted
date: 2026-11-18
---

# Reports and tasks become one *Einsatzauftrag*

## Context and Problem Statement

[ADR-0075](0075-reports-recorded-in-dispatch.md) kept a report (*Meldung*) and the task
that sends a unit (*Auftrag*) apart: a report had its own state and was closed by the
command, and each task had its own state as well. In the demo of 2026-11-17 the product
owner found this confusing. *Disposition* showed two lists, a report could stay open after
its unit was done, and the pin stayed on the map until someone closed the report too.
Setting the pin also sent the command to another page.

The product owner asked for one card with one state: assigning turns the report into the
unit's Auftrag, and when it ends, the pin leaves the map. They also asked that a person
sent to a UHS after a staff request gets an Auftrag
([UHS-15](../requirements/04-med.md#uhs-15--request-a-physician-or-a-paramedic)). They
decided on 2026-11-18 to keep one unit per Auftrag for now, and that the crew's "Erledigt"
ends it.

## Considered Options

* One Auftrag in the module `tasks`, which may come in without a unit
* Keep both modules and only show a report and its task as one card
* Several units with their own states on one Auftrag

## Decision Outcome

Chosen option: "One Auftrag in the module `tasks`", because then one record has one state,
and nothing has to be closed twice.

* **What came in.** `tasks.given` (schema version 3) also takes the kind (`event` or
  `patient`), the place as text and the time it came in. With a kind, the unit is
  optional: the Auftrag has been received and waits for one. Without a kind, it is a task
  as before and needs a unit, a position or a person.
* **State.** No new status: an open Auftrag without anyone is *Eingegangen*; with a unit,
  position or person it is *Zugewiesen*; then *In Arbeit* and *Erledigt*, or *Abgebrochen*
  (the command only). A status never goes back.
* **Assigning.** `tasks.assigned` names the unit (one per Auftrag) while the Auftrag has
  not started. "Weiteres Einsatzmittel schicken" gives a second Auftrag with the same kind,
  place and description, linked to the first by `follows`. Each unit has its own state.
* **Pin.** The pin is a map object of the new kind `task` (`map.objectPlaced`, schema
  version 4), set on the form itself, by tapping a small map or by address search
  ([ADR-0066](0066-address-search-with-nominatim.md)), and moved from the card. A
  linked Auftrag uses the first one's pin. The situation map shows the pin while one of
  them is open.
* **Patient from a patient Auftrag.** `tasks.patientRegistered` names the Auftrag and the
  unit, never the patient. `patients.registered` (schema version 5) names the Auftrag in
  `task`. Cedar lets medical roles write it besides `tasks.updated`.
* **Person.** A task may name a person (`person`, an account). When the command sends
  someone after a staff request, the person gets the Auftrag "Zur UHS … als …".
* **Old incidents.** The module `reports` gets no new events. Its events are still read: a
  report and its tasks show as Aufträge, and an open report without a task shows as
  received. Assigning it gives an Auftrag that names it, as before.

"Show both as one card" would keep two states under one card, and closing would still need
two events from different people. "Several units on one Auftrag" makes the card, the crew
view and the end of the Auftrag harder; the linked Auftrag covers that case for now.

### Consequences

* Good, because the command sees one list, and an Auftrag ends once, with its pin.
* Good, because the pin is set without leaving *Disposition*.
* Good, because a person sent to a UHS sees what is expected of them.
* Bad, because three payloads get new schema versions (tasks, map objects, patients). A
  device of an older version ignores the new fields and leaves out a map object of the kind
  `task`. No release has shipped the old forms.
* Bad, because the module `reports` stays in the code to read old incidents.

## More Information

Design items [Tasks](../architecture/05-building-block-view.md#tasks) and
[Reports](../architecture/05-building-block-view.md#reports). Supersedes ADR-0075 except
for the patient case's flow through the crew, which stays as it was.
