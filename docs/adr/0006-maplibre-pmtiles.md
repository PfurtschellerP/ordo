---
status: accepted
date: 2026-11-10
---

# MapLibre GL JS and PMTiles for offline maps

## Context and Problem Statement

The situation map must work offline (MAP-2) with open data, inside a Rust/WebAssembly app
that runs on tablets and phones and is used by keyboard and screen reader too
(EN 301 549).

## Considered Options

* MapLibre GL JS + PMTiles (OpenStreetMap, Protomaps basemap)
* Leaflet with raster tiles
* maplibre-rs (Rust)

## Decision Outcome

Chosen option: "MapLibre GL JS + PMTiles", because it is the maintained open-source vector
map (BSD-3), and PMTiles packs a region into one file that a server can serve and a device
can keep. maplibre-rs is not ready for production; raster tiles are many files and large.
First proposed on 2026-11-07, accepted with these details on 2026-11-10:

* **Libraries, vendored:** MapLibre GL JS 6.17.0, PMTiles 4.5.0 and the Protomaps basemap
  style 5.7.2, all BSD-3, taken from their npm packages (integrity checked) into
  `platform/ordo-app/map/vendor/`; label fonts (Noto Sans, OFL-1.1) from Protomaps'
  basemap assets into `map/glyphs/`. Nothing is loaded from a third-party server at run
  time; Dependabot cannot update them, so updates are a deliberate step (script
  `platform/ordo-app/vendor-map.sh`).
* **Thin glue:** ORDO's own `map/ordo-map.js` (an ES module) creates the map and shows
  ORDO's objects as GeoJSON; the Rust app imports it only when the map view opens, so the
  rest of the app does not load 1.6 MB of map code.
* **Map data:** a region of the Protomaps daily build (OpenStreetMap, ODbL) cut with
  `pmtiles extract --bbox`; the server serves it (`ORDO_MAP_FILE`) at
  `/tiles/region.pmtiles` with HTTP range requests. A device can keep the whole file in
  its Cache Storage (private to ORDO's origin and writable from the page in every browser;
  Safari cannot write OPFS files outside workers) and then reads the map from there, also
  offline. The map shows "© OpenStreetMap contributors".
* **Without a region file** the map shows ORDO's objects on a plain background, so it
  works before an operator has prepared one.
* **Accessible:** next to the canvas, a list of every object on the map (A11Y-5); objects
  are placed at the map's centre, which the keyboard moves (arrow keys), not only by
  pointing.

### Consequences

* Good, because regions can be downloaded per district and the map works without network.
* Good, because the map code is loaded only where it is needed.
* Bad, because the map is JavaScript and a canvas: it needs the glue layer and the list.
* Bad, because vendored libraries need a manual update step.

## More Information

Details: [05-building-block-view.md](../architecture/05-building-block-view.md),
[MapLibre GL JS](https://maplibre.org), [PMTiles](https://docs.protomaps.com/pmtiles/),
[Protomaps basemaps](https://docs.protomaps.com/basemaps/downloads).
