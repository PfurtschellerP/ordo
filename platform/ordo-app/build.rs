//! Embeds the profile catalogues the app needs. The ePCR forms are an HL7 FHIR
//! Questionnaire (ADR-0037); they are checked here, so a broken profile fails the build,
//! and handed to the app in ORDO's internal form model (smaller, nothing to parse twice).

use std::path::Path;

use ordo_med_patients::record::FormCatalogue;

fn main() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/de/med/epcr-forms.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = std::fs::read_to_string(&source).expect("read the ePCR forms of the profile");
    let catalogue = FormCatalogue::from_questionnaire_json(&text)
        .unwrap_or_else(|e| panic!("ePCR forms in {}: {e}", source.display()));
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("epcr-forms.json");
    std::fs::write(out, serde_json::to_string(&catalogue).expect("serialise"))
        .expect("write the ePCR forms");
}
