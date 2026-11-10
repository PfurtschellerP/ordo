//! Module `med.epcr`: the patient care record (ePCR).
//!
//! The structure of the record — forms, fields, units, choices — comes from the profile
//! as an HL7 FHIR `Questionnaire` (catalogue `epcr-forms`, EPCR-2, ADR-0037; read by
//! [`crate::questionnaire`]); this module only knows field *types*. Every entry
//! is an event in the protected compartment `med.clinical`, linked to the patient by the
//! triage tag (EPCR-1). Nothing is overwritten (EPCR-4):
//!
//! * In a **snapshot** form (e.g. identity), the latest value of each field counts; the
//!   earlier values stay in the history.
//! * In a **series** form (e.g. vital signs), every entry stays; a correction is a new
//!   entry that names the entry it replaces.
//!
//! The module documents and organises; it never interprets values (EPCR-9). The only
//! calculation is the sum of documented values that the profile asks for (e.g. GCS).

use std::collections::{BTreeMap, BTreeSet};

use ordo_core::codec::{self, CodecError};
use ordo_core::{AttachmentRef, Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "med.epcr";
/// Event type: values of one form were recorded for a patient.
pub const ENTRY_RECORDED: &str = "epcr.entryRecorded";
/// Event type: a photo of a patient was added inside the event (ADR-0028, superseded;
/// still read, no longer written).
pub const PHOTO_ADDED: &str = "epcr.photoAdded";
/// Event type: a photo of a patient was attached (ADR-0033): a small preview inside the
/// event, the full image as an encrypted attachment.
pub const PHOTO_ATTACHED: &str = "epcr.photoAttached";
/// Largest photo inside an `epcr.photoAdded` event, in bytes.
pub const MAX_PHOTO_BYTES: usize = 160 * 1024;
/// Largest preview inside an `epcr.photoAttached` event, in bytes.
pub const MAX_PREVIEW_BYTES: usize = 16 * 1024;
/// Largest full photo (plaintext) of an `epcr.photoAttached` event, in bytes.
pub const MAX_FULL_PHOTO_BYTES: usize = 1024 * 1024;
/// Bytes the attachment encryption adds (nonce and tag, see `ordo_crypto::attachment`).
const ATTACHMENT_OVERHEAD: u64 = 40;
/// Media types accepted for photos.
pub const PHOTO_MEDIA_TYPES: [&str; 2] = ["image/jpeg", "image/webp"];
/// Longest text value, in characters.
pub const MAX_TEXT_CHARS: usize = 4000;

// ---------------------------------------------------------------------------
// Profile catalogue `epcr-forms`
// ---------------------------------------------------------------------------

/// The ePCR forms of a profile, read from its FHIR Questionnaire
/// (`profiles/<id>/med/epcr-forms.json`) by [`FormCatalogue::from_questionnaire_json`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FormCatalogue {
    /// Catalogue format version; this module reads format 1.
    pub format: u32,
    /// Forms in display order.
    #[serde(rename = "form")]
    pub forms: Vec<Form>,
}

/// How entries of a form combine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FormKind {
    /// One current value per field (identity, history, disposition).
    Snapshot,
    /// A list of timed entries (vital signs, measures, medication).
    Series,
}

/// One form of the record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Form {
    /// Stable ID, e.g. `vitals`.
    pub id: String,
    /// Heading shown to the user.
    pub label: String,
    /// Snapshot or series.
    pub kind: FormKind,
    /// Fields in display order.
    #[serde(rename = "field")]
    pub fields: Vec<Field>,
    /// Sums of documented values shown with each entry (e.g. GCS total).
    #[serde(default, rename = "sum", skip_serializing_if = "Vec::is_empty")]
    pub sums: Vec<Sum>,
}

/// Input type of a field.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    /// One line of text.
    Text,
    /// Several lines of text.
    Longtext,
    /// A decimal number, e.g. `37.5`.
    Number,
    /// One of `options`.
    Choice,
    /// Any number of `options`.
    Choices,
    /// A date, `YYYY-MM-DD`.
    Date,
    /// A time of day, `HH:MM`.
    Time,
    /// A person (a member of the crew), as text: the app suggests the incident's people,
    /// any other name is accepted (FHIR: `reference` to `Practitioner`).
    Person,
}

/// One option of a choice field.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    /// Stored value, stable.
    pub id: String,
    /// Label shown to the user.
    pub label: String,
}

/// One field of a form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// Stable ID within the form, e.g. `heart_rate`.
    pub id: String,
    /// Label shown to the user.
    pub label: String,
    /// Input type.
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Unit shown next to a number, e.g. `/min`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    /// Smallest value that can be entered (input check, not a clinical limit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    /// Largest value that can be entered (input check, not a clinical limit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
    /// Options of a choice field.
    #[serde(default, rename = "option", skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<Choice>,
    /// Code in an open terminology, for later exchange (e.g. `loinc:8867-4`,
    /// `fhir:Patient.birthDate`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Short help text below the field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

