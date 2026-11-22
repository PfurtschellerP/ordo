//! Shared UI building blocks (architecture 8.6.2: own components after the WAI-ARIA
//! Authoring Practices). Views compose these instead of building each control again.

use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::shell::views::focus;

/// The icons the app ships (Lucide, `design/assets/icons/lucide`, ISC), and the SVG of each
/// by name.
macro_rules! icons {
    ($($name:literal),* $(,)?) => {
        /// Every icon's name, for the component gallery.
        pub const ICONS: &[&str] = &[$($name),*];

        /// The SVG of an icon, by name.
        fn svg(name: &str) -> &'static str {
            match name {
                $($name => include_str!(concat!("../../../../design/assets/icons/lucide/", $name, ".svg")),)*
                _ => "",
            }
        }
    };
}

icons!(
    "ambulance",
    "app-window",
    "arrow-right-left",
    "bell",
    "camera",
    "chart-column",
    "clipboard-list",
    "cloud-check",
    "cloud-off",
    "cloud-upload",
    "crosshair",
    "ellipsis",
    "file-down",
    "flag",
    "id-card",
    "info",
    "keyboard",
    "key-round",
    "layout-grid",
    "log-in",
    "log-out",
    "map",
    "map-pin",
    "mouse-pointer-2",
    "network",
    "notebook-text",
    "pencil",
    "pentagon",
    "plus",
    "qr-code",
    "refresh-cw",
    "scan-line",
    "search",
    "spline",
    "tablet-smartphone",
    "tent",
    "trash-2",
    "triangle-alert",
    "truck",
    "undo-2",
    "upload",
    "users",
    "x",
    "zap",
);

// [impl->dsn~compact-navigation~2]
/// A decorative icon: the text next to it carries the meaning (never an icon alone).
#[component]
pub fn Icon(name: &'static str) -> impl IntoView {
    view! { <span class="icon" aria-hidden="true" inner_html=svg(name)></span> }
}

// [impl->dsn~icon-rules~1]
/// A button for an action repeated on many items (the icon rules): its icon, a short word
/// where its container is wide enough, and the action with the item as accessible name and
/// as a tooltip on hover and keyboard focus.
#[component]
pub fn IconButton(
    icon: &'static str,
    /// The short word, e.g. "Bearbeiten".
    word: &'static str,
    /// The action with its item, e.g. "Bearbeiten: RTW 83/1".
    #[prop(into)]
    label: String,
    #[prop(optional)] quiet: bool,
    /// For a button that shows and hides something.
    #[prop(optional, into)]
    expanded: Option<Signal<bool>>,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button type="button" class="button button--small icon-button" class:button--quiet=quiet
            aria-label=label.clone() data-tip=label
            aria-expanded=move || expanded.map(|e| e.get().to_string())
            on:click=move |_| on_click.run(())>
            <Icon name=icon /><span class="icon-button__word" aria-hidden="true">{word}</span>
        </button>
    }
}

/// A removal that can be undone: what was removed, and how to put it back (a correcting
/// event; events are never deleted).
#[derive(Clone)]
pub struct Undo {
    pub message: String,
    /// Not a `Callback`: that would belong to the row that offered it, which a list drops
    /// as soon as the removal shows.
    pub run: std::sync::Arc<dyn Fn() + Send + Sync>,
}

/// The app's one undo offer, shown at the bottom of the page.
#[derive(Clone, Copy)]
pub struct UndoSlot(pub RwSignal<Option<Undo>>);

thread_local! {
    /// The shown undo bar's offer: click handlers run outside the component tree, where
    /// no context is found.
    static UNDO: std::cell::Cell<Option<UndoSlot>> = const { std::cell::Cell::new(None) };
}

// [impl->dsn~icon-rules~1]
/// Offers to undo a removal made by an icon button, instead of asking first.
pub fn offer_undo(message: String, run: impl Fn() + Send + Sync + 'static) {
    if let Some(slot) = UNDO.get() {
        let _ = slot.0.try_set(Some(Undo {
            message,
            run: std::sync::Arc::new(run),
        }));
    }
}

