//! Registration cards (HELP-5, ADR-0072): reads the text of a card's QR code with a format
//! from the profile (`helpers/registration-cards.toml`). The module knows no card; the
//! profile says which field of a card goes where.

use serde::{Deserialize, Serialize};

/// What the register takes from one field of the card.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Rule {
    /// Text with `{n}` for field n, counted from 1, e.g. `"{2} {1}"`.
    Template(String),
    /// A date (`DD`, `MM`, `YYYY`) or date and time (also `hh`, `mm`) in field `field`.
    Date {
        /// The field, counted from 1.
        field: usize,
        /// Its pattern, e.g. `DD.MM.YYYY - hh:mm`.
        pattern: String,
    },
}

/// A card format of the profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardFormat {
    /// Stable ID, e.g. `meldekarte`.
    pub id: String,
    /// Label shown, e.g. "Meldekarte".
    pub label: String,
    /// Separator between fields, e.g. `;`.
    pub separator: String,
    /// How many fields a card of this format has.
    pub fields: usize,
    /// The helper's name (required).
    pub name: Rule,
    /// Date of birth.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub birth_date: Option<Rule>,
    /// Postal address.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<Rule>,
    /// Phone number.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<Rule>,
    /// Organisation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organisation: Option<Rule>,
    /// Start of duty (date and time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty_start: Option<Rule>,
    /// End of duty (date and time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duty_end: Option<Rule>,
}

/// The card formats of the profile.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CardCatalogue {
    /// Formats, tried in order.
    #[serde(default)]
    pub card: Vec<CardFormat>,
}

/// What a card gives the helper register. Dates are `YYYY-MM-DD`, dates with time local
/// `YYYY-MM-DDThh:mm`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CardFields {
    /// The format that read the card.
    pub card: String,
    /// Name.
    pub name: String,
    /// Date of birth.
    pub birth_date: Option<String>,
    /// Postal address.
    pub address: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// Organisation.
    pub organisation: Option<String>,
    /// Start of duty, local.
    pub duty_start: Option<String>,
    /// End of duty, local.
    pub duty_end: Option<String>,
}

/// The parts of a template: literal text and field numbers.
enum Part<'a> {
    Text(&'a str),
    Field(usize),
}

fn parts(template: &str) -> Result<Vec<Part<'_>>, String> {
    let mut out = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push(Part::Text(&rest[..open]));
        let close = rest[open..]
            .find('}')
            .ok_or_else(|| format!("unclosed {{ in {template:?}"))?;
        let n = rest[open + 1..open + close]
            .parse::<usize>()
            .map_err(|_| format!("not a field number in {template:?}"))?;
        out.push(Part::Field(n));
        rest = &rest[open + close + 1..];
    }
    out.push(Part::Text(rest));
    Ok(out)
}

/// The fields a rule reads.
fn fields_of(rule: &Rule) -> Result<Vec<usize>, String> {
    Ok(match rule {
        Rule::Template(t) => parts(t)?
            .into_iter()
            .filter_map(|p| match p {
                Part::Field(n) => Some(n),
                Part::Text(_) => None,
            })
            .collect(),
        Rule::Date { field, .. } => vec![*field],
    })
}

