---
status: accepted
date: 2026-11-11
---

# A map region per incident

## Context and Problem Statement

[ADR-0057](0057-map-region-cut-on-the-server.md) gives the server one map region, cut by
someone with the organisation permission `manage_map`. One organisation runs several
incidents at the same time in different places, so one region per server does not fit
(decided with the product owner on 2026-11-11). The testers also found the map region under
"Organisation", although preparing the map is incident work
([ADR-0061](0061-workspaces-by-phase-and-administration-apart.md)).

A region also shows where an incident is. With one region per incident, its file should
reach only devices of people in that incident (need to know).

## Considered Options

* One region per incident, served only with a ticket from the sync connection
* One region per incident, served to anyone who knows the incident ID
* Keep one region per server, large enough for all incidents

## Decision Outcome

Chosen option: "One region per incident, served only with a ticket", because it keeps
regions small (a region is a few MB to a few hundred MB) and follows need to know as
attachments do ([ADR-0033](0033-attachments-in-an-object-store.md)). It amends ADR-0057 in
*where* and *who*; source and cutting stay as decided there.

* **Who:** the incident command of that incident, by the incident permission `manage_map`
  (Cedar, DE default: role *Einsatzleitung*). Planners plan incidents and lead them, so
  they cut regions as before. The organisation permission `manage_map` is no longer used.
* **Where:** one file per incident in the server's data directory
  (`maps/<incident ULID>.pmtiles`), replaced only when complete. A new cut replaces the
  incident's region. One region is cut at a time on a server.
* **Protocol** (sync 1.2, not yet released): `getMapRegion`, `cutMapRegion` and `mapRegion`
  name the incident.
* **Reading:** a device asks for a ticket over its sync connection (`mapTicket`, with the
  incident) and gets one if it may read the incident's shared compartment. The map reads
  `/tiles/<incident ULID>.pmtiles` with range requests and `Authorization: Bearer <ticket>`;
  a ticket is valid for 12 hours and asked anew on every connection.
* **Offline:** *Karte offline speichern* keeps the incident's region on the device, one per
  incident; the map uses it first, as before.
* **No region:** the map shows the objects on a plain background, and the *Lage* view says
  that no map region was prepared.

### Consequences

* Good, because incidents in different places each get their map, and a region reaches
  only the people of its incident.
* Good, because preparing the map moves to the incident command's preparation, where the
  testers looked for it.
* Bad, because an incident in the same place as an earlier one needs its region cut again;
  copying a region from another incident can come later.
* Neutral: there is no released installation yet, so the server-wide region of ADR-0057
  is not migrated.

## More Information

[ADR-0057](0057-map-region-cut-on-the-server.md), [ADR-0006](0006-maplibre-pmtiles.md),
[spec/sync/protocol.md](../../spec/sync/protocol.md).
