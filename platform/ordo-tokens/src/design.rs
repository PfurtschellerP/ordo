//! The design system: manifest, base tokens, themes and contrast checks.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use serde::Deserialize;

use crate::Error;
use crate::color::{Srgb, contrast_ratio};
use crate::dtcg::{self, RawValue, Token, TokenType};
use crate::value::Value;

/// `design.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Prefix of generated CSS custom properties.
    pub prefix: String,
    /// Theme-independent token files.
    pub base: Vec<String>,
    /// Token groups organisation themes may not override.
    #[serde(default)]
    pub protected: Vec<String>,
    /// Theme used when nothing else is selected.
    pub default_theme: String,
    /// Themes in the order their media queries apply.
    #[serde(rename = "theme")]
    pub themes: Vec<ThemeEntry>,
}

/// One theme in `design.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThemeEntry {
    /// Name, used as `data-theme="name"`.
    pub name: String,
    /// Token file.
    pub file: String,
    /// CSS media query that selects this theme automatically.
    pub media: Option<String>,
}

/// One entry of `contrast.toml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContrastPair {
    /// Foreground colour token.
    pub fg: String,
    /// Background colour token.
    pub bg: String,
    /// Minimum contrast ratio.
    pub min: f64,
    /// WCAG success criterion, e.g. `1.4.3`.
    pub criterion: String,
    /// What the pair is used for.
    pub purpose: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContrastFile {
    #[serde(rename = "pair", default)]
    pairs: Vec<ContrastPair>,
}

/// Result of checking one contrast pair in one theme.
#[derive(Debug, Clone)]
pub struct PairResult {
    /// The pair.
    pub pair: ContrastPair,
    /// Resolved foreground (composited over the background if transparent).
    pub fg: Srgb,
    /// Resolved background.
    pub bg: Srgb,
    /// Contrast ratio.
    pub ratio: f64,
}

impl PairResult {
    /// Whether the minimum is met (no rounding up).
    pub fn passes(&self) -> bool {
        self.ratio + 1e-9 >= self.pair.min
    }
}

/// Contrast results for one theme.
#[derive(Debug, Clone)]
pub struct ThemeReport {
    /// Theme name.
    pub theme: String,
    /// One result per contrast pair.
    pub results: Vec<PairResult>,
}

impl ThemeReport {
    /// Whether every pair passes.
    pub fn passes(&self) -> bool {
        self.results.iter().all(PairResult::passes)
    }

    /// The pairs that fail.
    pub fn failures(&self) -> impl Iterator<Item = &PairResult> {
        self.results.iter().filter(|r| !r.passes())
    }

    /// The pair with the lowest margin above its minimum.
    pub fn tightest(&self) -> Option<&PairResult> {
        self.results
            .iter()
            .min_by(|a, b| (a.ratio - a.pair.min).total_cmp(&(b.ratio - b.pair.min)))
    }
}

/// Contrast results for all themes.
#[derive(Debug, Clone)]
pub struct CheckReport {
    /// One report per theme, in manifest order.
    pub themes: Vec<ThemeReport>,
}

impl CheckReport {
    /// Whether every pair passes in every theme.
    pub fn passes(&self) -> bool {
        self.themes.iter().all(ThemeReport::passes)
    }
}

/// A loaded, structurally valid design system.
#[derive(Debug, Clone)]
pub struct DesignSystem {
    /// The manifest.
    pub manifest: Manifest,
    pub(crate) base: Vec<Token>,
    pub(crate) themes: Vec<(ThemeEntry, Vec<Token>)>,
    pairs: Vec<ContrastPair>,
}

impl DesignSystem {
    /// Loads `design.toml`, `contrast.toml` and all token files from `dir`.
    pub fn load(dir: &Path) -> Result<Self, Error> {
        let read = |relative: &str| {
            let path = dir.join(relative);
            std::fs::read_to_string(&path).map_err(|source| Error::Io {
                file: path.display().to_string(),
                source,
            })
        };
        Self::load_with(&read("design.toml")?, &read("contrast.toml")?, read)
    }

    /// Loads a design system from strings; `read` returns token files by relative path.
    pub fn load_with(
        manifest: &str,
        contrast: &str,
        read: impl Fn(&str) -> Result<String, Error>,
    ) -> Result<Self, Error> {
        let manifest: Manifest =
            toml::from_str(manifest).map_err(|e| Error::Manifest(e.message().to_owned()))?;
        let contrast: ContrastFile = toml::from_str(contrast).map_err(|e| Error::Parse {
            file: "contrast.toml".to_owned(),
            message: e.message().to_owned(),
        })?;
        let mut base = Vec::new();
        for file in &manifest.base {
            base.extend(dtcg::parse(&read(file)?, file)?);
        }
        let mut themes = Vec::new();
        for entry in &manifest.themes {
            themes.push((
                entry.clone(),
                dtcg::parse(&read(&entry.file)?, &entry.file)?,
            ));
        }
        let system = Self {
            manifest,
            base,
            themes,
            pairs: contrast.pairs,
        };
        system.validate()?;
        Ok(system)
    }

