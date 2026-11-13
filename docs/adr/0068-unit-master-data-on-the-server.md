---
status: superseded by ADR-0070
date: 2026-11-13
---

# The organisation's vehicles are master data on the server, managed in *Administration*

## Context and Problem Statement

Units are added to an incident from master data or ad hoc
([UNIT-1](../requirements/02-shared.md#unit-1--add-units)). So far the master data was a
TOML file on the server (`ORDO_VEHICLES_FILE`), which only whoever runs the server can
change. Roadmap item 6 (user test of 2026-11-11, plan approved the same day) asks for
unit master data in *Administration*: each vehicle as a template with its usual strength
and no crew, since the crew changes with every incident
(ORG-14, since removed).

Master data is organisation-wide and outlives incidents, while ORDO's events belong to one
incident ([ADR-0046](0046-roles-granted-per-incident.md)).

## Considered Options

* A table on the server, changed through sync messages, like the organisation roles
* An organisation-wide event stream
* Keep the file and let administrators upload it
* Take vehicles from an existing fleet or rostering system

## Decision Outcome

Chosen option: "A table on the server", because master data is a template and not a record
of what happened: `units.added` copies a vehicle's attributes into the incident, so the
incident's history never depends on the master data, and a later change touches no
incident. The organisation roles ([ORG-9](../requirements/02-shared.md#org-9--organisation-roles))
already work this way: a table, messages for administrators, fixed entries from the
configuration. An organisation-wide stream would need a new kind of stream, its own keys
and sync for data no device needs offline. Uploading a file keeps the problem of editing
it. No open standard covers vehicle master data of German aid organisations, and the
fleet and rostering systems in use (e.g. Divera 24/7, FireBoard) are proprietary; an
import from CSV ([ORG-2](../requirements/02-shared.md#org-2--csv-import)) stays possible.

* **Data:** call sign (the key), type, organisation, service from the profile, usual
  strength. No crew, no qualifications: they belong to the incident
  ([UNIT-5](../requirements/02-shared.md#unit-5--crew-members)) and later to the helper
  register.
* **Permission:** the organisation action `manage_master_data`, given to administrators
  in `admin.cedar`. Organisations give it to other roles in their own policies.
* **The file stays:** vehicles of `ORDO_VEHICLES_FILE` are listed as fixed, so an
  installation that keeps its master data elsewhere can go on generating the file.
* **Offline:** the list is asked for when the add form opens, as before. Without a
  connection, units are added ad hoc.

### Consequences

* Good, because administrators keep the master data current without access to the server.
* Good, because incidents stay self-contained: they keep the attributes a unit had when it
  joined.
* Bad, because master data is not versioned like events: the table holds the current list,
  and only the server's log shows who changed what.

## More Information

Superseded by [ADR-0070](0070-units-planned-in-preparation-led-in-command.md) on the day
it was accepted: ORDO keeps no organisation-wide vehicle master data; the table (dropped by
migration 0006), the message `changeVehicle` and the action `manage_master_data` were
removed again.
