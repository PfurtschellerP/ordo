//! The helper register: its own compartment, read by the incident command only, deleted
//! after its retention period (HELP-1, HELP-4, ADR-0071).

use ordo_acceptance::World;
use ordo_core::UserId;
use ordo_helpers::{Fields, HelperChange};
use ordo_sync::incidents::{self, IncidentState, Planned, RoleChange, StateChanged};
use ordo_sync::{Change, StaticPolicy};

const DAY_MS: u64 = 24 * 3_600_000;

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

fn register_helper(
    world: &mut World,
    name: &str,
    helper: &str,
) -> Result<(), ordo_sync::SyncError> {
    let payload = HelperChange {
        helper: helper.into(),
        fields: Fields {
            name: Some("Lena Berger".into()),
            phone: Some("0170 7654321".into()),
            ..Fields::default()
        },
    }
    .encode()
    .unwrap();
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(name)
        .record(
            &incident,
            ordo_helpers::COMPARTMENT,
            ordo_helpers::MODULE,
            ordo_helpers::REGISTERED,
            ordo_helpers::SCHEMA,
            &payload,
            now,
        )
        .map(|_| ())
}

fn helper_events(world: &mut World, name: &str) -> usize {
    let incident = world.incident.clone();
    world
        .replica(name)
        .events(&incident, ordo_helpers::COMPARTMENT)
        .map(|e| e.len())
        .unwrap_or(0)
}

// [itest->dsn~helper-retention~1]
/// The commander keeps the register; a medic's device never gets it. Once the incident has
/// been closed for the retention period, the server deletes the stream with its key, the
/// commander's device deletes its copy on reconnecting, and nothing is written to it again.
#[test]
fn helpers_data_reaches_only_the_command_and_is_deleted_after_the_incident() {
    let server = "ordo.drk-musterstadt.de".parse().unwrap();
    let dora = UserId::new("dora", &server).unwrap();
    let policy = StaticPolicy::default()
        .planner(&dora)
        .role("commander", &["incident", "people"])
        .role("medic", &["incident", "med.clinical"])
        .assigner("commander");
    let mut world = World::with_policy(policy);
    world.add_device("dora", 9);
    let incident = world.incident.clone();
    for name in ["ben", "anna"] {
        world
            .replica(name)
            .subscribe(&incident, ordo_helpers::COMPARTMENT);
    }
    world.sync();
    incident_event(
        &mut world,
        "dora",
        incidents::PLANNED,
        Planned {
            name: "Stadtfest".into(),
        }
        .encode()
        .unwrap(),
    );
    for (who, role) in [("ben", "commander"), ("anna", "medic")] {
        let payload = RoleChange {
            user: UserId::new(who, &server).unwrap(),
            role: role.into(),
        }
        .encode()
        .unwrap();
        incident_event(&mut world, "dora", incidents::ROLE_GRANTED, payload);
    }
    world.sync();

    register_helper(&mut world, "ben", "h1").expect("the commander registers helpers");
    world.sync();
    assert_eq!(helper_events(&mut world, "ben"), 1);
    assert_eq!(
        helper_events(&mut world, "anna"),
        0,
        "a medic's device never gets them"
    );

    // The incident is closed; the period counts from here.
    let closed = StateChanged {
        state: IncidentState::Closed,
    }
    .encode()
    .unwrap();
    incident_event(&mut world, "dora", incidents::STATE_CHANGED, closed);
    world.sync();
    let closed_at = world.now;
    assert!(
        world.hub.expired(closed_at + 89 * DAY_MS).is_empty(),
        "kept during the retention period"
    );
    let expired = world.hub.expired(closed_at + 90 * DAY_MS);
    assert_eq!(
        expired,
        [(incident.clone(), ordo_helpers::COMPARTMENT.parse().unwrap())]
    );

    // The server deletes the stream and stores that it did.
    let _ = world.hub.take_changes();
    world.hub.delete_stream(&expired[0]);
    assert!(
        world
            .hub
            .take_changes()
            .iter()
            .any(|c| matches!(c, Change::Deleted { stream } if *stream == expired[0]))
    );
    assert!(
        world.hub.expired(closed_at + 91 * DAY_MS).is_empty(),
        "once"
    );

    // The commander's device reconnects and deletes its copy; it cannot write there again.
    world.set_online("ben", false);
    world.set_online("ben", true);
    world.sync();
    assert_eq!(helper_events(&mut world, "ben"), 0, "deleted on the device");
    assert!(
        register_helper(&mut world, "ben", "h2").is_err(),
        "a deleted stream is not written again"
    );
}
