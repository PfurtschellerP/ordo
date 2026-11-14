---
status: accepted; partly superseded by ADR-0074
date: 2026-11-13
---

# Units are incident data, planned in *Vorbereitung* and led in *Führung*

## Context and Problem Statement

Roadmap item 6 put the organisation's vehicles as master data into a workspace
*Administration – Stammdaten* ([ADR-0068](0068-unit-master-data-on-the-server.md)). The
page *Vorbereitung – Einsatzmittel* held everything about units: adding them, crew,
position in the command structure, release, and also setting the status and showing the
open tasks.

The product owner reviewed it on 2026-11-13. What was meant: the incident command plans
its units and their people in *Vorbereitung*; setting the status and giving orders belong
to *Führung*. ORDO needs no organisation-wide master data of vehicles: an incident's units
are incident data. An administrator does not work on them, and they are not edited from
outside the incident. Later, vehicles and medical teams (*Sanitätstrupp*) are to get a
view of their own with their outstanding orders.

## Considered Options

* Status and orders in *Führung – Disposition*; units planned per incident in
  *Vorbereitung – Einsatzmittel*; no organisation-wide vehicle master data
* The same, with vehicle master data kept apart from incidents (ADR-0068)
* A fifth view *Einsatzmittel* in *Führung*

## Decision Outcome

Chosen option: "Status and orders in *Führung – Disposition*; units planned per
incident; no vehicle master data", decided with the product owner on 2026-11-13. It
supersedes ADR-0068.

* ***Führung – Disposition*** lists the units on scene below the open tasks: status since
  when, the status to set, the unit's open tasks and "Auftrag geben", which opens the form
  with the unit chosen. The command palette still sets a status from anywhere.
* ***Vorbereitung – Einsatzmittel*** plans: adding units, crew, position in the command
  structure, release, the strength on scene and the UHS. It shows a unit's status, but
  does not set it.
* **No vehicle master data:** the workspace *Stammdaten*, the message `changeVehicle`, the
  organisation action `manage_master_data` and the server's table `vehicles` go
  (migration 0006 drops it). An installation's file of vehicles (`ORDO_VEHICLES_FILE`),
  which only suggests call signs when a unit is added, stays as it was before ADR-0068.
  Repeating the same units every year is the job of incident templates (roadmap, later).

A fifth view would break the limit of four views per workspace (ADR-0061). Master data
would add an organisation-wide list that someone has to keep, for data each incident
plans anyway.

### Consequences

* Good, because each workspace keeps one phase: planning in *Vorbereitung*, leading in
  *Führung*.
* Good, because the board on *Disposition* is the start for a view of a unit's own
  outstanding orders (roadmap item 11).
* Good, because units stay incident data, with the incident's access rules and history.
* Bad, because the units of a recurring incident are entered again until incident
  templates exist.

## More Information

Design item *Workspaces by phase* in
[chapter 8](../architecture/08-concepts.md#workspaces-by-phase).
