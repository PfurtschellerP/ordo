//! Typed token values and their CSS form.

use serde_json::Value as Json;

use crate::color::{Srgb, trim};
use crate::dtcg::TokenType;

/// Unit of a dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DimensionUnit {
    /// CSS pixels.
    Px,
    /// Multiples of the root font size; grows with the user's text size.
    Rem,
}

/// Unit of a duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DurationUnit {
    /// Milliseconds.
    Ms,
    /// Seconds.
    S,
}

/// A resolved, typed token value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A colour.
    Color(Srgb),
    /// A length.
    Dimension {
        /// Amount.
        value: f64,
        /// Unit.
        unit: DimensionUnit,
    },
    /// Font family names, most preferred first.
    FontFamily(Vec<String>),
    /// Font weight, 1–1000.
    FontWeight(f64),
    /// A duration.
    Duration {
        /// Amount.
        value: f64,
        /// Unit.
        unit: DurationUnit,
    },
    /// A plain number.
    Number(f64),
}

impl Value {
    /// Interprets a literal `$value` as `ty`.
    pub fn parse(ty: TokenType, json: &Json) -> Result<Self, String> {
        match ty {
            TokenType::Color => parse_color(json).map(Value::Color),
            TokenType::Dimension => {
                let (value, unit) = amount_and_unit(json, "dimension")?;
                let unit = match unit {
                    "px" => DimensionUnit::Px,
                    "rem" => DimensionUnit::Rem,
                    other => return Err(format!("dimension unit {other:?} must be px or rem")),
                };
                Ok(Value::Dimension { value, unit })
            }
            TokenType::Duration => {
                let (value, unit) = amount_and_unit(json, "duration")?;
                let unit = match unit {
                    "ms" => DurationUnit::Ms,
                    "s" => DurationUnit::S,
                    other => return Err(format!("duration unit {other:?} must be ms or s")),
                };
                if value < 0.0 {
                    return Err("durations must not be negative".into());
                }
                Ok(Value::Duration { value, unit })
            }
            TokenType::FontFamily => match json {
                Json::String(name) => Ok(Value::FontFamily(vec![name.clone()])),
                Json::Array(items) if !items.is_empty() => items
                    .iter()
                    .map(|item| {
                        item.as_str()
                            .map(str::to_owned)
                            .ok_or_else(|| "font families must be strings".to_owned())
                    })
                    .collect::<Result<_, _>>()
                    .map(Value::FontFamily),
                _ => Err("a font family must be a string or a non-empty list of strings".into()),
            },
            TokenType::FontWeight => {
                let weight = match json {
                    Json::Number(n) => n.as_f64().unwrap_or(f64::NAN),
                    Json::String(keyword) => font_weight_keyword(keyword)
                        .ok_or_else(|| format!("unknown font weight {keyword:?}"))?,
                    _ => return Err("a font weight must be a number or keyword".into()),
                };
                if !(1.0..=1000.0).contains(&weight) {
                    return Err("font weights must be between 1 and 1000".into());
                }
                Ok(Value::FontWeight(weight))
            }
            TokenType::Number => json
                .as_f64()
                .map(Value::Number)
                .ok_or_else(|| "expected a number".into()),
        }
    }

    /// The value in CSS syntax.
    pub fn to_css(&self) -> String {
        match self {
            Value::Color(color) => color.to_css(),
            Value::Dimension { value, unit } => {
                let unit = match unit {
                    DimensionUnit::Px => "px",
                    DimensionUnit::Rem => "rem",
                };
                format!("{}{unit}", trim(*value))
            }
            Value::Duration { value, unit } => {
                let unit = match unit {
                    DurationUnit::Ms => "ms",
                    DurationUnit::S => "s",
                };
                format!("{}{unit}", trim(*value))
            }
            Value::FontFamily(names) => names
                .iter()
                .map(|name| css_font_name(name))
                .collect::<Vec<_>>()
                .join(", "),
            Value::FontWeight(weight) => trim(*weight),
            Value::Number(number) => trim(*number),
        }
    }
}

