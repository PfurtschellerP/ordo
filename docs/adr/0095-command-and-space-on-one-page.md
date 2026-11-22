---
status: accepted; its layout for narrow pages superseded by ADR-0097
date: 2026-11-22
---

# Führung & Raum: the command structure and the situation map on one page

## Context and Problem Statement

[ADR-0067](0067-command-overview-tiles-and-map-preparation.md) gave *Vorbereitung* the
views Einsatz · Lagekarte · Einsatzmittel · Personal, with the command structure inside
*Personal* (since 2026-11-21 its own tab there, UX-16). After the demo of 2026-11-17 the
product owner asked for one page *Führung & Raum*: who leads what, and where it is, are
prepared together. It replaces the view *Lagekarte* and keeps the limit of four views per
workspace. On 2026-11-22 they confirmed it, after the tab under *Personal* was built.

How do the command structure and the map share one page?

## Decision Drivers

* The command sets up sectors and places them (a command post, a UHS, a sector's
  leader) in one go; switching pages loses the place in both.
* At most four views per workspace; every main function at most two taps away (UX-1).
* The org chart needs about 40rem to be drawn as a chart (`dsn~org-chart~2`), the map
  editor needs width for its map.
* Phones and tablets, keyboard and screen readers (WCAG 2.2 AA).

## Considered Options

1. **Keep the command structure under *Personal*** and the map as *Lagekarte*.
2. **Two tabs on one page** (*Führungsstruktur*, *Lagekarte*).
3. **One page *Führung & Raum*, both at once**: the command structure, then the map
   editor; side by side where the page is wide enough for both, one below the other
   elsewhere. Each position in the tree offers to put it on the map, or to show it there.

## Decision Outcome

Chosen: option 3, decided by the product owner on 2026-11-17 and confirmed on 2026-11-22.
Tabs (option 2) would hide one of the two, which is what the page is for.

* *Vorbereitung* has Einsatz · Personal · Einsatzmittel · Führung & Raum. *Personal*
  keeps the tabs *Helfende* and *Geräte*.
* *Führung & Raum* (`command.prepare/map`, so the old links keep working, also
  `command.prepare/structure`) shows the command structure with its forms, then the map
  editor ([ADR-0064](0064-map-editor-with-terra-draw.md)). Where the page is at least
  84rem wide, they stand side by side, each wide enough for itself; elsewhere the
  structure comes first. The map region, set once at the start, comes last.
* For those who may edit, each position offers *Auf der Karte setzen*, or *Auf der Karte
  zeigen* once it is there. Setting opens the editor's place panel with the position
  chosen; showing chooses it on the map and centres the map on it. Focus moves to the
  editor's panel, so the next tap or *In der Kartenmitte platzieren* places it.
* *Führung – Lagekarte*, the map to read, stays as it is.
* On phones the bottom bar names the view "Struktur", since "Führung & Raum" takes two
  lines there (product owner, 2026-11-22; the name is to be revisited).

This supersedes the *Vorbereitung* row of ADR-0067's table and its rule that the command
structure is prepared with the people. ADR-0067's other parts stand.

### Consequences

* Good, because the command builds the structure and places it without changing pages.
* Good, because the tree tells which positions are not yet on the map.
* Bad, because the page is long on a phone: structure, forms, map, region. The forms stay
  below the tree, and the map is one tap away from each position.
* Neutral: *Personal* has two tabs now; the quick link *Führungsstruktur* on the overview
  opens *Führung & Raum*.

## More Information

[ADR-0031](0031-command-structure-from-doctrine.md) (command structure),
[ADR-0064](0064-map-editor-with-terra-draw.md) (map editor),
[ADR-0067](0067-command-overview-tiles-and-map-preparation.md),
requirement CMD-5.
