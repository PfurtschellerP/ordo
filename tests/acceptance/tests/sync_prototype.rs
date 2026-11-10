//! Sync prototype: the scenarios of tests/acceptance/features/ as executable tests.

use ordo_acceptance::World;
use ordo_contracts::TriageCategory::{self, Immediate, Minor, Urgent};
use ordo_core::{Hlc, Payload, SignedEvent};
use ordo_med_patients::{self as patients, PatientRegister, Registered, Retriaged};
use ordo_sync::SyncError;
use ordo_sync::message::{ClientMessage, Rejection};
use ordo_units::{self as units, StatusBoard, StatusChanged};

/// Records a clinical patient event on `user`'s device, one second after the last.
fn clinical(world: &mut World, user: &str, event_type: &str, payload: &[u8]) {
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(user)
        .record(
            &incident,
            patients::COMPARTMENT,
            patients::MODULE,
            event_type,
            1,
            payload,
            now,
        )
        .unwrap();
    world.tick(1_000);
}

fn register(world: &mut World, user: &str, tag: &str, category: TriageCategory) {
    let payload = Registered {
        tag: tag.into(),
        category,
        uhs: "UHS 1".into(),
        algorithm: None,
    }
    .encode()
    .unwrap();
    clinical(world, user, patients::REGISTERED, &payload);
}

fn retriage(world: &mut World, user: &str, tag: &str, category: TriageCategory) {
    let payload = Retriaged {
        tag: tag.into(),
        category,
        algorithm: None,
    }
    .encode()
    .unwrap();
    clinical(world, user, patients::RETRIAGED, &payload);
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
    let ben_hello = world.hello("ben", 900);
    world.hub.handle(900, ben_hello, world.now);
    let anna_hello = world.hello("anna", 901);
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
    for out in world.hub.connect(950) {
        tablet.receive(out.message, world.now).unwrap();
    }
    let hello = tablet.hello().expect("answer to the challenge");
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
    let mut hello = world.hello("ben", 1);
    if let ClientMessage::Hello(h) = &mut hello {
        h.user = "anna@ordo.drk-musterstadt.de".parse().unwrap(); // Ben's device, Anna's name
    }
    let out = world.hub.handle(1, hello, world.now);
    assert_eq!(
        out[0].message,
        ServerMessage::Refused(Rejection::UnknownDevice)
    );
}

// [itest->dsn~device-proof~1]
/// A connection only gets data after the device proved its key: a wrong proof, a hello
/// without challenge and a replayed hello are refused, and nothing is delivered.
#[test]
fn only_a_device_that_proves_its_key_gets_data() {
    use ordo_sync::message::ServerMessage;
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Immediate);
    world.sync();
    let refused = |out: &[ordo_sync::Outgoing]| {
        matches!(
            out,
            [o] if o.message == ServerMessage::Refused(Rejection::NotAuthenticated)
        )
    };

    // A forged proof: someone knows Anna's user and device ID, but not her key.
    world.set_online("anna", false);
    let mut forged = world.hello("anna", 10);
    if let ClientMessage::Hello(h) = &mut forged {
        h.proof[0] ^= 1;
    }
    let replay = forged.clone();
    assert!(
        refused(&world.hub.handle(10, forged, world.now)),
        "wrong proof"
    );

    // A hello on a connection that never got a challenge.
    let genuine = world.hello("anna", 11);
    assert!(
        refused(&world.hub.handle(12, genuine.clone(), world.now)),
        "no challenge"
    );

    // A genuine hello replayed on another connection: the challenge differs.
    world.hub.connect(13);
    assert!(
        refused(&world.hub.handle(13, genuine, world.now)),
        "replayed proof"
    );
    assert!(
        refused(&world.hub.handle(14, replay, world.now)),
        "replayed forgery"
    );

    // The real device still connects and gets its data, keys included.
    let hello = world.hello("anna", 15);
    let out = world.hub.handle(15, hello, world.now);
    assert!(matches!(&out[0].message, ServerMessage::Welcome(w) if !w.keys.is_empty()));
}

