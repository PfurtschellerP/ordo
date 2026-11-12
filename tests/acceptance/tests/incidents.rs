//! Incidents planned ahead and roles granted per incident (ADR-0046).

use ordo_acceptance::World;
use ordo_contracts::TriageCategory;
use ordo_core::UserId;
use ordo_med_patients::{self as patients, Registered};
use ordo_sync::StaticPolicy;
use ordo_sync::incidents::{self, Planned, Registry, RoleChange};
use ordo_sync::message::{ClientMessage, Rejection, ServerMessage};

fn user(world: &World, name: &str) -> UserId {
    UserId::new(name, &world.server).unwrap()
}

/// Records an incident event on `name`'s device.
fn incident_event(world: &mut World, name: &str, event_type: &str, payload: Vec<u8>) {
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(name)
        .record(
            &incident,
            incidents::COMPARTMENT,
            incidents::MODULE,
            event_type,
            1,
            &payload,
            now,
        )
        .unwrap();
}

fn grant(world: &mut World, by: &str, to: &str, role: &str, granted: bool) {
    let payload = RoleChange {
        user: user(world, to),
        role: role.into(),
    }
    .encode()
    .unwrap();
    let event_type = if granted {
        incidents::ROLE_GRANTED
    } else {
        incidents::ROLE_WITHDRAWN
    };
    incident_event(world, by, event_type, payload);
}

fn register(world: &mut World, name: &str, tag: &str) -> Result<(), ordo_sync::SyncError> {
    let payload = Registered {
        tag: tag.into(),
        category: TriageCategory::Urgent,
        uhs: String::new(),
        algorithm: None,
    }
    .encode()
    .unwrap();
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(name)
        .record(
            &incident,
            patients::COMPARTMENT,
            patients::MODULE,
            patients::REGISTERED,
            1,
            &payload,
            now,
        )
        .map(|_| ())
}

fn clinical_events(world: &mut World, name: &str) -> usize {
    let incident = world.incident.clone();
    world
        .replica(name)
        .events(&incident, patients::COMPARTMENT)
        .map(|e| e.len())
        .unwrap_or(0)
}

