//! What the handover document says, gathered from the events (ADR-0089): the data the
//! template lays out, as JSON. It holds IDs, the profile's labels and what people wrote;
//! the template (in the profile) chooses the words, so this code has none of its own.

use ordo_core::UserId;
use ordo_med_patients::orders::{OrderBook, OrderCatalogue, Ordered};
use ordo_med_patients::record::{FieldType, Form, FormCatalogue, FormKind, Record, Value};
use ordo_med_patients::{self as med, Holder, Patient, PatientRegister, Stay};
use ordo_sync::AppliedEvent;
use serde::Serialize;

/// A moment as the template shows it, in the device's time zone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Moment {
    /// `2026-11-21`.
    pub date: String,
    /// `14:35`.
    pub time: String,
    /// RFC 3339 with offset, for FHIR: `2026-11-21T14:35:00+01:00`.
    pub iso: String,
}

/// A label and its value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Fact {
    /// The label from the profile.
    pub label: String,
    /// The value in words.
    pub value: String,
}

/// One triage decision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Triage {
    /// When.
    pub at: Moment,
    /// The category's ID, e.g. `urgent`; the template names it.
    pub category: String,
    /// Who (their display name).
    pub by: String,
    /// The triage algorithm named, if any.
    pub algorithm: Option<String>,
}

/// One step of the patient's way.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Step {
    /// When.
    pub at: Moment,
    /// `registered`, `readmitted`, `moved`, `handedOver`, `taken`, `takenBack`,
    /// `discharged` or `handedOutside`.
    pub kind: &'static str,
    /// The holder or receiver it names, if any.
    pub holder: Option<String>,
    /// The discharge's outcome ID, if discharged.
    pub outcome: Option<String>,
    /// Who (their display name).
    pub by: String,
}

/// The handover the document is for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Handover {
    /// When.
    pub at: Moment,
    /// Who (their display name).
    pub by: String,
    /// Who held the patient.
    pub from: String,
    /// Who takes the patient (outside ORDO: as written; within: the holder).
    pub to: String,
    /// The note for the receiver.
    pub note: Option<String>,
}

/// One entry of a series form, or the current values of a snapshot form.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// When.
    pub at: Moment,
    /// Who (their display name).
    pub by: String,
    /// The values documented.
    pub values: Vec<Fact>,
}

/// A form of the record with what was documented in it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FormData {
    /// The form's ID from the profile.
    pub id: String,
    /// The label from the profile.
    pub label: String,
    /// `snapshot` or `series`.
    pub kind: &'static str,
    /// The entries, oldest first.
    pub entries: Vec<Entry>,
}

/// An order (ORDER-1) and how far it got.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OrderData {
    /// When.
    pub at: Moment,
    /// Who (their display name).
    pub by: String,
    /// What was ordered, in words.
    pub what: String,
    /// `given`, `acknowledged`, `done` or `withdrawn`.
    pub status: &'static str,
    /// When it was carried out.
    pub done: Option<Moment>,
    /// Who carried it out.
    pub performed_by: Option<String>,
}

/// Everything the template gets (`json("/data.json")`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Data {
    /// The triage tag number.
    pub tag: String,
    /// Created.
    pub created: Moment,
    /// The current category's ID.
    pub category: String,
    /// Triage.
    pub triage: Vec<Triage>,
    /// The header fields of the profile with their current values (ADR-0088).
    pub facts: Vec<Fact>,
    /// Handover.
    pub handover: Option<Handover>,
    /// Way.
    pub way: Vec<Step>,
    /// Forms.
    pub forms: Vec<FormData>,
    /// Orders.
    pub orders: Vec<OrderData>,
    /// The photo's path for `image(…)`, if the record has one in a format Typst reads.
    pub photo: Option<String>,
}

/// How times and people are named: the device's UTC offset and the people's names.
pub struct Naming<'a> {
    /// The device's offset from UTC in minutes.
    pub offset_minutes: i32,
    /// A person's display name.
    pub person: &'a dyn Fn(&UserId) -> String,
}

impl Naming<'_> {
    /// A moment `ms` after the Unix epoch, in the device's time zone.
    pub fn moment(&self, ms: u64) -> Moment {
        let offset = time::UtcOffset::from_whole_seconds(self.offset_minutes * 60)
            .unwrap_or(time::UtcOffset::UTC);
        let at = i64::try_from(ms / 1000)
            .ok()
            .and_then(|s| time::OffsetDateTime::from_unix_timestamp(s).ok())
            .unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
            .to_offset(offset);
        let date = format!(
            "{:04}-{:02}-{:02}",
            at.year(),
            u8::from(at.month()),
            at.day()
        );
        let time = format!("{:02}:{:02}", at.hour(), at.minute());
        let (h, m, _) = offset.as_hms();
        let sign = if self.offset_minutes < 0 { '-' } else { '+' };
        let iso = format!(
            "{date}T{time}:{:02}{sign}{:02}:{:02}",
            at.second(),
            h.unsigned_abs(),
            m.unsigned_abs()
        );
        Moment { date, time, iso }
    }

    fn person(&self, user: &UserId) -> String {
        (self.person)(user)
    }
}

