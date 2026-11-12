//! The overview of every on-site care unit for the medical lead (UHS-8): numbers per unit
//! from the clinical and the shared incident events a Med device already holds. It shows
//! numbers only and ranks nothing.

use std::collections::BTreeMap;

use ordo_contracts::PatientCountsV1;
use ordo_sync::AppliedEvent;

use crate::orders::OrderBook;
use crate::transport::TransportBoard;
use crate::{PatientRegister, places};

/// The numbers of one on-site care unit.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UhsSummary {
    /// Patients in the unit per triage category, and how many were discharged.
    pub counts: PatientCountsV1,
    /// Treatment places: free and in all (none if the unit has no places).
    pub places: Option<(usize, usize)>,
    /// Open orders of the unit's patients.
    pub open_orders: usize,
    /// Spoken orders of the unit's patients their giver has not confirmed (ORDER-6).
    pub unconfirmed_orders: usize,
    /// Open transports wanted now.
    pub transports_open: usize,
    /// Open planned transports.
    pub transports_planned: usize,
}

// [impl->dsn~uhs-overview~2]
/// The summary per unit ID, from events of `med.clinical` and of the `incident`
/// compartment. Units without any patient, place, order or transport are left out; the
/// caller lists every unit and shows zeros for them.
pub fn summaries(
    clinical: &[AppliedEvent],
    incident: &[AppliedEvent],
) -> BTreeMap<String, UhsSummary> {
    let register = PatientRegister::from_events(clinical);
    let mut out: BTreeMap<String, UhsSummary> = BTreeMap::new();
    for (uhs, counts) in register.counts() {
        out.entry(uhs).or_default().counts = counts;
    }
    for (uhs, list) in places(incident) {
        let taken = register
            .patients()
            .filter(|p| p.uhs == uhs && p.place.is_some())
            .count();
        out.entry(uhs).or_default().places = Some((list.len().saturating_sub(taken), list.len()));
    }
    let book = OrderBook::from_events(clinical);
    for order in book.open() {
        if let Some(p) = register.patient(&order.given.tag) {
            out.entry(p.uhs.clone()).or_default().open_orders += 1;
        }
    }
    // [impl->dsn~recorded-orders~1]
    for order in book
        .orders
        .values()
        .filter(|o| o.unconfirmed() && o.status != crate::orders::OrderStatus::Withdrawn)
    {
        if let Some(p) = register.patient(&order.given.tag) {
            out.entry(p.uhs.clone()).or_default().unconfirmed_orders += 1;
        }
    }
    for t in TransportBoard::from_events(incident)
        .transports
        .values()
        .filter(|t| t.status.is_open())
    {
        let entry = out.entry(t.request.uhs.clone()).or_default();
        if t.request.planned.is_some() {
            entry.transports_planned += 1;
        } else {
            entry.transports_open += 1;
        }
    }
    for (uhs, s) in &mut out {
        s.counts.uhs.clone_from(uhs);
    }
    out
}