// [itest->dsn~incident-roles~1]
/// A planner plans an incident and grants roles; access follows the roster at once: a
/// medic gets clinical data only while she holds the role, and loses it from her device
/// when the role is withdrawn. Only planners and the incident's command grant roles.
#[test]
fn roles_are_granted_per_incident_and_access_follows() {
    let server = "ordo.drk-musterstadt.de".parse().unwrap();
    let dora = UserId::new("dora", &server).unwrap();
    let policy = StaticPolicy::default()
        .planner(&dora)
        .role("medic", &["incident", "med.clinical"])
        .role("commander", &["incident"])
        .assigner("commander");
    let mut world = World::with_policy(policy);
    world.add_device("dora", 9);
    world.sync();
    assert!(
        register(&mut world, "anna", "DE-0001").is_err(),
        "without a role in this incident, Anna has no access"
    );

    // Dora plans the incident and makes Ben its commander.
    let planned = Planned {
        name: "Sanitätsdienst Stadtfest".into(),
    }
    .encode()
    .unwrap();
    incident_event(&mut world, "dora", incidents::PLANNED, planned);
    grant(&mut world, "dora", "ben", "commander", true);
    world.sync();

    // Ben, as commander, makes Anna a medic; Anna reconnects and may work.
    grant(&mut world, "ben", "anna", "medic", true);
    world.sync();
    register(&mut world, "anna", "DE-0001").expect("medic in this incident");
    world.sync();
    assert_eq!(clinical_events(&mut world, "anna"), 1);
    assert_eq!(
        clinical_events(&mut world, "ben"),
        0,
        "the commander gets no clinical data"
    );

    // A medic may not grant roles.
    grant(&mut world, "anna", "clara", "medic", true);
    world.sync();
    assert!(
        world
            .replica("anna")
            .rejected()
            .iter()
            .any(|r| r.reason == Rejection::Forbidden),
        "a medic's grant is refused"
    );

    // The roster as every device sees it, and the list the server gives.
    let incident = world.incident.clone();
    let events = world
        .replica("ben")
        .events(&incident, incidents::COMPARTMENT)
        .unwrap();
    let registry = Registry::from_events(&events);
    let entry = registry.get(&incident).unwrap();
    assert_eq!(entry.name, "Sanitätsdienst Stadtfest");
    assert_eq!(registry.roles(&user(&world, "anna"), &incident), ["medic"]);
    assert!(registry.roles(&user(&world, "clara"), &incident).is_empty());
    let hello = world.hello("ben", 900);
    let now = world.now;
    world.hub.handle(900, hello, now);
    let out = world.hub.handle(900, ClientMessage::ListIncidents, now);
    assert!(matches!(
        &out[0].message,
        ServerMessage::Incidents(list) if list.len() == 1 && list[0].roles == ["commander"] && list[0].manage
    ));

    // [itest->dsn~person-suggestions~1]
    // The commander may grant roles: the server suggests the people it knows. A medic,
    // who may not, gets an empty list.
    let out = world.hub.handle(900, ClientMessage::ListPeople, now);
    let ServerMessage::People(people) = &out[0].message else {
        panic!("people expected");
    };
    for name in ["anna", "ben", "dora"] {
        assert!(people.contains(&user(&world, name)), "{name} is suggested");
    }
    assert!(people.is_sorted());
    let hello = world.hello("anna", 901);
    world.hub.handle(901, hello, now);
    let out = world.hub.handle(901, ClientMessage::ListPeople, now);
    assert!(matches!(&out[0].message, ServerMessage::People(p) if p.is_empty()));

    // [itest->dsn~unit-register~2]
    // Vehicles of the master data: for planners (and those who edit units), protocol 1.1.
    let out = world.hub.handle(901, ClientMessage::ListVehicles, now);
    assert!(
        matches!(&out[0].message, ServerMessage::Vehicles(v) if v.is_empty()),
        "not for a medic"
    );
    let hello = world.hello("dora", 902);
    world.hub.handle(902, hello, now);
    let out = world.hub.handle(902, ClientMessage::ListVehicles, now);
    assert!(
        matches!(&out[0].message, ServerMessage::Vehicles(v) if v[0].unit_type == "RTW"),
        "for a planner"
    );
    let mut old = world.hello("dora", 903);
    if let ClientMessage::Hello(h) = &mut old {
        h.version = Some(ordo_sync::version::Version::new(1, 0));
    }
    world.hub.handle(903, old, now);
    assert!(
        world
            .hub
            .handle(903, ClientMessage::ListVehicles, now)
            .is_empty(),
        "a 1.0 session gets nothing newer"
    );

    // Ben withdraws Anna's role: her device loses the clinical data.
    grant(&mut world, "ben", "anna", "medic", false);
    world.sync();
    assert_eq!(
        clinical_events(&mut world, "anna"),
        0,
        "data of a withdrawn role is deleted from the device"
    );
    assert!(register(&mut world, "anna", "DE-0002").is_err());
}

/// Only planners plan incidents, and an incident is planned once.
#[test]
fn only_planners_plan_incidents() {
    let server = "ordo.drk-musterstadt.de".parse().unwrap();
    let dora = UserId::new("dora", &server).unwrap();
    let ben = UserId::new("ben", &server).unwrap();
    // Ben may write the shared compartment everywhere, but is no planner.
    let policy = StaticPolicy::default()
        .planner(&dora)
        .allow(&ben, &["incident"]);
    let mut world = World::with_policy(policy);
    world.add_device("dora", 9);
    world.sync();
    let planned = |name: &str| Planned { name: name.into() }.encode().unwrap();
    incident_event(&mut world, "ben", incidents::PLANNED, planned("Ben's"));
    world.sync();
    assert!(
        world
            .replica("ben")
            .rejected()
            .iter()
            .any(|r| r.reason == Rejection::Forbidden)
    );
    incident_event(&mut world, "dora", incidents::PLANNED, planned("Stadtfest"));
    world.sync();
    incident_event(&mut world, "dora", incidents::PLANNED, planned("Again"));
    world.sync();
    let incident = world.incident.clone();
    let events = world
        .replica("dora")
        .events(&incident, incidents::COMPARTMENT)
        .unwrap();
    assert_eq!(
        Registry::from_events(&events).get(&incident).unwrap().name,
        "Stadtfest"
    );
    assert_eq!(
        events.len(),
        1,
        "a second plan of the same incident is refused"
    );
}

