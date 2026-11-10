//! Licence notices and source code (LIC-1, ADR-0055): the footer under every screen and the
//! page it links to. The page shows ORDO's licence and source, the parts the app ships that
//! are not Rust crates, and the crates' notices, which `cargo-about` writes at build time
//! (`tools/notices`) and the page loads only when it opens.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::text;

/// The path of the page.
pub const PATH: &str = "/notices";
/// The crates' notices, an HTML fragment made at build time (escaped by its template).
const CRATES: &str = "/third-party/crates.html";

/// Where the source code of this build is: the workspace's repository, at the commit CI
/// built (`ORDO_COMMIT`), so a changed ORDO built elsewhere points to its own source.
pub fn source_url() -> String {
    let repository = env!("CARGO_PKG_REPOSITORY");
    match option_env!("ORDO_COMMIT") {
        Some(commit) if !commit.is_empty() => format!("{repository}/tree/{commit}"),
        _ => repository.to_owned(),
    }
}

/// A part the app ships that is not a Rust crate.
pub struct Part {
    /// Name and version.
    pub name: &'static str,
    /// What it does in ORDO.
    pub used_for: &'static str,
    /// Copyright holders as the part names them, with "©".
    pub copyright: &'static str,
    /// SPDX licence expression.
    pub licence: &'static str,
    /// The licence text: a file the app serves, or the licence's own page.
    pub text: &'static str,
}

// [impl->dsn~licence-notices~1]
/// The non-Rust parts. Keep in step with `vendor-map.sh`, `design/assets/icons`,
/// `tools/symbols` and the storage worker's SQLite.
pub const PARTS: &[Part] = &[
    Part {
        name: "MapLibre GL JS 6.17.0",
        used_for: text::NOTICES_USED_MAP,
        copyright: "© MapLibre contributors; Mapbox; Evan Wallace; Mike Bostock",
        licence: "BSD-3-Clause",
        text: "/map/vendor/maplibre-gl.LICENSE.txt",
    },
    Part {
        name: "PMTiles 4.5.0",
        used_for: text::NOTICES_USED_MAP,
        copyright: "© Protomaps LLC",
        licence: "BSD-3-Clause",
        text: "/map/vendor/pmtiles.LICENSE.txt",
    },
    Part {
        name: "Protomaps basemaps 5.7.2",
        used_for: text::NOTICES_USED_MAP,
        copyright: "© Protomaps LLC, Kelso Cartography",
        licence: "BSD-3-Clause AND CC0-1.0",
        text: "/map/vendor/basemaps.LICENSE.md",
    },
    Part {
        name: "Noto Sans",
        used_for: text::NOTICES_USED_MAP_FONT,
        copyright: "© The Noto Project Authors",
        licence: "OFL-1.1",
        text: "/map/glyphs/OFL.txt",
    },
    Part {
        name: "Lucide 1.70.0",
        used_for: text::NOTICES_USED_ICONS,
        copyright: "© Lucide Contributors; Cole Bemis (Feather)",
        licence: "ISC AND MIT",
        text: "/LICENSES/ISC.txt",
    },
    Part {
        name: "@taktische-zeichen/core 0.11.0",
        used_for: text::NOTICES_USED_SYMBOLS,
        copyright: "© Philipp Jardas",
        licence: "MIT",
        text: "/LICENSES/MIT.txt",
    },
    Part {
        name: "SQLite 3.53.4",
        used_for: text::NOTICES_USED_STORAGE,
        copyright: text::NOTICES_PUBLIC_DOMAIN,
        licence: "blessing",
        text: "https://sqlite.org/copyright.html",
    },
    Part {
        name: "OpenStreetMap",
        used_for: text::NOTICES_USED_MAP_DATA,
        copyright: "© OpenStreetMap contributors",
        licence: "ODbL-1.0",
        text: "https://www.openstreetmap.org/copyright",
    },
];

/// Every screen's footer: the way to the licences and the source code.
#[component]
pub fn SiteFooter() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <a href=PATH>{text::NOTICES_TITLE}</a>
        </footer>
    }
}

/// Loads the crates' notices; `None` if this build has none.
async fn crates() -> Option<String> {
    let window = web_sys::window()?;
    let response: web_sys::Response =
        wasm_bindgen_futures::JsFuture::from(window.fetch_with_str(CRATES))
            .await
            .ok()?
            .dyn_into()
            .ok()?;
    if !response.ok() {
        return None;
    }
    let body = wasm_bindgen_futures::JsFuture::from(response.text().ok()?)
        .await
        .ok()?
        .as_string()?;
    // The server answers unknown files with the app itself.
    (!body.trim().is_empty() && !body.contains("<html")).then_some(body)
}

/// The page "Lizenzen und Quellcode"; `back` is the screen the person came from.
#[component]
pub fn NoticesPage(back: String) -> impl IntoView {
    let fragment = LocalResource::new(crates);
    let source = source_url();
    view! {
        <main id="main" class="page page--narrow notices" tabindex="-1">
            <h1>{text::NOTICES_TITLE}</h1>
            <p><a href=back>{text::NOTICES_BACK}</a></p>
            <section class="card" aria-labelledby="notices-ordo">
                <h2 id="notices-ordo">{text::APP_NAME}</h2>
                <p>{text::NOTICES_ORDO_LICENCE} " "
                    <a href="/LICENSES/AGPL-3.0-or-later.txt">{text::NOTICES_LICENCE_TEXT}</a>
                </p>
                <p>{text::NOTICES_SOURCE} " " <a class="notices__source" href=source.clone()>{source.clone()}</a></p>
            </section>
            <section class="card" aria-labelledby="notices-parts">
                <h2 id="notices-parts">{text::NOTICES_PARTS}</h2>
                <ul class="notices__parts">
                    {PARTS
                        .iter()
                        .map(|p| view! {
                            <li class="notices__part">
                                <strong>{p.name}</strong> " – " {p.used_for}
                                <br />{p.copyright} " · "
                                <a href=p.text>{p.licence}</a>
                            </li>
                        })
                        .collect_view()}
                </ul>
            </section>
            <section class="card" aria-labelledby="notices-crates">
                <h2 id="notices-crates">{text::NOTICES_CRATES}</h2>
                <Suspense fallback=|| view! { <p>{text::NOTICES_LOADING}</p> }>
                    {move || {
                        fragment.get().map(|body| match body {
                            Some(html) => view! {
                                <p class="hint">{text::NOTICES_CRATES_HINT}</p>
                                <div class="notices__crates" inner_html=html></div>
                            }
                            .into_any(),
                            None => view! { <p>{text::NOTICES_CRATES_MISSING}</p> }.into_any(),
                        })
                    }}
                </Suspense>
            </section>
        </main>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~licence-notices~1]
    #[test]
    fn every_part_has_its_licence_text() {
        let app = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        for part in PARTS {
            assert!(!part.copyright.is_empty(), "{}", part.name);
            if part.text.starts_with("https://") {
                continue;
            }
            // Served from the app's files (`map/`) or the repository's `LICENSES/`.
            let file = match part.text.strip_prefix("/LICENSES/") {
                Some(name) => app.join("../../LICENSES").join(name),
                None => app.join(part.text.trim_start_matches('/')),
            };
            assert!(
                file.is_file(),
                "{}: {} is missing",
                part.name,
                file.display()
            );
        }
    }

    #[test]
    fn the_source_points_to_the_repository() {
        assert!(source_url().starts_with("https://"));
    }
}
