//! The app's texts at run time (ADR-0096): Fluent files per part of the app (`locales/<language>/<part>.ftl`),
//! read with Project Fluent's `fluent-bundle`. The build writes one function per message
//! into each part's `text` module (`build/fluent.rs`); they call [`message`] or [`format`].
//! The language is chosen once at start ([`choose`]); a message missing in it comes from
//! German.

use std::cell::RefCell;
use std::collections::HashMap;

use fluent_bundle::{FluentBundle, FluentResource, FluentValue};

include!(concat!(env!("OUT_DIR"), "/fluent_sources.rs"));

/// The language every text exists in, and the one used when no other fits.
pub const DEFAULT: &str = "de";

/// The arguments of a message.
pub type Args = fluent_bundle::FluentArgs<'static>;

/// A value for a message's variable: text, or a number (which selects plural forms).
pub trait Arg {
    fn value(self) -> FluentValue<'static>;
}

impl Arg for &str {
    fn value(self) -> FluentValue<'static> {
        FluentValue::from(self.to_owned())
    }
}

impl Arg for String {
    fn value(self) -> FluentValue<'static> {
        FluentValue::from(self)
    }
}

impl Arg for &String {
    fn value(self) -> FluentValue<'static> {
        FluentValue::from(self.clone())
    }
}

macro_rules! number_arg {
    ($($t:ty),*) => {$(
        impl Arg for $t {
            fn value(self) -> FluentValue<'static> {
                FluentValue::from(self)
            }
        }
    )*};
}
number_arg!(u8, u16, u32, u64, usize, i8, i16, i32, i64, isize, f32, f64);

thread_local! {
    static LANGUAGE: RefCell<&'static str> = const { RefCell::new(DEFAULT) };
    /// (language, part) -> its bundle, made when first needed.
    static BUNDLES: RefCell<HashMap<(&'static str, &'static str), FluentBundle<FluentResource>>> =
        RefCell::new(HashMap::new());
    /// (language, part, message) -> the text of a message without variables, kept for the
    /// page's life so it can be a `&'static str`.
    static KEPT: RefCell<HashMap<(&'static str, &'static str, &'static str), &'static str>> =
        RefCell::new(HashMap::new());
}

/// The languages there are texts for.
pub fn languages() -> Vec<&'static str> {
    let mut languages: Vec<&'static str> = SOURCES.iter().map(|(l, _, _)| *l).collect();
    languages.sort_unstable();
    languages.dedup();
    languages
}

/// Chooses the first of the `preferred` languages (BCP 47 tags, e.g. the browser's) that
/// has texts, by its primary subtag; German if none has. Call once, before showing text.
pub fn choose(preferred: &[String]) -> &'static str {
    let available = languages();
    let chosen = preferred
        .iter()
        .filter_map(|tag| tag.split(['-', '_']).next())
        .find_map(|primary| {
            available
                .iter()
                .find(|a| a.eq_ignore_ascii_case(primary))
                .copied()
        })
        .unwrap_or(DEFAULT);
    LANGUAGE.with(|l| *l.borrow_mut() = chosen);
    chosen
}

/// The chosen language.
pub fn language() -> &'static str {
    LANGUAGE.with(|l| *l.borrow())
}

fn source(language: &str, part: &str) -> Option<&'static str> {
    SOURCES
        .iter()
        .find(|(l, p, _)| *l == language && *p == part)
        .map(|(_, _, s)| *s)
}

/// Runs `f` with the bundle of `part` in `language`, made the first time.
fn with_bundle<R>(
    language: &'static str,
    part: &'static str,
    f: impl FnOnce(Option<&FluentBundle<FluentResource>>) -> R,
) -> R {
    BUNDLES.with(|bundles| {
        let mut bundles = bundles.borrow_mut();
        if !bundles.contains_key(&(language, part))
            && let Some(text) = source(language, part)
        {
            let tag = language.parse().unwrap_or_default();
            let mut bundle = FluentBundle::new(vec![tag]);
            // No Unicode isolation marks around variables: no right-to-left language yet,
            // and the marks would show up in copied text and tests.
            bundle.set_use_isolating(false);
            let resource =
                FluentResource::try_new(text.to_owned()).unwrap_or_else(|(resource, _)| resource);
            let _ = bundle.add_resource(resource);
            bundles.insert((language, part), bundle);
        }
        f(bundles.get(&(language, part)))
    })
}

/// Formats message `id` of `part` in the chosen language, else in German; the ID itself
/// if neither has it (the build makes sure German has).
fn formatted(part: &'static str, id: &'static str, args: Option<&Args>) -> String {
    let try_in = |language: &'static str| {
        with_bundle(language, part, |bundle| {
            let message = bundle?.get_message(id)?;
            let pattern = message.value()?;
            let mut errors = Vec::new();
            let text = bundle?.format_pattern(pattern, args, &mut errors);
            Some(text.into_owned())
        })
    };
    try_in(language())
        .or_else(|| try_in(DEFAULT))
        .unwrap_or_else(|| id.to_owned())
}

/// Where the device keeps the language chosen in the menu; none follows the browser.
pub const PREFERENCE: &str = "ordo.language";

