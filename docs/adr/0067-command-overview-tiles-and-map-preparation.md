---
status: accepted
date: 2026-11-13
---

# The command's overview is a grid of tiles, and *Vorbereitung* has its own *Lagekarte*

## Context and Problem Statement

The product owner's feedback on 2026-11-13:

* The *Übersicht* of *Führung* ([ADR-0061](0061-workspaces-by-phase-and-administration-apart.md):
  one card per page of the package) shows too much of *Vorbereitung*. During the incident
  the command needs the map, *Disposition*, its people and the state of the UHS, plus a
  few quick links. The approved concept is a grid of tiles of different sizes ("bento").
* Editing the map is one click too deep behind *Vorbereitung – Einsatz*
  ([ADR-0065](0065-map-editing-in-preparation.md)). Preparing the *Lagekarte*, that is
  getting the map region and placing the units on it, deserves its own view.
* The incident needs a description and its planned date and times, and a festival over
  several days must be paused overnight.

A workspace has at most four views, so the phone's bottom bar keeps four views and the
menu ([ADR-0026](0026-navigation-at-the-bottom-on-phones.md)).

## Considered Options

* *Vorbereitung*: Einsatz · Lagekarte · Einsatzmittel · Personal, with the command
  structure on the *Personal* page
* Five views in *Vorbereitung*, the phone bar with six places
* Five views, the fifth behind a "Mehr" place on phones

## Decision Outcome

Chosen option: the first, decided by the product owner on 2026-11-13. People and the
command structure are prepared together (who is there, who leads what), and the bar stays
as it is. This supersedes the table of workspaces and the overview rule of ADR-0061, and
the *Page* part of ADR-0065. ADR-0065's offline copy by default stands; its "Karte ohne
Netz" section with a delete button goes, since the device keeps the region by itself.

| Workspace | Views |
|---|---|
| Führung | Übersicht · Lagekarte (read only) · Disposition · Transporte |
| Vorbereitung | Einsatz (name, description, planned start and end, start, pause, resume, close) · Lagekarte (map region, address search and the map editor on one page) · Einsatzmittel · Personal (roles, devices in use, command structure) |

* **Übersicht** of *Führung*: a heading with the incident's name, state and planned start,
  then tiles in a grid of twelve columns: *Lagekarte* (large, a map that only shows, with
  the number of objects and whether the region is on the device), *Disposition* (open
  orders, follow-ups due, the three next to follow up), *Personal* (persons and units on
  scene, strength leaders / sub-leaders / crew), *UHS* (per UHS its load against capacity,
  patients per triage category, open transports; and the total) and *Schnellzugriff*
  (give an order, transports, incident log, command structure). On phones the tiles
  stack. Counts only, nothing ranked except the orders by their follow-up time.
* **Pause:** a running incident can be paused and resumed by those who may change its
  state. The state shows "Laufend – unterbrochen"; nothing else is locked, so a night shift
  can still document. New event types, so existing payloads stay unchanged:
  `incidents.detailsChanged` and `incidents.pauseChanged`.

### Consequences

* Good, because *Führung* shows what the command steers the incident with, and the map is
  prepared without a detour.
* Good, because a festival keeps one incident (one log, one map) over several days.
* Bad, because *Personal* grows; its sections have headings, and the command structure
  keeps its own anchor.
* Neutral: routes change. `command.prepare/structure` opens *Personal*;
  `command.prepare/incident/map…` opens *Lagekarte*.

## More Information

[ADR-0061](0061-workspaces-by-phase-and-administration-apart.md),
[ADR-0064](0064-map-editor-with-terra-draw.md), [ADR-0065](0065-map-editing-in-preparation.md),
[08-concepts.md](../architecture/08-concepts.md) (*Workspaces by phase*).
