---
status: accepted; its workspace table and overview rule superseded by ADR-0067, its later Stammdaten by ADR-0070
date: 2026-11-11
---

# Workspaces by phase; administration apart from the incident

## Context and Problem Statement

Two testers and a practitioner used the demo on 2026-11-11. The command's pages had grown into one workspace
*Lage* with four crowded views. The incident log was a package of its own ("Einsatz").
The package "Organisation" held both administrative functions (accounts, organisation
roles, all devices) and incident work (the map region). The testers found roles in two
places, "Organisation" and "Lage", and could not tell them apart. They asked for:

* an overview first, with a short summary of each page, direct links and quick actions;
* pages grouped by what comes first: preparing the incident, then working the situation;
* orders, the map editor and people and roles on pages of their own;
* administration (server access beyond one incident) apart from the incident, so the
  incident command does not see it.

The launcher groups workspaces by package, and a workspace has at most four views, so it
fits the bottom bar on phones ([ADR-0025](0025-workspace-navigation.md),
[ADR-0049](0049-header-layout-and-handedness.md)).

## Considered Options

* One workspace *Einsatzleitung* with all pages, grouped by phase in the rail, and a
  "more" menu on phones (supersedes the four-view limit)
* Several workspaces in the package *Einsatzleitung*, one per phase, each with at most
  four views

## Decision Outcome

Chosen option: "Several workspaces, one per phase", decided on 2026-11-11 with the
product owner. It keeps the navigation of ADR-0025 and ADR-0049 as it is; the launcher
already shows a package's workspaces as one group.

**Einsatzleitung** (package):

| Workspace | Views | Who |
|---|---|---|
| Führung (the running incident) | Übersicht · Lagekarte (with the page *Karte bearbeiten*) · Disposition (reported events, patient cases, orders) · Transporte | the incident command (`view_situation`) |
| Vorbereitung (set up before, adjusted during) | Einsatz (data, location, map region, start, close) · Einsatzmittel · Personal (helper register, roles in the incident, devices in use) · Führungsstruktur | the incident command |
| Einsatztagebuch | Einsatztagebuch | everyone in the incident; each device shows what its user may read (LOG-6) |

*Planung* was considered instead of *Vorbereitung* and not taken: in FwDV 100 it is a phase
of the command cycle during the incident (assessing and deciding), and *Einsatzplanung* is
already an organisation role. *Lage* stays where doctrine uses it: the *Lagekarte*.

**UHS** (package Sanität): unchanged, the patient list stays first, because registering is
the main task in a mass casualty incident; its *Übersicht* stays one tap away.

**Administration** (package, replaces "Organisation"): workspace *Zugriff* with *Konten*,
*Berechtigungen* (the organisation roles, renamed so they are not confused with the roles
in an incident) and *Geräte* (all devices). Only organisation permissions open it, never an
incident role; each view needs its own permission (`manage_accounts`, `manage_roles`,
`manage_devices`). Later: *Stammdaten* (ORG-1).

Rules:

* **Overview first.** The command's *Übersicht* has one short summary per page of the
  package (e.g. units on scene, open orders, devices, the last log entries), each linking
  to its page, and quick actions (start the incident, add a unit, give an order). It is a
  pattern for other packages: an overview with quick actions, details on their own pages.
* **Devices in the incident.** The incident command sees and blocks the devices used in
  its incident (on the page *Personal*); the full list stays with Administration.
* **The map region belongs to the incident**, not to the server: see
  [ADR-0062](0062-map-region-per-incident.md).

### Consequences

* Good, because administration and incident work no longer mix, and the incident command
  finds its pages in the order it uses them.
* Good, because the navigation keeps working on phones without a "more" menu.
* Bad, because the incident command switches between two workspaces, e.g. to register a
  helper who arrives during the incident; the overview's quick actions and links make that
  one tap each.
* Neutral: routes change (`/command.situation/units` becomes `/command.prepare/units`).
  Devices that open an old route land on the workspace's first view.

## More Information

User test feedback and plan: [roadmap](../roadmap.md), "Now". Navigation model:
[5.5.3 Web app](../architecture/05-building-block-view.md#553-web-app-ordo-app).