/// A hub restored from its recorded changes continues exactly where it stopped:
/// events, data keys and derived state.
#[test]
fn hub_restored_from_its_changes_continues_seamlessly() {
    use ordo_sync::Change;
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Immediate);
    world.sync();
    let changes = world.hub.take_changes();
    assert!(
        changes.iter().any(|c| matches!(c, Change::Key { .. })),
        "the data key must be recorded"
    );
    assert!(changes.iter().any(|c| matches!(c, Change::Event { .. })));

    // A fresh world (new hub) restored from the recorded changes.
    let mut restored = World::new();
    for change in &changes {
        if let Change::Key {
            stream,
            key_id,
            key,
        } = change
        {
            restored
                .hub
                .restore_key(stream.clone(), key_id.clone(), key.clone());
        }
    }
    for change in changes {
        if let Change::Event { stream, event } = change {
            restored.hub.restore_event(stream, event).unwrap();
        }
    }
    assert!(
        restored.hub.take_changes().is_empty(),
        "restoring records nothing new"
    );
    restored.sync();
    assert_eq!(
        register_of(&mut restored, "clara")
            .patient("DE-0042")
            .unwrap()
            .category(),
        Immediate
    );
    restored.tick(60_000); // the test's deterministic randomness would repeat event IDs
    register(&mut restored, "anna", "DE-0043", Urgent);
    restored.sync();
    assert_eq!(
        counts_of(&mut restored, "ben"),
        [(1, 1, 0)],
        "derived state was restored"
    );
}

/// Reusing an event ID for different content is rejected, resending is not.
#[test]
fn reused_event_id_with_other_content_is_rejected() {
    use ordo_sync::message::ServerMessage;
    let mut world = World::new();
    world.sync();
    set_status(&mut world, "ben", "RTW 1", "2", None);
    let ClientMessage::Push(events) = world.replica("ben").push().unwrap() else {
        unreachable!()
    };
    let original = events[0].clone();
    world.sync(); // ordered
    let mut other = original.decode_body().unwrap();
    other.payload = Payload::Plain(
        StatusChanged {
            unit: "RTW 1".into(),
            status: "4".into(),
            replaces: None,
        }
        .encode()
        .unwrap(),
    );
    let signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[3; 32]),
        "device-ben@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    let hello = world.hello("ben", 900);
    world.hub.handle(900, hello, world.now);
    let resend = world
        .hub
        .handle(900, ClientMessage::Push(vec![original]), world.now);
    assert!(
        resend.is_empty(),
        "resending the same event is silently ignored"
    );
    let reuse = world.hub.handle(
        900,
        ClientMessage::Push(vec![signer.sign(&other).unwrap()]),
        world.now,
    );
    assert!(matches!(
        reuse.as_slice(),
        [o] if matches!(&o.message, ServerMessage::Rejected(r) if r.reason == Rejection::DuplicateId)
    ));
}

// [itest->dsn~local-persistence~1]
/// A device can save its state (e.g. in the browser), load it later and continue.
#[test]
fn replica_state_survives_saving_and_loading() {
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Immediate);
    world.sync();
    world.set_online("anna", false);
    register(&mut world, "anna", "DE-0043", Urgent); // stays in the outbox

    let state = world.replica("anna").snapshot();
    let bytes = ordo_sync::message::encode(&state).unwrap();
    let state: ordo_sync::ReplicaState = ordo_sync::message::decode(&bytes).unwrap();
    let user: ordo_core::UserId = "anna@ordo.drk-musterstadt.de".parse().unwrap();
    let device = world.devices["anna"].replica.device();
    let signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[1; 32]),
        "device-anna@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    let loaded = ordo_sync::Replica::restore(
        user,
        device,
        signer,
        ordo_acceptance::test_entropy(77),
        state,
    )
    .unwrap();
    world.devices.get_mut("anna").unwrap().replica = loaded;

    assert_eq!(world.replica("anna").outbox_len(), 1);
    assert_eq!(
        register_of(&mut world, "anna").len(),
        2,
        "both patients visible after loading"
    );
    world.set_online("anna", true);
    world.sync();
    assert_eq!(world.replica("anna").outbox_len(), 0);
    assert_eq!(register_of(&mut world, "clara").len(), 2);
}

