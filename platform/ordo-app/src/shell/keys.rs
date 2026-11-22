//! Vim keys for the whole app (ADR-0090, UX-15): turned on per device in the menu; then,
//! while the focus is not in a text field, single keys move between items, tabs and views,
//! open the search, the palette and the menu, and scroll. No key changes data. The key help
//! (`?`, and *Tastenkürzel* in the menu) lists them in both settings.

use leptos::prelude::*;
use wasm_bindgen::JsCast;

use crate::shell::ui::EditDialog;
use crate::text;

/// The device setting (`""` = standard, `"vim"`).
pub(crate) const KEYMAP_KEY: &str = "ordo.keymap";

/// What a key asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Command {
    Help,
    Palette,
    Search,
    Down,
    Up,
    Left,
    Right,
    First,
    Last,
    NextTab,
    PreviousTab,
    /// The n-th view of the workspace, from 1.
    View(u8),
    Menu,
    Open,
    Insert,
    ScrollDown,
    ScrollUp,
}

/// The keys typed so far: vim's `g` prefix.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct Keys {
    /// When `g` was pressed (ms), while it waits for the second key.
    g: Option<f64>,
}

/// How long `g` waits for its second key.
const PREFIX_MS: f64 = 1000.0;

impl Keys {
    // [impl->dsn~vim-keys~2]
    /// The command of `key` pressed at `now` (ms), if any; `g` waits for the next key.
    pub(crate) fn press(&mut self, key: &str, now: f64) -> Option<Command> {
        let prefixed = self.g.take().is_some_and(|at| now - at <= PREFIX_MS);
        if prefixed {
            return match key {
                "g" => Some(Command::First),
                "t" => Some(Command::NextTab),
                "T" => Some(Command::PreviousTab),
                "m" => Some(Command::Menu),
                "1" | "2" | "3" | "4" => key.parse().ok().map(Command::View),
                _ => None,
            };
        }
        match key {
            "g" => {
                self.g = Some(now);
                None
            }
            "?" => Some(Command::Help),
            ":" => Some(Command::Palette),
            "/" => Some(Command::Search),
            "j" => Some(Command::Down),
            "k" => Some(Command::Up),
            "h" => Some(Command::Left),
            "l" => Some(Command::Right),
            "G" => Some(Command::Last),
            "o" => Some(Command::Open),
            "i" => Some(Command::Insert),
            "d" => Some(Command::ScrollDown),
            "u" => Some(Command::ScrollUp),
            _ => None,
        }
    }

    /// Whether `g` waits for its second key.
    pub(crate) fn waiting(&self) -> bool {
        self.g.is_some()
    }
}

/// The keys and what they do, for the key help: (keys, meaning).
pub(crate) const HELP: &[(&str, crate::shell::nav::Label)] = &[
    ("?", text::key_help),
    (":", text::key_palette),
    ("/", text::key_search),
    ("h j k l", text::key_move),
    ("g g / G", text::key_first_last),
    ("g t / g T", text::key_tabs),
    ("g 1 … g 4", text::key_views),
    ("g m", text::key_menu),
    ("o", text::key_open),
    ("i", text::key_insert),
    ("d / u", text::key_scroll),
    ("Esc", text::key_escape),
];

/// The keys that work in both settings (ADR-0030, ADR-0051).
pub(crate) const ALWAYS: &[(&str, crate::shell::nav::Label)] = &[
    ("Ctrl+K", text::key_palette),
    ("j / k, ↓ / ↑", text::key_in_list),
    ("Esc", text::key_close),
];

/// Whether vim keys are on for this device.
pub(crate) fn on() -> bool {
    root()
        .and_then(|r| r.get_attribute("data-keymap"))
        .as_deref()
        == Some("vim")
}

fn document() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

fn root() -> Option<web_sys::Element> {
    document()?.document_element()
}

/// Whether the focus is where keys type: a text field, a select or an editable area.
fn typing(active: Option<&web_sys::Element>) -> bool {
    let Some(el) = active else { return false };
    match el.tag_name().as_str() {
        "TEXTAREA" | "SELECT" => true,
        "INPUT" => {
            let kind = el.get_attribute("type").unwrap_or_default();
            !matches!(
                kind.as_str(),
                "checkbox" | "radio" | "button" | "submit" | "reset" | "file" | "range" | "color"
            )
        }
        _ => el
            .dyn_ref::<web_sys::HtmlElement>()
            .is_some_and(web_sys::HtmlElement::is_content_editable),
    }
}

/// Whether a modal dialog is open (the key help aside).
fn dialog_open() -> bool {
    document()
        .and_then(|d| {
            d.query_selector("dialog[open]:not(#keys-dialog)")
                .ok()
                .flatten()
        })
        .is_some()
}

