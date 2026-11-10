use ordo_core::{CompartmentId, EventBody, EventType, IncidentId, ModuleId};

/// An incident compartment: the unit of ordering, access and encryption.
pub type StreamKey = (IncidentId, CompartmentId);

/// An event as modules see it: decoded body and decrypted payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppliedEvent {
    /// Sequence number, or `None` while the event is still in the device's outbox.
    pub seq: Option<u64>,
    /// The decoded body.
    pub body: EventBody,
    /// The plaintext payload (CBOR, defined by the module).
    pub payload: Vec<u8>,
}

/// An event a [`Deriver`] wants to publish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Derived {
    /// Target compartment (normally `incident`).
    pub compartment: CompartmentId,
    /// Module that defines the contract.
    pub module: ModuleId,
    /// Event type of the contract.
    pub event_type: EventType,
    /// Schema version of the payload.
    pub schema_version: u32,
    /// CBOR-encoded payload.
    pub payload: Vec<u8>,
}

// [impl->dsn~uhs-staff~1]
/// Checks events a package's rules allow beyond the role policy (ADR-0056), e.g. that only
/// a UHS's staff writes its patients. The hub shows a guard every ordered event of its
/// compartments, also when it restores them, so the guard keeps the state it judges by.
pub trait WriteGuard: Send {
    /// Whether the guard reads events of `compartment`.
    fn observes(&self, compartment: &CompartmentId) -> bool;

    /// Whether `event` (decrypted, not yet ordered) may be accepted from its author, who
    /// has `permissions` in the incident (as in the welcome, e.g. `all_uhs`).
    fn allows(&self, event: &AppliedEvent, permissions: &[String]) -> bool;

    /// Learns from an ordered event of an observed compartment.
    fn observe(&mut self, event: &AppliedEvent);
}

// [impl->dsn~server-derived-connectors~1]
/// Derives connector events on the server that orders the source compartment
/// (ADR-0024). Called once for every newly ordered event of [`Deriver::source`].
pub trait Deriver: Send {
    /// The compartment whose events this deriver reads.
    fn source(&self) -> &CompartmentId;

    /// Returns the connector events to publish after `event`.
    fn on_event(&mut self, event: &AppliedEvent) -> Vec<Derived>;
}

/// Source of randomness (nonces, IDs, keys). Native code uses the operating system;
/// tests use a deterministic source.
pub trait Entropy: Send {
    /// Fills `buffer` with random bytes.
    fn fill(&mut self, buffer: &mut [u8]);
}

impl<F: FnMut(&mut [u8]) + Send> Entropy for F {
    fn fill(&mut self, buffer: &mut [u8]) {
        self(buffer)
    }
}

pub(crate) fn random_u128(entropy: &mut dyn Entropy) -> u128 {
    let mut bytes = [0u8; 16];
    entropy.fill(&mut bytes);
    u128::from_be_bytes(bytes)
}
