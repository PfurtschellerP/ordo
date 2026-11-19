//! ORDO web app (prototype). Build with Trunk, see Trunk.toml.

mod admin;
mod attachments;
mod core;
mod med;
mod shared;
mod shell;
mod storage;
mod text;
mod vault;

fn main() {
    log_panics();
    leptos::mount::mount_to_body(shell::views::App);
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
