//! End-to-end: real server, WebSocket clients, SQLite file, restart.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ordo_contracts::TriageCategory::{self, Immediate, Urgent};
use ordo_core::{DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_med_patients::orders::{self, OrderBook, OrderGiven, OrderStatus, OrderUpdated, Ordered};
use ordo_med_patients::{self as patients, PatientRegister, Registered};
use ordo_server::config::to_hex;
use ordo_server::{Config, Running, ServerError, start};
use ordo_sync::Replica;
use ordo_sync::message::{self, BlobRequest, ClientMessage, Rejection, ServerMessage};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite::Message;

const SERVER: &str = "ordo.drk-musterstadt.de";
const T0: u64 = 1_762_600_000_000;

fn server() -> ServerName {
    SERVER.parse().unwrap()
}

fn incident() -> IncidentId {
    IncidentId::new(Ulid::from_parts(T0, 1).unwrap(), &server())
}

/// (user, key seed, roles)
const PEOPLE: [(&str, u8, &[&str]); 5] = [
    ("anna", 1, &["medic"]),
    ("clara", 2, &["medic"]),
    ("ben", 3, &["commander"]),
    ("petra", 4, &["physician"]),
    ("dora", 5, &["planner"]),
];

fn device_id(seed: u8) -> DeviceId {
    DeviceId(Ulid::from_parts(T0, u128::from(seed)).unwrap())
}

fn write_setup(dir: &Path, master: [u8; 32]) {
    std::fs::write(dir.join("master.key"), to_hex(&master)).unwrap();
    let mut seed = String::new();
    for (name, key, roles) in PEOPLE {
        let list: Vec<String> = roles.iter().map(|r| format!("\"{r}\"")).collect();
        seed += &format!(
            "[[user]]\nid = \"{name}@{SERVER}\"\nroles = [{}]\n\n",
            list.join(", ")
        );
        let public = SigningKey::from_bytes(&[key; 32]).verifying_key();
        seed += &format!(
            "[[device]]\nid = \"{}\"\nuser = \"{name}@{SERVER}\"\npublic_key = \"{}\"\n\n",
            device_id(key).ulid(),
            to_hex(public.as_bytes())
        );
    }
    std::fs::write(dir.join("seed.toml"), seed).unwrap();
    // A stand-in for the incident's map region (PMTiles starts with this magic number).
    std::fs::create_dir_all(dir.join("maps")).unwrap();
    std::fs::write(
        dir.join("maps")
            .join(format!("{}.pmtiles", incident().ulid())),
        b"PMTiles\x03 region of the test",
    )
    .unwrap();
}

// [itest->dsn~environment-configuration~1]
async fn start_server(dir: &Path) -> Result<Running, ServerError> {
    start_server_with(dir, &[]).await
}

/// Starts the server with further settings (`name`, `value`), replacing the defaults.
async fn start_server_with(dir: &Path, extra: &[(&str, String)]) -> Result<Running, ServerError> {
    let mut vars: HashMap<&str, String> = HashMap::from([
        ("ORDO_SERVER_NAME", SERVER.to_owned()),
        ("ORDO_LISTEN", "127.0.0.1:0".to_owned()),
        ("ORDO_DATA_DIR", dir.join("data").display().to_string()),
        (
            "ORDO_MASTER_KEY_FILE",
            dir.join("master.key").display().to_string(),
        ),
        (
            "ORDO_SEED_FILE",
            dir.join("seed.toml").display().to_string(),
        ),
        ("ORDO_MAP_DIR", dir.join("maps").display().to_string()),
    ]);
    vars.extend(extra.iter().cloned());
    start(Config::from_lookup(|name| vars.get(name).cloned())?).await
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn replica(name: &str) -> Replica {
    let (_, seed, _) = PEOPLE.iter().find(|(n, _, _)| *n == name).unwrap();
    let user = UserId::new(name, &server()).unwrap();
    let signer = EventSigner::new(
        SigningKey::from_bytes(&[*seed; 32]),
        KeyId::new(&format!("device-{name}"), &server()).unwrap(),
    );
    let mut replica = Replica::new(user, device_id(*seed), signer, |b: &mut [u8]| {
        getrandom::fill(b).unwrap()
    });
    replica.subscribe(&incident(), "incident");
    replica.subscribe(&incident(), "med.clinical");
    replica
}

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// A device connected over a real WebSocket.
struct Client {
    socket: Socket,
    replica: Replica,
}

impl Client {
    async fn connect(addr: std::net::SocketAddr, replica: Replica) -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{addr}/sync"))
            .await
            .unwrap();
        let mut client = Self { socket, replica };
        // The server opens with a challenge; the hello proves the device key.
        let hello = loop {
            let frame = tokio::time::timeout(Duration::from_secs(10), client.socket.next())
                .await
                .expect("timed out waiting for the challenge")
                .expect("connection closed")
                .unwrap();
            if let Message::Binary(bytes) = frame {
                client
                    .replica
                    .receive(message::decode(&bytes).unwrap(), now())
                    .unwrap();
                if let Some(hello) = client.replica.hello() {
                    break hello;
                }
            }
        };
        client.send(hello).await;
        // Wait for the welcome (not for a fixed time: CI runners can be slow).
        client
            .receive_until(|r| r.is_granted(&incident(), "incident"))
            .await;
        client.settle().await;
        client
    }

    /// Receives until `done` holds; fails after 10 seconds.
    async fn receive_until(&mut self, done: impl Fn(&Replica) -> bool) {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
        while !done(&self.replica) {
            let frame = tokio::time::timeout_at(deadline, self.socket.next())
                .await
                .expect("timed out waiting for the server")
                .expect("connection closed")
                .unwrap();
            if let Message::Binary(bytes) = frame {
                let message: ServerMessage = message::decode(&bytes).unwrap();
                self.replica.receive(message, now()).unwrap();
            }
        }
    }

    async fn send(&mut self, message: ClientMessage) {
        let bytes = message::encode(&message).unwrap();
        self.socket
            .send(Message::Binary(bytes.into()))
            .await
            .unwrap();
    }

    /// Receives until the server has been quiet for a moment.
    async fn settle(&mut self) {
        while let Ok(Some(Ok(frame))) =
            tokio::time::timeout(Duration::from_millis(300), self.socket.next()).await
        {
            if let Message::Binary(bytes) = frame {
                let message: ServerMessage = message::decode(&bytes).unwrap();
                self.replica.receive(message, now()).unwrap();
            }
        }
    }

    /// Sends a map region message and waits for the answer.
    async fn map_region(&mut self, request: ClientMessage) -> message::MapRegion {
        self.send(request).await;
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(10), self.socket.next())
                .await
                .expect("timed out waiting for the map region")
                .expect("connection closed")
                .unwrap();
            if let Message::Binary(bytes) = frame {
                match message::decode::<ServerMessage>(&bytes).unwrap() {
                    ServerMessage::MapRegion(region) => return region,
                    other => self.replica.receive(other, now()).unwrap(),
                }
            }
        }
    }

    /// Asks for a ticket to read the incident's map region and waits for the answer.
    async fn map_ticket(&mut self) -> Option<String> {
        self.send(ClientMessage::MapTicket(incident())).await;
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(10), self.socket.next())
                .await
                .expect("timed out waiting for the map ticket")
                .expect("connection closed")
                .unwrap();
            if let Message::Binary(bytes) = frame {
                match message::decode::<ServerMessage>(&bytes).unwrap() {
                    ServerMessage::MapTicket(t) => {
                        assert_eq!(t.incident, incident());
                        return t.ticket;
                    }
                    other => self.replica.receive(other, now()).unwrap(),
                }
            }
        }
    }

    /// Asks for an attachment ticket and waits for the answer.
    async fn blob_ticket(&mut self, hash: [u8; 32], size: u64, upload: bool) -> Option<String> {
        self.send(ClientMessage::BlobTicket(BlobRequest {
            incident: incident(),
            compartment: "med.clinical".parse().unwrap(),
            hash,
            size,
            upload,
        }))
        .await;
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(10), self.socket.next())
                .await
                .expect("timed out waiting for the ticket")
                .expect("connection closed")
                .unwrap();
            if let Message::Binary(bytes) = frame {
                match message::decode::<ServerMessage>(&bytes).unwrap() {
                    ServerMessage::BlobTicket(t) if t.hash == hash => return t.ticket,
                    other => self.replica.receive(other, now()).unwrap(),
                }
            }
        }
    }

    async fn sync(&mut self) {
        if let Some(push) = self.replica.push() {
            self.send(push).await;
        }
        self.receive_until(|r| r.outbox_len() == 0).await;
        self.settle().await;
    }

    fn register(&mut self, tag: &str, category: TriageCategory) {
        let payload = Registered {
            tag: tag.into(),
            category,
            uhs: "UHS 1".into(),
            algorithm: None,
        }
        .encode()
        .unwrap();
        self.replica
            .record(
                &incident(),
                patients::COMPARTMENT,
                patients::MODULE,
                patients::REGISTERED,
                1,
                &payload,
                now(),
            )
            .unwrap();
    }

    fn patients(&self) -> PatientRegister {
        PatientRegister::from_events(
            &self
                .replica
                .events(&incident(), patients::COMPARTMENT)
                .unwrap(),
        )
    }

    async fn wait_for_counts(&mut self, expected: &[(u32, u32, u32)]) {
        self.receive_until(|r| counts_of(r) == expected).await;
    }

    fn counts(&self) -> Vec<(u32, u32, u32)> {
        counts_of(&self.replica)
    }
}

