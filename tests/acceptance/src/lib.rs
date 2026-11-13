//! Test harness: a home server and devices connected by a simulated network.
//!
//! Every message is encoded to CBOR and decoded again on the way, as over a real
//! connection. Devices can go offline and online; time is simulated.

use std::collections::BTreeMap;

use ordo_core::{Author, DeviceId, IncidentId, KeyId, ServerName, Ulid, UserId};
use ordo_crypto::EventSigner;
use ordo_crypto::ed25519_dalek::SigningKey;
use ordo_med_patients::PatientCountsDeriver;
use ordo_sync::message::{self, ClientMessage, ServerMessage};
use ordo_sync::{Hub, HubConfig, Outgoing, Replica, SessionId, StaticPolicy};

/// Deterministic randomness for tests (not for production use).
pub fn test_entropy(seed: u64) -> impl FnMut(&mut [u8]) + Send {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    move |buffer: &mut [u8]| {
        for byte in buffer {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = (state >> 24) as u8;
        }
    }
}

/// A device in the simulation.
pub struct Device {
    /// The device's replica.
    pub replica: Replica,
    /// Whether the device has network.
    pub online: bool,
    session: Option<SessionId>,
    /// Drop the next messages from the server (simulates a lost connection mid-sync).
    pub drop_incoming: bool,
}

/// Users of the standard scenario.
pub const MEDICS: [&str; 2] = ["anna", "clara"];
/// The incident commander, without access to clinical data.
pub const COMMANDER: &str = "ben";
/// An administrator: manages all devices, reads no incident data. Has no device unless a
/// test adds one.
pub const ADMIN: &str = "dora";

/// The simulated world: one home server, several devices, one incident.
pub struct World {
    /// The home server.
    pub hub: Hub<StaticPolicy>,
    /// Devices by user name.
    pub devices: BTreeMap<String, Device>,
    /// The incident.
    pub incident: IncidentId,
    /// The server name.
    pub server: ServerName,
    /// Simulated time in milliseconds.
    pub now: u64,
    next_session: SessionId,
}

impl World {
    /// Medics (incident + med.clinical) and a commander (incident only), all online.
    pub fn new() -> Self {
        let server: ServerName = "ordo.drk-musterstadt.de".parse().unwrap();
        let user = |name: &str| UserId::new(name, &server).unwrap();
        let mut policy = StaticPolicy::default();
        for medic in MEDICS {
            policy = policy.allow(&user(medic), &["incident", "med.clinical"]);
        }
        policy = policy.allow(&user(COMMANDER), &["incident"]);
        policy = policy.admin(&user(ADMIN));
        Self::with_policy(policy)
    }

    /// The same people and devices, with access decided by `policy` (e.g. by roles per
    /// incident instead of fixed grants).
    pub fn with_policy(policy: StaticPolicy) -> Self {
        let server: ServerName = "ordo.drk-musterstadt.de".parse().unwrap();
        let now = 1_762_600_000_000;
        let incident = IncidentId::new(Ulid::from_parts(now, 1).unwrap(), &server);
        let user = |name: &str| UserId::new(name, &server).unwrap();
        let hub_device = DeviceId(Ulid::from_parts(now, 999).unwrap());
        let mut hub = Hub::new(
            HubConfig {
                server: server.clone(),
                protected: vec!["med.clinical".parse().unwrap()],
                max_clock_ahead_ms: 60_000,
                vehicles: vec![ordo_sync::message::Vehicle {
                    unit: "Rotkreuz Musterstadt 83/1".into(),
                    unit_type: "RTW".into(),
                    organisation: "DRK Musterstadt".into(),
                    service: Some("rescue".into()),
                    strength: Some([0, 1, 1]),
                    fixed: true,
                }],
            },
            policy,
            EventSigner::new(
                SigningKey::from_bytes(&[99; 32]),
                KeyId::new("server-1", &server).unwrap(),
            ),
            Author {
                user: user("ordo-system"),
                device: hub_device,
            },
            test_entropy(99),
        );
        hub.add_deriver(PatientCountsDeriver::default());
        let mut world = Self {
            hub,
            devices: BTreeMap::new(),
            incident,
            server,
            now,
            next_session: 1,
        };
        for (i, name) in MEDICS.iter().chain([&COMMANDER]).enumerate() {
            world.add_device(name, i as u8 + 1);
        }
        world
    }

