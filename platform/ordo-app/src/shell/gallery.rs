//! The component gallery (DS-1, ADR-0094): every building block of the app with sample
//! data, in the theme chosen on the page, with when to use it and what it does for
//! accessibility. The shell's own blocks are here; each package adds its own through the
//! package registry. Open at `/gallery`, before pairing, like the licence notices. The
//! explanations are for developers and designers, in English; the examples use the app's
//! texts.

use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::shell::ui::{
    Action, Actions, Ago, CheckChips, Disclosure, EditButton, EditDialog, EmptyState, History,
    ICONS, Icon, IconButton, ItemRow, KeyValues, MoreActions, ObjectChip, RadioChips, SectionHead,
    StatusTag, Suggestions, Tone, UndoBar, UndoSlot, offer_undo,
};
use crate::shell::views::{Nav, show_theme, stored_theme};
use crate::text;

/// The page's path.
pub const PATH: &str = "/gallery";

/// One building block: what it is called, where it lives, when to use it, what it does
/// for accessibility, and its example.
pub struct Example {
    /// Anchor on the page.
    pub id: &'static str,
    /// Its name.
    pub name: &'static str,
    /// The component or the class, e.g. `shell::ui::IconButton` or `.notice`.
    pub code: &'static str,
    /// When to use it, and when not.
    pub usage: &'static str,
    /// What it does for accessibility.
    pub access: &'static str,
    /// The example, with its states.
    pub show: fn() -> AnyView,
}

/// A part of the gallery: a heading and its examples.
pub struct Part {
    /// Anchor on the page.
    pub id: &'static str,
    /// Its heading.
    pub title: &'static str,
    /// Its building blocks.
    pub examples: &'static [Example],
}

