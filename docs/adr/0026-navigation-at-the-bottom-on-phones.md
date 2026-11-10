---
status: accepted
date: 2026-11-08
---

# Navigation at the bottom on phones, menu included

## Context and Problem Statement

[ADR-0025](0025-workspace-navigation.md) put the workspace menu (launcher) at the top
left and the views in a bottom bar on phones. On phones the top left is the hardest place
to reach with one hand. Current mobile platforms put the main navigation at the bottom.
The prototype also had a layout bug: the fixed bottom bar could cover content.

## Considered Options

* Keep the menu at the top left on all screens
* Menu and views together in one bottom bar on phones and tablets in portrait; a rail on wider screens
* Bottom bar on every screen size

## Decision Outcome

Chosen option: "Menu and views together in one bottom bar on narrow screens", because
it puts everything that is used often within thumb reach and follows the platforms people
know (Android/Material: navigation bar on compact widths, rail on wider ones; iOS: tab bar
at the bottom).

Rules:

* Up to 48 rem window width (phones, small tablets in portrait): one bar at the bottom
  with the views and, at its end, the menu. The menu opens upwards as a sheet above the
  bar. The content keeps space free below, so the bar never covers it.
* Wider screens: a rail on the left with the menu at its top and the views below. The rail
  runs the full height, and its content stays in view while scrolling.
* The header shows the current workspace as the page heading (`h1`), the search
  ([ADR-0030](0030-command-palette.md)), the incident and the connection state.
* The rules of ADR-0025 on availability (disabled with reason, hidden when the package is
  not used) are unchanged.

### Consequences

* Good, because the most-used controls are within thumb reach on phones.
* Good, because the rendered-page audit now checks that fixed bars cover no content.
* Neutral: the breakpoint can be moved if tests with volunteers show that larger tablets
  should also use the bottom bar.

## More Information

Amends ADR-0025 (position of the menu). Sources: [standards register](../standards.md#accessibility-and-user-interface).
Design item: [5.5.3 Web app](../architecture/05-building-block-view.md#553-web-app-ordo-app).

Where the menu sits on wide screens, the profile menu and the bar for left-handed use are
decided in [ADR-0049](0049-header-layout-and-handedness.md).