/// Damaged local storage is noticed when loading.
#[test]
fn damaged_saved_state_is_rejected() {
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Immediate);
    world.sync();
    let mut state = world.replica("anna").snapshot();
    let stream = state
        .streams
        .iter_mut()
        .find(|(_, events)| !events.is_empty())
        .unwrap();
    let last = stream.1[0].event.body.len() - 1;
    stream.1[0].event.body[last] ^= 1;
    let user: ordo_core::UserId = "anna@ordo.drk-musterstadt.de".parse().unwrap();
    let device = world.devices["anna"].replica.device();
    let signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[1; 32]),
        "device-anna@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    let result = ordo_sync::Replica::restore(
        user,
        device,
        signer,
        ordo_acceptance::test_entropy(77),
        state,
    );
    assert!(result.is_err());
}

/// Several on-site care units and units (vehicles, teams) in one incident.
// [itest->dsn~uhs-registry~1]
#[test]
fn several_uhs_and_units_in_one_incident() {
    use ordo_med_patients::{UhsCreated, UhsKind};
    use ordo_units::{UnitAdded, UnitRegister};
    let mut world = World::new();
    world.sync();
    let incident = world.incident.clone();
    for (id, name, capacity) in [("uhs-a", "UHS Bühne Nord", 10), ("uhs-b", "UHS Eingang", 5)] {
        let payload = UhsCreated {
            id: id.into(),
            name: name.into(),
            kind: UhsKind::Uhs,
            capacity,
        }
        .encode()
        .unwrap();
        let now = world.now;
        world
            .replica("anna")
            .record(
                &incident,
                "incident",
                patients::UHS_MODULE,
                patients::UHS_CREATED,
                1,
                &payload,
                now,
            )
            .unwrap();
        world.tick(1_000);
    }
    let add = |world: &mut World, user: &str, unit: &str, unit_type: &str| {
        let payload = UnitAdded {
            unit: unit.into(),
            unit_type: unit_type.into(),
            organisation: "DRK Musterstadt".into(),
            service: None,
            strength: None,
        }
        .encode()
        .unwrap();
        let (incident, now) = (world.incident.clone(), world.now);
        world
            .replica(user)
            .record(
                &incident,
                "incident",
                units::MODULE,
                units::ADDED,
                1,
                &payload,
                now,
            )
            .unwrap();
        world.tick(1_000);
    };
    add(&mut world, "ben", "RTW 1", "RTW");
    add(&mut world, "ben", "San-Trupp 2", "Sanitätstrupp");
    for (tag, category, uhs) in [
        ("DE-0042", Immediate, "uhs-a"),
        ("DE-0043", Urgent, "uhs-b"),
        ("DE-0044", Minor, "uhs-b"),
    ] {
        let payload = Registered {
            tag: tag.into(),
            category,
            uhs: uhs.into(),
            algorithm: None,
        }
        .encode()
        .unwrap();
        let now = world.now;
        world
            .replica("anna")
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
    set_status(&mut world, "ben", "RTW 1", "3", None);
    world.sync();

    // Command sees both UHS with names and capacity, and counts per UHS.
    let events = world.replica("ben").events(&incident, "incident").unwrap();
    let list = patients::uhs_list(&events);
    assert_eq!(
        list.iter()
            .map(|u| (u.name.as_str(), u.capacity))
            .collect::<Vec<_>>(),
        [("UHS Bühne Nord", 10), ("UHS Eingang", 5)]
    );
    let counts = patients::latest_counts(&events);
    assert_eq!(
        (
            counts["uhs-a"].immediate,
            counts["uhs-b"].urgent,
            counts["uhs-b"].minor
        ),
        (1, 1, 1)
    );

    // Units with type, organisation and status, on every device.
    let register = UnitRegister::from_events(&events);
    assert_eq!(register.units.len(), 2);
    assert_eq!(register.units["RTW 1"].status.as_ref().unwrap().status, "3");
    assert_eq!(register.units["San-Trupp 2"].unit_type, "Sanitätstrupp");
    let anna_events = world.replica("anna").events(&incident, "incident").unwrap();
    assert_eq!(UnitRegister::from_events(&anna_events), register);
}

// [itest->dsn~access-timeout~1]
/// A login does not last forever: the device learns when it ends, an open connection is
/// cut when it passes, a new one is refused, and a renewal makes the device work again.
#[test]
fn access_ends_when_the_login_expires_and_returns_after_renewal() {
    use ordo_sync::message::ServerMessage;
    let mut world = World::new();
    world.sync();
    let anna: ordo_core::UserId = "anna@ordo.drk-musterstadt.de".parse().unwrap();
    let minute = 60_000;
    world.hub.set_valid_until(&anna, world.now + minute);
    // Reconnect, so the welcome tells the device when its login ends.
    world.set_online("anna", false);
    world.set_online("anna", true);
    world.sync();
    let start = world.now;
    assert_eq!(world.replica("anna").valid_until(), Some(start + minute));
    assert!(!world.replica("anna").login_expired(start));

    world.tick(minute + 1);
    let now = world.now;
    assert!(
        world.replica("anna").login_expired(now),
        "the device locks itself, also offline"
    );
    let cut = world.hub.expire(now);
    assert!(
        cut.iter()
            .any(|o| o.message == ServerMessage::Refused(Rejection::LoginExpired)),
        "the open connection is cut: {cut:?}"
    );
    let hello = world.hello("anna", 77);
    let out = world.hub.handle(77, hello, now);
    assert_eq!(
        out[0].message,
        ServerMessage::Refused(Rejection::LoginExpired)
    );

    // Re-verified (login or background renewal): the device works again.
    world.hub.set_valid_until(&anna, now + 48 * 60 * minute);
    let hello = world.hello("anna", 78);
    let out = world.hub.handle(78, hello, now);
    assert!(
        matches!(&out[0].message, ServerMessage::Welcome(w) if w.valid_until == Some(now + 48 * 60 * minute))
    );
}

// [itest->dsn~device-revocation~1]
/// People see and revoke their own devices, an administrator all of them. A revoked device
/// is cut off at once and refused from then on, even with a valid proof and after a
/// restart of the server.
#[test]
fn revoked_devices_are_cut_off_and_refused() {
    use ordo_sync::Change;
    use ordo_sync::message::ServerMessage;
    let mut world = World::new();
    world.add_device("dora", 9);
    world.sync();
    let anna = world.replica("anna").device();
    let ben = world.replica("ben").device();
    let devices = |out: &[ordo_sync::Outgoing]| -> Vec<ordo_core::DeviceId> {
        match out {
            [o] => match &o.message {
                ServerMessage::Devices(list) => list.iter().map(|d| d.device).collect(),
                other => panic!("expected the device list, got {other:?}"),
            },
            other => panic!("expected one message, got {other:?}"),
        }
    };

    // Anna sees only her own device and may not revoke Ben's.
    let hello = world.hello("anna", 50);
    let out = world.hub.handle(50, hello, world.now);
    assert!(matches!(&out[0].message, ServerMessage::Welcome(w) if w.permissions.is_empty()));
    let now = world.now;
    assert_eq!(
        devices(&world.hub.handle(50, ClientMessage::ListDevices, now)),
        [anna]
    );
    let out = world.hub.handle(50, ClientMessage::RevokeDevice(ben), now);
    assert_eq!(out[0].message, ServerMessage::Refused(Rejection::Forbidden));

    // The administrator sees all devices and revokes Anna's.
    let hello = world.hello("dora", 60);
    let out = world.hub.handle(60, hello, now);
    assert!(
        matches!(&out[0].message, ServerMessage::Welcome(w) if w.permissions == ["manage_devices"])
    );
    assert_eq!(
        devices(&world.hub.handle(60, ClientMessage::ListDevices, now)).len(),
        4
    );
    let out = world.hub.handle(60, ClientMessage::RevokeDevice(anna), now);
    let cut: Vec<_> = out
        .iter()
        .filter(|o| o.message == ServerMessage::Refused(Rejection::Revoked))
        .map(|o| o.session)
        .collect();
    assert_eq!(
        cut.len(),
        2,
        "both connections of Anna's device are cut: {out:?}"
    );
    assert!(
        out.iter()
            .any(|o| o.session == 60
                && matches!(&o.message, ServerMessage::Devices(l) if l.len() == 3)),
        "the administrator gets the new list"
    );
    let changes = world.hub.take_changes();
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, Change::Revoked { device } if *device == anna)),
        "the revocation is stored"
    );

    // A valid proof does not help: the device is told it was revoked.
    world.set_online("anna", false);
    let hello = world.hello("anna", 70);
    let out = world.hub.handle(70, hello, now);
    assert_eq!(out[0].message, ServerMessage::Refused(Rejection::Revoked));

    // Ben revokes his own device (e.g. a lost phone): his connection ends, too.
    let hello = world.hello("ben", 80);
    world.hub.handle(80, hello, now);
    let out = world.hub.handle(80, ClientMessage::RevokeDevice(ben), now);
    assert!(
        out.iter()
            .all(|o| o.message == ServerMessage::Refused(Rejection::Revoked)),
        "{out:?}"
    );

    // After a restart, the stored revocation still holds.
    let mut restarted = World::new();
    restarted.hub.restore_revoked(anna);
    let hello = restarted.hello("anna", 1);
    let out = restarted.hub.handle(1, hello, restarted.now);
    assert_eq!(out[0].message, ServerMessage::Refused(Rejection::Revoked));
}

