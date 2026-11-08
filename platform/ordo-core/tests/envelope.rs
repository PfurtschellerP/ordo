//! Encoding tests for the event envelope against spec/sync/event.md.

use ciborium::Value;
use ordo_core::event::ENVELOPE_VERSION;
use ordo_core::{
    Author, CodecError, DeviceId, EventBody, EventId, Hlc, IncidentId, Payload, SealedBox,
    ServerName, Signature, SignedEvent, Ulid, UserId,
};

fn sample_body(payload: Payload) -> EventBody {
    let server: ServerName = "ordo.drk-musterstadt.de".parse().unwrap();
    let device = DeviceId(Ulid::from_parts(1_762_600_000_000, 1).unwrap());
    EventBody {
        version: ENVELOPE_VERSION,
        id: EventId(Ulid::from_parts(1_762_600_000_123, 42).unwrap()),
        schema_version: 1,
        hlc: Hlc {
            wall_ms: 1_762_600_000_123,
            counter: 0,
            node: device,
        },
        incident: IncidentId::new(Ulid::from_parts(1_762_599_000_000, 7).unwrap(), &server),
        module: "med.patients".parse().unwrap(),
        compartment: "med.clinical".parse().unwrap(),
        event_type: "patients.registered".parse().unwrap(),
        author: Author {
            user: UserId::new("anna", &server).unwrap(),
            device,
        },
        origin: server,
        payload,
    }
}

fn plain() -> Payload {
    Payload::Plain(vec![0xa1, 0x61, 0x78, 0x01]) // {"x": 1}
}

fn sealed() -> Payload {
    Payload::Sealed(SealedBox {
        ct: vec![1, 2, 3],
        key: "med-clinical-1@ordo.drk-musterstadt.de".parse().unwrap(),
        nonce: [9; 24],
    })
}

fn to_value(bytes: &[u8]) -> Value {
    ciborium::from_reader(bytes).unwrap()
}

fn from_value(value: &Value) -> Vec<u8> {
    let mut out = Vec::new();
    ciborium::into_writer(value, &mut out).unwrap();
    out
}

/// Checks RFC 8949 §4.2.1 key order (by encoded key bytes) in all nested maps.
fn assert_deterministic_order(value: &Value) {
    match value {
        Value::Map(entries) => {
            let keys: Vec<Vec<u8>> = entries.iter().map(|(k, _)| from_value(k)).collect();
            let mut sorted = keys.clone();
            sorted.sort();
            assert_eq!(keys, sorted, "map keys are not in deterministic order");
            entries
                .iter()
                .for_each(|(_, v)| assert_deterministic_order(v));
        }
        Value::Array(items) => items.iter().for_each(assert_deterministic_order),
        _ => {}
    }
}

#[test]
fn body_round_trip() {
    for payload in [plain(), sealed()] {
        let body = sample_body(payload);
        let bytes = body.encode().unwrap();
        assert_eq!(EventBody::decode(&bytes).unwrap(), body);
    }
}

#[test]
fn body_uses_deterministic_key_order() {
    for payload in [plain(), sealed()] {
        let bytes = sample_body(payload).encode().unwrap();
        assert_deterministic_order(&to_value(&bytes));
    }
}

#[test]
fn body_matches_spec_field_names_and_types() {
    let bytes = sample_body(plain()).encode().unwrap();
    let Value::Map(entries) = to_value(&bytes) else {
        panic!("body must be a map");
    };
    let keys: Vec<&str> = entries.iter().map(|(k, _)| k.as_text().unwrap()).collect();
    assert_eq!(
        keys,
        [
            "v", "id", "sv", "hlc", "inc", "mod", "comp", "type", "author", "origin", "payload"
        ]
    );
    let get = |key: &str| {
        &entries
            .iter()
            .find(|(k, _)| k.as_text() == Some(key))
            .unwrap()
            .1
    };
    assert_eq!(get("id").as_bytes().unwrap().len(), 16);
    let hlc = get("hlc").as_array().unwrap();
    assert_eq!(hlc.len(), 3);
    assert_eq!(hlc[2].as_bytes().unwrap().len(), 16);
    assert!(
        get("inc")
            .as_text()
            .unwrap()
            .ends_with("@ordo.drk-musterstadt.de")
    );
}

#[test]
fn unknown_body_keys_are_ignored() {
    let body = sample_body(plain());
    let Value::Map(mut entries) = to_value(&body.encode().unwrap()) else {
        unreachable!()
    };
    entries.push((Value::Text("future".into()), Value::Text("field".into())));
    let extended = from_value(&Value::Map(entries));
    assert_eq!(EventBody::decode(&extended).unwrap(), body);
}

#[test]
fn unsupported_version_is_rejected() {
    let Value::Map(mut entries) = to_value(&sample_body(plain()).encode().unwrap()) else {
        unreachable!()
    };
    entries[0].1 = Value::Integer(2.into());
    let bytes = from_value(&Value::Map(entries));
    assert_eq!(
        EventBody::decode(&bytes),
        Err(CodecError::UnsupportedVersion(2))
    );
}

#[test]
fn trailing_bytes_are_rejected() {
    let mut bytes = sample_body(plain()).encode().unwrap();
    bytes.push(0);
    assert_eq!(EventBody::decode(&bytes), Err(CodecError::TrailingBytes));
}

// [utest->dsn~qualified-identifiers~1]
#[test]
fn invalid_identifiers_are_rejected_when_decoding() {
    let Value::Map(mut entries) = to_value(&sample_body(plain()).encode().unwrap()) else {
        unreachable!()
    };
    let origin = entries
        .iter_mut()
        .find(|(k, _)| k.as_text() == Some("origin"))
        .unwrap();
    origin.1 = Value::Text("NOT A SERVER".into());
    assert!(EventBody::decode(&from_value(&Value::Map(entries))).is_err());
}

#[test]
fn signed_event_is_a_two_element_array() {
    let event = SignedEvent {
        body: sample_body(plain()).encode().unwrap(),
        signature: Signature {
            kid: "device-1@ordo.drk-musterstadt.de".parse().unwrap(),
            val: [7; 64],
        },
    };
    let bytes = event.encode().unwrap();
    let value = to_value(&bytes);
    let items = value.as_array().expect("signed event must be an array");
    assert_eq!(items.len(), 2);
    assert!(items[0].is_bytes(), "body must be a byte string");
    assert_deterministic_order(&value);
    assert_eq!(SignedEvent::decode(&bytes).unwrap(), event);
}

#[test]
fn oversized_signed_event_is_rejected() {
    let bytes = vec![0u8; ordo_core::event::MAX_SIGNED_EVENT_LEN + 1];
    assert!(matches!(
        SignedEvent::decode(&bytes),
        Err(CodecError::TooLarge { .. })
    ));
}
