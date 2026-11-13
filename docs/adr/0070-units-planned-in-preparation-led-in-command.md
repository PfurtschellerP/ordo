---
status: accepted
date: 2026-11-13
---

# Units are planned in *Vorbereitung* and led in *Führung*; master data stays with them

## Context and Problem Statement

Roadmap item 6 put the organisation's vehicles as master data into a workspace
*Administration – Stammdaten* ([ADR-0068](0068-unit-master-data-on-the-server.md)), as
[ADR-0061](0061-workspaces-by-phase-and-administration-apart.md) had planned ("Later:
*Stammdaten*"). The page *Vorbereitung – Einsatzmittel* held everything about units:
adding them, crew, position in the command structure, release, and also setting the
status and showing the open tasks.

The product owner reviewed it on 2026-11-13. What was meant: the incident command plans
its units and their people in *Vorbereitung*; setting the status and giving orders belong
to *Führung*. The vehicles of the master data can live in *Einsatzleitung*, where units are
planned. Later, vehicles and medical teams (*Sanitätstrupp*) are to get a view of their
own with their outstanding orders.

## Considered Options

* Status and orders in *Führung – Disposition*; the units planned in *Vorbereitung –
  Einsatzmittel*, with the organisation's vehicles on the same page
* Status and orders in *Führung – Disposition*; the vehicles stay in *Administration*
* A fifth view *Einsatzmittel* in *Führung*

## Decision Outcome

Chosen option: "Status and orders in *Führung – Disposition*, the vehicles with the
units", decided with the product owner on 2026-11-13.

* ***Führung – Disposition*** lists the units on scene below the open tasks: status since
  when, the status to set, the unit's open tasks and "Auftrag geben", which opens the form
  with the unit chosen. The command palette still sets a status from anywhere.
* ***Vorbereitung – Einsatzmittel*** plans: adding units (from the master data or ad hoc),
  crew, position in the command structure, release, the strength on scene and the UHS.
  It shows a unit's status, but does not set it.
* **The organisation's vehicles** are kept at the end of *Vorbereitung – Einsatzmittel*,
  for whoever has `manage_master_data`. The workspace *Administration – Stammdaten* goes.
  This replaces the plan of ADR-0061 for *Stammdaten* in *Administration*. Everything
  else in ADR-0068 stands.

A fifth view would break the limit of four views per workspace (ADR-0061). Keeping the
vehicles in *Administration* would separate them from the page where they are used.

### Consequences

* Good, because each workspace keeps one phase: planning in *Vorbereitung*, leading in
  *Führung*.
* Good, because the board on *Disposition* is the start for a view of a unit's own
  outstanding orders (roadmap item 11).
* Bad, because master data now needs an incident to be open: an administrator without an
  incident opens or plans one first.

## More Information

Design items *Workspaces by phase* and *Vehicle master data* in
[chapter 8](../architecture/08-concepts.md#workspaces-by-phase).