fn transport_priority(world: &mut World, user: &str, tag: &str, level: Option<&str>) {
    let payload = patients::TransportPrioritySet {
        tag: tag.into(),
        level: level.map(str::to_owned),
    }
    .encode()
    .unwrap();
    clinical(world, user, patients::TRANSPORT_PRIORITY, &payload);
}

// [itest->dsn~transport-priority~1]
/// Transport priority is documented apart from the triage category, for the categories
/// the profile allows, and ends for good when the patient is re-triaged to a category
/// without it; unharmed people ("Betroffene") are registered and counted apart.
#[test]
fn transport_priority_follows_the_profile_and_ends_on_downgrade() {
    let config = patients::TransportPriorityConfig {
        categories: vec![Immediate, Urgent],
        levels: vec![patients::TransportPriorityLevel {
            id: "high".into(),
            label: "Transportpriorität".into(),
        }],
    };
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0001", Immediate);
    transport_priority(&mut world, "anna", "DE-0001", Some("high"));
    register(&mut world, "anna", "DE-0002", TriageCategory::Uninjured);
    world.sync();
    let reg = register_of(&mut world, "clara");
    assert_eq!(
        reg.patient("DE-0001").unwrap().transport_priority(&config),
        Some("high"),
        "seen on another medic's device"
    );
    assert_eq!(
        reg.counts().values().map(|c| c.uninjured).sum::<u32>(),
        1,
        "Betroffene are counted apart"
    );

    retriage(&mut world, "anna", "DE-0001", Minor);
    retriage(&mut world, "anna", "DE-0001", Urgent);
    world.sync();
    let reg = register_of(&mut world, "clara");
    assert_eq!(
        reg.patient("DE-0001").unwrap().transport_priority(&config),
        None,
        "a downgrade ends the priority for good"
    );
}

