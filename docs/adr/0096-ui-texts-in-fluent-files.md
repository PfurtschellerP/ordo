---
status: accepted
date: 2026-11-22
---

# The app's texts are Fluent files per package, read with Project Fluent's own crates

## Context and Problem Statement

The solution strategy names Project Fluent for translations (chapter 4), and I18N-1 asks
for German and English. Today about 830 text constants and 320 text functions live in Rust:
one `text.rs` for the shell and one per package (split on 2026-11-21). The product owner
decided on 2026-11-19 that the texts move to Fluent before the first release, after the
refactoring of the packages, and confirmed it on 2026-11-22.

Which library reads the Fluent files, and how does the code reach a message?

## Decision Drivers

* Open standards and open source first ([ADR-0022](0022-open-standards-first.md),
  [ADR-0032](0032-reuse-open-source-first.md)): Fluent's own format (FTL), so translators
  can use their tools (Pontoon, Weblate).
* A misspelt or missing message must fail the build, as a missing `text::…` does today.
* Every language has the same messages with the same variables; a test checks it.
* Packages bring their own texts ([ADR-0011](0011-workspaces-as-data.md)); the shell names
  no package.
* Later, profiles may change wording without recompiling (I18N-5, I18N-6): Fluent lets a
  later resource override a message.
* Few dependencies, maintained, licences compatible with the AGPL; works in WebAssembly.

## Considered Options

1. **Project Fluent's crates** (`fluent-bundle`, `fluent-syntax`; Apache-2.0
   or MIT): the reference implementation, used by Firefox's Rust code. Releases are slow (0.16, May 2025),
   commits continue (September 2026), about 9 million downloads in 90 days.
2. **`leptos-fluent`** (MIT): Leptos glue over `fluent-templates`, with macros that check
   the files. One maintainer, last release December 2025, about 4,500 downloads in 90
   days; it adds its own layer of macros and loaders.
3. **`leptos_i18n`** (MIT): active, checks keys and variables at compile time, but reads
   JSON, YAML or TOML, not Fluent.
4. **Keep Rust constants** and translate them in Rust: no standard format for
   translators.

## Decision Outcome

Chosen: option 1. It is the standard's own implementation, small, and widely used;
`leptos_i18n` gives up the format, and `leptos-fluent` adds a less maintained layer for
what a short build step does.

* **Files:** `platform/ordo-app/locales/<language>/<part>.ftl`, one per part of the app
  (`shell`, `shared`, `med`, `admin`), German (`de`) first, English (`en`) next. Message
  IDs are kebab-case, variables named (`{ $count }`); plurals and forms use Fluent
  selectors.
* **Checked at build time:** the app's build script reads the German files with
  `fluent-syntax` and writes one Rust function per message into each part's `text`
  module: `text::ws_prepare()` for a message without variables, `text::uhs_places_exceed(kept, capacity)`
  with its variables. A missing message is a compile error. A test checks that every
  other language has the same messages with the same variables.
* **At run time:** `fluent-bundle` formats the message in the device's language, chosen
  from the browser's languages, German if none fits. A message without variables is
  formatted once per language and kept, so it stays a `&'static str` where the code needs
  one. Unicode isolation marks are off for now (no right-to-left language yet).
* **Step by step:** first the build step, the run time and the shell's texts; then the
  shared modules, Med and Admin, each in its own commit; then English. Until a part has
  moved, its `text.rs` stays.

### Consequences

* Good, because translators work in Fluent files with their usual tools.
* Good, because a missing or misspelt message still fails the build.
* Bad, because the app gains five small crates (see `docs/dependencies.md`).
* Bad, because constants in `const` items (the views of a workspace) become functions;
  they are read when shown.
* Neutral: changing the wording through profiles at run time is possible later
  (`FluentBundle::add_resource_overriding`), not part of this decision.

## More Information

[Project Fluent](https://projectfluent.org/), [fluent-rs](https://github.com/projectfluent/fluent-rs),
requirement I18N-1, chapter 4 (solution strategy).
