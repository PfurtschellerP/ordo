//! Module `med.orders`: medical orders (*Anordnungen*) given by physicians and paramedics
//! and carried out by the crew (ORDER-1, ORDER-2, ORDER-4).
//!
//! ORDO records what was ordered, by whom, and what happened to it. It never proposes,
//! checks or calculates a medication, dose or measure (EPCR-9). Who may give an order is
//! decided by the server's policies (ORDER-3).

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

/// Module ID.
pub const MODULE: &str = "med.orders";
/// Orders are clinical.
pub const COMPARTMENT: &str = "med.clinical";
/// Event type: an order was given. Only roles the policies allow may write it.
pub const GIVEN: &str = "orders.given";
/// Event type: an order was acknowledged, done or withdrawn.
pub const UPDATED: &str = "orders.updated";

/// Profile settings (`med/orders.toml`): the routes a medication order can name.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderCatalogue {
    /// Routes of administration, e.g. `iv` "i.v.".
    #[serde(rename = "route", default)]
    pub routes: Vec<crate::transport::Choice>,
}

impl OrderCatalogue {
    /// The label of a route, or the ID if the profile no longer has it.
    pub fn route_label<'a>(&'a self, id: &'a str) -> &'a str {
        crate::transport::TransportCatalogue::label(&self.routes, id)
    }
}

/// What is ordered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Ordered {
    /// A medication, as written by the person giving the order.
    Medication {
        /// Substance, e.g. "Paracetamol" (free text until the medication catalogue).
        substance: String,
        /// Dose with unit, e.g. "1 g" (free text: ORDO never calculates doses).
        dose: String,
        /// Route ID from the profile.
        route: String,
    },
    /// A measure, e.g. "Wunde steril abdecken".
    Measure {
        /// What to do.
        text: String,
    },
}

/// Payload of `orders.given` (ORDER-1).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderGiven {
    /// New order ID (a ULID).
    pub order: String,
    /// Number of the triage tag.
    pub tag: String,
    /// What is ordered.
    pub ordered: Ordered,
    /// Optional note, e.g. "bei Schmerzen > 5".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// An order that is incomplete.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum OrderError {
    /// A required text is empty.
    #[error("{0} is missing")]
    Missing(&'static str),
    /// The route isn't in the profile.
    #[error("unknown route {0:?}")]
    UnknownRoute(String),
}

impl OrderGiven {
    /// Checks a new order for completeness (not for medical sense: that is the person's).
    pub fn check(&self, catalogue: &OrderCatalogue) -> Result<(), OrderError> {
        let filled = |text: &str, what| {
            if text.trim().is_empty() {
                Err(OrderError::Missing(what))
            } else {
                Ok(())
            }
        };
        filled(&self.tag, "tag")?;
        match &self.ordered {
            Ordered::Medication {
                substance,
                dose,
                route,
            } => {
                filled(substance, "substance")?;
                filled(dose, "dose")?;
                if !catalogue.routes.iter().any(|r| &r.id == route) {
                    return Err(OrderError::UnknownRoute(route.clone()));
                }
                Ok(())
            }
            Ordered::Measure { text } => filled(text, "measure"),
        }
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Status of an order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OrderStatus {
    /// Given, nobody has confirmed it yet.
    Given,
    /// A crew member confirmed it and takes care of it.
    Acknowledged,
    /// Carried out (end).
    Done,
    /// Not to be carried out any more (end).
    Withdrawn,
}

impl OrderStatus {
    /// Whether the order still waits for the crew.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Given | Self::Acknowledged)
    }

    /// The statuses that may follow: forward only; withdrawing while not done.
    pub fn next(self) -> Vec<Self> {
        match self {
            Self::Given => vec![Self::Acknowledged, Self::Done, Self::Withdrawn],
            Self::Acknowledged => vec![Self::Done, Self::Withdrawn],
            Self::Done | Self::Withdrawn => Vec::new(),
        }
    }
}

/// Payload of `orders.updated` (ORDER-2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderUpdated {
    /// The order.
    pub order: String,
    /// The new status.
    pub status: OrderStatus,
}

