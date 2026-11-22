---
status: accepted
date: 2026-11-22
---

# Führung & Raum in tabs where the page is narrow

## Context and Problem Statement

[ADR-0095](0095-command-and-space-on-one-page.md) put the command structure, the map
editor and the map region on one page, side by side where the page is at least 84rem wide
and one below the other elsewhere. It rejected tabs, since they hide one part. On phones
and tablets the page became long: the structure with its forms, then the map, then the
region. The product owner asked for tabs on 2026-11-22.

## Considered Options

1. Tabs on every screen.
2. Tabs only where the page is narrower than 84rem; side by side where it is wider.

## Decision Outcome

Chosen: option 2, decided by the product owner on 2026-11-22. Where both fit, seeing the
structure and the map together is still the point of the page.

* Narrower than 84rem (container query): tabs *Führungsstruktur*, *Lagekarte* and, for
  those who may prepare the map, *Kartengrundlage* (WAI-ARIA tabs, `shell::ui::TabList`).
* The page opens on the structure, or on the map when it was opened to place something
  (e.g. from a UHS's details).
* *Auf der Karte setzen* and *Auf der Karte zeigen* in the tree show the map's tab first.
* The map follows its element's size, so a map made in a hidden tab gets its size when
  the tab shows.

This supersedes ADR-0095's layout for narrow pages ("one below the other"); the rest of
ADR-0095 stands.

### Consequences

* Good, because the page is short on phones and tablets, and each part is one tap away.
* Bad, because on narrow screens the structure and the map are not seen at once; placing
  from the tree switches to the map.
