---
status: proposed
date: 2026-11-21
---

# The handover document is made from a Typst template in the profile

## Context and Problem Statement

A patient handed over outside ORDO needs a handover document
([ADR-0087](0087-patients-travel-between-units.md), EPCR-19): a PDF/A-3 that people read,
with the same data embedded machine-readably as an HL7 FHIR bundle. The product owner
confirmed PDF/A-3 on 2026-11-21 and asked whether a Typst template could make it, so that
the layout is part of the profile and each organisation can adapt it. What a hospital or
control centre expects to read differs by state and organisation; under AGENTS.md rule 6
it belongs in profiles, not in code.

## Considered Options

* A Typst template per profile, compiled by the `typst` crates (Apache-2.0) with the
  record as data
* A PDF drawn in code (e.g. `printpdf`), with a fixed layout
* An HTML page printed by the browser

## Decision Outcome

Proposed option: "Typst template per profile".

* **Template in the profile.** `profiles/<region>/med/handover.typ`, with its fonts
  (OFL) next to it. An organisation's profile may replace it. The template gets the record
  as JSON (`sys.inputs` or a data file): header facts, entries with time and person,
  orders, the way, the handover with receiver and note. It contains only layout; no logic
  that rates values (EPCR-9).
* **PDF/A-3b with the FHIR bundle attached.** Typst writes PDF/A-3b (`--pdf-standard
  a-3b`). In PDF/A-2 it cannot attach files, so the PDF must be PDF/A-3. It attaches files
  with `pdf.attach` (formerly `pdf.embed`), with relationship `data` for the FHIR bundle
  (JSON).
* **Accessible.** Typst also writes tagged PDF/UA-1. Whether one file can be PDF/A-3b and
  PDF/UA-1 at once is to be checked. If not, PDF/A-3b comes first: it is the format for
  archiving and for systems that read the bundle.
* **Checked in CI.** `ordo-profiles check` compiles each template with sample data, and
  veraPDF (open source) validates the result.

### Open question for the product owner

**Where is the PDF made?**
* **On the device:** Typst compiled to WebAssembly, so it works offline. That adds several
  megabytes to the app plus the fonts.
* **On the ORDO server:** smaller and simpler, but it needs a connection to the server at
  the scene.

The developers propose the server first, with the device to follow if field tests show the
need.

### Consequences

* Good, because each state and organisation changes the document by editing a text file
  in its profile, reviewed like any other profile change.
* Good, because Typst is open source, maintained, written in Rust and fast, and it
  produces PDF/A and tagged PDF without a browser.
* Bad, because Typst is a large dependency: its crates and fonts need the dependency
  analysis before they are added.
* Bad, because a template is code of a kind: an error in it shows only when it is
  compiled, so CI must compile every template.

## More Information

[EPCR-19](../requirements/04-med.md#epcr-19--handover-outside-ordo),
[Typst PDF export](https://typst.app/docs/reference/pdf/),
[pdf.attach](https://typst.app/docs/reference/pdf/attach/),
[ADR-0022](0022-open-standards-first.md), [ADR-0032](0032-reuse-open-source-first.md).
