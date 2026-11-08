---
status: proposed
date: 2026-11-08
---

# Federation across profiles: shared meanings, local labels

## Context and Problem Statement

Each German state introduces doctrine on its own (FwDV 100 as of 1999, an extended DV 100
in Rhineland-Palatinate, a DV 100 of 2011 in Saxony-Anhalt), sets its own status codes,
alerting rules and mass-casualty concepts ([German state sources](../standards-de-states.md)).
Across national borders the differences grow: Austria's SKKM guideline, the Netherlands'
GRIP, triage labels T1/P1/SK I. Yet a large incident at a state or national border needs
units from both sides in one incident, with one picture of the situation
([federation, architecture 8.4](../architecture/08-concepts.md#84-federation)).

How can servers with different profiles work on one incident without forcing one set of
rules on everybody, and without losing what each side meant?

## Considered Options

* One national (or European) catalogue that every installation must use
* Pairwise translation tables between every two profiles
* Shared core meanings as the contract, local labels for display, the original entry kept

## Decision Outcome

Proposed option: "Shared core meanings, local labels, original kept", because it works for
any number of profiles without pairwise tables, never changes what the sender meant, and
already matches how profiles are built (every catalogue entry maps to a core meaning,
architecture 8.8.2).

1. **The contract is the core meaning.** Every catalogue entry of every profile names a
   core meaning defined by the module that owns the catalogue (status `en_route`, triage
   `immediate`, command role `sector_lead`, staff function `s3`). Federation only promises
   that core meanings are understood everywhere.
2. **Events carry both.** An event names the stable entry ID of its author's profile
   (`de-rp.status.3`) *and* the core meaning (`en_route`). Receivers compute nothing back;
   the sender's intent is never rewritten. Events are immutable, so a later change of a
   profile cannot change old events.
3. **Profiles travel with the incident.** Each server publishes its effective profiles,
   content-addressed (hash) and signed with its server key. An incident records the
   profile hash of every participating organisation; partner servers fetch and cache them
   (public data — all German state profiles ship with ORDO, so it also works offline).
4. **Everyone sees their own labels, and the original.** A device shows a partner's unit
   status in its own words ("Status 3 – Anfahrt") and, on demand or where meanings are
   coarse, the original ("RP: Status 3"). Entries without a counterpart are shown verbatim
   with their origin, never dropped or guessed; their core meaning may be `other`.
5. **Finer mappings where needed, as data.** Where the core meaning is too coarse (vehicle
   types like HLF 20 vs. HLF 20/16, staff positions), a profile layer may declare explicit
   correspondences to entries of neighbouring profiles. They are optional and versioned
   like any profile content.
6. **Joint command follows doctrine, not a merged rulebook.** The incident's home (the
   leading organisation) sets the command structure for the incident; partners join as
   their own sectors (Einsatzabschnitte) or units with a liaison officer, as FwDV 100,
   SKKM and ISO 22320 (unified command) all foresee. Roles are mapped to core command
   roles, so permissions (Cedar policies) work across servers: a partner's "Verbindungsperson"
   gets what the federation agreement grants that core role.
7. **Need to know stays as it is.** Profiles change labels, not access: compartments,
   federation agreements and connectors decide what crosses a border.

### Consequences

* Good, because any two installations can work together on day one, by core meanings,
  without negotiating a common catalogue first.
* Good, because each side keeps its doctrine and its words; nothing is silently translated.
* Good, because cross-border cases (Austria, the Netherlands, Switzerland) use the same
  mechanism as state borders.
* Bad, because the core vocabulary becomes a contract that needs care: changes need a new
  version, and modules must define it carefully (with practitioners from several states).
* Bad, because coarse core meanings lose detail; the original entry and optional finer
  mappings compensate.

## More Information

To be confirmed with practitioners from neighbouring states before the federation phase.
First step without federation: give every catalogue entry its core meaning and carry both
in events (status codes already map to core meanings). Related:
[ADR-0009](0009-layered-data-profiles.md) (layered profiles),
[ADR-0016](0016-own-federation-protocol.md) (federation protocol),
[ADR-0031](0031-command-structure-from-doctrine.md) (command structure from doctrine).