fn counts_of(replica: &Replica) -> Vec<(u32, u32, u32)> {
    patients::latest_counts(&replica.events(&incident(), "incident").unwrap())
        .values()
        .map(|c| (c.immediate, c.urgent, c.minor))
        .collect()
}

/// A raw HTTP request with an optional ticket and body: (status, body).
async fn http(
    addr: std::net::SocketAddr,
    method: &str,
    path: &str,
    ticket: Option<&str>,
    body: &[u8],
) -> (u16, Vec<u8>) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let auth = ticket.map_or_else(String::new, |t| format!("Authorization: Bearer {t}\r\n"));
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n{auth}Content-Length: {}\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes()).await.unwrap();
    stream.write_all(body).await.unwrap();
    let mut response = Vec::new();
    stream.read_to_end(&mut response).await.unwrap();
    let text = String::from_utf8_lossy(&response);
    let status = text.split_whitespace().nth(1).unwrap().parse().unwrap();
    let start = response
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map_or(response.len(), |i| i + 4);
    (status, response[start..].to_vec())
}

// [itest->dsn~attachments~1]
/// Attachments: a medic uploads an encrypted attachment of the clinical stream with a
/// ticket; another medic downloads it; the commander gets no ticket; a ticket is bound to
/// its hash and direction; content that does not match its hash is refused.
#[tokio::test(flavor = "multi_thread")]
async fn attachments_follow_the_rules_of_their_stream() {
    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [6; 32]);
    let running = start_server(dir.path()).await.unwrap();
    let addr = running.addr;
    let mut anna = Client::connect(addr, replica("anna")).await;
    anna.settle().await;
    let blob = b"ciphertext of a photo".to_vec();
    let hash = *blake3::hash(&blob).as_bytes();
    let path = format!("/blobs/{}", blob_hex(&hash));

    let upload = anna
        .blob_ticket(hash, blob.len() as u64, true)
        .await
        .expect("medic may upload");
    let mut wrong = blob.clone();
    wrong[0] ^= 1;
    assert_eq!(
        http(addr, "PUT", &path, Some(&upload), &wrong).await.0,
        422,
        "content that does not match its hash is refused"
    );
    let upload = anna
        .blob_ticket(hash, blob.len() as u64, true)
        .await
        .unwrap();
    assert_eq!(http(addr, "PUT", &path, Some(&upload), &blob).await.0, 201);
    assert_eq!(
        http(addr, "PUT", &path, Some(&upload), &blob).await.0,
        403,
        "an upload ticket is used once"
    );

    let mut clara = Client::connect(addr, replica("clara")).await;
    clara.settle().await;
    let download = clara
        .blob_ticket(hash, 0, false)
        .await
        .expect("medic may download");
    assert_eq!(
        http(addr, "GET", &path, Some(&download), b"").await,
        (200, blob.clone())
    );
    assert_eq!(
        http(addr, "GET", &path, None, b"").await.0,
        403,
        "no ticket, no attachment"
    );
    assert_eq!(
        http(addr, "PUT", &path, Some(&download), &blob).await.0,
        403,
        "a download ticket does not upload"
    );

    let mut ben = Client::connect(addr, replica("ben")).await;
    ben.settle().await;
    assert_eq!(
        ben.blob_ticket(hash, 0, false).await,
        None,
        "the commander gets no clinical attachments"
    );
    let unknown = *blake3::hash(b"never uploaded").as_bytes();
    assert_eq!(
        clara.blob_ticket(unknown, 0, false).await,
        None,
        "no ticket for an attachment that is not in the stream"
    );
}