// [impl->dsn~icon-rules~1]
/// "RTW 83/1 entfernt. Rückgängig": until it is used, closed, or the next removal; announced
/// politely to screen readers.
#[component]
pub fn UndoBar(offer: UndoSlot) -> impl IntoView {
    UNDO.set(Some(offer));
    on_cleanup(|| UNDO.set(None));
    // It never stays over the page: it goes after ten seconds, or with the next press
    // anywhere else (each new offer starts its own ten seconds).
    let shown = StoredValue::new(0u64);
    Effect::new(move || {
        if offer.0.with(Option::is_some) {
            let mine = shown.get_value() + 1;
            shown.set_value(mine);
            set_timeout(
                move || {
                    if shown.try_get_value() == Some(mine) {
                        let _ = offer.0.try_set(None);
                    }
                },
                std::time::Duration::from_secs(10),
            );
        }
    });
    let outside = window_event_listener(leptos::ev::pointerdown, move |ev| {
        let inside = ev
            .target()
            .and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(t).ok())
            .and_then(|e| e.closest(".undo-bar").ok().flatten())
            .is_some();
        if !inside && offer.0.with_untracked(Option::is_some) {
            offer.0.set(None);
        }
    });
    on_cleanup(move || outside.remove());
    view! {
        <div class="undo-bar" role="status">
            {move || offer.0.get().map(|undo| {
                let run = undo.run.clone();
                view! {
                    <div class="undo-bar__inner">
                        <span>{undo.message.clone()}</span>
                        <button type="button" class="button button--small" id="undo"
                            on:click=move |_| { run(); offer.0.set(None); }>
                            <Icon name="undo-2" />{crate::text::undo()}
                        </button>
                        <button type="button" class="button button--small button--icon"
                            aria-label=crate::text::close_label(&undo.message)
                            on:click=move |_| offer.0.set(None)>
                            <Icon name="x" />
                        </button>
                    </div>
                }
            })}
        </div>
    }
}

/// A disclosure button (WAI-ARIA APG): shows and hides the panel `controls`.
#[component]
pub fn DisclosureButton(
    #[prop(into)] id: String,
    #[prop(into)] controls: String,
    open: RwSignal<bool>,
    #[prop(into)] class: String,
    /// Called before opening, e.g. to close other menus.
    #[prop(default = None)]
    on_open: Option<Callback<()>>,
    /// Accessible name when the visible content is not enough.
    #[prop(default = None)]
    label: Option<Signal<String>>,
    children: Children,
) -> impl IntoView {
    // [impl->dsn~icon-rules~1]
    // An icon button's name is its tooltip too.
    let tip = class.split_whitespace().any(|c| c == "icon-button");
    view! {
        <button
            id=id
            type="button"
            class=class
            aria-expanded=move || open.get().to_string()
            aria-controls=controls
            aria-label=move || label.map(|l| l.get())
            data-tip=move || label.filter(|_| tip).map(|l| l.get())
            on:click=move |_| {
                if !open.get_untracked()
                    && let Some(f) = on_open
                {
                    f.run(());
                }
                open.update(|o| *o = !*o);
            }
        >
            {children()}
        </button>
    }
}

// [impl->dsn~header-layout~2]
/// Closes a menu when the person clicks or taps anywhere outside the elements matching
/// `inside` (its buttons and panel), as menus do elsewhere. The focus stays where the click
/// put it. A click on an element that left the page meanwhile (the panel re-rendered)
/// does not count as outside.
pub fn close_on_click_outside(open: RwSignal<bool>, inside: String) {
    let clicks = window_event_listener(leptos::ev::click, move |ev| {
        if !open.get_untracked() {
            return;
        }
        let Some(target) = ev
            .target()
            .and_then(|t| wasm_bindgen::JsCast::dyn_into::<web_sys::Element>(t).ok())
        else {
            return;
        };
        if target.is_connected() && matches!(target.closest(&inside), Ok(None)) {
            open.set(false);
        }
    });
    on_cleanup(move || clicks.remove());
}

