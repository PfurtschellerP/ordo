//! Sync prototype: the scenarios of docs/requirements/acceptance/ as executable tests.

use ordo_acceptance::World;
use ordo_contracts::TriageCategory::{self, Immediate, Minor, Urgent};
use ordo_core::{Hlc, Payload, SignedEvent};
use ordo_med_patients::{self as patients, PatientRegister, Registered, Retriaged};
use ordo_sync::SyncError;
use ordo_sync::message::{ClientMessage, Rejection};
use ordo_units::{self as units, StatusBoard, StatusChanged};

fn register(world: &mut World, user: &str, tag: &str, category: TriageCategory) {
    let payload = Registered {
        tag: tag.into(),
        category,
        uhs: "UHS 1".into(),
    }
    .encode()
    .unwrap();
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(user)
        .record(
            &incident,
            patients::COMPARTMENT,
            patients::MODULE,
            patients::REGISTERED,
            1,
            &payload,
            now,
        )
        .unwrap();
    world.tick(1_000);
}

fn retriage(world: &mut World, user: &str, tag: &str, category: TriageCategory) {
    let payload = Retriaged {
        tag: tag.into(),
        category,
    }
    .encode()
    .unwrap();
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(user)
        .record(
            &incident,
            patients::COMPARTMENT,
            patients::MODULE,
            patients::RETRIAGED,
            1,
            &payload,
            now,
        )
        .unwrap();
    world.tick(1_000);
}

fn set_status(world: &mut World, user: &str, unit: &str, status: &str, replaces: Option<Hlc>) {
    let payload = StatusChanged {
        unit: unit.into(),
        status: status.into(),
        replaces,
    }
    .encode()
    .unwrap();
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(user)
        .record(
            &incident,
            units::COMPARTMENT,
            units::MODULE,
            units::STATUS_CHANGED,
            1,
            &payload,
            now,
        )
        .unwrap();
    world.tick(1_000);
}

fn register_of(world: &mut World, user: &str) -> PatientRegister {
    let incident = world.incident.clone();
    PatientRegister::from_events(
        &world
            .replica(user)
            .events(&incident, patients::COMPARTMENT)
            .unwrap(),
    )
}

fn board_of(world: &mut World, user: &str) -> StatusBoard {
    let incident = world.incident.clone();
    StatusBoard::from_events(
        &world
            .replica(user)
            .events(&incident, units::COMPARTMENT)
            .unwrap(),
    )
}

fn counts_of(world: &mut World, user: &str) -> Vec<(u32, u32, u32)> {
    let incident = world.incident.clone();
    let events = world.replica(user).events(&incident, "incident").unwrap();
    patients::latest_counts(&events)
        .values()
        .map(|c| (c.immediate, c.urgent, c.minor))
        .collect()
}

/// Scenario: Register a patient with minimal data / while offline.
// [itest->dsn~outbox-and-resend~1]
#[test]
fn offline_registration_appears_everywhere_after_reconnect() {
    let mut world = World::new();
    world.sync(); // first connection: keys and subscriptions
    world.set_online("anna", false);

    register(&mut world, "anna", "DE-0043", Minor);
    let anna = register_of(&mut world, "anna");
    assert_eq!(
        anna.patient("DE-0043").unwrap().category(),
        Minor,
        "visible locally at once"
    );
    assert_eq!(world.replica("anna").outbox_len(), 1);
    world.sync();
    assert!(
        register_of(&mut world, "clara")
            .patient("DE-0043")
            .is_none(),
        "not yet synced"
    );

    world.set_online("anna", true);
    world.sync();
    assert_eq!(
        world.replica("anna").outbox_len(),
        0,
        "confirmed by the server"
    );
    assert_eq!(
        register_of(&mut world, "clara")
            .patient("DE-0043")
            .unwrap()
            .category(),
        Minor
    );
}

/// Scenario: Command sees patient counts but never patient data.
// [itest->dsn~compartment-scoped-sync~1]
// [itest->dsn~server-derived-connectors~1]
#[test]
fn command_sees_counts_but_never_patient_data() {
    let mut world = World::new();
    world.sync();
    let incident = world.incident.clone();
    assert!(
        !world.replica("ben").is_granted(&incident, "med.clinical"),
        "access denied"
    );
    assert!(world.replica("ben").is_granted(&incident, "incident"));

    register(&mut world, "anna", "DE-0042", Immediate);
    register(&mut world, "anna", "DE-0043", Urgent);
    register(&mut world, "clara", "DE-0044", Urgent);
    world.sync();

    assert_eq!(
        counts_of(&mut world, "ben"),
        [(1, 2, 0)],
        "UHS 1: SK I 1, SK II 2, SK III 0"
    );

    // Ben's device stores nothing from med.clinical, and no tag number anywhere.
    let stored: Vec<&SignedEvent> = world.devices["ben"].replica.stored_events().collect();
    assert!(!stored.is_empty());
    for event in stored {
        let body = event.decode_body().unwrap();
        assert_eq!(body.compartment.as_str(), "incident");
        let raw = String::from_utf8_lossy(&event.body);
        assert!(
            !raw.contains("DE-004"),
            "a triage tag number reached the command tablet"
        );
    }
    // And Ben cannot write clinical data.
    let now = world.now;
    let error = world
        .replica("ben")
        .record(
            &incident,
            "med.clinical",
            patients::MODULE,
            patients::REGISTERED,
            1,
            b"x",
            now,
        )
        .unwrap_err();
    assert!(matches!(error, SyncError::NoAccess { .. }));
}

