---
status: accepted
date: 2026-11-11
---

# A UHS's patients are written by its staff, checked by the server per event

## Context and Problem Statement

Every person with a medical role may read and write the whole clinical compartment of an
incident, so anyone can register or document patients at every on-site care unit (UHS).
In the field each crew works at one UHS. A record written at the wrong UHS confuses counts
and responsibility (UHS-10). ORDO is offline-first: a device may document for hours before
it syncs, and documentation must never be lost because the server refuses it later.

## Considered Options

* The server checks each clinical event against the UHS's staff list valid at the event's
  time (a write guard in the hub, next to the derivers)
* One clinical compartment per UHS, so that devices only receive and write their UHS
* Only the app restricts input; the server accepts everything

## Decision Outcome

Chosen option: "The server checks each event", decided on 2026-11-11, because it enforces
the rule with a small change. The server already opens clinical payloads (it derives the
counts, [ADR-0001](0001-local-first-event-sourcing.md)):

* A package may register a **write guard** for compartments. The hub asks it after the
  role check and before ordering an event. Guards rebuild their state from the stored
  events, as derivers do.
* The Med guard knows each UHS's staff lists over time (`uhs.updated`) and each patient's
  UHS over time (registration, moves). It accepts a clinical event when the patient's UHS
  had no staff list at the event's time, or when the author was on it then. It also
  accepts events about a patient it does not know yet, which happens offline, and events
  from people whose roles allow every UHS (permission `all_uhs`; DE default: physicians).
  A move is judged by the UHS the patient leaves.
* Once a UHS has a staff list, only its staff and roles allowing every UHS may change it.
* The app shows the person's working UHS at the top of the UHS module (UHS-9) and does not
  offer writing where the rule would refuse it, so a refusal stays the exception.
* Judging by the event's clock, not by its arrival, keeps documentation made offline
  valid when the staff list changes in between. A device whose clock is wrong is already
  refused (`ClockAhead`).

### Consequences

* Good, because responsibility per UHS is enforced without changing compartments or keys.
* Good, because an empty staff list keeps small incidents as simple as before.
* Bad, because every medical device still receives every UHS's patients. One clinical
  compartment per UHS would also limit reading (need to know). It is on the roadmap as a
  separate step.
* Bad, because a staff entry that is not an account (a written name) grants no access.

## More Information

[UHS-9, UHS-10](../requirements/04-med.md#uhs-9--working-uhs),
[05-building-block-view.md](../architecture/05-building-block-view.md),
[ADR-0035](0035-authorization-with-cedar.md).
