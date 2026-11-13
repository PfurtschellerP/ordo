---
status: accepted
date: 2026-11-13
---

# A UHS counts as a unit on scene, without a second record

## Context and Problem Statement

In the user test of 2026-11-11 the testers found that a UHS stands apart from the units
(*Einsatzmittel*), so the strength on scene (*Kräfte vor Ort*,
[UNIT-4](../requirements/02-shared.md#unit-4--strength-on-scene)) leaves out the UHS and
the people working there. Roadmap item 6 (plan approved the same day): "a UHS is a unit,
so it counts in the forces on scene" ([UHS-12](../requirements/04-med.md#uhs-12--uhs-count-as-units-on-scene)).

A UHS is defined by the Med package (`uhs.created`, `uhs.updated`: name, kind, capacity,
staff list, [ADR-0056](0056-uhs-staff-write-guard.md) for who writes its patients). Units
are the shared module `units` (`units.added`: call sign, type, organisation, service,
strength, crew). Packages build on shared modules, not the other way round.

## Considered Options

* The Med package counts each UHS into the strength on scene, with organisation, service
  and strength kept on the UHS
* Creating a UHS also writes `units.added`, so it is a unit in the units register too
* Count only the UHS staff lists, as people without a unit

## Decision Outcome

Chosen option: "The Med package counts each UHS into the strength on scene", because the
UHS keeps one record. Writing `units.added` as well would give the same UHS two names, two
lists of people (its staff list and a crew) and two ways to be released, which drift apart
offline. Counting staff only would leave out the UHS as a unit and its organisation.

* `uhs.updated` schema version 3 gives a UHS what a unit has besides its call sign and
  type: organisation, service (from the profile) and strength. The UHS's name stands for
  the call sign, its kind for the type.
* The shared module offers `StrengthOnScene::count` for one unit; the Med package adds
  every UHS through it. The units module knows nothing of UHS.
* A UHS without a strength counts with its staff list as Mannschaft, because the list
  names the people who work there; without both it counts as unreported, like a unit.
* People who are on a UHS's staff list and also the crew of a unit on scene are counted
  twice. The command avoids it by not adding the same people as a unit; the helper
  register (roadmap item 9) will tie people to one place.

### Consequences

* Good, because the strength on scene includes the UHS without a second record or a new
  dependency.
* Good, because older `uhs.updated` events stay valid; a UHS without the new fields counts
  with its staff.
* Bad, because a UHS has no unit status (FMS) and is not released like a unit; it stays as
  long as the incident.

## More Information

Design item *UHS as a unit* in
[chapter 5](../architecture/05-building-block-view.md#uhs-as-a-unit).
