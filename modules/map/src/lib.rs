//! Shared module `map` (MAP-1, ADR-0006): the objects on the situation map. Shared
//! incident data, no patient data. Geometries follow GeoJSON (RFC 7946): longitude first.

use std::collections::BTreeMap;

use ordo_core::codec::{self, CodecError};
use ordo_core::{Hlc, UserId};
use ordo_sync::AppliedEvent;
use serde::{Deserialize, Serialize};

pub mod drawings;
pub mod symbols;
pub use drawings::{Category, DrawingCatalogue, LineStyle, Shape};
pub use symbols::{Symbol, SymbolCatalogue};

/// Module ID.
pub const MODULE: &str = "map";
/// Compartment: shared with the incident.
pub const COMPARTMENT: &str = "incident";
/// Event type: an object was put on the map, or moved.
pub const PLACED: &str = "map.objectPlaced";
/// Event type: an object was taken off the map.
pub const REMOVED: &str = "map.objectRemoved";
/// Schema version of `map.objectPlaced`: 2 since an object may stand for a report (REP-4,
/// ADR-0075); older payloads decode unchanged.
pub const PLACED_SCHEMA: u32 = 2;
/// The ID of the incident location: there is one per incident.
pub const INCIDENT_OBJECT: &str = "incident";

/// What an object stands for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ObjectKind {
    /// The incident location (*Einsatzort*).
    Incident,
    /// A unit (`reference`: call sign).
    Unit,
    /// An on-site care unit (`reference`: its ID).
    Uhs,
    /// A position of the command structure, e.g. a sector (`reference`: its ID).
    Position,
    /// Something drawn (MAP-4).
    Drawing,
    /// The place of a report (`reference`: its ID; REP-4). Since schema version 2.
    Report,
}

impl ObjectKind {
    /// The ID of the object for `reference`: one object per incident, unit, UHS or
    /// position, so placing it again moves it.
    pub fn object_id(self, reference: &str) -> String {
        match self {
            Self::Incident => INCIDENT_OBJECT.to_owned(),
            Self::Unit => format!("unit:{reference}"),
            Self::Uhs => format!("uhs:{reference}"),
            Self::Position => format!("position:{reference}"),
            Self::Drawing => format!("drawing:{reference}"),
            Self::Report => format!("report:{reference}"),
        }
    }
}

/// A geometry as in GeoJSON: `{"type": "Point", "coordinates": [lon, lat]}`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "coordinates")]
pub enum Geometry {
    /// A point: longitude, latitude.
    Point([f64; 2]),
    /// A line.
    LineString(Vec<[f64; 2]>),
    /// An area: an outer ring, then holes.
    Polygon(Vec<Vec<[f64; 2]>>),
}

impl Geometry {
    /// Whether every coordinate is a valid longitude and latitude.
    pub fn is_valid(&self) -> bool {
        let ok = |[lon, lat]: &[f64; 2]| {
            lon.is_finite()
                && lat.is_finite()
                && (-180.0..=180.0).contains(lon)
                && (-90.0..=90.0).contains(lat)
        };
        match self {
            Self::Point(p) => ok(p),
            Self::LineString(line) => line.len() >= 2 && line.iter().all(ok),
            Self::Polygon(rings) => {
                !rings.is_empty() && rings.iter().all(|r| r.len() >= 4 && r.iter().all(ok))
            }
        }
    }

    /// A point to centre the map on: the point, or the first coordinate.
    pub fn anchor(&self) -> Option<[f64; 2]> {
        match self {
            Self::Point(p) => Some(*p),
            Self::LineString(line) => line.first().copied(),
            Self::Polygon(rings) => rings.first().and_then(|r| r.first()).copied(),
        }
    }
}

/// Payload of `map.objectPlaced`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ObjectPlaced {
    /// Object ID ([`ObjectKind::object_id`]).
    pub object: String,
    /// What it stands for.
    pub kind: ObjectKind,
    /// The unit, UHS or position it stands for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    /// The label shown on the map and in the list.
    pub label: String,
    /// Where.
    pub geometry: Geometry,
}

impl ObjectPlaced {
    /// Checks a placement: a label and valid coordinates.
    pub fn check(&self) -> Result<(), String> {
        if self.label.trim().is_empty() {
            return Err("an object needs a label".into());
        }
        if !self.geometry.is_valid() {
            return Err("coordinates out of range".into());
        }
        Ok(())
    }

    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// Payload of `map.objectRemoved`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectRemoved {
    /// Object ID.
    pub object: String,
}

impl ObjectRemoved {
    /// CBOR payload.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }
}

/// An object on the map.
#[derive(Clone, Debug, PartialEq)]
pub struct MapObject {
    /// As last placed.
    pub placed: ObjectPlaced,
    /// When.
    pub at: Hlc,
    /// Who.
    pub by: UserId,
}

enum Change {
    Placed(ObjectPlaced),
    Removed(ObjectRemoved),
}

/// The objects on the situation map of an incident.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SituationMap {
    /// Objects by ID.
    pub objects: BTreeMap<String, MapObject>,
}

