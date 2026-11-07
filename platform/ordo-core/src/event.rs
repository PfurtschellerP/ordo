//! The event envelope (spec/sync/event.md, spec/sync/event.cddl).
//!
//! An event travels as a [`SignedEvent`]: the CBOR-encoded [`EventBody`] as raw bytes,
//! plus a [`Signature`] over exactly those bytes. Keeping the body as bytes means a
//! server can store and forward events it doesn't fully understand — newer envelope
//! fields, modules it doesn't run — without breaking their signatures.
//!
//! Signing and verification live in `ordo-crypto`; this module only defines the
//! structures and their encoding.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::codec::{self, CodecError, fixed_bytes};
use crate::hlc::Hlc;
use crate::ids::{
    CompartmentId, DeviceId, EventId, EventType, IncidentId, KeyId, ModuleId, ServerName, UserId,
};

/// The envelope format version defined by this crate.
pub const ENVELOPE_VERSION: u8 = 1;

/// Maximum size of an encoded signed event (1 MiB).
pub const MAX_SIGNED_EVENT_LEN: usize = 1024 * 1024;

/// The content of an event.
///
/// Fields are declared in deterministic CBOR key order (RFC 8949 §4.2.1: shorter keys
/// first, then bytewise), so that encoding the same body always gives the same bytes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventBody {
    /// Envelope format version; see [`ENVELOPE_VERSION`].
    #[serde(rename = "v")]
    pub version: u8,
    /// Globally unique event ID.
    pub id: EventId,
    /// Schema version of the event type's payload.
    #[serde(rename = "sv")]
    pub schema_version: u32,
    /// Hybrid logical clock timestamp.
    pub hlc: Hlc,
    /// Incident the event belongs to.
    #[serde(rename = "inc")]
    pub incident: IncidentId,
    /// Module that defines the event type.
    #[serde(rename = "mod")]
    pub module: ModuleId,
    /// Data compartment the event belongs to.
    #[serde(rename = "comp")]
    pub compartment: CompartmentId,
    /// Namespaced event type.
    #[serde(rename = "type")]
    pub event_type: EventType,
    /// Who created the event.
    pub author: Author,
    /// Server on which the event was created.
    pub origin: ServerName,
    /// Module-specific content.
    pub payload: Payload,
}

impl EventBody {
    /// Encodes the body as CBOR.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        codec::to_cbor(self)
    }

    /// Decodes a body. Unknown keys are ignored; an unsupported envelope version is an
    /// error.
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        #[derive(Deserialize)]
        struct VersionProbe {
            v: u8,
        }
        let probe: VersionProbe = codec::from_cbor(bytes)?;
        if probe.v != ENVELOPE_VERSION {
            return Err(CodecError::UnsupportedVersion(probe.v));
        }
        codec::from_cbor(bytes)
    }
}

/// The user and device that created an event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Author {
    /// The user, qualified with their server.
    pub user: UserId,
    /// The device the user worked on.
    pub device: DeviceId,
}

/// Module-specific content of an event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Payload {
    /// Unencrypted, CBOR-encoded payload.
    Plain(#[serde(with = "serde_bytes")] Vec<u8>),
    /// Payload encrypted with the compartment's data key.
    Sealed(SealedBox),
}

/// An encrypted payload (XChaCha20-Poly1305).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SealedBox {
    /// Ciphertext including the authentication tag.
    #[serde(with = "serde_bytes")]
    pub ct: Vec<u8>,
    /// The compartment data key that was used.
    pub key: KeyId,
    /// Random 24-byte nonce.
    #[serde(with = "fixed_bytes")]
    pub nonce: [u8; 24],
}

/// A signature over the body bytes of an event.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    /// The key that made the signature.
    pub kid: KeyId,
    /// The Ed25519 signature value.
    #[serde(with = "fixed_bytes")]
    pub val: [u8; 64],
}

/// An event as exchanged between devices and servers: `[body bytes, signature]`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SignedEvent {
    /// The CBOR-encoded [`EventBody`], kept byte-for-byte as received.
    pub body: Vec<u8>,
    /// Signature over `body`.
    pub signature: Signature,
}

impl SignedEvent {
    /// Decodes the body. This does **not** check the signature; use `ordo-crypto`.
    pub fn decode_body(&self) -> Result<EventBody, CodecError> {
        EventBody::decode(&self.body)
    }

    /// Encodes the signed event as CBOR.
    pub fn encode(&self) -> Result<Vec<u8>, CodecError> {
        let bytes = codec::to_cbor(self)?;
        check_len(bytes.len())?;
        Ok(bytes)
    }

    /// Decodes a signed event, enforcing [`MAX_SIGNED_EVENT_LEN`].
    pub fn decode(bytes: &[u8]) -> Result<Self, CodecError> {
        check_len(bytes.len())?;
        codec::from_cbor(bytes)
    }
}

fn check_len(len: usize) -> Result<(), CodecError> {
    if len > MAX_SIGNED_EVENT_LEN {
        return Err(CodecError::TooLarge {
            len,
            max: MAX_SIGNED_EVENT_LEN,
        });
    }
    Ok(())
}

impl Serialize for SignedEvent {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Repr<'a>(#[serde(with = "serde_bytes")] &'a [u8], &'a Signature);
        Repr(&self.body, &self.signature).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for SignedEvent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Repr(#[serde(with = "serde_bytes")] Vec<u8>, Signature);
        let Repr(body, signature) = Repr::deserialize(deserializer)?;
        Ok(Self { body, signature })
    }
}

/// An event after the incident's home server has put it in order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequencedEvent {
    /// Position within the incident and compartment, starting at 1.
    pub seq: u64,
    /// Hash chain value up to and including this event.
    #[serde(with = "fixed_bytes")]
    pub chain: [u8; 32],
    /// The event itself.
    pub event: SignedEvent,
}