/// Clinical payloads are sealed: the server stores only ciphertext in med.clinical.
// [itest->dsn~sealed-payloads~1]
#[test]
fn clinical_payloads_are_sealed_on_the_server() {
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Immediate);
    world.sync();
    let incident = world.incident.clone();
    let stream = world
        .hub
        .stream(&incident, &"med.clinical".parse().unwrap());
    assert_eq!(stream.len(), 1);
    let body = stream[0].event.decode_body().unwrap();
    assert!(matches!(body.payload, Payload::Sealed(_)));
    assert!(!String::from_utf8_lossy(&stream[0].event.body).contains("DE-0042"));
}

/// Scenario: Re-triage keeps the history; counts follow.
// [itest->dsn~patient-triage-history~1]
// [itest->dsn~server-derived-connectors~1]
#[test]
fn retriage_keeps_history_and_updates_counts() {
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Urgent);
    world.sync();
    retriage(&mut world, "clara", "DE-0042", Immediate);
    world.sync();

    let register = register_of(&mut world, "anna");
    let patient = register.patient("DE-0042").unwrap();
    assert_eq!(patient.category(), Immediate);
    let history: Vec<(TriageCategory, &str)> = patient
        .history
        .iter()
        .map(|e| (e.category, e.by.localpart()))
        .collect();
    assert_eq!(history, [(Urgent, "anna"), (Immediate, "clara")]);
    assert!(patient.history[0].at < patient.history[1].at);
    assert_eq!(counts_of(&mut world, "ben"), [(1, 0, 0)]);
}

/// Concurrent offline changes converge, and the lost update is visible.
// [itest->dsn~lost-update-detection~1]
#[test]
fn concurrent_status_changes_converge_and_show_lost_update() {
    let mut world = World::new();
    world.sync();
    set_status(&mut world, "ben", "RTW 1", "2", None);
    world.sync();
    let base = board_of(&mut world, "ben").units["RTW 1"].since;

    world.set_online("ben", false);
    world.set_online("clara", false);
    set_status(&mut world, "ben", "RTW 1", "4", Some(base));
    set_status(&mut world, "clara", "RTW 1", "3", Some(base)); // later, without seeing "4"
    world.set_online("clara", true);
    world.sync();
    world.set_online("ben", true);
    world.sync();

    let ben = board_of(&mut world, "ben");
    let clara = board_of(&mut world, "clara");
    assert_eq!(ben, clara, "all devices reach the same state");
    assert_eq!(ben.units["RTW 1"].status, "3", "last writer by HLC wins");
    assert_eq!(ben.lost_updates.len(), 1);
    assert_eq!(ben.lost_updates[0].overwritten, "4");

    // A normal sequence (each change saw the previous one) is no conflict.
    let latest = ben.units["RTW 1"].since;
    set_status(&mut world, "anna", "RTW 1", "1", Some(latest));
    world.sync();
    assert_eq!(board_of(&mut world, "anna").lost_updates.len(), 1);
}

/// Sending again after a lost confirmation does not duplicate events.
// [itest->dsn~outbox-and-resend~1]
#[test]
fn resend_after_lost_confirmation_is_idempotent() {
    let mut world = World::new();
    world.sync();
    world.devices.get_mut("anna").unwrap().drop_incoming = true;
    register(&mut world, "anna", "DE-0042", Urgent);
    world.sync(); // server orders the event, but anna never hears back
    assert_eq!(world.replica("anna").outbox_len(), 1);

    world.devices.get_mut("anna").unwrap().drop_incoming = false;
    world.set_online("anna", false);
    world.set_online("anna", true);
    world.sync(); // reconnect: hello catches up, push is sent again
    let incident = world.incident.clone();
    assert_eq!(
        world
            .hub
            .stream(&incident, &"med.clinical".parse().unwrap())
            .len(),
        1
    );
    assert_eq!(world.replica("anna").outbox_len(), 0);
    assert_eq!(register_of(&mut world, "anna").len(), 1);
}

