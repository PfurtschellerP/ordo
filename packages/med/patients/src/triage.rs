//! Triage algorithms as references (PAT-11, ADR-0045): the profile names the algorithms in
//! use with their source; the person chooses the algorithm used and always the category.
//! Nothing here proposes, pre-selects or checks a category.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

/// When an algorithm is used.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    /// Pre-triage (*Vorsichtung*) by the first crews.
    Pretriage,
    /// Triage (*Sichtung*), usually by a physician.
    Triage,
}

/// One algorithm with its official source.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Algorithm {
    /// Stable ID, recorded with each triage, e.g. `prior`.
    pub id: String,
    /// Name, e.g. "PRIOR".
    pub name: String,
    /// When it is used.
    pub stage: Stage,
    /// Long name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Who publishes it.
    pub publisher: String,
    /// Which edition the profile refers to.
    pub edition: String,
    /// Address of the official source.
    pub url: String,
}

/// The algorithms of the profile (`med/triage-algorithms.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriageCatalogue {
    /// Algorithms in the order they are offered.
    #[serde(rename = "algorithm", default)]
    pub algorithms: Vec<Algorithm>,
}

impl TriageCatalogue {
    // [impl->dsn~triage-reference~1]
    /// Checks that IDs are unique and every entry names its source.
    pub fn check(&self) -> Result<(), String> {
        let mut seen = HashSet::new();
        for a in &self.algorithms {
            if !seen.insert(a.id.as_str()) {
                return Err(format!("algorithm {:?} appears twice", a.id));
            }
            if a.name.trim().is_empty()
                || a.publisher.trim().is_empty()
                || !a.url.starts_with("https://")
            {
                return Err(format!(
                    "algorithm {:?} needs a name, a publisher and an https source",
                    a.id
                ));
            }
        }
        Ok(())
    }

    /// The algorithm with this ID, if the profile has it.
    pub fn get(&self, id: &str) -> Option<&Algorithm> {
        self.algorithms.iter().find(|a| a.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~triage-reference~1]
    #[test]
    fn the_profile_names_each_algorithm_with_its_source() {
        let de: TriageCatalogue = toml::from_str(include_str!(
            "../../../../profiles/de/med/triage-algorithms.toml"
        ))
        .expect("valid");
        de.check().expect("the DE profile");
        let prior = de.get("prior").expect("PRIOR");
        assert_eq!(prior.stage, Stage::Pretriage);
        assert!(prior.url.contains("bbk.bund.de"));

        let mut twice = de.clone();
        twice.algorithms.push(prior.clone());
        assert!(twice.check().is_err());
        let mut sourceless = de;
        sourceless.algorithms[0].url = "http://example.org".into();
        assert!(sourceless.check().is_err(), "an https source is required");
    }
}
