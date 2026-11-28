---
status: accepted
date: 2026-11-28
---

# One name per view, and rules for the navigation

## Context and Problem Statement

On 2026-11-22 the view *Führung & Raum* got a second, short label, "Struktur", for the
phone's bottom bar, where the long name took two lines. On 2026-11-28 the product owner
found that confusing: the bar said one thing, the header another. Other names meant two
places: "Transporte" and "Patienten" were views of both *Führung* or *UHS* and the crew's
workspace. Workspaces with a single view (*Einsatztagebuch*, *Info*) showed a rail with one
entry. The concept *Order and calm* (decision 2) had already asked for one rule per level
of the navigation; the concept *Struktur, Mittel, Auftrag* (decision 1) added the names.

## Considered Options

1. One name per view, used in the rail, the bottom bar, the header and the palette; names
   that differ where the places differ; no rail for a single view; checked by tests.
2. Keep short labels for the bar, with the full name as the link's name and tooltip.

## Decision Outcome

Chosen: option 1, decided by the product owner on 2026-11-28.

* A view has exactly one name. There are no short labels for views; tabs keep theirs,
  because the card under a tab carries the full name.
* *Führung & Raum* is renamed *Struktur*. Its tabs stay *Führungsstruktur · Lagekarte
  bearbeiten · Kartengrundlage* (short: *Struktur · Karte · Grundlage*).
* No two views share a name, in any language: the crew's views are *Meine Transporte* and
  *Meine Patienten*. A unit test over the registry checks it.
* A workspace with one view shows no list of views: on wide screens the page takes the
  rail's width, on phones the bar keeps only the menu.
* The other rules of *Order and calm* follow in later steps: tabs declared in the
  navigation everywhere, an item's own sections as a segmented switch.

### Consequences

* Good: what the bar says is what the header says; a name points to one place.
* Neutral: a long name may take two lines in the bar; whole words are kept where they fit.
* Bad: none known.

## More Information

* Concepts *Order and calm* (2026-11-27) and *Struktur, Mittel, Auftrag* (2026-11-28).
* Requirement UX-12, design items *Compact navigation* (chapter 5) and *Workspaces by
  phase* (chapter 8).
* Refines [ADR-0095](0095-command-and-space-on-one-page.md) (the page's name) and
  [ADR-0102](0102-building-blocks-screen-space-and-calm-pages.md).
