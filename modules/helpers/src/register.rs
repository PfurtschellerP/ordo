//! The profile's settings for the helper register (HELP-1, ADR-0076): how phone numbers
//! and postcodes of its country are checked, and the diets offered.

use serde::{Deserialize, Serialize};

/// Settings of the profile (`helpers/register.toml`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegisterSettings {
    /// Country calling code for numbers given without one, e.g. `49`.
    pub calling_code: String,
    /// Pattern of a postcode: `9` stands for a digit, `A` for a letter, anything else for
    /// itself; e.g. `99999`.
    pub postcode: String,
    /// The diets offered, as shown, e.g. "vegetarisch". Others can be typed in.
    #[serde(default)]
    pub diets: Vec<String>,
}

/// Characters people put between the digits of a phone number.
const SEPARATORS: [char; 6] = [' ', '-', '/', '(', ')', '.'];

impl RegisterSettings {
    /// Checks the settings: a calling code of one to three digits, a postcode pattern,
    /// diets that are given once each.
    ///
    /// # Errors
    ///
    /// Describes the first problem found.
    pub fn check(&self) -> Result<(), String> {
        let code = &self.calling_code;
        if code.is_empty() || code.len() > 3 || !code.bytes().all(|b| b.is_ascii_digit()) {
            return Err(format!("calling code {code:?}: one to three digits"));
        }
        if self.postcode.trim().is_empty() {
            return Err("a postcode pattern is needed".into());
        }
        let mut seen = std::collections::BTreeSet::new();
        for diet in &self.diets {
            if diet.trim().is_empty() || !seen.insert(diet.trim().to_lowercase()) {
                return Err(format!("diet {diet:?} is empty or twice"));
            }
        }
        Ok(())
    }

    // [impl->dsn~helper-register~3]
    /// A phone number in the international form of E.164: `+`, the country code and the
    /// number, 7 to 15 digits. Spaces and the usual separators are dropped, `00` becomes
    /// `+`, a leading `0` the profile's calling code, and an `(0)` after a country code
    /// goes. `None` for anything else, e.g. letters or a number without an area code.
    pub fn phone(&self, text: &str) -> Option<String> {
        let text = text.trim().replace("(0)", "");
        let plain: String = text.chars().filter(|c| !SEPARATORS.contains(c)).collect();
        let digits = if let Some(rest) = plain.strip_prefix('+') {
            rest.to_owned()
        } else if let Some(rest) = plain.strip_prefix("00") {
            rest.to_owned()
        } else {
            format!("{}{}", self.calling_code, plain.strip_prefix('0')?)
        };
        let ok = digits.bytes().all(|b| b.is_ascii_digit())
            && !digits.starts_with('0')
            && (7..=15).contains(&digits.len());
        ok.then(|| format!("+{digits}"))
    }

    // [impl->dsn~helper-register~3]
    /// Whether `text` is a postcode of the profile's pattern.
    pub fn postcode_ok(&self, text: &str) -> bool {
        let text: Vec<char> = text.trim().chars().collect();
        let pattern: Vec<char> = self.postcode.chars().collect();
        text.len() == pattern.len()
            && text.iter().zip(&pattern).all(|(c, p)| match p {
                '9' => c.is_ascii_digit(),
                'A' => c.is_alphabetic(),
                p => c.eq_ignore_ascii_case(p),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de() -> RegisterSettings {
        RegisterSettings {
            calling_code: "49".into(),
            postcode: "99999".into(),
            diets: vec!["vegetarisch".into(), "vegan".into()],
        }
    }

    // [utest->dsn~helper-register~3]
    #[test]
    fn phone_numbers_become_international() {
        let s = de();
        for (typed, number) in [
            ("0171 1234567", "+491711234567"),
            ("0171/123 45 67", "+491711234567"),
            ("+49 (0) 171 1234567", "+491711234567"),
            ("0049 171-1234567", "+491711234567"),
            ("+43 664 1234567", "+436641234567"),
            ("(06131) 12345", "+49613112345"),
        ] {
            assert_eq!(s.phone(typed).as_deref(), Some(number), "{typed}");
        }
        for wrong in [
            "1234567",
            "0171 12a4567",
            "+0171 1234567",
            "01",
            "+49 1234567890123456",
            "",
        ] {
            assert_eq!(s.phone(wrong), None, "{wrong}");
        }
    }

    // [utest->dsn~helper-register~3]
    #[test]
    fn postcodes_follow_the_pattern() {
        let s = de();
        assert!(s.postcode_ok("55116"));
        assert!(s.postcode_ok(" 01067 "));
        for wrong in ["5511", "551160", "5511a", "D-55116", ""] {
            assert!(!s.postcode_ok(wrong), "{wrong}");
        }
        let uk = RegisterSettings {
            postcode: "A9 9AA".into(),
            ..de()
        };
        assert!(uk.postcode_ok("W1 1aa"));
    }

    #[test]
    fn settings_are_checked() {
        assert_eq!(de().check(), Ok(()));
        let twice = RegisterSettings {
            diets: vec!["vegan".into(), "Vegan".into()],
            ..de()
        };
        assert!(twice.check().is_err());
        let code = RegisterSettings {
            calling_code: "+49".into(),
            ..de()
        };
        assert!(code.check().is_err());
    }
}
