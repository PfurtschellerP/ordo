//! ORDO web app (prototype). Build with Trunk, see Trunk.toml.

mod attachments;
mod command;
mod core;
mod devices;
mod images;
mod incidents;
mod nav;
mod orders;
mod overview;
mod palette;
mod patients;
mod profile;
mod storage;
mod tasks;
mod text;
mod transports;
mod ui;
mod units;
mod vault;
mod views;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(views::App);
}
