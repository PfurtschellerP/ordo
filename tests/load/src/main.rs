//! Load device of the field-near test (dsn~large-incident~1).
//!
//! Pairs like a shared tablet (QR pairing, RFC 8628: someone approves the printed address
//! in a browser), joins an incident by its name and fills it over the sync protocol to the
//! size PERF-3 names: units, sectors with units assigned, UHS, patients, then unit status
//! changes and re-triages up to the number of events. It uses only what every device uses:
//! the pairing endpoints, the sync protocol and the modules' payloads. Position kinds and
//! status codes come from the profile.
//!
//! ```text
//! ordo-load --server http://127.0.0.1:18300 --incident Großeinsatz [--profile profiles/de]
//! ```
//!
//! It prints `verify <address>` once the pairing waits for approval and, at the end,
//! `done <JSON>` with what it wrote and how long it took.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use futures_util::{SinkExt, StreamExt};
use ordo_command::{Core, StructureCatalogue};
use ordo_contracts::TriageCategory;
use ordo_core::{DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId, codec};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_med_patients as med;
use ordo_sync::Replica;
use ordo_sync::message::{self, ClientMessage, ServerMessage};
use ordo_units::{StatusBoard, StatusCatalogue};
use serde::Serialize;
use tokio_tungstenite::tungstenite::Message;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

/// How much to write; the defaults are PERF-3's numbers.
#[derive(Clone, Debug, Serialize)]
struct Size {
    units: usize,
    sectors: usize,
    uhs: usize,
    patients: usize,
    events: usize,
}

struct Options {
    server: String,
    incident: String,
    name: String,
    profile: PathBuf,
    size: Size,
}

fn options() -> Result<Options, Error> {
    let mut o = Options {
        server: String::new(),
        incident: String::new(),
        name: "Lastgerät".into(),
        profile: PathBuf::from("profiles/de"),
        size: Size {
            units: 100,
            sectors: 50,
            uhs: 5,
            patients: 500,
            events: 10_000,
        },
    };
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--server" => o.server = value()?.trim_end_matches('/').to_owned(),
            "--incident" => o.incident = value()?,
            "--name" => o.name = value()?,
            "--profile" => o.profile = value()?.into(),
            "--units" => o.size.units = value()?.parse()?,
            "--sectors" => o.size.sectors = value()?.parse()?,
            "--uhs" => o.size.uhs = value()?.parse()?,
            "--patients" => o.size.patients = value()?.parse()?,
            "--events" => o.size.events = value()?.parse()?,
            _ => return Err(format!("unknown argument {flag}").into()),
        }
    }
    if o.server.is_empty() || o.incident.is_empty() {
        return Err("usage: ordo-load --server URL --incident NAME [--profile DIR] [--name DEVICE] [--units N] [--sectors N] [--uhs N] [--patients N] [--events N]".into());
    }
    if o.size.units == 0 || o.size.sectors == 0 || o.size.uhs == 0 || o.size.patients == 0 {
        return Err("units, sectors, UHS and patients must be at least 1".into());
    }
    Ok(o)
}

fn now() -> u64 {
    u64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after 1970")
            .as_millis(),
    )
    .expect("time fits")
}

fn random_u128() -> u128 {
    let mut b = [0u8; 16];
    getrandom::fill(&mut b).expect("random numbers");
    u128::from_le_bytes(b)
}

fn new_ulid() -> String {
    Ulid::from_parts(now(), random_u128() >> 48)
        .expect("valid ULID")
        .to_string()
}

fn say(line: &str) {
    let mut out = std::io::stdout().lock();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
}