// [itest->dsn~treatment-places~1]
/// Treatment places are shared incident data (command sees them); which patient lies on
/// which place is clinical and reaches medics only.
#[test]
fn patients_are_assigned_to_treatment_places() {
    let mut world = World::new();
    world.sync();
    let (incident, now) = (world.incident.clone(), world.now);
    let places = patients::PlacesSet {
        uhs: "UHS 1".into(),
        places: (1..=3)
            .map(|i| patients::Place {
                id: format!("p{i}"),
                label: format!("Platz {i}"),
            })
            .collect(),
    }
    .encode()
    .unwrap();
    world
        .replica("anna")
        .record(
            &incident,
            patients::UHS_COMPARTMENT,
            patients::UHS_MODULE,
            patients::UHS_PLACES,
            1,
            &places,
            now,
        )
        .unwrap();
    register(&mut world, "anna", "DE-0007", Urgent);
    let assigned = patients::PlaceAssigned {
        tag: "DE-0007".into(),
        place: Some("p2".into()),
    }
    .encode()
    .unwrap();
    let now = world.now;
    world
        .replica("anna")
        .record(
            &incident,
            patients::COMPARTMENT,
            patients::MODULE,
            patients::PLACE_ASSIGNED,
            1,
            &assigned,
            now,
        )
        .unwrap();
    world.sync();

    assert_eq!(
        register_of(&mut world, "clara")
            .patient("DE-0007")
            .unwrap()
            .place
            .as_deref(),
        Some("p2"),
        "another medic sees the place"
    );
    let shared = world
        .replica("ben")
        .events(&incident, patients::UHS_COMPARTMENT)
        .unwrap();
    assert_eq!(
        patients::places(&shared)["UHS 1"].len(),
        3,
        "command sees the places"
    );
    assert!(
        world
            .replica("ben")
            .events(&incident, patients::COMPARTMENT)
            .map_or(true, |e| e.is_empty()),
        "but not who lies where"
    );
}