    /// Names of all themes, in manifest order.
    pub fn theme_names(&self) -> impl Iterator<Item = &str> {
        self.themes.iter().map(|(entry, _)| entry.name.as_str())
    }

    /// The contrast pairs from `contrast.toml`.
    pub fn pairs(&self) -> &[ContrastPair] {
        &self.pairs
    }

    /// Checks all contrast pairs in all themes.
    pub fn check(&self) -> Result<CheckReport, Error> {
        let themes = self
            .themes
            .iter()
            .map(|(entry, tokens)| {
                let resolver = Resolver::new(self.base.iter().chain(tokens));
                self.contrast(&entry.name, &resolver)
            })
            .collect::<Result<_, _>>()?;
        Ok(CheckReport { themes })
    }

    /// Validates an organisation theme (`overlay`, DTCG JSON) on top of `base_theme`.
    ///
    /// The overlay may only change existing semantic tokens of that theme, never tokens
    /// in protected groups, and the result must still pass every contrast pair.
    pub fn check_overlay(
        &self,
        base_theme: &str,
        overlay: &str,
        source: &str,
    ) -> Result<ThemeReport, Error> {
        let (_, theme_tokens) = self.theme(base_theme)?;
        let overlay = dtcg::parse(overlay, source)?;
        let theme_paths: BTreeSet<&str> = theme_tokens.iter().map(|t| t.path.as_str()).collect();
        let mut seen = HashMap::new();
        for token in &overlay {
            if let Some(group) = self
                .manifest
                .protected
                .iter()
                .find(|group| is_in_group(&token.path, group))
            {
                return Err(Error::Protected {
                    path: token.path.clone(),
                    group: group.clone(),
                });
            }
            if !theme_paths.contains(token.path.as_str()) {
                return Err(Error::UnknownOverride {
                    path: token.path.clone(),
                    theme: base_theme.to_owned(),
                });
            }
            if let Some(first) = seen.insert(token.path.as_str(), token.source.as_str()) {
                return Err(duplicate(token, first));
            }
        }
        let resolver = Resolver::new(self.base.iter().chain(theme_tokens).chain(&overlay));
        resolver.resolve_all()?;
        self.contrast(&format!("{base_theme} + {source}"), &resolver)
    }

    pub(crate) fn theme(&self, name: &str) -> Result<&(ThemeEntry, Vec<Token>), Error> {
        self.themes
            .iter()
            .find(|(entry, _)| entry.name == name)
            .ok_or_else(|| Error::theme(name, "no such theme in design.toml"))
    }

    fn contrast(&self, theme: &str, resolver: &Resolver<'_>) -> Result<ThemeReport, Error> {
        let mut results = Vec::with_capacity(self.pairs.len());
        for pair in &self.pairs {
            let bg = resolver.color(&pair.bg)?;
            if bg.alpha < 1.0 {
                return Err(Error::token(
                    &pair.bg,
                    "backgrounds in contrast pairs must be opaque",
                ));
            }
            let fg = resolver.color(&pair.fg)?.over(bg);
            results.push(PairResult {
                pair: pair.clone(),
                fg,
                bg,
                ratio: contrast_ratio(fg, bg),
            });
        }
        Ok(ThemeReport {
            theme: theme.to_owned(),
            results,
        })
    }