fn blob_hex(hash: &[u8; 32]) -> String {
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

async fn http_get(addr: std::net::SocketAddr, path: &str) -> (u16, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    let request = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    let status = response.split_whitespace().nth(1).unwrap().parse().unwrap();
    let body = response
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    (status, body)
}

// [itest->dsn~store-before-send~1]
// [itest->dsn~role-policies~1]
// [itest->dsn~device-proof~1]
#[tokio::test(flavor = "multi_thread")]
async fn sync_over_websocket_survives_a_restart() {
    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [5; 32]);
    let running = start_server(dir.path()).await.unwrap();
    let addr = running.addr;

    // A medic registers a patient; the commander sees only the counts.
    let mut anna = Client::connect(addr, replica("anna")).await;
    anna.register("DE-0042", Immediate);
    anna.sync().await;
    assert_eq!(anna.replica.outbox_len(), 0);

    let mut ben = Client::connect(addr, replica("ben")).await;
    assert!(!ben.replica.is_granted(&incident(), "med.clinical"));
    ben.wait_for_counts(&[(1, 0, 0)]).await;
    assert!(
        ben.replica
            .stored_events()
            .all(|e| e.decode_body().unwrap().compartment.as_str() == "incident")
    );

    // Health, readiness and metrics.
    assert_eq!(http_get(addr, "/healthz").await, (200, "ok\n".into()));
    assert_eq!(http_get(addr, "/readyz").await.0, 200);
    let (_, metrics) = http_get(addr, "/metrics").await;
    assert!(
        metrics.contains("ordo_events_total 2"),
        "patient + derived counts: {metrics}"
    );
    assert!(metrics.contains("ordo_sessions 2"), "{metrics}");

    // [itest->dsn~protocol-versions~1]
    // API versions (ADR-0052): announced, and negotiated per request.
    let http = reqwest::Client::new();
    let url = |path: &str| format!("http://{addr}{path}");
    let announced: serde_json::Value = serde_json::from_str(
        &http
            .get(url("/.well-known/ordo"))
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap(),
    )
    .unwrap();
    assert_eq!(announced["sync"]["min"], "1.0");
    assert_eq!(announced["http"]["max"], "1.0");
    let version_of = |r: &reqwest::Response| {
        r.headers()
            .get("ORDO-API-Version")
            .map(|v| v.to_str().unwrap().to_owned())
    };
    let none = http.get(url("/auth/config")).send().await.unwrap();
    assert_eq!(
        version_of(&none).as_deref(),
        Some("1.0"),
        "no version: the oldest"
    );
    let oldest = http
        .get(url("/auth/config"))
        .header("ORDO-API-Version", "1.0")
        .send()
        .await
        .unwrap();
    assert_eq!(version_of(&oldest).as_deref(), Some("1.0"));
    for unsupported in ["9.0", "0.1", "one"] {
        let refused = http
            .get(url("/auth/config"))
            .header("ORDO-API-Version", unsupported)
            .send()
            .await
            .unwrap();
        assert_eq!(refused.status(), 406, "{unsupported}");
        let range: serde_json::Value =
            serde_json::from_str(&refused.text().await.unwrap()).unwrap();
        assert_eq!(range["http"]["min"], "1.0", "the answer names the range");
    }

    // [itest->dsn~situation-map~2]
    // [itest->dsn~map-region~2]
    // The incident's map region is read in parts (range requests), as PMTiles does, with a
    // ticket for that incident.
    let region = format!("/tiles/{}.pmtiles", incident().ulid());
    let ticket = anna
        .map_ticket()
        .await
        .expect("Anna takes part in the incident");
    let part = http
        .get(url(&region))
        .header("Range", "bytes=0-6")
        .bearer_auth(&ticket)
        .send()
        .await
        .unwrap();
    assert_eq!(part.status(), 206);
    assert_eq!(part.text().await.unwrap(), "PMTiles");
    let status = |r: reqwest::Response| r.status();
    assert_eq!(
        status(http.get(url(&region)).send().await.unwrap()),
        403,
        "not without a ticket"
    );
    let other = format!("/tiles/{}.pmtiles", Ulid::from_parts(T0, 99).unwrap());
    assert_eq!(
        status(
            http.get(url(&other))
                .bearer_auth(&ticket)
                .send()
                .await
                .unwrap()
        ),
        403,
        "not for another incident"
    );

    drop((anna, ben));
    running.stop().await.unwrap();

    // Restart on the same data: events, keys and derived state are restored.
    let running = start_server(dir.path()).await.unwrap();
    let addr = running.addr;
    let clara = Client::connect(addr, replica("clara")).await;
    let register = clara.patients();
    assert_eq!(
        register.patient("DE-0042").unwrap().category(),
        Immediate,
        "decryptable after restart"
    );

    let mut anna = Client::connect(addr, replica("anna")).await;
    anna.register("DE-0043", Urgent);
    anna.sync().await;
    let ben = Client::connect(addr, replica("ben")).await;
    assert_eq!(
        ben.counts(),
        [(1, 1, 0)],
        "counts continue from the restored state"
    );

    running.stop().await.unwrap();
}

