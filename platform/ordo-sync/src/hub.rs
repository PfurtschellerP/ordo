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
use crate::incidents::{self, Effect, Registry};
use crate::message::{
    Challenge, ClientMessage, Delivery, DeviceInfo, Hello, IncidentPermissions, IncidentSummary,
    KeyGrant, Rejected, Rejection, ServerMessage, StreamRef, Subscription, Welcome,
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

/// Something the hub changed that must be stored before messages are sent.
#[derive(Debug, Clone)]
pub enum Change {
    /// A newly ordered event.
    Event {
        /// Its stream.
        stream: StreamKey,
        /// The event.
        event: SequencedEvent,
    },
    /// A newly created data key.
    Key {
        /// Its stream.
        stream: StreamKey,
        /// Name of the key.
        key_id: KeyId,
        /// The key.
        key: DataKey,
    },
    /// A device was revoked (SEC-4).
    Revoked {
        /// The device.
        device: DeviceId,
    },
}

/// Stored data could not be restored.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RestoreError {
    /// Sequence numbers are not consecutive.
    #[error("stream {stream}: expected sequence number {expected}, found {found}")]
    Gap {
        /// The stream.
        stream: String,
        /// Expected sequence number.
        expected: u64,
        /// Found sequence number.
        found: u64,
    },
    /// The hash chain doesn't match: stored data was changed.
    #[error("stream {stream}: hash chain broken at sequence number {seq}")]
    BrokenChain {
        /// The stream.
        stream: String,
        /// Sequence number.
        seq: u64,
    },
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
    /// End of the device's login (ms); `None` for devices without a limit.
    valid_until: Option<u64>,
    /// When it was paired (ms), if known.
    paired: Option<u64>,
}

impl RegisteredDevice {
    fn expired(&self, now_ms: u64) -> bool {
        self.valid_until.is_some_and(|until| now_ms >= until)
    }
}

