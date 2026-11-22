---
status: accepted
date: 2026-11-22
---

# Transports follow the unit's status, and the destination may come later

## Context and Problem Statement

A transport moved through ORDO's own steps: requested, assigned, loading, departed,
arrived, handed over (TRANS-3). The product owner's feedback of 2026-11-22: a transport
unit has the same status codes as every other unit, and dispatchers think in them. The
steps should be the unit's status 3, 4, 7, 8 and then 1 or 2, not "Verladen",
"Abgefahren", "Am Ziel" and "Patient am Ziel übergeben". And when the command sends a
vehicle, the destination is often not known yet; to save time the vehicle must be on its
way to the UHS before it is.

## Decision Drivers

* Units' status codes come from the profile (FMS/TMS in DE, `units/status-codes.toml`);
  no regional codes in code (I18N-4).
* Events are immutable; a changed payload gets a new schema version.
* One status per unit: setting it on the transport card and setting it in *Disposition*
  must not disagree.
* The transport's history and its counts per hospital keep working; transports recorded
  before stay readable.

## Considered Options

1. Keep the transport's own steps and only rename them.
2. Derive the transport's state from the unit's status alone, without transport events.
3. The profile maps unit statuses to the transport's steps; setting such a status on a
   unit with an open transport also records the transport's step, with the code.

## Decision Outcome

Chosen: option 3. Option 1 keeps two status systems for one vehicle; option 2 loses who
moved the transport and breaks the history of transports recorded before.

* The transport profile (`med/transport-requests.toml`) lists the steps: a unit status
  code and the transport status it means (DE: 3 assigned, 4 loading, 7 departed, 8
  arrived, 1 and 2 handed over). The build checks that every code exists in the unit
  status catalogue.
* The transport card offers these statuses as its steps, after the last one reached. A
  step sets the unit's status (`units.statusSet`) and records `transport.updated` with
  the transport status and the code (schema version 3, field `status_code`; versions 1
  and 2 decode unchanged).
* Setting a unit's status anywhere (*Disposition*, the crew's view) moves the unit's open
  transport the same way, so the two never disagree.
* The history and the record show a step by its unit status ("7 – Patient aufgenommen");
  steps recorded before by their old words.
* Assigning needs a unit; the destination may be left open and set later on the card,
  until the unit reaches it (`transport.updated` with the destination and the status
  unchanged). Counts per hospital follow the destination once it is set.
* Cancelling stays possible until the patient is aboard (status 7).

### Consequences

* Good, because dispatchers and crews use one status system, as on the radio.
* Good, because a vehicle can be sent before the destination is decided.
* Bad, because one press writes two events (unit status, transport step); both are small
  and in the same compartment.
* Neutral: profiles without steps keep ORDO's own steps.

## More Information

Requirement TRANS-3, [ADR-0070](0070-units-planned-in-preparation-led-in-command.md)
(units and status), `profiles/de/units/status-codes.toml`.
