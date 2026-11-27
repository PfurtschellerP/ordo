---
status: accepted
date: 2026-11-27
---

# Units and treatment places in *Führung – Lage*

## Context and Problem Statement

The demo feedback of 2026-11-27 found the table *Patienten je UHS* under the situation map
of little use: the overview's tile *Behandlungsplätze* already shows more. Units appear in
*Führung* only as rows of the status list in *Disposition* and as counts on the tile
*Personal*. Their crew, qualifications, current Auftrag and transport are visible only in
*Vorbereitung – Einsatzmittel*, where they are edited. The command lacks a view to lead
the units and the treatment places from.

*Führung* has four views (*Übersicht*, *Lagekarte*, *Disposition*, *Transporte*), the most
the phone's bottom bar takes with the menu.

## Considered Options

1. *Lagekarte* becomes *Lage*, with the tabs *Karte*, *Einsatzmittel* and
   *Behandlungsplätze*; the map beside the chosen list where the page is wide.
2. A tab *Einsatzmittel* in *Disposition*.
3. A fifth view *Einsatzmittel*.
4. Only more overview tiles.

## Decision Outcome

Chosen: option 1, decided by the product owner on 2026-11-27 (concept *Situation, units
and profile*).

* The view keeps its address (`command.lead/map`) and is named *Lage*.
* From 84rem the map stands beside the chosen list (*Einsatzmittel* or
  *Behandlungsplätze*); narrower, the three are tabs. Choosing a unit or a UHS in a list
  centres the map on it and opens its card.
* *Einsatzmittel* lists every unit of the incident, UHS included
  ([ADR-0069](0069-uhs-counts-as-a-unit.md)), grouped by what the command needs to know:
  available, on an Auftrag, transporting, not available. The group of each status code
  comes from the profile (`units/status-codes.toml`), not from code.
* *Behandlungsplätze* shows the free and all places per category over all UHS, then one
  card per UHS with its places, the patients waiting for a place or a transport, its staff
  by qualification, an open staff request and the discharged patients. It replaces
  *Patienten je UHS*.
* Editing stays in *Vorbereitung*; *Lage* has only the actions of leading (set a status,
  give an Auftrag, show on the map).
* The overview gets a tile *Einsatzmittel* (units per group, free units per type) that
  links to the tab; the tile *Behandlungsplätze* links to its tab.
* The four groups carry Lucide icons with their word, never alone: `circle-check`,
  `clipboard-list`, `ambulance`, `ban`. Shapes stay reserved for the triage categories.

### Consequences

* Good, because the command sees where each unit is and what it does without leaving
  *Führung*, and the map is beside the list on a command post's screen.
* Good, because the four-view rule and the phone's bottom bar stay as they are.
* Bad, because on phones the map and the lists are in separate tabs; choosing on the map
  switches tabs.
* Neutral: "discharged, of them transported" needs the counts by outcome, a new version of
  the counts event; until then only "discharged" is shown.

## Pros and Cons of the Options

### Option 2: a tab in *Disposition*

* Good, because units sit next to the Aufträge they get.
* Bad, because *Disposition* is already long, and the map is not on it.

### Option 3: a fifth view

* Bad, because it breaks the four-view rule and the phone's bottom bar.

### Option 4: only tiles

* Bad, because crew, Auftrag and transport per unit have no place on a tile.