/// The shell's parts, before the packages'.
const SHELL: &[Part] = &[
    Part {
        id: "foundations",
        title: "Foundations",
        examples: &[
            Example {
                id: "colours",
                name: "Colour roles",
                code: "--ordo-color-*",
                usage: "Semantic colours of the theme shown. Use the role, never a primitive colour or a hex value: themes and organisation layers change the roles only. Safety colours (triage, alarm) cannot be changed by a theme.",
                access: "Every pair of text and background is checked for WCAG contrast in every theme (design/contrast.toml). Colour never carries meaning alone.",
                show: colours,
            },
            Example {
                id: "type",
                name: "Type sizes",
                code: "--ordo-font-size-0 … 6",
                usage: "Size 2 is body text, 3 and 4 are headings inside cards, 5 and 6 page headings. Bold for labels and values, regular for text.",
                access: "Sizes are in rem, so the person's text size applies; layouts work at 200 %.",
                show: type_sizes,
            },
            Example {
                id: "space",
                name: "Spacing and radii",
                code: "--ordo-space-1 … 8, --ordo-radius-*",
                usage: "Space 2 inside a group (label and field), 3 between fields, 4 between groups, 5 inside and between cards. Radius small for fields, medium for cards and tiles, large for chips and tags.",
                access: "Touch targets are at least --ordo-size-touch-target (3 rem, about 12 mm).",
                show: spacing,
            },
            Example {
                id: "icons",
                name: "Icons",
                code: "shell::ui::Icon",
                usage: "Lucide icons, one icon one meaning (icon rules). An icon goes with a word; icon only for actions repeated on many items, through IconButton.",
                access: "Icons are decorative (aria-hidden); the word or the button's name carries the meaning.",
                show: icons,
            },
        ],
    },
    Part {
        id: "actions",
        title: "Actions",
        examples: &[
            Example {
                id: "buttons",
                name: "Buttons",
                code: ".button, --primary, --danger, --small, --with-icon",
                usage: "One primary button per form or card: its main action. Default for everything else; danger only to confirm a step with consequences. Small inside cards and lists. A button that pushes itself to the end of its row (cancel, withdraw, edit) adds --quiet.",
                access: "Native button elements. A button that is off is greyed out and dashed, not by colour alone; prefer saying why next to it.",
                show: buttons,
            },
            Example {
                id: "icon-buttons",
                name: "Icon buttons",
                code: "shell::ui::IconButton",
                usage: "Actions repeated on many items: show on the map, edit, remove, close. Never a form's main action. The word shows where the row is wide.",
                access: "The accessible name says action and item (\"Bearbeiten: RTW 83/1\"), shown as tooltip on hover and keyboard focus. At least 44 × 44 px.",
                show: icon_buttons,
            },
            Example {
                id: "action-rows",
                name: "Action rows",
                code: "shell::ui::Actions",
                usage: "The steps of an item (Verladen, Abgefahren, Am Ziel), in order; ending actions are quiet and go to the end.",
                access: "Each button's name names its item, so a list of rows stays clear to screen readers.",
                show: action_rows,
            },
            Example {
                id: "edit-dialog",
                name: "Edit dialog",
                code: "shell::ui::EditButton, shell::ui::EditDialog",
                usage: "Editing an item of a card list. One dialog per list; the card keeps its layout. Phones get the full screen.",
                access: "A modal dialog element: focus stays inside, Escape closes, the focus returns to the button that opened it.",
                show: edit_dialog,
            },
            Example {
                id: "undo",
                name: "Undo",
                code: "shell::ui::offer_undo, shell::ui::UndoBar",
                usage: "After a removal by icon button, instead of asking first. Ten seconds, or until the next press elsewhere.",
                access: "A polite status region: announced without moving the focus.",
                show: undo,
            },
        ],
    },
    Part {
        id: "fields",
        title: "Fields",
        examples: &[
            Example {
                id: "text-fields",
                name: "Text fields",
                code: ".field, .field--invalid, .hint, .field-error",
                usage: "A label above every field, a hint below it where needed, the error below that. Fields in a row (.row) wrap on narrow screens.",
                access: "The label names the field; hint and error are linked with aria-describedby. An invalid field has aria-invalid, a dashed border, a sign after its label and the message in words.",
                show: text_fields,
            },
            Example {
                id: "selects",
                name: "Selects and text areas",
                code: "select, textarea, shell::ui::Suggestions",
                usage: "A select for a closed list; a text input with suggestions (datalist) where anything else may be typed too, e.g. people and call signs.",
                access: "Native elements, so every platform's own picker works.",
                show: selects,
            },
            Example {
                id: "chips",
                name: "Chips",
                code: "shell::ui::RadioChips, shell::ui::CheckChips",
                usage: "A few short answers seen at once: one (radio) or several (check). More than about eight, or long answers: a select or check boxes in a list.",
                access: "A fieldset with legend, real radio buttons and check boxes. The chosen chip gets a thick border and bold text, not only colour.",
                show: chips,
            },
            Example {
                id: "check-box",
                name: "Check box",
                code: ".choice",
                usage: "A single yes or no with a sentence as label, e.g. a setting. Not a chip: chips are for groups of short answers.",
                access: "A real check box inside its label, so the whole line is the target.",
                show: check_box,
            },
        ],
    },
    Part {
        id: "status",
        title: "Status and messages",
        examples: &[
            Example {
                id: "notices",
                name: "Notices",
                code: ".notice, .notice--warning, .error, .hint",
                usage: "A notice explains a state of the page (read only, nothing chosen). A warning asks to check something. An error says what went wrong and what to do. A hint is plain secondary text; it is never boxed.",
                access: "Errors that appear after an action have role alert; notices are plain text. Each says its meaning in words, so colour is extra.",
                show: notices,
            },
            Example {
                id: "tags",
                name: "Tags",
                code: ".tag, .tag--warning, .state-tag",
                usage: "A word or two for a state next to a heading or an item (Geplant, 2 fällig). A sentence is not a tag: use a hint.",
                access: "The word carries the state; the colour repeats it.",
                show: tags,
            },
            Example {
                id: "ago",
                name: "Times",
                code: "shell::ui::Ago",
                usage: "A value's age next to its time, e.g. a vital sign. Counts on every 30 seconds.",
                access: "Not announced while it counts.",
                show: ago,
            },
        ],
    },
    Part {
        id: "containers",
        title: "Containers",
        examples: &[
            Example {
                id: "cards",
                name: "Cards",
                code: ".card",
                usage: "A section of a page with its heading (h2). Items inside a card are separated by space and headings; an item list may frame its items once more, never deeper.",
                access: "A section with aria-labelledby on its heading, so it is a landmark-like region in the reading order.",
                show: cards,
            },
            Example {
                id: "disclosure",
                name: "Disclosure",
                code: "shell::ui::Disclosure, details",
                usage: "Something seen only on demand: more actions, a history. details/summary for read-only content (History), Disclosure for menus and panels with controls.",
                access: "aria-expanded on the button; Escape closes and returns the focus.",
                show: disclosure,
            },
            Example {
                id: "tabs",
                name: "Tabs",
                code: "shell::ui::Parts (nav::View::tabs for a page), shell::ui::TabList",
                usage: "Separate tasks used one at a time: always tabs (Personal, Einsatzmittel, the patient list, the record). Parts used together: side by side from 60 or 84rem, tabs below (Führung & Raum, Transporte). Not for one list or form, steps in order, or a part shorter than a screen; what needs attention stays above the tabs. Two to five nouns, a short label for phones, the main part first; an action that needs another part shows it (chapter 8.6.5).",
                access: "WAI-ARIA tabs: arrow keys, Home and End move, vim gt/gT; the chosen tab is underlined thick and bold. Hidden parts keep their input; focusing into one shows it.",
                show: tabs,
            },
        ],
    },
    // [impl->dsn~building-blocks~1]
    Part {
        id: "blocks",
        title: "Building blocks of calm pages",
        examples: &[
            Example {
                id: "section-head",
                name: "Section head",
                code: "shell::ui::SectionHead",
                usage: "The head of each part of a page: icon, title, number, one action at the right (\"+ Neu\"). Close to its rows, apart from the section above (space and a thin line). Replaces a large heading inside each card (ADR-0102).",
                access: "A real heading; the number is text; the action names its section.",
                show: section_head,
            },
            Example {
                id: "object-chips",
                name: "Object chips and the \"⋯\" menu",
                code: "shell::ui::ObjectChip, shell::ui::MoreActions",
                usage: "An item's objects (its pin, its destination) as chips that open their dialog; dashed while not set. One next step at hand, the others in a menu over the page; ending steps last, set apart. Nothing opens inside the card (ADR-0104).",
                access: "Chips are buttons named with the item and say they open a dialog; the menu is a popover that Escape and a click outside close.",
                show: object_chips,
            },
            Example {
                id: "item-row",
                name: "Item row",
                code: "shell::ui::ItemRow, .item-rows",
                usage: "One item of a list: time and text at the left, the text taking the free width; the status tag, then icon actions at the far right, lined up down the list. Items are rows with a line between them, never cards inside a card. A documented entry is one row, opened for its details.",
                access: "A list item; tag and actions follow the text in reading order.",
                show: item_rows,
            },
            Example {
                id: "status-tag",
                name: "Status tag",
                code: "shell::ui::StatusTag, Tone",
                usage: "A state in four tones: ok, attention, urgent, neutral. Always icon or shape plus word.",
                access: "The word is the text; colour repeats it, never alone; contrast checked in every theme.",
                show: status_tags,
            },
            Example {
                id: "key-values",
                name: "Key-value grid",
                code: "shell::ui::KeyValues",
                usage: "Values that are read, not typed: several per line, the label small above. Not for forms.",
                access: "A description list: term and description.",
                show: key_values,
            },
            Example {
                id: "empty-state",
                name: "Empty state",
                code: "shell::ui::EmptyState",
                usage: "Nothing here yet: an icon, one sentence, and the way on.",
                access: "Text with a link; the icon is decoration.",
                show: empty_state,
            },
        ],
    },
    Part {
        id: "lists",
        title: "Lists and values",
        examples: &[
            Example {
                id: "facts",
                name: "Facts",
                code: ".facts",
                usage: "Documented values with their labels. In narrow places label and value share a line.",
                access: "A description list: the label is the term, the value its description.",
                show: facts,
            },
            Example {
                id: "history",
                name: "History",
                code: "shell::ui::History",
                usage: "Every change of an item with time and person, collapsed.",
                access: "details/summary and an ordered list, read in order.",
                show: history,
            },
            Example {
                id: "tables",
                name: "Tables",
                code: ".table",
                usage: "Numbers to compare across rows (patients per UHS). Not for layout. Where a table has more columns than a phone shows, a list of cards takes its place there (only-wide, only-narrow).",
                access: "A real table with header cells; numbers in tabular figures.",
                show: table,
            },
        ],
    },
];