fn parse_color(json: &Json) -> Result<Srgb, String> {
    let object = json
        .as_object()
        .ok_or("a colour must be an object with colorSpace and components")?;
    match object.get("colorSpace").and_then(Json::as_str) {
        Some("srgb") => {}
        Some(other) => {
            return Err(format!(
                "colour space {other:?} is not supported yet (use srgb)"
            ));
        }
        None => return Err("colorSpace is missing".into()),
    }
    let components = object
        .get("components")
        .and_then(Json::as_array)
        .filter(|c| c.len() == 3)
        .ok_or("components must be a list of three numbers")?;
    let mut channels = [0.0; 3];
    for (channel, component) in channels.iter_mut().zip(components) {
        *channel = match component {
            Json::String(none) if none == "none" => 0.0,
            other => other.as_f64().ok_or("components must be numbers")?,
        };
        if !(0.0..=1.0).contains(channel) {
            return Err("sRGB components must be between 0 and 1".into());
        }
    }
    let alpha = match object.get("alpha") {
        None => 1.0,
        Some(value) => value
            .as_f64()
            .filter(|a| (0.0..=1.0).contains(a))
            .ok_or("alpha must be between 0 and 1")?,
    };
    let color = Srgb {
        r: channels[0],
        g: channels[1],
        b: channels[2],
        alpha,
    };
    if let Some(hex) = object.get("hex") {
        let hex = hex.as_str().ok_or("hex must be a string")?;
        let parsed =
            Srgb::from_hex(hex).ok_or_else(|| format!("hex {hex:?} must have the form #rrggbb"))?;
        if parsed.to_bytes() != color.to_bytes() {
            return Err(format!(
                "hex {hex} does not match the components (which give {})",
                color.to_hex()
            ));
        }
    }
    Ok(color)
}

fn amount_and_unit<'a>(json: &'a Json, what: &str) -> Result<(f64, &'a str), String> {
    let object = json
        .as_object()
        .ok_or_else(|| format!("a {what} must be an object with value and unit"))?;
    let value = object
        .get("value")
        .and_then(Json::as_f64)
        .ok_or_else(|| format!("the {what} value must be a number"))?;
    let unit = object
        .get("unit")
        .and_then(Json::as_str)
        .ok_or_else(|| format!("the {what} unit must be a string"))?;
    Ok((value, unit))
}

fn font_weight_keyword(keyword: &str) -> Option<f64> {
    Some(match keyword {
        "thin" | "hairline" => 100.0,
        "extra-light" | "ultra-light" => 200.0,
        "light" => 300.0,
        "normal" | "regular" | "book" => 400.0,
        "medium" => 500.0,
        "semi-bold" | "demi-bold" => 600.0,
        "bold" => 700.0,
        "extra-bold" | "ultra-bold" => 800.0,
        "black" | "heavy" => 900.0,
        "extra-black" | "ultra-black" => 950.0,
        _ => return None,
    })
}

fn css_font_name(name: &str) -> String {
    const GENERIC: &[&str] = &[
        "serif",
        "sans-serif",
        "monospace",
        "cursive",
        "fantasy",
        "system-ui",
        "ui-serif",
        "ui-sans-serif",
        "ui-monospace",
        "ui-rounded",
        "math",
        "emoji",
        "fangsong",
    ];
    if GENERIC.contains(&name) {
        name.to_owned()
    } else {
        format!("\"{}\"", name.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn colours() {
        let value =
            json!({"colorSpace": "srgb", "components": [0.1137, 0.3608, 0.7529], "hex": "#1D5CC0"});
        assert_eq!(
            Value::parse(TokenType::Color, &value).unwrap().to_css(),
            "#1d5cc0"
        );
        let mismatch = json!({"colorSpace": "srgb", "components": [1, 1, 1], "hex": "#000000"});
        assert!(
            Value::parse(TokenType::Color, &mismatch)
                .unwrap_err()
                .contains("does not match")
        );
        let p3 = json!({"colorSpace": "display-p3", "components": [1, 0, 0]});
        assert!(
            Value::parse(TokenType::Color, &p3)
                .unwrap_err()
                .contains("not supported")
        );
    }

    #[test]
    fn other_types_to_css() {
        let cases = [
            (
                TokenType::Dimension,
                json!({"value": 0.25, "unit": "rem"}),
                "0.25rem",
            ),
            (
                TokenType::Dimension,
                json!({"value": 3, "unit": "px"}),
                "3px",
            ),
            (
                TokenType::Duration,
                json!({"value": 200, "unit": "ms"}),
                "200ms",
            ),
            (TokenType::FontWeight, json!("bold"), "700"),
            (TokenType::Number, json!(1.5), "1.5"),
            (
                TokenType::FontFamily,
                json!(["Atkinson Hyperlegible Next", "system-ui", "sans-serif"]),
                "\"Atkinson Hyperlegible Next\", system-ui, sans-serif",
            ),
        ];
        for (ty, json, css) in cases {
            assert_eq!(Value::parse(ty, &json).unwrap().to_css(), css);
        }
    }

    #[test]
    fn invalid_values() {
        assert!(Value::parse(TokenType::Dimension, &json!({"value": 1, "unit": "em"})).is_err());
        assert!(Value::parse(TokenType::Duration, &json!({"value": -1, "unit": "ms"})).is_err());
        assert!(Value::parse(TokenType::FontWeight, &json!(1200)).is_err());
        assert!(Value::parse(TokenType::FontFamily, &json!([])).is_err());
    }
}
