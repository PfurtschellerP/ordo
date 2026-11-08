---
status: accepted
date: 2026-11-08
---

# Patient care record: forms from the profile, values as events

## Context and Problem Statement

Medical staff expect a complete patient care record: identity, history, findings,
vital signs, measures, medication, disposition. Its structure differs by country and
organisation (EPCR-2), records must never be overwritten (EPCR-4), and ORDO must not
interpret clinical values (EPCR-9).

## Considered Options

* Fixed Rust types per form
* Forms as profile data (catalogue `epcr-forms`); the module knows only field types; each saved form is one event
* A generic document per patient, overwritten on every change

## Decision Outcome

Chosen option: "Forms as profile data, one event per saved form", because organisations
can adapt forms without code, and events keep the full history.

* A form is a *snapshot* (one current value per field, e.g. identity) or a *series*
  (timed entries, e.g. vital signs). A correction of a series entry is a new entry that
  names the entry it replaces.
* Field types: text, long text, number (with unit and input limits), choice, multiple
  choice, date, time. Input limits catch typing errors; they are not clinical limits.
* The only calculation is a sum of documented values that the profile defines (e.g. GCS).
* Fields carry codes of open terminologies (HL7 FHIR element, LOINC) for the later
  handover export.
* The app checks the profile at build time and embeds it. Later profiles come from the
  server as data (architecture 8.8).
* Events live in the protected compartment `med.clinical`.

### Consequences

* Good, because a new field or form is a profile change, checked by tests.
* Good, because every value keeps author, time and history.
* Bad, because cross-field rules (e.g. "dose needs a unit") need an extension of the
  catalogue format.

## More Information

The catalogue format is an HL7 FHIR Questionnaire since [ADR-0037](0037-epcr-forms-as-fhir-questionnaire.md).
Profile: `profiles/de/med/epcr-forms.json`. Module: `packages/med/patients/src/record.rs`.
Standards: [register](../standards.md#medical-response-and-patient-records).
