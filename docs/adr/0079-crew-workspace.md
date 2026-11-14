---
status: accepted
date: 2026-11-14
---

# Crews get a workspace of their own: status, tasks, patient cases and transports

## Context and Problem Statement

A unit's crew (a medical team, *Sanitätstrupp*, or an ambulance crew) gets its work from the
incident command: tasks, patient cases (REP-3) and transports (TRANS-2). Until now the
crew saw its patient cases at the top of *UHS – Patienten*, which the product owner found
unclear on 2026-11-14, and nothing else: the command set the crew's status and moved its
tasks and transports on, on the crew's word over the radio. Roadmap item 11 asks for a
workspace for crews to see, confirm and carry out their transports and outstanding orders
and to set their own status, built on the units board of *Führung – Disposition*
([ADR-0070](0070-units-planned-in-preparation-led-in-command.md)).

Who is on which crew is incident data: the crew list of `units.crewSet`, with each member's
ORDO account where known (schema version 3). Units and tasks belong to the incident
command; Cedar lets medical roles write neither module.

## Considered Options

* A workspace for crews, found by the crew lists; medical roles may write the status and
  task updates
* A crew role per unit, given by the command, that the server checks on every write
* The crew's things inside *UHS* and *Führung*, as before

## Decision Outcome

Chosen option: "A workspace for crews, found by the crew lists".

* ***Einsatzmittel – Besatzung*** (incident permission `crew_work`, given to the medical
  roles) shows the units whose crew names the person's account. View *Aufträge*: each
  unit's status, set here like on *Disposition*; its open tasks, moved on here (*In
  Arbeit*, *Erledigt*); its open patient cases (REP-3, moved here from *UHS – Patienten*),
  with the place and "Patient aufnehmen", which opens the registration in *UHS –
  Patienten* with the case. View *Transporte*: the transports assigned to the unit, with
  their next steps (*Patient wird geladen* … *Übergeben*). Without a crew, the views say
  so.
* **Writing:** medical roles may write `units.statusChanged` and `tasks.updated` besides
  what they wrote before (`med.cedar`); giving tasks and everything else of the units stays
  with the command. Transports they could move on already.
* The app offers only the person's own units. The server cannot check that a status or a
  task is the writer's own: the crew lists are incident events, not part of the policy's
  context. Every change carries its author, and the incident log shows it.

A crew role per unit would let the server check each write, but the command would have to
give it to every crew member again, next to the crew list, and keep both in step. Leaving
the crew's things in *UHS* and *Führung* is what the product owner found unclear.

### Consequences

* Good, because a crew sees in one place what it has to do, and the command's board shows
  the crew's status and progress without a radio call.
* Good, because patient cases are where the crew works, not in the UHS's view.
* Bad, because a member of the medical service could set another unit's status or task
  through the protocol; the app does not offer it, and the log shows who did.
* Bad, because crew members without a medical role (e.g. a driver) do not get the
  workspace until a role for them exists.

## More Information

Design item *Crew workspace* in
[chapter 5](../architecture/05-building-block-view.md#crew-workspace); requirement UNIT-9.
