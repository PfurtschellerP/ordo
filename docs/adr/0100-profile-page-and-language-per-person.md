---
status: accepted
date: 2026-11-27
---

# A profile page, and the language per person

## Context and Problem Statement

The person's menu at the top right held nine things by 2026-11-27: picture, display name,
roles, appearance, hand, keyboard, the shortcut list, language, my devices, change incident
and unpair. It was long, and it mixed settings of the person (name, picture), which follow
them to every device, with settings of the device (appearance, hand, keyboard), which apply
to whoever uses it, e.g. a crew's shared tablet. The language was a setting of the device
([ADR-0096](0096-ui-texts-in-fluent-files.md), I18N-1).

## Considered Options

For the settings:

1. A view *Profil* in the workspace *Person*, the menu short again.
2. Keep everything in the menu, folded into groups.

For the language:

1. Per device, as built.
2. Per person, following them to every device they pair; no override on the device.
3. Per person, with an override on the device.

## Decision Outcome

Chosen: a view *Profil* and the language per person (option 2), decided by the product
owner on 2026-11-27 (concept *Situation, units and profile*, decisions 3 and 4).

* *Person – Profil* comes before *Person – Geräte*. It has two parts, each saying where it
  applies: *Ich* (on all my devices: picture, display name, language, the roles in this
  incident) and *Dieses Gerät* (for everyone using it: appearance with high contrast, hand,
  vim keys as a check box, the shortcut list, the triage aid, and signing out). On phones it
  is one column; from 60rem the two parts stand side by side.
* The menu keeps who is signed in, the roles in this incident, the appearance as radio
  chips *Auto · Tag · Nacht* (one tap at dusk; *Auto* follows the device's light or dark
  setting), and links: *Profil und Einstellungen*, *Meine Geräte*, *Einsatz wechseln*,
  *Abmelden*.
* *Abmelden* replaces *Gerät entkoppeln*: signing out unpairs the device, so the device must
  be paired again; the page says so under the button.
* Every device in ORDO is paired to one person, so the person's language applies wherever
  they are signed in. The server keeps it with the display name (`setLanguage`, protocol
  1.6) and tells it in the welcome; the device keeps a copy for offline starts. Before
  sign-in (pairing, the login page) the browser's language applies. There is no override
  on the device.

This supersedes ADR-0096's choice of the language on the device; the rest of ADR-0096
stands.

### Consequences

* Good, because the menu is short, and the profile page says what follows the person and
  what stays with the device.
* Good, because a person sees their language on every device, also on a tablet they borrow.
* Bad, because a person who wants another language on one device changes it for all.
* Neutral: the language is sent like the display name, so it needs the server once; a
  device without connection keeps the last known one.
