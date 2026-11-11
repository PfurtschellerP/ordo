---
status: accepted
date: 2026-11-11
---

# Drawing on the map: corners set at the map's centre, no drawing library yet

## Context and Problem Statement

Testing the demo asked for the incident area on the map. MAP-4 asks for areas and lines in
general: hazard zone, cordon, staging area, hose line. The map must work by keyboard and
with a screen reader too (EN 301 549, A11Y-5). Objects are already placed by moving the map
and placing at its centre (ADR-0006). The events can already carry lines and areas as
GeoJSON (`map.objectPlaced`, kind `drawing`).

## Considered Options

* Set the corners at the map's centre: move the map (arrow keys, touch or mouse), then
  "Eckpunkt setzen"
* Terra Draw (MIT), the drawing library for MapLibre
* MapLibre's former draw plug-in (Mapbox GL Draw)

## Decision Outcome

Chosen option: "Corners at the map's centre", decided on 2026-11-11. It works the same
by keyboard, touch and mouse, and needs no new library. Terra Draw draws by pointer only;
ORDO would need the centre-based way anyway for keyboard users. Mapbox GL Draw is no longer
maintained for MapLibre.

* **Categories** come from the profile (`map/drawings.toml`): id, label, area or line, and
  a line pattern (solid, dashed, dotted). The map tells categories apart by pattern and
  label, never by colour alone.
* **Drawing:** the person chooses a category and a name and sets corners. A crosshair marks
  the map's centre, and the map previews the shape as it grows. Undo removes the last
  corner. An area needs three corners, a line two.
* **Events:** `map.objectPlaced` with kind `drawing`, the category as its reference and the
  geometry as GeoJSON. Removing works as for every map object, and the list beside the map
  names each drawing.

### Consequences

* Good, because drawing is accessible from the start and adds no code to load.
* Bad, because a drawn shape cannot be reshaped by dragging its corners; it is drawn again.
  If that is needed, Terra Draw is the candidate for pointer editing, next to this way.

## More Information

[ADR-0006](0006-maplibre-pmtiles.md), [Terra Draw](https://github.com/JamesLMilner/terra-draw),
[05-building-block-view.md](../architecture/05-building-block-view.md).
