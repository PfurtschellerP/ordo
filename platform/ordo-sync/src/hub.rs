//! The server side: orders, stores and distributes events.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ordo_core::event::ENVELOPE_VERSION;
use ordo_core::{
    Author, CompartmentId, DeviceId, EventBody, EventId, HlcClock, IncidentId, KeyId, Payload,
    SequencedEvent, ServerName, SignedEvent, Ulid, UserId,
};
use ordo_crypto::ed25519_dalek::VerifyingKey;
use ordo_crypto::seal::{self, DataKey, SealContext};
use ordo_crypto::{Chain, EventSigner};

use crate::applied::{AppliedEvent, Derived, Deriver, Entropy, StreamKey, random_u128};
use crate::message::{
    ClientMessage, Delivery, Hello, KeyGrant, Rejected, Rejection, ServerMessage, StreamRef,
    Subscription, Welcome,
};
use crate::policy::AccessPolicy;

/// Identifies one connection of a device.
pub type SessionId = u64;

/// A message for one session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outgoing {
    /// Receiving session.
    pub session: SessionId,
    /// The message.
    pub message: ServerMessage,
}

/// Configuration of a hub.
#[derive(Debug, Clone)]
pub struct HubConfig {
    /// Name of this server.
    pub server: ServerName,
    /// Compartments whose payloads must be sealed, e.g. `med.clinical`.
    pub protected: Vec<CompartmentId>,
    /// How far an event's clock may be ahead of the server's.
    pub max_clock_ahead_ms: u64,
}

struct RegisteredDevice {
    user: UserId,
    key: VerifyingKey,
}

#[derive(Default)]
struct Stream {
    chain: Chain,
    events: Vec<SequencedEvent>,
    ids: HashSet<EventId>,
}

struct Session {
    device: DeviceId,
    user: UserId,
    streams: HashSet<StreamKey>,
}

/// The server side of sync for the incidents this server is home of.
pub struct Hub<P> {
    config: HubConfig,
    policy: P,
    signer: EventSigner,
    identity: Author,
    clock: HlcClock,
    entropy: Box<dyn Entropy>,
    devices: HashMap<DeviceId, RegisteredDevice>,
    streams: HashMap<StreamKey, Stream>,
    keys: HashMap<StreamKey, (KeyId, DataKey)>,
    sessions: BTreeMap<SessionId, Session>,
    derivers: Vec<Box<dyn Deriver>>,
}

impl<P: AccessPolicy> Hub<P> {
    /// A hub that signs its own (derived) events with `signer`, as `identity`.
    pub fn new(
        config: HubConfig,
        policy: P,
        signer: EventSigner,
        identity: Author,
        entropy: impl Entropy + 'static,
    ) -> Self {
        Self {
            clock: HlcClock::with_max_ahead(identity.device, config.max_clock_ahead_ms),
            config,
            policy,
            signer,
            identity,
            entropy: Box::new(entropy),
            devices: HashMap::new(),
            streams: HashMap::new(),
            keys: HashMap::new(),
            sessions: BTreeMap::new(),
            derivers: Vec::new(),
        }
    }

