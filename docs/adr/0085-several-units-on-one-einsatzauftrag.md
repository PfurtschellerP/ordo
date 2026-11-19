---
status: accepted
date: 2026-11-19
---

# Several units on one Einsatzauftrag, each with its own state

Partly supersedes [ADR-0081](0081-one-einsatzauftrag.md): one unit per Auftrag and the
linked Auftrag for a further unit. Everything else in ADR-0081 stands.

## Context and Problem Statement

Under ADR-0081 an Auftrag has one unit. "Weiteres Einsatzmittel schicken" gives a second,
linked Auftrag for the same place and pin. In the demo of 2026-11-18 the product owner
found that this lets the same unit be sent twice, and that it is unclear which of the
linked Aufträge ends the pin. They asked for one Auftrag with all its units, each unit with
its own *In Arbeit* and *Erledigt*. The last unit to report *Erledigt* ends the Auftrag
and takes its pin off the map.

## Considered Options

* One Auftrag with a list of units, each with its own state
* Keep linked Aufträge, but refuse a unit that is already on one of them
* Keep linked Aufträge and show them as one card

## Decision Outcome

Chosen option: "One Auftrag with a list of units, each with its own state", decided by the
product owner on 2026-11-19.

* **Units.** `tasks.assigned` (schema version 2) adds a unit to an open Auftrag; a unit
  already on it is not added again, and the form does not offer it. Version 1 still means
  what it meant: it names the one unit of an Auftrag that has not started.
* **States per unit.** `tasks.updated` (schema version 2) may name a `unit`: the step
  applies to that unit's part only (*Zugewiesen* → *In Arbeit* → *Erledigt*, never back).
  The crew moves only its own unit's part; the command may move any, for example after a
  radio message.
* **State of the Auftrag.** It follows from its units: *In Arbeit* once one unit is, and
  *Erledigt* once every unit is done. Then it is no longer open and its pin leaves the
  map. A step without a unit still applies to the whole Auftrag, as before: an Auftrag
  for a position or a person, the command's *Abbrechen*, which ends every unit's part,
  and the events of version 1.
* **Old incidents.** Linked Aufträge (`follows`) stay readable and keep sharing their pin.
  No new ones are made.

"Refuse a unit already sent" fixes the double sending but keeps two cards and two states
for one job. "Show linked Aufträge as one card" would hide that their states differ.

### Consequences

* Good, because one card shows every unit on the job and how far each one is.
* Good, because the pin stays until the last unit is done.
* Bad, because two payloads get new schema versions. A device of an older version reads a
  unit's step as a step of the whole Auftrag, and the second unit as a replacement of the
  first. No release has shipped ADR-0081.

## More Information

Design items [Tasks](../architecture/05-building-block-view.md#tasks) and
[Reports](../architecture/05-building-block-view.md#reports),
[REP-2](../requirements/02-shared.md#rep-2--assign-reports-to-units),
[REP-5](../requirements/02-shared.md#rep-5--close-reports).
