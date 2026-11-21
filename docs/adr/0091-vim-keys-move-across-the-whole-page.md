---
status: accepted
date: 2026-11-21
---

# Vim keys move across the whole page

Supersedes in part [ADR-0090](0090-vim-keys-for-the-whole-app.md): what `j`, `k`, `gg` and
`G` do, and adds `h` and `l`.

## Context and Problem Statement

The product owner tried the vim keys on 2026-11-21: a good first draft, but `j` and `k`
moved only through one list, so long pages with several lists, cards side by side and forms
could not be crossed; there was no way sideways; and after `gg` or `G` the eye had to search
where the focus went.

## Considered Options

* Lists only, as in ADR-0090
* Tab order: `j` as Tab, `k` as Shift+Tab
* Spatial: `h`, `j`, `k`, `l` go to the nearest element left, below, above or right

## Decision Outcome

Chosen option: "Spatial", as in TV interfaces and the CSS spatial navigation draft.

* **Targets** are the links, buttons, summaries, check boxes, radio buttons and focusable
  areas of the page's main content that are shown. Text fields and selects are left out,
  so a move never lands where the next key would be typed; `i` and Tab reach them.
* **Nearest that way:** a target must lie beyond the current one in the direction; among
  them the closest counts, the distance across counting double (`j`, `k`) or triple (`h`,
  `l`), so a move stays in its column or row where it can. From outside the targets, `j`
  and `l` go to the first, `k` and `h` to the last.
* **`gg` and `G`** go to the first and the last target of the page.
* **Seen at once:** the target is scrolled to the middle of the screen and lights up for
  under a second besides its focus ring; without the motion where people asked for less.
* Everything else of ADR-0090 stands: off by default, per device, only outside text
  fields, no key changes data.

### Consequences

* Good, because one set of keys crosses any page, whatever its layout.
* Good, because pages whose Tab order is hard to follow can still be crossed by sight.
* Bad, because spatial moves depend on the layout: two pages that look alike move alike,
  which is the point, but a layout change changes the moves.
