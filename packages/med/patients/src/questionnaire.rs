//! Reads ePCR forms from an HL7 FHIR R4 `Questionnaire` (ADR-0037).
//!
//! ORDO supports a documented subset (`spec/fhir/questionnaire-subset.md`):
//!
//! * Top-level items are `group`s, one per form. A repeating group is a *series* form
//!   (timed entries); a non-repeating group is a *snapshot* form.
//! * Questions are direct children of a group. Their `linkId` is `<group>.<field>`, so it
//!   is unique in the whole questionnaire, as FHIR requires.
//! * Question types: `string`, `text`, `decimal`, `integer`, `choice` (with
//!   `answerOption`; `repeats` for several answers), `date`, `time`.
//! * Extensions: `questionnaire-unit`, `minValue`, `maxValue`, and a `display` child with
//!   `questionnaire-itemControl` for a hint.
//! * Codes: `item.code` (e.g. LOINC) and `item.definition` (FHIR element).
//! * Sums: a read-only `decimal` item with the SDC `calculatedExpression`, restricted to
//!   a sum of questions of the same group. Anything else is rejected: ORDO calculates
//!   nothing but sums of documented values (EPCR-9).
//!
//! Everything outside the subset that changes meaning (e.g. `enableWhen`) is rejected with
//! a message, so form authors know it is not supported yet.

use serde::Deserialize;

use crate::record::{CatalogueError, Choice, Field, FieldType, Form, FormCatalogue, FormKind, Sum};

const UNIT: &str = "http://hl7.org/fhir/StructureDefinition/questionnaire-unit";
const MIN: &str = "http://hl7.org/fhir/StructureDefinition/minValue";
const MAX: &str = "http://hl7.org/fhir/StructureDefinition/maxValue";
const CALCULATED: &str =
    "http://hl7.org/fhir/uv/sdc/StructureDefinition/sdc-questionnaire-calculatedExpression";
