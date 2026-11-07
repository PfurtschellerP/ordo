//! Signature and hash-chain tests against spec/sync/event.md §4 and §6.

use ciborium::Value;
use ordo_core::event::ENVELOPE_VERSION;
use ordo_core::{
    Author, DeviceId, EventBody, EventId, Hlc, IncidentId, Payload, ServerName, SignedEvent, Ulid,
    UserId,
};
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_crypto::{Chain, ChainError, CryptoError, EventSigner, event_hash, verify, verify_chain};

fn server() -> ServerName {
    "ordo.drk-musterstadt.de".parse().unwrap()
}

fn signer(seed: u8) -> EventSigner {
    EventSigner::new(
        SigningKey::from_bytes(&[seed; 32]),
        format!("device-{seed}@ordo.drk-musterstadt.de")
            .parse()
            .unwrap(),
    )
}

fn body(n: u128) -> EventBody {
    let device = DeviceId(Ulid::from_parts(1_762_600_000_000, 1).unwrap());
    EventBody {
        version: ENVELOPE_VERSION,
        id: EventId(Ulid::from_parts(1_762_600_000_000, n).unwrap()),
        schema_version: 1,
        hlc: Hlc {
            wall_ms: 1_762_600_000_000,
            counter: n as u16,
            node: device,
        },
        incident: IncidentId::new(Ulid::from_parts(1_762_599_000_000, 7).unwrap(), &server()),
        module: "units".parse().unwrap(),
        compartment: "incident".parse().unwrap(),
        event_type: "units.statusChanged".parse().unwrap(),
        author: Author {
            user: UserId::new("anna", &server()).unwrap(),
            device,
        },
        origin: server(),
        payload: Payload::Plain(vec![n as u8]),
    }
}

#[test]
fn signed_event_survives_transport_and_verifies() {
    let signer = signer(1);
    let event = signer.sign(&body(1)).unwrap();
    let received = SignedEvent::decode(&event.encode().unwrap()).unwrap();
    assert_eq!(verify(&received, &signer.verifying_key()), Ok(()));
    assert_eq!(received.decode_body().unwrap(), body(1));
}

#[test]
fn changed_body_fails_verification() {
    let signer = signer(1);
    let mut event = signer.sign(&body(1)).unwrap();
    let last = event.body.len() - 1;
    event.body[last] ^= 1;
    assert_eq!(
        verify(&event, &signer.verifying_key()),
        Err(CryptoError::BadSignature)
    );
}

#[test]
fn wrong_key_fails_verification() {
    let event = signer(1).sign(&body(1)).unwrap();
    assert_eq!(
        verify(&event, &signer(2).verifying_key()),
        Err(CryptoError::BadSignature)
    );
}

#[test]
fn events_with_unknown_fields_still_verify() {
    // A newer sender adds a field this version doesn't know. The body bytes are kept
    // as received, so the signature stays valid and the event can be forwarded.
    let mut bytes = body(1).encode().unwrap();
    let Value::Map(mut entries) = ciborium::from_reader::<Value, _>(&bytes[..]).unwrap() else {
        unreachable!()
    };
    entries.push((Value::Text("future".into()), Value::Bool(true)));
    bytes.clear();
    ciborium::into_writer(&Value::Map(entries), &mut bytes).unwrap();

    let signer = signer(1);
    let event = signer.sign_bytes(bytes);
    let forwarded = SignedEvent::decode(&event.encode().unwrap()).unwrap();
    assert_eq!(verify(&forwarded, &signer.verifying_key()), Ok(()));
    assert_eq!(forwarded.decode_body().unwrap(), body(1));
    assert_eq!(forwarded.body, event.body);
}

#[test]
fn chain_detects_changes_removals_and_reordering() {
    let signer = signer(1);
    let mut chain = Chain::new();
    let sequence: Vec<_> = (1..=3)
        .map(|n| chain.append(signer.sign(&body(n)).unwrap()))
        .collect();
    assert_eq!(
        sequence.iter().map(|e| e.seq).collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(verify_chain(&sequence).unwrap().head(), chain.head());

    // Changed event
    let mut changed = sequence.clone();
    changed[1].event = signer.sign(&body(9)).unwrap();
    assert_eq!(
        verify_chain(&changed).unwrap_err(),
        ChainError::Mismatch { seq: 2 }
    );

    // Removed event
    let removed = [sequence[0].clone(), sequence[2].clone()];
    assert_eq!(
        verify_chain(&removed).unwrap_err(),
        ChainError::Gap {
            expected: 2,
            found: 3
        }
    );

    // Reordered events, with sequence numbers rewritten to hide it
    let mut reordered = [
        sequence[0].clone(),
        sequence[2].clone(),
        sequence[1].clone(),
    ];
    reordered[1].seq = 2;
    reordered[2].seq = 3;
    assert_eq!(
        verify_chain(&reordered).unwrap_err(),
        ChainError::Mismatch { seq: 2 }
    );
}

#[test]
fn event_hash_depends_only_on_body() {
    let a = signer(1).sign(&body(1)).unwrap();
    let b = signer(2).sign(&body(1)).unwrap();
    assert_eq!(event_hash(&a), event_hash(&b));
    assert_ne!(
        event_hash(&a),
        event_hash(&signer(1).sign(&body(2)).unwrap())
    );
}
