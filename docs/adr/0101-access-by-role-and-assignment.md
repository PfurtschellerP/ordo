---
status: accepted
date: 2026-11-27
---

# Access by role and assignment

## Context and Problem Statement

In ORDO, "role" mixed three things. *Sanität – Besatzung* opened for every medical role,
also for people on no crew, who then read that they are on none. A medical role opened the
incident's patient data (`med.clinical`) on the device, whether the person worked at a UHS
or on a crew or not; only writing followed the assignment
([ADR-0082](0082-only-a-uhss-staff-writes-it.md), [ADR-0087](0087-patients-travel-between-units.md)).
A driver without a medical role had no way into the crew workspace at all. The product
owner asked on 2026-11-27 for a model that holds as more roles and services come
(concept *Order and calm*, decision 1).

## Considered Options

1. Access = role × assignment: the role says what a person may do, the assignment where.
2. Only hide *Besatzung* without a crew; keep the rest.

## Decision Outcome

Chosen: option 1, decided by the product owner on 2026-11-27.

* **Role** (granted per incident, [ADR-0046](0046-roles-granted-per-incident.md)): what a
  person may do. Policies (Cedar), as before. New role **Einsatzkraft** (`crew`): crew work
  without patient data.
* **Qualification** (helper register): what a person has learned. It suggests and counts
  people, never grants anything: it is not checked.
* **Assignment** (incident data): on the crew of a unit (`units.crewSet`), on the staff of
  a UHS (`uhs.updated`). It changes during the incident.

Rules:

* *Besatzung* opens for people on a crew of the incident, whatever their role. Its
  *Aufträge* and *Transporte* need the permission `crew_work` (DE: every role that may be
  on a crew, Einsatzkraft included); its *Patienten* need patient data as below.
* Patient data (`med.clinical`) is read by people whose role permits it **and** who are on
  a crew or a UHS's staff of the incident, or whose role permits work at every UHS
  (`all_uhs`, DE: physicians). The server evaluates the policies with the person's
  assignments in the context (`context.assignments`, the crews and staff lists the person
  is on, valid now), so organisations keep writing their rules as policies.
* When an assignment changes the permissions of a connected person, the server closes the
  connection as for a role change (`rolesChanged`), so the device reconnects with what the
  person may do now; a device losing `med.clinical` removes its copy as today.
* Writing stays as ADR-0082 and ADR-0087 have it: the holder's people write its patients.

The device receives the incident's patient data as one compartment: sealed with one key
and hash-chained, it cannot be split by holder. A person on any crew or staff list
therefore has the data of the incident's patients on the device, and the app shows them
those of their holder. Separating them on the device would need a compartment per holder;
that is left for later and noted here.

### Consequences

* Good, because nobody without a place of work carries patient data, and a driver gets
  his crew's work without it.
* Good, because new services add roles and assignments without changing the model.
* Bad, because a connection restarts when a person's assignment changes their access.
* Neutral: within the incident, a person on one crew still has the whole clinical
  compartment on the device (above); the app limits what it shows.
