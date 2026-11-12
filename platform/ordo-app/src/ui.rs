//! Shared UI building blocks (architecture 8.6.2: own components after the WAI-ARIA
//! Authoring Practices). Views compose these instead of building each control again.

use std::collections::BTreeSet;

use leptos::prelude::*;

use crate::views::focus;

/// The SVG of a Lucide icon (`design/assets/icons/lucide`, ISC), by name.
fn svg(name: &str) -> &'static str {
    macro_rules! icons {
        ($($name:literal),* $(,)?) => {
            match name {
                $($name => include_str!(concat!("../../../design/assets/icons/lucide/", $name, ".svg")),)*
                _ => "",
            }
        };
    }
    icons!(
        "ambulance",
        "app-window",
        "chart-column",
        "clipboard-list",
        "cloud-check",
        "cloud-off",
        "cloud-upload",
        "ellipsis",
        "flag",
        "id-card",
        "key-round",
        "layout-grid",
        "log-in",
        "map",
        "network",
        "notebook-text",
        "refresh-cw",
        "search",
        "tablet-smartphone",
        "tent",
        "truck",
        "users",
        "zap",
    )
}

// [impl->dsn~compact-navigation~2]
/// A decorative icon: the text next to it carries the meaning (never an icon alone).
#[component]
pub fn Icon(name: &'static str) -> impl IntoView {
    view! { <span class="icon" aria-hidden="true" inner_html=svg(name)></span> }
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
    view! {
        <button
            id=id
            type="button"
            class=class
            aria-expanded=move || open.get().to_string()
            aria-controls=controls
            aria-label=move || label.map(|l| l.get())
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

/// One entry of a history: time, what happened, who.
pub type HistoryEntry = (String, String, String);

/// The history of a record (status changes): collapsed, every entry with time and person.
#[component]
pub fn History(#[prop(into)] entries: Signal<Vec<HistoryEntry>>) -> impl IntoView {
    view! {
        <details class="history-list">
            <summary>{crate::text::TRANSPORT_HISTORY}</summary>
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

/// Choices as chips: any number of them (checkboxes) in a fieldset with a legend.
#[component]
pub fn CheckChips(
    legend: &'static str,
    options: Vec<(String, String)>,
    value: RwSignal<BTreeSet<String>>,
) -> impl IntoView {
    view! {
        <fieldset class="choices">
            <legend>{legend}</legend>
            {options
                .into_iter()
                .map(|(id, text)| {
                    let checked = {
                        let id = id.clone();
                        move || value.with(|s| s.contains(&id))
                    };
                    view! {
                        <label class="choice chip">
                            <input type="checkbox" prop:checked=checked
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
                            {text}
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
                        let label = core.map(|c| crate::profile::display_text(c, &p)).filter(|l| *l != p);
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