// [itest->dsn~uhs-staff~1]
#[tokio::test(flavor = "multi_thread")]
async fn only_a_uhs_staff_registers_its_patients_once_it_has_a_staff_list() {
    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [7; 32]);
    let running = start_server(dir.path()).await.unwrap();
    let addr = running.addr;

    // Without a staff list, every medic registers (as before).
    let mut clara = Client::connect(addr, replica("clara")).await;
    clara.register("DE-0001", Urgent);
    clara.push_and_settle().await;
    assert!(
        clara.replica.rejected().is_empty(),
        "{:?}",
        clara.replica.rejected()
    );

    // UHS 1 gets a staff list with Anna only.
    let mut anna = Client::connect(addr, replica("anna")).await;
    let staff = patients::UhsUpdated {
        id: "UHS 1".into(),
        name: None,
        location: None,
        capacity: None,
        staff: Some(vec![format!("anna@{SERVER}")]),
    }
    .encode()
    .unwrap();
    anna.replica
        .record(
            &incident(),
            patients::UHS_COMPARTMENT,
            patients::UHS_MODULE,
            patients::UHS_UPDATED,
            1,
            &staff,
            now(),
        )
        .unwrap();
    anna.push_and_settle().await;
    assert!(
        anna.replica.rejected().is_empty(),
        "{:?}",
        anna.replica.rejected()
    );

    // Clara is not on it: the server refuses her registration there.
    clara.register("DE-0002", Urgent);
    clara.push_and_settle().await;
    assert!(
        clara
            .replica
            .rejected()
            .iter()
            .any(|r| r.reason == Rejection::Forbidden),
        "{:?}",
        clara.replica.rejected()
    );
    // Anna is; a physician works at every UHS.
    anna.register("DE-0003", Immediate);
    anna.push_and_settle().await;
    let mut petra = Client::connect(addr, replica("petra")).await;
    petra.register("DE-0004", Immediate);
    petra.push_and_settle().await;
    assert!(anna.replica.rejected().is_empty() && petra.replica.rejected().is_empty());

    running.stop().await.unwrap();
}

