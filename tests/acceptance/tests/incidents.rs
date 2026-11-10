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