/// The name of `language` in itself ("Deutsch", "English"), so each person finds their own.
pub fn name_of(language: &'static str) -> String {
    with_bundle(language, "shell", |bundle| {
        let bundle = bundle?;
        let pattern = bundle.get_message("language-name")?.value()?;
        let mut errors = Vec::new();
        Some(
            bundle
                .format_pattern(pattern, None, &mut errors)
                .into_owned(),
        )
    })
    .unwrap_or_else(|| language.to_owned())
}

/// The text of a message without variables.
pub fn message(part: &'static str, id: &'static str) -> &'static str {
    let language = language();
    KEPT.with(|kept| {
        *kept
            .borrow_mut()
            .entry((language, part, id))
            .or_insert_with(|| Box::leak(formatted(part, id, None).into_boxed_str()))
    })
}

/// The text of a message with its variables.
pub fn format(part: &'static str, id: &'static str, args: &Args) -> String {
    formatted(part, id, Some(args))
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_syntax::ast;
    use std::collections::BTreeMap;

    /// Message ID -> its variables, sorted.
    fn messages(source: &str) -> BTreeMap<String, Vec<String>> {
        let resource = fluent_syntax::parser::parse(source).expect("checked by the build");
        resource
            .body
            .iter()
            .filter_map(|entry| match entry {
                ast::Entry::Message(m) => Some(m),
                _ => None,
            })
            .map(|m| {
                let mut variables = Vec::new();
                if let Some(p) = &m.value {
                    collect(p, &mut variables);
                }
                variables.sort();
                variables.dedup();
                (m.id.name.to_owned(), variables)
            })
            .collect()
    }

    fn collect(pattern: &ast::Pattern<&str>, found: &mut Vec<String>) {
        for element in &pattern.elements {
            if let ast::PatternElement::Placeable { expression } = element {
                expression_of(expression, found);
            }
        }
    }

    fn expression_of(expression: &ast::Expression<&str>, found: &mut Vec<String>) {
        match expression {
            ast::Expression::Inline(i) => inline_of(i, found),
            ast::Expression::Select { selector, variants } => {
                inline_of(selector, found);
                for v in variants {
                    collect(&v.value, found);
                }
            }
        }
    }

    fn inline_of(inline: &ast::InlineExpression<&str>, found: &mut Vec<String>) {
        match inline {
            ast::InlineExpression::VariableReference { id } => found.push(id.name.to_owned()),
            ast::InlineExpression::FunctionReference { arguments, .. } => {
                for a in &arguments.positional {
                    inline_of(a, found);
                }
                for n in &arguments.named {
                    inline_of(&n.value, found);
                }
            }
            ast::InlineExpression::Placeable { expression } => expression_of(expression, found),
            _ => {}
        }
    }

    // [utest->dsn~ui-texts~2]
    #[test]
    fn every_language_has_the_german_messages_with_the_same_variables() {
        for (language, part, source) in SOURCES.iter().filter(|(l, _, _)| *l != DEFAULT) {
            let german = messages(super::source(DEFAULT, part).expect("German has every part"));
            let other = messages(source);
            assert_eq!(
                german.keys().collect::<Vec<_>>(),
                other.keys().collect::<Vec<_>>(),
                "{language}/{part}.ftl: the same messages as German"
            );
            for (id, variables) in &german {
                assert_eq!(&other[id], variables, "{language}/{part}.ftl: {id}");
            }
        }
    }

    // [utest->dsn~ui-texts~2]
    #[test]
    fn every_language_has_every_part() {
        let german: Vec<&str> = SOURCES
            .iter()
            .filter(|(l, _, _)| *l == DEFAULT)
            .map(|(_, p, _)| *p)
            .collect();
        for language in languages() {
            for part in &german {
                assert!(
                    super::source(language, part).is_some(),
                    "{language}/{part}.ftl is missing"
                );
            }
        }
    }

    // [utest->dsn~ui-texts~2]
    #[test]
    fn every_message_formats_without_errors() {
        for (language, part, source) in SOURCES {
            let bundle_language: &'static str = language;
            let part: &'static str = part;
            for (id, variables) in messages(source) {
                let mut args = Args::new();
                for v in &variables {
                    args.set(v.clone(), FluentValue::from(2));
                }
                with_bundle(bundle_language, part, |bundle| {
                    let bundle = bundle.expect("a bundle");
                    let message = bundle.get_message(&id).expect("the message");
                    let mut errors = Vec::new();
                    let text = bundle.format_pattern(
                        message.value().expect("a value"),
                        Some(&args),
                        &mut errors,
                    );
                    assert!(errors.is_empty(), "{language}/{part}.ftl: {id}: {errors:?}");
                    assert!(
                        !text.trim().is_empty(),
                        "{language}/{part}.ftl: {id} is empty"
                    );
                });
            }
        }
    }

    #[test]
    fn the_language_follows_the_browser_else_german() {
        assert_eq!(choose(&["fr-FR".into(), "de-AT".into()]), "fr");
        assert_eq!(choose(&["pt-BR".into(), "es-419".into()]), "es");
        assert_eq!(choose(&["it".into(), "de-AT".into()]), "de");
        assert_eq!(choose(&["xx".into()]), DEFAULT);
        assert_eq!(language(), DEFAULT);
    }
}
