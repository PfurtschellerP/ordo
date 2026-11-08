---
status: accepted
date: 2026-11-08
---

# ePCR forms as an HL7 FHIR Questionnaire

## Context and Problem Statement

[ADR-0027](0027-epcr-forms-from-the-profile.md) made the ePCR forms profile data, in an own
TOML format. An open standard exists for exactly this: the HL7 FHIR `Questionnaire`
(ADR-0022), and existing open-source form editors and renderers work with it (ADR-0032).

## Considered Options

* Keep the own TOML format
* FHIR R4 `Questionnaire`, in a documented subset
* FHIR R5 `Questionnaire`

## Decision Outcome

Chosen option: "FHIR R4 `Questionnaire` in a documented subset", because R4 is the version
used by German health-care interfaces and most tools, and a subset keeps ORDO's reader
small.

* One questionnaire per profile; one group per form; a repeating group is a series form.
* Units in UCUM, codes in LOINC, FHIR element definitions for later extraction.
* Sums only, via the SDC `calculatedExpression`, restricted to a plain sum of answers of the
  same group (EPCR-9). Everything else that changes meaning is rejected.
* The subset is an open specification: [spec/profiles/questionnaire-subset.md](../../spec/profiles/questionnaire-subset.md).
* The app checks the questionnaire at build time and embeds ORDO's internal form model;
  events and the record model are unchanged.

### Consequences

* Good, because hospitals and other systems can read ORDO's form definitions, and form
  editors for FHIR can be used instead of editing files.
* Good, because the later handover as `QuestionnaireResponse` or FHIR resources needs no
  mapping table.
* Bad, because the JSON is verbose for hand editing.
* Neutral: conditional questions (`enableWhen`) come later.

## More Information

Amends ADR-0027 (format of the catalogue). Reader: `packages/med/patients/src/questionnaire.rs`.
