//! End-to-end: real server, WebSocket clients, SQLite file, restart.

use std::collections::HashMap;
use std::path::Path;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use ordo_contracts::TriageCategory::{self, Immediate, Urgent};
use ordo_core::{DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_med_patients::{self as patients, PatientRegister, Registered};
use ordo_server::config::to_hex;
use ordo_server::{Config, Running, ServerError, start};
use ordo_sync::Replica;
use ordo_sync::message::{self, BlobRequest, ClientMessage, ServerMessage};
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
const PEOPLE: [(&str, u8, &[&str]); 3] = [
    ("anna", 1, &["medic"]),
    ("clara", 2, &["medic"]),
    ("ben", 3, &["commander"]),
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
}

// [itest->dsn~environment-configuration~1]
async fn start_server(dir: &Path) -> Result<Running, ServerError> {
    let vars: HashMap<&str, String> = HashMap::from([
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
    ]);
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