/// The page's ID of a part or an entry: prefixed, so it never meets an ID the examples use
/// (the undo bar's `undo`).
fn anchor(id: &str) -> String {
    format!("gallery-{id}")
}

/// Every part: the shell's, then each package's in launcher order.
fn parts() -> impl Iterator<Item = &'static Part> {
    SHELL.iter().chain(
        crate::shell::registry::PACKAGES
            .iter()
            .flat_map(|p| p.gallery.iter()),
    )
}

// [impl->dsn~component-gallery~1]
/// The gallery page: a theme switch that is not stored, the parts' contents, then the parts.
#[component]
pub fn GalleryPage(
    /// The screen before, to return to.
    back: String,
) -> impl IntoView {
    let theme = RwSignal::new(stored_theme());
    // The device's own theme comes back when the page closes.
    on_cleanup(|| show_theme(&stored_theme()));
    provide_context(Nav::inert());
    let undo = UndoSlot(RwSignal::new(None));
    view! {
        <a class="skip-link" href="#main">{text::skip_link()}</a>
        <main id="main" class="page gallery" tabindex="-1" lang="en">
            <a class="back-link" href=back lang="de"><span aria-hidden="true">"←"</span>{text::notices_back()}</a>
            <h1>"Component gallery"</h1>
            <p>
                "Every building block of ORDO with sample data, in the theme chosen below. "
                "Each says when to use it and what it does for accessibility (architecture 8.6, ADR-0094)."
            </p>
            <div class="gallery__theme field" lang="de">
                <crate::shell::ui::SettingSelect id="gallery-theme" label=text::appearance()
                    options=&*text::THEMES value=theme apply=show_theme />
            </div>
            <nav aria-labelledby="gallery-contents">
                <h2 id="gallery-contents" class="gallery__contents-title">"Contents"</h2>
                <ul class="gallery__contents">
                    {parts().map(|p| view! { <li><a href=format!("#{}", anchor(p.id))>{p.title}</a></li> }).collect_view()}
                </ul>
            </nav>
            {parts().map(|part| view! {
                <section class="gallery__part" aria-labelledby=anchor(part.id)>
                    <h2 id=anchor(part.id)>{part.title}</h2>
                    {part.examples.iter().map(|e| {
                        let (title, labelled) = (format!("{}-title", anchor(e.id)), format!("{}-title", anchor(e.id)));
                        view! {
                            <article class="card gallery__entry" id=anchor(e.id) aria-labelledby=labelled>
                                <h3 id=title>{e.name}</h3>
                                <p class="gallery__code"><code>{e.code}</code></p>
                                <dl class="gallery__notes">
                                    <div><dt>"Use"</dt><dd>{e.usage}</dd></div>
                                    <div><dt>"Accessibility"</dt><dd>{e.access}</dd></div>
                                </dl>
                                <div class="gallery__example" lang="de">{(e.show)()}</div>
                            </article>
                        }
                    }).collect_view()}
                </section>
            }).collect_view()}
        </main>
        <UndoBar offer=undo />
    }
}

