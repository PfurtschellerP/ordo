//! Licence notices and source code (LIC-1, ADR-0055): *Administration – Info* for everyone
//! who uses the app, and a page of their own linked from the pairing screen, before anyone
//! has logged in. The page shows ORDO's licence and source, the parts the app ships that
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

// [impl->dsn~licence-notices~2]
/// The non-Rust parts. Keep in step with `vendor-map.sh`, `vendor-fonts.sh`, `design/assets/icons`,
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
        name: "Terra Draw 1.39.0, MapLibre adapter 1.5.0",
        used_for: text::NOTICES_USED_MAP,
        copyright: "© James Milner",
        licence: "MIT",
        text: "/map/vendor/terra-draw.LICENSE.txt",
    },
    Part {
        name: "Atkinson Hyperlegible Next 5.3.0",
        used_for: text::NOTICES_USED_FONT,
        copyright: "© The Atkinson Hyperlegible Next Project Authors",
        licence: "OFL-1.1",
        text: "/fonts/OFL-atkinson-hyperlegible-next.txt",
    },
    Part {
        name: "Atkinson Hyperlegible Mono 5.3.0",
        used_for: text::NOTICES_USED_FONT,
        copyright: "© The Atkinson Hyperlegible Mono Project Authors",
        licence: "OFL-1.1",
        text: "/fonts/OFL-atkinson-hyperlegible-mono.txt",
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

/// The way to the licences and the source code on the pairing screen, before any menu.
#[component]
pub fn NoticesLink() -> impl IntoView {
    view! {
        <p class="notices-link">
            <a href=PATH>{text::NOTICES_TITLE}</a>
        </p>
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

/// The page "Lizenzen und Quellcode" before pairing; `back` is the screen the person came
/// from.
#[component]
pub fn NoticesPage(back: String) -> impl IntoView {
    view! {
        <main id="main" class="page page--narrow notices" tabindex="-1">
            <h1>{text::NOTICES_TITLE}</h1>
            <p><a href=back>{text::NOTICES_BACK}</a></p>
            <Notices />
        </main>
    }
}

// [impl->dsn~licence-notices~2]
/// *Administration – Info*: the versions of app and server, then the notices.
#[component]
pub fn InfoView() -> impl IntoView {
    let server = LocalResource::new(|| async {
        crate::core::fetch_json("GET", "/.well-known/ordo", None)
            .await
            .ok()
    });
    let app = text::info_build(env!("CARGO_PKG_VERSION"), commit());
    let app_protocols = text::info_protocols(
        &range(&serde_json::to_value(ordo_sync::version::SYNC).unwrap_or_default()),
        &range(&serde_json::to_value(ordo_sync::version::HTTP).unwrap_or_default()),
    );
    view! {
        <div class="info notices">
            <section class="card" aria-labelledby="info-version">
                <h2 id="info-version">{text::INFO_VERSION}</h2>
                <dl class="facts">
                    <dt>{text::INFO_APP}</dt>
                    <dd id="info-app">{app} <br /> {app_protocols}</dd>
                    <dt>{text::INFO_SERVER}</dt>
                    <dd id="info-server">
                        <Suspense fallback=|| text::INFO_LOADING>
                            {move || server.get().map(|answer| match answer {
                                Some(v) => {
                                    let build = text::info_build(
                                        v["build"]["version"].as_str().unwrap_or("?"),
                                        v["build"]["commit"].as_str().filter(|c| !c.is_empty()),
                                    );
                                    let protocols = text::info_protocols(&range(&v["sync"]), &range(&v["http"]));
                                    view! { {build} <br /> {protocols} }.into_any()
                                }
                                None => text::INFO_SERVER_OFFLINE.into_any(),
                            })}
                        </Suspense>
                    </dd>
                </dl>
            </section>
            <Notices />
        </div>
    }
}

/// The commit this app was built from, if CI or the container build named it.
fn commit() -> Option<&'static str> {
    option_env!("ORDO_COMMIT").filter(|c| !c.is_empty())
}

/// "1.0–1.2" from `{"min": "1.0", "max": "1.2"}`, or just "1.0" when both are the same.
fn range(v: &serde_json::Value) -> String {
    let (min, max) = (
        v["min"].as_str().unwrap_or("?"),
        v["max"].as_str().unwrap_or("?"),
    );
    if min == max {
        min.to_owned()
    } else {
        format!("{min}–{max}")
    }
}

/// ORDO's licence and source, the parts the app ships, and the crates' notices.
#[component]
fn Notices() -> impl IntoView {
    let fragment = LocalResource::new(crates);
    let source = source_url();
    view! {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~licence-notices~2]
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
