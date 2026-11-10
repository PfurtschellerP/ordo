//! Embeds the profile catalogues the app needs, checked here so a broken profile fails the
//! build: the ePCR forms, an HL7 FHIR Questionnaire (ADR-0037), handed to the app in ORDO's
//! internal form model; the transport priority settings; the transport request catalogue;
//! the routes for medical orders; the hospital catalogue; the triage algorithms; the status
//! codes of units.

use std::path::Path;

use ordo_command::StructureCatalogue;
use ordo_med_patients::TransportPriorityConfig;
use ordo_med_patients::hospitals::HospitalCatalogue;
use ordo_med_patients::orders::OrderCatalogue;
use ordo_med_patients::record::FormCatalogue;
use ordo_med_patients::transport::TransportCatalogue;
use ordo_med_patients::triage::TriageCatalogue;
use ordo_tasks::TemplateCatalogue;
use ordo_units::{StatusCatalogue, UnitCatalogue};
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Reads a TOML catalogue of the profile, checks it and writes it as JSON to `OUT_DIR`.
fn embed<T: DeserializeOwned + Serialize>(
    profile: &Path,
    file: &str,
    check: impl Fn(&T) -> Result<(), String>,
) {
    let source = profile.join(file);
    println!("cargo:rerun-if-changed={}", source.display());
    let text = std::fs::read_to_string(&source)
        .unwrap_or_else(|e| panic!("read {}: {e}", source.display()));
    let value: T = toml::from_str(&text).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
    check(&value).unwrap_or_else(|e| panic!("{}: {e}", source.display()));
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join(
        Path::new(file)
            .with_extension("json")
            .file_name()
            .expect("file name"),
    );
    std::fs::write(out, serde_json::to_string(&value).expect("serialise"))
        .unwrap_or_else(|e| panic!("write {file}: {e}"));
}

fn main() {
    let profile = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/de");
    let med = profile.join("med");

    let source = med.join("epcr-forms.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = std::fs::read_to_string(&source).expect("read the ePCR forms of the profile");
    let catalogue = FormCatalogue::from_questionnaire_json(&text)
        .unwrap_or_else(|e| panic!("ePCR forms in {}: {e}", source.display()));
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).to_path_buf();
    std::fs::write(
        out.join("epcr-forms.json"),
        serde_json::to_string(&catalogue).expect("serialise"),
    )
    .expect("write the ePCR forms");

    embed(
        &med,
        "transport-priority.toml",
        |c: &TransportPriorityConfig| {
            (!c.levels.is_empty())
                .then_some(())
                .ok_or_else(|| "at least one level".to_owned())
        },
    );
    embed(&med, "transport-requests.toml", |c: &TransportCatalogue| {
        c.check().map_err(|e| e.to_string())
    });
    // The order settings must fit the ePCR forms they document into (ORDER-5).
    embed(&med, "orders.toml", |c: &OrderCatalogue| {
        if c.routes.is_empty() {
            return Err("at least one route".to_owned());
        }
        c.check_record(&catalogue)
    });
    embed(&med, "hospitals.toml", |c: &HospitalCatalogue| {
        c.check().map_err(|e| e.to_string())
    });
    embed(&med, "triage-algorithms.toml", TriageCatalogue::check);
    embed(
        &profile.join("units"),
        "catalogue.toml",
        UnitCatalogue::check,
    );
    embed(
        &profile.join("tasks"),
        "templates.toml",
        TemplateCatalogue::check,
    );
    embed(
        &profile.join("command"),
        "structure.toml",
        StructureCatalogue::check,
    );
    embed(
        &profile.join("units"),
        "status-codes.toml",
        StatusCatalogue::check,
    );
}
