---
status: accepted
date: 2026-11-14
---

# The incident's people and their roles in one list and one form

## Context and Problem Statement

*Vorbereitung – Personal* had two lists: the helper register (personal data in the
compartment `people`, [ADR-0071](0071-helper-register-in-its-own-compartment.md)) and
"Rollen im Einsatz", the roster that gives access ([ADR-0046](0046-roles-granted-per-incident.md)).
The product owner found the flow hard to grasp on 2026-11-14: the same person was entered
twice, once with their data and once with a role.

There are two kinds of helpers. Members of the organisation have an ORDO account: they
should be chosen from a list, which brings their name; a Meldekarte adds the rest; they get
their role with the registration and log in on their own devices. Spontaneous helpers and
those of other organisations have no account; they need access too (a later decision).

The product owner also asked for a list of diets, a checked phone number, and the address
from the map's address search. That search sends what is typed to the installation's
geocoder, by default the public Nominatim service ([ADR-0066](0066-address-search-with-nominatim.md)):
a helper's home address would leave the installation. Decided with the product owner:
plain fields and a postcode check instead.

## Considered Options

* One list, joined by account; one form that registers and grants the role
* Two lists as before, with a link between them
* Roles kept in the register (the compartment `people`)

## Decision Outcome

Chosen option: "One list, joined by account; one form that registers and grants the role".

* **Data stays where it is:** personal data in `people`, read only by the incident
  command; roles in the roster in `incident`, which every device of the incident reads, so
  names can be shown and access decided. The list joins them by the account. Moving roles
  into `people` would hide who has which role from everyone but the command, and the server
  decides access from the roster.
* **The list** shows every helper with their roles, then everyone with a role who is not
  in the register ("Angaben ergänzen"). Without access to the register, e.g. a planner of
  another incident, it shows the roles only.
* **The form** starts with the ORDO account, chosen from the people the server knows, and
  takes the name from it; a Meldekarte adds the other fields. The role in the incident is
  chosen in the same form and granted on saving. A role needs an account; access without
  one is decided apart.
* **Address** as street, postcode and town (schema version 3 of the helper events); the
  postcode is checked against the profile's pattern. No address search: nothing about a
  helper leaves the installation.
* **Phone** in the international form of E.164, normalised with the profile's calling
  code, by a few lines of ORDO's own. A full library (ports of libphonenumber) would add
  its metadata of every country to the app, which has a size budget (PERF-5); the register
  needs only "is it a number with an area code".
* **Diet** from a list in the profile (`helpers/register.toml`), or typed in; the
  card format maps a card's wording to the list.

### Consequences

* Good, because each person is entered once, and the list says who is there and with
  which access.
* Good, because need to know stays as before: the roster says nothing personal.
* Bad, because granting a role to a person with access to the register now also registers
  them as a helper; for someone who only needs access, the personal fields stay empty.
* Bad, because the phone check knows no country's numbering plan: it accepts every number
  of the right form, also ones that do not exist.

## More Information

Design item *Helper register* in
[chapter 5](../architecture/05-building-block-view.md#helper-register); requirements HELP-1
and HELP-6.
