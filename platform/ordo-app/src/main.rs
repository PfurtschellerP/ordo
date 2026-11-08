//! ORDO web app (prototype). Build with Trunk, see Trunk.toml.

mod core;
mod devices;
mod nav;
mod palette;
mod patients;
mod text;
mod views;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(views::App);
}