    /// Adds and registers a device for `name`, subscribed to both compartments.
    pub fn add_device(&mut self, name: &str, seed: u8) {
        let user = UserId::new(name, &self.server).unwrap();
        let device = DeviceId(Ulid::from_parts(self.now, u128::from(seed)).unwrap());
        let key = SigningKey::from_bytes(&[seed; 32]);
        self.hub
            .register_device(device, user.clone(), key.verifying_key());
        let signer = EventSigner::new(
            key,
            KeyId::new(&format!("device-{name}"), &self.server).unwrap(),
        );
        let mut replica = Replica::new(user, device, signer, test_entropy(u64::from(seed)));
        replica.subscribe(&self.incident, "incident");
        replica.subscribe(&self.incident, "med.clinical");
        self.devices.insert(
            name.to_owned(),
            Device {
                replica,
                online: true,
                session: None,
                drop_incoming: false,
            },
        );
    }

    /// Opens connection `session` for `name` by hand and returns the device's answer to
    /// the server's challenge (for tests that drive the hub directly).
    pub fn hello(&mut self, name: &str, session: SessionId) -> ClientMessage {
        let now = self.now;
        for out in self.hub.connect(session) {
            self.replica(name).receive(out.message, now).unwrap();
        }
        self.replica(name).hello().expect("answer to the challenge")
    }

    /// The replica of `name`.
    pub fn replica(&mut self, name: &str) -> &mut Replica {
        &mut self.devices.get_mut(name).unwrap().replica
    }

    /// Advances simulated time.
    pub fn tick(&mut self, ms: u64) {
        self.now += ms;
    }

    /// Switches the network of `name` on or off.
    pub fn set_online(&mut self, name: &str, online: bool) {
        let device = self.devices.get_mut(name).unwrap();
        device.online = online;
        if !online && let Some(session) = device.session.take() {
            self.hub.disconnect(session);
        }
    }

    /// Lets all online devices connect and exchange messages until nothing changes.
    pub fn sync(&mut self) {
        for _ in 0..10 {
            let mut traffic = false;
            let names: Vec<String> = self.devices.keys().cloned().collect();
            for name in names {
                let device = self.devices.get_mut(&name).unwrap();
                if !device.online {
                    continue;
                }
                let mut messages = Vec::new();
                if device.session.is_none() {
                    let session = self.next_session;
                    device.session = Some(session);
                    self.next_session += 1;
                    // The server's challenge, answered by the hello below.
                    for out in self.hub.connect(session) {
                        let message: ServerMessage =
                            message::decode(&message::encode(&out.message).unwrap()).unwrap();
                        device.replica.receive(message, self.now).unwrap();
                    }
                }
                messages.extend(device.replica.hello());
                messages.extend(device.replica.push());
                let session = device.session.unwrap();
                for message in messages {
                    traffic = true;
                    let message: ClientMessage =
                        message::decode(&message::encode(&message).unwrap()).unwrap();
                    let outgoing = self.hub.handle(session, message, self.now);
                    self.route(outgoing);
                }
            }
            if !traffic {
                break;
            }
        }
    }

    fn route(&mut self, outgoing: Vec<Outgoing>) {
        for Outgoing { session, message } in outgoing {
            let now = self.now;
            let Some(device) = self
                .devices
                .values_mut()
                .find(|d| d.session == Some(session))
            else {
                continue;
            };
            if !device.online || device.drop_incoming {
                continue;
            }
            let message: ServerMessage =
                message::decode(&message::encode(&message).unwrap()).unwrap();
            match device.replica.receive(message, now) {
                Ok(()) => {}
                // The server ended the connection (e.g. roles changed): reconnect later.
                Err(ordo_sync::SyncError::Refused(_)) => device.session = None,
                Err(e) => panic!("{e}"),
            }
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