// [itest->dsn~device-storage~1]
/// Device storage keeps events row by row: the state without events plus each stream's
/// events restore the same replica, and after a change only the new events are written.
#[test]
fn replica_is_stored_row_by_row() {
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0042", Immediate);
    world.sync();
    let heads = world.replica("anna").stream_heads();
    let saved: Vec<(ordo_sync::StreamKey, u64)> = heads.clone();

    register(&mut world, "clara", "DE-0043", Urgent);
    world.sync();
    let replica = world.replica("anna");
    let new: usize = saved
        .iter()
        .map(|(key, seq)| replica.events_after(key, *seq).len())
        .sum();
    assert_eq!(
        new, 2,
        "only the new events are written: the registration and the updated counts"
    );

    // Reassemble from rows, as the device storage does when the app starts.
    let mut state = replica.snapshot_without_events();
    for (key, events) in &mut state.streams {
        *events = replica.events_after(key, 0).to_vec();
    }
    let user: ordo_core::UserId = "anna@ordo.drk-musterstadt.de".parse().unwrap();
    let device = replica.device();
    let signer = ordo_crypto::EventSigner::new(
        ordo_crypto::ed25519_dalek::SigningKey::from_bytes(&[1; 32]),
        "device-anna@ordo.drk-musterstadt.de".parse().unwrap(),
    );
    let loaded = ordo_sync::Replica::restore(
        user,
        device,
        signer,
        ordo_acceptance::test_entropy(78),
        state,
    )
    .unwrap();
    let original = world.replica("anna");
    let mut a = loaded.stream_heads();
    let mut b = original.stream_heads();
    a.sort();
    b.sort();
    assert_eq!(a, b, "the same streams up to the same events");
    for (key, _) in &a {
        assert_eq!(loaded.events_after(key, 0), original.events_after(key, 0));
    }
    assert_eq!(loaded.outbox_len(), original.outbox_len());
}

// [itest->dsn~uhs-discharge~1]
/// A discharged patient leaves the unit's load; command sees how many were discharged,
/// never who. A readmission brings the patient back on every device.
#[test]
fn discharge_and_readmission_reach_every_device_and_the_counts() {
    let mut world = World::new();
    world.sync();
    register(&mut world, "anna", "DE-0010", Immediate);
    register(&mut world, "anna", "DE-0011", Minor);
    world.sync();
    let discharged = patients::Discharged {
        tag: "DE-0010".into(),
        outcome: patients::Outcome::Transported,
        transport: Some("01JTRANSPORT".into()),
    }
    .encode()
    .unwrap();
    clinical(&mut world, "anna", patients::DISCHARGED, &discharged);
    world.sync();

    let seen = register_of(&mut world, "clara");
    assert!(
        !seen.patient("DE-0010").unwrap().is_present(),
        "another medic sees the discharge"
    );
    assert_eq!(
        counts_of(&mut world, "ben"),
        [(0, 0, 1)],
        "the load leaves out discharged patients"
    );
    let incident = world.incident.clone();
    let events = world.replica("ben").events(&incident, "incident").unwrap();
    assert_eq!(
        patients::latest_counts(&events)["UHS 1"].discharged,
        1,
        "command sees how many left"
    );

    let readmitted = patients::Readmitted {
        tag: "DE-0010".into(),
    }
    .encode()
    .unwrap();
    clinical(&mut world, "clara", patients::READMITTED, &readmitted);
    world.sync();
    assert!(
        register_of(&mut world, "anna")
            .patient("DE-0010")
            .unwrap()
            .is_present()
    );
    assert_eq!(counts_of(&mut world, "ben"), [(1, 0, 1)]);
}