/// Pairs by the device flow and returns the person who approved it.
async fn pair(o: &Options, device: DeviceId, key: &SigningKey) -> Result<UserId, Error> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let http = reqwest::Client::new();
    let public: String = key
        .verifying_key()
        .to_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    let body = serde_json::json!({ "device": device.0.to_string(), "key": public, "name": o.name });
    let response = http
        .post(format!("{}/auth/device", o.server))
        .header("content-type", "application/json")
        .body(body.to_string())
        .send()
        .await?;
    if !response.status().is_success() {
        return Err(format!("pairing refused: {}", response.text().await?).into());
    }
    let started: serde_json::Value = serde_json::from_str(&response.text().await?)?;
    let id = started["id"].as_str().ok_or("no pairing ID")?;
    let address = started["verification_uri_complete"]
        .as_str()
        .or(started["verification_uri"].as_str())
        .ok_or("no address to approve")?;
    say(&format!("verify {address}"));
    let deadline = Instant::now() + Duration::from_secs(600);
    while Instant::now() < deadline {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let text = http
            .get(format!("{}/auth/device/{id}", o.server))
            .send()
            .await?
            .text()
            .await?;
        let status: serde_json::Value = serde_json::from_str(&text)?;
        match status["status"].as_str() {
            Some("pending") => {}
            Some("done") => return Ok(status["user"].as_str().ok_or("no user")?.parse()?),
            _ => return Err(format!("pairing failed: {text}").into()),
        }
    }
    Err("pairing was not approved in time".into())
}

/// A connection to the server with the device's replica.
struct Line {
    socket: Socket,
    replica: Replica,
    incidents: Option<Vec<message::IncidentSummary>>,
}

impl Line {
    async fn open(server: &str, replica: Replica) -> Result<Self, Error> {
        let ws = server.replacen("http", "ws", 1);
        let (socket, _) = tokio_tungstenite::connect_async(format!("{ws}/sync")).await?;
        let mut line = Self {
            socket,
            replica,
            incidents: None,
        };
        // The server opens with a challenge; the hello proves the device key.
        let hello = loop {
            line.receive().await?;
            if let Some(hello) = line.replica.hello() {
                break hello;
            }
        };
        line.send(&hello).await?;
        Ok(line)
    }

    async fn send(&mut self, message: &ClientMessage) -> Result<(), Error> {
        let bytes = message::encode(message)?;
        self.socket.send(Message::Binary(bytes.into())).await?;
        Ok(())
    }

    /// Receives one message (fails after 60 s of silence).
    async fn receive(&mut self) -> Result<(), Error> {
        loop {
            let frame = tokio::time::timeout(Duration::from_secs(60), self.socket.next())
                .await
                .map_err(|_| "the server went silent")?
                .ok_or("the server closed the connection")??;
            if let Message::Binary(bytes) = frame {
                let message: ServerMessage = message::decode(&bytes)?;
                if let ServerMessage::Incidents(list) = &message {
                    self.incidents = Some(list.clone());
                }
                if let ServerMessage::Refused(reason) = &message {
                    return Err(format!("refused: {reason:?}").into());
                }
                self.replica.receive(message, now())?;
                return Ok(());
            }
        }
    }

    async fn until(&mut self, done: impl Fn(&Self) -> bool) -> Result<(), Error> {
        while !done(self) {
            self.receive().await?;
        }
        Ok(())
    }

    /// Sends everything recorded and waits until the server confirmed it.
    async fn flush(&mut self) -> Result<(), Error> {
        loop {
            while let Some(push) = self.replica.push() {
                self.send(&push).await?;
            }
            if self.replica.outbox_len() == 0 {
                break;
            }
            self.receive().await?;
        }
        if let Some(r) = self.replica.rejected().first() {
            return Err(format!(
                "{} event(s) rejected, first: {r:?}",
                self.replica.rejected().len()
            )
            .into());
        }
        Ok(())
    }
}

/// Writes the incident's events, flushing now and then so the server confirms them.
struct Writer {
    line: Line,
    incident: IncidentId,
    written: usize,
    unsent: usize,
}

