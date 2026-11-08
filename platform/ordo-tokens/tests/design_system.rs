//! Tests for loading, layering, checking and CSS output.

use std::collections::HashMap;
use std::path::Path;

use ordo_tokens::{DesignSystem, Error, css};

const MANIFEST: &str = r#"
prefix = "t"
base = ["base.json"]
protected = ["color.safety"]
default_theme = "light"

[[theme]]
name = "light"
file = "light.json"

[[theme]]
name = "dark"
file = "dark.json"
media = "(prefers-color-scheme: dark)"
"#;

const CONTRAST: &str = r#"
[[pair]]
fg = "color.text"
bg = "color.surface"
min = 4.5
criterion = "1.4.3"
purpose = "Text"

[[pair]]
fg = "color.safety.onAlarm"
bg = "color.safety.alarm"
min = 4.5
criterion = "1.4.3"
purpose = "Alarm"
"#;

fn rgb(hex: &str) -> String {
    let h = hex.trim_start_matches('#');
    let c = |i: usize| f64::from(u8::from_str_radix(&h[i..i + 2], 16).unwrap()) / 255.0;
    format!(
        r#"{{"colorSpace": "srgb", "components": [{:.4}, {:.4}, {:.4}]}}"#,
        c(0),
        c(2),
        c(4)
    )
}

fn base() -> String {
    format!(
        r#"{{
          "color": {{
            "$type": "color",
            "white": {{"$value": {}}},
            "black": {{"$value": {}}},
            "red": {{"$value": {}}},
            "grey": {{"$value": {}}}
          }},
          "motion": {{"fast": {{"$type": "duration", "$value": {{"value": 100, "unit": "ms"}}}}}},
          "component": {{"page": {{"background": {{"$value": "{{color.surface}}"}}}}}}
        }}"#,
        rgb("#ffffff"),
        rgb("#000000"),
        rgb("#c42b26"),
        rgb("#999999")
    )
}

fn theme(surface: &str, text: &str) -> String {
    format!(
        r#"{{"color": {{"$type": "color",
            "surface": {{"$value": "{{color.{surface}}}"}},
            "text": {{"$value": "{{color.{text}}}"}},
            "safety": {{"alarm": {{"$value": "{{color.red}}"}}, "onAlarm": {{"$value": "{{color.white}}"}}}}
        }}}}"#
    )
}

fn load(files: &[(&str, String)]) -> Result<DesignSystem, Error> {
    let files: HashMap<&str, String> = files.iter().cloned().collect();
    DesignSystem::load_with(MANIFEST, CONTRAST, |name| {
        files.get(name).cloned().ok_or_else(|| Error::Parse {
            file: name.to_owned(),
            message: "missing in test".to_owned(),
        })
    })
}

fn valid() -> DesignSystem {
    load(&[
        ("base.json", base()),
        ("light.json", theme("white", "black")),
        ("dark.json", theme("black", "white")),
    ])
    .unwrap()
}

#[test]
fn valid_system_passes() {
    let report = valid().check().unwrap();
    assert!(report.passes());
    assert_eq!(report.themes.len(), 2);
    assert!((report.themes[0].results[0].ratio - 21.0).abs() < 1e-6);
}

// [utest->dsn~theme-validation~1]
#[test]
fn low_contrast_is_reported_not_hidden() {
    let system = load(&[
        ("base.json", base()),
        ("light.json", theme("white", "grey")),
        ("dark.json", theme("black", "white")),
    ])
    .unwrap();
    let report = system.check().unwrap();
    assert!(!report.passes());
    let failures: Vec<_> = report.themes[0].failures().collect();
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].pair.purpose, "Text");
    assert!(report.themes[1].passes());
}

/// (description, files, expected part of the error message)
type ErrorCase = (&'static str, Vec<(&'static str, String)>, &'static str);

