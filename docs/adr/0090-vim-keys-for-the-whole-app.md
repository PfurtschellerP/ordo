---
status: accepted
date: 2026-11-21
---

# Vim keys for the whole app, turned on per device

Extends [ADR-0051](0051-vim-movement-keys.md).

## Context and Problem Statement

The product owner asked on 2026-11-21 for vim key bindings in the whole app, so that power
users work fast on a desktop. ADR-0051 gave lists `j`/`k` while the focus is on one of
their items, and the command palette Ctrl+J/Ctrl+K. Moving between views and tabs, finding
the search field or jumping to the first item still takes the mouse or many presses of
Tab. Single-letter keys that work anywhere on the page conflict with screen readers'
reading keys and with speech input; WCAG 2.2 success criterion 2.1.4 (Character Key
Shortcuts) allows them only if they can be turned off or remapped, or work only while a
component has the focus.

## Considered Options

* Keep ADR-0051 only
* Vim keys always on, outside text fields
* Vim keys outside text fields, off by default and turned on per device in the menu

## Decision Outcome

Chosen option: the third one.

* **Turned on per device.** The menu's settings get *Tastatur: Standard / Vim-Tasten*,
  stored on the device like the theme and the hand. Off by default, so screen reader and
  speech input users are not surprised (WCAG 2.1.4: they can be turned off). ADR-0051's
  list keys and Ctrl+K stay as they are in both settings.
* **Normal mode** is whenever the focus is not in a text field, select or editable
  area, no modifier but Shift is held, no dialog other than the key help is open, and no
  input method is composing. In a text field Esc leaves it, as in vim; typing is never
  affected.
* **Keys**, the same in every language ([ADR-0036](0036-language-independent-shortcuts.md)):
  `?` the keys; `:` the command palette; `/` the view's search or filter field, else the
  palette; `j`/`k` the next and previous item of the page's list, from outside a list its
  first or last item; `gg`/`G` the first and last item, else the top and bottom of the
  page; `gt`/`gT` the next and previous tab; `g1` to `g4` the views of the workspace (at
  most four, [ADR-0025](0025-workspace-navigation.md)); `gm` the menu; `o` opens the
  focused item; `i` goes into the first field of the page; `d`/`u` scroll half a page.
* **Help.** `?` and a button *Tastenkürzel* in the menu open a dialog with all keys,
  shown in both settings.
* **Keys act on the page, not on data.** They move the focus, open and scroll. A key never
  saves, removes or hands over anything; those stay on buttons with their words.

### Consequences

* Good, because people who know vim, Vimium, Gmail or `less` move through ORDO without the
  mouse.
* Good, because nothing changes for anyone who doesn't turn them on.
* Bad, because a second key map has to be kept in step with the views; the key help and the
  browser test cover it.

## More Information

[UX-15](../requirements/07-quality.md#ux-15--vim-keys), design item *Vim keys* in
[chapter 5](../architecture/05-building-block-view.md#vim-keys).