impl Writer {
    async fn record<T: Serialize>(
        &mut self,
        compartment: &str,
        module: &str,
        event_type: &str,
        schema: u32,
        payload: &T,
    ) -> Result<(), Error> {
        let bytes = codec::to_cbor(payload)?;
        self.line.replica.record(
            &self.incident,
            compartment,
            module,
            event_type,
            schema,
            &bytes,
            now(),
        )?;
        self.written += 1;
        self.unsent += 1;
        if self.unsent >= 500 {
            self.flush().await?;
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), Error> {
        self.line.flush().await?;
        self.unsent = 0;
        Ok(())
    }
}

fn read_toml<T: serde::de::DeserializeOwned>(path: PathBuf) -> Result<T, Error> {
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(toml::from_str(&text)?)
}

async fn fill(w: &mut Writer, o: &Options) -> Result<(), Error> {
    let size = &o.size;
    let structure: StructureCatalogue = read_toml(o.profile.join("command/structure.toml"))?;
    let status: StatusCatalogue = read_toml(o.profile.join("units/status-codes.toml"))?;
    let kind = |core: Core| {
        structure
            .kinds
            .iter()
            .find(|k| k.core == core)
            .map(|k| k.id.clone())
            .ok_or(format!("the profile has no position kind {core:?}"))
    };
    let codes: Vec<String> = status.codes.iter().map(|c| c.code.clone()).collect();
    if codes.is_empty() {
        return Err("the profile has no status codes".into());
    }

    // The command structure: the incident command with its sectors.
    let top = new_ulid();
    w.record(
        ordo_command::COMPARTMENT,
        ordo_command::MODULE,
        ordo_command::POSITION_CREATED,
        1,
        &ordo_command::PositionCreated {
            position: top.clone(),
            kind: kind(Core::Command)?,
            name: "Großeinsatz".into(),
            parent: None,
            service: None,
            organisation: None,
        },
    )
    .await?;
    let sector_kind = kind(Core::Sector)?;
    let mut sectors = Vec::new();
    for i in 1..=size.sectors {
        let id = new_ulid();
        w.record(
            ordo_command::COMPARTMENT,
            ordo_command::MODULE,
            ordo_command::POSITION_CREATED,
            1,
            &ordo_command::PositionCreated {
                position: id.clone(),
                kind: sector_kind.clone(),
                name: format!("EA {i}"),
                parent: Some(top.clone()),
                service: None,
                organisation: None,
            },
        )
        .await?;
        sectors.push(id);
    }

    // The UHS.
    let mut uhs = Vec::new();
    let capacity = u32::try_from(size.patients.div_ceil(size.uhs) + 20)?;
    for i in 1..=size.uhs {
        let id = format!("uhs-{}", new_ulid());
        w.record(
            med::UHS_COMPARTMENT,
            med::UHS_MODULE,
            med::UHS_CREATED,
            1,
            &med::UhsCreated {
                id: id.clone(),
                name: format!("UHS {i}"),
                kind: med::UhsKind::Uhs,
                capacity,
            },
        )
        .await?;
        uhs.push(id);
    }

    // Units, each put under a sector.
    let units: Vec<String> = (1..=size.units).map(|i| format!("Last {i:03}")).collect();
    for unit in &units {
        w.record(
            ordo_units::COMPARTMENT,
            ordo_units::MODULE,
            ordo_units::ADDED,
            ordo_units::ADDED_SCHEMA,
            &ordo_units::UnitAdded {
                unit: unit.clone(),
                unit_type: "RTW".into(),
                organisation: "Lasttest".into(),
                service: None,
                strength: Some(ordo_units::Strength::from([0, 1, 2])),
                special: Default::default(),
            },
        )
        .await?;
    }
    for (i, unit) in units.iter().enumerate() {
        w.record(
            ordo_command::COMPARTMENT,
            ordo_command::MODULE,
            ordo_command::ASSIGNED,
            1,
            &ordo_command::Assigned {
                unit: Some(unit.clone()),
                uhs: None,
                position: Some(sectors[i % sectors.len()].clone()),
            },
        )
        .await?;
    }

    // Patients, spread over the UHS.
    let categories = [
        TriageCategory::Immediate,
        TriageCategory::Urgent,
        TriageCategory::Minor,
        TriageCategory::Minor,
    ];
    let tags: Vec<String> = (1..=size.patients).map(|i| format!("GE-{i:04}")).collect();
    for (i, tag) in tags.iter().enumerate() {
        w.record(
            med::COMPARTMENT,
            med::MODULE,
            med::REGISTERED,
            med::REGISTERED_SCHEMA,
            &med::Registered {
                report: None,
                tag: tag.clone(),
                category: categories[i % categories.len()],
                uhs: uhs[i % uhs.len()].clone(),
                algorithm: None,
            },
        )
        .await?;
    }
    w.flush().await?;

    // The rest of the events: four in five are status changes, in rounds over all units
    // (each naming the status it replaces), the others re-triages.
    let rest = size.events.saturating_sub(w.written);
    let rounds = rest * 4 / 5 / units.len();
    for round in 0..rounds {
        let board = StatusBoard::from_events(
            &w.line
                .replica
                .events(&w.incident, ordo_units::COMPARTMENT)?,
        );
        for (i, unit) in units.iter().enumerate() {
            let payload = ordo_units::StatusChanged {
                unit: unit.clone(),
                status: codes[(round + i) % codes.len()].clone(),
                replaces: board.units.get(unit).map(|s| s.since),
            };
            w.record(
                ordo_units::COMPARTMENT,
                ordo_units::MODULE,
                ordo_units::STATUS_CHANGED,
                1,
                &payload,
            )
            .await?;
        }
        w.flush().await?;
    }
    let mut i = 0;
    while w.written < size.events {
        w.record(
            med::COMPARTMENT,
            med::MODULE,
            med::RETRIAGED,
            med::TRIAGE_SCHEMA,
            &med::Retriaged {
                tag: tags[i % tags.len()].clone(),
                category: categories[(i / tags.len() + i + 1) % categories.len()],
                algorithm: None,
            },
        )
        .await?;
        i += 1;
    }
    w.flush().await
}

async fn run() -> Result<(), Error> {
    let o = options()?;
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed)?;
    let key = SigningKey::from_bytes(&seed);
    let device = DeviceId(Ulid::from_parts(now(), random_u128() >> 48)?);
    let user = pair(&o, device, &key).await?;
    let server: ServerName = user.server().parse()?;
    let signer = EventSigner::new(key, KeyId::new(&format!("device-{}", device.0), &server)?);
    let replica = Replica::new(user, device, signer, |b: &mut [u8]| {
        getrandom::fill(b).expect("random numbers");
    });

