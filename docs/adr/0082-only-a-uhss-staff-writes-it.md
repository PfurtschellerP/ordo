---
status: accepted; partly superseded by ADR-0086
date: 2026-11-18
---

# Only a UHS's staff writes it; an empty list leaves it to its creator

## Context and Problem Statement

[ADR-0056](0056-uhs-staff-write-guard.md) lets only a UHS's staff write its patients, but
a UHS without a staff list stays open to every medical role, and its settings (name,
capacity, places) are open to every medical role in any case. In the demo of 2026-11-18
the product owner put a medic on a UHS's staff list and took him off again. With him on
it, the others could only read the UHS; with nobody on it, everyone could write it again.
The product owner called that a bug: being assigned to a UHS is what gives access to it.
The same feedback asked that someone who may only read a UHS cannot even start editing.

## Considered Options

* Only the staff writes a UHS, its patients and its settings; an empty list leaves it to
  the person who created it
* The same, but the person who creates a UHS is put on its staff list
* Keep ADR-0056: an empty list is open to every medical role

## Decision Outcome

Chosen option: "Only the staff writes a UHS; an empty list leaves it to its creator". The
product owner decided on 2026-11-18 that the assignment gives access. Putting the creator
on the list would make a medic who opens two UHS count twice in the strength and show as
booked twice (UNIT-8), so the creator works there without being on the list.
It supersedes ADR-0056 where that ADR keeps a UHS without a staff list open. The rest of
ADR-0056 stands: the server's write guard judges each event by the staff list valid at
its clock.

* A UHS's patients are written by the people on its staff list and by roles allowed at
  every UHS (`all_uhs`; DE default: physicians). While nobody is on the list, the person
  who created the UHS writes them too; nobody else does.
* A UHS's settings and staff list (`uhs.updated`, `uhs.placesSet`) are changed by the
  same people and by those who manage the incident's resources (`edit_units`, the
  incident command).
* A UHS or patient the guard has not seen yet (recorded offline) cannot be judged and is
  accepted, as before.
* The app offers nothing to write where the guard would refuse it: the patient record of
  another UHS is read-only with a note why, registration lists only the person's UHS,
  and *UHS verwalten* shows another UHS's settings without the forms.

### Consequences

* Good, because access follows the assignment, as people expect in the field.
* Good, because a medic who opens a UHS can work there at once, and is counted once.
* Bad, because in a small incident other medics need a staff list before they document
  at a UHS someone else opened.
* Bad, because the creator's access is not visible on the staff list.
* Bad, because a staff entry that is not an account (a written name) still grants no
  access.

## More Information

[UHS-10](../requirements/04-med.md#uhs-10--only-a-uhss-staff-writes-its-patients),
[UHS staff](../architecture/05-building-block-view.md#uhs-staff),
[ADR-0056](0056-uhs-staff-write-guard.md).