/// Records a unit event on `user`'s device, one second after the last.
fn unit_event(world: &mut World, user: &str, event_type: &str, payload: &[u8]) {
    let (incident, now) = (world.incident.clone(), world.now);
    let schema = if event_type == units::ADDED {
        units::ADDED_SCHEMA
    } else {
        1
    };
    world
        .replica(user)
        .record(
            &incident,
            units::COMPARTMENT,
            units::MODULE,
            event_type,
            schema,
            payload,
            now,
        )
        .unwrap();
    world.tick(1_000);
}

// [itest->dsn~unit-register~2]
/// Units with service and strength, a crew and a release reach every device the same way,
/// and the strength on scene counts only the units present.
#[test]
fn units_with_strength_crew_and_release_reach_every_device() {
    let mut world = World::new();
    world.sync();
    for (unit, org, service, strength) in [
        ("RTW 1", "DRK", "rescue", [0, 1, 1]),
        ("HLF 1", "FF Musterstadt", "fire", [0, 1, 8]),
    ] {
        let a = units::UnitAdded {
            unit: unit.into(),
            unit_type: "RTW".into(),
            organisation: org.into(),
            service: Some(service.into()),
            strength: Some(units::Strength::from(strength)),
        }
        .encode()
        .unwrap();
        unit_event(&mut world, "ben", units::ADDED, &a);
    }
    let crew = units::CrewSet {
        unit: "RTW 1".into(),
        members: vec![units::CrewMember {
            name: "anna@ordo.example.org".into(),
            qualifications: vec!["paramedic".into()],
        }],
    }
    .encode()
    .unwrap();
    unit_event(&mut world, "ben", units::CREW_SET, &crew);
    let release = units::Released {
        unit: "HLF 1".into(),
    }
    .encode()
    .unwrap();
    unit_event(&mut world, "ben", units::RELEASED, &release);
    world.sync();

    let incident = world.incident.clone();
    let seen = |world: &mut World, user: &str| {
        units::UnitRegister::from_events(
            &world
                .replica(user)
                .events(&incident, units::COMPARTMENT)
                .unwrap(),
        )
    };
    let anna = seen(&mut world, "anna");
    assert_eq!(
        anna,
        seen(&mut world, "clara"),
        "every device computes the same register"
    );
    assert!(!anna.units["HLF 1"].is_present());
    assert_eq!(anna.units["RTW 1"].crew.len(), 1);
    let s = anna.strength_on_scene();
    assert_eq!(
        (s.units, s.total.total()),
        (1, 2),
        "the released fire engine is not counted"
    );
}

// [itest->dsn~tasks~1]
/// A task given by the command reaches every device; its status moves on and a due
/// follow-up comes first in the open list.
#[test]
fn tasks_reach_every_device_with_status_and_follow_up() {
    use ordo_tasks::{self as tasks, TaskGiven, TaskList, TaskStatus, TaskUpdated};
    let mut world = World::new();
    world.sync();
    let record = |world: &mut World, event_type: &str, payload: Vec<u8>| {
        let (incident, now) = (world.incident.clone(), world.now);
        world
            .replica("ben")
            .record(
                &incident,
                tasks::COMPARTMENT,
                tasks::MODULE,
                event_type,
                1,
                &payload,
                now,
            )
            .unwrap();
        world.tick(1_000);
    };
    for (id, follow_up) in [("t1", None), ("t2", Some(1))] {
        let t = TaskGiven {
            task: id.into(),
            unit: Some("RTW 1".into()),
            sector: None,
            text: "Lagemeldung abgeben".into(),
            template: None,
            follow_up,
        };
        record(&mut world, tasks::GIVEN, t.encode().unwrap());
    }
    let u = TaskUpdated {
        task: "t1".into(),
        status: Some(TaskStatus::InProgress),
        follow_up: None,
    };
    record(&mut world, tasks::UPDATED, u.encode().unwrap());
    world.sync();

    let incident = world.incident.clone();
    let list = TaskList::from_events(
        &world
            .replica("clara")
            .events(&incident, tasks::COMPARTMENT)
            .unwrap(),
    );
    assert_eq!(list.tasks["t1"].status, TaskStatus::InProgress);
    let open: Vec<&str> = list
        .open(world.now)
        .iter()
        .map(|t| t.given.task.as_str())
        .collect();
    assert_eq!(open, ["t2", "t1"], "the due follow-up first");
}
