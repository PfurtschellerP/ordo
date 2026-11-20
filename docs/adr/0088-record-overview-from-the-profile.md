---
status: accepted
date: 2026-11-20
---

# The record overview comes from the profile

## Context and Problem Statement

The patient record lists every section one below the other: place, photo, triage,
orders, stay, eight forms and the transport. The product owner found it too crowded to
see the patient's state at a glance (2026-11-19) and accepted a concept with a header,
quick actions, an overview of tiles and tabs (2026-11-20, EPCR-14). Which forms get a
tile or a quick action, which fields belong in the header and what their short labels
are is regional and organisational: it must come from the profile, not from code
(AGENTS.md rule 6). The forms are an HL7 FHIR Questionnaire
([ADR-0037](0037-epcr-forms-as-fhir-questionnaire.md)).

## Considered Options

* Standard extensions where they exist (SDC `shortText` for short labels) and one ORDO
  extension that marks forms and fields for the overview
* Fixed choices in code (vital signs, allergies by their field IDs or LOINC codes)
* A separate catalogue next to the Questionnaire

## Decision Outcome

Chosen option: "Standard extensions plus one ORDO extension".

* **Short labels:** `http://hl7.org/fhir/uv/sdc/StructureDefinition/sdc-questionnaire-shortText`
  (`valueString`) on an item, e.g. "HF" for *Herzfrequenz*. Without it the label is used.
* **Overview marks:** `urn:ordo:fhir:StructureDefinition:record-overview` (`valueCode`),
  repeatable. On a form (group item): `tile` shows its latest entry or current values on
  the overview, `quick` puts it in the quick actions. On a field: `header` shows its value
  in the record's header.
* The build-time check rejects unknown codes, as it rejects any unsupported construct.
* Code knows only the marks, never a form or field ID.

### Consequences

* Good, because a state or organisation changes the overview by editing its profile.
* Good, because other FHIR tools ignore the unknown extension and still read the forms.
* Bad, because the ORDO extension is not a published StructureDefinition yet; it is
  documented in the profile README and will be published with the other
  specifications in `spec/`.

## More Information

[EPCR-14](../requirements/04-med.md#epcr-14--record-overview),
[Record overview](../architecture/05-building-block-view.md#record-overview),
[SDC shortText](https://build.fhir.org/ig/HL7/sdc/StructureDefinition-sdc-questionnaire-shortText.html).
