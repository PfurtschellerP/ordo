---
status: accepted
date: 2026-11-08
---

# Shortcuts and command names are the same in every language

## Context and Problem Statement

Translated shortcuts and command names break muscle memory, documentation and training
across countries; spreadsheet formulas translated per language are a well-known example.
ORDO serves several countries and organisations that work together at one incident.

## Considered Options

* Translate everything, including key names and command names
* Keys and command names in English everywhere; labels translated; search finds both

## Decision Outcome

Chosen option: "Keys and command names in English everywhere", amending
[ADR-0030](0030-command-palette.md):

* Shortcuts use the same keys in every language and are never remapped per language.
* Key names are shown in English (Ctrl, Enter, Esc), with ⌘ on Apple devices.
* Every command has a stable English name (e.g. `register patient`, `retriage immediate`,
  `status on scene`, `vitals`). The command palette always finds it, in addition to the
  label in the user's language. Training material can use the English names.
* Unit statuses are found by number and by the English core meaning (`on scene`).

### Consequences

* Good, because typed commands work the same in every country and every UI language.
* Neutral: the English names are a small vocabulary that may need explaining once.

## More Information

App: `platform/ordo-app/src/palette.rs`.
