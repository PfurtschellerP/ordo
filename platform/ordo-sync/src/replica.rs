//! The device side: local copy of the streams the user may read, plus the outbox.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use ordo_core::event::ENVELOPE_VERSION;
use ordo_core::{
    Author, CompartmentId, DeviceId, EventBody, EventId, EventType, HlcClock, IncidentId, KeyId,
    ModuleId, Payload, SequencedEvent, SignedEvent, Ulid, UserId,
};
use ordo_crypto::seal::{self, DataKey, SealContext};
use ordo_crypto::{CHAIN_GENESIS, EventSigner, chain_next, event_hash};

use crate::SyncError;
use crate::applied::{AppliedEvent, Entropy, StreamKey, random_u128};
use crate::message::{ClientMessage, Hello, Rejected, ServerMessage, Subscription};

#[derive(Default)]
struct LocalStream {
    head: Option<[u8; 32]>,
    events: Vec<SequencedEvent>,
}

impl LocalStream {
    fn last_seq(&self) -> u64 {
        self.events.last().map_or(0, |e| e.seq)
    }
}

/// A device's local copy. It works without a connection; [`Replica::hello`] and
/// [`Replica::push`] produce the messages to send when connected.
///
/// Prototype limitation: events from other devices are trusted after the server has
/// verified their signatures; the device checks order and hash chain. Verifying every
/// signature on the device needs the key directory of the federation specification.
pub struct Replica {
    author: Author,
    signer: EventSigner,
    clock: HlcClock,
    entropy: Box<dyn Entropy>,
    wanted: BTreeSet<StreamKey>,
    granted: BTreeSet<StreamKey>,
    denied: BTreeSet<StreamKey>,
    protected: Option<Vec<CompartmentId>>,
    keys: HashMap<StreamKey, (KeyId, DataKey)>,
    streams: BTreeMap<StreamKey, LocalStream>,
    outbox: Vec<(StreamKey, SignedEvent)>,
    rejected: Vec<Rejected>,
}

impl Replica {
    /// A replica for `user` on `device`, signing with the device key.
    pub fn new(
        user: UserId,
        device: DeviceId,
        signer: EventSigner,
        entropy: impl Entropy + 'static,
    ) -> Self {
        Self {
            author: Author { user, device },
            signer,
            clock: HlcClock::new(device),
            entropy: Box::new(entropy),
            wanted: BTreeSet::new(),
            granted: BTreeSet::new(),
            denied: BTreeSet::new(),
            protected: None,
            keys: HashMap::new(),
            streams: BTreeMap::new(),
            outbox: Vec::new(),
            rejected: Vec::new(),
        }
    }

    /// The device this replica runs on.
    pub fn device(&self) -> DeviceId {
        self.author.device
    }

    /// Asks for a stream on the next connection.
    pub fn subscribe(&mut self, incident: &IncidentId, compartment: &str) {
        let compartment: CompartmentId = compartment.parse().expect("valid compartment ID");
        self.wanted.insert((incident.clone(), compartment));
    }

    /// The first message after connecting.
    pub fn hello(&self) -> ClientMessage {
        ClientMessage::Hello(Hello {
            user: self.author.user.clone(),
            device: self.author.device,
            subscriptions: self
                .wanted
                .iter()
                .map(|(incident, compartment)| Subscription {
                    incident: incident.clone(),
                    compartment: compartment.clone(),
                    after: self
                        .streams
                        .get(&(incident.clone(), compartment.clone()))
                        .map_or(0, LocalStream::last_seq),
                })
                .collect(),
        })
    }

    // [impl->dsn~outbox-and-resend~1]
    /// All unconfirmed events, to send after [`Replica::hello`]. Sending them again is
    /// safe: the server ignores events it has already ordered.
    pub fn push(&self) -> Option<ClientMessage> {
        (!self.outbox.is_empty())
            .then(|| ClientMessage::Push(self.outbox.iter().map(|(_, e)| e.clone()).collect()))
    }

    /// Number of events not yet confirmed by the server.
    pub fn outbox_len(&self) -> usize {
        self.outbox.len()
    }

    /// Events the server did not accept.
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Whether the server granted the stream.
    pub fn is_granted(&self, incident: &IncidentId, compartment: &str) -> bool {
        compartment
            .parse()
            .is_ok_and(|c: CompartmentId| self.granted.contains(&(incident.clone(), c)))
    }

    /// Everything the device stores: ordered events and outbox. Used to check that a
    /// device holds no data it may not read.
    pub fn stored_events(&self) -> impl Iterator<Item = &SignedEvent> {
        self.streams
            .values()
            .flat_map(|s| s.events.iter().map(|e| &e.event))
            .chain(self.outbox.iter().map(|(_, e)| e))
    }

