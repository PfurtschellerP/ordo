//! Units: the status codes of the profile (checked at build time, see `build.rs`).

use std::sync::LazyLock;

use ordo_units::StatusCatalogue;

/// Status codes from the profile (`units/status-codes.toml`).
pub static STATUS: LazyLock<StatusCatalogue> = LazyLock::new(|| {
    serde_json::from_str(include_str!(concat!(env!("OUT_DIR"), "/status-codes.json")))
        .expect("status codes were checked at build time")
});