/// A sum of documented number fields, e.g. the GCS total (allowed by EPCR-9).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sum {
    /// Stable ID.
    pub id: String,
    /// Label shown with the result.
    pub label: String,
    /// The fields that are added up; the sum is shown only when all are documented.
    pub of: Vec<String>,
    /// Code in an open terminology.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// A mistake in a form catalogue.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum CatalogueError {
    /// Unsupported format version.
    #[error("format {0} is not supported (expected 1)")]
    Format(u32),
    /// Two forms, fields or options share an ID.
    #[error("duplicate ID {0}")]
    Duplicate(String),
    /// A choice field without options, or options on another field type.
    #[error("field {0}: options only and always for choice fields")]
    Options(String),
    /// `min` is larger than `max`, or limits on a non-number field.
    #[error("field {0}: invalid limits")]
    Limits(String),
    /// A sum names a field that is not a number field of the form.
    #[error("sum {0}: every part must be a number field of the form")]
    Sum(String),
    /// An ID that cannot be stored safely.
    #[error("invalid ID {0:?}: use a-z, 0-9 and _")]
    Id(String),
    /// Something outside ORDO's subset of FHIR Questionnaire.
    #[error("not supported by ORDO's Questionnaire subset: {0}")]
    Unsupported(String),
}

fn check_id(id: &str) -> Result<(), CatalogueError> {
    if !id.is_empty()
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        Ok(())
    } else {
        Err(CatalogueError::Id(id.to_owned()))
    }
}

impl FormCatalogue {
    /// Checks the catalogue: IDs, options, limits and sums.
    pub fn check(&self) -> Result<(), CatalogueError> {
        if self.format != 1 {
            return Err(CatalogueError::Format(self.format));
        }
        let mut forms = BTreeSet::new();
        for form in &self.forms {
            check_id(&form.id)?;
            if !forms.insert(&form.id) {
                return Err(CatalogueError::Duplicate(form.id.clone()));
            }
            let mut fields = BTreeSet::new();
            for field in &form.fields {
                let name = format!("{}.{}", form.id, field.id);
                check_id(&field.id)?;
                if !fields.insert(&field.id) {
                    return Err(CatalogueError::Duplicate(name));
                }
                let is_choice = matches!(field.field_type, FieldType::Choice | FieldType::Choices);
                if is_choice == field.options.is_empty() {
                    return Err(CatalogueError::Options(name));
                }
                let mut options = BTreeSet::new();
                for option in &field.options {
                    check_id(&option.id)?;
                    if !options.insert(&option.id) {
                        return Err(CatalogueError::Duplicate(format!("{name}.{}", option.id)));
                    }
                }
                let has_limits = field.min.is_some() || field.max.is_some();
                let limits_ok = match (field.min, field.max) {
                    (Some(min), Some(max)) => min <= max,
                    _ => true,
                };
                if (has_limits && field.field_type != FieldType::Number) || !limits_ok {
                    return Err(CatalogueError::Limits(name));
                }
            }
            for sum in &form.sums {
                check_id(&sum.id)?;
                let parts_ok = !sum.of.is_empty()
                    && sum.of.iter().all(|part| {
                        form.fields
                            .iter()
                            .any(|f| &f.id == part && f.field_type == FieldType::Number)
                    });
                if !parts_ok {
                    return Err(CatalogueError::Sum(format!("{}.{}", form.id, sum.id)));
                }
            }
        }
        Ok(())
    }

    /// The form with `id`.
    pub fn form(&self, id: &str) -> Option<&Form> {
        self.forms.iter().find(|f| f.id == id)
    }
}

// ---------------------------------------------------------------------------
// Values and their checks
// ---------------------------------------------------------------------------

/// A recorded value: text for most field types, a list for multiple choice. Numbers,
/// dates and times are stored as their canonical text (`37.5`, `2026-11-08`, `14:05`),
/// so nothing is lost by rounding.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// One value. An empty text clears a snapshot field.
    One(String),
    /// Several options.
    Many(Vec<String>),
}

impl Value {
    /// Whether the value is empty (cleared).
    pub fn is_empty(&self) -> bool {
        match self {
            Value::One(text) => text.trim().is_empty(),
            Value::Many(list) => list.is_empty(),
        }
    }
}

/// Why a value cannot be recorded.
#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ValueError {
    /// The form has no such field.
    #[error("unknown field")]
    UnknownField,
    /// Not a decimal number.
    #[error("not a number")]
    NotANumber,
    /// Outside the input limits of the field.
    #[error("outside {min:?}…{max:?}")]
    OutOfRange {
        /// Lower limit.
        min: Option<f64>,
        /// Upper limit.
        max: Option<f64>,
    },
    /// Not one of the options.
    #[error("unknown option {0}")]
    UnknownOption(String),
    /// Not a date `YYYY-MM-DD`.
    #[error("not a date")]
    NotADate,
    /// Not a time `HH:MM`.
    #[error("not a time")]
    NotATime,
    /// Too long.
    #[error("too long")]
    TooLong,
    /// A list for a single-value field or the other way round.
    #[error("wrong kind of value")]
    WrongKind,
}