    // Find the incident by its name, then connect again subscribed to it.
    let mut line = Line::open(&o.server, replica).await?;
    line.send(&ClientMessage::ListIncidents).await?;
    line.until(|l| l.incidents.is_some()).await?;
    let incident = line
        .incidents
        .iter()
        .flatten()
        .find(|i| i.name == o.incident)
        .map(|i| i.incident.clone())
        .ok_or(format!("no incident {:?} for this person", o.incident))?;
    let mut replica = line.replica;
    replica.subscribe(&incident, ordo_units::COMPARTMENT);
    replica.subscribe(&incident, med::COMPARTMENT);
    let mut line = Line::open(&o.server, replica).await?;
    line.until(|l| {
        l.replica.is_granted(&incident, ordo_units::COMPARTMENT)
            && l.replica.is_granted(&incident, med::COMPARTMENT)
    })
    .await?;

    let start = Instant::now();
    let mut writer = Writer {
        line,
        incident,
        written: 0,
        unsent: 0,
    };
    fill(&mut writer, &o).await?;
    let ms = start.elapsed().as_millis();
    let mut by_type = BTreeMap::<String, usize>::new();
    for compartment in [ordo_units::COMPARTMENT, med::COMPARTMENT] {
        for e in writer.line.replica.events(&writer.incident, compartment)? {
            if e.body.author.device == device {
                *by_type.entry(e.body.event_type.to_string()).or_default() += 1;
            }
        }
    }
    let done = serde_json::json!({
        "size": o.size,
        "written": writer.written,
        "confirmed": by_type.values().sum::<usize>(),
        "byType": by_type,
        "ms": ms,
    });
    say(&format!("done {done}"));
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("ordo-load: {e}");
        std::process::exit(1);
    }
}
