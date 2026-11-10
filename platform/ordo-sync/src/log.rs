//! The automatic incident log (LOG-1, LOG-3, LOG-6): a view over the events a device holds,
//! not a store of its own. Events are immutable, so the log can only grow; a device holds
//! only the compartments its user may read, so the log shows nothing else.

use std::collections::BTreeSet;

use crate::AppliedEvent;

// [impl->dsn~incident-log~1]
/// The events of an incident's compartments as one log: newest first by their clock,
/// each event once (also if it was handed over twice), events still in the outbox
/// included.
pub fn newest_first(events: impl IntoIterator<Item = AppliedEvent>) -> Vec<AppliedEvent> {
    let mut seen = BTreeSet::new();
    let mut log: Vec<AppliedEvent> = events
        .into_iter()
        .filter(|e| seen.insert(e.body.id))
        .collect();
    log.sort_by_key(|e| std::cmp::Reverse(e.body.hlc));
    log
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, Hlc, IncidentId, ServerName, Ulid};

    fn event(id: u64, wall_ms: u64, compartment: &str, seq: Option<u64>) -> AppliedEvent {
        let server: ServerName = "ordo.example.org".parse().expect("valid server");
        let device = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
        AppliedEvent {
            seq,
            body: EventBody {
                version: 1,
                id: EventId(Ulid::from_parts(id, u128::from(id)).expect("valid")),
                schema_version: 1,
                hlc: Hlc {
                    wall_ms,
                    counter: 0,
                    node: device,
                },
                incident: IncidentId::new(Ulid::from_parts(9, 9).expect("valid"), &server),
                module: "units".parse().expect("valid"),
                compartment: compartment.parse().expect("valid"),
                event_type: "units.statusChanged".parse().expect("valid"),
                author: Author {
                    user: "ben@ordo.example.org".parse().expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload: Vec::new(),
        }
    }

    // [utest->dsn~incident-log~1]
    #[test]
    fn the_log_is_every_event_once_newest_first() {
        let incident = vec![
            event(1, 1_000, "incident", Some(1)),
            event(3, 3_000, "incident", None),
        ];
        let clinical = vec![
            event(2, 2_000, "med.clinical", Some(1)),
            event(1, 1_000, "incident", Some(1)),
        ];
        let log = newest_first(incident.into_iter().chain(clinical));
        let times: Vec<u64> = log.iter().map(|e| e.body.hlc.wall_ms).collect();
        assert_eq!(
            times,
            [3_000, 2_000, 1_000],
            "each event once, newest first"
        );
        assert_eq!(
            log[0].seq, None,
            "an event still in the outbox is in the log"
        );
        assert_eq!(log[1].body.compartment.as_str(), "med.clinical");
    }
}
