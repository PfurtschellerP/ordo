---
status: accepted
date: 2026-11-14
---

# No file of the installation's vehicles

Partly supersedes [ADR-0070](0070-units-planned-in-preparation-led-in-command.md): the
installation's file of vehicles goes too. Everything else in ADR-0070 stands.

## Context and Problem Statement

ADR-0070 removed the organisation-wide vehicle master data, but kept a file on the server
(`ORDO_VEHICLES_FILE`) whose vehicles the add form suggested as call signs, filling in
type, organisation, service and strength. It needed its own messages in the sync protocol
(`listVehicles` and `vehicles`, version 1.1) and a permission check on the server.

Only whoever runs the server can change the file, so it is out of date as soon as a
vehicle changes. And it is master data by another name: a list of vehicles kept apart
from incidents, which ADR-0070 decided ORDO does not need. The product owner agreed on
2026-11-14 to remove it in the architecture sweep before the first release.

## Considered Options

* Remove the file, its messages and the suggestions
* Keep the file as it is

## Decision Outcome

Chosen option: "Remove the file, its messages and the suggestions". Units are entered per
incident ([UNIT-1](../requirements/02-shared.md#unit-1--add-units)). The add form offers
the types and organisations of the units already in the incident as choices. The units of
a recurring incident are the job of incident templates (roadmap, later).

* The server no longer reads `ORDO_VEHICLES_FILE`; the variable is ignored.
* The sync protocol drops `listVehicles` and `vehicles`. Version 1.1 stays a number in
  the version table, with no message of its own: no release spoke it.

### Consequences

* Good, because there is one way to add a unit, and no list to keep up to date.
* Good, because the protocol, the server and the app lose code that only this file used.
* Bad, because an installation that kept its vehicles in the file types them again per
  incident until incident templates exist.

## More Information

Design item *Unit register* in
[chapter 5](../architecture/05-building-block-view.md#unit-register).
