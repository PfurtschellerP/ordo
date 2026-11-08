---
status: accepted
date: 2026-11-08
---

# Command palette and keyboard shortcuts

## Context and Problem Statement

Under stress, a few keystrokes are faster than finding a view and a button. Users of
laptops and tablets with keyboards should reach every patient, unit, view and frequent
command directly (UX-10), similar to Spotlight on macOS.

## Considered Options

* Many single-key shortcuts
* One shortcut, Ctrl+K (⌘K on Apple devices), opening a search for everything with quick actions
* No keyboard support beyond the browser defaults

## Decision Outcome

Chosen option: "Ctrl+K / ⌘K command palette", because one shortcut is easy to learn,
is widely known from other tools, and does not conflict with screen-reader keys.

* The palette searches views, commands, patients (by triage tag number) and units (by
  call sign). Matching ignores case and umlauts; matches at word starts rank first.
* Choosing a patient or unit shows its quick actions: open the record, add a vital-sign
  entry, take a photo, re-triage; set a unit status.
* Views and commands the user may not use are shown disabled with the reason, as in the
  menu (ADR-0025).
* Accessibility: a modal `<dialog>` with a combobox and a listbox (WAI-ARIA Authoring
  Practices); arrow keys choose, Enter runs, Backspace goes back, Escape closes and
  returns focus; the number of results is announced. A visible "Search" button in the
  header opens it too, with the shortcut shown.
* No single-key shortcuts (WCAG 2.1.4). Every command is also reachable without the
  palette.
* Need to know: no search by name in the palette. Name search is a separate, logged
  function for authorised roles (PAT-7).

### Consequences

* Good, because frequent actions take two or three keystrokes.
* Good, because the palette is tested with real key presses in the browser test.
* Neutral: further shortcuts can be added later if tests with users ask for them.

## More Information

Amended by [ADR-0036](0036-language-independent-shortcuts.md): key names and command
names are English in every language.

App: `platform/ordo-app/src/palette.rs`. Standards: [register](../standards.md#accessibility-and-user-interface).
