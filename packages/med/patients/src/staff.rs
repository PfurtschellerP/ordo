//! Only a UHS's staff writes its patients (UHS-10, ADR-0056, ADR-0082, ADR-0086): the
//! server's write guard for the clinical compartment and the UHS's own settings. A UHS
//! without anyone on its staff list is written only by roles allowed at every UHS; its
//! settings and staff list only by the incident command. Each event is judged by the staff
//! list valid at its clock, so documentation made offline stays valid when the list changes
//! in between. A staff request (UHS-15) is made only by someone on the UHS's staff list.

use std::collections::{BTreeMap, BTreeSet};

use ordo_core::codec;
use ordo_core::{CompartmentId, Hlc};
use ordo_sync::{AppliedEvent, WriteGuard};
use serde::Deserialize;

use crate::{
    COMPARTMENT, DISCHARGED, MOVED, READMITTED, REGISTERED, UHS_CLOSED, UHS_COMPARTMENT,
    UHS_CREATED, UHS_PLACES, UHS_UPDATED, UhsClosed, UhsCreated, UhsUpdated,
};

/// The permission of roles allowed at every UHS (Cedar action `all_uhs`).
pub const ALL_UHS: &str = "all_uhs";
/// The permission of those who manage the incident's resources; they set up every UHS and
/// assign its staff (UHS-1, UHS-11, ADR-0086), without writing clinical data.
pub const EDIT_UNITS: &str = "edit_units";

/// What the guard needs from any clinical payload: the patient, the order, the UHS.
#[derive(Deserialize)]
struct Refers {
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    order: Option<String>,
    #[serde(default)]
    uhs: Option<String>,
}

/// A value over time, in clock order.
type History<T> = Vec<(Hlc, T)>;

fn insert<T>(history: &mut History<T>, at: Hlc, value: T) {
    let i = history.partition_point(|(t, _)| *t <= at);
    history.insert(i, (at, value));
}

/// The value valid at `at`: the last one set at or before it.
fn at<T>(history: &History<T>, at: Hlc) -> Option<&T> {
    let i = history.partition_point(|(t, _)| *t <= at);
    i.checked_sub(1).map(|i| &history[i].1)
}

/// One incident's staff lists and where its patients are, over time.
#[derive(Default)]
struct Incident {
    staff: BTreeMap<String, History<Vec<String>>>,
    /// The UHS set up so far.
    created: BTreeSet<String>,
    patients: BTreeMap<String, History<String>>,
    /// Whether a patient is discharged, over time.
    discharged: BTreeMap<String, History<bool>>,
    /// When each closed UHS was closed (UHS-17).
    closed: BTreeMap<String, Hlc>,
    orders: BTreeMap<String, String>,
}

impl Incident {
    /// Whether the guard knows `uhs`: created or given staff. One it does not know yet
    /// (recorded offline, or by a client that skipped it) cannot be judged.
    fn knows(&self, uhs: &str) -> bool {
        self.created.contains(uhs) || self.staff.contains_key(uhs)
    }

    /// Whether `user` may write the patients of `uhs` at `when`: on its staff list then
    /// (ADR-0086: an empty list names nobody, also not the UHS's creator).
    fn may(&self, uhs: &str, user: &str, when: Hlc) -> bool {
        !self.knows(uhs) || self.on_list(uhs, user, when)
    }

    /// Whether `user` is on the staff list of `uhs` at `when` (an empty list names nobody).
    fn on_list(&self, uhs: &str, user: &str, when: Hlc) -> bool {
        self.staff
            .get(uhs)
            .and_then(|h| at(h, when))
            .is_some_and(|list| list.iter().any(|s| s.trim().eq_ignore_ascii_case(user)))
    }

    // [impl->dsn~uhs-closing~1]
    /// Whether `uhs` is closed at `when`.
    fn closed_at(&self, uhs: &str, when: Hlc) -> bool {
        self.closed.get(uhs).is_some_and(|c| *c <= when)
    }

    /// Whether a patient not discharged is in `uhs` at `when`.
    fn holds(&self, uhs: &str, when: Hlc) -> bool {
        self.patients.iter().any(|(tag, h)| {
            at(h, when).is_some_and(|u| u == uhs)
                && !self
                    .discharged
                    .get(tag)
                    .and_then(|d| at(d, when))
                    .copied()
                    .unwrap_or(false)
        })
    }

