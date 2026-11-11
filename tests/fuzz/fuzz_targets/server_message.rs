//! What a device receives from the server (SEC-17).
// [itest->dsn~fuzzing~1]
#![no_main]

use libfuzzer_sys::fuzz_target;
use ordo_sync::message::ServerMessage;

fuzz_target!(|data: &[u8]| {
    ordo_fuzz::round_trip::<ServerMessage>(data);
});