/// The server rejects forged, unauthorised and unsealed events.
// [itest->dsn~compartment-scoped-sync~1]
// [itest->dsn~sealed-payloads~1]
#[test]
fn server_rejects_forged_unauthorised_and_unsealed_events() {
    use ordo_sync::message::ServerMessage;
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Urgent);
    let ClientMessage::Push(mut events) = world.replica("anna").push().unwrap() else {
        unreachable!()
    };
    let genuine = events.remove(0);
    let reason = |world: &mut World, session, event: SignedEvent| {
        let out = world
            .hub
            .handle(session, ClientMessage::Push(vec![event]), world.now);
        match out.as_slice() {
            [o] => match &o.message {
                ServerMessage::Rejected(r) => r.reason,
                other => panic!("expected rejection, got {other:?}"),
            },
            other => panic!("expected one message, got {other:?}"),
        }
    };
    let ben_hello = world.replica("ben").hello();
    world.hub.handle(900, ben_hello, world.now);
    let anna_hello = world.replica("anna").hello();
    world.hub.handle(901, anna_hello, world.now);

    // Anna's event sent over Ben's connection.
    assert_eq!(
        reason(&mut world, 900, genuine.clone()),
        Rejection::WrongAuthor
    );
    // Changed body: signature no longer valid.
    let mut forged = genuine.clone();
    let last = forged.body.len() - 1;
    forged.body[last] ^= 1;
    assert!(matches!(
        reason(&mut world, 901, forged),
        Rejection::BadSignature | Rejection::Malformed
    ));

    // Ben writes into the incident compartment — fine — but not into med.clinical.
    let incident = world.incident.clone();
    let mut body = genuine.decode_body().unwrap();
    body.author.user = "ben@ordo.drk-musterstadt.de".parse().unwrap();
    body.author.device = world.devices["ben"].replica.device();
    body.payload = Payload::Plain(b"x".to_vec());
    let ben_signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[3; 32]),
        "device-ben@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    assert_eq!(
        reason(&mut world, 900, ben_signer.sign(&body).unwrap()),
        Rejection::Forbidden
    );

    // Anna sends an unsealed payload into med.clinical.
    let mut plain = genuine.decode_body().unwrap();
    plain.payload = Payload::Plain(b"DE-0042".to_vec());
    let anna_signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[1; 32]),
        "device-anna@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    assert_eq!(
        reason(&mut world, 901, anna_signer.sign(&plain).unwrap()),
        Rejection::WrongSealing
    );
    assert!(
        world
            .hub
            .stream(&incident, &"med.clinical".parse().unwrap())
            .is_empty()
    );
}

/// A device detects changed events from the server through the hash chain.
#[test]
fn device_detects_a_changed_event_in_the_chain() {
    use ordo_sync::message::{Delivery, ServerMessage};
    let mut world = World::new();
    world.sync();
    set_status(&mut world, "ben", "RTW 1", "2", None);
    world.sync();
    let incident = world.incident.clone();
    let mut sequenced = world.hub.stream(&incident, &"incident".parse().unwrap())[0].clone();
    // Re-sign a changed body with Ben's key: the signature is fine, the chain is not.
    let mut body = sequenced.event.decode_body().unwrap();
    body.payload = Payload::Plain(b"changed".to_vec());
    let signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[3; 32]),
        "device-ben@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    sequenced.event = signer.sign(&body).unwrap();
    // A second, freshly paired device of Ben receives the changed event.
    let ben: ordo_core::UserId = "ben@ordo.drk-musterstadt.de".parse().unwrap();
    let device = ordo_core::DeviceId(ordo_core::Ulid::from_parts(world.now, 8).unwrap());
    let key = ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[8; 32]);
    world
        .hub
        .register_device(device, ben.clone(), key.verifying_key());
    let mut tablet = ordo_sync::Replica::new(
        ben,
        device,
        ordo_crypto::EventSigner::new(key, "device-ben-2@ordo.drk-musterstadt.de".parse().unwrap()),
        ordo_acceptance::test_entropy(8),
    );
    tablet.subscribe(&incident, "incident");
    let hello = tablet.hello();
    for out in world.hub.handle(950, hello, world.now) {
        if let ServerMessage::Welcome(_) = out.message {
            tablet.receive(out.message, world.now).unwrap();
        }
    }
    let error = tablet
        .receive(
            ServerMessage::Deliver(Delivery {
                incident,
                compartment: "incident".parse().unwrap(),
                events: vec![sequenced],
            }),
            world.now,
        )
        .unwrap_err();
    assert_eq!(error, SyncError::BrokenChain { seq: 1 });
}

/// An unregistered device is refused.
#[test]
fn unknown_device_is_refused() {
    use ordo_sync::message::ServerMessage;
    let mut world = World::new();
    let mut hello = world.replica("ben").hello();
    if let ClientMessage::Hello(h) = &mut hello {
        h.user = "anna@ordo.drk-musterstadt.de".parse().unwrap(); // Ben's device, Anna's name
    }
    let out = world.hub.handle(1, hello, world.now);
    assert_eq!(
        out[0].message,
        ServerMessage::Refused(Rejection::UnknownDevice)
    );
}