/// The token names of a token file whose path starts with `group` (e.g. `color`), as CSS
/// names without the `--ordo-` prefix (`color-text-primary`).
fn token_names(json: &str, group: &str) -> Vec<String> {
    fn walk(value: &serde_json::Value, path: &mut Vec<String>, out: &mut Vec<String>) {
        let Some(map) = value.as_object() else {
            return;
        };
        if map.contains_key("$value") {
            out.push(path.join("-"));
            return;
        }
        for (key, child) in map.iter().filter(|(k, _)| !k.starts_with('$')) {
            path.push(kebab(key));
            walk(child, path, out);
            path.pop();
        }
    }
    let value: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    let mut out = Vec::new();
    walk(&value, &mut Vec::new(), &mut out);
    out.retain(|n| n.split('-').next() == Some(group));
    out
}

/// `touchTarget` → `touch-target`, as the CSS of the tokens names it.
fn kebab(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

const THEME: &str = include_str!("../../../../design/themes/day.tokens.json");
const PRIMITIVE: &str = include_str!("../../../../design/tokens/primitive.tokens.json");

fn colours() -> AnyView {
    view! {
        <ul class="gallery__swatches">
            {token_names(THEME, "color").into_iter().map(|name| {
                let colour = format!("var(--ordo-{name})");
                view! {
                    <li class="gallery__swatch">
                        <span class="gallery__colour" style:background=colour></span>
                        <code>{name}</code>
                    </li>
                }
            }).collect_view()}
        </ul>
    }
    .into_any()
}

fn type_sizes() -> AnyView {
    view! {
        <ul class="gallery__scale">
            {token_names(PRIMITIVE, "font").into_iter().filter(|n| n.starts_with("font-size")).map(|name| view! {
                <li>
                    <code>{name.clone()}</code>
                    <span style:font-size=format!("var(--ordo-{name})")>"Sichtung SK I · 14:05"</span>
                </li>
            }).collect_view()}
        </ul>
    }
    .into_any()
}

fn spacing() -> AnyView {
    view! {
        <ul class="gallery__scale">
            {token_names(PRIMITIVE, "space").into_iter().filter(|n| n != "space-0").map(|name| view! {
                <li>
                    <code>{name.clone()}</code>
                    <span class="gallery__space" style:inline-size=format!("var(--ordo-{name})")></span>
                </li>
            }).collect_view()}
        </ul>
        <ul class="gallery__radii">
            {token_names(PRIMITIVE, "radius").into_iter().map(|name| {
                let radius = format!("var(--ordo-{name})");
                view! { <li class="gallery__radius" style:border-radius=radius><code>{name}</code></li> }
            }).collect_view()}
        </ul>
    }
    .into_any()
}

/// What each icon means (the icon rules, architecture 8.6); the frame's own icons say where
/// they sit.
pub(crate) fn icon_meaning(name: &str) -> &'static str {
    match name {
        "ambulance" => "transports",
        "app-window" => "workspace",
        "arrow-right-left" => "hand over a patient",
        "ban" => "a unit not available",
        "bell" => "what needs attention",
        "camera" => "take a photo",
        "chart-column" => "figures, reports",
        "chevron-right" => "opens its dialog (an object chip)",
        "circle-check" => "a unit available",
        "clipboard-list" => "Aufträge, Disposition",
        "cloud-check" => "connected and sent",
        "cloud-off" => "offline",
        "cloud-upload" => "sending",
        "crosshair" => "my position",
        "ellipsis" => "more actions",
        "file-down" => "download a document",
        "flag" => "the incident",
        "hospital" => "a hospital, a transport's destination",
        "id-card" => "people (Personal)",
        "info" => "details",
        "keyboard" => "keyboard shortcuts",
        "key-round" => "access, roles",
        "layout-grid" => "menu, overview",
        "log-in" => "log in",
        "log-out" => "log out, left the incident",
        "map" => "situation map",
        "map-pin" => "show on the map",
        "mouse-pointer-2" => "select on the map",
        "network" => "command structure",
        "notebook-text" => "incident log",
        "pencil" => "edit",
        "pentagon" => "draw an area",
        "plus" => "add",
        "qr-code" => "show a code to scan",
        "refresh-cw" => "load again",
        "scan-line" => "scan a code",
        "search" => "search, command palette",
        "spline" => "draw a line",
        "tablet-smartphone" => "devices",
        "tent" => "a UHS",
        "trash-2" => "remove or dissolve",
        "triangle-alert" => "a warning to check",
        "truck" => "a unit",
        "undo-2" => "undo",
        "upload" => "upload",
        "users" => "patients, crews",
        "x" => "withdraw or close",
        "zap" => "quick access",
        _ => "",
    }
}