/// A disclosure: a button and its panel. Escape closes it wherever the focus is and
/// returns the focus to the button. IDs: `{id}-button`, `{id}-panel`.
#[component]
pub fn Disclosure(
    #[prop(into)] id: String,
    open: RwSignal<bool>,
    /// Class of the wrapper; may change with state.
    #[prop(into)]
    class: Signal<String>,
    #[prop(into)] button_class: String,
    #[prop(into)] panel_class: String,
    /// The button's content.
    button: Children,
    #[prop(default = None)] on_open: Option<Callback<()>>,
    #[prop(default = None)] label: Option<Signal<String>>,
    children: Children,
) -> impl IntoView {
    let (button_id, panel_id) = (format!("{id}-button"), format!("{id}-panel"));
    let target = button_id.clone();
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" && open.get_untracked() {
            open.set(false);
            focus(&target);
        }
    });
    on_cleanup(move || keys.remove());
    close_on_click_outside(open, format!("#{button_id}, #{panel_id}"));
    view! {
        <div class=move || class.get()>
            <DisclosureButton id=button_id controls=panel_id.clone() open class=button_class on_open label>
                {button()}
            </DisclosureButton>
            <div id=panel_id class=panel_class hidden=move || !open.get()>
                {children()}
            </div>
        </div>
    }
}

// [impl->dsn~edit-dialog~3]
/// The button that opens a list's [`EditDialog`] for one item: it names the item, says it
/// opens a dialog, and gets the focus back when the dialog closes (its ID is the dialog's
/// `back`).
#[component]
pub fn EditButton(
    #[prop(into)] id: String,
    /// The button's accessible name, with the item, e.g. "Ort und Personal: UHS Eingang".
    #[prop(into)]
    label: String,
    icon: &'static str,
    /// The visible word, e.g. "Ort und Personal".
    word: &'static str,
    on_click: Callback<()>,
) -> impl IntoView {
    view! {
        <button id=id type="button" class="button button--small button--quiet button--with-icon"
            aria-haspopup="dialog" aria-label=label on:click=move |_| on_click.run(())>
            <Icon name=icon />{word}
        </button>
    }
}

// [impl->dsn~edit-dialog~3]
/// What an [`EditDialog`] saves before it closes: each part with input typed and not yet
/// saved registers its save, which says whether the dialog may close (its check passed).
#[derive(Clone, Copy)]
pub struct PendingSaves {
    saves: StoredValue<Vec<(u64, Callback<(), bool>)>>,
    next: StoredValue<u64>,
}

impl PendingSaves {
    fn new() -> Self {
        Self {
            saves: StoredValue::new(Vec::new()),
            next: StoredValue::new(0),
        }
    }

    /// Registers `save` with the dialog around the calling component, for as long as that
    /// component lives. Outside an edit dialog it does nothing.
    pub fn register(save: Callback<(), bool>) {
        let Some(pending) = use_context::<Self>() else {
            return;
        };
        let key = pending.next.get_value();
        pending.next.set_value(key + 1);
        pending.saves.update_value(|s| s.push((key, save)));
        on_cleanup(move || {
            pending
                .saves
                .try_update_value(|s| s.retain(|(k, _)| *k != key));
        });
    }

    /// Runs every save; true when all passed.
    fn save_all(self) -> bool {
        let saves = self.saves.get_value();
        saves
            .iter()
            .fold(true, |all, (_, save)| save.run(()) && all)
    }
}

