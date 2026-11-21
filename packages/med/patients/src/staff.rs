//! Only a patient's holder writes them (UHS-10, ADR-0056, ADR-0082, ADR-0086, ADR-0087):
//! the server's write guard for the clinical compartment and the UHS's own settings. The
//! holder is a UHS, written by its staff list, or a unit, written by its crew (UNIT-5). A
//! UHS without anyone on its staff list is written only by roles allowed at every UHS; its
//! settings and staff list only by the incident command. Each event is judged by the lists
//! valid at its clock, so documentation made offline stays valid when a list changes in
//! between. A handover is taken by the people of the holder that takes it. A staff request
//! (UHS-15) is made only by someone on the UHS's staff list.

use std::collections::{BTreeMap, BTreeSet};

use ordo_core::codec;
use ordo_core::{CompartmentId, Hlc};
use ordo_sync::{AppliedEvent, WriteGuard};
use serde::Deserialize;

use crate::{
    COMPARTMENT, DISCHARGED, HANDED_OVER, HandedOver, Holder, MOVED, READMITTED, REGISTERED, TAKEN,
    Taken, UHS_CLOSED, UHS_COMPARTMENT, UHS_CREATED, UHS_PLACES, UHS_UPDATED, UhsClosed,
    UhsCreated, UhsUpdated,
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
    #[serde(default)]
    unit: Option<String>,
}

impl Refers {
    /// The holder a registration names.
    fn holder(&self) -> Option<Holder> {
        match (&self.unit, &self.uhs) {
            (Some(unit), _) => Some(Holder::Unit(unit.clone())),
            (None, Some(uhs)) => Some(Holder::Uhs(uhs.clone())),
            (None, None) => None,
        }
    }
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

/// One incident's staff lists, crews and who holds its patients, over time.
#[derive(Default)]
struct Incident {
    staff: BTreeMap<String, History<Vec<String>>>,
    /// The crew of each unit (accounts and names), over time (UNIT-5).
    crews: BTreeMap<String, History<Vec<String>>>,
    /// The UHS set up so far.
    created: BTreeSet<String>,
    patients: BTreeMap<String, History<Holder>>,
    /// Each handover within ORDO: its patient and the holder it names, if any.
    handovers: BTreeMap<String, (String, Option<Holder>)>,
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

