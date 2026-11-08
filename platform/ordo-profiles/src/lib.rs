//! Profile tooling (architecture 8.8): checks profile manifests, generates the source
//! register from them, and watches the sources a profile is based on for changes.
//!
//! Every source in a `profile.toml` records when a maintainer last confirmed the profile
//! against it (`checked`) and a fingerprint of the version they saw (`fingerprint`). The
//! watcher fetches every source again and reports those whose fingerprint changed or that
//! cannot be reached. Nothing here is regional: which profiles exist is data.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};

/// A profile manifest (`profile.toml`).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// Profile ID, equal to the directory name.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Content version.
    pub version: String,
    /// Profile format version (I18N-8).
    pub format: u32,
    /// Profiles this one extends.
    pub extends: Vec<String>,
    /// Languages of the labels.
    pub languages: Vec<String>,
    /// Documents the content is based on.
    #[serde(default)]
    pub source: Vec<Source>,
}

/// A document a profile is based on.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// What it covers, e.g. `command`, `status-codes`.
    pub topic: String,
    /// Title.
    pub title: String,
    /// Edition, date of the introducing decree, …
    #[serde(default)]
    pub edition: Option<String>,
    /// Where to get it (HTTPS).
    pub url: String,
    /// Remarks.
    #[serde(default)]
    pub note: Option<String>,
    /// How to watch it: `content` (the bytes; default for documents), `text` (the visible
    /// text of a web page), `links` (the documents a download page links to; default for
    /// web pages), `manual` (sites that refuse automated requests: checked by hand, a
    /// reminder after [`MANUAL_DAYS`]).
    #[serde(default)]
    pub watch: Option<String>,
    /// When a maintainer last confirmed the profile against this source.
    #[serde(default)]
    pub checked: Option<toml::value::Datetime>,
    /// Fingerprint of the version confirmed then, e.g. `sha256:…`.
    #[serde(default)]
    pub fingerprint: Option<String>,
}

/// A profile on disk.
pub struct Profile {
    /// Its `profile.toml`.
    pub path: PathBuf,
    /// Its manifest.
    pub manifest: Manifest,
}

/// Loads every profile below `root` (one directory per profile), by ID.
pub fn load(root: &Path) -> Result<BTreeMap<String, Profile>, String> {
    let mut profiles = BTreeMap::new();
    let entries = std::fs::read_dir(root).map_err(|e| format!("{}: {e}", root.display()))?;
    for entry in entries {
        let dir = entry.map_err(|e| e.to_string())?.path();
        if !dir.is_dir() {
            continue;
        }
        let path = dir.join("profile.toml");
        let text =
            std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let manifest: Manifest =
            toml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
        let name = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        profiles.insert(name, Profile { path, manifest });
    }
    Ok(profiles)
}

/// Problems in the manifests: IDs, layering, sources.
pub fn validate(profiles: &BTreeMap<String, Profile>) -> Vec<String> {
    let mut problems = Vec::new();
    for (dir, profile) in profiles {
        let m = &profile.manifest;
        if &m.id != dir {
            problems.push(format!("{dir}: id {:?} differs from the directory", m.id));
        }
        if m.format != 1 {
            problems.push(format!("{dir}: unknown profile format {}", m.format));
        }
        if m.name.is_empty() || m.version.is_empty() || m.languages.is_empty() {
            problems.push(format!("{dir}: name, version and languages are required"));
        }
        for parent in &m.extends {
            if !profiles.contains_key(parent) {
                problems.push(format!("{dir}: extends {parent}, which does not exist"));
            }
        }
        for s in &m.source {
            if s.topic.is_empty() || s.title.is_empty() {
                problems.push(format!("{dir}: a source needs a topic and a title"));
            }
            if !s.url.starts_with("https://") {
                problems.push(format!("{dir}: {} must use HTTPS", s.url));
            }
            if let Some(w) = &s.watch
                && !matches!(w.as_str(), "content" | "text" | "links" | "manual")
            {
                problems.push(format!("{dir}: unknown watch mode {w:?}"));
            }
            if s.checked.is_some() != s.fingerprint.is_some() {
                problems.push(format!(
                    "{dir}: {}: `checked` and `fingerprint` go together",
                    s.url
                ));
            }
        }
    }
    problems
}

/// Days after which a source checked by hand is due again.
pub const MANUAL_DAYS: i64 = 180;

