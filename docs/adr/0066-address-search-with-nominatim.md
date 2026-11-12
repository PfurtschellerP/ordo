---
status: accepted
date: 2026-11-13
---

# Addresses are searched with Nominatim, through the ORDO server

## Context and Problem Statement

The incident command enters the centre of the map region in degrees, and places objects by
moving the map. People know addresses, not coordinates. The roadmap (step 5) asks for an
address search with an open geocoder at planning time. ORDO must not send devices to
third parties directly ([ADR-0047](0047-profile-pictures.md) keeps Gravatar on the
server for the same reason), and a geocoder's terms must be kept.

## Considered Options

* The OpenStreetMap Foundation's public Nominatim, asked by the ORDO server, which an
  operator can replace with their own Nominatim or switch off
* Only an operator's own Nominatim or Photon; without one, no search
* Photon (komoot's public instance)

## Decision Outcome

Chosen option: "the public Nominatim through the ORDO server, replaceable", decided by the
product owner on 2026-11-12 ("Address search via OpenStreetMap sounds fine"). Nominatim is
open source (GPL-2.0) and searches OpenStreetMap, the data ORDO's map already shows. Its
API is the de-facto standard that many hosted and self-run geocoders also offer, so an
operator can point ORDO to another one without code changes
([ADR-0022](0022-open-standards-first.md), [ADR-0032](0032-reuse-open-source-first.md)).

* **Path:** the app sends `geocode` (incident and query) over the sync connection; the
  server asks `<ORDO_GEOCODER>/search?format=jsonv2&limit=5` and answers `geocoded` with
  up to five places (name, latitude, longitude). Devices never contact the geocoder.
* **Who:** those with `manage_map` or `edit_units` in the incident. Others get `refused`,
  and the geocoder is not asked.
* **Policy of the public Nominatim**
  ([usage policy](https://operations.osmfoundation.org/policies/nominatim/)): at most one
  request per second per server (requests queue), a User-Agent naming ORDO, its version
  and the installation's address, answers cached for 24 hours (up to 1,000 queries), and
  no search as you type: the app searches when the person presses "Suchen". Results are
  credited to OpenStreetMap (ODbL).
* **Where:** *Vorbereitung – Einsatz*, above the centre of a new map region (a place fills
  latitude and longitude), and the map editor (a place moves the map there, where
  "In der Kartenmitte platzieren" puts a symbol).
* **Configuration:** `ORDO_GEOCODER` (default `https://nominatim.openstreetmap.org`; `off`
  switches the search off), `ORDO_GEOCODER_COUNTRIES` (e.g. `de`, optional).

### Consequences

* Good, because addresses work at once, with no extra service to run.
* Good, because the policy is kept in one place, and an operator can replace the
  geocoder or switch it off.
* Bad, because what the command types (often the incident's address) goes to the
  OpenStreetMap Foundation's servers in the UK. The query is not logged by ORDO. Operators
  who may not share it set `ORDO_GEOCODER` to their own Nominatim or to `off`; the
  operator documentation says so.
* Bad, because the public service gives no guarantee; a busy or blocked server answers
  `failed`, and coordinates can still be typed.

## More Information

[ADR-0062](0062-map-region-per-incident.md), [ADR-0065](0065-map-editing-in-preparation.md),
[spec/sync/protocol.cddl](../../spec/sync/protocol.cddl) (`geocode`, `geocoded`),
[05-building-block-view.md](../architecture/05-building-block-view.md) (*Address search*).