pub(crate) fn parse_number(text: &str) -> Option<f64> {
    // Accept a decimal comma, as typed on German keyboards.
    let text = text.trim().replace(',', ".");
    let valid = !text.is_empty()
        && text.chars().filter(|c| *c == '.').count() <= 1
        && text
            .strip_prefix('-')
            .unwrap_or(&text)
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
        && text.chars().any(|c| c.is_ascii_digit());
    if valid { text.parse().ok() } else { None }
}

fn digits(text: &str, len: usize) -> Option<u32> {
    (text.len() == len && text.bytes().all(|b| b.is_ascii_digit()))
        .then(|| text.parse().ok())
        .flatten()
}

fn is_date(text: &str) -> bool {
    let mut parts = text.split('-');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(y), Some(m), Some(d), None) => {
            digits(y, 4).is_some()
                && digits(m, 2).is_some_and(|m| (1..=12).contains(&m))
                && digits(d, 2).is_some_and(|d| (1..=31).contains(&d))
        }
        _ => false,
    }
}

fn is_time(text: &str) -> bool {
    match text.split_once(':') {
        Some((h, m)) => {
            digits(h, 2).is_some_and(|h| h < 24) && digits(m, 2).is_some_and(|m| m < 60)
        }
        None => false,
    }
}

impl Field {
    /// Checks one value and returns it in canonical form (trimmed; numbers with a
    /// decimal point). Empty values are valid: they clear a field.
    pub fn normalise(&self, value: &Value) -> Result<Value, ValueError> {
        if value.is_empty() {
            return Ok(match self.field_type {
                FieldType::Choices => Value::Many(Vec::new()),
                _ => Value::One(String::new()),
            });
        }
        match (self.field_type, value) {
            (FieldType::Choices, Value::Many(list)) => {
                for id in list {
                    if !self.options.iter().any(|o| &o.id == id) {
                        return Err(ValueError::UnknownOption(id.clone()));
                    }
                }
                // Keep the profile's order and drop duplicates.
                Ok(Value::Many(
                    self.options
                        .iter()
                        .filter(|o| list.contains(&o.id))
                        .map(|o| o.id.clone())
                        .collect(),
                ))
            }
            (FieldType::Choices, Value::One(_)) | (_, Value::Many(_)) => Err(ValueError::WrongKind),
            (field_type, Value::One(text)) => {
                let text = text.trim();
                if text.chars().count() > MAX_TEXT_CHARS {
                    return Err(ValueError::TooLong);
                }
                match field_type {
                    FieldType::Text | FieldType::Longtext | FieldType::Person => {
                        Ok(Value::One(text.to_owned()))
                    }
                    FieldType::Number => {
                        let number = parse_number(text).ok_or(ValueError::NotANumber)?;
                        let below = self.min.is_some_and(|min| number < min);
                        let above = self.max.is_some_and(|max| number > max);
                        if below || above {
                            return Err(ValueError::OutOfRange {
                                min: self.min,
                                max: self.max,
                            });
                        }
                        Ok(Value::One(text.replace(',', ".")))
                    }
                    FieldType::Choice => {
                        if self.options.iter().any(|o| o.id == text) {
                            Ok(Value::One(text.to_owned()))
                        } else {
                            Err(ValueError::UnknownOption(text.to_owned()))
                        }
                    }
                    FieldType::Date if is_date(text) => Ok(Value::One(text.to_owned())),
                    FieldType::Date => Err(ValueError::NotADate),
                    FieldType::Time if is_time(text) => Ok(Value::One(text.to_owned())),
                    FieldType::Time => Err(ValueError::NotATime),
                    FieldType::Choices => unreachable!("handled above"),
                }
            }
        }
    }

    /// The label of an option.
    pub fn option_label(&self, id: &str) -> Option<&str> {
        self.options
            .iter()
            .find(|o| o.id == id)
            .map(|o| o.label.as_str())
    }

    /// The value as the user reads it: option labels, number with unit.
    pub fn display(&self, value: &Value) -> String {
        match value {
            Value::Many(list) => list
                .iter()
                .map(|id| self.option_label(id).unwrap_or(id))
                .collect::<Vec<_>>()
                .join(", "),
            Value::One(text) => match self.field_type {
                FieldType::Choice => self.option_label(text).unwrap_or(text).to_owned(),
                FieldType::Number => match &self.unit {
                    Some(unit) => format!("{text} {unit}"),
                    None => text.clone(),
                },
                _ => text.clone(),
            },
        }
    }
}

impl Form {
    /// The field with `id`.
    pub fn field(&self, id: &str) -> Option<&Field> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Checks all values. Returns the canonical values, or the problems per field.
    pub fn normalise(
        &self,
        values: &BTreeMap<String, Value>,
    ) -> Result<BTreeMap<String, Value>, Vec<(String, ValueError)>> {
        let mut clean = BTreeMap::new();
        let mut problems = Vec::new();
        for (id, value) in values {
            match self.field(id).map(|f| f.normalise(value)) {
                Some(Ok(v)) => {
                    clean.insert(id.clone(), v);
                }
                Some(Err(e)) => problems.push((id.clone(), e)),
                None => problems.push((id.clone(), ValueError::UnknownField)),
            }
        }
        if problems.is_empty() {
            Ok(clean)
        } else {
            Err(problems)
        }
    }