    /// The public key of the hub's own events.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signer.verifying_key()
    }

    /// Registers a paired device of `user` with its public key.
    pub fn register_device(&mut self, device: DeviceId, user: UserId, key: VerifyingKey) {
        self.devices.insert(device, RegisteredDevice { user, key });
    }

    /// Adds a connector deriver.
    pub fn add_deriver(&mut self, deriver: impl Deriver + 'static) {
        self.derivers.push(Box::new(deriver));
    }

    /// All ordered events of a stream (for persistence and tests).
    pub fn stream(&self, incident: &IncidentId, compartment: &CompartmentId) -> &[SequencedEvent] {
        self.streams
            .get(&(incident.clone(), compartment.clone()))
            .map(|s| s.events.as_slice())
            .unwrap_or_default()
    }

    /// The session ended (network lost, app closed).
    pub fn disconnect(&mut self, session: SessionId) {
        self.sessions.remove(&session);
    }

    /// Handles one message from `session` and returns the messages to send.
    pub fn handle(
        &mut self,
        session: SessionId,
        message: ClientMessage,
        now_ms: u64,
    ) -> Vec<Outgoing> {
        match message {
            ClientMessage::Hello(hello) => self.hello(session, hello),
            ClientMessage::Push(events) => self.push(session, events, now_ms),
        }
    }

    // [impl->dsn~compartment-scoped-sync~1]
    fn hello(&mut self, session: SessionId, hello: Hello) -> Vec<Outgoing> {
        let registered = self
            .devices
            .get(&hello.device)
            .is_some_and(|d| d.user == hello.user);
        if !registered {
            return vec![Outgoing {
                session,
                message: ServerMessage::Refused(Rejection::UnknownDevice),
            }];
        }
        let mut welcome = Welcome {
            granted: Vec::new(),
            denied: Vec::new(),
            protected: self.config.protected.clone(),
            keys: Vec::new(),
        };
        let mut deliveries = Vec::new();
        let mut streams = HashSet::new();
        for sub in hello.subscriptions {
            let key = (sub.incident.clone(), sub.compartment.clone());
            if !self
                .policy
                .can_read(&hello.user, &sub.incident, &sub.compartment)
            {
                welcome.denied.push(StreamRef {
                    incident: sub.incident,
                    compartment: sub.compartment,
                });
                continue;
            }
            if self.is_protected(&sub.compartment) {
                let (key_id, data_key) = self.data_key(&key);
                welcome.keys.push(KeyGrant {
                    incident: sub.incident.clone(),
                    compartment: sub.compartment.clone(),
                    key_id,
                    key: *data_key.as_bytes(),
                });
            }
            let missing: Vec<SequencedEvent> = self
                .streams
                .get(&key)
                .map(|s| {
                    s.events
                        .iter()
                        .filter(|e| e.seq > sub.after)
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            if !missing.is_empty() {
                deliveries.push(Delivery {
                    incident: sub.incident.clone(),
                    compartment: sub.compartment.clone(),
                    events: missing,
                });
            }
            streams.insert(key);
            welcome.granted.push(sub);
        }
        self.sessions.insert(
            session,
            Session {
                device: hello.device,
                user: hello.user,
                streams,
            },
        );
        let mut out = vec![Outgoing {
            session,
            message: ServerMessage::Welcome(welcome),
        }];
        out.extend(deliveries.into_iter().map(|d| Outgoing {
            session,
            message: ServerMessage::Deliver(d),
        }));
        out
    }

    // [impl->dsn~compartment-scoped-sync~1]
    // [impl->dsn~outbox-and-resend~1]
    fn push(&mut self, session: SessionId, events: Vec<SignedEvent>, now_ms: u64) -> Vec<Outgoing> {
        let mut out = Vec::new();
        let mut new: BTreeMap<StreamKey, Vec<SequencedEvent>> = BTreeMap::new();
        let Some(s) = self.sessions.get(&session) else {
            return events
                .into_iter()
                .filter_map(|e| e.decode_body().ok())
                .map(|body| Outgoing {
                    session,
                    message: ServerMessage::Rejected(Rejected {
                        event: body.id,
                        reason: Rejection::NotConnected,
                    }),
                })
                .collect();
        };
        let (device, user) = (s.device, s.user.clone());
        for event in events {
            match self.accept(&device, &user, event, now_ms) {
                Ok(Some((key, sequenced, body))) => {
                    new.entry(key.clone()).or_default().push(sequenced);
                    for (dkey, derived) in self.derive(&key, &body, now_ms) {
                        new.entry(dkey).or_default().push(derived);
                    }
                }
                Ok(None) => {} // duplicate: already ordered, nothing to do
                Err((id, reason)) => out.push(Outgoing {
                    session,
                    message: ServerMessage::Rejected(Rejected { event: id, reason }),
                }),
            }
        }
        for (key, events) in new {
            for (&sid, session) in &self.sessions {
                if session.streams.contains(&key) {
                    out.push(Outgoing {
                        session: sid,
                        message: ServerMessage::Deliver(Delivery {
                            incident: key.0.clone(),
                            compartment: key.1.clone(),
                            events: events.clone(),
                        }),
                    });
                }
            }
        }
        out
    }

    /// Checks and orders one event. `Ok(None)` means it was already ordered.
    #[allow(clippy::type_complexity)]
    fn accept(
        &mut self,
        device: &DeviceId,
        user: &UserId,
        event: SignedEvent,
        now_ms: u64,
    ) -> Result<Option<(StreamKey, SequencedEvent, AppliedEvent)>, (EventId, Rejection)> {
        let body = event
            .decode_body()
            .map_err(|_| (EventId(Ulid::from_bytes([0; 16])), Rejection::Malformed))?;
        let id = body.id;
        let reject = |reason| (id, reason);
        if body.author.device != *device || body.author.user != *user {
            return Err(reject(Rejection::WrongAuthor));
        }
        let registered = self
            .devices
            .get(device)
            .ok_or(reject(Rejection::UnknownDevice))?;
        ordo_crypto::verify(&event, &registered.key)
            .map_err(|_| reject(Rejection::BadSignature))?;
        if !self
            .policy
            .can_write(user, &body.incident, &body.compartment)
        {
            return Err(reject(Rejection::Forbidden));
        }
        let key = (body.incident.clone(), body.compartment.clone());
        if self.streams.get(&key).is_some_and(|s| s.ids.contains(&id)) {
            return Ok(None);
        }
        let payload = self
            .open_payload(&key, &body)
            .ok_or(reject(Rejection::WrongSealing))?;
        self.clock
            .observe(&body.hlc, now_ms)
            .map_err(|_| reject(Rejection::ClockAhead))?;
        let sequenced = self.append(&key, event);
        let applied = AppliedEvent {
            seq: Some(sequenced.seq),
            body,
            payload,
        };
        Ok(Some((key, sequenced, applied)))
    }

    // [impl->dsn~sealed-payloads~1]
    /// Checks the sealing rule and returns the plaintext payload.
    fn open_payload(&mut self, key: &StreamKey, body: &EventBody) -> Option<Vec<u8>> {
        match (&body.payload, self.is_protected(&key.1)) {
            (Payload::Plain(bytes), false) => Some(bytes.clone()),
            (Payload::Sealed(sealed), true) => {
                let (key_id, data_key) = self.keys.get(key)?;
                if sealed.key != *key_id {
                    return None;
                }
                let context = SealContext {
                    event: body.id,
                    incident: &key.0,
                    compartment: &key.1,
                };
                seal::open(data_key, sealed, context).ok()
            }
            _ => None,
        }
    }

    fn append(&mut self, key: &StreamKey, event: SignedEvent) -> SequencedEvent {
        let stream = self.streams.entry(key.clone()).or_default();
        let id = event.decode_body().map(|b| b.id).ok();
        let sequenced = stream.chain.append(event);
        if let Some(id) = id {
            stream.ids.insert(id);
        }
        stream.events.push(sequenced.clone());
        sequenced
    }

    // [impl->dsn~server-derived-connectors~1]
    /// Runs the derivers for a newly ordered event and orders their output.
    fn derive(
        &mut self,
        source: &StreamKey,
        event: &AppliedEvent,
        now_ms: u64,
    ) -> Vec<(StreamKey, SequencedEvent)> {
        let derived: Vec<Derived> = self
            .derivers
            .iter_mut()
            .filter(|d| d.source() == &source.1)
            .flat_map(|d| d.on_event(event))
            .collect();
        let mut out = Vec::new();
        for d in derived {
            let key = (source.0.clone(), d.compartment.clone());
            let Ok(hlc) = self.clock.now(now_ms) else {
                continue;
            };
            let id = EventId(
                Ulid::from_parts(now_ms, random_u128(self.entropy.as_mut()))
                    .expect("timestamp fits into 48 bits"),
            );
            let payload = if self.is_protected(&key.1) {
                let (key_id, data_key) = self.data_key(&key);
                let mut nonce = [0u8; 24];
                self.entropy.fill(&mut nonce);
                let context = SealContext {
                    event: id,
                    incident: &key.0,
                    compartment: &key.1,
                };
                Payload::Sealed(seal::seal(&data_key, key_id, nonce, context, &d.payload))
            } else {
                Payload::Plain(d.payload)
            };
            let body = EventBody {
                version: ENVELOPE_VERSION,
                id,
                schema_version: d.schema_version,
                hlc,
                incident: key.0.clone(),
                module: d.module,
                compartment: d.compartment,
                event_type: d.event_type,
                author: self.identity.clone(),
                origin: self.config.server.clone(),
                payload,
            };
            let Ok(signed) = self.signer.sign(&body) else {
                continue;
            };
            let sequenced = self.append(&key, signed);
            out.push((key, sequenced));
        }
        out
    }

    fn is_protected(&self, compartment: &CompartmentId) -> bool {
        self.config.protected.contains(compartment)
    }

    /// The data key of a protected stream, created on first use.
    fn data_key(&mut self, key: &StreamKey) -> (KeyId, DataKey) {
        if let Some((id, data_key)) = self.keys.get(key) {
            return (id.clone(), data_key.clone());
        }
        let mut bytes = [0u8; 32];
        self.entropy.fill(&mut bytes);
        let name = format!("{}-{}", key.0.ulid(), key.1.as_str().replace('.', "-"));
        let id = KeyId::new(&name, &self.config.server).expect("valid key name");
        let data_key = DataKey::from_bytes(bytes);
        self.keys
            .insert(key.clone(), (id.clone(), data_key.clone()));
        (id, data_key)
    }

    /// Streams a session currently receives (for tests).
    pub fn session_streams(&self, session: SessionId) -> BTreeSet<StreamKey> {
        self.sessions
            .get(&session)
            .map(|s| s.streams.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl From<StreamKey> for Subscription {
    fn from((incident, compartment): StreamKey) -> Self {
        Subscription {
            incident,
            compartment,
            after: 0,
        }
    }
}
