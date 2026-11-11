//! Command palette: Ctrl+K (⌘K on Apple devices) opens one search field for views,
//! patients, units and commands; Enter runs the chosen entry (UX-10, ADR-0030).
//!
//! Accessibility: a modal `<dialog>` (focus stays inside, Escape closes, focus returns),
//! with a combobox and a listbox following the WAI-ARIA Authoring Practices; the number
//! of results is announced. Every command is also reachable without the palette.
//!
//! Need to know: patients are found by triage tag number only. Searching by name is a
//! separate, logged function for authorised roles (PAT-7).
//!
//! Language: keys and command names are the same in every language (ADR-0036). Every
//! command has a stable English name, which is always searched besides the label.

use leptos::prelude::*;
use ordo_contracts::TriageCategory;
use ordo_med_patients::{self as med, PatientRegister};
use ordo_units::{self as units, UnitRegister};

use crate::core::{self, Handle};
use crate::nav::{self, Route};
use crate::patients::{FORMS, record_route};
use crate::text;
use crate::views::{CATEGORIES, Nav, access, apply_theme, click, focus, uhs_list};
use ordo_med_patients::record::FormKind;

/// The shortcut, with English key names in every language (ADR-0036).
pub const SHORTCUT: &str = "Ctrl+K";
/// The shortcut on Apple devices.
pub const SHORTCUT_MAC: &str = "⌘K";

/// English core meanings of unit status codes, for typed commands (`status on scene`).
/// English name of a triage category (its core meaning).
fn category_name(category: TriageCategory) -> &'static str {
    match category {
        TriageCategory::Immediate => "immediate",
        TriageCategory::Urgent => "urgent",
        TriageCategory::Minor => "minor",
        TriageCategory::Expectant => "expectant",
        TriageCategory::Deceased => "deceased",
        TriageCategory::Uninjured => "uninjured",
    }
}

/// What an entry does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Go to a route.
    Go(Route),
    /// Go to a route, then focus an element (e.g. the first field of a form).
    GoFocus(Route, String),
    /// Go to a route, then click an element (e.g. "new entry").
    GoClick(Route, String),
    /// Show the commands for one patient.
    Patient(String),
    /// Re-triage a patient.
    Retriage(String, TriageCategory),
    /// Show the commands for one unit.
    Unit(String),
    /// Set the status of a unit.
    SetStatus(String, String),
    /// Switch the colour theme.
    Theme(&'static str),
}

/// One entry of the palette.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item {
    /// Kind, shown before the label.
    pub group: &'static str,
    /// Label.
    pub label: String,
    /// Further text, also searched.
    pub detail: String,
    /// Stable English name of a command, searched in every language (ADR-0036).
    pub name: String,
    /// Why the entry cannot be used now.
    pub disabled: Option<&'static str>,
    /// What it does.
    pub action: Action,
    /// Shown before anything is typed.
    pub suggested: bool,
}

/// Which list the palette shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Page {
    /// Everything.
    Root,
    /// Commands for one patient.
    Patient(String),
    /// Commands for one unit.
    Unit(String),
}

/// Folds text for matching: lower case, accents and umlauts without marks, ß as ss.
pub fn fold(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' | 'à' | 'á' | 'â' => out.push('a'),
            'ö' | 'ò' | 'ó' | 'ô' => out.push('o'),
            'ü' | 'ù' | 'ú' | 'û' => out.push('u'),
            'é' | 'è' | 'ê' => out.push('e'),
            'ß' => out.push_str("ss"),
            c => out.push(c),
        }
    }
    out
}

/// How well `haystack` matches `query`: every word of the query must occur; matches at
/// the start of the text or of a word count more. `None`: no match.
pub fn score(query: &str, haystack: &str) -> Option<u32> {
    let hay = fold(haystack);
    let mut total = 0;
    for word in fold(query).split_whitespace() {
        let best = hay
            .match_indices(word)
            .map(|(i, _)| {
                if i == 0 {
                    4
                } else if hay[..i]
                    .chars()
                    .next_back()
                    .is_some_and(|c| !c.is_alphanumeric())
                {
                    3
                } else {
                    1
                }
            })
            .max()?;
        total += best;
    }
    Some(total)
}