    /// The sums of an entry that can be computed: all parts must be documented.
    /// Only adds documented values (EPCR-9).
    pub fn sums(&self, values: &BTreeMap<String, Value>) -> Vec<(&Sum, String)> {
        self.sums
            .iter()
            .filter_map(|sum| {
                let mut total = 0.0;
                for part in &sum.of {
                    match values.get(part) {
                        Some(Value::One(text)) => total += parse_number(text)?,
                        _ => return None,
                    }
                }
                Some((sum, format_number(total)))
            })
            .collect()
    }
}

fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{n:.0}")
    } else {
        let text = format!("{n:.3}");
        text.trim_end_matches('0').trim_end_matches('.').to_owned()
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Payload of `epcr.entryRecorded`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntryRecorded {
    /// Triage tag number of the patient.
    pub tag: String,
    /// Form ID from the profile.
    pub form: String,
    /// Values by field ID, in canonical form.
    pub values: BTreeMap<String, Value>,
    /// Series forms: ID of the entry this one corrects.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replaces: Option<String>,
}

impl EntryRecorded {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `epcr.photoAdded`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoAdded {
    /// Triage tag number of the patient.
    pub tag: String,
    /// `image/jpeg` or `image/webp`.
    pub media_type: String,
    /// Image bytes, re-encoded on the device (no camera metadata).
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

/// Why a photo cannot be recorded.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum PhotoError {
    /// Larger than [`MAX_PHOTO_BYTES`].
    #[error("photo has {0} bytes, more than {MAX_PHOTO_BYTES}")]
    TooLarge(usize),
    /// Not an accepted media type.
    #[error("media type {0} is not accepted")]
    MediaType(String),
}

impl PhotoAdded {
    /// Checks size and media type.
    pub fn check(&self) -> Result<(), PhotoError> {
        if !PHOTO_MEDIA_TYPES.contains(&self.media_type.as_str()) {
            return Err(PhotoError::MediaType(self.media_type.clone()));
        }
        if self.data.len() > MAX_PHOTO_BYTES {
            return Err(PhotoError::TooLarge(self.data.len()));
        }
        Ok(())
    }

    /// CBOR payload; checks size and media type first.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.check().map_err(|e| e.to_string())?;
        codec::to_cbor(self).map_err(|e| e.to_string())
    }
}

/// Payload of `epcr.photoAttached`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoAttached {
    /// Triage tag number of the patient.
    pub tag: String,
    /// Media type of the preview.
    pub preview_type: String,
    /// A small preview, re-encoded on the device (no camera metadata).
    #[serde(with = "serde_bytes")]
    pub preview: Vec<u8>,
    /// The full photo, an encrypted attachment.
    pub photo: AttachmentRef,
}

impl PhotoAttached {
    /// Checks sizes and media types.
    pub fn check(&self) -> Result<(), PhotoError> {
        for media_type in [&self.preview_type, &self.photo.media_type] {
            if !PHOTO_MEDIA_TYPES.contains(&media_type.as_str()) {
                return Err(PhotoError::MediaType(media_type.clone()));
            }
        }
        if self.preview.len() > MAX_PREVIEW_BYTES {
            return Err(PhotoError::TooLarge(self.preview.len()));
        }
        if self.photo.size > MAX_FULL_PHOTO_BYTES as u64 + ATTACHMENT_OVERHEAD {
            return Err(PhotoError::TooLarge(
                usize::try_from(self.photo.size).unwrap_or(usize::MAX),
            ));
        }
        Ok(())
    }

    /// CBOR payload; checks sizes and media types first.
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        self.check().map_err(|e| e.to_string())?;
        codec::to_cbor(self).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// The record, computed from events
// ---------------------------------------------------------------------------

/// A value with its origin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stamped<T> {
    /// The value.
    pub value: T,
    /// When it was recorded.
    pub at: Hlc,
    /// Who recorded it.
    pub by: UserId,
}

/// One entry of a series form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    /// Event ID; corrections refer to it.
    pub id: String,
    /// Values.
    pub values: BTreeMap<String, Value>,
    /// When and by whom.
    pub at: Hlc,
    /// Author.
    pub by: UserId,
    /// The entry this one corrects.
    pub replaces: Option<String>,
    /// The entry that corrects this one, if any.
    pub replaced_by: Option<String>,
}

/// A photo of the patient.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Photo {
    /// Event ID.
    pub id: String,
    /// Media type of `data`.
    pub media_type: String,
    /// The photo inside the event: the preview, or the whole photo of older events.
    pub data: Vec<u8>,
    /// The full photo as an attachment, if any.
    pub full: Option<AttachmentRef>,
    /// When and by whom.
    pub at: Hlc,
}

