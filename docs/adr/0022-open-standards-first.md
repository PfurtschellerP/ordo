---
status: accepted
date: 2026-11-08
---

# Open standards first

## Context and Problem Statement

ORDO must interoperate with other systems, outlive its tools and be trustworthy for public bodies.

## Considered Options

* Own formats where convenient
* Open standards wherever they exist and fit; own formats only as published open specifications

## Decision Outcome

Chosen option: "Open standards wherever they exist and fit", because standards make ORDO interoperable, reviewable and replaceable piece by piece, and avoid lock-in.

### Consequences

* Good, because data and interfaces stay usable beyond ORDO.
* Good, because contributors already know the formats.
* Bad, because a standard sometimes covers more than needed; we then use a documented subset.
* Rule: a new format, protocol or tool is checked against existing open standards first; the choice is recorded in an ADR. The list of standards in use is in architecture/04-solution-strategy.md.

## More Information

Details: [04-solution-strategy.md#43-open-standards-in-use](../architecture/04-solution-strategy.md#43-open-standards-in-use)