/// Whether a manual check of `checked` is due on `today`.
pub fn manual_due(checked: Option<toml::value::Datetime>, today: time::Date) -> bool {
    let Some(date) = checked.and_then(|c| c.date) else {
        return true;
    };
    let Ok(month) = time::Month::try_from(date.month) else {
        return true;
    };
    match time::Date::from_calendar_date(i32::from(date.year), month, date.day) {
        Ok(d) => (today - d).whole_days() > MANUAL_DAYS,
        Err(_) => true,
    }
}

/// Start and end markers of the generated table in a register document.
pub const REGISTER_START: &str = "<!-- generated by ordo-profiles register: start -->";
/// End marker.
pub const REGISTER_END: &str = "<!-- generated by ordo-profiles register: end -->";

/// The register table of all profiles extending `parent`: name, profile, command doctrine
/// (topic `command`), other sources, and the oldest confirmation date. Links are relative
/// to `doc_dir`'s sibling `profiles/` directory (`../profiles/…`).
pub fn register_table(profiles: &BTreeMap<String, Profile>, parent: &str) -> String {
    let link = |s: &Source| {
        let mut text = format!("[{}]({})", s.title, s.url);
        if let Some(e) = &s.edition {
            text.push_str(", ");
            text.push_str(e);
        }
        text
    };
    let mut out = String::from(
        "| Region | Profile | Command doctrine as introduced | Other sources | Checked |\n|---|---|---|---|---|\n",
    );
    let mut rows: Vec<&Profile> = profiles
        .values()
        .filter(|p| p.manifest.extends.iter().any(|e| e == parent))
        .collect();
    rows.sort_by(|a, b| a.manifest.name.cmp(&b.manifest.name));
    for p in rows {
        let m = &p.manifest;
        let command: Vec<String> = m
            .source
            .iter()
            .filter(|s| s.topic == "command")
            .map(|s| {
                let mut t = link(s);
                if let Some(n) = &s.note {
                    t.push_str(" — ");
                    t.push_str(n);
                }
                t
            })
            .collect();
        let other: Vec<String> = m
            .source
            .iter()
            .filter(|s| s.topic != "command")
            .map(link)
            .collect();
        let checked = m
            .source
            .iter()
            .map(|s| s.checked.map(|d| d.to_string()))
            .min_by(|a, b| match (a, b) {
                (None, _) => std::cmp::Ordering::Less,
                (_, None) => std::cmp::Ordering::Greater,
                (Some(a), Some(b)) => a.cmp(b),
            })
            .map(|d| d.unwrap_or_else(|| "never".to_owned()))
            .unwrap_or_else(|| "—".to_owned());
        out.push_str(&format!(
            "| {} | [`{}`](../profiles/{}/profile.toml) | {} | {} | {} |\n",
            m.name,
            m.id,
            m.id,
            if command.is_empty() {
                "*not found yet*".to_owned()
            } else {
                command.join("; ")
            },
            if other.is_empty() {
                "—".to_owned()
            } else {
                other.join("; ")
            },
            checked,
        ));
    }
    out
}

/// Replaces the generated part of a register document; `None` without markers.
pub fn splice_register(doc: &str, table: &str) -> Option<String> {
    let start = doc.find(REGISTER_START)? + REGISTER_START.len();
    let end = doc.find(REGISTER_END)?;
    (start <= end).then(|| format!("{}\n{}{}", &doc[..start], table, &doc[end..]))
}

/// How a fetched source is compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Watch {
    /// The bytes.
    Content,
    /// The visible text of a web page.
    Text,
    /// The documents a page links to.
    Links,
}

impl Watch {
    /// The mode of `source`, by its setting or else by the content type.
    pub fn of(source: &Source, content_type: &str) -> Self {
        match source.watch.as_deref() {
            Some("content") => Self::Content,
            Some("text") => Self::Text,
            Some("links") => Self::Links,
            _ if content_type.contains("html") => Self::Links,
            _ => Self::Content,
        }
    }
}

/// The fingerprint of a fetched source: SHA-256 of what `watch` compares.
pub fn fingerprint(watch: Watch, body: &[u8]) -> String {
    let (prefix, data) = match watch {
        Watch::Content => ("sha256", body.to_vec()),
        Watch::Text => (
            "text-sha256",
            visible_text(&String::from_utf8_lossy(body)).into_bytes(),
        ),
        Watch::Links => (
            "links-sha256",
            document_links(&String::from_utf8_lossy(body))
                .join("\n")
                .into_bytes(),
        ),
    };
    let hash = Sha256::digest(&data);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("{prefix}:{hex}")
}