    /// Whether `user` may write the patients of `holder` at `when`: on the UHS's staff list
    /// then (ADR-0086: an empty list names nobody, also not the UHS's creator), or on the
    /// unit's crew (ADR-0087). A unit whose crew is not known yet cannot be judged.
    fn may(&self, holder: &Holder, user: &str, when: Hlc) -> bool {
        match holder {
            Holder::Uhs(uhs) => !self.knows(uhs) || self.on_list(uhs, user, when),
            Holder::Unit(unit) => self.crews.get(unit).is_none_or(|h| {
                at(h, when)
                    .is_some_and(|crew| crew.iter().any(|m| ordo_units::same_person(m, user)))
            }),
        }
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
            at(h, when).is_some_and(|u| u.uhs() == Some(uhs))
                && !self
                    .discharged
                    .get(tag)
                    .and_then(|d| at(d, when))
                    .copied()
                    .unwrap_or(false)
        })
    }

    /// The holder of the patient `tag` at `when`, if known.
    fn holder_at(&self, tag: &str, when: Hlc) -> Option<&Holder> {
        self.patients.get(tag).and_then(|h| at(h, when))
    }

    /// The holder a clinical event is about, if known: the one registered with, the one a
    /// patient leaves, or the patient's holder at the event's time.
    fn holder_of(&self, e: &AppliedEvent) -> Option<Holder> {
        let refers: Refers = codec::from_cbor(&e.payload).ok()?;
        if e.body.event_type.as_str() == REGISTERED {
            return refers.holder();
        }
        let tag = refers
            .tag
            .or_else(|| refers.order.and_then(|o| self.orders.get(&o).cloned()))?;
        self.holder_at(&tag, e.body.hlc).cloned()
    }

    // [impl->dsn~patient-handover~1]
    /// Whether `user` may take a handover with `taken` at `when`: they write the holder
    /// that takes the patient, and a handover that names a holder is taken by it, or back
    /// by the patient's holder.
    fn may_take(&self, taken: &Taken, user: &str, when: Hlc) -> bool {
        if !self.may(&taken.holder, user, when) {
            return false;
        }
        match self.handovers.get(&taken.handover) {
            Some((tag, Some(to))) if *tag == taken.tag => {
                *to == taken.holder || self.holder_at(tag, when) == Some(&taken.holder)
            }
            // A quick handover: any holder its taker writes. One not known yet (made
            // offline) cannot be judged.
            _ => true,
        }
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
        // The units' crews live in the incident compartment too.
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
        let into = match e.body.event_type.as_str() {
            REGISTERED | MOVED => codec::from_cbor::<Refers>(&e.payload)
                .ok()
                .and_then(|r| r.uhs),
            TAKEN => codec::from_cbor::<Taken>(&e.payload)
                .ok()
                .and_then(|t| t.holder.uhs().map(str::to_owned)),
            _ => None,
        };
        if let (Some(incident), Some(uhs)) = (self.incident(e), into)
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
        let user = e.body.author.user.as_ref();
        if e.body.event_type.as_str() == TAKEN {
            return codec::from_cbor::<Taken>(&e.payload)
                .is_ok_and(|t| incident.may_take(&t, user, e.body.hlc));
        }
        // A patient not known yet (documented offline) cannot be judged.
        incident
            .holder_of(e)
            .is_none_or(|holder| incident.may(&holder, user, e.body.hlc))
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
                if let Ok(refers) = codec::from_cbor::<Refers>(&e.payload)
                    && let (Some(tag), Some(holder)) = (refers.tag.clone(), refers.holder())
                {
                    insert(incident.patients.entry(tag).or_default(), when, holder);
                }
            }
            TAKEN => {
                if let Ok(Taken { tag, holder, .. }) = codec::from_cbor(&e.payload) {
                    insert(incident.patients.entry(tag).or_default(), when, holder);
                }
            }
            // [impl->dsn~patient-handover~1]
            // Handed over outside ORDO, the patient has left, as at a discharge.
            HANDED_OVER => {
                if let Ok(h) = codec::from_cbor::<HandedOver>(&e.payload) {
                    if h.outside.is_some() {
                        insert(incident.discharged.entry(h.tag).or_default(), when, true);
                    } else {
                        incident.handovers.insert(h.handover, (h.tag, h.to));
                    }
                }
            }
            ordo_units::CREW_SET if e.body.module.as_str() == ordo_units::MODULE => {
                if let Ok(crew) = codec::from_cbor::<ordo_units::CrewSet>(&e.payload) {
                    let people = crew
                        .members
                        .into_iter()
                        .map(|m| m.account.unwrap_or(m.name))
                        .collect();
                    insert(incident.crews.entry(crew.unit).or_default(), when, people);
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
            unit: None,
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

    fn crew(n: u64, unit: &str, people: &[&str]) -> AppliedEvent {
        let c = ordo_units::CrewSet {
            unit: unit.into(),
            members: people
                .iter()
                .map(|p| ordo_units::CrewMember {
                    name: (*p).into(),
                    qualifications: Vec::new(),
                    helper: None,
                    account: Some(format!("{p}@ordo.example.org")),
                })
                .collect(),
        };
        event(
            n,
            "dora",
            ordo_units::MODULE,
            ordo_units::COMPARTMENT,
            ordo_units::CREW_SET,
            c.encode().unwrap(),
        )
    }

    fn with_crew(n: u64, by: &str, tag: &str, unit: &str) -> AppliedEvent {
        let r = Registered {
            report: None,
            unit: Some(unit.into()),
            task: None,
            tag: tag.into(),
            category: TriageCategory::Minor,
            uhs: String::new(),
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

    fn handed(n: u64, by: &str, id: &str, to: Option<Holder>) -> AppliedEvent {
        let h = HandedOver {
            tag: "DE-1".into(),
            handover: id.into(),
            to,
            outside: None,
            transport: None,
            note: None,
        };
        event(
            n,
            by,
            crate::MODULE,
            COMPARTMENT,
            HANDED_OVER,
            h.encode().unwrap(),
        )
    }

    fn take(n: u64, by: &str, id: &str, holder: Holder) -> AppliedEvent {
        let t = Taken {
            tag: "DE-1".into(),
            handover: id.into(),
            holder,
        };
        event(
            n,
            by,
            crate::MODULE,
            COMPARTMENT,
            TAKEN,
            t.encode().unwrap(),
        )
    }

    // [utest->dsn~patient-holder~1]
    // [utest->dsn~patient-handover~1]
    #[test]
    fn a_crew_writes_its_patient_until_the_uhs_named_takes_it() {
        let (unit, nord) = (Holder::Unit("SanTrp 1".into()), Holder::Uhs("nord".into()));
        let start = [
            created(1, "dora", "nord"),
            staff(2, "dora", "nord", &["anna"]),
            crew(3, "SanTrp 1", &["max"]),
        ];
        let g = guard(&start);
        assert!(
            g.allows(&with_crew(4, "max", "DE-1", "SanTrp 1"), &[]),
            "its crew registers"
        );
        assert!(
            !g.allows(&with_crew(4, "anna", "DE-1", "SanTrp 1"), &[]),
            "nobody else"
        );
        let mut events = start.to_vec();
        events.extend([
            with_crew(4, "max", "DE-1", "SanTrp 1"),
            handed(5, "max", "h1", Some(nord.clone())),
        ]);
        let g = guard(&events);
        assert!(
            g.allows(&retriaged(6, "max", "DE-1"), &[]),
            "the crew writes until taken"
        );
        assert!(
            !g.allows(&retriaged(6, "anna", "DE-1"), &[]),
            "the UHS not yet"
        );
        assert!(
            !g.allows(&take(6, "max", "h1", nord.clone()), &[]),
            "the crew does not take it for the UHS"
        );
        assert!(
            !g.allows(&take(6, "anna", "h1", Holder::Uhs("sued".into())), &[]),
            "nor anyone for another holder"
        );
        assert!(
            g.allows(&take(6, "anna", "h1", nord.clone()), &[]),
            "the UHS's staff takes it"
        );
        assert!(
            g.allows(&take(6, "max", "h1", unit.clone()), &[]),
            "or the crew takes it back"
        );
        events.push(take(6, "anna", "h1", nord.clone()));
        let g = guard(&events);
        assert!(
            g.allows(&retriaged(7, "anna", "DE-1"), &[]),
            "now the UHS writes"
        );
        assert!(
            !g.allows(&retriaged(7, "max", "DE-1"), &[]),
            "and the crew reads only"
        );

        // A quick handover: whoever takes it, for a holder they write.
        let mut events = start.to_vec();
        events.extend([
            with_crew(4, "max", "DE-1", "SanTrp 1"),
            handed(5, "max", "h2", None),
        ]);
        let g = guard(&events);
        assert!(g.allows(&take(6, "anna", "h2", nord.clone()), &[]));
        assert!(
            !g.allows(&take(6, "carl", "h2", nord), &[]),
            "not someone off its staff list"
        );
    }
}
