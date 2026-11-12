---
status: accepted
date: 2026-11-13
---

# The map is edited in *Vorbereitung*, and kept offline by default

## Context and Problem Statement

[ADR-0064](0064-map-editor-with-terra-draw.md) put the editor page under *Führung –
Lagekarte*, opened with "Karte bearbeiten". The product owner's feedback on 2026-11-12:
*Führung* is about conducting the running incident; editing the map belongs to
*Vorbereitung*. The same feedback asked that the map region be kept on the device by
default, saved by itself the first time the map opens, not only when someone presses
"Karte offline speichern". A region is a few hundred MB.

## Considered Options

* The editor page under *Vorbereitung – Einsatz*, which already holds the map region
* A fifth view *Lagekarte* in *Vorbereitung*
* Keep the editor under *Führung* and only rename the button

## Decision Outcome

Chosen option: "The editor page under *Vorbereitung – Einsatz*", decided on 2026-11-13
from the product owner's feedback. A fifth view would break the rule of at most four views
per workspace, which keeps the phone's bottom bar to five places
([ADR-0026](0026-navigation-at-the-bottom-on-phones.md)). *Einsatz* is where the incident
command already prepares the map region, so the map's base and its content are prepared
in one place. This supersedes the *Page* part of ADR-0064; its toolbar, tools, Terra Draw
reshaping, events and library stand.

* **Page:** *Vorbereitung – Einsatz* shows, for those who may edit the map, a card
  *Lagekarte* with "Lagekarte bearbeiten". It opens the editor page (route
  `command.prepare/incident/map`; to place one object
  `command.prepare/incident/map:<kind>:<ID>`), whose back link returns to *Einsatz*. The
  UHS details still open it ready to place their UHS.
* ***Führung – Lagekarte*** shows the map, the list of objects and the offline copy, with
  no editing. Those who conduct the incident read the map there; whoever changes it goes to
  *Vorbereitung*, also during the incident.
* **Offline by default:** the first time a device opens the map of an incident (in
  *Führung* or the editor) and the server has a region the device does not keep, the
  device saves the whole region without being asked, and asks the browser for persistent
  storage. "Karte offline speichern" stays for saving again; "Offline-Karte löschen"
  deletes the copy, and the device remembers that, so it does not save it again by itself
  for that incident.

### Consequences

* Good, because *Führung* is only about the running incident, and the map is ready
  offline without anyone thinking of it.
* Bad, because a change during the incident takes one more step (menu, *Vorbereitung*,
  *Einsatz*). If the field test finds that too slow, a shortcut from the Lagekarte can be
  added without moving the page again.
* Bad, because every device that opens the map downloads the region (a few hundred MB),
  also over mobile data. Devices are prepared on site Wi-Fi before an incident; the size
  is shown, and the copy can be deleted.

## More Information

[ADR-0064](0064-map-editor-with-terra-draw.md), [ADR-0062](0062-map-region-per-incident.md),
[05-building-block-view.md](../architecture/05-building-block-view.md) (*Situation map*).
