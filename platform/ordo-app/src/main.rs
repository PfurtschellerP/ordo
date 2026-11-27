//! ORDO web app (prototype). Build with Trunk, see Trunk.toml.

mod admin;
mod attachments;
mod core;
mod i18n;
mod med;
mod shared;
mod shell;
mod storage;
mod text;
mod vault;

fn main() {
    log_panics();
    choose_language();
    leptos::mount::mount_to_body(shell::views::App);
}

// [impl->dsn~ui-texts~2]
/// The texts' language: the one chosen on this device, else the first of the browser's
/// languages ORDO has texts for, else German (ADR-0096); the page's `lang` follows, for
/// screen readers and hyphenation.
fn choose_language() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let preferred: Vec<String> = core::preference(i18n::PREFERENCE)
        .filter(|l| !l.is_empty())
        .into_iter()
        .chain(
            window
                .navigator()
                .languages()
                .iter()
                .filter_map(|l| l.as_string()),
        )
        .collect();
    let chosen = i18n::choose(&preferred);
    if let Some(root) = window.document().and_then(|d| d.document_element()) {
        let _ = root.set_attribute("lang", chosen);
    }
}

/// Writes a panic to the browser console with the JavaScript stack, where it can be found
/// (a WebAssembly panic otherwise shows only "unreachable"). Replaces the crate
/// `console_error_panic_hook`, whose repository is archived.
fn log_panics() {
    std::panic::set_hook(Box::new(|info| {
        let stack = js_sys::Reflect::get(&js_sys::Error::new(""), &"stack".into())
            .ok()
            .and_then(|s| s.as_string())
            .unwrap_or_default();
        web_sys::console::error_1(&format!("{info}\n\nStack:\n\n{stack}").into());
    }));
}
