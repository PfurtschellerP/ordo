//! Reports (REP-1…5, ADR-0075) across devices: the command records a patient case and
//! assigns it to a unit; the unit's medic registers the patient from it; the command learns
//! that the case has a patient, and nothing clinical.

use ordo_acceptance::World;
use ordo_contracts::TriageCategory;
use ordo_core::codec;
use ordo_med_patients::{self as patients, Registered};
use ordo_reports::{self as reports, PatientRegistered, Recorded, ReportKind, ReportList};
use ordo_tasks::{self as tasks, TaskGiven, TaskList};

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

// [itest->dsn~reports~1]
#[test]
fn a_patient_case_becomes_a_patient_without_telling_the_command_who() {
    let mut world = World::new();
    world.sync();

    // Ben (command) records the case and assigns it to the medical team.
    let case = Recorded {
        report: "01JD0000000000000000000001".into(),
        kind: ReportKind::Patient,
        place: "Bühne Nord, links".into(),
        text: "Person liegt am Boden".into(),
        received_at: world.now,
    };
    case.check().unwrap();
    let payload = case.encode().unwrap();
    record(
        &mut world,
        "ben",
        reports::COMPARTMENT,
        reports::MODULE,
        reports::RECORDED,
        1,
        &payload,
    );
    let task = TaskGiven {
        task: "01JD0000000000000000000002".into(),
        unit: Some("SanTrp 1".into()),
        sector: None,
        text: "Patient: Bühne Nord, links – Person liegt am Boden".into(),
        template: None,
        follow_up: None,
        report: Some(case.report.clone()),
    };
    let payload = task.encode().unwrap();
    record(
        &mut world,
        "ben",
        tasks::COMPARTMENT,
        tasks::MODULE,
        tasks::GIVEN,
        tasks::GIVEN_SCHEMA,
        &payload,
    );
    world.sync();

    // Anna (medic) sees the case and its task, and registers the patient from it.
    let shared = events(&mut world, "anna", "incident");
    assert_eq!(ReportList::from_events(&shared).open().len(), 1);
    assert_eq!(
        TaskList::from_events(&shared).of_report(&case.report).len(),
        1
    );
    let patient = Registered {
        tag: "DE-0042".into(),
        category: TriageCategory::Urgent,
        uhs: "uhs-1".into(),
        algorithm: None,
        report: Some(case.report.clone()),
    };
    let payload = patient.encode().unwrap();
    record(
        &mut world,
        "anna",
        patients::COMPARTMENT,
        patients::MODULE,
        patients::REGISTERED,
        patients::REGISTERED_SCHEMA,
        &payload,
    );
    let taken = PatientRegistered {
        report: case.report.clone(),
        unit: "SanTrp 1".into(),
    };
    let payload = taken.encode().unwrap();
    record(
        &mut world,
        "anna",
        reports::COMPARTMENT,
        reports::MODULE,
        reports::PATIENT_REGISTERED,
        1,
        &payload,
    );
    world.sync();

    // Ben learns that the case has a patient, from which unit, and nothing else.
    let shared = events(&mut world, "ben", "incident");
    let list = ReportList::from_events(&shared);
    let report = &list.reports[&case.report];
    assert_eq!(report.patients.len(), 1);
    assert_eq!(report.patients[0].unit, "SanTrp 1");
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
    assert_eq!(registered.report.as_deref(), Some(case.report.as_str()));
    assert_eq!(clinical[0].body.schema_version, patients::REGISTERED_SCHEMA);

    // Ben closes the case: it stays, no longer open.
    let closed = reports::Closed {
        report: case.report.clone(),
    }
    .encode()
    .unwrap();
    record(
        &mut world,
        "ben",
        reports::COMPARTMENT,
        reports::MODULE,
        reports::CLOSED,
        1,
        &closed,
    );
    world.sync();
    let list = ReportList::from_events(&events(&mut world, "anna", "incident"));
    assert!(list.open().is_empty());
    assert!(!list.reports[&case.report].is_open());
}
