---
status: accepted
date: 2026-11-10
---

# Vim movement keys in lists and the command palette

## Context and Problem Statement

The project lead set "vim key bindings" as a guiding principle for working with the
keyboard: people who know vim, `less`, Gmail or many terminal tools move down and up with
`j` and `k`. ORDO's lists (patients, orders, transports, hospitals, units, roster,
devices, incidents) are moved through with Tab only, and the command palette
([ADR-0030](0030-command-palette.md)) with the arrow keys. Single-letter keys must not
get in the way of typing, and WCAG 2.2 success criterion 2.1.4 (Character Key Shortcuts)
allows them only if they can be turned off or remapped, or if they work only while the
component has the focus.

## Considered Options

* Arrow keys and Tab only, as now
* `j`/`k` everywhere on the page (e.g. moving a selection in the main list)
* `j`/`k` and the arrow keys inside a list while the focus is on one of its items; in the
  palette, where letters are typed, Ctrl+J and Ctrl+K

## Decision Outcome

Chosen option: the third one.

* **Lists:** a list marked for it (`data-keys="list"`) moves the focus to the next item
  with `j` or Arrow Down and to the previous one with `k` or Arrow Up, when the focus is on
  a link, button or summary inside one of its items and no modifier key is held. At the
  first or last item the focus stays, as in vim. In text fields, selects and other
  controls the keys keep their usual meaning, so typing and radio groups are unaffected.
  The keys work only while the list has the focus, which is the exception WCAG 2.1.4 makes.
* **Command palette:** letters are typed into its search, so Ctrl+J moves down and
  Ctrl+K moves up, next to the arrow keys, as in fzf and similar tools. Ctrl+K opens the
  palette from anywhere; while it is open, Ctrl+K moves up and Esc closes it. On Apple
  devices ⌘ works in place of Ctrl.
* The keys are the same in every language ([ADR-0036](0036-language-independent-shortcuts.md)).

### Consequences

* Good, because keyboard users move through long lists without Tab-ing through every
  button of every card.
* Good, because nothing changes while typing.
* Bad, because Ctrl+K no longer closes the open palette; Esc does, as for every dialog.
