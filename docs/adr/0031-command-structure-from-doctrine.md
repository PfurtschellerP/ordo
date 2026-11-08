---
status: accepted
date: 2026-11-08
---

# Command structure as data, following the official doctrine

## Context and Problem Statement

Several modules (Med, Fire, command) and, within them, sub-organisations have their own
people in charge. Should ORDO model the command hierarchy, and if so, how? The question
was first checked against official doctrine ([standards register](../standards.md#command-and-control)).

What the doctrine says, in our words:

* **FwDV 100 (DE)**: the command organisation is a hierarchy that grows with the incident.
  The incident commander leads; the scene is divided into sectors (usually up to five),
  if needed into sub-sectors; each has its own leader (3.2.3). Each level leads only a few
  subordinate elements (three to five, 3.2.4). Command levels A–D add a command unit and
  finally a staff (3.2.5). A staff works in functions S1 personnel, S2 situation,
  S3 operations, S4 logistics, optionally S5 media and S6 communication (3.2.2.2).
  Specialist advisers and liaison officers of other organisations join the command.
  Documentation: incident log and situation map (3.3.5).
* **SKKM guideline (AT, 2026)**: the same staff functions; every authority and
  organisation keeps its own command organisation, and the interfaces between them
  matter most (2.3).
* **Medical response (DE)**: a medical command (Sanitätseinsatzleitung: LNA and OrgL)
  leads the medical sector with sub-sectors such as patient collection point, treatment
  area and transport organisation; details are state law and MANV concepts. DIN 13050
  defines the terms.
* **International**: ISO 22320 and the US Incident Command System describe the same
  principles: chain and unity of command, a modular structure, joint (unified) command
  of several organisations, and a limited span of control. Police doctrine (PDV 100) is
  not public.

So yes: the hierarchy is written down, and its principles agree across countries.

## Considered Options

* No hierarchy: flat roles per person
* One fixed tree in code (FwDV 100)
* Command structure as incident data, with position types from the profile; one tree per organisation, linked by joint command

## Decision Outcome

Chosen option: "Command structure as incident data", because it follows the doctrine,
fits every country through profiles, and keeps each organisation's own chain of command.

Decided on 2026-11-08:

* **Start with the medical branch** (medical command → sectors such as patient collection
  point, treatment area / UHS, transport organisation). Wherever it touches the general
  command structure (incident command, sectors, sub-sectors, staff functions, names and
  levels), it is checked against FwDV 100, so the medical branch fits into the general tree
  later without changes.
* **Access follows roles, never people.** People hold positions; positions and
  organisational functions (e.g. "medic") give **roles**; roles are allowed to do things.
  There is no direct mapping from a person to an access right. Organisations can define
  their own roles in their profile layer without changing code. Roles and permissions are
  policies evaluated by Cedar ([ADR-0035](0035-authorization-with-cedar.md)).

* **Positions** are created during the incident as it grows: incident command, sectors
  (by area or task), sub-sectors, staff functions (S1–S6) and liaison officers. The
  profile catalogue `command-structure` defines the position types and their regional
  names; core meanings follow ISO 22320 (commander, sector lead, staff function, liaison).
* **One tree per organisation.** Joint command links the trees at the top; liaison
  officers sit in another organisation's command. ORDO never merges the chains.
* **People hold positions.** A person can hold several positions; every handover is an
  event with time and both people (incident log).
* **Units and UHS are assigned to positions** (e.g. a UHS to the treatment-area lead).
* **Access follows roles.** A position gives roles; roles have permissions (e.g. the
  role of a treatment-area lead may read `med.clinical` of its UHS); the menu shows the
  workspaces of all roles a person holds (ADR-0025). This replaces the manual grants of
  the prototype.
* **Span of control** is shown, never enforced: ORDO can point out that a leader has more
  than five direct subordinates, as the doctrine suggests, but decisions stay with people.

### Consequences

* Good, because ORDO mirrors how responders already think and work.
* Good, because access rights stay correct when people hand over a position.
* Bad, because positions, assignments and handovers add events and screens.

## More Information

Glossary: [incident command](../glossary.md#incident-command-shared).
Profile concept: [architecture 8.8](../architecture/08-concepts.md#88-regional-profiles).
