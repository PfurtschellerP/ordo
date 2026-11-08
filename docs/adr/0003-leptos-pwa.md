---
status: proposed
date: 2026-11-07
---

# Leptos as UI framework; PWA first

## Context and Problem Statement

The client must run on tablets, laptops and phones, offline, and be accessible (A11Y-4).

## Considered Options

* Leptos (DOM, fine-grained reactivity)
* Dioxus
* Yew
* Canvas-based UI frameworks

## Decision Outcome

Chosen option: "Leptos, delivered as a Progressive Web App; Tauri 2 as native shell later", because it renders real HTML (needed for screen readers and zoom) and updates only what changes.

### Consequences

* Good, because one codebase serves all devices without app stores.
* Bad, because browser storage limits (especially iOS) need a native shell later.
* Neutral: canvas UIs are ruled out for accessibility.

## More Information

Details: [08-concepts.md#86-design-system-and-accessibility](../architecture/08-concepts.md#86-design-system-and-accessibility)