impl OrderUpdated {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// One change of an order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderStep {
    /// The status from then on.
    pub status: OrderStatus,
    /// When.
    pub at: Hlc,
    /// Who.
    pub by: UserId,
}

/// An order with its current status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Order {
    /// The order as given.
    pub given: OrderGiven,
    /// Current status (of the latest update).
    pub status: OrderStatus,
    /// Every change, oldest first; the first is the order itself, with the person who
    /// gave it.
    pub history: Vec<OrderStep>,
}

impl Order {
    /// Who gave the order.
    pub fn by(&self) -> &UserId {
        &self.history[0].by
    }

    /// When it was given.
    pub fn at(&self) -> Hlc {
        self.history[0].at
    }
}

/// All orders of an incident, computed from events of `med.clinical`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OrderBook {
    /// Orders by ID.
    pub orders: BTreeMap<String, Order>,
}

impl OrderBook {
    // [impl->dsn~medical-orders~1]
    /// Computes the book. The result doesn't depend on the order of `events`: an order
    /// given twice keeps its first version, updates apply in clock order and the latest
    /// sets the status, updates of unknown orders are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut given: Vec<(Hlc, UserId, OrderGiven)> = Vec::new();
        let mut updates: Vec<(Hlc, UserId, OrderUpdated)> = Vec::new();
        for e in events.iter().filter(|e| e.body.module.as_str() == MODULE) {
            let (at, by) = (e.body.hlc, e.body.author.user.clone());
            match e.body.event_type.as_str() {
                GIVEN => {
                    if let Ok(g) = codec::from_cbor(&e.payload) {
                        given.push((at, by, g));
                    }
                }
                UPDATED => {
                    if let Ok(u) = codec::from_cbor(&e.payload) {
                        updates.push((at, by, u));
                    }
                }
                _ => {}
            }
        }
        given.sort_by_key(|(at, _, _)| *at);
        updates.sort_by_key(|(at, _, _)| *at);
        let mut orders = BTreeMap::new();
        for (at, by, g) in given {
            orders.entry(g.order.clone()).or_insert_with(|| Order {
                given: g,
                status: OrderStatus::Given,
                history: vec![OrderStep {
                    status: OrderStatus::Given,
                    at,
                    by,
                }],
            });
        }
        for (at, by, u) in updates {
            let Some(order) = orders.get_mut(&u.order) else {
                continue;
            };
            if at <= order.at() {
                continue;
            }
            order.status = u.status;
            order.history.push(OrderStep {
                status: u.status,
                at,
                by,
            });
        }
        Self { orders }
    }

    /// The open orders of all patients, oldest first.
    pub fn open(&self) -> Vec<&Order> {
        let mut open: Vec<&Order> = self
            .orders
            .values()
            .filter(|o| o.status.is_open())
            .collect();
        open.sort_by_key(|o| o.at());
        open
    }

    /// The orders of one patient: open ones first, each group oldest first.
    pub fn of_patient(&self, tag: &str) -> Vec<&Order> {
        let mut list: Vec<&Order> = self
            .orders
            .values()
            .filter(|o| o.given.tag == tag)
            .collect();
        list.sort_by_key(|o| (!o.status.is_open(), o.at()));
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::Choice;
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, IncidentId, ServerName, Ulid};

    fn catalogue() -> OrderCatalogue {
        OrderCatalogue {
            routes: vec![Choice {
                id: "iv".into(),
                label: "i.v.".into(),
                description: None,
            }],
        }
    }

    fn event(n: u64, user: &str, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
        let server: ServerName = "ordo.example.org".parse().expect("valid server");
        let device = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
        AppliedEvent {
            seq: Some(n),
            body: EventBody {
                version: 1,
                id: EventId(Ulid::from_parts(n, u128::from(n)).expect("valid")),
                schema_version: 1,
                hlc: Hlc {
                    wall_ms: 1000 + n,
                    counter: 0,
                    node: device,
                },
                incident: IncidentId::new(Ulid::from_parts(9, 9).expect("valid"), &server),
                module: MODULE.parse().expect("valid"),
                compartment: COMPARTMENT.parse().expect("valid"),
                event_type: event_type.parse().expect("valid"),
                author: Author {
                    user: format!("{user}@ordo.example.org")
                        .parse()
                        .expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload,
        }
    }

    fn medication(order: &str, tag: &str) -> OrderGiven {
        OrderGiven {
            order: order.into(),
            tag: tag.into(),
            ordered: Ordered::Medication {
                substance: "Paracetamol".into(),
                dose: "1 g".into(),
                route: "iv".into(),
            },
            note: None,
        }
    }

    fn given(n: u64, order: &str, tag: &str) -> AppliedEvent {
        event(n, "petra", GIVEN, medication(order, tag).encode().unwrap())
    }

    fn update(n: u64, user: &str, order: &str, status: OrderStatus) -> AppliedEvent {
        let u = OrderUpdated {
            order: order.into(),
            status,
        };
        event(n, user, UPDATED, u.encode().unwrap())
    }

    // [utest->dsn~medical-orders~1]
    #[test]
    fn an_order_is_acknowledged_and_done_with_who_and_when() {
        use OrderStatus::*;
        let mut events = vec![
            given(1, "o1", "DE-1"),
            update(2, "anna", "o1", Acknowledged),
            update(3, "anna", "o1", Done),
        ];
        events.reverse();
        let book = OrderBook::from_events(&events);
        let order = &book.orders["o1"];
        assert_eq!(order.status, Done);
        assert_eq!(
            order.by().as_str(),
            "petra@ordo.example.org",
            "the person who ordered"
        );
        let steps: Vec<_> = order
            .history
            .iter()
            .map(|s| (s.status, s.by.as_str().to_owned()))
            .collect();
        assert_eq!(
            steps,
            vec![
                (Given, "petra@ordo.example.org".to_owned()),
                (Acknowledged, "anna@ordo.example.org".to_owned()),
                (Done, "anna@ordo.example.org".to_owned()),
            ]
        );
    }

    // [utest->dsn~medical-orders~1]
    #[test]
    fn open_orders_of_all_patients_oldest_first_and_per_patient_open_first() {
        let events = vec![
            given(1, "old", "DE-1"),
            given(2, "done", "DE-1"),
            given(3, "new", "DE-2"),
            given(4, "withdrawn", "DE-2"),
            update(5, "anna", "done", OrderStatus::Done),
            update(6, "petra", "withdrawn", OrderStatus::Withdrawn),
            update(7, "anna", "unknown", OrderStatus::Done),
        ];
        let book = OrderBook::from_events(&events);
        let open: Vec<_> = book.open().iter().map(|o| o.given.order.as_str()).collect();
        assert_eq!(open, vec!["old", "new"]);
        let de1: Vec<_> = book
            .of_patient("DE-1")
            .iter()
            .map(|o| o.given.order.as_str())
            .collect();
        assert_eq!(de1, vec!["old", "done"], "open first");
        assert!(!book.orders.contains_key("unknown"));
    }

    // [utest->dsn~medical-orders~1]
    #[test]
    fn steps_go_forward_and_orders_must_be_complete() {
        use OrderStatus::*;
        assert_eq!(Given.next(), vec![Acknowledged, Done, Withdrawn]);
        assert_eq!(Acknowledged.next(), vec![Done, Withdrawn]);
        assert!(Done.next().is_empty() && Withdrawn.next().is_empty());

        let c = catalogue();
        assert_eq!(medication("o", "DE-1").check(&c), Ok(()));
        let mut no_dose = medication("o", "DE-1");
        if let Ordered::Medication { dose, .. } = &mut no_dose.ordered {
            dose.clear();
        }
        assert_eq!(no_dose.check(&c), Err(OrderError::Missing("dose")));
        let mut other_route = medication("o", "DE-1");
        if let Ordered::Medication { route, .. } = &mut other_route.ordered {
            *route = "intrathecal".into();
        }
        assert!(other_route.check(&c).is_err());
        let measure = OrderGiven {
            ordered: Ordered::Measure { text: " ".into() },
            ..medication("o", "DE-1")
        };
        assert_eq!(measure.check(&c), Err(OrderError::Missing("measure")));
    }
}
