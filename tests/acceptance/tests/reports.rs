//! Reports as Aufträge (REP-1…5, ADR-0081) across devices: the command records a patient
//! Auftrag and assigns its unit; the unit's medic registers the patient from it; the command
//! learns that it has a patient, and nothing clinical; the crew's "Erledigt" ends it.

use ordo_acceptance::World;
use ordo_contracts::TriageCategory;
use ordo_core::codec;
use ordo_med_patients::{self as patients, Registered};
use ordo_reports::{self as reports, Recorded, ReportKind, ReportList};
use ordo_tasks::{
    self as tasks, PatientRegistered, Phase, TaskAssigned, TaskGiven, TaskKind, TaskList,
    TaskStatus, TaskUpdated,
};

fn record(
    world: &mut World,
    name: &str,
    compartment: &str,
    module: &str,
    event_type: &str,
    schema: u32,
    payload: &[u8],
) {
    let (incident, now) = (world.incident.clone(), world.now);
    world
        .replica(name)
        .record(
            &incident,
            compartment,
            module,
            event_type,
            schema,
            payload,
            now,
        )
        .unwrap();
}

fn events(world: &mut World, name: &str, compartment: &str) -> Vec<ordo_sync::AppliedEvent> {
    let incident = world.incident.clone();
    world
        .replica(name)
        .events(&incident, compartment)
        .unwrap_or_default()
}

// [itest->dsn~reports~3]
#[test]
fn a_patient_auftrag_becomes_a_patient_without_telling_the_command_who() {
    let mut world = World::new();
    world.sync();

    // Ben (command) records the report as an Auftrag, then assigns its one unit.
    let task = TaskGiven {
        task: "01JD0000000000000000000002".into(),
        kind: Some(TaskKind::Patient),
        place: Some("Bühne Nord, links".into()),
        text: "Person liegt am Boden".into(),
        received_at: Some(world.now),
        ..TaskGiven::default()
    };
    task.check().unwrap();
    let id = task.task.clone();
    record(
        &mut world,
        "ben",
        tasks::COMPARTMENT,
        tasks::MODULE,
        tasks::GIVEN,
        tasks::GIVEN_SCHEMA,
        &task.encode().unwrap(),
    );
    world.sync();
    let shared = events(&mut world, "anna", "incident");
    assert_eq!(
        TaskList::from_events(&shared).tasks[&id].phase(),
        Phase::Received
    );
    let assigned = TaskAssigned {
        task: id.clone(),
        unit: "SanTrp 1".into(),
    };
    record(
        &mut world,
        "ben",
        tasks::COMPARTMENT,
        tasks::MODULE,
        tasks::ASSIGNED,
        tasks::ASSIGNED_SCHEMA,
        &assigned.encode().unwrap(),
    );
    world.sync();

    // Anna (medic) sees the Auftrag of her unit and registers the patient from it.
    let shared = events(&mut world, "anna", "incident");
    let list = TaskList::from_events(&shared);
    assert_eq!(list.of_unit("SanTrp 1").len(), 1);
    assert_eq!(list.tasks[&id].phase(), Phase::Assigned);
    let patient = Registered {
        tag: "DE-0042".into(),
        category: TriageCategory::Urgent,
        uhs: "uhs-1".into(),
        algorithm: None,
        report: None,
        task: Some(id.clone()),
    };
    record(
        &mut world,
        "anna",
        patients::COMPARTMENT,
        patients::MODULE,
        patients::REGISTERED,
        patients::REGISTERED_SCHEMA,
        &patient.encode().unwrap(),
    );
    let taken = PatientRegistered {
        task: id.clone(),
        unit: "SanTrp 1".into(),
    };
    record(
        &mut world,
        "anna",
        tasks::COMPARTMENT,
        tasks::MODULE,
        tasks::PATIENT_REGISTERED,
        1,
        &taken.encode().unwrap(),
    );
    world.sync();

    // Ben learns that the Auftrag has a patient, from which unit, and nothing else.
    let shared = events(&mut world, "ben", "incident");
    let list = TaskList::from_events(&shared);
    assert_eq!(list.tasks[&id].patients.len(), 1);
    assert_eq!(list.tasks[&id].patients[0].unit, "SanTrp 1");
    assert!(
        events(&mut world, "ben", patients::COMPARTMENT).is_empty(),
        "no clinical data"
    );
    for e in &shared {
        assert!(
            !e.payload.windows(7).any(|w| w == b"DE-0042"),
            "the tag number stays in med.clinical"
        );
    }
    // The patient record keeps where the patient came from.
    let clinical = events(&mut world, "clara", patients::COMPARTMENT);
    let registered: Registered = codec::from_cbor(&clinical[0].payload).unwrap();
    assert_eq!(registered.task.as_deref(), Some(id.as_str()));
    assert_eq!(clinical[0].body.schema_version, patients::REGISTERED_SCHEMA);

    // Anna's crew is done, the Auftrag's only unit: it ends once, and its pin is no longer
    // needed (ADR-0085).
    let done = TaskUpdated {
        task: id.clone(),
        status: Some(TaskStatus::Done),
        follow_up: None,
        unit: Some("SanTrp 1".into()),
    };
    record(
        &mut world,
        "anna",
        tasks::COMPARTMENT,
        tasks::MODULE,
        tasks::UPDATED,
        tasks::UPDATED_SCHEMA,
        &done.encode().unwrap(),
    );
    world.sync();
    let list = TaskList::from_events(&events(&mut world, "ben", "incident"));
    assert!(list.open(world.now).is_empty());
    assert!(!list.pin_needed(&id));
    assert_eq!(list.tasks[&id].phase(), Phase::Done);
}

// [itest->dsn~reports~3]
#[test]
fn reports_of_an_older_incident_are_still_read() {
    let mut world = World::new();
    world.sync();
    let case = Recorded {
        report: "01JD0000000000000000000001".into(),
        kind: ReportKind::Patient,
        place: "Bühne Nord, links".into(),
        text: "Person liegt am Boden".into(),
        received_at: world.now,
    };
    record(
        &mut world,
        "ben",
        reports::COMPARTMENT,
        reports::MODULE,
        reports::RECORDED,
        1,
        &case.encode().unwrap(),
    );
    world.sync();
    let list = ReportList::from_events(&events(&mut world, "anna", "incident"));
    assert_eq!(list.open().len(), 1);
}
