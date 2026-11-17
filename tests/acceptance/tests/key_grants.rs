//! Data keys travel only sealed for the device (SEC-19, ADR-0080): a proxy that ends TLS,
//! or a recording of the traffic, never sees one, and cannot swap the device's key.

use ordo_acceptance::World;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_crypto::kem::{self, DeviceKem, SealedKey};
use ordo_sync::message::{self, ClientMessage, KeyGrant, Rejection, ServerMessage};

/// Anna's device seed in the [`World`].
const ANNA_SEED: [u8; 32] = [1; 32];

fn open(device: &DeviceKem, device_id: &[u8; 16], grant: &KeyGrant) -> Option<[u8; 32]> {
    let aad = kem::grant_aad(
        device_id,
        grant.incident.as_str(),
        grant.compartment.as_str(),
        grant.key_id.as_str(),
    );
    let sealed = SealedKey {
        encapsulated: grant.encapsulated.clone(),
        ciphertext: grant.ciphertext.clone(),
    };
    device.open(&sealed, &aad).ok()
}

// [itest->dsn~sealed-key-grants~1]
#[test]
fn data_keys_reach_only_the_device_they_are_sealed_for() {
    let mut world = World::new();
    world.sync();
    let now = world.now;

    let hello = world.hello("anna", 900);
    let ClientMessage::Hello(h) = &hello else {
        panic!("a hello");
    };
    let anna = h.device.ulid().to_bytes();
    let ServerMessage::Welcome(welcome) = world.hub.handle(900, hello, now).remove(0).message
    else {
        panic!("a welcome");
    };
    let grant = welcome
        .keys
        .iter()
        .find(|g| g.compartment.as_str() == "med.clinical")
        .expect("a key for the protected compartment");

    // Anna's device opens it; the key is nowhere in the message as sent.
    let kem = DeviceKem::from_device_seed(&SigningKey::from_bytes(&ANNA_SEED).to_bytes());
    let key = open(&kem, &anna, grant).expect("sealed for Anna's device");
    let sent = message::encode(&ServerMessage::Welcome(welcome.clone())).unwrap();
    assert!(!sent.windows(32).any(|w| w == key), "no key in plain text");

    // Another device opens nothing, not even with Anna's device ID.
    let clara = DeviceKem::from_device_seed(&[2; 32]);
    assert_eq!(open(&clara, &anna, grant), None);

    // A changed grant is refused by the device, and nothing of the welcome is taken.
    let mut changed = welcome.clone();
    changed.keys[0].ciphertext[0] ^= 1;
    assert!(
        world
            .replica("anna")
            .receive(ServerMessage::Welcome(changed), now)
            .is_err()
    );
    world
        .replica("anna")
        .receive(ServerMessage::Welcome(welcome), now)
        .expect("the welcome as sent");
}

// [itest->dsn~sealed-key-grants~1]
#[test]
fn a_device_key_swapped_in_the_hello_is_refused() {
    let mut world = World::new();
    world.sync();
    let now = world.now;

    // Someone in between puts in a key of their own, to open the grants themselves.
    let mut hello = world.hello("anna", 901);
    let ClientMessage::Hello(h) = &mut hello else {
        panic!("a hello");
    };
    h.kem_key = DeviceKem::from_device_seed(&[66; 32]).public_key();
    let out = world.hub.handle(901, hello, now);
    assert!(matches!(
        &out[..],
        [o] if o.message == ServerMessage::Refused(Rejection::NotAuthenticated)
    ));

    // Or none at all, as a device before protocol 1.4 would send.
    let mut hello = world.hello("anna", 902);
    let ClientMessage::Hello(h) = &mut hello else {
        panic!("a hello");
    };
    h.kem_key.clear();
    let out = world.hub.handle(902, hello, now);
    assert!(matches!(
        &out[..],
        [o] if o.message == ServerMessage::Refused(Rejection::NotAuthenticated)
    ));
}
