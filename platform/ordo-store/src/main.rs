//! ORDO device storage worker (ADR-0034, ADR-0041).
//!
//! A dedicated web worker that keeps the device's data in SQLite on the browser's file
//! system (OPFS, sync access handles). It is a plain key-value store: the app encrypts
//! every value with its storage key before sending it here, so this worker only ever sees
//! ciphertext and needs no keys.
//!
//! Once it listens, the worker sends `["ready"]`; the app waits for it before its first
//! request (messages sent earlier would be lost). Messages (JavaScript arrays), each answered
//! with `[id, "ok", …]` or `[id, "error", text]`:
//!
//! - `["open", id]`: open the database (once, before anything else);
//! - `["load", id, prefix]`: all entries whose key starts with `prefix`, as `[[key, bytes]]`;
//! - `["write", id, puts, prefixes]`: in one transaction, delete every key starting with
//!   one of `prefixes`, then store `puts` (`[[key, bytes]]`);
//! - `["clear", id]`: delete everything.

#[cfg(target_arch = "wasm32")]
mod worker;

fn main() {
    #[cfg(target_arch = "wasm32")]
    worker::start();
}
