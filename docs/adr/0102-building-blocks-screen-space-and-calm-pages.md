---
status: accepted
date: 2026-11-27
---

# Building blocks, screen space and calm pages

## Context and Problem Statement

By 2026-11-27 the product owner found pages too crowded and the look uneven: forms stood
open on pages where they are seldom used, explanations of two lines stood under buttons,
read-only values were set like forms, every page built its own heads, cards and lists, and
cards sat inside cards. The header took about 120 pixels on a tablet and the rail 240; on
phones header and bottom bar covered a quarter of the screen. Screen space matters most on
phones (concept *Order and calm*, decisions 3 and 4).

## Considered Options

1. A few shared building blocks, six rules for calm pages, a slim header and an icon rail;
   accents per package only after a mock-up.
2. Rework each page on its own.

## Decision Outcome

Chosen: option 1, decided by the product owner on 2026-11-27.

**Rules for calm pages** (chapter 8.6.5 and the component gallery):

1. Create and edit in a dialog: a list shows its items and one "+ Neu" at its head; nothing
   to create stands open by default.
2. Read compactly: read values as a key-value grid; a documented entry is one line, opened
   for its details.
3. One line of hint at most; longer explanations behind an ⓘ disclosure.
4. Repeated actions are icons with their word as name and tooltip; a card or row has at
   most one visible text button, the rest in a "⋯" menu.
5. Colour supports state, always with icon and word, in four tones (ok, attention, urgent,
   neutral).
6. Empty states are an icon and one sentence with the way on.

**Building blocks** (`shell::ui`, in the gallery, every theme):

* *Section head*: icon, title, number, one action at the right; close to its own rows and
  apart from the section above (more space and a thin divider; law of proximity).
* *Item row*: time and text at the left, the text taking the free width; the status tag at
  the right, so tags line up; icon actions at the far right, at the section head's edge.
  Items in a section are rows with a line between them; no card inside a card.
* *Status tag*: the four tones, icon and word.
* *Key-value grid* and *entry line*.
* *Empty state*.

**Screen space:**

* The header is one row of 48 pixels: the menu button (the ORDO mark, the ring and centre
  of the app icon without its tile, in the text colour, with the word ORDO; "Menü" as its
  name and tooltip), the path ("Führung › Lage"), the incident's name in small type, then
  icon buttons only: search, bell, connection, the person's picture.
* The rail is 72 pixels: icons with a short label under them, as the bottom bar.
* On phones the same header row holds the mark alone and the view's name; the menu stays in
  the bottom bar, which gets slimmer; cards have less padding.
* Every change is designed and checked at phone width too.

Accents per package follow only after a mock-up of two pages.

This supersedes the header layout of [ADR-0049](0049-header-layout-and-handedness.md)
(heading over two lines, search with its word in the middle); its order (menu at the
left, person at the right) and its handedness stand.

### Consequences

* Good, because pages show more content and less chrome, on phones above all.
* Good, because new pages are built from the same blocks and look alike.
* Bad, because the header's words become tooltips: a first look needs the icons learned
  (they keep their names for screen readers and in tooltips).
* Bad, because every page has to be reworked once; it is done page by page, the record and
  *Führung & Raum* first.
