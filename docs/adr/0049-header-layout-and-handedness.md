---
status: accepted
date: 2026-11-10
---

# Header layout, profile menu and handedness

## Context and Problem Statement

[ADR-0026](0026-navigation-at-the-bottom-on-phones.md) put the menu at the top of the rail
on wide screens and at the end of the bottom bar on phones. The header showed the page
heading, the incident, the connection, the signed-in person and the search, in that order;
the person's settings (profile picture, appearance, change incident, unpair) were in the
menu, next to the workspaces. Practitioners asked for the layout of common web
applications, and for a bottom bar that left-handed people reach as easily as
right-handed ones.

## Considered Options

* Keep ADR-0026 as it is
* Header in the common order — menu, page heading, search in the middle, connection and
  person on the right — with the person's settings in a menu of their own; handedness as
  a setting of the device

## Decision Outcome

Chosen option (asked for by the project lead on 2026-11-10): the common header order with
a profile menu and a handedness setting. It replaces the rule of ADR-0026 on where the menu
sits on wide screens; everything else in ADR-0025 and ADR-0026 stands.

* **Wide screens (from 48 rem):** the header holds, from left to right, the menu button,
  the page heading with the incident below it, the search in the middle, the connection
  state, and the signed-in person. The rail on the left keeps only the views. The menu
  opens below its button.
* **Phones (up to 48 rem):** the header holds the page heading with the incident, the
  connection state and the person, and below them the search. The bottom bar holds the
  views and the menu: the menu at the right end, or at the left end for left-handed use.
* **Profile menu:** the person in the header is a disclosure button (picture and name). It
  opens the person's and device's settings: who is signed in, profile picture, appearance,
  handedness, change incident, unpair this device. The main menu then lists only the
  workspaces.
* **Handedness** is a setting of the device, like the colour theme: kept on the device,
  applied as `data-hand` on `<html>`, mirrored by style rules only. It changes the bottom
  bar; wide screens keep the rail on the left.
* One menu panel serves both places: on wide screens the menu button in the header opens
  it, on phones the one in the bottom bar; only the button of the current layout is shown,
  so keyboard and screen reader users meet exactly one.

### Consequences

* Good, because the layout matches what people know from other web applications, and the
  menu stays within thumb reach for both hands on phones.
* Good, because the main menu is about workspaces only; personal settings have one place.
* Bad, because two menu buttons exist in the page, of which style rules hide one: tests
  check that exactly one is visible at each width.
