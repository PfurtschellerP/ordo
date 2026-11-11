//! The device side: local copy of the streams the user may read, plus the outbox.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use ordo_core::event::ENVELOPE_VERSION;
use ordo_core::{
    Author, CompartmentId, DeviceId, EventBody, EventId, EventType, HlcClock, IncidentId, KeyId,
    ModuleId, Payload, SequencedEvent, SignedEvent, Ulid, UserId,
};
use ordo_crypto::seal::{self, DataKey, SealContext};
use ordo_crypto::{CHAIN_GENESIS, EventSigner, chain_next, event_hash};

use crate::SyncError;
use crate::applied::{AppliedEvent, Entropy, StreamKey, random_u128};
use crate::message::{ClientMessage, Hello, KeyGrant, Rejected, ServerMessage, Subscription};
use crate::version::{SYNC as SYNC_VERSIONS, Version};

/// The most a [`Replica::push`] carries, in bytes of events (an event is at most 1 MiB);
/// the server accepts messages of up to 16 MiB.
pub const PUSH_BYTES: usize = 4 * 1024 * 1024;

/// Whether a rejection is about the event itself, so sending it again cannot help.
fn is_final(reason: crate::message::Rejection) -> bool {
    use crate::message::Rejection as R;
    matches!(
        reason,
        R::Malformed
            | R::WrongAuthor
            | R::BadSignature
            | R::Forbidden
            | R::WrongSealing
            | R::DuplicateId
    )
}

/// The format of [`ReplicaState`] this version writes. Raise it when the stored form
/// changes, and handle older formats in [`ReplicaState::decode`].
pub const REPLICA_FORMAT: u32 = 1;

/// Everything a device stores locally, for saving and loading (e.g. browser storage).
///
/// Contains the compartment data keys. Stored at rest, it must be protected by the
/// device: the web app encrypts it with its storage key (ADR-0041).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ReplicaState {
    /// The stored format ([`REPLICA_FORMAT`] when written by this version).
    #[serde(default)]
    pub format: u32,
    /// Compartments that must be sealed, once known.
    pub protected: Option<Vec<CompartmentId>>,
    /// Streams the device asks for.
    pub wanted: Vec<StreamKey>,
    /// Streams the server granted.
    pub granted: Vec<StreamKey>,
    /// Streams the server denied.
    pub denied: Vec<StreamKey>,
    /// Data keys.
    pub keys: Vec<KeyGrant>,
    /// Ordered events per stream.
    pub streams: Vec<(StreamKey, Vec<SequencedEvent>)>,
    /// Unconfirmed events.
    pub outbox: Vec<(StreamKey, SignedEvent)>,
    /// End of the device's login (ADR-0040).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<u64>,
}

/// Why a stored [`ReplicaState`] cannot be used.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StoredStateError {
    /// Written by a newer version of ORDO: kept untouched until that version runs again.
    #[error("stored by a newer version of ORDO (format {0})")]
    Newer(u32),
    /// Not a stored state, or damaged.
    #[error("the stored state cannot be read: {0}")]
    Unreadable(String),
}

