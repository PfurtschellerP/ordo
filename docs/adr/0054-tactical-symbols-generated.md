---
status: accepted
date: 2026-11-10
---

# Tactical symbols: generated at build time with `@taktische-zeichen/core`, kept in the profile

## Context and Problem Statement

The situation map and the lists beside it show units, UHS, command positions and the
incident location with tactical symbols (MAP-3). The symbols are regional doctrine: in
Germany DV 102 (*Taktische Zeichen*), with the BBK's recommendations of 2024; other
countries use other systems. Rule 6 keeps them out of the platform. Drawing them by hand
is slow and error-prone; loading a symbol library into the app adds code to every device.

## Considered Options

* Generate SVG files at build time with `@taktische-zeichen/core` and keep them in the
  profile
* Run `@taktische-zeichen/core` in the app
* Draw or collect SVG files by hand (e.g. from the Wikimedia Commons collection)

## Decision Outcome

Chosen option: "Generate at build time, keep in the profile", because the maintained
open-source library (MIT, Philipp Jardas, follows DV 102) draws every combination from a
few parameters, while the app stays free of it and the profile stays plain data:

* `profiles/de/symbols/symbols.json` describes each symbol by DV 102 parameters and says
  which map object gets which symbol; one SVG file per symbol sits next to it.
* `tools/symbols` (Node, library version pinned, Dependabot keeps it current) writes the
  files; CI checks that the files are what it generates. Each file carries the library's
  licence notice.
* The app embeds the files at build time, so symbols work offline with no extra request.
* Where the library cannot draw a sign of the 2024 edition yet, the profile keeps a file
  drawn by hand, its source named in the file (no `spec`; the check only wants the file).
* Without the embedded font, a few labels fall back to the device's serif font.

### Consequences

* Good, because a symbol is one line of data, and another profile can bring its own
  files, generated or drawn.
* Good, because the app does not load the library.
* Bad, because a library update can change the drawings; CI shows it as a diff to review.
* Bad, because the library follows DV 102 in parts older than the BBK's 2024 edition; the
  profile records the comparison and draws the differences by hand.

## More Information

Details: [05-building-block-view.md](../architecture/05-building-block-view.md),
[taktische-zeichen on GitHub](https://github.com/phjardas/taktische-zeichen),
[ADR-0006](0006-maplibre-pmtiles.md) (the map), [ADR-0032](0032-reuse-open-source-first.md).
