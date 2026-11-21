---
status: accepted
date: 2026-11-21
---

# The component gallery is a page of the app

## Context and Problem Statement

DS-1 asks for one design system, documented with live examples. Chapter 8.6.5 planned a
component gallery "built with ORDO itself, served by `ordo serve --gallery`". The product
owner asked for it on 2026-11-21, to review and unify every UI element while building it.
Where does the gallery live, and what builds it?

## Decision Drivers

* The examples must be the real components, so the gallery cannot drift from the app.
* Open source first ([ADR-0032](0032-reuse-open-source-first.md)): use a story tool if one
  fits.
* Organisations that add a theme layer (DS-6) should see their theme on every component.
* The rendered-page audit (Playwright and axe-core, `tests/e2e`) should run on
  every component in every theme, without building a whole incident first.

## Considered Options

1. **Storybook** (MIT): the common story tool. It renders components of JavaScript
   frameworks; Leptos components are Rust compiled to WebAssembly and would each need a
   wrapper as a web component, plus a Node toolchain for the build.
2. **Leptos Forge** (MIT, `leptos_forge`): a story library for Leptos. One release
   (0.6.3, October 2025), about 150 downloads, and it calls itself "bleeding edge".
3. **A page of the app, `/gallery`**: the app renders its own components with sample
   data, like the licence notices page, before anyone has paired the device.
4. **A server switch, `ordo serve --gallery`**, as 8.6.5 planned: the same page, only
   served when the switch is set.

## Decision Outcome

Chosen: option 3. Storybook cannot show Leptos components without a wrapper per
component, and Leptos Forge is not maintained enough to depend on. The page shows the
real components, needs no new dependency, and is always there, so an organisation can
check its theme on every component. It holds sample data only. A switch (option 4) would
hide it from those who need it and adds nothing to protect.

* Each package adds its own examples through the package registry, as it adds its page
  sections ([ADR-0011](0011-workspaces-as-data.md)); the shell names no package.
* The page's explanations (usage, accessibility) are for developers and designers. They
  are in English and live in the gallery's code, not in `text.rs`; the examples use the
  app's real texts.
* The gallery is the page the rendered-page audit runs against in every theme.

### Consequences

* Good, because a component shows up in the gallery as it is in the app.
* Good, because the audit covers states that are hard to reach in a scenario (errors,
  empty lists, a full UHS).
* Bad, because the app gets bigger by the gallery's code; it is small, and it loads only
  sample data.
* Bad, because components that need a device's state (a paired core) show only in the
  scenario test, not in the gallery.
