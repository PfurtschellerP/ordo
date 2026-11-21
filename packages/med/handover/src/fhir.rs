//! The machine-readable part of the handover document (ADR-0089): an HL7 FHIR R4 Bundle
//! of type `document`. A Composition names the patient, the handover and the record's
//! parts; the Patient carries the triage tag number and what the identity form documents;
//! every entry of the record is a QuestionnaireResponse to the profile's Questionnaire
//! (ADR-0037), with the profile's field IDs as `linkId`s.

use std::hash::{DefaultHasher, Hash, Hasher};

use ordo_med_patients::record::{Field, FieldType, Form, FormCatalogue, FormKind, Record, Value};
use serde_json::{Value as Json, json};

use crate::data::{Data, Naming};

/// The profile's Questionnaire: its canonical URL and version.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Questionnaire {
    /// The canonical URL.
    pub url: String,
    /// The version.
    pub version: String,
}

impl Questionnaire {
    /// Reads the canonical URL and version of a Questionnaire in JSON.
    pub fn from_json(text: &str) -> Option<Self> {
        let v: Json = serde_json::from_str(text).ok()?;
        Some(Self {
            url: v.get("url")?.as_str()?.to_owned(),
            version: v.get("version")?.as_str()?.to_owned(),
        })
    }

    fn canonical(&self) -> String {
        format!("{}|{}", self.url, self.version)
    }
}

/// A UUID URN for the bundle's entries, the same for the same seed (a version 4 layout).
fn urn(seed: &str) -> String {
    let half = |salt: u8| {
        let mut h = DefaultHasher::new();
        (seed, salt).hash(&mut h);
        h.finish()
    };
    let (a, b) = (half(1), half(2));
    format!(
        "urn:uuid:{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        a >> 32,
        (a >> 16) & 0xffff,
        a & 0x0fff,
        0x8000 | ((b >> 48) & 0x3fff),
        b & 0xffff_ffff_ffff
    )
}

/// An answer of a QuestionnaireResponse item.
fn answers(field: &Field, value: &Value, naming: &Naming) -> Vec<Json> {
    let coding = |id: &str| {
        let display = field
            .options
            .iter()
            .find(|o| o.id == id)
            .map(|o| o.label.clone());
        match display {
            Some(d) => json!({ "valueCoding": { "code": id, "display": d } }),
            // An open choice's own answer (EPCR-12).
            None => json!({ "valueString": id }),
        }
    };
    match value {
        Value::Many(list) => list.iter().map(|v| coding(v)).collect(),
        Value::One(text) => vec![match field.field_type {
            FieldType::Choice | FieldType::Choices => coding(text),
            FieldType::Number => match text.parse::<f64>() {
                Ok(n) => json!({ "valueDecimal": n }),
                Err(_) => json!({ "valueString": text }),
            },
            FieldType::Date => json!({ "valueDate": text }),
            FieldType::Time => json!({ "valueTime": format!("{text}:00") }),
            FieldType::Person => json!({
                "valueReference": { "display": text
                    .parse::<ordo_core::UserId>()
                    .map_or_else(|_| text.clone(), |u| (naming.person)(&u)) }
            }),
            FieldType::Text | FieldType::Longtext => json!({ "valueString": text }),
        }],
    }
}

/// The items of one form's values.
fn items(form: &Form, values: impl Fn(&str) -> Option<Value>, naming: &Naming) -> Vec<Json> {
    form.fields
        .iter()
        .filter_map(|f| {
            let v = values(&f.id).filter(|v| !v.is_empty())?;
            Some(json!({
                "linkId": format!("{}.{}", form.id, f.id),
                "text": f.label,
                "answer": answers(f, &v, naming),
            }))
        })
        .collect()
}

fn response(
    q: &Questionnaire,
    patient: &str,
    form: &Form,
    items: Vec<Json>,
    authored: &str,
    author: &str,
) -> Json {
    json!({
        "resourceType": "QuestionnaireResponse",
        "questionnaire": q.canonical(),
        "status": "completed",
        "subject": { "reference": patient },
        "authored": authored,
        "author": { "display": author },
        "item": [{ "linkId": form.id, "text": form.label, "item": items }],
    })
}