// [itest->dsn~incident-devices~1]
/// The incident command sees the devices of the people with a role in its incident, and
/// whether they are connected, and blocks a lost one; it sees no other devices, and a
/// medic sees none and blocks none.
#[test]
fn the_incident_command_blocks_a_device_used_in_its_incident() {
    use ordo_sync::message::{DeviceBlock, IncidentDevices};
    let server = "ordo.drk-musterstadt.de".parse().unwrap();
    let dora = UserId::new("dora", &server).unwrap();
    let policy = StaticPolicy::default()
        .planner(&dora)
        .role("medic", &["incident", "med.clinical"])
        .role("commander", &["incident"])
        .assigner("commander")
        .permit("commander", ordo_sync::BLOCK_DEVICES);
    let mut world = World::with_policy(policy);
    world.add_device("dora", 9);
    world.sync();
    let planned = Planned {
        name: "Sanitätsdienst Stadtfest".into(),
    }
    .encode()
    .unwrap();
    incident_event(&mut world, "dora", incidents::PLANNED, planned);
    grant(&mut world, "dora", "ben", "commander", true);
    world.sync();
    grant(&mut world, "ben", "anna", "medic", true);
    world.sync();
    let incident = world.incident.clone();
    let (anna, ben, clara) = (
        world.replica("anna").device(),
        world.replica("ben").device(),
        world.replica("clara").device(),
    );
    let list = |out: &[ordo_sync::Outgoing], session| -> Vec<(ordo_core::DeviceId, bool)> {
        out.iter()
            .find_map(|o| match &o.message {
                ServerMessage::IncidentDevices(IncidentDevices { devices, .. })
                    if o.session == session =>
                {
                    Some(devices.iter().map(|d| (d.device, d.connected)).collect())
                }
                _ => None,
            })
            .unwrap_or_else(|| panic!("no device list for {session}: {out:?}"))
    };
    let block = |device| {
        ClientMessage::BlockDevice(DeviceBlock {
            incident: incident.clone(),
            device,
        })
    };
    let now = world.now;

    // Ben sees the devices of the incident's people (his and Anna's), not Clara's or the
    // planner's; his own is connected.
    let hello = world.hello("ben", 910);
    world.hub.handle(910, hello, now);
    let out = world.hub.handle(
        910,
        ClientMessage::ListIncidentDevices(incident.clone()),
        now,
    );
    let seen = list(&out, 910);
    let mut ids: Vec<_> = seen.iter().map(|(d, _)| *d).collect();
    ids.sort();
    let mut expected = vec![anna, ben];
    expected.sort();
    assert_eq!(ids, expected);
    assert!(seen.contains(&(ben, true)));

    // A medic gets an empty list and may not block.
    let hello = world.hello("anna", 911);
    world.hub.handle(911, hello, now);
    let out = world.hub.handle(
        911,
        ClientMessage::ListIncidentDevices(incident.clone()),
        now,
    );
    assert!(list(&out, 911).is_empty());
    let out = world.hub.handle(911, block(ben), now);
    assert_eq!(out[0].message, ServerMessage::Refused(Rejection::Forbidden));

    // A device outside the incident cannot be blocked from it.
    let hello = world.hello("ben", 912);
    world.hub.handle(912, hello, now);
    let out = world.hub.handle(912, block(clara), now);
    assert_eq!(out[0].message, ServerMessage::Refused(Rejection::Forbidden));

    // Ben blocks Anna's device: its connections are cut, it is revoked for good, and Ben
    // gets the new list.
    let hello = world.hello("anna", 913);
    world.hub.handle(913, hello, now);
    let hello = world.hello("ben", 914);
    world.hub.handle(914, hello, now);
    let out = world.hub.handle(914, block(anna), now);
    assert!(
        out.iter()
            .any(|o| o.session == 913 && o.message == ServerMessage::Refused(Rejection::Revoked)),
        "{out:?}"
    );
    assert_eq!(
        list(&out, 914).iter().map(|(d, _)| *d).collect::<Vec<_>>(),
        [ben]
    );
    assert!(
        world
            .hub
            .take_changes()
            .iter()
            .any(|c| matches!(c, ordo_sync::Change::Revoked { device } if *device == anna)),
        "the revocation is stored"
    );
    let hello = world.hello("anna", 915);
    let out = world.hub.handle(915, hello, now);
    assert_eq!(out[0].message, ServerMessage::Refused(Rejection::Revoked));
}