// [impl->dsn~edit-dialog~3]
/// Editing an item of a card list (pattern "edit dialog"): a modal dialog with the item's
/// form, one per list rather than one per card, so the list can update under it (new
/// events, other people's changes) without closing or rebuilding it. Header with the title
/// and a close button stays at the top, a footer with "Fertig" at the bottom; the body
/// scrolls between them, in columns on wide screens. Escape, either button or a click on
/// the backdrop closes it, after saving what was typed and not yet saved
/// ([`PendingSaves`]); a failed check keeps it open with its error in view. The focus then
/// returns to `back`. On phones it fills the screen.
#[component]
pub fn EditDialog(
    #[prop(into)] id: String,
    /// Whether it is shown.
    #[prop(into)]
    open: Signal<bool>,
    /// Closes it (the caller forgets the item).
    on_close: Callback<()>,
    /// The dialog's heading, e.g. "UHS Eingang bearbeiten".
    #[prop(into)]
    title: Signal<String>,
    /// The ID of the button that opened it.
    #[prop(into)]
    back: Signal<String>,
    children: Children,
) -> impl IntoView {
    use wasm_bindgen::JsCast;
    let dialog = NodeRef::<leptos::html::Dialog>::new();
    let title_id = format!("{id}-title");
    Effect::new(move |was: Option<bool>| {
        let now = open.get();
        if let Some(d) = dialog.get() {
            if now && !d.open() {
                let _ = d.show_modal();
            } else if !now && d.open() {
                d.close();
            }
            if was == Some(true) && !now {
                focus(&back.get_untracked());
            }
        }
        now
    });
    // Only a press and release both on the backdrop close it: not a click on the
    // scrollbar (inside the box), nor a text selection dragged out of a field.
    let outside = move |ev: &leptos::ev::MouseEvent| {
        let Some(d) = dialog.get_untracked() else {
            return false;
        };
        let d: &web_sys::Element = d.as_ref();
        let target = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok());
        let r = d.get_bounding_client_rect();
        let (x, y) = (f64::from(ev.client_x()), f64::from(ev.client_y()));
        target.is_some_and(|t| t == *d)
            && (x < r.left() || x > r.right() || y < r.top() || y > r.bottom())
    };
    let pressed_outside = StoredValue::new(false);
    let pending = PendingSaves::new();
    provide_context(pending);
    // Closing saves first; a part whose check failed keeps the dialog open, its error
    // brought into view once it is shown.
    let done = Callback::new(move |()| {
        if pending.save_all() {
            on_close.run(());
            return;
        }
        set_timeout(
            move || {
                let error = dialog.get_untracked().and_then(|d| {
                    let d: &web_sys::Element = d.as_ref();
                    d.query_selector(".error:not(:empty)").ok().flatten()
                });
                if let Some(e) = error {
                    e.scroll_into_view();
                }
            },
            std::time::Duration::ZERO,
        );
    });
    view! {
        <dialog node_ref=dialog id=format!("{id}-dialog") class="edit-dialog" aria-labelledby=title_id.clone()
            on:cancel=move |ev: leptos::ev::Event| {
                ev.prevent_default();
                done.run(());
            }
            on:close=move |_| {
                if open.get_untracked() {
                    on_close.run(());
                }
            }
            on:pointerdown=move |ev: leptos::ev::PointerEvent| pressed_outside.set_value(outside(&ev))
            on:click=move |ev| {
                if pressed_outside.get_value() && outside(&ev) {
                    done.run(());
                }
                pressed_outside.set_value(false);
            }
        >
            <header class="edit-dialog__header">
                <h2 id=title_id.clone() class="edit-dialog__title">{move || title.get()}</h2>
                <IconButton icon="x" word=crate::text::close() quiet=true label=crate::text::close()
                    on_click=done />
            </header>
            <div class="edit-dialog__body">{children()}</div>
            <footer class="edit-dialog__footer">
                <button type="button" class="button" on:click=move |_| done.run(())>{crate::text::done()}</button>
            </footer>
        </dialog>
    }
}

