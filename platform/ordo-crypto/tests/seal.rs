//! Sealed payload tests against spec/sync/event.md §5.

use ordo_core::{CompartmentId, EventId, IncidentId, KeyId, ServerName, Ulid};
use ordo_crypto::CryptoError;
use ordo_crypto::seal::{DataKey, SealContext, open, seal};

fn ids() -> (IncidentId, CompartmentId, KeyId) {
    let server: ServerName = "ordo.drk-musterstadt.de".parse().unwrap();
    (
        IncidentId::new(Ulid::from_parts(1_762_600_000_000, 1).unwrap(), &server),
        "med.clinical".parse().unwrap(),
        KeyId::new("med-clinical-1", &server).unwrap(),
    )
}

fn event(n: u128) -> EventId {
    EventId(Ulid::from_parts(1_762_600_000_000, n).unwrap())
}

// [utest->dsn~sealed-payloads~1]
#[test]
fn round_trip() {
    let (incident, compartment, key_id) = ids();
    let key = DataKey::from_bytes([7; 32]);
    let context = SealContext {
        event: event(1),
        incident: &incident,
        compartment: &compartment,
    };
    let sealed = seal(&key, key_id.clone(), [1; 24], context, b"SK II, UHS 1");
    assert_ne!(sealed.ct, b"SK II, UHS 1");
    assert_eq!(sealed.key, key_id);
    assert_eq!(open(&key, &sealed, context).unwrap(), b"SK II, UHS 1");
}

// [utest->dsn~sealed-payloads~1]
#[test]
fn wrong_key_changed_data_or_other_event_fail() {
    let (incident, compartment, key_id) = ids();
    let key = DataKey::from_bytes([7; 32]);
    let context = SealContext {
        event: event(1),
        incident: &incident,
        compartment: &compartment,
    };
    let sealed = seal(&key, key_id, [1; 24], context, b"secret");

    let wrong_key = DataKey::from_bytes([8; 32]);
    assert_eq!(
        open(&wrong_key, &sealed, context),
        Err(CryptoError::CannotOpen)
    );

    let mut changed = sealed.clone();
    changed.ct[0] ^= 1;
    assert_eq!(open(&key, &changed, context), Err(CryptoError::CannotOpen));

    let other_event = SealContext {
        event: event(2),
        ..context
    };
    assert_eq!(
        open(&key, &sealed, other_event),
        Err(CryptoError::CannotOpen)
    );

    let incident_compartment: CompartmentId = "incident".parse().unwrap();
    let other_compartment = SealContext {
        compartment: &incident_compartment,
        ..context
    };
    assert_eq!(
        open(&key, &sealed, other_compartment),
        Err(CryptoError::CannotOpen)
    );
}
