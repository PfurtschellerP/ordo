//! Embeds the profile catalogues the app needs. The TOML files are checked here, so a
//! broken profile fails the build, and handed to the app as JSON (no TOML parser in the
//! WebAssembly bundle).

use std::path::Path;

use ordo_med_patients::record::FormCatalogue;

fn main() {
    let source =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/de/med/epcr-forms.toml");
    println!("cargo:rerun-if-changed={}", source.display());
    let text = std::fs::read_to_string(&source).expect("read the ePCR forms of the profile");
    let catalogue: FormCatalogue = toml::from_str(&text).expect("ePCR forms: valid TOML");
    catalogue.check().expect("ePCR forms: valid catalogue");
    let out = Path::new(&std::env::var("OUT_DIR").expect("OUT_DIR")).join("epcr-forms.json");
    std::fs::write(out, serde_json::to_string(&catalogue).expect("serialise"))
        .expect("write the ePCR forms");
}
