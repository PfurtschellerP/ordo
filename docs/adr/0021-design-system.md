---
status: proposed
date: 2026-11-08
---

# Design system on W3C design tokens

## Context and Problem Statement

All packages need one consistent, themeable look that cannot break accessibility (DS-1…7).

## Considered Options

* CSS framework (e.g. Tailwind)
* Style Dictionary build
* DTCG tokens with an own Rust tool (ordo-tokens), protected safety tokens, own components following WAI-ARIA APG

## Decision Outcome

Chosen option: "DTCG tokens with ordo-tokens, protected safety tokens, own accessible components", because the token format is a standard, the build stays Rust-only, and contrast is checked automatically.

### Consequences

* Good, because no theme can break contrast or change triage colours.
* Bad, because components are built ourselves until a Leptos library proves itself.

## More Information

Details: [README.md](../../design/README.md)
