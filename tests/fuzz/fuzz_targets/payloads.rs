//! The payload of every event type, as modules decode them on devices and the server
// (derivers, write guards) (SEC-17).
// [itest->dsn~fuzzing~1]
#![no_main]

use libfuzzer_sys::fuzz_target;
use ordo_fuzz::round_trip;
use ordo_med_patients as med;

fuzz_target!(|data: &[u8]| {
    // Incidents and rosters (checked by the hub itself).
    round_trip::<ordo_sync::incidents::Planned>(data);
    round_trip::<ordo_sync::incidents::StateChanged>(data);
    round_trip::<ordo_sync::incidents::RoleChange>(data);
    for event_type in [
        ordo_sync::incidents::PLANNED,
        ordo_sync::incidents::STATE_CHANGED,
        ordo_sync::incidents::ROLE_GRANTED,
    ] {
        let _ = ordo_sync::incidents::is_valid(event_type, data);
    }
    // Shared modules.
    round_trip::<ordo_units::UnitAdded>(data);
    round_trip::<ordo_units::Released>(data);
    round_trip::<ordo_units::CrewSet>(data);
    round_trip::<ordo_units::StatusChanged>(data);
    round_trip::<ordo_tasks::TaskGiven>(data);
    round_trip::<ordo_tasks::TaskUpdated>(data);
    round_trip::<ordo_command::PositionCreated>(data);
    round_trip::<ordo_command::PositionClosed>(data);
    round_trip::<ordo_command::LeaderSet>(data);
    round_trip::<ordo_command::Assigned>(data);
    round_trip::<ordo_map::ObjectPlaced>(data);
    round_trip::<ordo_map::ObjectRemoved>(data);
    // Med package.
    round_trip::<med::UhsCreated>(data);
    round_trip::<med::UhsUpdated>(data);
    round_trip::<med::PlacesSet>(data);
    round_trip::<med::PlaceAssigned>(data);
    round_trip::<med::Registered>(data);
    round_trip::<med::Retriaged>(data);
    round_trip::<med::Moved>(data);
    round_trip::<med::Discharged>(data);
    round_trip::<med::Readmitted>(data);
    round_trip::<med::TransportPrioritySet>(data);
    round_trip::<med::transport::TransportRequested>(data);
    round_trip::<med::transport::TransportUpdated>(data);
    round_trip::<med::transport::TransportLinked>(data);
    round_trip::<med::orders::OrderGiven>(data);
    round_trip::<med::orders::OrderUpdated>(data);
    round_trip::<med::hospitals::HospitalSet>(data);
    round_trip::<med::hospitals::CapacitySet>(data);
    round_trip::<med::record::EntryRecorded>(data);
    round_trip::<med::record::PhotoAttached>(data);
    // Connectors (derived by the server).
    round_trip::<ordo_contracts::PatientCountsV1>(data);
});
