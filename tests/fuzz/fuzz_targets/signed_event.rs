//! Signed events and their bodies, as pushed by devices and delivered by servers (SEC-17).
// [itest->dsn~fuzzing~1]
#![no_main]

use libfuzzer_sys::fuzz_target;
use ordo_core::event::SignedEvent;

fuzz_target!(|data: &[u8]| {
    if let Ok(event) = SignedEvent::decode(data) {
        let bytes = event.encode().expect("encodes");
        assert_eq!(SignedEvent::decode(&bytes).expect("decodes"), event);
        // The body is decoded by every receiver before the signature is checked.
        if let Ok(body) = event.decode_body() {
            let again = body.encode().expect("a decoded body encodes");
            let _ = ordo_core::event::EventBody::decode(&again).expect("and decodes again");
        }
    }
    // A body on its own, as the server's hub decodes it.
    let _ = ordo_core::event::EventBody::decode(data);
});
