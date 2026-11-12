---
status: accepted
date: 2026-11-12
---

# The map is edited on its own page, with a toolbar, taps and Terra Draw

## Context and Problem Statement

The user test on 2026-11-11 found the drawing of [ADR-0058](0058-drawing-at-the-map-centre.md)
slow and unusual: testers moved the map, then pressed "Eckpunkt setzen", and could not
correct a shape without drawing it again. Placing and drawing forms below the map also
pushed the map off small screens. The plan approved that day asked for editing on its own
page with a toolbar, with Terra Draw as the candidate. Keyboard and screen-reader users
must still be able to do everything (EN 301 549, WCAG 2.2: 2.1.1 Keyboard, 2.5.7 Dragging
Movements).

## Considered Options

* An editor page with a toolbar: new shapes corner by corner, a corner set by a tap or at
  the map's centre; existing shapes reshaped with Terra Draw's select mode
* The same, with new shapes drawn by Terra Draw's polygon and line modes
* Keep ADR-0058 and add Terra Draw only for reshaping

## Decision Outcome

Chosen option: "An editor page, corners by tap or at the centre, reshaping with Terra
Draw", decided on 2026-11-12 after the plan of 2026-11-11. It supersedes ADR-0058.
Drawing new shapes needs nothing more than a list of corners. Pointer and keyboard users
then draw the same way, with the same "Fertig" and undo buttons. Terra Draw's polygon mode
finishes a shape when its first corner is tapped again, which testers would have to learn,
and it has no way to set a corner by keyboard. Dragging corners, inserting corners
between two others and moving a whole shape are the hard parts. Terra Draw's select mode
does these well.

* **Page:** *Führung – Lagekarte* shows the map, the list of objects (A11Y-5) and, for
  those who may edit, "Karte bearbeiten". The editor is its own page (route
  `command.lead/map/edit`, so the back button works), with the map as large as the
  screen allows, a toolbar above it and the list below it.
* **Toolbar:** four tools, each with an icon and a text label, the chosen one marked
  as pressed (`aria-pressed`) and by its frame, not by colour alone.
  * *Auswählen* chooses an object by a tap on the map or "Bearbeiten" in the list.
  * *Symbol setzen* places the incident location, a unit, a UHS or a position.
  * *Fläche* draws an area.
  * *Linie* draws a line.
* **Drawing:** the tool panel takes the category from the profile (`map/drawings.toml`)
  and a name. Each tap on the map sets a corner, and so does "Ecke in Kartenmitte setzen"
  at the crosshair (the map is moved by keyboard). The map previews the shape. "Letzte
  Ecke entfernen" undoes, "Fertig" saves, "Abbrechen" discards.
* **Reshaping:** a chosen drawing goes into Terra Draw's select mode. Its corners can be
  dragged, a new corner is dragged out of the middle of a side, and the whole shape can
  be dragged. "Übernehmen" saves the new shape as one event; "Verwerfen" restores it.
  Without dragging, "Neu zeichnen" sets the corners again by tap or at the centre, under
  the same ID. This is the single-pointer and keyboard way (WCAG 2.5.7, 2.1.1).
* **Moving and placing:** a chosen symbol, or one about to be placed, goes where the
  next tap is, or to the centre with "In Kartenmitte setzen". Nothing is dragged.
* **UHS:** a unit's place is its symbol on the map, not a text field. Its details panel
  shows the coordinates and opens the editor page ready to place it (route
  `command.lead/map/place:uhs:<ID>`). Location texts written before stay visible.
* **Events:** unchanged. `map.objectPlaced` with the same object ID replaces the object
  (the latest event counts), and `map.objectRemoved` takes it off. The history stays in
  the incident log.
* **Library:** Terra Draw 1.39.0 and its MapLibre adapter 1.5.0 (both MIT, by James
  Milner), vendored from npm into `map/vendor/` like MapLibre (`vendor-map.sh`, integrity
  checked), their UMD builds loaded only when a drawing is reshaped. Its styles are the
  design tokens, read at run time.

### Consequences

* Good, because drawing is the same for tap, mouse and keyboard, and shapes can be
  corrected without drawing them again.
* Good, because the map gets the whole page while editing, and the situation view stays
  short.
* Bad, because there is one more vendored library (about 280 kB, loaded only for
  reshaping) to update by hand.
* Bad, because a shape being reshaped is shown by Terra Draw, without its category's line
  pattern, until it is saved. The tool panel names the category meanwhile.

## More Information

[ADR-0006](0006-maplibre-pmtiles.md), [ADR-0058](0058-drawing-at-the-map-centre.md),
[Terra Draw](https://github.com/JamesLMilner/terra-draw),
[05-building-block-view.md](../architecture/05-building-block-view.md) (*Drawing on the
map*, *Situation map*).