fn icons() -> AnyView {
    view! {
        <ul class="gallery__icons">
            {ICONS.iter().map(|name| view! {
                <li><Icon name=name /><code>{*name}</code><span class="hint" lang="en">{icon_meaning(name)}</span></li>
            }).collect_view()}
        </ul>
    }
    .into_any()
}

fn buttons() -> AnyView {
    view! {
        <div class="row">
            <button type="button" class="button button--primary">"Registrieren"</button>
            <button type="button" class="button">{text::save()}</button>
            <button type="button" class="button button--danger">"Einsatz abschließen"</button>
            <button type="button" class="button" disabled>"Arzt/Ärztin anfordern"</button>
        </div>
        <div class="row">
            <button type="button" class="button button--small button--with-icon"><Icon name="plus" />"Vitalwerte"</button>
            <button type="button" class="button button--small">"Verladen"</button>
            <a class="button button--small" href="#gallery-buttons"><Icon name="map" />"Karte bearbeiten"</a>
            <button type="button" class="button button--small button--quiet">{text::cancel()}</button>
        </div>
    }
    .into_any()
}

fn icon_buttons() -> AnyView {
    view! {
        <div class="row">
            <IconButton icon="map-pin" word="Karte" label="Auf der Karte zeigen: RTW 83/1" on_click=Callback::new(|()| {}) />
            <IconButton icon="pencil" word=text::edit() label="Bearbeiten: RTW 83/1" on_click=Callback::new(|()| {}) />
            <IconButton icon="trash-2" word="Entfernen" label="Entfernen: RTW 83/1" on_click=Callback::new(|()| {}) />
            <IconButton icon="x" word=text::close() label="Schließen: RTW 83/1" quiet=true on_click=Callback::new(|()| {}) />
        </div>
    }
    .into_any()
}

fn action_rows() -> AnyView {
    let none = || Callback::new(|()| {});
    view! {
        <Actions actions=vec![
            crate::shell::ui::Action { text: "Verladen", label: "Verladen: KTW 2".into(), quiet: false, run: none() },
            crate::shell::ui::Action { text: "Abgefahren", label: "Abgefahren: KTW 2".into(), quiet: false, run: none() },
            crate::shell::ui::Action { text: "Stornieren", label: "Stornieren: KTW 2".into(), quiet: true, run: none() },
        ] />
    }
    .into_any()
}