impl SituationMap {
    // [impl->dsn~situation-map~5]
    /// Computes the map. The result doesn't depend on the order of `events`: the latest
    /// event of each object counts, so placing moves it and removing takes it off until it
    /// is placed again. Placements with invalid coordinates are ignored.
    pub fn from_events(events: &[AppliedEvent]) -> Self {
        let mut changes: Vec<(Hlc, UserId, Change)> = events
            .iter()
            .filter(|e| e.body.module.as_str() == MODULE)
            .filter_map(|e| {
                let change = match e.body.event_type.as_str() {
                    PLACED => Change::Placed(codec::from_cbor(&e.payload).ok()?),
                    REMOVED => Change::Removed(codec::from_cbor(&e.payload).ok()?),
                    _ => return None,
                };
                Some((e.body.hlc, e.body.author.user.clone(), change))
            })
            .collect();
        changes.sort_by_key(|(at, _, _)| *at);
        let mut objects = BTreeMap::new();
        for (at, by, change) in changes {
            match change {
                Change::Placed(p) if p.geometry.is_valid() => {
                    objects.insert(p.object.clone(), MapObject { placed: p, at, by });
                }
                Change::Placed(_) => {}
                Change::Removed(r) => {
                    objects.remove(&r.object);
                }
            }
        }
        Self { objects }
    }

    /// Where to open the map: the incident location, else the first object.
    pub fn centre(&self) -> Option<[f64; 2]> {
        self.objects
            .get(INCIDENT_OBJECT)
            .or_else(|| self.objects.values().next())
            .and_then(|o| o.placed.geometry.anchor())
    }

    /// The objects in list order: the incident first, then by kind and label (A11Y-5).
    pub fn list(&self) -> Vec<&MapObject> {
        let mut list: Vec<&MapObject> = self.objects.values().collect();
        list.sort_by(|a, b| {
            (a.placed.kind, &a.placed.label).cmp(&(b.placed.kind, &b.placed.label))
        });
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::event::{Author, EventBody, Payload};
    use ordo_core::{DeviceId, EventId, IncidentId, ServerName, Ulid};

    fn event(n: u64, event_type: &str, payload: Vec<u8>) -> AppliedEvent {
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
                    user: "ben@ordo.example.org".parse().expect("valid user"),
                    device,
                },
                origin: server,
                payload: Payload::Plain(Vec::new()),
            },
            payload,
        }
    }

    fn place(n: u64, kind: ObjectKind, reference: &str, at: [f64; 2]) -> AppliedEvent {
        let p = ObjectPlaced {
            object: kind.object_id(reference),
            kind,
            reference: (kind != ObjectKind::Incident).then(|| reference.to_owned()),
            label: reference.to_owned(),
            geometry: Geometry::Point(at),
        };
        event(n, PLACED, p.encode().unwrap())
    }

    // [utest->dsn~situation-map~5]
    #[test]
    fn the_latest_placement_counts_and_removing_takes_an_object_off() {
        let mut events = vec![
            place(1, ObjectKind::Unit, "RTW 1", [10.9, 48.4]),
            place(2, ObjectKind::Incident, "Stadtfest", [10.89, 48.37]),
            place(3, ObjectKind::Unit, "RTW 1", [10.95, 48.41]), // moved
            place(4, ObjectKind::Uhs, "uhs-1", [10.8, 48.3]),
            event(
                5,
                REMOVED,
                ObjectRemoved {
                    object: "uhs:uhs-1".into(),
                }
                .encode()
                .unwrap(),
            ),
            place(6, ObjectKind::Unit, "Bad", [200.0, 48.0]), // invalid: ignored
        ];
        events.reverse();
        let map = SituationMap::from_events(&events);
        assert_eq!(map.objects.len(), 2);
        assert_eq!(
            map.objects["unit:RTW 1"].placed.geometry,
            Geometry::Point([10.95, 48.41])
        );
        assert_eq!(
            map.centre(),
            Some([10.89, 48.37]),
            "the map opens on the incident"
        );
        let labels: Vec<&str> = map.list().iter().map(|o| o.placed.label.as_str()).collect();
        assert_eq!(
            labels,
            ["Stadtfest", "RTW 1"],
            "the incident first in the list"
        );
    }

    // [utest->dsn~situation-map~5]
    #[test]
    fn geometries_are_geojson_and_checked() {
        let point = Geometry::Point([10.9, 48.4]);
        assert_eq!(
            serde_json::to_string(&point).unwrap(),
            r#"{"type":"Point","coordinates":[10.9,48.4]}"#
        );
        let area = Geometry::Polygon(vec![vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]);
        assert!(area.is_valid());
        assert!(
            !Geometry::LineString(vec![[0.0, 0.0]]).is_valid(),
            "a line needs two points"
        );
        assert!(!Geometry::Point([0.0, 91.0]).is_valid());
        assert!(!Geometry::Point([f64::NAN, 0.0]).is_valid());
    }
}