    /// The UHS a clinical event is about, if known: the one registered at, the one a
    /// patient leaves, or the patient's UHS at the event's time.
    fn uhs_of(&self, e: &AppliedEvent) -> Option<String> {
        let refers: Refers = codec::from_cbor(&e.payload).ok()?;
        if e.body.event_type.as_str() == REGISTERED {
            return refers.uhs;
        }
        let tag = refers
            .tag
            .or_else(|| refers.order.and_then(|o| self.orders.get(&o).cloned()))?;
        self.patients
            .get(&tag)
            .and_then(|h| at(h, e.body.hlc))
            .cloned()
    }
}

// [impl->dsn~uhs-staff~3]
/// The Med write guard (ADR-0056).
#[derive(Default)]
pub struct UhsStaffGuard {
    incidents: BTreeMap<String, Incident>,
}

impl UhsStaffGuard {
    fn incident(&self, e: &AppliedEvent) -> Option<&Incident> {
        self.incidents.get(&e.body.incident.to_string())
    }
}

impl WriteGuard for UhsStaffGuard {
    fn observes(&self, compartment: &CompartmentId) -> bool {
        compartment.as_str() == COMPARTMENT || compartment.as_str() == UHS_COMPARTMENT
    }

    fn allows(&self, e: &AppliedEvent, permissions: &[String]) -> bool {
        // [impl->dsn~uhs-staff-request~2]
        // A UHS asks through its staff only, whatever else the person may do.
        if e.body.event_type.as_str() == crate::staffing::REQUESTED {
            let user = e.body.author.user.to_string();
            return codec::from_cbor::<crate::staffing::Requested>(&e.payload).is_ok_and(|r| {
                self.incident(e)
                    .is_some_and(|i| i.on_list(&r.uhs, &user, e.body.hlc))
            });
        }
        // Shared data: a UHS is set up and changed only by those who manage the incident's
        // resources (ADR-0086); the Med policy keeps medical roles off the module too.
        if e.body.compartment.as_str() != COMPARTMENT {
            return match e.body.event_type.as_str() {
                UHS_CREATED | UHS_UPDATED | UHS_PLACES => {
                    permissions.iter().any(|p| p == EDIT_UNITS)
                }
                // [impl->dsn~uhs-closing~1]
                // A UHS is closed only while it holds no patient.
                UHS_CLOSED => {
                    permissions.iter().any(|p| p == EDIT_UNITS)
                        && codec::from_cbor::<UhsClosed>(&e.payload).is_ok_and(|c| {
                            self.incident(e).is_none_or(|i| !i.holds(&c.id, e.body.hlc))
                        })
                }
                _ => true,
            };
        }
        // Nobody brings a patient into a closed UHS.
        if matches!(e.body.event_type.as_str(), REGISTERED | MOVED)
            && let (Some(incident), Ok(Refers { uhs: Some(uhs), .. })) =
                (self.incident(e), codec::from_cbor::<Refers>(&e.payload))
            && incident.closed_at(&uhs, e.body.hlc)
        {
            return false;
        }
        if permissions.iter().any(|p| p == ALL_UHS) {
            return true;
        }
        let Some(incident) = self.incident(e) else {
            return true;
        };
        // A patient not known yet (documented offline) cannot be judged.
        incident
            .uhs_of(e)
            .is_none_or(|uhs| incident.may(&uhs, e.body.author.user.as_ref(), e.body.hlc))
    }

