---
status: accepted
date: 2026-11-11
---

# Map regions are cut on the server from the open world map

## Context and Problem Statement

[ADR-0006](0006-maplibre-pmtiles.md) gives the server one map region (`/tiles/region.pmtiles`)
and leaves cutting it to the operator, with the `pmtiles` command-line tool. Testing the
demo showed this is a step few will take: an administrator wants to choose a centre and a
radius in ORDO and get the map (MAP-8).

## Considered Options

* The server cuts the region itself, with the Rust library `pmtiles` (pmtiles-rs)
* The server runs the `pmtiles` command-line tool (Go)
* The operator keeps cutting by hand

## Decision Outcome

Chosen option: "The server cuts the region itself", decided on 2026-11-11. pmtiles-rs is
maintained (Stadia Maps), MIT/Apache-2.0, and reads PMTiles over HTTP range requests or from
a file and writes them. No second program needs installing, and the server's image stays
one static binary. It extends ADR-0006 and does not replace it.

* **Source:** a PMTiles world map, `ORDO_MAP_SOURCE`, as a URL or a local file. The default
  is the Protomaps daily build of the day before (OpenStreetMap, ODbL). An installation
  without internet points it at a copy on its own network.
* **Cutting:** the tiles of every zoom level up to the source's maximum that touch the
  square around the circle. They are fetched in parallel, with the source's directories
  cached, and written in tile order into a new file. That file replaces the old one only
  when complete, so devices never get a half-written region. Radius 1–50 km: about
  3,000 tiles for 20 km.
* **Who:** the organisation permission `manage_map` (DE default: administrators and
  planners). The app asks over the sync connection (protocol 1.2: `cutMapRegion`,
  `getMapRegion`, `mapRegion`) and shows the progress. One region is cut at a time.
* **Where:** the region file is `ORDO_MAP_FILE`, by default in the server's data directory.
  Devices keep their offline copy until someone saves the map offline again.

### Consequences

* Good, because an administrator gets a map without command-line tools.
* Good, because the server's data stays in one place, and backups include the region.
* Bad, because the default source is a third-party server on the internet. The server
  sends it only range requests for tiles, no data about incidents.
* Bad, because devices with an old offline copy do not notice a new region by themselves.

## More Information

[pmtiles-rs](https://github.com/stadiamaps/pmtiles-rs),
[Protomaps basemap downloads](https://docs.protomaps.com/basemaps/downloads),
[spec/sync/protocol.md](../../spec/sync/protocol.md).