fn query_all(selector: &str) -> Vec<web_sys::HtmlElement> {
    let Some(list) = document().and_then(|d| d.query_selector_all(selector).ok()) else {
        return Vec::new();
    };
    (0..list.length())
        .filter_map(|i| list.item(i)?.dyn_into::<web_sys::HtmlElement>().ok())
        .filter(|e| e.offset_parent().is_some())
        .collect()
}

/// A box on the page, in CSS pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    fn right(self) -> f64 {
        self.x + self.w
    }
    fn bottom(self) -> f64 {
        self.y + self.h
    }
}

/// The gap between two ranges on one axis; 0 where they overlap.
fn gap(a0: f64, a1: f64, b0: f64, b1: f64) -> f64 {
    (b0 - a1).max(a0 - b1).max(0.0)
}

// [impl->dsn~vim-keys~2]
/// The box nearest to `from` in the direction of `command` (`Down`, `Up`, `Left`,
/// `Right`): it must lie beyond `from` that way; the distance across counts more, so the
/// move stays in its column or row where it can.
pub(crate) fn nearest(from: Rect, boxes: &[Rect], command: Command) -> Option<usize> {
    const SLACK: f64 = 2.0;
    boxes
        .iter()
        .enumerate()
        .filter(|(_, b)| **b != from)
        .filter_map(|(i, b)| {
            let across_x = gap(from.x, from.right(), b.x, b.right());
            let across_y = gap(from.y, from.bottom(), b.y, b.bottom());
            let score = match command {
                Command::Down if b.y >= from.bottom() - SLACK => {
                    (b.y - from.bottom()).max(0.0) + 2.0 * across_x
                }
                Command::Up if b.bottom() <= from.y + SLACK => {
                    (from.y - b.bottom()).max(0.0) + 2.0 * across_x
                }
                Command::Right if b.x >= from.right() - SLACK && across_y < from.h.max(b.h) => {
                    (b.x - from.right()).max(0.0) + 3.0 * across_y
                }
                Command::Left if b.right() <= from.x + SLACK && across_y < from.h.max(b.h) => {
                    (from.x - b.right()).max(0.0) + 3.0 * across_y
                }
                _ => return None,
            };
            Some((i, score))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// What the movement keys go to: links, buttons, summaries, choices, and areas that take
/// the focus; not text fields and selects, where the keys would type.
const TARGETS: &[&str] = &[
    "a[href]",
    "button:not([disabled])",
    "summary",
    "input[type=checkbox]:not([disabled])",
    "input[type=radio]:not([disabled])",
    "label.button",
    "[tabindex='0']",
];

fn rect(e: &web_sys::HtmlElement) -> Rect {
    let r = e.get_bounding_client_rect();
    Rect {
        x: r.left(),
        y: r.top(),
        w: r.width(),
        h: r.height(),
    }
}

/// The targets of the page's main content, shown, in document order.
fn targets() -> Vec<web_sys::HtmlElement> {
    let selector = TARGETS
        .iter()
        .map(|t| format!("#main {t}"))
        .collect::<Vec<_>>()
        .join(", ");
    query_all(&selector)
        .into_iter()
        .filter(|e| {
            let r = rect(e);
            r.w > 0.0
                && r.h > 0.0
                && e.closest("[hidden], dialog:not([open])")
                    .ok()
                    .flatten()
                    .is_none()
        })
        .collect()
}

/// Focuses `target`, brings it into view and lights it up briefly, so the eye finds it.
fn go(target: &web_sys::HtmlElement) {
    let options = web_sys::FocusOptions::new();
    options.set_prevent_scroll(true);
    let _ = target.focus_with_options(&options);
    let into = web_sys::ScrollIntoViewOptions::new();
    into.set_block(web_sys::ScrollLogicalPosition::Center);
    into.set_inline(web_sys::ScrollLogicalPosition::Nearest);
    target.scroll_into_view_with_scroll_into_view_options(&into);
    // One light at a time: the one before goes out.
    for lit in query_all(".keys-flash") {
        let _ = lit.class_list().remove_1("keys-flash");
    }
    let list = target.class_list();
    let _ = target.offset_width(); // restarts the animation
    let _ = list.add_1("keys-flash");
    let again = target.clone();
    let off = wasm_bindgen::closure::Closure::once_into_js(move || {
        let _ = again.class_list().remove_1("keys-flash");
    });
    if let Some(w) = web_sys::window() {
        let _ = w.set_timeout_with_callback_and_timeout_and_arguments_0(off.unchecked_ref(), 900);
    }
}

/// Moves the focus to the nearest target that way; from outside the targets, to the first
/// one (down, right) or the last (up, left).
fn step(command: Command) {
    let all = targets();
    let active = document()
        .and_then(|d| d.active_element())
        .and_then(|a| a.dyn_into::<web_sys::HtmlElement>().ok());
    let from = active.filter(|a| all.iter().any(|t| t == a));
    let target = match from {
        Some(from) => {
            let boxes: Vec<Rect> = all.iter().map(rect).collect();
            nearest(rect(&from), &boxes, command).and_then(|i| all.get(i))
        }
        None if matches!(command, Command::Down | Command::Right) => all.first(),
        None => all.last(),
    };
    if let Some(t) = target {
        go(t);
    }
}

fn scroll(by: f64) {
    if let Some(w) = web_sys::window() {
        let height = w
            .inner_height()
            .ok()
            .and_then(|h| h.as_f64())
            .unwrap_or(0.0);
        w.scroll_by_with_x_and_y(0.0, by * height / 2.0);
    }
}

/// The tab next to the chosen one of the main tab list, by `step`.
fn tab(step: i32) {
    let tabs = query_all("#main [role=tablist] [role=tab]");
    let Some(now) = tabs
        .iter()
        .position(|t| t.get_attribute("aria-selected").as_deref() == Some("true"))
    else {
        return;
    };
    let n = i32::try_from(tabs.len()).unwrap_or(1);
    let next = usize::try_from((i32::try_from(now).unwrap_or(0) + step).rem_euclid(n)).unwrap_or(0);
    if let Some(t) = tabs.get(next) {
        t.click();
        let _ = t.focus();
    }
}

/// What the commands open in the shell.
#[derive(Clone, Copy)]
pub(crate) struct Shell {
    pub palette: RwSignal<bool>,
    pub menu: RwSignal<bool>,
    pub help: RwSignal<bool>,
}

// [impl->dsn~vim-keys~2]
/// Runs `command` on the page, as a person would.
fn run(command: Command, shell: Shell) {
    match command {
        Command::Help => {
            remember_focus();
            shell.help.set(true);
        }
        Command::Palette => shell.palette.set(true),
        Command::Search => match query_all("#main input[type=search], #main [data-keys=search]")
            .into_iter()
            .next()
        {
            Some(field) => {
                let _ = field.focus();
            }
            None => shell.palette.set(true),
        },
        Command::Down | Command::Up | Command::Left | Command::Right => step(command),
        Command::First | Command::Last => {
            let all = targets();
            let target = if command == Command::First {
                all.first()
            } else {
                all.last()
            };
            if let Some(t) = target {
                go(t);
            }
        }
        Command::NextTab => tab(1),
        Command::PreviousTab => tab(-1),
        Command::View(n) => {
            if let Some(v) = query_all(".views .views__item").get(usize::from(n) - 1) {
                v.click();
            }
        }
        Command::Menu => {
            shell.palette.set(false);
            shell.menu.set(true);
            crate::shell::views::after_render(|| {
                if let Some(first) = query_all("#launcher-panel .launcher__item").first() {
                    let _ = first.focus();
                }
            });
        }
        Command::Open => {
            if let Some(a) = document()
                .and_then(|d| d.active_element())
                .and_then(|a| a.dyn_into::<web_sys::HtmlElement>().ok())
            {
                a.click();
            }
        }
        Command::Insert => {
            if let Some(field) = query_all(
                "#main input:not([type=hidden]):not([type=checkbox]):not([type=radio]):not([disabled]), #main textarea:not([disabled])",
            )
            .into_iter()
            .next()
            {
                let _ = field.focus();
            }
        }
        Command::ScrollDown => scroll(1.0),
        Command::ScrollUp => scroll(-1.0),
    }
}

/// Gives the focused element an ID, so the key help returns the focus there.
fn remember_focus() {
    if let Some(a) = document().and_then(|d| d.active_element())
        && a.id().is_empty()
        && a.tag_name() != "BODY"
    {
        a.set_id("keys-return");
    }
}

/// The element the key help returns the focus to.
fn return_id() -> String {
    document()
        .and_then(|d| d.active_element())
        .filter(|a| a.tag_name() != "BODY" && !a.id().is_empty())
        .map_or_else(|| "main".to_owned(), |a| a.id())
}

// [impl->dsn~vim-keys~2]
/// Handles a key of the window's key handler; returns whether it used it.
pub(crate) fn handle(ev: &web_sys::KeyboardEvent, keys: StoredValue<Keys>, shell: Shell) -> bool {
    if !on() || ev.ctrl_key() || ev.alt_key() || ev.meta_key() || ev.is_composing() {
        return false;
    }
    let active = document().and_then(|d| d.active_element());
    if typing(active.as_ref()) {
        // Esc leaves the field, as in vim; inside a dialog it closes the dialog instead.
        if ev.key() == "Escape" && !dialog_open() {
            if let Some(a) = active.and_then(|a| a.dyn_into::<web_sys::HtmlElement>().ok()) {
                let _ = a.blur();
            }
            return true;
        }
        return false;
    }
    if dialog_open() || shell.palette.get_untracked() {
        return false;
    }
    let now = js_sys::Date::now();
    let mut state = keys.get_value();
    let waiting = state.waiting();
    let command = state.press(&ev.key(), now);
    keys.set_value(state);
    match command {
        Some(c) => {
            ev.prevent_default();
            run(c, shell);
            true
        }
        // `g` itself, or the key after it, is used up.
        None => waiting || state.waiting(),
    }
}

/// The key help: the vim keys and the keys that always work.
#[component]
pub(crate) fn KeyHelp(
    open: RwSignal<bool>,
    /// The device setting, shared with the menu's select.
    keymap: RwSignal<String>,
) -> impl IntoView {
    let back = RwSignal::new("main".to_owned());
    Effect::new(move || {
        if open.get() {
            back.set(return_id());
        }
    });
    let table = |caption: &'static str,
                 rows: &'static [(&'static str, crate::shell::nav::Label)]| {
        view! {
            <table class="table keys-table">
                <caption>{caption}</caption>
                <thead><tr><th scope="col">{text::key_keys()}</th><th scope="col">{text::key_does()}</th></tr></thead>
                <tbody>
                    {rows.iter().map(|(keys, does)| view! {
                        <tr><td><kbd>{*keys}</kbd></td><td>{does()}</td></tr>
                    }).collect_view()}
                </tbody>
            </table>
        }
    };
    view! {
        <EditDialog id="keys" open=open on_close=Callback::new(move |()| open.set(false))
            title=Signal::derive(|| text::keys_title().to_owned()) back>
            <p>{move || if keymap.get() == "vim" { text::keys_on() } else { text::keys_off() }}</p>
            <div class="actions">
                <button id="keys-toggle" type="button" class="button" on:click=move |_| {
                    let next = if keymap.get_untracked() == "vim" { "" } else { "vim" };
                    crate::shell::views::apply_keymap(next);
                    keymap.set(next.to_owned());
                }>
                    {move || if keymap.get() == "vim" { text::keys_turn_off() } else { text::keys_turn_on() }}
                </button>
            </div>
            {table(text::keys_vim(), HELP)}
            {table(text::keys_always(), ALWAYS)}
        </EditDialog>
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, Keys};

    // [utest->dsn~vim-keys~2]
    #[test]
    fn single_keys_and_the_g_prefix_name_their_commands() {
        let mut keys = Keys::default();
        assert_eq!(keys.press("j", 0.0), Some(Command::Down));
        assert_eq!(keys.press("G", 0.0), Some(Command::Last));
        assert_eq!(keys.press("g", 0.0), None);
        assert!(keys.waiting());
        assert_eq!(keys.press("g", 500.0), Some(Command::First));
        assert!(!keys.waiting());
        assert_eq!(keys.press("g", 1000.0), None);
        assert_eq!(keys.press("t", 1200.0), Some(Command::NextTab));
        keys.press("g", 0.0);
        assert_eq!(keys.press("3", 10.0), Some(Command::View(3)));
        keys.press("g", 0.0);
        assert_eq!(keys.press("5", 10.0), None, "at most four views");
        assert_eq!(keys.press("x", 20.0), None, "an unknown key does nothing");
    }

    // [utest->dsn~vim-keys~2]
    #[test]
    fn hjkl_go_to_the_nearest_box_that_way_staying_in_column_or_row() {
        use super::{Rect, nearest};
        let r = |x: f64, y: f64| Rect {
            x,
            y,
            w: 100.0,
            h: 40.0,
        };
        // Two columns of cards and a wide button below them.
        let boxes = [
            r(0.0, 0.0),
            r(200.0, 0.0),
            r(0.0, 100.0),
            r(200.0, 100.0),
            Rect {
                x: 0.0,
                y: 200.0,
                w: 300.0,
                h: 40.0,
            },
        ];
        assert_eq!(
            nearest(boxes[0], &boxes, Command::Down),
            Some(2),
            "j: below, same column"
        );
        assert_eq!(
            nearest(boxes[0], &boxes, Command::Right),
            Some(1),
            "l: beside"
        );
        assert_eq!(
            nearest(boxes[3], &boxes, Command::Left),
            Some(2),
            "h: beside"
        );
        assert_eq!(nearest(boxes[3], &boxes, Command::Up), Some(1), "k: above");
        assert_eq!(
            nearest(boxes[3], &boxes, Command::Down),
            Some(4),
            "j: across the whole page"
        );
        assert_eq!(
            nearest(boxes[1], &boxes, Command::Right),
            None,
            "nothing further right"
        );
    }

    // [utest->dsn~vim-keys~2]
    #[test]
    fn g_waits_one_second_then_the_key_counts_on_its_own() {
        let mut keys = Keys::default();
        keys.press("g", 0.0);
        assert_eq!(keys.press("j", 1500.0), Some(Command::Down));
    }
}
