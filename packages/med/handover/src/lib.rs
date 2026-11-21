//! The handover document of the Med package (ADR-0089, EPCR-20): when a patient is handed
//! over outside ORDO, the server makes a PDF/A-3a that is also PDF/UA-1, laid out by the
//! profile's Typst template, with the same data as an HL7 FHIR bundle attached. Runs on the
//! server only, which may read the clinical compartment (ADR-0014).

pub mod data;
pub mod fhir;
mod world;

use ordo_med_patients::orders::OrderCatalogue;
use ordo_med_patients::record::FormCatalogue;
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};

pub use data::{Data, Events, Naming};

/// Why no document was made.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The patient is not in the incident.
    #[error("no patient with this triage tag")]
    UnknownPatient,
    /// The profile's template does not compile or breaks a PDF standard.
    #[error("the handover template failed: {0}")]
    Template(String),
    /// A profile file could not be read.
    #[error("the handover profile is invalid: {0}")]
    Profile(String),
}

/// What the document is made from: the profile's forms, order routes, Questionnaire,
/// template and fonts.
pub struct Profile {
    forms: FormCatalogue,
    routes: OrderCatalogue,
    questionnaire: fhir::Questionnaire,
    template: String,
    fonts: Vec<Vec<u8>>,
}

/// The DE profile's files, embedded when the server is built.
mod embedded {
    pub const FORMS: &str = include_str!("../../../../profiles/de/med/epcr-forms.json");
    pub const ORDERS: &str = include_str!("../../../../profiles/de/med/orders.toml");
    pub const TEMPLATE: &str = include_str!("../../../../profiles/de/med/handover/handover.typ");
    pub const FONTS: &[&[u8]] = &[
        include_bytes!(
            "../../../../profiles/de/med/handover/fonts/AtkinsonHyperlegibleNext[wght].ttf"
        ),
        include_bytes!(
            "../../../../profiles/de/med/handover/fonts/AtkinsonHyperlegibleNext-Italic[wght].ttf"
        ),
    ];
}

impl Profile {
    /// The profile the server was built with.
    pub fn embedded() -> Result<Self, Error> {
        Self::new(
            embedded::FORMS,
            embedded::ORDERS,
            embedded::TEMPLATE.to_owned(),
            embedded::FONTS.iter().map(|f| f.to_vec()).collect(),
        )
    }

    /// A profile from its files: the forms (FHIR Questionnaire), the order catalogue, the
    /// template and its fonts (TrueType or OpenType).
    pub fn new(
        forms: &str,
        orders: &str,
        template: String,
        fonts: Vec<Vec<u8>>,
    ) -> Result<Self, Error> {
        Ok(Self {
            forms: FormCatalogue::from_questionnaire_json(forms)
                .map_err(|e| Error::Profile(e.to_string()))?,
            routes: toml::from_str(orders).map_err(|e| Error::Profile(e.to_string()))?,
            questionnaire: fhir::Questionnaire::from_json(forms)
                .ok_or_else(|| Error::Profile("the Questionnaire has no url or version".into()))?,
            template,
            fonts,
        })
    }

    /// The same profile with an organisation's own template and fonts.
    pub fn with_template(mut self, template: String, fonts: Vec<Vec<u8>>) -> Self {
        self.template = template;
        if !fonts.is_empty() {
            self.fonts = fonts;
        }
        self
    }
}

/// A handover document: the PDF, and the bundle attached to it.
pub struct Document {
    /// The PDF/A-3a (and PDF/UA-1) file.
    pub pdf: Vec<u8>,
    /// The HL7 FHIR bundle attached to it.
    pub bundle: serde_json::Value,
}

/// The file name of the attached bundle, e.g. `fhir-bundle-DE-0045.json`: PDF readers
/// list it under the PDF's attachments.
pub fn bundle_name(tag: &str) -> String {
    let tag: String = tag
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("fhir-bundle-{tag}.json")
}

/// The line added to every template: the FHIR bundle is always attached, whatever the
/// template does.
fn attach(name: &str) -> String {
    format!(
        "\n#pdf.attach(\"/{name}\", relationship: \"data\", mime-type: \"application/fhir+json\", description: \"HL7 FHIR Bundle\")\n"
    )
}

