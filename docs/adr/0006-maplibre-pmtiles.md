---
status: proposed
date: 2026-11-07
---

# MapLibre GL JS and PMTiles for offline maps

## Context and Problem Statement

The situation map must work offline (MAP-2) with open data.

## Considered Options

* MapLibre GL JS + PMTiles (OpenStreetMap)
* Leaflet with raster tiles
* maplibre-rs (Rust)

## Decision Outcome

Chosen option: "MapLibre GL JS + PMTiles, bound from Rust via wasm-bindgen", because it is the mature open-source vector map; PMTiles packs a region into one file for offline use.

### Consequences

* Good, because regions can be downloaded per district.
* Bad, because the map is JavaScript and a canvas: it needs a thin binding layer and an accessible list view (A11Y-5).

## More Information

Details: [05-building-block-view.md](../architecture/05-building-block-view.md)