    fn validate(&self) -> Result<(), Error> {
        let m = &self.manifest;
        if !is_css_ident(&m.prefix) {
            return Err(Error::Manifest(format!(
                "prefix {:?} may only contain a-z, 0-9 and '-'",
                m.prefix
            )));
        }
        let mut names = BTreeSet::new();
        for (entry, _) in &self.themes {
            if !is_css_ident(&entry.name) {
                return Err(Error::theme(
                    &entry.name,
                    "names may only contain a-z, 0-9 and '-'",
                ));
            }
            if !names.insert(entry.name.as_str()) {
                return Err(Error::theme(&entry.name, "is defined twice"));
            }
        }
        let (_, default_tokens) = self.theme(&m.default_theme).map_err(|_| {
            Error::Manifest(format!(
                "default_theme {:?} is not defined",
                m.default_theme
            ))
        })?;

        // Base files: no duplicates.
        let mut base_paths: HashMap<&str, &str> = HashMap::new();
        for token in &self.base {
            if let Some(first) = base_paths.insert(&token.path, &token.source) {
                return Err(duplicate(token, first));
            }
        }

        // Themes: only new tokens, no duplicates, all themes define the same tokens.
        let expected: BTreeSet<&str> = default_tokens.iter().map(|t| t.path.as_str()).collect();
        for (entry, tokens) in &self.themes {
            let mut paths: HashMap<&str, &str> = HashMap::new();
            for token in tokens {
                if let Some(base_file) = base_paths.get(token.path.as_str()) {
                    return Err(Error::theme(
                        &entry.name,
                        format!(
                            "{} is already defined in {base_file}; themes may only add semantic tokens",
                            token.path
                        ),
                    ));
                }
                if let Some(first) = paths.insert(&token.path, &token.source) {
                    return Err(duplicate(token, first));
                }
            }
            let actual: BTreeSet<&str> = paths.keys().copied().collect();
            if let Some(missing) = expected.difference(&actual).next() {
                return Err(Error::theme(
                    &entry.name,
                    format!(
                        "{missing} is missing (it is defined in theme {})",
                        m.default_theme
                    ),
                ));
            }
            if let Some(extra) = actual.difference(&expected).next() {
                return Err(Error::theme(
                    &entry.name,
                    format!(
                        "{extra} is not defined in the default theme {}",
                        m.default_theme
                    ),
                ));
            }
            Resolver::new(self.base.iter().chain(tokens)).resolve_all()?;
        }

        // CSS names must be unique.
        let mut css_names: HashMap<String, &str> = HashMap::new();
        for token in self.base.iter().chain(default_tokens) {
            let name = crate::css::var_name(&m.prefix, &token.path);
            if let Some(other) = css_names.insert(name.clone(), &token.path) {
                return Err(Error::token(
                    &token.path,
                    format!("gives the same CSS name {name} as {other}"),
                ));
            }
        }
        Ok(())
    }
}

fn duplicate(token: &Token, first: &str) -> Error {
    Error::Duplicate {
        path: token.path.clone(),
        first: first.to_owned(),
        second: token.source.clone(),
    }
}

fn is_in_group(path: &str, group: &str) -> bool {
    path == group
        || path
            .strip_prefix(group)
            .is_some_and(|rest| rest.starts_with('.'))
}

fn is_css_ident(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// Resolves aliases within one layered set of tokens. Later tokens replace earlier ones
/// with the same path.
pub(crate) struct Resolver<'a> {
    tokens: HashMap<&'a str, &'a Token>,
}

impl<'a> Resolver<'a> {
    pub(crate) fn new(tokens: impl IntoIterator<Item = &'a Token>) -> Self {
        Self {
            tokens: tokens.into_iter().map(|t| (t.path.as_str(), t)).collect(),
        }
    }

    pub(crate) fn resolve(&self, path: &str) -> Result<(TokenType, Value), Error> {
        self.resolve_inner(path, &mut Vec::new())
    }

    fn resolve_all(&self) -> Result<(), Error> {
        let mut paths: Vec<&str> = self.tokens.keys().copied().collect();
        paths.sort_unstable();
        paths
            .into_iter()
            .try_for_each(|path| self.resolve(path).map(|_| ()))
    }

    fn color(&self, path: &str) -> Result<Srgb, Error> {
        match self.resolve(path)? {
            (_, Value::Color(color)) => Ok(color),
            (ty, _) => Err(Error::token(
                path,
                format!("is a {} token, but contrast pairs need colours", ty.name()),
            )),
        }
    }

    fn resolve_inner(
        &self,
        path: &str,
        stack: &mut Vec<String>,
    ) -> Result<(TokenType, Value), Error> {
        if let Some(start) = stack.iter().position(|p| p == path) {
            let mut cycle = stack[start..].to_vec();
            cycle.push(path.to_owned());
            return Err(Error::token(
                path,
                format!("aliases form a cycle: {}", cycle.join(" → ")),
            ));
        }
        let token = self
            .tokens
            .get(path)
            .ok_or_else(|| Error::token(path, "does not exist"))?;
        match &token.value {
            RawValue::Alias(target) => {
                if !self.tokens.contains_key(target.as_str()) {
                    return Err(Error::token(
                        path,
                        format!("refers to {{{target}}}, which does not exist"),
                    ));
                }
                stack.push(path.to_owned());
                let (ty, value) = self.resolve_inner(target, stack)?;
                stack.pop();
                match token.ty {
                    Some(declared) if declared != ty => Err(Error::token(
                        path,
                        format!(
                            "has type {} but refers to {{{target}}} of type {}",
                            declared.name(),
                            ty.name()
                        ),
                    )),
                    _ => Ok((ty, value)),
                }
            }
            RawValue::Literal(json) => {
                let ty = token.ty.ok_or_else(|| {
                    Error::token(path, "has no $type (set it on the token or a parent group)")
                })?;
                let value =
                    Value::parse(ty, json).map_err(|message| Error::token(path, message))?;
                Ok((ty, value))
            }
        }
    }
}