// [impl->dsn~handover-document~1]
/// Makes the handover document of the patient `tag` at `now` (ms since the Unix epoch).
pub fn make(
    profile: &Profile,
    events: &Events,
    tag: &str,
    now: u64,
    naming: &Naming,
) -> Result<Document, Error> {
    let data = data::gather(events, &profile.forms, &profile.routes, tag, now, naming)
        .ok_or(Error::UnknownPatient)?;
    let records = ordo_med_patients::record::Records::from_events(events.clinical);
    let record = records.get(tag);
    let bundle = fhir::bundle(
        &data,
        record,
        &profile.forms,
        &profile.questionnaire,
        naming,
    );
    let pdf = render(
        profile,
        &data,
        &bundle,
        record.and_then(|r| r.photo()),
        naming.offset_minutes,
    )?;
    Ok(Document { pdf, bundle })
}

/// Compiles the template with `data` and writes PDF/A-3a and PDF/UA-1.
fn render(
    profile: &Profile,
    data: &Data,
    bundle: &serde_json::Value,
    photo: Option<&ordo_med_patients::record::Photo>,
    offset_minutes: i32,
) -> Result<Vec<u8>, Error> {
    let bundle_path = format!("/{}", bundle_name(&data.tag));
    let mut files = vec![
        ("/data.json", serde_json::to_vec(data).unwrap_or_default()),
        (
            bundle_path.as_str(),
            serde_json::to_vec(bundle).unwrap_or_default(),
        ),
    ];
    if let (Some(photo), Some(path)) = (photo, data.photo.as_deref()) {
        files.push((path, photo.data.clone()));
    }
    let part = |text: &str, range: std::ops::Range<usize>| -> u8 {
        text.get(range).and_then(|p| p.parse().ok()).unwrap_or(0)
    };
    let (date, time) = (&data.created.date, &data.created.time);
    let year: i32 = date.get(0..4).and_then(|y| y.parse().ok()).unwrap_or(1970);
    let today = (year, part(date, 5..7), part(date, 8..10));
    let world = world::DocumentWorld::new(
        format!("{}{}", profile.template, attach(&bundle_name(&data.tag))),
        files,
        &profile.fonts,
        today,
    );
    let document: PagedDocument = typst::compile(&world)
        .output
        .map_err(|errors| Error::Template(messages(&errors)))?;
    // PDF/A needs the document's date: when it was made, in the device's time zone.
    let timestamp = typst::foundations::Datetime::from_ymd_hms(
        today.0,
        today.1,
        today.2,
        part(time, 0..2),
        part(time, 3..5),
        0,
    )
    .and_then(|at| typst_pdf::Timestamp::new_local(at, offset_minutes));
    let options = PdfOptions {
        standards: PdfStandards::new(&[PdfStandard::A_3a, PdfStandard::Ua_1])
            .map_err(|e| Error::Template(e.message().to_string()))?,
        timestamp,
        ..PdfOptions::default()
    };
    typst_pdf::pdf(&document, &options).map_err(|errors| Error::Template(messages(&errors)))
}