/// The date in `text` after `pattern`: `YYYY-MM-DD`, or with `hh` and `mm` in the pattern
/// `YYYY-MM-DDThh:mm`. `None` if it does not fit.
fn date(text: &str, pattern: &str) -> Option<String> {
    let (t, p) = (text.trim().as_bytes(), pattern.as_bytes());
    if t.len() != p.len() {
        return None;
    }
    let take = |token: &[u8]| -> Option<String> {
        let at = p.windows(token.len()).position(|w| w == token)?;
        let digits = &t[at..at + token.len()];
        digits
            .iter()
            .all(u8::is_ascii_digit)
            .then(|| String::from_utf8_lossy(digits).into_owned())
    };
    // Everything that is not a token must match as it is.
    let tokens: &[&[u8]] = &[b"YYYY", b"DD", b"MM", b"hh", b"mm"];
    let mut covered = vec![false; p.len()];
    for token in tokens {
        if let Some(at) = p.windows(token.len()).position(|w| w == *token) {
            covered[at..at + token.len()].fill(true);
        }
    }
    if (0..p.len()).any(|i| !covered[i] && t[i] != p[i]) {
        return None;
    }
    let (year, month, day) = (take(b"YYYY")?, take(b"MM")?, take(b"DD")?);
    let in_range =
        |v: &str, lo: u32, hi: u32| v.parse::<u32>().is_ok_and(|n| (lo..=hi).contains(&n));
    if !in_range(&month, 1, 12) || !in_range(&day, 1, 31) {
        return None;
    }
    let date = format!("{year}-{month}-{day}");
    match (take(b"hh"), take(b"mm")) {
        (Some(h), Some(m)) if in_range(&h, 0, 23) && in_range(&m, 0, 59) => {
            Some(format!("{date}T{h}:{m}"))
        }
        (None, None) => Some(date),
        _ => None,
    }
}

/// Trims what is left over between empty fields, e.g. ", " at the ends or twice.
fn tidy(text: &str) -> String {
    let mut out = text.split_whitespace().collect::<Vec<_>>().join(" ");
    while out.contains(", ,") {
        out = out.replace(", ,", ",");
    }
    out.trim_matches(|c: char| c == ',' || c.is_whitespace())
        .to_owned()
}

impl CardFormat {
    /// The value of `rule` on a card with `fields`; `None` if all its fields are empty or a
    /// date does not fit.
    fn value(rule: &Rule, fields: &[&str]) -> Option<String> {
        let field = |n: usize| fields.get(n.wrapping_sub(1)).map_or("", |f| f.trim());
        match rule {
            Rule::Template(t) => {
                let parts = parts(t).ok()?;
                let any = parts
                    .iter()
                    .any(|p| matches!(p, Part::Field(n) if !field(*n).is_empty()));
                any.then(|| {
                    tidy(
                        &parts
                            .iter()
                            .map(|p| match p {
                                Part::Text(s) => *s,
                                Part::Field(n) => field(*n),
                            })
                            .collect::<String>(),
                    )
                })
            }
            Rule::Date { field: n, pattern } => date(field(*n), pattern),
        }
    }

    // [impl->dsn~registration-card~1]
    /// Reads a card's text; `None` if it is not of this format or gives no name.
    pub fn read(&self, text: &str) -> Option<CardFields> {
        let fields: Vec<&str> = text
            .trim_end_matches(['\r', '\n'])
            .split(self.separator.as_str())
            .collect();
        if fields.len() != self.fields {
            return None;
        }
        let get = |rule: &Option<Rule>| rule.as_ref().and_then(|r| Self::value(r, &fields));
        Some(CardFields {
            card: self.id.clone(),
            name: Self::value(&self.name, &fields).filter(|n| !n.is_empty())?,
            birth_date: get(&self.birth_date),
            address: get(&self.address),
            phone: get(&self.phone),
            organisation: get(&self.organisation),
            duty_start: get(&self.duty_start),
            duty_end: get(&self.duty_end),
        })
    }
}

impl CardCatalogue {
    /// Reads a card's text with the first format that fits.
    pub fn read(&self, text: &str) -> Option<CardFields> {
        self.card.iter().find_map(|f| f.read(text))
    }