/// The events the document reads: the clinical compartment, and the incident's (UHS).
pub struct Events<'a> {
    /// The clinical compartment's events.
    pub clinical: &'a [AppliedEvent],
    /// The incident compartment's events (UHS names).
    pub incident: &'a [AppliedEvent],
}

/// A holder as the document names it: a UHS by its name, a unit by its call sign.
fn holder_name(uhs: &[med::UhsCreated], holder: &Holder) -> String {
    match holder {
        Holder::Uhs(id) => uhs
            .iter()
            .find(|u| &u.id == id)
            .map_or_else(|| id.clone(), |u| u.name.clone()),
        Holder::Unit(unit) => unit.clone(),
    }
}

/// A value in words: choices by their labels, numbers with their unit, people by name.
fn words(field: &ordo_med_patients::record::Field, value: &Value, naming: &Naming) -> String {
    let label = |id: &str| {
        field
            .options
            .iter()
            .find(|o| o.id == id)
            .map_or_else(|| id.to_owned(), |o| o.label.clone())
    };
    match value {
        Value::Many(list) => list.iter().map(|v| label(v)).collect::<Vec<_>>().join(", "),
        Value::One(text) => match field.field_type {
            FieldType::Choice => label(text),
            FieldType::Number => match &field.unit {
                Some(unit) => format!("{text} {unit}"),
                None => text.clone(),
            },
            FieldType::Person => text
                .parse::<UserId>()
                .map_or_else(|_| text.clone(), |u| naming.person(&u)),
            _ => text.clone(),
        },
    }
}

fn facts(form: &Form, values: impl Fn(&str) -> Option<Value>, naming: &Naming) -> Vec<Fact> {
    form.fields
        .iter()
        .filter_map(|f| {
            let v = values(&f.id).filter(|v| !v.is_empty())?;
            Some(Fact {
                label: f.label.clone(),
                value: words(f, &v, naming),
            })
        })
        .collect()
}

fn forms(forms: &FormCatalogue, record: Option<&Record>, naming: &Naming) -> Vec<FormData> {
    let Some(record) = record else {
        return Vec::new();
    };
    forms
        .forms
        .iter()
        .filter_map(|form| {
            let entries: Vec<Entry> = match form.kind {
                FormKind::Snapshot => {
                    let values = facts(
                        form,
                        |field| record.current(&form.id, field).map(|s| s.value.clone()),
                        naming,
                    );
                    // The latest change of the form names when and by whom.
                    let latest = form
                        .fields
                        .iter()
                        .filter_map(|f| record.current(&form.id, &f.id))
                        .max_by_key(|s| s.at);
                    match latest {
                        Some(s) if !values.is_empty() => vec![Entry {
                            at: naming.moment(s.at.wall_ms),
                            by: naming.person(&s.by),
                            values,
                        }],
                        _ => Vec::new(),
                    }
                }
                FormKind::Series => record
                    .current_entries(&form.id)
                    .map(|e| Entry {
                        at: naming.moment(e.at.wall_ms),
                        by: naming.person(&e.by),
                        values: facts(form, |field| e.values.get(field).cloned(), naming),
                    })
                    .filter(|e| !e.values.is_empty())
                    .collect(),
            };
            (!entries.is_empty()).then(|| FormData {
                id: form.id.clone(),
                label: form.label.clone(),
                kind: match form.kind {
                    FormKind::Snapshot => "snapshot",
                    FormKind::Series => "series",
                },
                entries,
            })
        })
        .collect()
}

fn way(patient: &Patient, uhs: &[med::UhsCreated], naming: &Naming) -> Vec<Step> {
    let mut holder = first_holder(patient);
    patient
        .stay
        .iter()
        .enumerate()
        .map(|(i, e)| {
            let name = |h: &Holder| Some(holder_name(uhs, h));
            let (kind, named, outcome) = match &e.change {
                Stay::Admitted if i == 0 => ("registered", name(&holder), None),
                Stay::Admitted => ("readmitted", name(&holder), None),
                Stay::Moved { to, .. } => {
                    holder = Holder::Uhs(to.clone());
                    ("moved", name(&holder), None)
                }
                Stay::HandedOver { to, .. } => ("handedOver", to.as_ref().and_then(name), None),
                Stay::Taken { from, to } if from == to => ("takenBack", name(to), None),
                Stay::Taken { to, .. } => {
                    holder = to.clone();
                    ("taken", name(to), None)
                }
                Stay::Discharged { outcome, .. } => {
                    ("discharged", None, Some(outcome.id().to_owned()))
                }
                Stay::HandedOutside { receiver, .. } => {
                    ("handedOutside", Some(receiver.clone()), None)
                }
            };
            Step {
                at: naming.moment(e.at.wall_ms),
                kind,
                holder: named,
                outcome,
                by: naming.person(&e.by),
            }
        })
        .collect()
}

