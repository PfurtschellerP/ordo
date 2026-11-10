# ePCR forms: ORDO's subset of HL7 FHIR Questionnaire

Status: draft 0 · Licence: CC BY 4.0 · Decision: [ADR-0037](../../docs/adr/0037-epcr-forms-as-fhir-questionnaire.md)

A regional or organisational profile defines the forms of the patient care record (ePCR)
as one [HL7 FHIR R4 `Questionnaire`](https://hl7.org/fhir/R4/questionnaire.html) in JSON,
e.g. `profiles/de/med/epcr-forms.json`. ORDO reads the subset below. Anything outside it
that would change the meaning of a form is rejected with a message when the profile is
built or loaded; other elements (`id`, `url`, `version`, `title`, `status`, `copyright`,
…) are allowed and ignored.

## Structure

| Element | Rule |
|---|---|
| `resourceType` | `Questionnaire` |
| top-level `item` | `type: group`, one per form. `linkId` is the form ID (`a-z`, `0-9`, `_`). |
| group `repeats` | `true`: a **series** form (timed entries, e.g. vital signs). Absent or `false`: a **snapshot** form (one current value per field, e.g. identity). |
| question `item` | Direct child of a group. `linkId` is `<form>.<field>`, so it is unique in the questionnaire. Nested groups are not supported yet. |
| `text` | Label shown to the user. |
| `type` | `string`, `text`, `decimal`, `integer`, `choice`, `date`, `time`, `reference` (see below). |
| `repeats` on a question | Only for `choice`: several answers may be chosen. |
| `answerOption.valueCoding` | Options of a `choice`; `code` is stored (`a-z`, `0-9`, `_`), `display` is shown. |
| `code` | Terminology code of the question, e.g. LOINC. |
| `definition` | FHIR element the question fills, e.g. `…/Patient#Patient.birthDate`, for later extraction. |
| child item `type: display` | A hint shown below the field (optionally with `questionnaire-itemControl` `prompt`). |

## Extensions

| Extension | Use |
|---|---|
| `http://hl7.org/fhir/StructureDefinition/questionnaire-unit` | Unit of a number (`valueCoding`, UCUM code, `display` shown). |
| `http://hl7.org/fhir/StructureDefinition/questionnaire-referenceResource` | Required on a `reference` question, with `valueCode` `Practitioner`: the question names a person of the crew (who carried out a measure, ordered or gave a medication). ORDO stores the answer as text and suggests the incident's people; any other name is accepted. Other resource types are rejected. |
| `http://hl7.org/fhir/StructureDefinition/minValue`, `…/maxValue` | Input limits of a number. They catch typing errors; they are **not clinical limits**. |
| `http://hl7.org/fhir/uv/sdc/StructureDefinition/sdc-questionnaire-calculatedExpression` | Only on a `readOnly` `decimal` item: a **sum** of questions of the same group, written as `%resource.repeat(item).where(linkId='<form>.<field>').answer.value + …`. Nothing else is accepted: ORDO calculates only sums of documented values and never gives clinical decision support. ORDO evaluates the sum within one entry. |

## Not supported yet

`enableWhen` (conditional questions), `answerValueSet`, `initial`, nested groups,
`required`. They are rejected, so a form never silently behaves differently from what its
author intended.

## Answers

ORDO stores each saved form as an event (`epcr.entryRecorded`, see the Med package) with
the values by field ID. An export as `QuestionnaireResponse` and extraction to FHIR
resources (e.g. `Observation` for vital signs) are planned for the handover (EPCR-6).