fn edit_dialog() -> AnyView {
    let open = RwSignal::new(false);
    view! {
        <div class="row">
            <EditButton id="gallery-edit" label="Ort und Personal: UHS Eingang" icon="pencil"
                word="Ort und Personal" on_click=Callback::new(move |()| open.set(true)) />
        </div>
        <EditDialog id="gallery-dialog" open on_close=Callback::new(move |()| open.set(false))
            title="UHS Eingang bearbeiten".to_owned() back="gallery-edit".to_owned()>
            <div class="edit-dialog__part">
                <h3>"Angaben"</h3>
                <div class="field">
                    <label for="gallery-dialog-name">"Name der UHS"</label>
                    <input type="text" id="gallery-dialog-name" value="UHS Eingang" />
                </div>
                <div class="field field--small">
                    <label for="gallery-dialog-capacity">"Kapazität (Patienten)"</label>
                    <input type="number" id="gallery-dialog-capacity" value="10" />
                </div>
            </div>
        </EditDialog>
    }
    .into_any()
}

fn undo() -> AnyView {
    view! {
        <div class="row">
            <IconButton icon="trash-2" word="Entfernen" label="Entfernen: Sammelstelle"
                on_click=Callback::new(|()| offer_undo("Sammelstelle entfernt.".to_owned(), || {})) />
        </div>
    }
    .into_any()
}

fn text_fields() -> AnyView {
    view! {
        <div class="row">
            <div class="field">
                <label for="gallery-call-sign">"Funkrufname"</label>
                <input type="text" id="gallery-call-sign" aria-describedby="gallery-call-sign-hint" />
                <p class="hint" id="gallery-call-sign-hint">"Wie über Funk gerufen, z. B. RTW 83/1."</p>
            </div>
            <div class="field field--invalid">
                <label for="gallery-tag">"Nummer der Patientenanhängekarte"</label>
                <input type="text" class="input--code" id="gallery-tag" value="DE-0042"
                    aria-invalid="true" aria-describedby="gallery-tag-error" />
                <p class="field-error" id="gallery-tag-error">"Diese Nummer hat schon ein anderer Patient."</p>
            </div>
            <div class="field field--small">
                <label for="gallery-count">"Kapazität"</label>
                <input type="number" id="gallery-count" value="10" />
            </div>
        </div>
    }
    .into_any()
}

fn selects() -> AnyView {
    let people = Signal::derive(|| vec!["Anna Schmidt".to_owned(), "Ben Weber".to_owned()]);
    view! {
        <div class="row">
            <div class="field">
                <label for="gallery-select">"Ziel"</label>
                <select id="gallery-select">
                    <option value="">"Ziel wählen"</option>
                    <option>"Klinikum Mitte"</option>
                </select>
            </div>
            <div class="field">
                <label for="gallery-person">"Angeordnet von"</label>
                <input type="text" id="gallery-person" list="gallery-people" />
                <Suggestions id="gallery-people" values=people />
            </div>
        </div>
        <div class="field">
            <label for="gallery-text">"Beschreibung"</label>
            <textarea id="gallery-text" rows="3">"Stadtfest in der Altstadt, drei Tage."</textarea>
        </div>
    }
    .into_any()
}

fn chips() -> AnyView {
    let way = RwSignal::new("radio".to_owned());
    let wards = RwSignal::new(BTreeSet::from(["trauma".to_owned()]));
    let off = Signal::derive(|| BTreeSet::from(["burns".to_owned()]));
    view! {
        <RadioChips legend="Übermittelt" name="gallery-way" value=way options=vec![
            ("person".into(), "persönlich".into()),
            ("radio".into(), "per Funk".into()),
            ("phone".into(), "per Telefon".into()),
        ] />
        <CheckChips legend="Fachabteilungen" value=wards disabled=off options=vec![
            ("trauma".into(), "Unfallchirurgie".into()),
            ("cardio".into(), "Kardiologie".into()),
            ("burns".into(), "Brandverletztenzentrum".into()),
        ] />
    }
    .into_any()
}

fn check_box() -> AnyView {
    view! {
        <label class="choice">
            <input type="checkbox" />
            "Stärke ausführlich angeben: Führer von Verbänden und Zugführer getrennt"
        </label>
    }
    .into_any()
}

fn notices() -> AnyView {
    view! {
        <p class="notice">"Patient in der Liste wählen, um die Akte zu öffnen."</p>
        <p class="notice notice--warning"><Icon name="triangle-alert" />"Diese Karte wurde zweimal registriert."</p>
        <p class="error" role="alert">"Keine Verbindung zum Server. Bitte erneut versuchen."</p>
        <p class="hint">"Foto vom QR-Code einer Schnellübergabe."</p>
    }
    .into_any()
}