/// The holder the patient was registered with.
fn first_holder(patient: &Patient) -> Holder {
    patient
        .stay
        .iter()
        .find_map(|s| match &s.change {
            Stay::Moved { from, .. } => Some(Holder::Uhs(from.clone())),
            Stay::Taken { from, .. } => Some(from.clone()),
            _ => None,
        })
        .unwrap_or_else(|| patient.holder())
}

/// The latest handover of the patient, with the holder it left.
fn handover(
    clinical: &[AppliedEvent],
    patient: &Patient,
    uhs: &[med::UhsCreated],
    naming: &Naming,
) -> Option<Handover> {
    let latest = clinical
        .iter()
        .filter(|e| e.body.event_type.as_str() == med::HANDED_OVER)
        .filter_map(|e| {
            let h: med::HandedOver = ordo_core::codec::from_cbor(&e.payload).ok()?;
            (h.tag == patient.tag).then_some((e, h))
        })
        .max_by_key(|(e, _)| e.body.hlc)?;
    let (event, h) = latest;
    // Who held the patient when it was handed over: the holder before the latest
    // taking after it, else the holder now.
    let at = event.body.hlc;
    let from = patient
        .stay
        .iter()
        .filter(|s| s.at > at)
        .find_map(|s| match &s.change {
            Stay::Taken { from, .. } => Some(from.clone()),
            _ => None,
        })
        .unwrap_or_else(|| patient.holder());
    let to = match (&h.outside, &h.to) {
        (Some(receiver), _) => receiver.clone(),
        (None, Some(to)) => holder_name(uhs, to),
        (None, None) => String::new(),
    };
    Some(Handover {
        at: naming.moment(at.wall_ms),
        by: naming.person(&event.body.author.user),
        from: holder_name(uhs, &from),
        to,
        note: h.note,
    })
}

fn orders(
    clinical: &[AppliedEvent],
    routes: &OrderCatalogue,
    tag: &str,
    naming: &Naming,
) -> Vec<OrderData> {
    OrderBook::from_events(clinical)
        .of_patient(tag)
        .into_iter()
        .map(|o| {
            let what = match &o.given.ordered {
                Ordered::Medication {
                    substance,
                    dose,
                    route,
                } => format!("{substance} {dose} {}", routes.route_label(route.as_str())),
                Ordered::Measure { text } => text.clone(),
            };
            use ordo_med_patients::orders::OrderStatus as S;
            let done = o.done();
            OrderData {
                at: naming.moment(o.at().wall_ms),
                by: o.giver(),
                what,
                status: match o.status {
                    S::Given => "given",
                    S::Acknowledged => "acknowledged",
                    S::Done => "done",
                    S::Withdrawn => "withdrawn",
                },
                done: done.map(|d| naming.moment(d.at.wall_ms)),
                performed_by: done.map(|d| d.performer()),
            }
        })
        .collect()
}

// [impl->dsn~handover-document~1]
/// The document's data for the patient `tag`, or `None` if they are unknown. `now` is when
/// it is made (ms since the Unix epoch).
pub fn gather(
    events: &Events,
    catalogue: &FormCatalogue,
    routes: &OrderCatalogue,
    tag: &str,
    now: u64,
    naming: &Naming,
) -> Option<Data> {
    let register = PatientRegister::from_events(events.clinical);
    let patient = register.patient(tag)?;
    let uhs = med::uhs_list(events.incident);
    let records = med::record::Records::from_events(events.clinical);
    let record = records.get(tag);
    let facts = catalogue
        .forms
        .iter()
        .flat_map(|form| {
            form.fields.iter().filter(|f| f.header).filter_map(|f| {
                let v = record?.current(&form.id, &f.id)?;
                Some(Fact {
                    label: f.label.clone(),
                    value: words(f, &v.value, naming),
                })
            })
        })
        .collect();
    Some(Data {
        tag: patient.tag.clone(),
        created: naming.moment(now),
        category: id(patient.category()),
        triage: patient
            .history
            .iter()
            .map(|t| Triage {
                at: naming.moment(t.at.wall_ms),
                category: id(t.category),
                by: naming.person(&t.by),
                algorithm: t.algorithm.clone(),
            })
            .collect(),
        facts,
        handover: handover(events.clinical, patient, &uhs, naming),
        way: way(patient, &uhs, naming),
        forms: forms(catalogue, record, naming),
        orders: orders(events.clinical, routes, tag, naming),
        photo: record
            .and_then(Record::photo)
            .and_then(|p| photo_path(&p.media_type))
            .map(str::to_owned),
    })
}

/// The ID of a triage category as in events, e.g. `urgent`.
fn id(category: ordo_contracts::TriageCategory) -> String {
    serde_json::to_value(category)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Where the photo is given to the template, for the formats Typst reads.
pub(crate) fn photo_path(media_type: &str) -> Option<&'static str> {
    match media_type {
        "image/jpeg" => Some("/photo.jpg"),
        "image/png" => Some("/photo.png"),
        "image/webp" => Some("/photo.webp"),
        _ => None,
    }
}