/// The Patient: the triage tag number as identifier, and name, birth date and sex where
/// the identity form documents them with their FHIR definition.
fn patient(data: &Data, record: Option<&Record>, catalogue: &FormCatalogue) -> Json {
    let mut p = json!({
        "resourceType": "Patient",
        "identifier": [{ "system": "urn:ordo:triage-tag", "value": data.tag }],
    });
    let Some(record) = record else { return p };
    let by_code = |code: &str| {
        catalogue.forms.iter().find_map(|form| {
            let field = form
                .fields
                .iter()
                .find(|f| f.code.as_deref() == Some(code))?;
            match &record.current(&form.id, &field.id)?.value {
                Value::One(text) => Some(text.clone()),
                Value::Many(_) => None,
            }
        })
    };
    let family = by_code("fhir:Patient.name.family");
    let given = by_code("fhir:Patient.name.given");
    if family.is_some() || given.is_some() {
        let mut name = json!({});
        if let Some(f) = family {
            name["family"] = json!(f);
        }
        if let Some(g) = given {
            name["given"] = json!([g]);
        }
        p["name"] = json!([name]);
    }
    if let Some(birth) = by_code("fhir:Patient.birthDate") {
        p["birthDate"] = json!(birth);
    }
    if let Some(sex) = by_code("fhir:Patient.gender")
        .filter(|s| ["male", "female", "other", "unknown"].contains(&s.as_str()))
    {
        p["gender"] = json!(sex);
    }
    p
}

/// XHTML narrative for a Composition section, escaped.
fn narrative(lines: &[String]) -> Json {
    let escape = |t: &str| {
        t.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    };
    let body: String = lines
        .iter()
        .map(|l| format!("<p>{}</p>", escape(l)))
        .collect();
    json!({
        "status": "generated",
        "div": format!("<div xmlns=\"http://www.w3.org/1999/xhtml\">{body}</div>"),
    })
}

// [impl->dsn~handover-document~1]
/// The bundle for `data`, with the record's entries from `record`.
pub fn bundle(
    data: &Data,
    record: Option<&Record>,
    catalogue: &FormCatalogue,
    questionnaire: &Questionnaire,
    naming: &Naming,
) -> Json {
    let seed = format!("{}|{}", data.tag, data.created.iso);
    let patient_urn = urn(&format!("{seed}|patient"));
    let mut entries = vec![json!({
        "fullUrl": patient_urn,
        "resource": patient(data, record, catalogue),
    })];
    let mut sections = Vec::new();
    if let Some(record) = record {
        for form in &catalogue.forms {
            let mut refs = Vec::new();
            let mut add = |items: Vec<Json>, at: &str, by: &str, n: usize| {
                if items.is_empty() {
                    return;
                }
                let id = urn(&format!("{seed}|{}|{n}", form.id));
                refs.push(json!({ "reference": id }));
                entries.push(json!({
                    "fullUrl": id,
                    "resource": response(questionnaire, &patient_urn, form, items, at, by),
                }));
            };
            match form.kind {
                FormKind::Snapshot => {
                    let latest = form
                        .fields
                        .iter()
                        .filter_map(|f| record.current(&form.id, &f.id))
                        .max_by_key(|s| s.at);
                    if let Some(s) = latest {
                        let items = items(
                            form,
                            |f| record.current(&form.id, f).map(|s| s.value.clone()),
                            naming,
                        );
                        add(
                            items,
                            &naming.moment(s.at.wall_ms).iso,
                            &(naming.person)(&s.by),
                            0,
                        );
                    }
                }
                FormKind::Series => {
                    for (n, e) in record.current_entries(&form.id).enumerate() {
                        let items = items(form, |f| e.values.get(f).cloned(), naming);
                        add(
                            items,
                            &naming.moment(e.at.wall_ms).iso,
                            &(naming.person)(&e.by),
                            n,
                        );
                    }
                }
            }
            if !refs.is_empty() {
                sections.push(json!({ "title": form.label, "entry": refs }));
            }
        }
    }
    let triage: Vec<String> = data
        .triage
        .iter()
        .map(|t| format!("{} {}: {} ({})", t.at.date, t.at.time, t.category, t.by))
        .collect();
    sections.insert(
        0,
        json!({ "title": "Triage", "code": { "text": "triage" }, "text": narrative(&triage) }),
    );
    if let Some(h) = &data.handover {
        let mut lines = vec![format!(
            "{} {}: {} → {} ({})",
            h.at.date, h.at.time, h.from, h.to, h.by
        )];
        lines.extend(h.note.clone());
        sections.insert(0, json!({ "title": "Handover", "code": { "text": "handover" }, "text": narrative(&lines) }));
    }
    let author = data
        .handover
        .as_ref()
        .map_or_else(|| "ORDO".to_owned(), |h| h.by.clone());
    let composition = json!({
        "resourceType": "Composition",
        "status": "final",
        "type": { "text": "Patient handover" },
        "subject": { "reference": patient_urn },
        "date": data.created.iso,
        "author": [{ "display": author }],
        "title": format!("Patient handover {}", data.tag),
        "section": sections,
    });
    entries.insert(
        0,
        json!({ "fullUrl": urn(&format!("{seed}|composition")), "resource": composition }),
    );
    json!({
        "resourceType": "Bundle",
        "identifier": { "system": "urn:ordo:handover-document", "value": urn(&seed) },
        "type": "document",
        "timestamp": data.created.iso,
        "entry": entries,
    })
}