/// The entries to show for `query`, best first; suggested entries when it is empty.
pub fn rank(query: &str, items: Vec<Item>) -> Vec<Item> {
    if query.trim().is_empty() {
        return items.into_iter().filter(|i| i.suggested).collect();
    }
    let mut scored: Vec<(u32, usize, Item)> = items
        .into_iter()
        .enumerate()
        .filter_map(|(n, item)| {
            let haystack = format!(
                "{} {} {} {}",
                item.label, item.detail, item.group, item.name
            );
            let label_bonus = u32::from(score(query, &item.label).is_some());
            score(query, &haystack).map(|s| (s * 2 + label_bonus, n, item))
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored
        .into_iter()
        .map(|(_, _, item)| item)
        .take(50)
        .collect()
}

/// The icon of a result's kind: a view to open, or a command to run.
fn group_icon(group: &str) -> &'static str {
    if group == text::GROUP_VIEWS {
        "app-window"
    } else {
        "zap"
    }
}

fn item(group: &'static str, label: String, action: Action) -> Item {
    Item {
        group,
        label,
        name: String::new(),
        detail: String::new(),
        disabled: None,
        action,
        suggested: false,
    }
}

impl Item {
    /// Sets the stable English name (ADR-0036).
    fn named(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }
}

/// All entries of a page, from the current state.
fn items(core: Handle, page: &Page) -> Vec<Item> {
    let med_access =
        nav::workspace("med.uhs").map_or(Err(text::REASON_NO_ACCESS), |w| access(core, w));
    let command_access = nav::workspace("command.situation")
        .map_or(Err(text::REASON_NO_ACCESS), |w| access(core, w));
    let events = |compartment: &str| core.with_value(|c| c.events(compartment));
    match page {
        Page::Root => {
            let mut list = Vec::new();
            for w in nav::WORKSPACES {
                let reason = access(core, w).err();
                for v in w.views {
                    let go = Action::Go(Route::view(w.id, v.id));
                    list.push(Item {
                        detail: format!("{} – {}", w.package, w.label),
                        disabled: reason,
                        suggested: true,
                        ..item(text::GROUP_VIEWS, v.label.to_owned(), go).named(v.name)
                    });
                }
            }
            let actions = [
                (
                    text::ACTION_REGISTER,
                    "register patient",
                    Route::view("med.uhs", "patients"),
                    "tag",
                    med_access,
                ),
                (
                    text::ACTION_CREATE_UHS,
                    "create uhs",
                    Route::view("med.uhs", "uhs"),
                    "uhs-name",
                    med_access,
                ),
                (
                    text::ACTION_ADD_UNIT,
                    "add unit",
                    Route::view("command.situation", "units"),
                    "unit-call-sign",
                    command_access,
                ),
            ];
            for (label, name, route, target, allowed) in actions {
                let go = Action::GoFocus(route, target.to_owned());
                list.push(Item {
                    disabled: allowed.err(),
                    suggested: true,
                    ..item(text::GROUP_ACTIONS, label.to_owned(), go).named(name)
                });
            }
            for (id, label) in text::THEMES {
                let name = format!("theme {}", if id.is_empty() { "auto" } else { id });
                list.push(
                    item(
                        text::GROUP_ACTIONS,
                        text::theme_set(label),
                        Action::Theme(id),
                    )
                    .named(name),
                );
            }
            if med_access.is_ok() {
                let names: std::collections::HashMap<String, String> =
                    uhs_list(core).into_iter().map(|u| (u.id, u.name)).collect();
                for p in PatientRegister::from_events(&events(med::COMPARTMENT)).patients() {
                    let uhs = names.get(&p.uhs).unwrap_or(&p.uhs);
                    list.push(Item {
                        detail: format!("{} · {uhs}", text::label(p.category())),
                        ..item(
                            text::GROUP_PATIENTS,
                            p.tag.clone(),
                            Action::Patient(p.tag.clone()),
                        )
                    });
                }
            }
            if command_access.is_ok() {
                let register = UnitRegister::from_events(&events(units::COMPARTMENT));
                for u in register.units.into_values() {
                    let status = u.status.as_ref().map_or_else(
                        || text::NO_STATUS.to_owned(),
                        |s| text::status_label(&s.status),
                    );
                    list.push(Item {
                        detail: format!("{} · {} · {status}", u.unit_type, u.organisation),
                        ..item(
                            text::GROUP_UNITS,
                            u.unit.clone(),
                            Action::Unit(u.unit.clone()),
                        )
                    });
                }
            }
            list
        }
        Page::Patient(tag) => {
            let record = record_route(tag);
            let current = PatientRegister::from_events(&events(med::COMPARTMENT))
                .patient(tag)
                .map(|p| p.category());
            let group = text::GROUP_PATIENTS;
            let mut list = vec![
                item(
                    group,
                    text::ACTION_OPEN_RECORD.to_owned(),
                    Action::Go(record.clone()),
                )
                .named("open record"),
                item(
                    group,
                    text::ACTION_TAKE_PHOTO.to_owned(),
                    Action::GoFocus(record.clone(), "photo-camera".to_owned()),
                )
                .named("photo"),
            ];
            for f in FORMS.forms.iter().filter(|f| f.kind == FormKind::Series) {
                let go = Action::GoClick(record.clone(), format!("new-{}", f.id));
                list.push(
                    item(group, text::action_entry(&f.label), go).named(format!("add {}", f.id)),
                );
            }
            for c in CATEGORIES.into_iter().filter(|c| Some(*c) != current) {
                let action = Action::Retriage(tag.clone(), c);
                list.push(
                    item(group, text::action_retriage(c), action)
                        .named(format!("retriage {}", category_name(c))),
                );
            }
            for i in &mut list {
                i.detail = text::patient_title(tag);
                i.suggested = true;
            }
            list
        }
        Page::Unit(unit) => {
            let can_edit = core.with_value(|c| {
                c.incident
                    .as_ref()
                    .is_some_and(|i| c.replica.may(i, "edit_units"))
            });
            crate::units::STATUS
                .codes
                .iter()
                .map(|status| {
                    let code = status.code.as_str();
                    let name = format!("status {code} {}", status.name);
                    let action = Action::SetStatus(unit.clone(), code.to_owned());
                    Item {
                        detail: unit.clone(),
                        suggested: true,
                        disabled: (!can_edit).then_some(text::REASON_NO_ACCESS),
                        ..item(text::GROUP_UNITS, text::action_status(code), action).named(name)
                    }
                })
                .collect()
        }
    }
}

// [impl->dsn~command-palette~1]
/// The palette dialog. `open` is toggled by Ctrl+K / ⌘K and the search button.
#[component]
pub fn Palette(
    core: Handle,
    revision: RwSignal<u64>,
    nav: Nav,
    open: RwSignal<bool>,
    theme: RwSignal<String>,
) -> impl IntoView {
    let dialog = NodeRef::<leptos::html::Dialog>::new();
    let query = RwSignal::new(String::new());
    let page = RwSignal::new(Page::Root);
    let active = RwSignal::new(0usize);
    let results = Memo::new(move |_| {
        revision.track();
        if !open.get() {
            return Vec::new();
        }
        rank(&query.get(), items(core, &page.get()))
    });
    Effect::new(move |_| {
        let Some(d) = dialog.get() else { return };
        if open.get() {
            query.set(String::new());
            page.set(Page::Root);
            active.set(0);
            if !d.open() {
                let _ = d.show_modal();
            }
            focus("palette-input");
        } else if d.open() {
            d.close();
        }
    });
    // Keep the active option visible.
    Effect::new(move |_| {
        let id = format!("palette-option-{}", active.get());
        results.track();
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(&id))
        {
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Nearest);
            el.scroll_into_view_with_scroll_into_view_options(&options);
        }
    });
    let after = |then: Box<dyn FnOnce()>| {
        set_timeout(then, std::time::Duration::from_millis(80));
    };
    let run = move |item: Item| {
        if item.disabled.is_some() {
            return;
        }
        match item.action {
            Action::Patient(tag) => {
                page.set(Page::Patient(tag));
                query.set(String::new());
                active.set(0);
                focus("palette-input");
            }
            Action::Unit(unit) => {
                page.set(Page::Unit(unit));
                query.set(String::new());
                active.set(0);
                focus("palette-input");
            }
            Action::Go(route) => {
                open.set(false);
                nav.go(route);
                after(Box::new(|| focus("main")));
            }
            Action::GoFocus(route, target) => {
                open.set(false);
                nav.go(route);
                after(Box::new(move || focus(&target)));
            }
            Action::GoClick(route, target) => {
                open.set(false);
                nav.go(route);
                after(Box::new(move || click(&target)));
            }
            Action::Retriage(tag, category) => {
                open.set(false);
                let _ = core::retriage(core, revision, &tag, category);
            }
            Action::SetStatus(unit, status) => {
                open.set(false);
                let _ = core::set_status(core, revision, &unit, &status);
            }
            Action::Theme(id) => {
                open.set(false);
                apply_theme(id);
                theme.set(id.to_owned());
            }
        }
    };
    let keydown = move |ev: leptos::ev::KeyboardEvent| {
        let n = results.with(Vec::len);
        // [impl->dsn~list-keys~1]
        // Ctrl+J / Ctrl+K (⌘ on Apple devices) move like the arrow keys (ADR-0051).
        let key = match ev.key().to_lowercase().as_str() {
            "j" if ev.ctrl_key() || ev.meta_key() => "ArrowDown".to_owned(),
            "k" if ev.ctrl_key() || ev.meta_key() => "ArrowUp".to_owned(),
            _ => ev.key(),
        };
        match key.as_str() {
            "ArrowDown" if n > 0 => {
                ev.prevent_default();
                active.update(|a| *a = (*a + 1) % n);
            }
            "ArrowUp" if n > 0 => {
                ev.prevent_default();
                active.update(|a| *a = (*a + n - 1) % n);
            }
            "Enter" => {
                ev.prevent_default();
                if let Some(item) = results.with(|r| r.get(active.get()).cloned()) {
                    run(item);
                }
            }
            "Backspace" if query.with(String::is_empty) && page.get() != Page::Root => {
                ev.prevent_default();
                page.set(Page::Root);
                active.set(0);
            }
            _ => {}
        }
    };
    let breadcrumb = move || match page.get() {
        Page::Root => None,
        Page::Patient(tag) => Some(text::patient_title(&tag)),
        Page::Unit(unit) => Some(unit),
    };
    view! {
        <dialog
            node_ref=dialog
            id="palette"
            class="palette"
            aria-labelledby="palette-title"
            // Escape: the browser closes the dialog itself; `cancel` comes at once, `close`
            // later. Updating the state on `cancel` lets an immediate Ctrl+K reopen it.
            on:cancel=move |_: leptos::ev::Event| open.set(false)
            on:close=move |_| {
                // A late `close` event of an earlier closing must not close the palette
                // again after it was reopened (seen in Chromium).
                if dialog.get_untracked().is_some_and(|d| !d.open()) {
                    open.set(false);
                }
            }
            on:click=move |ev| {
                // A click on the backdrop (outside the content) closes the palette.
                if ev.target() == ev.current_target() {
                    open.set(false);
                }
            }
        >
            <div class="palette__content">
                <h2 id="palette-title" class="palette__title">
                    {text::PALETTE_TITLE}
                    {move || breadcrumb().map(|b| view! { <span class="palette__crumb">" › " {b}</span> })}
                </h2>
                <input
                    id="palette-input"
                    class="palette__input"
                    type="text"
                    role="combobox"
                    autocomplete="off"
                    spellcheck="false"
                    aria-label=text::PALETTE_TITLE
                    aria-expanded="true"
                    aria-controls="palette-list"
                    aria-autocomplete="list"
                    aria-activedescendant=move || {
                        (results.with(Vec::len) > 0).then(|| format!("palette-option-{}", active.get()))
                    }
                    placeholder=text::PALETTE_PLACEHOLDER
                    prop:value=move || query.get()
                    on:input=move |ev| {
                        query.set(event_target_value(&ev));
                        active.set(0);
                    }
                    on:keydown=keydown
                />
                <ul id="palette-list" class="palette__list" role="listbox" aria-label=text::PALETTE_RESULTS>
                    {move || {
                        results
                            .get()
                            .into_iter()
                            .enumerate()
                            .map(|(i, item)| {
                                let chosen = item.clone();
                                view! {
                                    <li
                                        id=format!("palette-option-{i}")
                                        class="palette__option"
                                        role="option"
                                        aria-selected=move || (active.get() == i).to_string()
                                        aria-disabled=item.disabled.is_some().then_some("true")
                                        on:click=move |_| run(chosen.clone())
                                        on:pointermove=move |_| {
                                            if active.get_untracked() != i {
                                                active.set(i);
                                            }
                                        }
                                    >
                                        // The kind as an icon, leaving room for the text; the
                                        // word stays for screen readers and as a tooltip.
                                        <span class="palette__group" title=item.group>
                                            <crate::ui::Icon name=group_icon(item.group) />
                                            <span class="visually-hidden">{item.group}</span>
                                        </span>
                                        <span class="palette__label">{item.label.clone()}</span>
                                        <span class="palette__detail">{item.disabled.map_or(item.detail.clone(), str::to_owned)}</span>
                                        {(!item.name.is_empty()).then(|| view! { <span class="palette__name" lang="en">{item.name.clone()}</span> })}
                                    </li>
                                }
                            })
                            .collect_view()
                    }}
                </ul>
                <p class="palette__count" role="status">{move || text::palette_count(results.with(Vec::len))}</p>
                <p class="palette__keys" aria-hidden="true">
                    <kbd>"↑"</kbd><kbd>"↓"</kbd> " " {text::PALETTE_KEYS} " · "
                    <kbd>"Enter"</kbd> " " {text::PALETTE_RUN} " · "
                    <kbd>"Backspace"</kbd> " " {text::PALETTE_BACK} " · "
                    <kbd>"Esc"</kbd> " " {text::PALETTE_CLOSE}
                </p>
            </div>
        </dialog>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(label: &str, detail: &str, name: &str) -> Item {
        Item {
            detail: detail.to_owned(),
            suggested: label.starts_with('V'),
            ..item(text::GROUP_ACTIONS, label.to_owned(), Action::Theme("")).named(name)
        }
    }

    #[test]
    fn folding_ignores_case_and_umlauts() {
        assert_eq!(fold("Bühne STRAßE"), "buhne strasse");
    }

    // [utest->dsn~command-palette~1]
    #[test]
    fn every_word_must_match_and_word_starts_count_more() {
        assert_eq!(score("rtw", "RTW 83/1"), Some(4));
        assert_eq!(score("83", "RTW 83/1"), Some(3));
        assert_eq!(score("tw", "RTW 83/1"), Some(1));
        assert_eq!(score("rtw 2", "RTW 83/1"), None);
        assert_eq!(score("buhne", "UHS Bühne Nord"), Some(3));
        assert_eq!(score("0042", "DE-0042"), Some(3));
    }

    // [utest->dsn~command-palette~1]
    #[test]
    fn results_are_ranked_and_suggestions_shown_without_query() {
        let items = vec![
            entry("Vitalwerte eintragen", "Patient DE-0042", "add vitals"),
            entry("Patient registrieren", "", "register patient"),
            entry("Einsatzmittel", "Einsatzleitung", "units"),
        ];
        let labels = |list: Vec<Item>| list.into_iter().map(|i| i.label).collect::<Vec<_>>();
        assert_eq!(
            labels(rank("", items.clone())),
            vec!["Vitalwerte eintragen"]
        );
        assert_eq!(
            labels(rank("patient", items.clone())),
            vec!["Patient registrieren", "Vitalwerte eintragen"],
            "a match in the label ranks first"
        );
        assert_eq!(
            labels(rank("einsatz", items.clone())),
            vec!["Einsatzmittel"]
        );
        assert!(rank("xyz", items.clone()).is_empty());
        // ADR-0036: the English names work in every UI language.
        assert_eq!(
            labels(rank("register", items.clone())),
            vec!["Patient registrieren"]
        );
        assert_eq!(
            labels(rank("add vitals", items)),
            vec!["Vitalwerte eintragen"]
        );
    }
}