// [itest->dsn~map-region~2]
#[tokio::test(flavor = "multi_thread")]
async fn the_incident_command_cuts_its_map_region_from_the_world_map_over_http() {
    use message::{MapRegionCut, MapRegionRequest, MapRegionState};
    // The "world map", served with range requests like the Protomaps build.
    let world = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/e2e/support/world.pmtiles");
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let source = format!("http://{}/world.pmtiles", listener.local_addr().unwrap());
    let files = axum::Router::new().route_service(
        "/world.pmtiles",
        tower_http::services::ServeFile::new(world),
    );
    tokio::spawn(async move { axum::serve(listener, files).await.unwrap() });

    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [8; 32]);
    let maps = dir.path().join("data/maps");
    let region = maps.join(format!("{}.pmtiles", incident().ulid()));
    let running = start_server_with(
        dir.path(),
        &[
            ("ORDO_MAP_SOURCE", source),
            ("ORDO_MAP_DIR", maps.display().to_string()),
        ],
    )
    .await
    .unwrap();
    let addr = running.addr;
    let augsburg = MapRegionRequest {
        lat_e7: 483_700_000,
        lon_e7: 109_000_000,
        radius_m: 10_000,
    };

    let cut = ClientMessage::CutMapRegion(MapRegionCut {
        incident: incident(),
        area: augsburg,
    });
    let get = ClientMessage::GetMapRegion(incident());

    // A medic may not cut; she learns nothing.
    let mut anna = Client::connect(addr, replica("anna")).await;
    let answer = anna.map_region(cut.clone()).await;
    assert_eq!(
        (answer.state, answer.incident.as_ref(), answer.area),
        (MapRegionState::None, Some(&incident()), None)
    );

    // The incident command cuts it; the server answers with the progress until it is ready.
    let mut ben = Client::connect(addr, replica("ben")).await;
    let before = ben.map_region(get.clone()).await;
    assert_eq!(before.state, MapRegionState::None, "no region yet");
    let started = ben.map_region(cut.clone()).await;
    assert!(matches!(
        started.state,
        MapRegionState::Running | MapRegionState::Ready
    ));
    let mut state = started;
    for _ in 0..100 {
        if state.state != MapRegionState::Running {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
        state = ben.map_region(get.clone()).await;
    }
    assert_eq!(state.state, MapRegionState::Ready, "{state:?}");
    assert_eq!(state.area, Some(augsburg));
    assert!(state.bytes.unwrap_or(0) > 0);

    // The region is a PMTiles archive with the tiles around Augsburg, zoom 0 to 3.
    let reader = pmtiles::AsyncPmTilesReader::new_with_path(&region)
        .await
        .unwrap();
    assert_eq!(reader.get_header().max_zoom, 3);
    let centre = pmtiles::TileCoord::new(3, 4, 2).unwrap();
    assert!(
        reader.get_tile(centre).await.unwrap().is_some(),
        "Augsburg's tile at zoom 3"
    );
    let far = pmtiles::TileCoord::new(3, 0, 7).unwrap();
    assert!(
        reader.get_tile(far).await.unwrap().is_none(),
        "nothing far away"
    );

    // The devices of the incident read it with a ticket.
    let http = reqwest::Client::new();
    let ticket = anna.map_ticket().await.expect("a ticket for the incident");
    let part = http
        .get(format!("http://{addr}/tiles/{}.pmtiles", incident().ulid()))
        .header("Range", "bytes=0-6")
        .bearer_auth(ticket)
        .send()
        .await
        .unwrap();
    assert_eq!(part.status(), 206);
    assert_eq!(part.text().await.unwrap(), "PMTiles");

    // After a restart, the server still knows the region; planners see it too.
    drop((anna, ben));
    running.stop().await.unwrap();
    let running = start_server_with(dir.path(), &[("ORDO_MAP_DIR", maps.display().to_string())])
        .await
        .unwrap();
    let mut dora = Client::connect(running.addr, replica("dora")).await;
    let after = dora.map_region(get).await;
    assert_eq!(
        (after.state, after.area),
        (MapRegionState::Ready, Some(augsburg))
    );
    running.stop().await.unwrap();
}

