//! Parsing of the W3C Design Tokens format (DTCG 2025.10).
//!
//! A file is a tree of groups and tokens. A token is an object with `$value`; a group is
//! any other object. `$type` on a group applies to all tokens inside it. A `$value` of the
//! form `"{group.token}"` is an alias to another token.

use serde_json::{Map, Value};

use crate::Error;

/// Token types ORDO supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    /// A colour.
    Color,
    /// A length in `px` or `rem`.
    Dimension,
    /// A font family name or list of names.
    FontFamily,
    /// A font weight.
    FontWeight,
    /// A duration in `ms` or `s`.
    Duration,
    /// A plain number.
    Number,
}

impl TokenType {
    fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "color" => Self::Color,
            "dimension" => Self::Dimension,
            "fontFamily" => Self::FontFamily,
            "fontWeight" => Self::FontWeight,
            "duration" => Self::Duration,
            "number" => Self::Number,
            _ => return None,
        })
    }

    /// The DTCG name of the type.
    pub fn name(self) -> &'static str {
        match self {
            Self::Color => "color",
            Self::Dimension => "dimension",
            Self::FontFamily => "fontFamily",
            Self::FontWeight => "fontWeight",
            Self::Duration => "duration",
            Self::Number => "number",
        }
    }
}

/// The `$value` of a token before resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum RawValue {
    /// A reference to another token, by dotted path.
    Alias(String),
    /// A literal value, interpreted according to the token's type.
    Literal(Value),
}

/// One token from a token file.
#[derive(Debug, Clone)]
pub struct Token {
    /// Dotted path, e.g. `color.text.primary`.
    pub path: String,
    /// Explicit or inherited type. Aliases may leave it open.
    pub ty: Option<TokenType>,
    /// The value.
    pub value: RawValue,
    /// Optional description.
    pub description: Option<String>,
    /// The file the token comes from.
    pub source: String,
}

/// Parses one token file. `source` is used in error messages.
pub fn parse(text: &str, source: &str) -> Result<Vec<Token>, Error> {
    let root: Value = serde_json::from_str(text).map_err(|e| Error::Parse {
        file: source.to_owned(),
        message: e.to_string(),
    })?;
    let Value::Object(root) = root else {
        return Err(Error::Parse {
            file: source.to_owned(),
            message: "a token file must contain a JSON object".to_owned(),
        });
    };
    if root.contains_key("$value") {
        return Err(Error::Parse {
            file: source.to_owned(),
            message: "the top level must be a group, not a token".to_owned(),
        });
    }
    let mut tokens = Vec::new();
    walk(&root, &mut Vec::new(), None, source, &mut tokens)?;
    Ok(tokens)
}

fn walk(
    node: &Map<String, Value>,
    path: &mut Vec<String>,
    inherited: Option<TokenType>,
    source: &str,
    tokens: &mut Vec<Token>,
) -> Result<(), Error> {
    let here = path.join(".");
    let ty = match node.get("$type") {
        None => inherited,
        Some(Value::String(name)) => Some(TokenType::parse(name).ok_or_else(|| {
            Error::token(
                &here,
                format!(
                    "type {name:?} is not supported \
                     (supported: color, dimension, fontFamily, fontWeight, duration, number)"
                ),
            )
        })?),
        Some(_) => return Err(Error::token(&here, "$type must be a string")),
    };
    if node.contains_key("$extends") {
        return Err(Error::token(&here, "$extends is not supported yet"));
    }

    if let Some(value) = node.get("$value") {
        if let Some(child) = node.keys().find(|k| !k.starts_with('$')) {
            return Err(Error::token(
                &here,
                format!("a token cannot contain other tokens or groups ({child:?})"),
            ));
        }
        let description = match node.get("$description") {
            None => None,
            Some(Value::String(text)) => Some(text.clone()),
            Some(_) => return Err(Error::token(&here, "$description must be a string")),
        };
        let value = match value {
            Value::String(text)
                if text.len() > 2 && text.starts_with('{') && text.ends_with('}') =>
            {
                let target = &text[1..text.len() - 1];
                if target.split('.').any(|part| check_name(part).is_err()) {
                    return Err(Error::token(&here, format!("invalid alias {text:?}")));
                }
                RawValue::Alias(target.to_owned())
            }
            other => RawValue::Literal(other.clone()),
        };
        tokens.push(Token {
            path: here,
            ty,
            value,
            description,
            source: source.to_owned(),
        });
        return Ok(());
    }

    for (name, child) in node {
        if name.starts_with('$') {
            continue;
        }
        let child_path = if here.is_empty() {
            name.clone()
        } else {
            format!("{here}.{name}")
        };
        check_name(name).map_err(|message| Error::token(&child_path, message))?;
        let Value::Object(child) = child else {
            return Err(Error::token(
                &child_path,
                "groups and tokens must be JSON objects",
            ));
        };
        path.push(name.clone());
        walk(child, path, ty, source, tokens)?;
        path.pop();
    }
    Ok(())
}

fn check_name(name: &str) -> Result<(), &'static str> {
    if name.is_empty() {
        Err("names must not be empty")
    } else if name.starts_with('$') {
        Err("names must not start with '$'")
    } else if name.contains(['.', '{', '}']) {
        Err("names must not contain '.', '{' or '}'")
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_type_is_inherited_and_aliases_are_detected() {
        let tokens = parse(
            r##"{
                "color": {
                    "$type": "color",
                    "blue": { "$value": { "colorSpace": "srgb", "components": [0, 0, 1] } },
                    "link": { "$value": "{color.blue}", "$description": "Links" }
                },
                "space": { "4": { "$type": "dimension", "$value": { "value": 1, "unit": "rem" } } }
            }"##,
            "test.json",
        )
        .unwrap();
        let paths: Vec<_> = tokens.iter().map(|t| t.path.as_str()).collect();
        assert_eq!(paths, ["color.blue", "color.link", "space.4"]);
        assert_eq!(tokens[0].ty, Some(TokenType::Color));
        assert_eq!(tokens[1].value, RawValue::Alias("color.blue".into()));
        assert_eq!(tokens[1].description.as_deref(), Some("Links"));
        assert_eq!(tokens[2].ty, Some(TokenType::Dimension));
    }

    #[test]
    fn invalid_structures_are_rejected() {
        for (json, expected) in [
            (r#"{"a.b": {"$value": 1}}"#, "must not contain"),
            (
                r#"{"a": {"$value": 1, "b": {"$value": 2}}}"#,
                "cannot contain",
            ),
            (
                r#"{"a": {"$type": "gradient", "$value": 1}}"#,
                "not supported",
            ),
            (r#"{"a": 1}"#, "must be JSON objects"),
            (r#"{"a": {"$value": "{b..c}"}}"#, "invalid alias"),
            (r#"[]"#, "JSON object"),
        ] {
            let error = parse(json, "test.json").unwrap_err().to_string();
            assert!(error.contains(expected), "{json}: {error}");
        }
    }
}
