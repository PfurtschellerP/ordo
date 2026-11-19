---
status: accepted
date: 2026-11-19
---

# The incident command sets up and manages every UHS

Partly supersedes [ADR-0082](0082-only-a-uhss-staff-writes-it.md): the UHS's creator
writing it while its staff list is empty, its staff changing its settings, and the view
*UHS verwalten*. Everything else in ADR-0082 stands.

## Context and Problem Statement

A medic can set up a UHS in *Sanität – UHS – UHS verwalten* and change its name, capacity,
places and staff list there. The incident command does the same in *Vorbereitung –
Einsatzmittel* (UHS-11). In the demo of 2026-11-19 the product owner asked for a clear
separation of concerns. Creating and managing a UHS belongs to the incident command. The
UHS module neither creates nor manages one. All the module needs about the other UHS is
an overview: which are full, and where else a patient could go.

## Considered Options

* Only the incident command creates a UHS and changes its settings, places and staff;
  the UHS module shows the treatment places of every UHS, read-only
* The command creates a UHS, and its staff still change its settings and places
* Keep both ways

## Decision Outcome

Chosen option: "Only the incident command creates and manages a UHS", decided by the
product owner on 2026-11-19.

* **Who sets up.** `uhs.created`, `uhs.updated` and `uhs.placesSet` (module `med.uhs`)
  are written by those who manage the incident's resources (`edit_units`, the incident
  command), on *Vorbereitung – Einsatzmittel*. The Med policy no longer lets medical roles
  write the module `med.uhs`, and the Med write guard accepts settings and staff lists
  only with `edit_units`.
* **Who writes a UHS's patients.** Its staff list and roles allowed at every UHS
  (`all_uhs`; DE default: physicians), judged by the list valid at the event's clock, as
  in ADR-0082. The creator no longer writes a UHS while its list is empty: the creator is
  now the command, which writes no clinical data. Until the command puts someone on the
  list, only roles allowed at every UHS document there.
* **The UHS module.** *UHS verwalten* is gone. Its view *Behandlungsplätze* shows every
  UHS the person may read: free and all places per triage category they are kept for,
  and the patients without a place. The incident command sees the same on *Führung –
  Übersicht*, from the patient counts, which name the taken places (IDs only) and how many
  patients have no place.
* **Places.** A UHS gets one place per patient it can take when it is set up. Changing
  its capacity sets its places again: one per patient, or per triage category where it
  has a capacity per category (UHS-13). The separate count of places goes.

"The command creates, the staff manages" keeps two places to change the same settings.
"Keep both ways" is what the product owner asked to end.

### Consequences

* Good, because one place sets up and changes a UHS, and the UHS module only shows them.
* Good, because access to a UHS follows the staff list alone, as the product owner asked
  on 2026-11-18.
* Bad, because a medic can no longer open a UHS on their own in a small incident. The
  command, or a role with `edit_units`, sets it up and puts them on its list first.
* Bad, because places can no longer differ from the capacity.
* Bad, because the patient counts get a new schema version. A device of an older version
  ignores the new fields; no release has shipped the counts.

## More Information

[UHS-1](../requirements/04-med.md#uhs-1--on-site-care-units),
[UHS-8](../requirements/04-med.md#uhs-8--treatment-places-of-every-uhs),
[UHS-10](../requirements/04-med.md#uhs-10--only-a-uhss-staff-writes-its-patients),
[UHS-16](../requirements/04-med.md#uhs-16--free-treatment-places-for-the-incident-command),
[PAT-6](../requirements/04-med.md#pat-6--patient-counts-for-command),
design items [UHS staff](../architecture/05-building-block-view.md#uhs-staff) and
[UHS overview](../architecture/05-building-block-view.md#uhs-overview).