fn messages(errors: &[typst::diag::SourceDiagnostic]) -> String {
    errors
        .iter()
        .map(|e| e.message.to_string())
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, Hlc, IncidentId, ServerName, Ulid, UserId};
    use ordo_med_patients::record::{self, EntryRecorded, Value};
    use ordo_med_patients::{self as med, HandedOver, Registered, UhsCreated, UhsKind};
    use ordo_sync::AppliedEvent;

    use super::*;

    /// The `n`-th event of one incident, by `user`, at minute `n` after 14:00 UTC.
    fn event(
        n: u64,
        user: &str,
        module: &str,
        compartment: &str,
        kind: &str,
        payload: Vec<u8>,
    ) -> AppliedEvent {
        let server: ServerName = "ordo.example.org".parse().expect("valid server");
        let device = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
        AppliedEvent {
            seq: Some(n),
            body: EventBody {
                version: 1,
                id: EventId(Ulid::from_parts(n, u128::from(n)).expect("valid")),
                schema_version: 1,
                hlc: Hlc {
                    wall_ms: 1_795_788_000_000 + n * 60_000,
                    counter: 0,
                    node: device,
                },
                incident: IncidentId::new(Ulid::from_parts(9, 9).expect("valid"), &server),
                module: module.parse().expect("valid"),
                compartment: compartment.parse().expect("valid"),
                event_type: kind.parse().expect("valid"),
                author: Author {
                    user: format!("{user}@ordo.example.org")
                        .parse()
                        .expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload,
        }
    }

    fn entry(n: u64, form: &str, values: &[(&str, &str)]) -> AppliedEvent {
        let e = EntryRecorded {
            tag: "DE-0045".into(),
            form: form.into(),
            values: values
                .iter()
                .map(|(k, v)| ((*k).to_owned(), Value::One((*v).to_owned())))
                .collect::<BTreeMap<_, _>>(),
            replaces: None,
            order: None,
        };
        event(
            n,
            "anna",
            record::MODULE,
            med::COMPARTMENT,
            record::ENTRY_RECORDED,
            e.encode().unwrap(),
        )
    }

    // [utest->dsn~handover-document~1]
    #[test]
    fn a_patient_handed_outside_gets_a_pdf_a_with_the_fhir_bundle() {
        let uhs = UhsCreated {
            id: "eingang".into(),
            name: "UHS Eingang".into(),
            kind: UhsKind::Uhs,
            capacity: 5,
        };
        let registered = Registered {
            report: None,
            unit: None,
            task: None,
            tag: "DE-0045".into(),
            uhs: "eingang".into(),
            category: ordo_contracts::TriageCategory::Urgent,
            algorithm: None,
        };
        let handed = HandedOver {
            tag: "DE-0045".into(),
            handover: "01JDHANDOVER0000000000000".into(),
            to: None,
            outside: Some("Klinikum Mitte, ZNA".into()),
            transport: None,
            note: Some("Schmerzen im linken Arm".into()),
        };
        let incident = vec![event(
            1,
            "ben",
            med::UHS_MODULE,
            med::UHS_COMPARTMENT,
            med::UHS_CREATED,
            uhs.encode().unwrap(),
        )];
        let clinical = vec![
            event(
                2,
                "anna",
                med::MODULE,
                med::COMPARTMENT,
                med::REGISTERED,
                registered.encode().unwrap(),
            ),
            entry(
                3,
                "identity",
                &[("family_name", "Muster"), ("given_name", "Max")],
            ),
            entry(4, "vitals", &[("heart_rate", "88")]),
            event(
                5,
                "anna",
                med::MODULE,
                med::COMPARTMENT,
                med::HANDED_OVER,
                codec(&handed),
            ),
        ];
        let person = |u: &UserId| match u.localpart() {
            "anna" => "Anna Schmidt".to_owned(),
            other => other.to_owned(),
        };
        let naming = Naming {
            offset_minutes: 60,
            person: &person,
        };
        let profile = Profile::embedded().expect("the DE profile");
        let events = Events {
            clinical: &clinical,
            incident: &incident,
        };
        let now = 1_795_788_000_000 + 6 * 60_000;
        let doc = make(&profile, &events, "DE-0045", now, &naming).expect("a document");

        assert!(doc.pdf.starts_with(b"%PDF-"), "a PDF");
        if let Ok(path) = std::env::var("ORDO_HANDOVER_SAMPLE") {
            std::fs::write(path, &doc.pdf).unwrap();
        }
        let text = String::from_utf8_lossy(&doc.pdf);
        assert!(text.contains("pdfaid:part"), "PDF/A metadata");
        assert!(text.contains("pdfuaid:part"), "PDF/UA metadata");
        assert!(text.contains("EmbeddedFile"), "the bundle is attached");
        assert!(text.contains("fhir-bundle-DE-0045.json"), "under its name");

        let b = &doc.bundle;
        assert_eq!(b["type"], "document");
        let entries = b["entry"].as_array().unwrap();
        assert_eq!(
            entries[0]["resource"]["resourceType"], "Composition",
            "first, as FHIR requires"
        );
        let patient = &entries[1]["resource"];
        assert_eq!(patient["identifier"][0]["value"], "DE-0045");
        assert_eq!(patient["name"][0]["family"], "Muster");
        let vitals = entries
            .iter()
            .map(|e| &e["resource"])
            .find(|r| r["item"][0]["linkId"] == "vitals")
            .expect("the vital signs as a QuestionnaireResponse");
        assert_eq!(vitals["item"][0]["item"][0]["linkId"], "vitals.heart_rate");
        assert_eq!(
            vitals["item"][0]["item"][0]["answer"][0]["valueDecimal"],
            88.0
        );
        assert_eq!(vitals["author"]["display"], "Anna Schmidt");
        assert_eq!(
            b["timestamp"], "2026-11-27T15:06:00+01:00",
            "in the device's time zone"
        );
    }

    fn codec(h: &HandedOver) -> Vec<u8> {
        ordo_core::codec::to_cbor(h).unwrap()
    }

    #[test]
    fn an_unknown_patient_gets_no_document() {
        let profile = Profile::embedded().unwrap();
        let person = |u: &UserId| u.to_string();
        let naming = Naming {
            offset_minutes: 0,
            person: &person,
        };
        let events = Events {
            clinical: &[],
            incident: &[],
        };
        assert!(matches!(
            make(&profile, &events, "DE-0001", 0, &naming),
            Err(Error::UnknownPatient)
        ));
    }
}