impl ReplicaState {
    // [impl->dsn~local-persistence~1]
    /// Decodes a stored state. The format is read first, so a state written by a newer
    /// version is recognised even if its form changed, and never mistaken for none.
    pub fn decode(bytes: &[u8]) -> Result<Self, StoredStateError> {
        #[derive(serde::Deserialize)]
        struct Format {
            #[serde(default)]
            format: u32,
        }
        let format = crate::message::decode::<Format>(bytes)
            .map_err(|e| StoredStateError::Unreadable(e.to_string()))?
            .format;
        if format > REPLICA_FORMAT {
            return Err(StoredStateError::Newer(format));
        }
        // Format 0 (before the first release) has the same form as format 1.
        crate::message::decode(bytes).map_err(|e| StoredStateError::Unreadable(e.to_string()))
    }
}

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
    /// Outbox events sent on this connection and not yet confirmed or rejected.
    in_flight: HashSet<EventId>,
    rejected: Vec<Rejected>,
    /// The server's challenge on the current connection, until the hello answers it.
    challenge: Option<[u8; 32]>,
    /// The protocol version to ask for on this connection (ADR-0052).
    asked: Version,
    /// The protocol version the server confirmed on the current connection.
    version: Option<Version>,
    /// End of this device's login, as the server said in its last welcome (ADR-0040).
    valid_until: Option<u64>,
    /// Permissions beyond compartments, from the last welcome (e.g. `manage_devices`).
    permissions: Vec<String>,
    /// Views and functions per incident, from the last welcome (e.g. `view_situation`).
    incident_permissions: Vec<crate::message::IncidentPermissions>,
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
            in_flight: HashSet::new(),
            rejected: Vec::new(),
            challenge: None,
            asked: SYNC_VERSIONS.max,
            version: None,
            valid_until: None,
            permissions: Vec::new(),
            incident_permissions: Vec::new(),
        }
    }

    /// The local state, for saving.
    pub fn snapshot(&self) -> ReplicaState {
        ReplicaState {
            streams: self
                .streams
                .iter()
                .map(|(key, stream)| (key.clone(), stream.events.clone()))
                .collect(),
            ..self.snapshot_meta()
        }
    }

    /// The state without the streams' events, for storage that keeps events row by row
    /// (see [`Replica::stream_heads`] and [`Replica::events_after`]).
    pub fn snapshot_without_events(&self) -> ReplicaState {
        let mut state = self.snapshot_meta();
        state.streams = self
            .streams
            .keys()
            .map(|key| (key.clone(), Vec::new()))
            .collect();
        state
    }

    fn snapshot_meta(&self) -> ReplicaState {
        ReplicaState {
            format: REPLICA_FORMAT,
            protected: self.protected.clone(),
            wanted: self.wanted.iter().cloned().collect(),
            granted: self.granted.iter().cloned().collect(),
            denied: self.denied.iter().cloned().collect(),
            keys: self
                .keys
                .iter()
                .map(|((incident, compartment), (key_id, key))| KeyGrant {
                    incident: incident.clone(),
                    compartment: compartment.clone(),
                    key_id: key_id.clone(),
                    key: *key.as_bytes(),
                })
                .collect(),
            streams: Vec::new(),
            outbox: self.outbox.clone(),
            valid_until: self.valid_until,
        }
    }

    /// Every stored stream with its last sequence number.
    pub fn stream_heads(&self) -> Vec<(StreamKey, u64)> {
        self.streams
            .iter()
            .map(|(key, stream)| (key.clone(), stream.last_seq()))
            .collect()
    }

    /// The ordered events of a stream after sequence number `after`.
    pub fn events_after(&self, key: &StreamKey, after: u64) -> &[SequencedEvent] {
        self.streams.get(key).map_or(&[][..], |s| {
            let start = s.events.partition_point(|e| e.seq <= after);
            &s.events[start..]
        })
    }

    /// Loads a saved state. Hash chains are checked again, so damaged storage is noticed.
    pub fn restore(
        user: UserId,
        device: DeviceId,
        signer: EventSigner,
        entropy: impl Entropy + 'static,
        state: ReplicaState,
    ) -> Result<Self, SyncError> {
        let mut replica = Self::new(user, device, signer, entropy);
        replica.protected = state.protected;
        replica.wanted = state.wanted.into_iter().collect();
        replica.granted = state.granted.into_iter().collect();
        replica.denied = state.denied.into_iter().collect();
        for grant in state.keys {
            replica.keys.insert(
                (grant.incident, grant.compartment),
                (grant.key_id, DataKey::from_bytes(grant.key)),
            );
        }
        for (key, events) in state.streams {
            for event in events {
                let wall = event.event.decode_body()?.hlc.wall_ms;
                replica.append(&key, event, wall)?;
            }
        }
        for (key, event) in state.outbox {
            let body = event.decode_body()?;
            let _ = replica.clock.observe(&body.hlc, body.hlc.wall_ms);
            replica.outbox.push((key, event));
        }
        replica.valid_until = state.valid_until;
        Ok(replica)
    }

    /// The device this replica runs on.
    pub fn device(&self) -> DeviceId {
        self.author.device
    }

    /// The user of this device.
    pub fn user(&self) -> &UserId {
        &self.author.user
    }

    /// Asks for a stream on the next connection.
    pub fn subscribe(&mut self, incident: &IncidentId, compartment: &str) {
        let compartment: CompartmentId = compartment.parse().expect("valid compartment ID");
        self.wanted.insert((incident.clone(), compartment));
    }

    /// The protocol version the server confirmed on the current connection, if any.
    pub fn version(&self) -> Option<Version> {
        self.version
    }

    /// The first message after connecting.
    ///
    /// Answers the server's challenge with a proof of the device key (ADR-0038), so it
    /// returns `None` until a [`ServerMessage::Challenge`] arrived on this connection.
    // [impl->dsn~device-proof~1]
    pub fn hello(&mut self) -> Option<ClientMessage> {
        let challenge = self.challenge.take()?;
        // A new connection: whatever was sent on the last one goes again.
        self.in_flight.clear();
        let proof = self
            .signer
            .sign_hello(&challenge, &self.author.device, &self.author.user);
        Some(ClientMessage::Hello(Hello {
            user: self.author.user.clone(),
            device: self.author.device,
            proof,
            version: Some(self.asked),
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
        }))
    }

    // [impl->dsn~outbox-and-resend~1]
    /// The next unconfirmed events not yet sent on this connection, oldest first, up to
    /// [`PUSH_BYTES`] (at least one), to send after [`Replica::hello`]; `None` when all
    /// are on their way. Call it again as the server confirms them. Sending an event
    /// again is safe: the server ignores events it has already ordered.
    pub fn push(&mut self) -> Option<ClientMessage> {
        let mut batch = Vec::new();
        let mut bytes = 0;
        for (_, event) in &self.outbox {
            let Ok(id) = event.decode_body().map(|b| b.id) else {
                continue;
            };
            if self.in_flight.contains(&id) {
                continue;
            }
            let size = event.body.len() + 96;
            if !batch.is_empty() && bytes + size > PUSH_BYTES {
                break;
            }
            bytes += size;
            self.in_flight.insert(id);
            batch.push(event.clone());
        }
        (!batch.is_empty()).then_some(ClientMessage::Push(batch))
    }

    /// Number of events not yet confirmed by the server.
    pub fn outbox_len(&self) -> usize {
        self.outbox.len()
    }

    /// Events the server did not accept.
    pub fn rejected(&self) -> &[Rejected] {
        &self.rejected
    }

    /// Whether the device has received a welcome from the server at least once (until
    /// then it does not know which streams it may use).
    pub fn is_ready(&self) -> bool {
        self.protected.is_some()
    }

    // [impl->dsn~access-timeout~1]
    /// Whether this device's login has expired at `now_ms`, as far as the device knows:
    /// it then locks itself, also offline, until the person logs in again.
    pub fn login_expired(&self, now_ms: u64) -> bool {
        self.valid_until.is_some_and(|until| now_ms >= until)
    }

    /// The end of this device's login, if limited.
    pub fn valid_until(&self) -> Option<u64> {
        self.valid_until
    }

    /// Whether the server granted a permission beyond compartments, e.g. `manage_devices`.
    pub fn has_permission(&self, permission: &str) -> bool {
        self.permissions.iter().any(|p| p == permission)
    }

    /// Whether the user may use a view or function (e.g. `view_situation`) in `incident`.
    pub fn may(&self, incident: &IncidentId, permission: &str) -> bool {
        self.incident_permissions
            .iter()
            .any(|p| &p.incident == incident && p.permissions.iter().any(|q| q == permission))
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
            ServerMessage::Challenge(challenge) => {
                self.challenge = Some(challenge.nonce);
                // [impl->dsn~protocol-versions~1]
                // The newest version both sides speak; without a shared one, ask for our
                // newest, which the server refuses with `unsupportedVersion`.
                self.asked = challenge
                    .versions
                    .and_then(|server| SYNC_VERSIONS.agree(&server))
                    .unwrap_or(SYNC_VERSIONS.max);
                Ok(())
            }
            ServerMessage::Welcome(welcome) => {
                self.version = welcome.version;
                self.protected = Some(welcome.protected);
                self.valid_until = welcome.valid_until;
                self.permissions = welcome.permissions;
                self.incident_permissions = welcome.incident_permissions;
                for sub in welcome.granted {
                    let key = (sub.incident, sub.compartment);
                    self.denied.remove(&key);
                    self.granted.insert(key);
                }
                // [impl->dsn~incident-roles~1]
                // A stream the user may no longer read (e.g. a role was withdrawn): its
                // data and key are deleted from the device (need to know).
                for denied in welcome.denied {
                    let key = (denied.incident, denied.compartment);
                    self.granted.remove(&key);
                    self.streams.remove(&key);
                    self.keys.remove(&key);
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
            // Device and incident lists are for the app, not for the replica.
            ServerMessage::Devices(_)
            | ServerMessage::Incidents(_)
            | ServerMessage::People(_)
            | ServerMessage::Vehicles(_)
            | ServerMessage::MapRegion(_)
            | ServerMessage::Avatars(_)
            | ServerMessage::OrganisationRoles(_)
            | ServerMessage::Accounts(_)
            | ServerMessage::BlobTicket(_)
            | ServerMessage::Pong => Ok(()),
            // [impl->dsn~outbox-and-resend~1]
            // Refused for what the event is: it leaves the outbox. Refused for the moment
            // (no session, login expired, clock ahead, …): it stays and goes again on the
            // next connection, so no input is lost.
            ServerMessage::Rejected(rejected) => {
                self.in_flight.remove(&rejected.event);
                if is_final(rejected.reason) {
                    self.outbox
                        .retain(|(_, e)| e.decode_body().map(|b| b.id) != Ok(rejected.event));
                    self.rejected.push(rejected);
                }
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
        self.in_flight.remove(&body.id);
        Ok(())
    }

    // [impl->dsn~incident-log~1]
    /// The incident log (LOG-1, LOG-3, LOG-6): the events of every compartment of
    /// `incident` this device holds, newest first. A compartment the device cannot open
    /// (no key) is left out, as its events are not for this user.
    pub fn log(&self, incident: &IncidentId) -> Vec<AppliedEvent> {
        let compartments: BTreeSet<&CompartmentId> = self
            .streams
            .keys()
            .chain(self.outbox.iter().map(|(k, _)| k))
            .filter(|(i, _)| i == incident)
            .map(|(_, c)| c)
            .collect();
        crate::log::newest_first(
            compartments
                .into_iter()
                .filter_map(|c| self.events(incident, c.as_str()).ok())
                .flatten(),
        )
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

#[cfg(test)]
mod stored_state_tests {
    use super::*;

    fn state(format: u32) -> ReplicaState {
        ReplicaState {
            format,
            protected: None,
            wanted: Vec::new(),
            granted: Vec::new(),
            denied: Vec::new(),
            keys: Vec::new(),
            streams: Vec::new(),
            outbox: Vec::new(),
            valid_until: Some(42),
        }
    }

    // [utest->dsn~local-persistence~1]
    #[test]
    fn a_stored_state_names_its_format_and_newer_ones_are_never_taken_for_none() {
        let bytes = crate::message::encode(&state(REPLICA_FORMAT)).unwrap();
        let decoded = ReplicaState::decode(&bytes).unwrap();
        assert_eq!(
            (decoded.format, decoded.valid_until),
            (REPLICA_FORMAT, Some(42))
        );

        // Before the first release the field was missing: format 0, same form.
        #[derive(serde::Serialize)]
        struct Unversioned {
            protected: Option<Vec<CompartmentId>>,
            wanted: Vec<StreamKey>,
            granted: Vec<StreamKey>,
            denied: Vec<StreamKey>,
            keys: Vec<KeyGrant>,
            streams: Vec<(StreamKey, Vec<SequencedEvent>)>,
            outbox: Vec<(StreamKey, SignedEvent)>,
        }
        let old = crate::message::encode(&Unversioned {
            protected: None,
            wanted: Vec::new(),
            granted: Vec::new(),
            denied: Vec::new(),
            keys: Vec::new(),
            streams: Vec::new(),
            outbox: Vec::new(),
        })
        .unwrap();
        assert_eq!(ReplicaState::decode(&old).unwrap().format, 0);

        // A newer version's state, even of another form, is recognised as such.
        #[derive(serde::Serialize)]
        struct Future {
            format: u32,
            everything: String,
        }
        let newer = crate::message::encode(&Future {
            format: REPLICA_FORMAT + 1,
            everything: "different".into(),
        })
        .unwrap();
        assert_eq!(
            ReplicaState::decode(&newer).err(),
            Some(StoredStateError::Newer(REPLICA_FORMAT + 1))
        );
        assert!(matches!(
            ReplicaState::decode(b"not a state"),
            Err(StoredStateError::Unreadable(_))
        ));
    }
}
