//! ORDO web app (prototype). Build with Trunk, see Trunk.toml.

mod accounts;
mod address;
mod attachments;
mod command;
mod core;
mod devices;
mod helpers;
mod images;
mod incidents;
mod log;
mod login_providers;
mod map_region;
mod nav;
mod notices;
mod orders;
mod overview;
mod palette;
mod patients;
mod profile;
mod qr;
mod reports;
mod roles;
mod situation;
mod storage;
mod summary;
mod tasks;
mod text;
mod transports;
mod ui;
mod units;
mod vault;
mod views;

fn main() {
    log_panics();
    leptos::mount::mount_to_body(views::App);
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
