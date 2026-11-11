//! What the server receives from any device (SEC-17).
// [itest->dsn~fuzzing~1]
#![no_main]

use libfuzzer_sys::fuzz_target;
use ordo_sync::message::{self, ClientMessage};

fuzz_target!(|data: &[u8]| {
    if let Ok(m) = message::decode::<ClientMessage>(data) {
        let bytes = message::encode(&m).expect("encodes");
        assert_eq!(
            message::decode::<ClientMessage>(&bytes).expect("decodes"),
            m
        );
    }
});