#[derive(Default)]
struct Stream {
    chain: Chain,
    events: Vec<SequencedEvent>,
    /// Event ID → event hash, to tell resends from conflicting reuse of an ID.
    ids: HashMap<EventId, [u8; 32]>,
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
    /// Challenges sent to connections that have not said hello yet.
    challenges: HashMap<SessionId, [u8; 32]>,
    /// Revoked devices: refused for good, and told to delete their data.
    revoked: HashSet<DeviceId>,
    /// Incidents and their rosters, from incident events (ADR-0046).
    registry: Registry,
    derivers: Vec<Box<dyn Deriver>>,
    changes: Vec<Change>,
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
            challenges: HashMap::new(),
            revoked: HashSet::new(),
            registry: Registry::default(),
            derivers: Vec::new(),
            changes: Vec::new(),
        }
    }

    /// Changes since the last call. The server stores them before it sends the messages
    /// returned by [`Hub::handle`]; if storing fails, it must not send them.
    pub fn take_changes(&mut self) -> Vec<Change> {
        std::mem::take(&mut self.changes)
    }

    /// Restores a stored data key (before restoring events).
    pub fn restore_key(&mut self, stream: StreamKey, key_id: KeyId, key: DataKey) {
        self.keys.insert(stream, (key_id, key));
    }

    /// Restores a stored event. Events of a stream must come in sequence order; the hash
    /// chain is checked. Derivers see source events again, so their state is rebuilt,
    /// but their output is discarded because derived events are restored as well.
    pub fn restore_event(
        &mut self,
        stream: StreamKey,
        event: SequencedEvent,
    ) -> Result<(), RestoreError> {
        let name = format!("{}/{}", stream.0, stream.1);
        let entry = self.streams.entry(stream.clone()).or_default();
        let expected = entry.chain.last_seq() + 1;
        if event.seq != expected {
            return Err(RestoreError::Gap {
                stream: name,
                expected,
                found: event.seq,
            });
        }
        let recomputed = entry.chain.append(event.event.clone());
        if recomputed.chain != event.chain {
            return Err(RestoreError::BrokenChain {
                stream: name,
                seq: event.seq,
            });
        }
        let body = event.event.decode_body().ok();
        if let Some(body) = &body {
            entry
                .ids
                .insert(body.id, ordo_crypto::event_hash(&event.event));
        }
        entry.events.push(event.clone());
        if let Some(body) = body {
            let _ = self.clock.observe(&body.hlc, body.hlc.wall_ms);
            if let Payload::Plain(payload) = &body.payload {
                self.registry.apply(&AppliedEvent {
                    seq: Some(event.seq),
                    body: body.clone(),
                    payload: payload.clone(),
                });
            }
            if self.derivers.iter().any(|d| d.source() == &stream.1)
                && let Some(payload) = self.open_payload(&stream, &body)
            {
                let applied = AppliedEvent {
                    seq: Some(event.seq),
                    body,
                    payload,
                };
                for deriver in self.derivers.iter_mut().filter(|d| d.source() == &stream.1) {
                    let _ = deriver.on_event(&applied);
                }
            }
        }
        Ok(())
    }

    /// The public key of the hub's own events.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signer.verifying_key()
    }

    /// Registers a paired device of `user` with its public key, without a time limit
    /// (devices paired by hand for development).
    pub fn register_device(&mut self, device: DeviceId, user: UserId, key: VerifyingKey) {
        self.register_device_until(device, user, key, None);
    }

    // [impl->dsn~access-timeout~1]
    /// Registers a paired device whose login is valid until `valid_until` (ms).
    pub fn register_device_until(
        &mut self,
        device: DeviceId,
        user: UserId,
        key: VerifyingKey,
        valid_until: Option<u64>,
    ) {
        let paired = self.devices.get(&device).and_then(|d| d.paired);
        self.devices.insert(
            device,
            RegisteredDevice {
                user,
                key,
                valid_until,
                paired,
            },
        );
        self.revoked.remove(&device);
    }

    /// Records when a device was paired (for the device list).
    pub fn set_paired(&mut self, device: &DeviceId, paired_ms: u64) {
        if let Some(d) = self.devices.get_mut(device) {
            d.paired = Some(paired_ms);
        }
    }

    /// Restores a revoked device (from storage).
    pub fn restore_revoked(&mut self, device: DeviceId) {
        self.devices.remove(&device);
        self.revoked.insert(device);
    }

    /// The user of an authenticated connection.
    pub fn session_user(&self, session: SessionId) -> Option<UserId> {
        self.sessions.get(&session).map(|s| s.user.clone())
    }

    // [impl->dsn~incident-roles~1]
    /// The incidents `user` takes part in; all of them for planners.
    fn incident_list(&self, user: &UserId) -> Vec<IncidentSummary> {
        let planner = self.policy.can_manage_incidents(user);
        let mut list: Vec<IncidentSummary> = self
            .registry
            .iter()
            .filter_map(|(id, incident)| {
                let roles = self.registry.roles(user, id);
                (planner || !roles.is_empty()).then(|| IncidentSummary {
                    incident: id.clone(),
                    name: incident.name.clone(),
                    state: incident.state,
                    manage: planner || self.policy.can_assign_roles(user, id, &roles),
                    roles,
                })
            })
            .collect();
        list.sort_by(|a, b| (a.state, &a.name).cmp(&(b.state, &b.name)));
        list
    }

    // [impl->dsn~device-revocation~1]
    /// The devices `user` may see: all with `manage_devices`, else their own.
    fn device_list(&self, user: &UserId) -> Vec<DeviceInfo> {
        let all = self.policy.can_manage_devices(user);
        let mut list: Vec<DeviceInfo> = self
            .devices
            .iter()
            .filter(|(_, d)| all || &d.user == user)
            .map(|(id, d)| DeviceInfo {
                device: *id,
                user: d.user.clone(),
                paired: d.paired,
                valid_until: d.valid_until,
            })
            .collect();
        list.sort_by(|a, b| (&a.user, a.device).cmp(&(&b.user, b.device)));
        list
    }

    // [impl->dsn~device-revocation~1]
    /// Revokes `device` on behalf of `user` (their own device, or any with
    /// `manage_devices`): its sessions are cut with `revoked`, and it is refused from now
    /// on. Returns `false` if the user may not.
    fn revoke(&mut self, user: &UserId, device: DeviceId, out: &mut Vec<Outgoing>) -> bool {
        let allowed = self
            .devices
            .get(&device)
            .is_some_and(|d| &d.user == user || self.policy.can_manage_devices(user));
        if !allowed {
            return false;
        }
        self.devices.remove(&device);
        self.revoked.insert(device);
        self.changes.push(Change::Revoked { device });
        let cut: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.device == device)
            .map(|(id, _)| *id)
            .collect();
        for session in cut {
            self.sessions.remove(&session);
            out.push(Outgoing {
                session,
                message: ServerMessage::Refused(Rejection::Revoked),
            });
        }
        true
    }

    // [impl->dsn~access-timeout~1]
    /// Sets the end of the login of every device of `user` (re-verified: later; refused by
    /// the identity provider: now).
    pub fn set_valid_until(&mut self, user: &UserId, valid_until: u64) {
        for device in self.devices.values_mut().filter(|d| &d.user == user) {
            device.valid_until = Some(valid_until);
        }
    }

    // [impl->dsn~access-timeout~1]
    /// Ends the sessions of devices whose login has expired; returns the refusals to send.
    /// The server calls this regularly, so an open connection does not outlive the login.
    pub fn expire(&mut self, now_ms: u64) -> Vec<Outgoing> {
        let expired: Vec<SessionId> = self
            .sessions
            .iter()
            .filter(|(_, s)| {
                self.devices
                    .get(&s.device)
                    .is_none_or(|d| d.expired(now_ms))
            })
            .map(|(id, _)| *id)
            .collect();
        expired
            .into_iter()
            .map(|session| {
                self.sessions.remove(&session);
                Outgoing {
                    session,
                    message: ServerMessage::Refused(Rejection::LoginExpired),
                }
            })
            .collect()
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

    // [impl->dsn~device-proof~1]
    /// A connection opened: returns the challenge the device must sign in its hello.
    pub fn connect(&mut self, session: SessionId) -> Vec<Outgoing> {
        let mut nonce = [0u8; 32];
        self.entropy.fill(&mut nonce);
        self.challenges.insert(session, nonce);
        vec![Outgoing {
            session,
            message: ServerMessage::Challenge(Challenge { nonce }),
        }]
    }

    /// The session ended (network lost, app closed).
    pub fn disconnect(&mut self, session: SessionId) {
        self.sessions.remove(&session);
        self.challenges.remove(&session);
    }

    /// Handles one message from `session` and returns the messages to send.
    pub fn handle(
        &mut self,
        session: SessionId,
        message: ClientMessage,
        now_ms: u64,
    ) -> Vec<Outgoing> {
        if let ClientMessage::Hello(hello) = message {
            return self.hello(session, hello, now_ms);
        }
        // A connection does not outlive the login of its device.
        let expired = self.expire(now_ms);
        if let Some(refusal) = expired.iter().find(|o| o.session == session) {
            return vec![refusal.clone()];
        }
        let Some(user) = self.sessions.get(&session).map(|s| s.user.clone()) else {
            return match message {
                ClientMessage::Push(events) => self.push(session, events, now_ms),
                _ => vec![Outgoing {
                    session,
                    message: ServerMessage::Refused(Rejection::NotConnected),
                }],
            };
        };
        match message {
            ClientMessage::Hello(_) => unreachable!("handled above"),
            ClientMessage::Push(events) => self.push(session, events, now_ms),
            // Profile pictures are kept by the server's store, not by the hub.
            ClientMessage::GetAvatars(_) | ClientMessage::SetAvatar(_) => Vec::new(),
            ClientMessage::ListIncidents => vec![Outgoing {
                session,
                message: ServerMessage::Incidents(self.incident_list(&user)),
            }],
            ClientMessage::ListDevices => vec![Outgoing {
                session,
                message: ServerMessage::Devices(self.device_list(&user)),
            }],
            ClientMessage::RevokeDevice(device) => {
                let mut out = Vec::new();
                if !self.revoke(&user, device, &mut out) {
                    return vec![Outgoing {
                        session,
                        message: ServerMessage::Refused(Rejection::Forbidden),
                    }];
                }
                // The requester gets the new list, unless it revoked its own connection.
                if self.sessions.contains_key(&session) {
                    out.push(Outgoing {
                        session,
                        message: ServerMessage::Devices(self.device_list(&user)),
                    });
                }
                out
            }
        }
    }

    // [impl->dsn~compartment-scoped-sync~1]
    fn hello(&mut self, session: SessionId, hello: Hello, now_ms: u64) -> Vec<Outgoing> {
        let refuse = |reason| {
            vec![Outgoing {
                session,
                message: ServerMessage::Refused(reason),
            }]
        };
        // [impl->dsn~device-proof~1]
        // Every challenge is used once; a hello without one is refused.
        let Some(challenge) = self.challenges.remove(&session) else {
            return refuse(Rejection::NotAuthenticated);
        };
        // [impl->dsn~device-revocation~1]
        // A revoked device is told so (and deletes its data), whatever it proves.
        if self.revoked.contains(&hello.device) {
            return refuse(Rejection::Revoked);
        }
        let Some(device) = self
            .devices
            .get(&hello.device)
            .filter(|d| d.user == hello.user)
        else {
            return refuse(Rejection::UnknownDevice);
        };
        if ordo_crypto::verify_hello(
            &device.key,
            &challenge,
            &hello.device,
            &hello.user,
            &hello.proof,
        )
        .is_err()
        {
            return refuse(Rejection::NotAuthenticated);
        }
        // [impl->dsn~access-timeout~1]
        if device.expired(now_ms) {
            return refuse(Rejection::LoginExpired);
        }
        let valid_until = device.valid_until;
        let mut welcome = Welcome {
            granted: Vec::new(),
            denied: Vec::new(),
            protected: self.config.protected.clone(),
            keys: Vec::new(),
            permissions: [
                (
                    "manage_devices",
                    self.policy.can_manage_devices(&hello.user),
                ),
                (
                    "manage_incidents",
                    self.policy.can_manage_incidents(&hello.user),
                ),
            ]
            .into_iter()
            .filter(|(_, allowed)| *allowed)
            .map(|(name, _)| name.to_owned())
            .collect(),
            incident_permissions: Vec::new(),
            valid_until,
        };
        let mut deliveries = Vec::new();
        let mut streams = HashSet::new();
        let incidents: BTreeSet<IncidentId> = hello
            .subscriptions
            .iter()
            .map(|s| s.incident.clone())
            .collect();
        for incident in incidents {
            let roles = self.registry.roles(&hello.user, &incident);
            let permissions = self
                .policy
                .incident_permissions(&hello.user, &incident, &roles);
            welcome.incident_permissions.push(IncidentPermissions {
                incident,
                permissions,
            });
        }
        for sub in hello.subscriptions {
            let key = (sub.incident.clone(), sub.compartment.clone());
            let roles = self.registry.roles(&hello.user, &sub.incident);
            if !self
                .policy
                .can_read(&hello.user, &sub.incident, &sub.compartment, &roles)
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
        let mut roles_changed = HashSet::new();
        for event in events {
            match self.accept(&device, &user, event, now_ms) {
                Ok(Some((key, sequenced, body))) => {
                    if let Effect::Roles(changed) = self.registry.apply(&body) {
                        roles_changed.insert(changed);
                    }
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
        // [impl->dsn~incident-roles~1]
        // Whose roles changed reconnects, to receive what they may read now.
        if !roles_changed.is_empty() {
            let cut: Vec<SessionId> = self
                .sessions
                .iter()
                .filter(|(_, s)| roles_changed.contains(&s.user))
                .map(|(id, _)| *id)
                .collect();
            for sid in cut {
                self.sessions.remove(&sid);
                out.push(Outgoing {
                    session: sid,
                    message: ServerMessage::Refused(Rejection::RolesChanged),
                });
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
        let roles = self.registry.roles(user, &body.incident);
        let incident_event = body.module.as_str() == incidents::MODULE;
        let allowed = if incident_event {
            // [impl->dsn~incident-roles~1]
            // Planning and rosters (ADR-0046): only planners plan; planners and those whose
            // incident role allows it grant roles and change the state.
            let exists = self.registry.get(&body.incident).is_some();
            body.compartment.as_str() == incidents::COMPARTMENT
                && match body.event_type.as_str() {
                    incidents::PLANNED => !exists && self.policy.can_manage_incidents(user),
                    _ => {
                        exists
                            && (self.policy.can_manage_incidents(user)
                                || self.policy.can_assign_roles(user, &body.incident, &roles))
                    }
                }
        } else {
            self.policy.can_write(
                user,
                &body.incident,
                &body.compartment,
                body.module.as_str(),
                &roles,
            )
        };
        if !allowed {
            return Err(reject(Rejection::Forbidden));
        }
        let key = (body.incident.clone(), body.compartment.clone());
        if let Some(known) = self.streams.get(&key).and_then(|s| s.ids.get(&id)) {
            return if *known == ordo_crypto::event_hash(&event) {
                Ok(None) // the same event sent again
            } else {
                Err(reject(Rejection::DuplicateId))
            };
        }
        let payload = self
            .open_payload(&key, &body)
            .ok_or(reject(Rejection::WrongSealing))?;
        if incident_event && !incidents::is_valid(body.event_type.as_str(), &payload) {
            return Err(reject(Rejection::Malformed));
        }
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
    fn open_payload(&self, key: &StreamKey, body: &EventBody) -> Option<Vec<u8>> {
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
            stream
                .ids
                .insert(id, ordo_crypto::event_hash(&sequenced.event));
        }
        stream.events.push(sequenced.clone());
        self.changes.push(Change::Event {
            stream: key.clone(),
            event: sequenced.clone(),
        });
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
        self.changes.push(Change::Key {
            stream: key.clone(),
            key_id: id.clone(),
            key: data_key.clone(),
        });
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