    // [impl->dsn~outbox-and-resend~1]
    // [impl->dsn~sealed-payloads~1]
    /// Records a new event. It is stored in the outbox and visible locally at once.
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        incident: &IncidentId,
        compartment: &str,
        module: &str,
        event_type: &str,
        schema_version: u32,
        payload: &[u8],
        now_ms: u64,
    ) -> Result<EventId, SyncError> {
        let protected = self.protected.as_ref().ok_or(SyncError::NotReady)?;
        let compartment: CompartmentId = compartment
            .parse()
            .map_err(|e| SyncError::Invalid(format!("{e}")))?;
        let key = (incident.clone(), compartment.clone());
        if !self.granted.contains(&key) {
            return Err(SyncError::NoAccess {
                incident: incident.clone(),
                compartment,
            });
        }
        let id = EventId(
            Ulid::from_parts(now_ms, random_u128(self.entropy.as_mut()))
                .map_err(|e| SyncError::Invalid(e.to_string()))?,
        );
        let payload = if protected.contains(&compartment) {
            let (key_id, data_key) = self.keys.get(&key).ok_or_else(|| SyncError::NoKey {
                incident: incident.clone(),
                compartment: compartment.clone(),
            })?;
            let mut nonce = [0u8; 24];
            self.entropy.fill(&mut nonce);
            let context = SealContext {
                event: id,
                incident,
                compartment: &compartment,
            };
            Payload::Sealed(seal::seal(
                data_key,
                key_id.clone(),
                nonce,
                context,
                payload,
            ))
        } else {
            Payload::Plain(payload.to_vec())
        };
        let body = EventBody {
            version: ENVELOPE_VERSION,
            id,
            schema_version,
            hlc: self
                .clock
                .now(now_ms)
                .map_err(|e| SyncError::Invalid(e.to_string()))?,
            incident: incident.clone(),
            module: module
                .parse::<ModuleId>()
                .map_err(|e| SyncError::Invalid(format!("{e}")))?,
            compartment,
            event_type: event_type
                .parse::<EventType>()
                .map_err(|e| SyncError::Invalid(format!("{e}")))?,
            author: self.author.clone(),
            origin: incident
                .server()
                .parse()
                .map_err(|e| SyncError::Invalid(format!("{e}")))?,
            payload,
        };
        let signed = self.signer.sign(&body)?;
        self.outbox.push((key, signed));
        Ok(id)
    }

    /// Handles a message from the server.
    pub fn receive(&mut self, message: ServerMessage, now_ms: u64) -> Result<(), SyncError> {
        match message {
            ServerMessage::Welcome(welcome) => {
                self.protected = Some(welcome.protected);
                for sub in welcome.granted {
                    self.granted.insert((sub.incident, sub.compartment));
                }
                for denied in welcome.denied {
                    let key = (denied.incident, denied.compartment);
                    self.granted.remove(&key);
                    self.denied.insert(key);
                }
                for grant in welcome.keys {
                    self.keys.insert(
                        (grant.incident, grant.compartment),
                        (grant.key_id, DataKey::from_bytes(grant.key)),
                    );
                }
                Ok(())
            }
            ServerMessage::Refused(reason) => Err(SyncError::Refused(reason)),
            ServerMessage::Rejected(rejected) => {
                self.outbox
                    .retain(|(_, e)| e.decode_body().map(|b| b.id) != Ok(rejected.event));
                self.rejected.push(rejected);
                Ok(())
            }
            ServerMessage::Deliver(delivery) => {
                let key = (delivery.incident, delivery.compartment);
                // [impl->dsn~compartment-scoped-sync~1]
                if !self.granted.contains(&key) {
                    return Ok(()); // never store what the user may not read
                }
                for event in delivery.events {
                    self.append(&key, event, now_ms)?;
                }
                Ok(())
            }
        }
    }

    fn append(
        &mut self,
        key: &StreamKey,
        event: SequencedEvent,
        now_ms: u64,
    ) -> Result<(), SyncError> {
        let stream = self.streams.entry(key.clone()).or_default();
        let expected = stream.last_seq() + 1;
        if event.seq < expected {
            return Ok(()); // already have it
        }
        if event.seq > expected {
            return Err(SyncError::Gap {
                expected,
                received: event.seq,
            });
        }
        let previous = stream.head.unwrap_or(CHAIN_GENESIS);
        let head = chain_next(&previous, &event_hash(&event.event));
        if head != event.chain {
            return Err(SyncError::BrokenChain { seq: event.seq });
        }
        let body = event.event.decode_body()?;
        let _ = self.clock.observe(&body.hlc, now_ms);
        stream.head = Some(head);
        stream.events.push(event);
        self.outbox
            .retain(|(_, e)| e.decode_body().map(|b| b.id) != Ok(body.id));
        Ok(())
    }

    /// The events of a stream as modules see them: ordered events first, then the
    /// device's own unconfirmed events, with payloads decrypted.
    pub fn events(
        &self,
        incident: &IncidentId,
        compartment: &str,
    ) -> Result<Vec<AppliedEvent>, SyncError> {
        let compartment: CompartmentId = compartment
            .parse()
            .map_err(|e| SyncError::Invalid(format!("{e}")))?;
        let key = (incident.clone(), compartment);
        let ordered = self
            .streams
            .get(&key)
            .into_iter()
            .flat_map(|s| s.events.iter().map(|e| (Some(e.seq), &e.event)));
        let pending = self
            .outbox
            .iter()
            .filter(|(k, _)| *k == key)
            .map(|(_, e)| (None, e));
        ordered
            .chain(pending)
            .map(|(seq, event)| {
                let body = event.decode_body()?;
                let payload = match &body.payload {
                    Payload::Plain(bytes) => bytes.clone(),
                    Payload::Sealed(sealed) => {
                        let (_, data_key) =
                            self.keys.get(&key).ok_or_else(|| SyncError::NoKey {
                                incident: key.0.clone(),
                                compartment: key.1.clone(),
                            })?;
                        let context = SealContext {
                            event: body.id,
                            incident: &key.0,
                            compartment: &key.1,
                        };
                        seal::open(data_key, sealed, context)?
                    }
                };
                Ok(AppliedEvent { seq, body, payload })
            })
            .collect()
    }
}