/// The documents (PDF, office files, archives) a web page links to, sorted.
pub fn document_links(html: &str) -> Vec<String> {
    let mut links: Vec<String> = html
        .split("href=")
        .skip(1)
        .filter_map(|rest| {
            let quote = rest.chars().next()?;
            let rest = rest.get(1..)?;
            let end = if quote == '"' || quote == '\'' {
                rest.find(quote)?
            } else {
                return None;
            };
            Some(rest[..end].trim().to_owned())
        })
        .filter(|href| {
            let path = href
                .split(['?', '#'])
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            [".pdf", ".doc", ".docx", ".odt", ".xls", ".xlsx", ".zip"]
                .iter()
                .any(|ext| path.ends_with(ext))
        })
        .collect();
    links.sort();
    links.dedup();
    links
}

/// The visible text of a web page: without tags, scripts and styles, whitespace collapsed.
pub fn visible_text(html: &str) -> String {
    let mut text = String::new();
    let mut rest = html;
    while let Some(open) = rest.find('<') {
        text.push_str(&rest[..open]);
        text.push(' ');
        let after = &rest[open..];
        let lower = after.get(..8).unwrap_or("").to_ascii_lowercase();
        let skip_to = if lower.starts_with("<script") {
            after.to_ascii_lowercase().find("</script>").map(|i| i + 9)
        } else if lower.starts_with("<style") {
            after.to_ascii_lowercase().find("</style>").map(|i| i + 8)
        } else {
            after.find('>').map(|i| i + 1)
        };
        match skip_to {
            Some(i) => rest = &after[i..],
            None => {
                rest = "";
                break;
            }
        }
    }
    text.push_str(rest);
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Today's date (UTC) as a TOML date.
pub fn today() -> toml_edit::Datetime {
    let d = time::OffsetDateTime::now_utc().date();
    toml_edit::Datetime {
        date: Some(toml_edit::Date {
            year: d.year() as u16,
            month: d.month() as u8,
            day: d.day(),
        }),
        time: None,
        offset: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_ignore_everything_but_documents() {
        let html = r#"<a href="/a.pdf">A</a> <a href='b.DOCX?x=1'>B</a> <a href="/news">N</a>
            <a href="/a.pdf">again</a> <link href="style.css">"#;
        assert_eq!(document_links(html), ["/a.pdf", "b.DOCX?x=1"]);
    }

    #[test]
    fn text_drops_markup_scripts_and_layout() {
        let html = "<html><style>p{}</style><script>var t=1;</script><p>FwDV  100</p>\n<p>Stand 1999</p></html>";
        assert_eq!(visible_text(html), "FwDV 100 Stand 1999");
        assert_eq!(
            fingerprint(Watch::Text, html.as_bytes()),
            fingerprint(Watch::Text, b"<div>FwDV 100</div> Stand   1999"),
            "layout changes do not count"
        );
    }

    #[test]
    fn a_changed_document_changes_the_fingerprint() {
        assert_ne!(
            fingerprint(Watch::Content, b"v1"),
            fingerprint(Watch::Content, b"v2")
        );
        assert!(fingerprint(Watch::Links, b"").starts_with("links-sha256:"));
    }

    #[test]
    fn manual_checks_fall_due_after_half_a_year() {
        let checked: toml::value::Datetime = "2026-01-01".parse().unwrap();
        let day = |m, d| time::Date::from_calendar_date(2026, m, d).unwrap();
        assert!(!manual_due(Some(checked), day(time::Month::May, 1)));
        assert!(manual_due(Some(checked), day(time::Month::August, 1)));
        assert!(manual_due(None, day(time::Month::May, 1)));
    }

    #[test]
    fn the_register_is_spliced_between_the_markers() {
        let doc = format!("intro\n{REGISTER_START}\nold\n{REGISTER_END}\noutro");
        let new = splice_register(&doc, "| table |\n").unwrap();
        assert_eq!(
            new,
            format!("intro\n{REGISTER_START}\n| table |\n{REGISTER_END}\noutro")
        );
        assert!(splice_register("no markers", "x").is_none());
    }

    #[test]
    fn the_profiles_of_this_repository_are_valid() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles");
        let profiles = load(&root).expect("profiles load");
        assert!(!profiles.is_empty());
        assert_eq!(validate(&profiles), Vec::<String>::new());
    }
}