fn tags() -> AnyView {
    view! {
        <div class="row">
            <span class="state-tag tag" data-state="planned">"Geplant"</span>
            <span class="state-tag tag" data-state="active">"Laufend"</span>
            <span class="state-tag tag" data-state="paused">"Unterbrochen"</span>
            <span class="tag">"3 Objekte"</span>
            <span class="tag tag--warning">"2 fällig"</span>
        </div>
    }
    .into_any()
}

fn ago() -> AnyView {
    let at = crate::core::now_ms().saturating_sub(3 * 60 * 1000);
    view! { <p>"Herzfrequenz 92 /min " <Ago at /></p> }.into_any()
}

fn cards() -> AnyView {
    view! {
        <section class="card" aria-labelledby="gallery-card-title">
            <h3 id="gallery-card-title">"Freie Transportmittel"</h3>
            <p class="hint">"Keine freien Transportmittel."</p>
        </section>
    }
    .into_any()
}

fn disclosure() -> AnyView {
    let open = RwSignal::new(false);
    view! {
        <Disclosure id="gallery-more" open class="gallery__disclosure".to_owned()
            button_class="button button--small button--with-icon" panel_class="gallery__panel"
            button=Box::new(|| view! { <Icon name="ellipsis" />"Weitere Aktionen" }.into_any())>
            <p>"Aus dem Einsatz entlassen, Status ändern."</p>
        </Disclosure>
    }
    .into_any()
}

fn tabs() -> AnyView {
    use crate::shell::ui::{Column, Part, Parts, TabsMode};
    let part = |key, label: &str, short: Option<&str>, column, text: &'static str| Part {
        key,
        label: label.to_owned(),
        short: short.map(str::to_owned),
        column,
        content: view! { <section class="card"><p>{text}</p></section> }.into_any(),
    };
    let tasks = RwSignal::new("helpers");
    let together = RwSignal::new("requests");
    view! {
        <p class="hint">"Separate tasks: always tabs."</p>
        <Parts id="gallery-tabs" label="Beispiel" chosen=tasks mode=TabsMode::Always parts=vec![
            part("helpers", "Helfende", None, Column::Start, "12 Helfende im Einsatz."),
            part("devices", "Geräte", None, Column::Start, "4 Geräte gekoppelt."),
        ] />
        <p class="hint">"Parts used together: columns from 60rem, tabs below."</p>
        <Parts id="gallery-columns" label="Beispiel" chosen=together mode=TabsMode::ColumnsFrom60 parts=vec![
            part("requests", "Transporte", None, Column::Start, "RTW für UHS Eingang, 18:26."),
            part("free", "Freie Transportmittel", Some("Freie Mittel"), Column::End, "RTW 83/1 · einsatzbereit."),
            part("hospitals", "Kliniken", None, Column::End, "Klinikum Mitte: SK I 2 · SK II 3."),
        ] />
    }
    .into_any()
}

fn facts() -> AnyView {
    view! {
        <dl class="facts">
            <div><dt>"Herzfrequenz"</dt><dd>"92 /min"</dd></div>
            <div><dt>"SpO₂"</dt><dd>"97 %"</dd></div>
            <div><dt>"GCS gesamt"</dt><dd>"15"</dd></div>
        </dl>
    }
    .into_any()
}

fn history() -> AnyView {
    let entries = Signal::derive(|| {
        vec![
            (
                "16:11".to_owned(),
                "Angefordert".to_owned(),
                "Anna Schmidt".to_owned(),
            ),
            (
                "16:13".to_owned(),
                "Zugewiesen".to_owned(),
                "Ben Weber".to_owned(),
            ),
        ]
    });
    view! { <History entries /> }.into_any()
}

fn section_head() -> AnyView {
    view! {
        <section aria-labelledby="gallery-sh-orders">
            <SectionHead id="gallery-sh-orders" icon="clipboard-list" title="Anordnungen" count=Signal::derive(|| 2)>
                <button type="button" class="button button--small"><Icon name="plus" />"Anordnung"</button>
            </SectionHead>
            <p class="hint">"Zwei Einträge …"</p>
            <SectionHead id="gallery-sh-vitals" title="Vitalwerte" />
            <p class="hint">"Ein Eintrag …"</p>
        </section>
    }
    .into_any()
}

