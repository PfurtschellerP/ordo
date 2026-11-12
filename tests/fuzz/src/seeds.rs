//! Writes seed corpora for the fuzz targets from real values (ADR-0059), so fuzzing starts
//! from valid messages, events and payloads.
//!
//!   cargo run --bin seeds -- <corpus directory>

use std::path::Path;

use ordo_core::codec;
use ordo_core::event::{Author, EventBody, Payload, Signature, SignedEvent};
use ordo_core::{DeviceId, EventId, Hlc, IncidentId, KeyId, ServerName, Ulid};
use ordo_med_patients as med;
use ordo_sync::message::{
    ClientMessage, MapRegion, MapRegionCut, MapRegionRequest, MapRegionState, MapTicket,
    Rejection, ServerMessage,
};

fn write(dir: &Path, target: &str, name: &str, bytes: &[u8]) {
    let dir = dir.join(target);
    std::fs::create_dir_all(&dir).expect("corpus directory");
    std::fs::write(dir.join(name), bytes).expect("seed file");
}

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "corpus".into());
    let out = Path::new(&out);
    let server: ServerName = "ordo.example.org".parse().expect("valid");
    let device = DeviceId(Ulid::from_parts(1, 1).expect("valid"));
    let incident = IncidentId::new(Ulid::from_parts(9, 9).expect("valid"), &server);
    let area = MapRegionRequest {
        lat_e7: 483_700_000,
        lon_e7: 109_000_000,
        radius_m: 10_000,
    };

    // Payloads of a few event types.
    let registered = med::Registered {
        tag: "DE-0042".into(),
        category: ordo_contracts::TriageCategory::Immediate,
        uhs: "nord".into(),
        algorithm: Some("prior".into()),
    };
    let payloads: Vec<(&str, Vec<u8>)> = vec![
        ("registered", codec::to_cbor(&registered).expect("encodes")),
        (
            "uhs-updated",
            codec::to_cbor(&med::UhsUpdated {
                id: "nord".into(),
                name: None,
                location: Some("Bühne Nord".into()),
                capacity: Some(10),
                staff: Some(vec!["anna@ordo.example.org".into()]),
            })
            .expect("encodes"),
        ),
        (
            "order-updated",
            codec::to_cbor(&med::orders::OrderUpdated {
                order: "o1".into(),
                status: med::orders::OrderStatus::Done,
                performed_by: Some("Paul".into()),
            })
            .expect("encodes"),
        ),
        (
            "status-changed",
            codec::to_cbor(&ordo_units::StatusChanged {
                unit: "RTW 83/1".into(),
                status: "3".into(),
                replaces: None,
            })
            .expect("encodes"),
        ),
        (
            "drawing",
            codec::to_cbor(&ordo_map::ObjectPlaced {
                object: "drawing:1".into(),
                kind: ordo_map::ObjectKind::Drawing,
                reference: Some("incident-area".into()),
                label: "Einsatzraum".into(),
                geometry: ordo_map::Geometry::Polygon(vec![vec![
                    [10.9, 48.37],
                    [10.91, 48.37],
                    [10.91, 48.38],
                    [10.9, 48.37],
                ]]),
            })
            .expect("encodes"),
        ),
    ];
    for (name, bytes) in &payloads {
        write(out, "payloads", name, bytes);
    }

    // A signed event, and its body alone.
    let body = EventBody {
        version: 1,
        id: EventId(Ulid::from_parts(2, 2).expect("valid")),
        schema_version: 3,
        hlc: Hlc {
            wall_ms: 1_800_000_000_000,
            counter: 0,
            node: device,
        },
        incident: incident.clone(),
        module: med::MODULE.parse().expect("valid"),
        compartment: med::COMPARTMENT.parse().expect("valid"),
        event_type: med::REGISTERED.parse().expect("valid"),
        author: Author {
            user: "anna@ordo.example.org".parse().expect("valid"),
            device,
        },
        origin: server.clone(),
        payload: Payload::Plain(payloads[0].1.clone()),
    };
    let body_bytes = body.encode().expect("encodes");
    let event = SignedEvent {
        body: body_bytes.clone(),
        signature: Signature {
            kid: KeyId::new("device-anna", &server).expect("valid"),
            val: [7; 64],
        },
    };
    write(
        out,
        "signed_event",
        "event",
        &event.encode().expect("encodes"),
    );
    write(out, "signed_event", "body", &body_bytes);

    // Messages in both directions.
    let client = [
        ("list-incidents", ClientMessage::ListIncidents),
        ("list-vehicles", ClientMessage::ListVehicles),
        ("get-map-region", ClientMessage::GetMapRegion(incident.clone())),
        (
            "cut-map-region",
            ClientMessage::CutMapRegion(MapRegionCut {
                incident: incident.clone(),
                area,
            }),
        ),
        ("map-ticket", ClientMessage::MapTicket(incident.clone())),
        ("revoke", ClientMessage::RevokeDevice(device)),
        (
            "list-incident-devices",
            ClientMessage::ListIncidentDevices(incident.clone()),
        ),
        (
            "block-device",
            ClientMessage::BlockDevice(ordo_sync::message::DeviceBlock {
                incident: incident.clone(),
                device,
            }),
        ),
        ("push", ClientMessage::Push(vec![event.clone()])),
    ];
    for (name, m) in &client {
        write(
            out,
            "client_message",
            name,
            &ordo_sync::message::encode(m).expect("encodes"),
        );
    }
    let server_messages = [
        ("refused", ServerMessage::Refused(Rejection::Forbidden)),
        ("incidents", ServerMessage::Incidents(Vec::new())),
        (
            "incident-devices",
            ServerMessage::IncidentDevices(ordo_sync::message::IncidentDevices {
                incident: incident.clone(),
                devices: vec![ordo_sync::message::DeviceInfo {
                    device,
                    user: "anna@ordo.example.org".parse().expect("valid user"),
                    paired: Some(1_800_000_000_000),
                    valid_until: None,
                    name: Some("Firefox auf Android".into()),
                    connected: true,
                }],
            }),
        ),
        (
            "map-region",
            ServerMessage::MapRegion(MapRegion {
                incident: Some(incident.clone()),
                state: MapRegionState::Ready,
                bytes: Some(3_553_183),
                area: Some(area),
                at: Some(1_800_000_000_000),
                ..MapRegion::default()
            }),
        ),
        (
            "map-ticket",
            ServerMessage::MapTicket(MapTicket {
                incident: incident.clone(),
                ticket: Some("0".repeat(64)),
            }),
        ),
    ];
    for (name, m) in &server_messages {
        write(
            out,
            "server_message",
            name,
            &ordo_sync::message::encode(m).expect("encodes"),
        );
    }
    println!("seed corpora written to {}", out.display());
}
