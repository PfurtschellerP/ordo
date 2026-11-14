//! Tactical symbols (MAP-3): which symbol of the profile a map object gets. The profile
//! describes its symbols and the rules in `symbols/symbols.json` and keeps one SVG file per
//! symbol next to it; the platform only knows the rules' shape, never a region's symbols.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A symbol of the profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Symbol {
    /// Stable ID, also the name of its SVG file (`<id>.svg`).
    pub id: String,
    /// What it stands for, e.g. "Rettungswagen".
    pub label: String,
    /// How the profile's generator draws it (parameters of the regional standard).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub spec: BTreeMap<String, String>,
    /// The topic it is offered under when placed on the map (MAP-10); none: not offered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// A topic of the symbols offered for placing (MAP-10), e.g. "Personen".
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolGroup {
    /// Stable ID, e.g. `persons`.
    pub id: String,
    /// What it is called, e.g. "Personen".
    pub label: String,
}

/// Which symbol a unit gets: by its type first, else by its service.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitRules {
    /// Unit type (e.g. `RTW`, compared without regard to case) → symbol ID.
    #[serde(default)]
    pub by_type: BTreeMap<String, String>,
    /// Service ID of the unit catalogue (e.g. `rescue`) → symbol ID.
    #[serde(default)]
    pub by_service: BTreeMap<String, String>,
}

/// The symbols of a profile and which map object gets which.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SymbolCatalogue {
    /// The symbols.
    #[serde(default)]
    pub symbols: Vec<Symbol>,
    /// The topics symbols are offered under, in menu order (MAP-10).
    #[serde(default)]
    pub groups: Vec<SymbolGroup>,
    /// Units.
    #[serde(default)]
    pub units: UnitRules,
    /// UHS kind (e.g. `treatmentArea`) → symbol ID.
    #[serde(default)]
    pub uhs: BTreeMap<String, String>,
    /// Position kind of the command structure (e.g. `sector`) → symbol ID.
    #[serde(default)]
    pub positions: BTreeMap<String, String>,
    /// Report kind (`event`, `patient`) → symbol ID (REP-4).
    #[serde(default)]
    pub reports: BTreeMap<String, String>,
    /// The incident location's symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incident: Option<String>,
}

