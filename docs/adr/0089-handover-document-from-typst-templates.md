---
status: accepted
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

Chosen option: "Typst template per profile", made on the server (product owner,
2026-11-21).

* **Template in the profile.** `profiles/<region>/med/handover/handover.typ`, with its
  fonts (OFL) in `fonts/` next to it; the DE profile uses Atkinson Hyperlegible Next, the
  app's typeface. An organisation's profile may replace both. The template reads the
  record as `json("/data.json")`: the header facts, the triage history, the way, the
  orders, the record's forms with time and person, the handover with receiver and note,
  and the photo. The data holds IDs, the profile's labels and what people wrote; the
  template holds the words and the layout, so the code has no language of its own. No
  logic in it rates values (EPCR-9).
* **PDF/A-3a and PDF/UA-1, with the FHIR bundle attached.** Typst 0.15 writes PDF/A-3a
  (the accessible level of PDF/A-3) and PDF/UA-1 in one file, which settles the question
  of combining them. In PDF/A-2 it cannot attach files, so the PDF is PDF/A-3. The server
  adds `pdf.attach` for the bundle to every template (relationship `Data`, `application/fhir+json`),
  so a template cannot leave it out.
* **The bundle** is an HL7 FHIR R4 Bundle of type `document`: a Composition, the Patient
  with the triage tag number, and a QuestionnaireResponse per entry of the record against
  the profile's Questionnaire ([ADR-0037](0037-epcr-forms-as-fhir-questionnaire.md)).
* **Checked.** A test compiles the DE template with sample data and checks the PDF/A and
  PDF/UA marks and the attachment; Typst itself refuses output that breaks either
  standard. In CI, veraPDF (the reference validator of the PDF Association) validates
  that sample against PDF/A-3a and PDF/UA-1.

### Where the PDF is made

On the ORDO server: the product owner decided on 2026-11-21, since a patient handed over
outside ORDO is handed over where there is a connection. The device asks for a ticket over
its sync connection and fetches the PDF over HTTP (sync protocol 1.5, HTTP API 1.2); the
server decrypts the clinical events for it and logs that it did
([ADR-0014](0014-clinical-trust-model.md)). The app stays as small as it is.

### Consequences

* Good, because each state and organisation changes the document by editing a text file
  in its profile, reviewed like any other profile change.
* Good, because Typst is open source, maintained, written in Rust and fast, and it
  produces PDF/A and tagged PDF without a browser.
* Bad, because Typst is a large dependency (about 300 crates, see
  [dependencies](../dependencies.md)); it is built into the server only, not the app.
* Bad, because a template is code of a kind: an error in it shows only when it is
  compiled, so CI must compile every template.

## More Information

[EPCR-19](../requirements/04-med.md#epcr-19--handover-outside-ordo),
[Typst PDF export](https://typst.app/docs/reference/pdf/),
[pdf.attach](https://typst.app/docs/reference/pdf/attach/),
[ADR-0022](0022-open-standards-first.md), [ADR-0032](0032-reuse-open-source-first.md).
