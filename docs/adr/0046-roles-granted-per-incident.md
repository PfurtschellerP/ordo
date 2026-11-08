---
status: accepted
date: 2026-11-09
---

# Roles are granted per incident in ORDO, not taken from identity provider groups

## Context and Problem Statement

So far, groups of the identity provider became ORDO roles (ADR-0038): someone in the
group `medic` was a medic in every incident. In practice roles depend on the incident:
the same volunteer leads a UHS at a concert today and works on a treatment place
tomorrow; a sector leader is appointed when the incident is planned or as it grows.
Groups in an identity provider are managed by other people, change slowly and know
nothing about incidents.

## Considered Options

* Roles from identity provider groups (as before)
* Roles granted per incident in ORDO; the identity provider only says who someone is
* Both, merged

## Decision Outcome

Chosen option: "Roles granted per incident in ORDO", decided by the project lead on
2026-11-09, because only the incident's own command knows who does what in it.

* **Identity from the identity provider, roles from ORDO.** Group claims no longer grant
  roles.
* **Incidents can be planned.** An incident is created ahead (planned → active → closed),
  with a roster: the people taking part and their roles (e.g. Einsatzleiter, LNA, OrgL,
  Einsatzabschnittsleiter, UHS-Leitung, crew of a UHS). Roles and what they may do come
  from the profile and the organisation's Cedar policies (ADR-0035); positions in the
  command structure grant their roles (ADR-0031).
* **Granting and withdrawing are events of the incident**, with who, to whom, which role,
  and when; they are audited like everything else and work offline on a field server.
  The server accepts a grant only from someone allowed to assign that role in this
  incident (Cedar action `assign_role`).
* **Access follows at once:** with a new role a device receives the compartments and keys
  it now may read; when a role is withdrawn it loses them and deletes their data.
* **Organisation roles** (administrator, incident planner) are also kept in ORDO,
  managed by administrators; the first administrator is named in the server
  configuration.

### Consequences

* Good, because roles fit the incident, change during it, and every change is traceable.
* Good, because organisations need no group management in their identity provider.
* Bad, because someone has to assign roles for every incident; planned incidents with a
  roster and templates (e.g. "Sanitätsdienst Konzert": roles and UHS) keep that quick.

## More Information

Amends [ADR-0038](0038-login-and-device-pairing.md) (groups no longer become roles).
Related: offline device approval ("half-time substitution", ADR-0039) uses the same
permission to admit a person to an incident.