// [impl->dsn~uhs-overview~2]
/// All units added up: the whole incident for the medical lead (MANV). Places count only
/// where units have them.
pub fn total<'a>(summaries: impl IntoIterator<Item = &'a UhsSummary>) -> UhsSummary {
    let mut t = UhsSummary::default();
    for s in summaries {
        t.counts.merge(&s.counts);
        if let Some((free, all)) = s.places {
            let (f, a) = t.places.unwrap_or((0, 0));
            t.places = Some((f + free, a + all));
        }
        t.open_orders += s.open_orders;
        t.unconfirmed_orders += s.unconfirmed_orders;
        t.transports_open += s.transports_open;
        t.transports_planned += s.transports_planned;
    }
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orders::{self, OrderGiven, OrderStatus, OrderUpdated, Ordered};
    use crate::transport::{self, TransportRequested};
    use crate::{
        COMPARTMENT, Discharged, MODULE, Outcome, PLACE_ASSIGNED, Place, PlaceAssigned, PlacesSet,
        REGISTERED, Registered, UHS_COMPARTMENT, UHS_MODULE, UHS_PLACES,
    };
    use ordo_contracts::TriageCategory;

    fn clinical(n: u64, module: &str, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        crate::testing::event(n, "anna", module, COMPARTMENT, event_type, payload)
    }

    fn shared(n: u64, module: &str, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        crate::testing::event(n, "anna", module, UHS_COMPARTMENT, event_type, payload)
    }

    fn patient(n: u64, tag: &str, uhs: &str, category: TriageCategory) -> AppliedEvent {
        let r = Registered {
            tag: tag.into(),
            category,
            uhs: uhs.into(),
            algorithm: None,
        };
        clinical(n, MODULE, REGISTERED, r.encode().unwrap())
    }

    fn order(n: u64, id: &str, tag: &str) -> AppliedEvent {
        let o = OrderGiven {
            order: id.into(),
            tag: tag.into(),
            ordered: Ordered::Measure {
                text: "Lagerung".into(),
            },
            note: None,
        };
        clinical(n, orders::MODULE, orders::GIVEN, o.encode().unwrap())
    }

    fn transport(n: u64, id: &str, uhs: &str, planned: Option<u64>) -> AppliedEvent {
        let r = TransportRequested {
            request: id.into(),
            uhs: uhs.into(),
            transport_type: "rtw".into(),
            priority: "normal".into(),
            needs: Vec::new(),
            planned,
        };
        shared(
            n,
            transport::MODULE,
            transport::REQUESTED,
            r.encode().unwrap(),
        )
    }

    // [utest->dsn~recorded-orders~1]
    #[test]
    fn the_overview_counts_spoken_orders_not_yet_confirmed() {
        let spoken = |n, id: &str| {
            let r = orders::OrderRecorded {
                order: id.into(),
                tag: "A".into(),
                ordered: Ordered::Measure {
                    text: "Lagerung".into(),
                },
                note: None,
                ordered_by: "Dr. M.".into(),
                channel: orders::Channel::Radio,
                read_back: true,
            };
            clinical(n, orders::MODULE, orders::RECORDED, r.encode().unwrap())
        };
        let confirmed = orders::OrderConfirmed { order: "s2".into() };
        let events = vec![
            patient(1, "A", "nord", TriageCategory::Immediate),
            spoken(2, "s1"),
            spoken(3, "s2"),
            clinical(
                4,
                orders::MODULE,
                orders::CONFIRMED,
                confirmed.encode().unwrap(),
            ),
            order(5, "o1", "A"),
        ];
        let s = summaries(&events, &[]);
        assert_eq!(
            (s["nord"].open_orders, s["nord"].unconfirmed_orders),
            (3, 1)
        );
        assert_eq!(total(s.values()).unconfirmed_orders, 1);
    }

    // [utest->dsn~uhs-overview~2]
    #[test]
    fn one_summary_per_unit_with_load_places_orders_and_transports() {
        let places = PlacesSet {
            uhs: "nord".into(),
            places: (1..=3)
                .map(|i| Place {
                    id: format!("p{i}"),
                    label: format!("Platz {i}"),
                })
                .collect(),
        };
        let assigned = PlaceAssigned {
            tag: "A".into(),
            place: Some("p1".into()),
        };
        let discharged = Discharged {
            tag: "C".into(),
            outcome: Outcome::ReleasedOnSite,
            transport: None,
        };
        let done = OrderUpdated {
            order: "o2".into(),
            status: OrderStatus::Done,
            performed_by: None,
        };
        let clinical_events = vec![
            patient(1, "A", "nord", TriageCategory::Immediate),
            patient(2, "B", "nord", TriageCategory::Minor),
            patient(3, "C", "nord", TriageCategory::Minor),
            patient(4, "D", "eingang", TriageCategory::Urgent),
            clinical(5, MODULE, PLACE_ASSIGNED, assigned.encode().unwrap()),
            clinical(6, MODULE, crate::DISCHARGED, discharged.encode().unwrap()),
            order(7, "o1", "A"),
            order(8, "o2", "B"),
            clinical(9, orders::MODULE, orders::UPDATED, done.encode().unwrap()),
            order(10, "o3", "D"),
        ];
        let incident_events = vec![
            shared(11, UHS_MODULE, UHS_PLACES, places.encode().unwrap()),
            transport(12, "t1", "nord", None),
            transport(13, "t2", "nord", Some(99_000)),
            transport(14, "t3", "eingang", None),
        ];
        let s = summaries(&clinical_events, &incident_events);
        let nord = &s["nord"];
        assert_eq!(
            (
                nord.counts.immediate,
                nord.counts.minor,
                nord.counts.discharged
            ),
            (1, 1, 1),
            "patients in the unit by category; the discharged apart"
        );
        assert_eq!(nord.counts.uhs, "nord");
        assert_eq!(nord.places, Some((2, 3)), "one of three places taken");
        assert_eq!(nord.open_orders, 1, "a done order is not open");
        assert_eq!((nord.transports_open, nord.transports_planned), (1, 1));
        let eingang = &s["eingang"];
        assert_eq!(
            (
                eingang.counts.urgent,
                eingang.open_orders,
                eingang.transports_open
            ),
            (1, 1, 1)
        );
        assert_eq!(eingang.places, None);

        let all = total(s.values());
        assert_eq!(
            (
                all.counts.immediate,
                all.counts.urgent,
                all.counts.minor,
                all.counts.discharged
            ),
            (1, 1, 1, 1)
        );
        assert_eq!((all.places, all.open_orders), (Some((2, 3)), 2));
        assert_eq!((all.transports_open, all.transports_planned), (2, 1));
    }
}
