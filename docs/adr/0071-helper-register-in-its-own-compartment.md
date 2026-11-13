---
status: accepted
date: 2026-11-13
---

# The helper register is a compartment of its own, deleted after the incident

## Context and Problem Statement

Roadmap item 9, from the practitioner's test of 2026-11-11: the incident command registers
its helpers on *Vorbereitung – Personal* with name, address, phone, date of birth,
qualifications and the start and end of duty; crews are taken from the registered helpers;
a helper can be linked to an ORDO account. Until now a crew member is a name typed into
`units.crewSet`, which everyone in the incident reads, and ORDO keeps no personal data of
helpers.

Address, phone and date of birth are personal data (GDPR Art. 6) that the incident needs
only while it runs and for a short time after, e.g. for proof of service. They must not
reach everyone who reads the shared incident data, and they must be deleted again
([SEC-5](../requirements/07-quality.md#sec-5--gdpr)).

## Considered Options

* A compartment `people` per incident, read only by the incident command, deleted as a
  whole after a retention period
* The helpers' data in the shared `incident` compartment
* An organisation-wide member register on the server (ORG-1), referenced by incidents

## Decision Outcome

Chosen option: "A compartment `people` per incident", because it uses what compartments
already give ([8.2](../architecture/08-concepts.md#82-data-compartments)): its own access
rule, sync scope, data key and retention. Data in `incident` would reach every device of
the incident. An organisation-wide register would keep personal data of every member for
good, and the product owner decided against organisation-wide master data for units
([ADR-0070](0070-units-planned-in-preparation-led-in-command.md)); it stays possible later
as an import (ORG-1, ORG-2).

* **Module:** a shared module `helpers` (like `units`). Events in `people`:
  `helpers.registered` (a new helper with an ID and what is known) and `helpers.updated`
  (only the changed fields; a field can be cleared). The latest value per field counts, by
  hybrid logical clock.
* **Protected:** `people` is sealed like `med.clinical` (it is in `ORDO_PROTECTED` by
  default). The commander role reads and writes it; organisations change that in their
  policies.
* **Crews:** a crew member keeps the name, as before, plus the helper's ID
  (`units.crewSet` schema version 2). Nothing else leaves `people`. A device that cannot
  read `people` shows the name. Double bookings (UNIT-8) are matched by helper where the
  link exists.
* **Accounts:** a helper may name an ORDO user; the roster, the crews and the UHS staff
  lists then refer to the same person.
* **Deletion:** a closed incident's `people` stream is deleted on the server, with its
  data key, once the retention period has passed: 90 days by default, set per installation
  (`ORDO_PEOPLE_RETENTION_DAYS`). Devices are told the stream is gone and delete their
  copy. This is retention as planned in 8.2, not a change of history: the stream goes as a
  whole. Crews in `incident` keep the names, under that compartment's retention.
* **Later:** registering from the *Meldekarte* waits for its QR format (roadmap, blocked).

### Consequences

* Good, because personal data of helpers reaches only the incident command and is deleted
  when no longer needed.
* Good, because crews stay readable for everyone in the incident by name, as before.
* Bad, because the first deletion of stored events needs its own path on the server and
  on devices; until then (part 3 of item 9) `people` is kept like other streams.
* Neutral: the product owner confirmed the default of 90 days on 2026-11-13.

## More Information

Requirements HELP-1 to HELP-4 in
[chapter 4.6a](../requirements/02-shared.md#46a-shared-helpers-in-an-incident); design
items *Helper register*, *Crews from the register* and *Retention of helpers' data* in
[chapter 5](../architecture/05-building-block-view.md#helper-register).