/// The care record of one patient.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Record {
    /// Snapshot forms: current value per field, with every earlier value in `history`.
    pub fields: BTreeMap<(String, String), Vec<Stamped<Value>>>,
    /// Series forms: entries, oldest first (corrected entries included, marked).
    pub entries: BTreeMap<String, Vec<Entry>>,
    /// Photos, oldest first.
    pub photos: Vec<Photo>,
}

impl Record {
    /// The current value of a snapshot field, if documented and not cleared.
    pub fn current(&self, form: &str, field: &str) -> Option<&Stamped<Value>> {
        self.fields
            .get(&(form.to_owned(), field.to_owned()))
            .and_then(|history| history.last())
            .filter(|s| !s.value.is_empty())
    }

    /// The entries of a series form that are not corrected by a later entry.
    pub fn current_entries(&self, form: &str) -> impl Iterator<Item = &Entry> {
        self.entries
            .get(form)
            .into_iter()
            .flatten()
            .filter(|e| e.replaced_by.is_none())
    }

    /// The latest photo.
    pub fn photo(&self) -> Option<&Photo> {
        self.photos.last()
    }
}

/// All care records of an incident, by triage tag.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Records {
    records: BTreeMap<String, Record>,
}

enum Change {
    Entry(EntryRecorded),
    Photo(PhotoAdded),
    Attached(PhotoAttached),
}