    /// Checks the catalogue: unique IDs, a separator, fields within the card, patterns
    /// with a day, month and year.
    pub fn check(&self) -> Result<(), String> {
        let mut ids = std::collections::BTreeSet::new();
        for f in &self.card {
            if !ids.insert(&f.id) {
                return Err(format!("card {:?} appears twice", f.id));
            }
            if f.separator.is_empty() || f.fields == 0 {
                return Err(format!(
                    "card {:?}: a separator and fields are needed",
                    f.id
                ));
            }
            let rules = [
                Some(&f.name),
                f.birth_date.as_ref(),
                f.address.as_ref(),
                f.phone.as_ref(),
                f.organisation.as_ref(),
                f.duty_start.as_ref(),
                f.duty_end.as_ref(),
            ];
            for rule in rules.into_iter().flatten() {
                let fields = fields_of(rule).map_err(|e| format!("card {:?}: {e}", f.id))?;
                if fields.is_empty() || fields.iter().any(|n| *n == 0 || *n > f.fields) {
                    return Err(format!(
                        "card {:?}: fields are counted from 1 to {}",
                        f.id, f.fields
                    ));
                }
                if let Rule::Date { pattern, .. } = rule
                    && !["DD", "MM", "YYYY"].iter().all(|t| pattern.contains(t))
                {
                    return Err(format!(
                        "card {:?}: a date pattern needs DD, MM and YYYY",
                        f.id
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalogue() -> CardCatalogue {
        let text = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../profiles/de/helpers/registration-cards.toml"
        ))
        .expect("profile file");
        toml::from_str(&text).expect("valid catalogue")
    }

    // [utest->dsn~registration-card~1]
    #[test]
    fn a_meldekarte_gives_what_the_register_keeps() {
        let c = catalogue();
        c.check().expect("valid");
        // The QR code of the sample card (made-up person).
        let card = "Mustermann;Max;01.01.2001;m;12345;Testendorf;deutsch;Musterstraße 1;\
                    +4915112312285;Testverband;Malteser Testgruppe;Testeinsatz;\
                    Einsatzinformationen;01.01.2027 - 18:00;;Ernährung ohne Einschränkung";
        assert_eq!(
            c.read(card),
            Some(CardFields {
                card: "meldekarte".into(),
                name: "Max Mustermann".into(),
                birth_date: Some("2001-01-01".into()),
                address: Some("Musterstraße 1, 12345 Testendorf".into()),
                phone: Some("+4915112312285".into()),
                organisation: Some("Malteser Testgruppe, Testverband".into()),
                duty_start: Some("2027-01-01T18:00".into()),
                duty_end: None,
            })
        );
        // Empty fields leave no separators behind; a date that does not fit is left out.
        let sparse = "Muster;Erika;1.1.2001;w;;;;Hauptstr. 2;;;Wasserwacht;;;;;";
        let read = c.read(sparse).expect("read");
        assert_eq!(read.address.as_deref(), Some("Hauptstr. 2"));
        assert_eq!(read.organisation.as_deref(), Some("Wasserwacht"));
        assert_eq!(read.birth_date, None);
        assert_eq!(read.phone, None);
        // Other texts are not cards.
        assert_eq!(c.read("https://example.org"), None);
        assert_eq!(c.read(";;;;;;;;;;;;;;;"), None, "no name");
        assert_eq!(c.read(&format!("{card};zu viel")), None);
    }

    #[test]
    fn mistakes_in_the_catalogue_are_found() {
        let mut c = catalogue();
        c.card[0].fields = 10;
        assert!(c.check().is_err(), "a field beyond the card");
        let mut c = catalogue();
        c.card[0].birth_date = Some(Rule::Date {
            field: 3,
            pattern: "MM/YYYY".into(),
        });
        assert!(c.check().is_err(), "a date needs a day");
        let mut c = catalogue();
        c.card.push(c.card[0].clone());
        assert!(c.check().is_err(), "IDs are unique");
        assert_eq!(
            date("31.12.1999 - 23:59", "DD.MM.YYYY - hh:mm").as_deref(),
            Some("1999-12-31T23:59")
        );
        assert_eq!(date("31.13.1999", "DD.MM.YYYY"), None);
        assert_eq!(
            date("31-12-1999", "DD.MM.YYYY"),
            None,
            "the separators must match"
        );
    }
}