fn object_chips() -> AnyView {
    let none = || Callback::new(|()| {});
    let step = |text: &'static str, quiet: bool| Action {
        text,
        label: format!("{text}: Patient: Bühne Nord"),
        quiet,
        run: Callback::new(|()| {}),
    };
    view! {
        <div class="task-card">
            <p class="task-card__what"><strong>"Bühne Nord"</strong> " – Person liegt am Boden"</p>
            <div class="object-chips">
                <ObjectChip icon="map-pin" value="Auf der Karte" label="Pin: Patient: Bühne Nord" on_click=none() />
                <ObjectChip icon="hospital" value="Ziel setzen" empty=true label="Ziel: RTW für UHS Eingang" on_click=none() />
            </div>
            <MoreActions id="gallery-more" label="Weitere Schritte: Patient: Bühne Nord"
                actions=vec![step("In Arbeit", false), step("Erledigt", false), step("In 10 Minuten nachfragen", false), step("Abbrechen", true)] />
        </div>
    }
    .into_any()
}

fn item_rows() -> AnyView {
    let more = || {
        view! { <IconButton icon="ellipsis" word="Mehr" label="Mehr: Paracetamol" on_click=Callback::new(|()| {}) /> }.into_any()
    };
    view! {
        <ul class="item-rows">
            <ItemRow time="15:51" status=view! { <StatusTag tone=Tone::Ok icon="circle-check" text="durchgeführt" /> }.into_any() actions=more()>
                <strong>"Paracetamol 1 g i.v."</strong> " · Dr. Lang"
            </ItemRow>
            <ItemRow time="15:52" status=view! { <StatusTag tone=Tone::Attention icon="triangle-alert" text="zu bestätigen" /> }.into_any() actions=more()>
                <strong>"Novaminsulfon 1 g i.v."</strong> " · per Funk"
            </ItemRow>
            <ItemRow time="15:51" actions=view! { <IconButton icon="pencil" word="Bearbeiten" label="Bearbeiten: Vitalwerte 15:51" on_click=Callback::new(|()| {}) /> }.into_any()>
                "HF " <strong>"92"</strong> " · SpO₂ " <strong>"97 %"</strong> " · GCS " <strong>"15"</strong> <span class="hint">" · Anna Schmidt"</span>
            </ItemRow>
        </ul>
    }
    .into_any()
}

fn status_tags() -> AnyView {
    view! {
        <div class="row">
            <StatusTag tone=Tone::Ok icon="circle-check" text="einsatzbereit" />
            <StatusTag tone=Tone::Attention icon="triangle-alert" text="Rückfrage fällig" />
            <StatusTag tone=Tone::Urgent icon="triangle-alert" text="Sofort" />
            <StatusTag tone=Tone::Neutral text="geplant" />
        </div>
    }
    .into_any()
}

fn key_values() -> AnyView {
    let items = [
        ("Name", "Mustermann, Max"),
        ("Alter", "40 J. (geschätzt)"),
        ("Geschlecht", "männlich"),
        ("Allergien", "Penicillin"),
    ]
    .map(|(k, v)| (k.to_owned(), v.to_owned()))
    .to_vec();
    view! { <KeyValues items /> }.into_any()
}

fn empty_state() -> AnyView {
    view! {
        <EmptyState icon="tent" text="Noch keine UHS.">
            <a href="#gallery-empty-state">"UHS anlegen"</a>
        </EmptyState>
    }
    .into_any()
}

fn table() -> AnyView {
    view! {
        <table class="table">
            <thead><tr><th scope="col">"UHS"</th><th scope="col" class="number">"Belegung"</th></tr></thead>
            <tbody>
                <tr><th scope="row">"UHS Bühne Nord"</th><td class="number">"3 / 10"</td></tr>
                <tr><th scope="row">"UHS Eingang"</th><td class="number">"1 / 5"</td></tr>
            </tbody>
        </table>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~component-gallery~1]
    #[test]
    fn every_icon_has_one_meaning_and_every_anchor_is_unique() {
        for name in ICONS {
            assert!(!icon_meaning(name).is_empty(), "{name} has no meaning");
        }
        let mut ids = BTreeSet::new();
        for part in parts() {
            assert!(ids.insert(part.id), "{} twice", part.id);
            for e in part.examples {
                assert!(ids.insert(e.id), "{} twice", e.id);
                assert!(!e.usage.is_empty() && !e.access.is_empty(), "{}", e.id);
            }
        }
    }

    #[test]
    fn the_token_names_are_those_of_the_css() {
        let colours = token_names(THEME, "color");
        assert!(colours.contains(&"color-text-primary".to_owned()));
        assert!(colours.contains(&"color-action-primary-background-hover".to_owned()));
        let sizes = token_names(PRIMITIVE, "size");
        assert!(sizes.contains(&"size-touch-target".to_owned()));
    }
}