impl Records {
    // [impl->dsn~patient-care-record~1]
    /// Computes the records. The result doesn't depend on the order of `events`.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(&AppliedEvent, Change)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let change = match e.body.event_type.as_str() {
                    ENTRY_RECORDED => Change::Entry(codec::from_cbor(&e.payload).ok()?),
                    PHOTO_ADDED => Change::Photo(codec::from_cbor(&e.payload).ok()?),
                    PHOTO_ATTACHED => Change::Attached(codec::from_cbor(&e.payload).ok()?),
                    _ => return None,
                };
                Some((e, change))
            })
            .collect();
        changes.sort_by_key(|(e, _)| e.body.hlc);
        let mut records = Self::default();
        for (event, change) in changes {
            let (at, by, id) = (
                event.body.hlc,
                event.body.author.user.clone(),
                event.body.id.to_string(),
            );
            match change {
                Change::Photo(photo) => {
                    if photo.check().is_ok() {
                        records
                            .records
                            .entry(photo.tag)
                            .or_default()
                            .photos
                            .push(Photo {
                                id,
                                media_type: photo.media_type,
                                data: photo.data,
                                full: None,
                                at,
                            });
                    }
                }
                Change::Attached(photo) => {
                    if photo.check().is_ok() {
                        records
                            .records
                            .entry(photo.tag)
                            .or_default()
                            .photos
                            .push(Photo {
                                id,
                                media_type: photo.preview_type,
                                data: photo.preview,
                                full: Some(photo.photo),
                                at,
                            });
                    }
                }
                Change::Entry(entry) => {
                    let record = records.records.entry(entry.tag).or_default();
                    // The record keeps both views of every entry (field history and
                    // entry list); the form kind in the profile decides which is shown.
                    for (field, value) in &entry.values {
                        record
                            .fields
                            .entry((entry.form.clone(), field.clone()))
                            .or_default()
                            .push(Stamped {
                                value: value.clone(),
                                at,
                                by: by.clone(),
                            });
                    }
                    let list = record.entries.entry(entry.form).or_default();
                    if let Some(old) = &entry.replaces
                        && let Some(replaced) = list.iter_mut().find(|e| &e.id == old)
                        && replaced.replaced_by.is_none()
                    {
                        replaced.replaced_by = Some(id.clone());
                    }
                    list.push(Entry {
                        id,
                        values: entry.values,
                        at,
                        by,
                        replaces: entry.replaces,
                        replaced_by: None,
                    });
                }
            }
        }
        records
    }

    /// The record of the patient with triage tag `tag`.
    pub fn get(&self, tag: &str) -> Option<&Record> {
        self.records.get(tag)
    }

    /// Whether a patient has a photo.
    pub fn has_photo(&self, tag: &str) -> bool {
        self.get(tag).is_some_and(|r| !r.photos.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::{EventId, Ulid};

    /// A small questionnaire in ORDO's FHIR subset.
    fn catalogue() -> FormCatalogue {
        FormCatalogue::from_questionnaire_json(QUESTIONNAIRE).expect("valid questionnaire")
    }

    const QUESTIONNAIRE: &str = r#"{
      "resourceType": "Questionnaire",
      "status": "draft",
      "item": [
        { "linkId": "identity", "text": "Personalien", "type": "group", "item": [
          { "linkId": "identity.family_name", "text": "Name", "type": "string",
            "definition": "http://hl7.org/fhir/StructureDefinition/Patient#Patient.name.family" },
          { "linkId": "identity.sex", "text": "Geschlecht", "type": "choice", "answerOption": [
            { "valueCoding": { "code": "female", "display": "weiblich" } },
            { "valueCoding": { "code": "male", "display": "männlich" } } ] },
          { "linkId": "identity.recorded_by", "text": "Aufgenommen von", "type": "reference",
            "extension": [{ "url": "http://hl7.org/fhir/StructureDefinition/questionnaire-referenceResource",
              "valueCode": "Practitioner" }] } ] },
        { "linkId": "vitals", "text": "Vitalwerte", "type": "group", "repeats": true, "item": [
          { "linkId": "vitals.heart_rate", "text": "Herzfrequenz", "type": "decimal",
            "code": [{ "system": "http://loinc.org", "code": "8867-4" }],
            "extension": [
              { "url": "http://hl7.org/fhir/StructureDefinition/questionnaire-unit",
                "valueCoding": { "system": "http://unitsofmeasure.org", "code": "/min", "display": "/min" } },
              { "url": "http://hl7.org/fhir/StructureDefinition/minValue", "valueDecimal": 0 },
              { "url": "http://hl7.org/fhir/StructureDefinition/maxValue", "valueDecimal": 300 } ],
            "item": [{ "linkId": "vitals.heart_rate.hint", "text": "Schläge pro Minute", "type": "display" }] },
          { "linkId": "vitals.gcs_eye", "text": "GCS Augen", "type": "integer",
            "extension": [
              { "url": "http://hl7.org/fhir/StructureDefinition/minValue", "valueInteger": 1 },
              { "url": "http://hl7.org/fhir/StructureDefinition/maxValue", "valueInteger": 4 } ] },
          { "linkId": "vitals.gcs_verbal", "text": "GCS verbal", "type": "integer",
            "extension": [
              { "url": "http://hl7.org/fhir/StructureDefinition/minValue", "valueInteger": 1 },
              { "url": "http://hl7.org/fhir/StructureDefinition/maxValue", "valueInteger": 5 } ] },
          { "linkId": "vitals.measures", "text": "Maßnahmen", "type": "choice", "repeats": true, "answerOption": [
            { "valueCoding": { "code": "oxygen", "display": "Sauerstoff" } },
            { "valueCoding": { "code": "splint", "display": "Schiene" } } ] },
          { "linkId": "vitals.gcs", "text": "GCS", "type": "decimal", "readOnly": true,
            "extension": [{ "url": "http://hl7.org/fhir/uv/sdc/StructureDefinition/sdc-questionnaire-calculatedExpression",
              "valueExpression": { "language": "text/fhirpath",
                "expression": "%resource.repeat(item).where(linkId='vitals.gcs_eye').answer.value + %resource.repeat(item).where(linkId='vitals.gcs_verbal').answer.value" } }] } ] }
      ]
    }"#;

    // [utest->dsn~patient-care-record~1]
    #[test]
    fn questionnaires_are_read_in_the_ordo_subset() {
        let c = catalogue();
        let identity = c.form("identity").expect("form");
        assert_eq!(identity.kind, FormKind::Snapshot);
        let name = identity.field("family_name").expect("field");
        assert_eq!(name.code.as_deref(), Some("fhir:Patient.name.family"));
        // [utest->dsn~person-suggestions~1]
        assert_eq!(
            identity.field("recorded_by").map(|f| f.field_type),
            Some(FieldType::Person),
            "a reference to a Practitioner is a person field"
        );
        let vitals = c.form("vitals").expect("form");
        assert_eq!(
            vitals.kind,
            FormKind::Series,
            "a repeating group is a series"
        );
        let hr = vitals.field("heart_rate").expect("field");
        assert_eq!(hr.unit.as_deref(), Some("/min"));
        assert_eq!((hr.min, hr.max), (Some(0.0), Some(300.0)));
        assert_eq!(hr.code.as_deref(), Some("loinc:8867-4"));
        assert_eq!(hr.hint.as_deref(), Some("Schläge pro Minute"));
        assert_eq!(
            vitals.field("gcs_eye").map(|f| f.field_type),
            Some(FieldType::Number)
        );
        assert_eq!(
            vitals.field("measures").map(|f| f.field_type),
            Some(FieldType::Choices)
        );
        assert_eq!(vitals.sums[0].of, vec!["gcs_eye", "gcs_verbal"]);
    }

    #[test]
    fn anything_outside_the_subset_is_rejected() {
        let replace = |from: &str, to: &str| {
            FormCatalogue::from_questionnaire_json(&QUESTIONNAIRE.replace(from, to))
        };
        // Calculations other than sums (EPCR-9).
        let product = replace(".answer.value + %resource", ".answer.value * %resource");
        assert!(
            matches!(product, Err(CatalogueError::Unsupported(_))),
            "{product:?}"
        );
        // Sums across forms.
        assert!(replace("linkId='vitals.gcs_verbal'", "linkId='identity.sex'").is_err());
        // Conditional questions are not supported yet.
        let conditional = replace(
            r#""text": "Name", "type": "string","#,
            r#""text": "Name", "type": "string", "enableWhen": [{}],"#,
        );
        assert!(matches!(conditional, Err(CatalogueError::Unsupported(_))));
        // References only to people (Practitioner).
        let patient = replace(
            r#""valueCode": "Practitioner""#,
            r#""valueCode": "Patient""#,
        );
        assert!(
            matches!(patient, Err(CatalogueError::Unsupported(_))),
            "{patient:?}"
        );
        // linkIds must name their group.
        assert!(
            replace(
                r#""linkId": "identity.family_name""#,
                r#""linkId": "family_name""#
            )
            .is_err()
        );
        assert!(FormCatalogue::from_questionnaire_json(r#"{"resourceType": "Patient"}"#).is_err());
    }

    fn one(text: &str) -> Value {
        Value::One(text.to_owned())
    }

    fn event(n: u64, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        crate::testing::event(n, "anna", MODULE, "med.clinical", event_type, payload)
    }

    fn entry(n: u64, form: &str, values: &[(&str, Value)], replaces: Option<u64>) -> AppliedEvent {
        let replaces = replaces
            .map(|r| EventId(Ulid::from_parts(r, u128::from(r)).expect("valid")).to_string());
        let payload = EntryRecorded {
            tag: "DE-0042".into(),
            form: form.into(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect(),
            replaces,
        }
        .encode()
        .expect("encodes");
        event(n, ENTRY_RECORDED, payload)
    }

    #[test]
    fn the_profile_catalogue_is_valid() {
        catalogue().check().expect("valid");
        let de = FormCatalogue::from_questionnaire_json(include_str!(
            "../../../../profiles/de/med/epcr-forms.json"
        ))
        .expect("DE profile is a valid questionnaire");
        assert_eq!(de.forms.len(), 8);
        let vitals = de.form("vitals").expect("vital signs");
        assert_eq!(vitals.kind, FormKind::Series);
        assert_eq!(
            vitals.sums[0].of,
            vec!["gcs_eye", "gcs_verbal", "gcs_motor"]
        );
    }

    #[test]
    fn mistakes_in_a_catalogue_are_found() {
        let mut c = catalogue();
        c.forms[0].fields[1].options.clear();
        assert_eq!(
            c.check(),
            Err(CatalogueError::Options("identity.sex".into()))
        );
        let mut c = catalogue();
        c.forms[1].sums[0].of.push("measures".into());
        assert_eq!(c.check(), Err(CatalogueError::Sum("vitals.gcs".into())));
        let mut c = catalogue();
        c.forms[1].fields[0].id = "Heart rate".into();
        assert_eq!(c.check(), Err(CatalogueError::Id("Heart rate".into())));
        let mut c = catalogue();
        c.forms[1].fields[1].id = "heart_rate".into();
        assert_eq!(
            c.check(),
            Err(CatalogueError::Duplicate("vitals.heart_rate".into()))
        );
    }

    #[test]
    fn values_are_checked_and_normalised() {
        let c = catalogue();
        let vitals = c.form("vitals").expect("form");
        let hr = vitals.field("heart_rate").expect("field");
        assert_eq!(hr.normalise(&one(" 72 ")), Ok(one("72")));
        assert_eq!(hr.normalise(&one("37,5")), Ok(one("37.5")));
        assert_eq!(hr.normalise(&one("zwölf")), Err(ValueError::NotANumber));
        assert_eq!(hr.normalise(&one("1.2.3")), Err(ValueError::NotANumber));
        assert!(matches!(
            hr.normalise(&one("400")),
            Err(ValueError::OutOfRange { .. })
        ));
        assert_eq!(hr.normalise(&one("")), Ok(one("")));
        let measures = vitals.field("measures").expect("field");
        assert_eq!(
            measures.normalise(&Value::Many(vec![
                "splint".into(),
                "oxygen".into(),
                "splint".into()
            ])),
            Ok(Value::Many(vec!["oxygen".into(), "splint".into()])),
            "profile order, no duplicates"
        );
        assert_eq!(
            measures.normalise(&Value::Many(vec!["magic".into()])),
            Err(ValueError::UnknownOption("magic".into()))
        );
        let sex = c
            .form("identity")
            .and_then(|f| f.field("sex"))
            .expect("field");
        assert_eq!(sex.normalise(&one("female")), Ok(one("female")));
        assert_eq!(sex.display(&one("female")), "weiblich");
        assert_eq!(hr.display(&one("72")), "72 /min");
        let problems = vitals
            .normalise(&[("pulse".to_owned(), one("1"))].into())
            .expect_err("unknown field");
        assert_eq!(
            problems,
            vec![("pulse".to_owned(), ValueError::UnknownField)]
        );
    }

    #[test]
    fn dates_and_times_are_checked() {
        assert!(is_date("2026-11-08"));
        assert!(!is_date("08.11.2026"));
        assert!(!is_date("2026-13-01"));
        assert!(is_time("23:59"));
        assert!(!is_time("24:00"));
        assert!(!is_time("9:5"));
    }

    #[test]
    fn sums_need_every_part_and_only_add() {
        let c = catalogue();
        let vitals = c.form("vitals").expect("form");
        let values: BTreeMap<String, Value> = [
            ("gcs_eye".to_owned(), one("3")),
            ("gcs_verbal".to_owned(), one("4")),
        ]
        .into();
        let sums = vitals.sums(&values);
        assert_eq!(sums.len(), 1);
        assert_eq!(sums[0].1, "7");
        let partial: BTreeMap<String, Value> = [("gcs_eye".to_owned(), one("3"))].into();
        assert!(
            vitals.sums(&partial).is_empty(),
            "no total from part of the values"
        );
    }

    // [utest->dsn~patient-care-record~1]
    #[test]
    fn snapshot_fields_keep_their_history() {
        let events = [
            entry(1, "identity", &[("family_name", one("Muster"))], None),
            entry(
                2,
                "identity",
                &[("family_name", one("Mustermann")), ("sex", one("male"))],
                None,
            ),
            entry(3, "identity", &[("sex", one(""))], None),
        ];
        let records = Records::from_events(&events);
        let record = records.get("DE-0042").expect("record");
        assert_eq!(
            record.current("identity", "family_name").map(|s| &s.value),
            Some(&one("Mustermann"))
        );
        assert_eq!(
            record.current("identity", "sex"),
            None,
            "a cleared field is empty"
        );
        let history = &record.fields[&("identity".to_owned(), "family_name".to_owned())];
        assert_eq!(history.len(), 2, "earlier values stay");
    }

    // [utest->dsn~patient-care-record~1]
    #[test]
    fn corrections_replace_entries_without_deleting_them() {
        let mut events = vec![
            entry(1, "vitals", &[("heart_rate", one("27"))], None),
            entry(2, "vitals", &[("heart_rate", one("90"))], None),
            entry(3, "vitals", &[("heart_rate", one("72"))], Some(1)),
        ];
        let forward = Records::from_events(&events);
        events.reverse();
        assert_eq!(
            Records::from_events(&events),
            forward,
            "order of arrival doesn't matter"
        );
        let record = forward.get("DE-0042").expect("record");
        let current: Vec<_> = record
            .current_entries("vitals")
            .map(|e| e.values["heart_rate"].clone())
            .collect();
        assert_eq!(current, vec![one("90"), one("72")]);
        assert_eq!(
            record.entries["vitals"].len(),
            3,
            "the corrected entry stays"
        );
        assert!(record.entries["vitals"][0].replaced_by.is_some());
    }

    // [utest->dsn~patient-photo~2]
    #[test]
    fn photos_are_limited_in_size_and_type() {
        let photo = |media_type: &str, len: usize| PhotoAdded {
            tag: "DE-0042".into(),
            media_type: media_type.into(),
            data: vec![0xff; len],
        };
        assert!(photo("image/jpeg", 1000).encode().is_ok());
        assert_eq!(
            photo("image/jpeg", MAX_PHOTO_BYTES + 1).check(),
            Err(PhotoError::TooLarge(MAX_PHOTO_BYTES + 1))
        );
        assert!(
            photo("image/svg+xml", 10).check().is_err(),
            "no active content"
        );
        let ok = photo("image/jpeg", 1000).encode().expect("encodes");
        // CBOR byte string, not an array of numbers: little overhead.
        assert!(ok.len() < 1100, "{} bytes", ok.len());
        let records = Records::from_events(&[event(1, PHOTO_ADDED, ok)]);
        assert!(records.has_photo("DE-0042"));
        assert!(!records.has_photo("DE-0043"));
    }

    // [utest->dsn~patient-photo~2]
    #[test]
    fn attached_photos_keep_a_small_preview_and_refer_to_the_full_photo() {
        let attached = |preview: usize, size: u64, media_type: &str| PhotoAttached {
            tag: "DE-0042".into(),
            preview_type: "image/jpeg".into(),
            preview: vec![0xff; preview],
            photo: AttachmentRef {
                hash: [1; 32],
                size,
                media_type: media_type.into(),
                key: [2; 32],
            },
        };
        assert_eq!(
            attached(MAX_PREVIEW_BYTES + 1, 1000, "image/jpeg").check(),
            Err(PhotoError::TooLarge(MAX_PREVIEW_BYTES + 1))
        );
        assert!(
            attached(1000, MAX_FULL_PHOTO_BYTES as u64 + 41, "image/jpeg")
                .check()
                .is_err()
        );
        assert!(attached(1000, 1000, "image/svg+xml").check().is_err());
        let ok = attached(1000, 200_000, "image/jpeg")
            .encode()
            .expect("encodes");
        assert!(ok.len() < 1300, "the event stays small: {} bytes", ok.len());
        let records = Records::from_events(&[event(1, PHOTO_ATTACHED, ok)]);
        let photo = records
            .get("DE-0042")
            .and_then(Record::photo)
            .expect("photo");
        assert_eq!(photo.data.len(), 1000, "the preview");
        assert_eq!(photo.full.as_ref().map(|f| f.size), Some(200_000));
    }
}
