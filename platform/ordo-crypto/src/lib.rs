//! ORDO cryptography: event signatures and the hash chain
//! (spec/sync/event.md §4 and §6).
//!
//! Payload encryption for protected compartments will be added here as well.

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use ordo_core::{CodecError, EventBody, KeyId, SequencedEvent, Signature, SignedEvent};

pub use ed25519_dalek;

/// Prefix of every signed message: `ORDO event v1` followed by a zero byte.
pub const SIGNING_CONTEXT: &[u8] = b"ORDO event v1\0";

/// BLAKE3 key-derivation context for event hashes.
pub const EVENT_HASH_CONTEXT: &str = "ORDO event hash v1";

/// BLAKE3 key-derivation context for the hash chain.
pub const CHAIN_CONTEXT: &str = "ORDO chain v1";

/// Chain value before the first event of a sequence.
pub const CHAIN_GENESIS: [u8; 32] = [0; 32];

/// A cryptographic check failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CryptoError {
    /// Encoding the event failed.
    #[error(transparent)]
    Codec(#[from] CodecError),
    /// The signature does not match the event and key.
    #[error("the signature is not valid")]
    BadSignature,
}

/// Signs events with one Ed25519 key.
// [impl->dsn~signed-events~1]
pub struct EventSigner {
    key: SigningKey,
    kid: KeyId,
}

impl EventSigner {
    /// A signer using `key`, announced to others as `kid`.
    pub fn new(key: SigningKey, kid: KeyId) -> Self {
        Self { key, kid }
    }

    /// The public key others need to verify this signer's events.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.key.verifying_key()
    }

    /// Encodes and signs an event body.
    pub fn sign(&self, body: &EventBody) -> Result<SignedEvent, CryptoError> {
        Ok(self.sign_bytes(body.encode()?))
    }

    /// Signs already encoded body bytes.
    pub fn sign_bytes(&self, body: Vec<u8>) -> SignedEvent {
        let signature = self.key.sign(&signing_message(&body));
        SignedEvent {
            body,
            signature: Signature {
                kid: self.kid.clone(),
                val: signature.to_bytes(),
            },
        }
    }
}

/// Verifies the signature of an event over its body bytes as received.
// [impl->dsn~signed-events~1]
pub fn verify(event: &SignedEvent, key: &VerifyingKey) -> Result<(), CryptoError> {
    let signature = ed25519_dalek::Signature::from_bytes(&event.signature.val);
    key.verify_strict(&signing_message(&event.body), &signature)
        .map_err(|_| CryptoError::BadSignature)
}

fn signing_message(body: &[u8]) -> Vec<u8> {
    let mut message = Vec::with_capacity(SIGNING_CONTEXT.len() + body.len());
    message.extend_from_slice(SIGNING_CONTEXT);
    message.extend_from_slice(body);
    message
}

/// The hash of an event, computed over its body bytes.
pub fn event_hash(event: &SignedEvent) -> [u8; 32] {
    blake3::derive_key(EVENT_HASH_CONTEXT, &event.body)
}

/// The next chain value after `previous` for an event with hash `event_hash`.
pub fn chain_next(previous: &[u8; 32], event_hash: &[u8; 32]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new_derive_key(CHAIN_CONTEXT);
    hasher.update(previous);
    hasher.update(event_hash);
    *hasher.finalize().as_bytes()
}

/// The ordered event sequence of one incident compartment, as kept by its home server.
#[derive(Debug, Clone)]
// [impl->dsn~hash-chain~1]
pub struct Chain {
    last_seq: u64,
    head: [u8; 32],
}

impl Default for Chain {
    fn default() -> Self {
        Self::new()
    }
}

impl Chain {
    /// An empty chain.
    pub fn new() -> Self {
        Self {
            last_seq: 0,
            head: CHAIN_GENESIS,
        }
    }

    /// The sequence number of the last appended event (0 if empty).
    pub fn last_seq(&self) -> u64 {
        self.last_seq
    }

    /// The current chain value.
    pub fn head(&self) -> [u8; 32] {
        self.head
    }

    /// Appends an event and returns it with its sequence number and chain value.
    pub fn append(&mut self, event: SignedEvent) -> SequencedEvent {
        self.last_seq += 1;
        self.head = chain_next(&self.head, &event_hash(&event));
        SequencedEvent {
            seq: self.last_seq,
            chain: self.head,
            event,
        }
    }
}

/// A sequence of events does not form an unbroken chain.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ChainError {
    /// A sequence number is missing or out of order.
    #[error("expected sequence number {expected}, found {found}")]
    Gap {
        /// The sequence number that should have come next.
        expected: u64,
        /// The sequence number that was found.
        found: u64,
    },
    /// The chain value doesn't match: an event was changed, removed or reordered.
    #[error("chain value of event {seq} does not match")]
    Mismatch {
        /// Sequence number of the first event whose chain value is wrong.
        seq: u64,
    },
}

/// Checks that `events` form an unbroken chain starting at sequence number 1, and
/// returns the resulting chain.
// [impl->dsn~hash-chain~1]
pub fn verify_chain<'a>(
    events: impl IntoIterator<Item = &'a SequencedEvent>,
) -> Result<Chain, ChainError> {
    let mut chain = Chain::new();
    for sequenced in events {
        let expected = chain.last_seq + 1;
        if sequenced.seq != expected {
            return Err(ChainError::Gap {
                expected,
                found: sequenced.seq,
            });
        }
        let head = chain_next(&chain.head, &event_hash(&sequenced.event));
        if head != sequenced.chain {
            return Err(ChainError::Mismatch { seq: sequenced.seq });
        }
        chain.last_seq = expected;
        chain.head = head;
    }
    Ok(chain)
}