// [impl->dsn~record-overview~3]
/// The tabs of a page (WAI-ARIA tabs pattern): one tab per part, the chosen one pressed;
/// arrow keys, Home and End move between them. The parts are [`TabPanel`]s with the same
/// `id` and keys.
#[component]
pub fn TabList(
    #[prop(into)] id: String,
    /// What the tabs are, for screen readers, e.g. "Bereiche der Akte".
    #[prop(into)]
    label: String,
    /// Each tab, in order.
    tabs: Vec<Tab>,
    chosen: RwSignal<&'static str>,
) -> impl IntoView {
    let keys: Vec<&'static str> = tabs.iter().map(|(k, ..)| *k).collect();
    let tab_id = {
        let id = id.clone();
        move |key: &str| format!("{id}-tab-{key}")
    };
    let keydown = {
        let (keys, tab_id) = (keys.clone(), tab_id.clone());
        move |ev: leptos::ev::KeyboardEvent| {
            let Some(at) = keys.iter().position(|k| *k == chosen.get_untracked()) else {
                return;
            };
            let next = match ev.key().as_str() {
                "ArrowRight" => (at + 1) % keys.len(),
                "ArrowLeft" => (at + keys.len() - 1) % keys.len(),
                "Home" => 0,
                "End" => keys.len() - 1,
                _ => return,
            };
            ev.prevent_default();
            chosen.set(keys[next]);
            focus(&tab_id(keys[next]));
        }
    };
    view! {
        <div role="tablist" class="tabs" aria-label=label on:keydown=keydown>
            {tabs
                .into_iter()
                .map(|(key, text, short)| {
                    let on = move || chosen.get() == key;
                    // On phones the short label shows instead; the name is what is seen.
                    let label = match short {
                        Some(short) => view! {
                            <span class="tabs__long">{text}</span><span class="tabs__short">{short}</span>
                        }.into_any(),
                        None => text.into_any(),
                    };
                    view! {
                        <button type="button" role="tab" class="tabs__tab" id=tab_id(key)
                            aria-selected=move || on().to_string()
                            aria-controls=format!("{id}-panel-{key}")
                            tabindex=move || if on() { "0" } else { "-1" }
                            on:click=move |_| chosen.set(key)>
                            {label}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// One tab of a [`TabList`]: its key, its label, and a shorter label for phones, if the
/// label is long (e.g. "Doku" for "Dokumentation").
pub type Tab = (&'static str, String, Option<String>);

/// Where a part stands when its tabs become columns ([`Parts`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Column {
    Start,
    End,
    Below,
}

/// How [`Parts`] shows its parts (chapter 8.6.5, *Tabs*): always in tabs (separate tasks),
/// or side by side from a width and in tabs below it (parts used together).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabsMode {
    Always,
    /// Columns from 60rem, else tabs.
    ColumnsFrom60,
    /// Columns from 84rem (a part needs 40rem, e.g. the org chart), else tabs.
    ColumnsFrom84,
}

/// One part of [`Parts`]: its key, label, short label, column and content.
pub struct Part {
    pub key: &'static str,
    pub label: String,
    pub short: Option<String>,
    pub column: Column,
    pub content: AnyView,
}

/// The chosen part of the page's [`Parts`], for its sections: an action that needs
/// another part shows it first (e.g. *Auf der Karte setzen*). A Leptos context.
#[derive(Clone, Copy)]
pub struct PageTabs(pub RwSignal<&'static str>);

impl PageTabs {
    /// Shows the part `key`.
    pub fn show(self, key: &'static str) {
        self.0.set(key);
    }
}

// [impl->dsn~page-tabs~4]
/// A page's parts in tabs, or side by side where `mode` allows and there is room (chapter
/// 8.6.5, *Tabs*). Hidden parts stay in the page, so their input survives; the tab list is
/// [`TabList`] (WAI-ARIA tabs).
#[component]
pub fn Parts(
    #[prop(into)] id: String,
    /// What the tabs are, for screen readers.
    #[prop(into)]
    label: String,
    parts: Vec<Part>,
    chosen: RwSignal<&'static str>,
    mode: TabsMode,
) -> impl IntoView {
    let tabs = parts
        .iter()
        .map(|p| (p.key, p.label.clone(), p.short.clone()))
        .collect();
    let class = match mode {
        TabsMode::Always => "parts parts--always",
        TabsMode::ColumnsFrom60 => "parts parts--columns parts--from-60",
        TabsMode::ColumnsFrom84 => "parts parts--columns parts--from-84",
    };
    let panel = {
        let id = id.clone();
        move |part: Part| {
            let key = part.key;
            view! {
                <div role="tabpanel" id=format!("{id}-panel-{key}") aria-labelledby=format!("{id}-tab-{key}")
                    class="parts__part" class:parts__part--chosen=move || chosen.get() == key>
                    {part.content}
                </div>
            }
        }
    };
    let mut columns: [Vec<AnyView>; 3] = Default::default();
    for part in parts {
        let at = match part.column {
            Column::Start => 0,
            Column::End => 1,
            Column::Below => 2,
        };
        columns[at].push(panel(part).into_any());
    }
    let [start, end, below] = columns;
    view! {
        <div class=class>
            <TabList id=id.clone() label tabs chosen />
            <div class="parts__grid">
                <div class="parts__column">{start}</div>
                <div class="parts__column">{end}</div>
                {below}
            </div>
        </div>
    }
}

/// One part of a page with [`TabList`]: shown while its tab is chosen. Hidden parts stay
/// on the page, so unsaved input survives switching (ADR-0029).
#[component]
pub fn TabPanel(
    #[prop(into)] id: String,
    key: &'static str,
    chosen: RwSignal<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <div role="tabpanel" class="tabs__panel" id=format!("{id}-panel-{key}")
            aria-labelledby=format!("{id}-tab-{key}") hidden=move || chosen.get() != key>
            {children()}
        </div>
    }
}

// [impl->dsn~record-overview~3]
/// How long ago `at` was, e.g. "(vor 59 Min.)", counted on every 30 seconds, so a value's
/// age is seen at a glance. Not announced as it changes.
#[component]
pub fn Ago(at: u64) -> impl IntoView {
    let now = RwSignal::new(crate::core::now_ms());
    if let Ok(tick) = set_interval_with_handle(
        move || now.set(crate::core::now_ms()),
        std::time::Duration::from_secs(30),
    ) {
        on_cleanup(move || tick.clear());
    }
    view! {
        <span class="ago">{move || format!("({})", crate::text::ago(now.get().saturating_sub(at)))}</span>
    }
}

/// One entry of a history: time, what happened, who.
pub type HistoryEntry = (String, String, String);

/// The history of a record (status changes): collapsed, every entry with time and person.
#[component]
pub fn History(#[prop(into)] entries: Signal<Vec<HistoryEntry>>) -> impl IntoView {
    view! {
        <details class="history-list">
            <summary>{crate::text::transport_history()}</summary>
            <ol>
                {move || {
                    entries
                        .get()
                        .into_iter()
                        .map(|(at, what, by)| view! { <li>{at} " " {what} " · " {by}</li> })
                        .collect_view()
                }}
            </ol>
        </details>
    }
}

/// A button that moves something on: its visible text, its accessible name (with what it
/// acts on), whether it is the quiet kind (cancel, withdraw), and what it does.
pub struct Action {
    /// Visible text.
    pub text: &'static str,
    /// Accessible name.
    pub label: String,
    /// Quiet style for actions that end something.
    pub quiet: bool,
    /// What it does.
    pub run: Callback<()>,
}

/// A row of small action buttons.
#[component]
pub fn Actions(actions: Vec<Action>) -> impl IntoView {
    view! {
        <div class="row">
            {actions
                .into_iter()
                .map(|a| {
                    let class = if a.quiet {
                        "button button--small button--quiet"
                    } else {
                        "button button--small"
                    };
                    let run = a.run;
                    view! {
                        <button type="button" class=class aria-label=a.label on:click=move |_| run.run(())>
                            {a.text}
                        </button>
                    }
                })
                .collect_view()}
        </div>
    }
}

/// Choices as chips: one of them (radio buttons) in a fieldset with a legend.
#[component]
pub fn RadioChips(
    legend: &'static str,
    name: &'static str,
    options: Vec<(String, String)>,
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <fieldset class="choices">
            <legend>{legend}</legend>
            {options
                .into_iter()
                .map(|(id, text)| {
                    let checked = {
                        let id = id.clone();
                        move || value.get() == id
                    };
                    view! {
                        <label class="choice chip">
                            <input type="radio" name=name prop:checked=checked
                                on:change=move |_| value.set(id.clone()) />
                            {text}
                        </label>
                    }
                })
                .collect_view()}
        </fieldset>
    }
}

/// Choices as chips: any number of them (checkboxes) in a fieldset with a legend. With
/// `short`, each chip shows the short form (by ID) and keeps the full text for screen
/// readers and as its tooltip, so many fit on a line (the icon rules).
#[component]
pub fn CheckChips(
    legend: &'static str,
    options: Vec<(String, String)>,
    value: RwSignal<BTreeSet<String>>,
    #[prop(optional)] short: Option<fn(&str) -> String>,
    /// Choices that cannot be taken now (shown, but not to be ticked).
    #[prop(optional)]
    disabled: Option<Signal<BTreeSet<String>>>,
    /// The fieldset's ID, e.g. to move the focus to it.
    #[prop(optional)]
    id: Option<&'static str>,
) -> impl IntoView {
    view! {
        <fieldset class="choices" id=id>
            <legend>{legend}</legend>
            {options
                .into_iter()
                .map(|(id, text)| {
                    let shown = short.map(|f| f(&id));
                    let checked = {
                        let id = id.clone();
                        move || value.with(|s| s.contains(&id))
                    };
                    let off = {
                        let id = id.clone();
                        move || disabled.is_some_and(|d| d.with(|d| d.contains(&id)))
                    };
                    view! {
                        <label class="choice chip">
                            <input type="checkbox" prop:checked=checked prop:disabled=off
                                on:change=move |ev| {
                                    let on = event_target_checked(&ev);
                                    value.update(|s| {
                                        if on {
                                            s.insert(id.clone());
                                        } else {
                                            s.remove(&id);
                                        }
                                    });
                                } />
                            {match shown.filter(|s| *s != text) {
                                Some(s) => view! {
                                    <abbr title=text.clone() aria-hidden="true">{s}</abbr>
                                    <span class="visually-hidden">{text.clone()}</span>
                                }.into_any(),
                                None => view! { {text.clone()} }.into_any(),
                            }}
                        </label>
                    }
                })
                .collect_view()}
        </fieldset>
    }
}

/// A labelled select whose value is a device setting (theme, handedness).
#[component]
pub fn SettingSelect(
    id: &'static str,
    label: &'static str,
    options: &'static [(&'static str, &'static str)],
    value: RwSignal<String>,
    apply: fn(&str),
) -> impl IntoView {
    view! {
        <label for=id>{label}</label>
        <select id=id prop:value=move || value.get() on:change=move |ev| {
            let chosen = event_target_value(&ev);
            apply(&chosen);
            value.set(chosen);
        }>
            {options.iter().map(|(v, text)| view! { <option value=*v>{*text}</option> }).collect_view()}
        </select>
    }
}

// [impl->dsn~person-suggestions~1]
/// Suggestions for a text input (native `datalist`, id `{id}`): people, call signs; the
/// input names it with `list`, and anything else can still be typed.
#[component]
pub fn Suggestions(
    #[prop(into)] id: String,
    #[prop(into)] values: Signal<Vec<String>>,
    /// For people: each option shows the person's display name, its value stays the user
    /// ID (ORG-7).
    #[prop(optional)]
    core: Option<crate::core::Handle>,
) -> impl IntoView {
    view! {
        <datalist id=id>
            {move || {
                let values = values.get();
                if let Some(core) = core {
                    let users: Vec<ordo_core::UserId> =
                        values.iter().filter_map(|v| v.parse().ok()).collect();
                    crate::core::request_avatars(core, &users);
                }
                values
                    .into_iter()
                    .map(|p| {
                        let label = core.map(|c| crate::shell::profile::display_text(c, &p)).filter(|l| *l != p);
                        view! { <option value=p.clone()>{label}</option> }
                    })
                    .collect_view()
            }}
        </datalist>
    }
}

/// The people of the incident, for person fields of forms (provided by the patient view).
#[derive(Clone, Copy)]
pub struct IncidentPeople(pub Signal<Vec<String>>, pub crate::core::Handle);

// [impl->dsn~list-keys~1]
/// Moves the focus to the next or previous item of a list marked `data-keys="list"`, with
/// `j`/Arrow Down or `k`/Arrow Up, when the focus is on a link, button or summary inside
/// one of its items and no modifier is held (ADR-0051). At either end the focus stays.
/// Returns whether it handled the key.
pub fn move_in_list(ev: &web_sys::KeyboardEvent) -> bool {
    use wasm_bindgen::JsCast;
    let modified = ev.ctrl_key() || ev.meta_key() || ev.alt_key() || ev.shift_key();
    let Some(forward) = list_direction(&ev.key(), modified) else {
        return false;
    };
    let Some(active) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element())
    else {
        return false;
    };
    if !matches!(active.tag_name().as_str(), "A" | "BUTTON" | "SUMMARY") {
        return false;
    }
    let Ok(Some(item)) = active.closest("[data-keys=list] > li") else {
        return false;
    };
    let mut next = if forward {
        item.next_element_sibling()
    } else {
        item.previous_element_sibling()
    };
    // Skip items without anything to focus.
    while let Some(candidate) = next {
        if let Ok(Some(target)) = candidate.query_selector(FOCUSABLE)
            && let Ok(target) = target.dyn_into::<web_sys::HtmlElement>()
        {
            ev.prevent_default();
            let _ = target.focus();
            return true;
        }
        next = if forward {
            candidate.next_element_sibling()
        } else {
            candidate.previous_element_sibling()
        };
    }
    // At the end: the key is used up, the focus stays.
    ev.prevent_default();
    true
}

// [impl->dsn~list-keys~1]
/// The direction a key moves in a list: `Some(true)` down, `Some(false)` up, `None` for
/// any other key or with a modifier held.
fn list_direction(key: &str, modified: bool) -> Option<bool> {
    match (key, modified) {
        ("j" | "ArrowDown", false) => Some(true),
        ("k" | "ArrowUp", false) => Some(false),
        _ => None,
    }
}

/// What the list keys move the focus to within an item.
const FOCUSABLE: &str = "a[href], button:not([hidden]):not([disabled]), summary";

#[cfg(test)]
mod tests {
    use super::list_direction;

    // [utest->dsn~list-keys~1]
    #[test]
    fn j_and_k_move_like_the_arrows_and_never_with_a_modifier() {
        assert_eq!(list_direction("j", false), Some(true));
        assert_eq!(list_direction("ArrowDown", false), Some(true));
        assert_eq!(list_direction("k", false), Some(false));
        assert_eq!(list_direction("ArrowUp", false), Some(false));
        assert_eq!(list_direction("j", true), None, "Ctrl+J is the palette's");
        assert_eq!(list_direction("J", false), None);
        assert_eq!(list_direction("x", false), None);
    }
}
