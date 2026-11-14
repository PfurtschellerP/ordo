//! Protocol versions (ADR-0052): the server announces its range, devices old and new keep
//! working, and a version outside the range is refused clearly.

use ordo_acceptance::World;
use ordo_sync::message::{ClientMessage, Rejection, ServerMessage};
use ordo_sync::version::{self, Version};

/// Connects `name` with the hello its replica builds, changed by `change`, and returns
/// the server's answer.
fn connect(
    world: &mut World,
    name: &str,
    session: u64,
    change: impl FnOnce(&mut Option<Version>),
) -> ServerMessage {
    let mut hello = world.hello(name, session);
    let ClientMessage::Hello(h) = &mut hello else {
        panic!("a hello");
    };
    change(&mut h.version);
    let now = world.now;
    world.hub.handle(session, hello, now).remove(0).message
}

// [itest->dsn~protocol-versions~1]
#[test]
fn the_oldest_version_and_no_version_work_and_others_are_refused() {
    let mut world = World::new();
    world.sync();

    // The challenge names the range the server supports.
    let challenge = world.hub.connect(800).remove(0).message;
    assert!(matches!(
        challenge,
        ServerMessage::Challenge(c) if c.versions == Some(version::SYNC)
    ));
    world.hub.disconnect(800);

    // The device asks for the newest shared version, and the welcome confirms it.
    let welcome = connect(&mut world, "anna", 801, |_| {});
    assert!(matches!(&welcome, ServerMessage::Welcome(w) if w.version == Some(version::SYNC.max)));
    assert_eq!(world.hub.session_version(801), Some(version::SYNC.max));

    // The oldest supported version, as an old device would ask.
    let oldest = connect(&mut world, "ben", 802, |v| *v = Some(version::SYNC.min));
    assert!(matches!(&oldest, ServerMessage::Welcome(w) if w.version == Some(version::SYNC.min)));

    // A device from before versions existed sends none: it gets the oldest.
    let none = connect(&mut world, "clara", 803, |v| *v = None);
    assert!(matches!(&none, ServerMessage::Welcome(w) if w.version == Some(version::SYNC.min)));

    // Newer than the server, or older than its range: refused, and nothing else is sent.
    let newer = Version::new(version::SYNC.max.major + 1, 0);
    let refused = connect(&mut world, "anna", 804, |v| *v = Some(newer));
    assert!(matches!(
        refused,
        ServerMessage::Refused(Rejection::UnsupportedVersion)
    ));
    assert_eq!(world.hub.session_version(804), None);
    let older = Version::new(0, 9);
    let refused = connect(&mut world, "anna", 805, |v| *v = Some(older));
    assert!(matches!(
        refused,
        ServerMessage::Refused(Rejection::UnsupportedVersion)
    ));
}

// [utest->dsn~connection-liveness~1]
#[test]
fn a_ping_is_answered_from_version_1_2_on() {
    let mut world = World::new();
    world.sync();
    let now = world.now;
    let welcome = connect(&mut world, "anna", 810, |_| {});
    assert!(matches!(welcome, ServerMessage::Welcome(_)));
    let out = world.hub.handle(810, ClientMessage::Ping, now);
    assert!(matches!(&out[..], [o] if o.session == 810 && o.message == ServerMessage::Pong));

    // A device on 1.1 does not know the answer: it gets none.
    let older = connect(&mut world, "ben", 811, |v| {
        *v = Some(version::Version::new(1, 1))
    });
    assert!(matches!(older, ServerMessage::Welcome(_)));
    assert!(world.hub.handle(811, ClientMessage::Ping, now).is_empty());
}
