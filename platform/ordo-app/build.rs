//! Embeds the profile catalogues the app needs, checked here so a broken profile fails the
//! build: the ePCR forms, an HL7 FHIR Questionnaire (ADR-0037), handed to the app in ORDO's
//! internal form model; the transport priority settings.

use std::path::Path;

use ordo_med_patients::TransportPriorityConfig;
use ordo_med_patients::record::FormCatalogue;

fn main() {
    let profile = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/de/med");
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).to_path_buf();

    let source = profile.join("epcr-forms.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = std::fs::read_to_string(&source).expect("read the ePCR forms of the profile");
    let catalogue = FormCatalogue::from_questionnaire_json(&text)
        .unwrap_or_else(|e| panic!("ePCR forms in {}: {e}", source.display()));
    std::fs::write(
        out.join("epcr-forms.json"),
        serde_json::to_string(&catalogue).expect("serialise"),
    )
    .expect("write the ePCR forms");

    let source = profile.join("transport-priority.toml");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = std::fs::read_to_string(&source).expect("read the transport priority settings");
    let config: TransportPriorityConfig = toml::from_str(&text)
        .unwrap_or_else(|e| panic!("transport priority in {}: {e}", source.display()));
    assert!(
        !config.levels.is_empty(),
        "{}: at least one level",
        source.display()
    );
    std::fs::write(
        out.join("transport-priority.json"),
        serde_json::to_string(&config).expect("serialise"),
    )
    .expect("write the transport priority settings");
}
