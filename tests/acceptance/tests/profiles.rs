//! Regional profiles are data (I18N-4): their manifests must be consistent, so the
//! resolver can layer them (architecture 8.8.3).

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    id: String,
    name: String,
    version: String,
    format: u32,
    extends: Vec<String>,
    languages: Vec<String>,
    #[serde(default)]
    source: Vec<Source>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    topic: String,
    title: String,
    #[serde(default)]
    edition: Option<String>,
    url: String,
    #[serde(default)]
    note: Option<String>,
}

/// Every profile directory has a valid manifest named like the directory; every profile
/// it extends exists; every source names a topic and an HTTPS address.
#[test]
fn profile_manifests_are_consistent() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles");
    let mut profiles = BTreeMap::new();
    for entry in std::fs::read_dir(&root).expect("profiles directory") {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let name = dir.file_name().unwrap().to_string_lossy().into_owned();
        let text = std::fs::read_to_string(dir.join("profile.toml"))
            .unwrap_or_else(|e| panic!("{name}: profile.toml: {e}"));
        let manifest: Manifest =
            toml::from_str(&text).unwrap_or_else(|e| panic!("{name}/profile.toml: {e}"));
        assert_eq!(manifest.id, name, "the ID is the directory name");
        assert!(!manifest.name.is_empty() && !manifest.version.is_empty());
        assert_eq!(manifest.format, 1, "{name}: profile format");
        assert!(!manifest.languages.is_empty(), "{name}: languages");
        for source in &manifest.source {
            assert!(
                !source.topic.is_empty() && !source.title.is_empty(),
                "{name}: source without topic or title"
            );
            assert!(
                source.url.starts_with("https://"),
                "{name}: {} must link over HTTPS",
                source.url
            );
            let _ = (&source.edition, &source.note);
        }
        profiles.insert(name, manifest);
    }
    assert!(profiles.contains_key("de"));
    for (name, manifest) in &profiles {
        for parent in &manifest.extends {
            assert!(
                profiles.contains_key(parent),
                "{name} extends {parent}, which does not exist"
            );
        }
    }
    let states = profiles.values().filter(|m| m.extends == ["de"]).count();
    assert_eq!(states, 16, "one profile per German state");
}