const LOINC: &str = "http://loinc.org";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Questionnaire {
    resource_type: String,
    #[serde(default)]
    item: Vec<Item>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Item {
    link_id: String,
    #[serde(default)]
    text: Option<String>,
    #[serde(rename = "type")]
    item_type: String,
    #[serde(default)]
    repeats: bool,
    #[serde(default)]
    read_only: bool,
    #[serde(default)]
    code: Vec<Coding>,
    #[serde(default)]
    definition: Option<String>,
    #[serde(default)]
    answer_option: Vec<AnswerOption>,
    #[serde(default)]
    extension: Vec<Extension>,
    #[serde(default)]
    item: Vec<Item>,
    #[serde(default)]
    enable_when: Vec<serde_json::Value>,
    #[serde(default)]
    answer_value_set: Option<String>,
}

#[derive(Deserialize)]
struct Coding {
    #[serde(default)]
    system: Option<String>,
    code: String,
    #[serde(default)]
    display: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AnswerOption {
    value_coding: Coding,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Extension {
    url: String,
    #[serde(default)]
    value_decimal: Option<f64>,
    #[serde(default)]
    value_integer: Option<i64>,
    #[serde(default)]
    value_coding: Option<Coding>,
    #[serde(default)]
    value_expression: Option<Expression>,
}

#[derive(Deserialize)]
struct Expression {
    language: String,
    expression: String,
}

fn unsupported(link_id: &str, what: &str) -> CatalogueError {
    CatalogueError::Unsupported(format!("{link_id}: {what}"))
}

impl Item {
    fn extension(&self, url: &str) -> Option<&Extension> {
        self.extension.iter().find(|e| e.url == url)
    }

    fn number(&self, url: &str) -> Option<f64> {
        self.extension(url)
            .and_then(|e| e.value_decimal.or(e.value_integer.map(|i| i as f64)))
    }

    fn label(&self) -> String {
        self.text.clone().unwrap_or_else(|| self.link_id.clone())
    }
}

/// The field ID of a question: its `linkId` without the `<group>.` prefix.
fn field_id(group: &str, link_id: &str) -> Result<String, CatalogueError> {
    link_id
        .strip_prefix(group)
        .and_then(|rest| rest.strip_prefix('.'))
        .map(str::to_owned)
        .ok_or_else(|| unsupported(link_id, &format!("linkId must start with \"{group}.\"")))
}

/// Reads a sum: `%resource.repeat(item).where(linkId='g.a').answer.value + …`.
fn sum(group: &str, item: &Item) -> Result<Sum, CatalogueError> {
    let expression = item
        .extension(CALCULATED)
        .and_then(|e| e.value_expression.as_ref())
        .ok_or_else(|| {
            unsupported(
                &item.link_id,
                "read-only items must be sums (calculatedExpression)",
            )
        })?;
    if expression.language != "text/fhirpath" {
        return Err(unsupported(&item.link_id, "expressions must be FHIRPath"));
    }
    let of = expression
        .expression
        .split(" + ")
        .map(|part| {
            part.trim()
                .strip_prefix("%resource.repeat(item).where(linkId='")
                .and_then(|rest| rest.strip_suffix("').answer.value"))
                // A plain linkId, nothing else (no further operators or functions).
                .filter(|id| {
                    id.bytes().all(|b| {
                        b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'.'
                    })
                })
                .ok_or_else(|| {
                    unsupported(
                        &item.link_id,
                        "only sums of answers of the same group are allowed (EPCR-9)",
                    )
                })
                .and_then(|link_id| field_id(group, link_id))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Sum {
        id: field_id(group, &item.link_id)?,
        label: item.label(),
        of,
        code: code(item),
    })
}

/// The code of an item: LOINC as `loinc:<code>`, a FHIR element as `fhir:<path>`.
fn code(item: &Item) -> Option<String> {
    if let Some(coding) = item.code.first() {
        return Some(match coding.system.as_deref() {
            Some(LOINC) => format!("loinc:{}", coding.code),
            Some(system) => format!("{system}|{}", coding.code),
            None => coding.code.clone(),
        });
    }
    item.definition
        .as_deref()
        .and_then(|d| d.split_once('#'))
        .map(|(_, path)| format!("fhir:{path}"))
}

fn field(group: &str, item: &Item) -> Result<Field, CatalogueError> {
    let link_id = &item.link_id;
    if !item.enable_when.is_empty() {
        return Err(unsupported(link_id, "enableWhen is not supported yet"));
    }
    if item.answer_value_set.is_some() {
        return Err(unsupported(
            link_id,
            "answerValueSet is not supported yet; use answerOption",
        ));
    }
    let field_type = match (item.item_type.as_str(), item.repeats) {
        ("string", false) => FieldType::Text,
        ("text", false) => FieldType::Longtext,
        ("decimal" | "integer", false) => FieldType::Number,
        ("choice", false) => FieldType::Choice,
        ("choice", true) => FieldType::Choices,
        ("date", false) => FieldType::Date,
        ("time", false) => FieldType::Time,
        (other, true) => {
            return Err(unsupported(
                link_id,
                &format!("repeating {other} questions are not supported"),
            ));
        }
        (other, false) => {
            return Err(unsupported(
                link_id,
                &format!("question type {other} is not supported"),
            ));
        }
    };
    let mut hint = None;
    for child in &item.item {
        match child.item_type.as_str() {
            "display" => hint = child.text.clone(),
            _ => {
                return Err(unsupported(
                    &child.link_id,
                    "questions cannot be nested in questions",
                ));
            }
        }
    }
    Ok(Field {
        id: field_id(group, link_id)?,
        label: item.label(),
        field_type,
        unit: item
            .extension(UNIT)
            .and_then(|e| e.value_coding.as_ref())
            .map(|c| c.display.clone().unwrap_or_else(|| c.code.clone())),
        min: item.number(MIN),
        max: item.number(MAX),
        options: item
            .answer_option
            .iter()
            .map(|o| Choice {
                id: o.value_coding.code.clone(),
                label: o
                    .value_coding
                    .display
                    .clone()
                    .unwrap_or_else(|| o.value_coding.code.clone()),
            })
            .collect(),
        code: code(item),
        hint,
    })
}

impl FormCatalogue {
    /// Reads the forms from a FHIR R4 `Questionnaire` in JSON and checks them.
    pub fn from_questionnaire_json(json: &str) -> Result<Self, CatalogueError> {
        let questionnaire: Questionnaire = serde_json::from_str(json).map_err(|e| {
            CatalogueError::Unsupported(format!("not a readable Questionnaire: {e}"))
        })?;
        if questionnaire.resource_type != "Questionnaire" {
            return Err(CatalogueError::Unsupported(format!(
                "resourceType is {}, expected Questionnaire",
                questionnaire.resource_type
            )));
        }
        let mut forms = Vec::new();
        for group in &questionnaire.item {
            if group.item_type != "group" {
                return Err(unsupported(
                    &group.link_id,
                    "top-level items must be groups (one per form)",
                ));
            }
            let mut form = Form {
                id: group.link_id.clone(),
                label: group.label(),
                kind: if group.repeats {
                    FormKind::Series
                } else {
                    FormKind::Snapshot
                },
                fields: Vec::new(),
                sums: Vec::new(),
            };
            for item in &group.item {
                match (item.item_type.as_str(), item.read_only) {
                    ("group", _) => {
                        return Err(unsupported(
                            &item.link_id,
                            "nested groups are not supported yet",
                        ));
                    }
                    ("display", _) => {}
                    (_, true) => form.sums.push(sum(&group.link_id, item)?),
                    (_, false) => form.fields.push(field(&group.link_id, item)?),
                }
            }
            forms.push(form);
        }
        let catalogue = Self { format: 1, forms };
        catalogue.check()?;
        Ok(catalogue)
    }
}