impl Client {
    /// Records an event of the orders module; returns its ID.
    fn order_event(&mut self, event_type: &str, payload: &[u8]) -> ordo_core::EventId {
        self.replica
            .record(
                &incident(),
                orders::COMPARTMENT,
                orders::MODULE,
                event_type,
                1,
                payload,
                now(),
            )
            .unwrap()
    }

    /// Pushes and waits until the server has ordered or rejected everything.
    async fn push_and_settle(&mut self) {
        if let Some(push) = self.replica.push() {
            self.send(push).await;
        }
        self.receive_until(|r| r.outbox_len() == 0).await;
    }
}

// [itest->dsn~order-permission~1]
// [itest->dsn~medical-orders~2]
#[tokio::test(flavor = "multi_thread")]
async fn only_a_physician_gives_orders_and_the_crew_carries_them_out() {
    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [6; 32]);
    let running = start_server(dir.path()).await.unwrap();
    let addr = running.addr;
    let order = |id: &str| {
        OrderGiven {
            order: id.into(),
            tag: "DE-0042".into(),
            ordered: Ordered::Medication {
                substance: "Paracetamol".into(),
                dose: "1 g".into(),
                route: "iv".into(),
            },
            note: None,
        }
        .encode()
        .unwrap()
    };

    // A medic's device writes an order anyway (e.g. an old app): the server refuses it.
    let mut anna = Client::connect(addr, replica("anna")).await;
    let refused = anna.order_event(orders::GIVEN, &order("by-anna"));
    anna.push_and_settle().await;
    assert!(
        anna.replica
            .rejected()
            .iter()
            .any(|r| r.event == refused && r.reason == Rejection::Forbidden),
        "{:?}",
        anna.replica.rejected()
    );

    // The physician gives it; the medic acknowledges and completes it.
    let mut petra = Client::connect(addr, replica("petra")).await;
    petra.order_event(orders::GIVEN, &order("by-petra"));
    petra.sync().await;
    anna.receive_until(|r| {
        OrderBook::from_events(&r.events(&incident(), orders::COMPARTMENT).unwrap())
            .orders
            .contains_key("by-petra")
    })
    .await;
    for status in [OrderStatus::Acknowledged, OrderStatus::Done] {
        let update = OrderUpdated {
            order: "by-petra".into(),
            status,
            performed_by: None,
        }
        .encode()
        .unwrap();
        anna.order_event(orders::UPDATED, &update);
    }
    anna.sync().await;
    let book = OrderBook::from_events(
        &anna
            .replica
            .events(&incident(), orders::COMPARTMENT)
            .unwrap(),
    );
    assert!(
        !book.orders.contains_key("by-anna"),
        "the refused order is gone"
    );
    let done = &book.orders["by-petra"];
    assert_eq!(done.status, OrderStatus::Done);
    assert_eq!(done.by().as_str(), format!("petra@{SERVER}"));
    assert_eq!(done.history[2].by.as_str(), format!("anna@{SERVER}"));

    running.stop().await.unwrap();
}