    fn observe(&mut self, e: &AppliedEvent) {
        let incident = self
            .incidents
            .entry(e.body.incident.to_string())
            .or_default();
        let when = e.body.hlc;
        match e.body.event_type.as_str() {
            UHS_CREATED => {
                if let Ok(UhsCreated { id, .. }) = codec::from_cbor(&e.payload) {
                    incident.created.insert(id);
                }
            }
            UHS_UPDATED => {
                if let Ok(UhsUpdated {
                    id,
                    staff: Some(staff),
                    ..
                }) = codec::from_cbor(&e.payload)
                {
                    insert(incident.staff.entry(id).or_default(), when, staff);
                }
            }
            REGISTERED | MOVED => {
                if let Ok(Refers {
                    tag: Some(tag),
                    uhs: Some(uhs),
                    ..
                }) = codec::from_cbor(&e.payload)
                {
                    insert(incident.patients.entry(tag).or_default(), when, uhs);
                }
            }
            UHS_CLOSED => {
                if let Ok(UhsClosed { id }) = codec::from_cbor(&e.payload) {
                    let c = incident.closed.entry(id).or_insert(when);
                    *c = (*c).min(when);
                }
            }
            DISCHARGED | READMITTED => {
                if let Ok(Refers { tag: Some(tag), .. }) = codec::from_cbor(&e.payload) {
                    insert(
                        incident.discharged.entry(tag).or_default(),
                        when,
                        e.body.event_type.as_str() == DISCHARGED,
                    );
                }
            }
            crate::orders::GIVEN | crate::orders::RECORDED => {
                if let Ok(Refers {
                    tag: Some(tag),
                    order: Some(order),
                    ..
                }) = codec::from_cbor(&e.payload)
                {
                    incident.orders.insert(order, tag);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::event;
    use crate::{Moved, PlacesSet, Registered, Retriaged, TriageCategory, UHS_MODULE};

    fn staff(n: u64, by: &str, uhs: &str, people: &[&str]) -> AppliedEvent {
        let update = UhsUpdated {
            id: uhs.into(),
            name: None,
            location: None,
            capacity: None,
            staff: Some(
                people
                    .iter()
                    .map(|p| format!("{p}@ordo.example.org"))
                    .collect(),
            ),
            ..Default::default()
        };
        event(
            n,
            by,
            UHS_MODULE,
            UHS_COMPARTMENT,
            UHS_UPDATED,
            update.encode().unwrap(),
        )
    }

    fn registered(n: u64, by: &str, tag: &str, uhs: &str) -> AppliedEvent {
        let r = Registered {
            report: None,
            task: None,
            tag: tag.into(),
            category: TriageCategory::Minor,
            uhs: uhs.into(),
            algorithm: None,
        };
        event(
            n,
            by,
            crate::MODULE,
            COMPARTMENT,
            REGISTERED,
            r.encode().unwrap(),
        )
    }

    fn retriaged(n: u64, by: &str, tag: &str) -> AppliedEvent {
        let r = Retriaged {
            tag: tag.into(),
            category: TriageCategory::Urgent,
            algorithm: None,
        };
        event(
            n,
            by,
            crate::MODULE,
            COMPARTMENT,
            crate::RETRIAGED,
            r.encode().unwrap(),
        )
    }

    fn moved(n: u64, by: &str, tag: &str, to: &str) -> AppliedEvent {
        let m = Moved {
            tag: tag.into(),
            uhs: to.into(),
        };
        event(
            n,
            by,
            crate::MODULE,
            COMPARTMENT,
            MOVED,
            m.encode().unwrap(),
        )
    }

    fn created(n: u64, by: &str, uhs: &str) -> AppliedEvent {
        let c = UhsCreated {
            id: uhs.into(),
            name: uhs.into(),
            kind: crate::UhsKind::Uhs,
            capacity: 10,
        };
        event(
            n,
            by,
            UHS_MODULE,
            UHS_COMPARTMENT,
            UHS_CREATED,
            c.encode().unwrap(),
        )
    }

    /// Feeds ordered events and returns the guard.
    fn guard(events: &[AppliedEvent]) -> UhsStaffGuard {
        let mut g = UhsStaffGuard::default();
        for e in events {
            g.observe(e);
        }
        g
    }

    // [utest->dsn~uhs-staff~3]
    #[test]
    fn without_a_staff_list_only_roles_allowed_everywhere_write() {
        let g = guard(&[created(1, "dora", "nord")]);
        assert!(
            !g.allows(&registered(2, "anna", "DE-1", "nord"), &[]),
            "nobody on the list: others read only"
        );
        assert!(
            !g.allows(&registered(2, "dora", "DE-1", "nord"), &[]),
            "its creator neither (ADR-0086)"
        );
        assert!(g.allows(&registered(2, "anna", "DE-1", "nord"), &[ALL_UHS.into()]));
        let g = guard(&[
            created(1, "dora", "nord"),
            staff(2, "dora", "nord", &["anna"]),
            registered(3, "anna", "DE-1", "nord"),
            staff(4, "dora", "nord", &[]),
        ]);
        assert!(
            !g.allows(&retriaged(5, "anna", "DE-1"), &[]),
            "taken off the list, she reads only"
        );
    }

    // [utest->dsn~uhs-staff~3]
    #[test]
    fn only_the_command_sets_up_a_uhs_and_changes_it() {
        let places = |n, by: &str| {
            let p = PlacesSet {
                uhs: "nord".into(),
                places: Vec::new(),
            };
            event(
                n,
                by,
                UHS_MODULE,
                UHS_COMPARTMENT,
                UHS_PLACES,
                codec::to_cbor(&p).unwrap(),
            )
        };
        let command = [EDIT_UNITS.to_owned()];
        assert!(g_allows_created(&command), "the command sets a UHS up");
        assert!(!g_allows_created(&[]), "nobody else");
        let g = guard(&[created(1, "dora", "nord")]);
        assert!(g.allows(&staff(2, "dora", "nord", &["anna"]), &command));
        assert!(g.allows(&places(2, "dora"), &command));
        assert!(!g.allows(&places(2, "carl"), &[]), "others read only");
        let g = guard(&[
            created(1, "dora", "nord"),
            staff(2, "dora", "nord", &["anna"]),
        ]);
        assert!(
            !g.allows(&places(3, "anna"), &[]),
            "its staff neither (ADR-0086)"
        );
        assert!(
            !g.allows(&staff(3, "anna", "nord", &["anna", "carl"]), &[]),
            "nor its staff list"
        );
        assert!(
            !g.allows(&places(3, "petra"), &[ALL_UHS.into()]),
            "nor roles allowed at every UHS"
        );
    }

    fn g_allows_created(permissions: &[String]) -> bool {
        guard(&[]).allows(&created(1, "anna", "nord"), permissions)
    }

    // [utest->dsn~uhs-staff~3]
    // [utest->dsn~uhs-closing~1]
    #[test]
    fn a_uhs_is_closed_only_without_patients_and_takes_none_afterwards() {
        let closed = |n, by: &str| {
            event(
                n,
                by,
                UHS_MODULE,
                UHS_COMPARTMENT,
                UHS_CLOSED,
                UhsClosed { id: "nord".into() }.encode().unwrap(),
            )
        };
        let discharged = |n, tag: &str| {
            let d = crate::Discharged {
                tag: tag.into(),
                outcome: crate::Outcome::ReleasedOnSite,
                transport: None,
            };
            event(
                n,
                "anna",
                crate::MODULE,
                COMPARTMENT,
                DISCHARGED,
                codec::to_cbor(&d).unwrap(),
            )
        };
        let command = [EDIT_UNITS.to_owned()];
        let mut g = guard(&[
            created(1, "dora", "nord"),
            staff(2, "dora", "nord", &["anna"]),
            registered(3, "anna", "DE-1", "nord"),
        ]);
        assert!(
            !g.allows(&closed(4, "dora"), &command),
            "a patient is in it"
        );
        assert!(
            g.allows(&closed(2, "dora"), &command),
            "before the patient came"
        );
        g.observe(&discharged(5, "DE-1"));
        assert!(
            g.allows(&closed(6, "dora"), &command),
            "discharged ones do not count"
        );
        assert!(
            !g.allows(&closed(6, "carl"), &[]),
            "only the command closes"
        );
        g.observe(&closed(6, "dora"));
        assert!(
            !g.allows(&registered(7, "anna", "DE-2", "nord"), &[ALL_UHS.into()]),
            "nobody registers into a closed UHS"
        );
        assert!(
            g.allows(&registered(5, "anna", "DE-2", "nord"), &[]),
            "documented before it closed"
        );
    }

    #[test]
    fn with_a_staff_list_only_its_staff_writes_judged_at_the_events_time() {
        let g = guard(&[
            created(1, "dora", "sued"),
            staff(1, "dora", "nord", &["anna"]),
            registered(2, "anna", "DE-1", "nord"),
            staff(5, "dora", "nord", &["anna", "ben"]),
        ]);
        assert!(g.allows(&retriaged(6, "anna", "DE-1"), &[]), "staff");
        assert!(
            g.allows(&retriaged(7, "ben", "DE-1"), &[]),
            "added later, writing after"
        );
        assert!(
            !g.allows(&retriaged(4, "ben", "DE-1"), &[]),
            "written before he was added"
        );
        assert!(
            !g.allows(&registered(8, "carl", "DE-2", "nord"), &[]),
            "not on the list"
        );
        assert!(
            !g.allows(&registered(8, "carl", "DE-2", "sued"), &[]),
            "a UHS without a list"
        );
        assert!(
            g.allows(&registered(8, "carl", "DE-2", "nord"), &[ALL_UHS.into()]),
            "roles allowed everywhere"
        );
        assert!(
            g.allows(&retriaged(8, "carl", "DE-9"), &[]),
            "a patient not known yet (offline)"
        );
        assert!(
            g.allows(&registered(8, "carl", "DE-3", "west"), &[]),
            "a UHS not known yet (offline)"
        );
    }

    // [utest->dsn~uhs-staff~3]
    #[test]
    fn a_move_is_judged_by_the_uhs_left_and_staff_lists_by_their_staff() {
        let g = guard(&[
            created(1, "dora", "sued"),
            staff(1, "dora", "nord", &["anna"]),
            registered(2, "anna", "DE-1", "nord"),
        ]);
        assert!(g.allows(&moved(3, "anna", "DE-1", "sued"), &[]));
        assert!(!g.allows(&moved(3, "carl", "DE-1", "sued"), &[]));
        assert!(
            !g.allows(&staff(3, "carl", "nord", &["carl"]), &[]),
            "no one adds themselves"
        );
        assert!(
            g.allows(
                &staff(3, "ben", "nord", &["anna", "carl"]),
                &[EDIT_UNITS.into()]
            ),
            "the incident command assigns staff"
        );
        // After the move, the new UHS's staff writes.
        let g = guard(&[
            staff(1, "petra", "nord", &["anna"]),
            staff(1, "petra", "sued", &["carl"]),
            registered(2, "anna", "DE-1", "nord"),
            moved(3, "anna", "DE-1", "sued"),
        ]);
        assert!(g.allows(&retriaged(4, "carl", "DE-1"), &[]));
        assert!(!g.allows(&retriaged(4, "anna", "DE-1"), &[]));
    }

    // [utest->dsn~uhs-staff~3]
    #[test]
    fn order_updates_follow_the_orders_patient() {
        use crate::orders::{GIVEN, OrderGiven, OrderStatus, OrderUpdated, Ordered, UPDATED};
        let given = OrderGiven {
            order: "o1".into(),
            tag: "DE-1".into(),
            ordered: Ordered::Measure {
                text: "Lagerung".into(),
            },
            note: None,
        };
        let update = OrderUpdated {
            order: "o1".into(),
            status: OrderStatus::Done,
            performed_by: None,
        };
        let module = crate::orders::MODULE;
        let g = guard(&[
            staff(1, "petra", "nord", &["anna"]),
            registered(2, "anna", "DE-1", "nord"),
            event(
                3,
                "anna",
                module,
                COMPARTMENT,
                GIVEN,
                given.encode().unwrap(),
            ),
        ]);
        let done = |by| {
            event(
                4,
                by,
                module,
                COMPARTMENT,
                UPDATED,
                update.encode().unwrap(),
            )
        };
        assert!(g.allows(&done("anna"), &[]));
        assert!(!g.allows(&done("carl"), &[]));
    }

    // [utest->dsn~uhs-staff-request~2]
    #[test]
    fn only_a_uhss_staff_asks_for_staff() {
        let ask = |n, by: &str, uhs: &str| {
            let r = crate::staffing::Requested {
                request: format!("r{n}"),
                uhs: uhs.into(),
                need: crate::staffing::Need::Physician,
            };
            event(
                n,
                by,
                crate::staffing::MODULE,
                crate::staffing::COMPARTMENT,
                crate::staffing::REQUESTED,
                r.encode().unwrap(),
            )
        };
        let g = guard(&[staff(2, "petra", "nord", &["anna"])]);
        assert!(g.allows(&ask(3, "anna", "nord"), &[]), "on the list");
        assert!(
            !g.allows(&ask(1, "anna", "nord"), &[]),
            "asked before she was on it"
        );
        assert!(!g.allows(&ask(3, "carl", "nord"), &[]), "not on the list");
        assert!(
            !g.allows(
                &ask(3, "carl", "nord"),
                &[ALL_UHS.into(), EDIT_UNITS.into()]
            ),
            "not even the command or roles allowed everywhere"
        );
        assert!(
            !g.allows(&ask(3, "anna", "sued"), &[]),
            "a UHS without a list"
        );
    }
}