impl SymbolCatalogue {
    /// The symbol with this ID.
    pub fn symbol(&self, id: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|s| s.id == id)
    }

    // [impl->dsn~tactical-symbols~2]
    /// A unit's symbol: by its type (case does not matter), else by its service, else none.
    pub fn for_unit(&self, unit_type: &str, service: Option<&str>) -> Option<&Symbol> {
        let unit_type = unit_type.trim();
        self.units
            .by_type
            .iter()
            .find(|(t, _)| t.eq_ignore_ascii_case(unit_type))
            .map(|(_, id)| id)
            .or_else(|| service.and_then(|s| self.units.by_service.get(s)))
            .and_then(|id| self.symbol(id))
    }

    /// A UHS's symbol by its kind.
    pub fn for_uhs(&self, kind: &str) -> Option<&Symbol> {
        self.uhs.get(kind).and_then(|id| self.symbol(id))
    }

    /// A position's symbol by its kind.
    pub fn for_position(&self, kind: &str) -> Option<&Symbol> {
        self.positions.get(kind).and_then(|id| self.symbol(id))
    }

    /// A report's symbol by its kind (`event`, `patient`).
    pub fn for_report(&self, kind: &str) -> Option<&Symbol> {
        self.reports.get(kind).and_then(|id| self.symbol(id))
    }

    // [impl->dsn~tactical-symbols~2]
    /// The symbols offered for placing, by topic in menu order (MAP-10).
    pub fn by_group(&self) -> Vec<(&SymbolGroup, Vec<&Symbol>)> {
        self.groups
            .iter()
            .map(|g| {
                let list = self
                    .symbols
                    .iter()
                    .filter(|s| s.group.as_deref() == Some(g.id.as_str()))
                    .collect();
                (g, list)
            })
            .filter(|(_, list): &(_, Vec<&Symbol>)| !list.is_empty())
            .collect()
    }

    /// The incident location's symbol.
    pub fn for_incident(&self) -> Option<&Symbol> {
        self.incident.as_deref().and_then(|id| self.symbol(id))
    }

    /// Checks that IDs are unique and safe as file names, labels are not empty, and every
    /// rule names a symbol of the catalogue.
    ///
    /// # Errors
    ///
    /// Describes the first problem found.
    pub fn check(&self) -> Result<(), String> {
        let mut seen = std::collections::BTreeSet::new();
        for s in &self.symbols {
            let safe = !s.id.is_empty()
                && s.id
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
            if !safe {
                return Err(format!("symbol ID {:?}: only a-z, 0-9 and -", s.id));
            }
            if !seen.insert(s.id.as_str()) {
                return Err(format!("symbol {} twice", s.id));
            }
            if s.label.trim().is_empty() {
                return Err(format!("symbol {} has no label", s.id));
            }
        }
        let mut groups = std::collections::BTreeSet::new();
        for g in &self.groups {
            if g.label.trim().is_empty() || !groups.insert(g.id.as_str()) {
                return Err(format!("group {:?} twice or without a label", g.id));
            }
        }
        if let Some(s) = self
            .symbols
            .iter()
            .find(|s| s.group.as_deref().is_some_and(|g| !groups.contains(g)))
        {
            return Err(format!("symbol {}: no group {:?}", s.id, s.group));
        }
        let rules = self
            .units
            .by_type
            .iter()
            .chain(&self.units.by_service)
            .chain(&self.uhs)
            .chain(&self.positions)
            .chain(&self.reports)
            .map(|(from, id)| (from.as_str(), id.as_str()))
            .chain(self.incident.as_deref().map(|id| ("incident", id)));
        for (from, id) in rules {
            if !seen.contains(id) {
                return Err(format!("{from}: no symbol {id}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> SymbolCatalogue {
        serde_json::from_str(
            r#"{
              "symbols": [
                { "id": "incident", "label": "Einsatzstelle", "spec": { "grundzeichen": "anlass" }, "group": "scene" },
                { "id": "rtw", "label": "Rettungswagen", "group": "units" },
                { "id": "rescue-vehicle", "label": "Fahrzeug Rettungsdienst" },
                { "id": "treatment-area", "label": "Behandlungsplatz" },
                { "id": "sector-command", "label": "Einsatzabschnittsleitung" }
              ],
              "groups": [
                { "id": "units", "label": "Einsatzmittel" },
                { "id": "scene", "label": "Einsatzstelle" },
                { "id": "persons", "label": "Personen" }
              ],
              "units": { "by_type": { "RTW": "rtw" }, "by_service": { "rescue": "rescue-vehicle" } },
              "uhs": { "treatmentArea": "treatment-area" },
              "positions": { "sector": "sector-command", "sub-sector": "sector-command" },
              "incident": "incident"
            }"#,
        )
        .expect("valid catalogue")
    }

    fn id(s: Option<&Symbol>) -> Option<&str> {
        s.map(|s| s.id.as_str())
    }

    // [utest->dsn~tactical-symbols~2]
    #[test]
    fn objects_get_the_profiles_symbols() {
        let c = catalogue();
        assert_eq!(c.check(), Ok(()));
        // The type wins, whatever its case; else the service; else no symbol.
        assert_eq!(id(c.for_unit("rtw", Some("fire"))), Some("rtw"));
        assert_eq!(id(c.for_unit(" RTW ", None)), Some("rtw"));
        assert_eq!(
            id(c.for_unit("GW-San", Some("rescue"))),
            Some("rescue-vehicle")
        );
        assert_eq!(id(c.for_unit("GW-San", Some("police"))), None);
        assert_eq!(id(c.for_unit("GW-San", None)), None);
        assert_eq!(id(c.for_uhs("treatmentArea")), Some("treatment-area"));
        assert_eq!(id(c.for_uhs("uhs")), None);
        assert_eq!(id(c.for_position("sub-sector")), Some("sector-command"));
        assert_eq!(id(c.for_position("s1")), None);
        assert_eq!(id(c.for_incident()), Some("incident"));
        assert_eq!(
            c.symbol("incident")
                .and_then(|s| s.spec.get("grundzeichen")),
            Some(&"anlass".to_owned())
        );
    }

    // [utest->dsn~tactical-symbols~2]
    #[test]
    fn symbols_are_offered_by_topic_in_menu_order() {
        let c = catalogue();
        let menu: Vec<(&str, Vec<&str>)> = c
            .by_group()
            .into_iter()
            .map(|(g, list)| {
                (
                    g.label.as_str(),
                    list.iter().map(|s| s.id.as_str()).collect(),
                )
            })
            .collect();
        assert_eq!(
            menu,
            [
                ("Einsatzmittel", vec!["rtw"]),
                ("Einsatzstelle", vec!["incident"])
            ],
            "a topic without symbols is left out; symbols without a topic are not offered"
        );
        let mut c = catalogue();
        c.symbols[1].group = Some("nowhere".into());
        assert!(c.check().is_err(), "a symbol's topic must exist");
    }

    // [utest->dsn~tactical-symbols~2]
    #[test]
    fn check_finds_broken_catalogues() {
        let mut c = catalogue();
        c.positions.insert("s1".to_owned(), "missing".to_owned());
        assert_eq!(c.check(), Err("s1: no symbol missing".to_owned()));

        let mut c = catalogue();
        c.symbols[1].id = "../rtw".to_owned();
        assert!(c.check().is_err());

        let mut c = catalogue();
        c.symbols.push(c.symbols[0].clone());
        assert_eq!(c.check(), Err("symbol incident twice".to_owned()));

        let mut c = catalogue();
        c.symbols[0].label = " ".to_owned();
        assert!(c.check().is_err());
    }
}
