---
status: accepted
date: 2026-11-14
---

# Reports are recorded in *Disposition*, assigned as tasks, and become patients through the crew

## Context and Problem Statement

Reports reach the incident command by radio, phone or in person: a fight at gate 3, a
person lying at the north stage. The command writes them down, sends a unit and follows
them until they are dealt with. In ORDO so far a task held only a text for a unit
([TASK-1](../requirements/02-shared.md#task-1--give-tasks)); where the report came from,
where exactly, and whether a patient came of it, was lost or written into the text.

A patient case ends with a patient. Patient records are clinical data in `med.clinical`,
which the incident command does not read (need to know, rule 2). The command still needs to
know that the case was taken over. Roadmap item 10; decided with the product owner on
2026-11-14: one list for events and patient cases, the crew chooses the patient's UHS, the
place as text with an optional point on the map, which the crew sees to find the patient.

## Considered Options

* Reports as their own records, assigned through linked tasks
* Reports as tasks with a place (no record of their own)
* Patient cases as patients without a UHS, registered by the command

## Decision Outcome

Chosen option: "Reports as their own records, assigned through linked tasks". A report is
what happened, a task is what a unit is to do about it: one report can need several units,
one after the other, and stays open while its tasks change.

* **Module `reports`** in the shared `incident` compartment: `reports.recorded` (kind
  event or patient, place, short description, the time it came in), `reports.closed`, and
  `reports.patientRegistered`. No clinical data: the form says so, and the description is
  short.
* **Assigning** gives a task (`tasks.given`, schema version 2) that names the report. Task
  status works as before; the report lists its units and their status.
* **The place on the map** is a map object of the new kind `report` (`map.objectPlaced`,
  schema version 2), placed in the map editor like a unit. The crews of the assigned units
  see it on a small map in the case, and as a `geo:` link (RFC 5870) that opens the
  device's maps app.
* **Patient from a case:** a crew member of an assigned unit registers the patient from
  the case, choosing the UHS (the unit's own UHS in the command structure is offered
  first). `patients.registered` (schema version 4) names the report, in `med.clinical`.
  The device also writes `reports.patientRegistered` to the incident compartment, which
  names only the report and the unit: the command learns that a patient was taken over,
  not who or how. Medical roles write only this event of the module `reports`.
* **Closing** is the command's: a closed report stays in the history and the incident log.

"Reports as tasks with a place" would mix up what happened with what to do, and a second
unit for the same report would be a second, unconnected task. Registering a patient without
a UHS would need a new kind of patient and changes to every UHS count; the crew knows the
UHS.

### Consequences

* Good, because the command sees each report from the call to its end, with its units and
  whether a patient came of it, without clinical data.
* Good, because the crew finds the place on the map or in their maps app.
* Good, because the patient record keeps where the patient came from.
* Bad, because three payloads get new schema versions (tasks, map objects, patients). A
  device of an older version ignores the new fields, and leaves out a map object of the
  kind `report`. No release has shipped the old forms.
* Bad, because a short description can still carry clinical details if someone types them;
  the form asks not to, and ORDO cannot check it.

## More Information

Design item *Reports* in
[chapter 5](../architecture/05-building-block-view.md#reports). Units and medical teams get
their own workspace later (roadmap item 11); until then, crews see their cases in the UHS
workspace.