// [utest->dsn~design-token-layers~1]
#[test]
fn structural_errors() {
    let cases: Vec<ErrorCase> = vec![
        (
            "theme overrides a base token",
            vec![
                ("base.json", base()),
                (
                    "light.json",
                    theme("white", "black").replacen(
                        r#""color": {"$type": "color","#,
                        &format!(
                            r#""color": {{"$type": "color", "white": {{"$value": {}}},"#,
                            rgb("#eeeeee")
                        ),
                        1,
                    ),
                ),
                ("dark.json", theme("black", "white")),
            ],
            "themes may only add semantic tokens",
        ),
        (
            "theme misses a token",
            vec![
                ("base.json", base()),
                ("light.json", theme("white", "black")),
                (
                    "dark.json",
                    r#"{"color": {"$type": "color", "surface": {"$value": "{color.black}"}}}"#
                        .into(),
                ),
            ],
            "is missing",
        ),
        (
            "alias to nothing",
            vec![
                ("base.json", base()),
                ("light.json", theme("white", "purple")),
                ("dark.json", theme("black", "white")),
            ],
            "which does not exist",
        ),
        (
            "alias cycle",
            vec![
                (
                    "base.json",
                    base().replacen(
                        r#""color": {"#,
                        r#""color": {"a": {"$value": "{color.b}"}, "b": {"$value": "{color.a}"},"#,
                        1,
                    ),
                ),
                ("light.json", theme("white", "black")),
                ("dark.json", theme("black", "white")),
            ],
            "cycle",
        ),
        (
            "type mismatch",
            vec![
                (
                    "base.json",
                    base().replacen(
                        r#""motion": {"#,
                        r#""motion": {"bad": {"$type": "duration", "$value": "{color.red}"},"#,
                        1,
                    ),
                ),
                ("light.json", theme("white", "black")),
                ("dark.json", theme("black", "white")),
            ],
            "has type duration but refers to",
        ),
    ];
    for (name, files, expected) in cases {
        let error = load(&files).expect_err(name).to_string();
        assert!(error.contains(expected), "{name}: {error}");
    }
}

#[test]
fn organisation_theme_can_change_semantic_tokens() {
    let overlay = format!(
        r#"{{"color": {{"$type": "color", "surface": {{"$value": {}}}}}}}"#,
        rgb("#fff8e7")
    );
    let report = valid()
        .check_overlay("light", &overlay, "org.json")
        .unwrap();
    assert!(report.passes());
}

#[test]
fn organisation_theme_cannot_break_contrast() {
    let overlay = format!(
        r#"{{"color": {{"$type": "color", "text": {{"$value": {}}}}}}}"#,
        rgb("#bbbbbb")
    );
    let report = valid()
        .check_overlay("light", &overlay, "org.json")
        .unwrap();
    assert!(!report.passes());
}

// [utest->dsn~theme-validation~1]
#[test]
fn organisation_theme_cannot_touch_safety_tokens() {
    let overlay = format!(
        r#"{{"color": {{"$type": "color", "safety": {{"alarm": {{"$value": {}}}}}}}}}"#,
        rgb("#ff00ff")
    );
    let error = valid()
        .check_overlay("light", &overlay, "org.json")
        .unwrap_err();
    assert!(matches!(error, Error::Protected { .. }), "{error}");
}

#[test]
fn organisation_theme_cannot_add_or_override_base_tokens() {
    let overlay = format!(
        r#"{{"color": {{"$type": "color", "red": {{"$value": {}}}}}}}"#,
        rgb("#ff0000")
    );
    let error = valid()
        .check_overlay("light", &overlay, "org.json")
        .unwrap_err();
    assert!(matches!(error, Error::UnknownOverride { .. }), "{error}");
}

// [utest->dsn~css-themes~1]
#[test]
fn css_output_layers_themes() {
    let css = css::generate(&valid()).unwrap();
    assert!(
        css.contains(":root {\n  --t-color-white: #ffffff;"),
        "{css}"
    );
    assert!(css.contains("--t-component-page-background: var(--t-color-surface);"));
    assert!(
        css.contains(":root[data-theme=\"dark\"] {\n  --t-color-surface: var(--t-color-black);")
    );
    assert!(css.contains("@media (prefers-color-scheme: dark) {\n  :root:not([data-theme]) {"));
    assert!(css.contains("@media (prefers-reduced-motion: reduce) {\n  :root, :root[data-theme] {\n    --t-motion-fast: 0ms;"));
    assert!(
        !css.contains(":root[data-theme=\"light\"]"),
        "default theme lives in :root"
    );
}

#[test]
fn ordo_design_system_passes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../design");
    let system = DesignSystem::load(&dir).unwrap();
    let report = system.check().unwrap();
    for theme in &report.themes {
        let failures: Vec<_> = theme.failures().map(|r| &r.pair.purpose).collect();
        assert!(failures.is_empty(), "{}: {failures:?}", theme.theme);
    }
    assert_eq!(
        system.theme_names().collect::<Vec<_>>(),
        ["day", "night", "high-contrast"]
    );
    // High contrast keeps all text at AAA level (7:1).
    let hc = report
        .themes
        .iter()
        .find(|t| t.theme == "high-contrast")
        .unwrap();
    assert!(
        hc.results
            .iter()
            .filter(|r| r.pair.criterion == "1.4.3")
            .all(|r| r.ratio >= 7.0)
    );

    let css = css::generate(&system).unwrap();
    assert!(css.contains("--ordo-color-safety-triage-immediate-on-fill:"));
    assert!(css.contains(
        "--ordo-font-family-sans: \"Atkinson Hyperlegible Next\", system-ui, sans-serif;"
    ));
}