// [itest->dsn~wrapped-keys-at-rest~1]
#[tokio::test(flavor = "multi_thread")]
async fn a_wrong_master_key_cannot_open_stored_keys() {
    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [5; 32]);
    let running = start_server(dir.path()).await.unwrap();
    let mut anna = Client::connect(running.addr, replica("anna")).await;
    anna.register("DE-0042", Immediate);
    anna.sync().await;
    drop(anna);
    running.stop().await.unwrap();

    // The data key of med.clinical is stored, but only wrapped with the master key.
    let store = ordo_server::store::Store::open(&dir.path().join("data/ordo.db"))
        .await
        .unwrap();
    let master = ordo_crypto::seal::DataKey::from_bytes([5; 32]);
    let keys = store.load_keys(&master).await.unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].0.1.as_str(), "med.clinical");
    let wrong = ordo_crypto::seal::DataKey::from_bytes([6; 32]);
    let error = store
        .load_keys(&wrong)
        .await
        .expect_err("wrong key must fail");
    assert!(
        error.to_string().contains("data key cannot be unwrapped"),
        "{error}"
    );

    std::fs::write(dir.path().join("master.key"), to_hex(&[6; 32])).unwrap();
    let error = start_server(dir.path())
        .await
        .err()
        .expect("start must fail");
    assert!(error.to_string().contains("wrong master key"), "{error}");
}

// [itest->dsn~wrapped-keys-at-rest~1]
#[tokio::test(flavor = "multi_thread")]
async fn the_database_holds_no_clinical_plaintext() {
    let dir = tempfile::tempdir().unwrap();
    write_setup(dir.path(), [5; 32]);
    let running = start_server(dir.path()).await.unwrap();
    let mut anna = Client::connect(running.addr, replica("anna")).await;
    anna.register("DE-0042", Immediate);
    anna.sync().await;
    drop(anna);
    running.stop().await.unwrap();

    let mut bytes = Vec::new();
    for entry in std::fs::read_dir(dir.path().join("data")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_file() {
            bytes.extend(std::fs::read(path).unwrap());
        }
    }
    assert!(!bytes.is_empty());
    assert!(
        !String::from_utf8_lossy(&bytes).contains("DE-0042"),
        "tag number stored in plain text"
    );
}
