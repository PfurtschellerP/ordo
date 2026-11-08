//! Sealed payloads for protected compartments (spec/sync/event.md §5).
//!
//! Payloads are encrypted with XChaCha20-Poly1305 using the data key of the incident
//! compartment. The additional authenticated data binds the ciphertext to its event:
//! 16 event-ID bytes, the UTF-8 incident ID, a zero byte and the UTF-8 compartment ID.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use ordo_core::{CompartmentId, EventId, IncidentId, KeyId, SealedBox};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::CryptoError;

/// A 256-bit data key of one incident compartment. Wiped from memory when dropped.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct DataKey([u8; 32]);

impl DataKey {
    /// A key from 32 random bytes. The caller provides the randomness.
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// The raw key bytes (for key distribution).
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl core::fmt::Debug for DataKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("DataKey(…)")
    }
}

/// What a sealed payload is bound to.
#[derive(Debug, Clone, Copy)]
pub struct SealContext<'a> {
    /// The event the payload belongs to.
    pub event: EventId,
    /// The incident of the event.
    pub incident: &'a IncidentId,
    /// The compartment of the event.
    pub compartment: &'a CompartmentId,
}

impl SealContext<'_> {
    fn aad(&self) -> Vec<u8> {
        let mut aad = Vec::with_capacity(16 + self.incident.as_str().len() + 1 + 32);
        aad.extend_from_slice(&self.event.ulid().to_bytes());
        aad.extend_from_slice(self.incident.as_str().as_bytes());
        aad.push(0);
        aad.extend_from_slice(self.compartment.as_str().as_bytes());
        aad
    }
}

// [impl->dsn~sealed-payloads~1]
/// Encrypts `plaintext` for the event described by `context`. `nonce` must be random
/// and never reused with the same key.
pub fn seal(
    key: &DataKey,
    key_id: KeyId,
    nonce: [u8; 24],
    context: SealContext<'_>,
    plaintext: &[u8],
) -> SealedBox {
    let cipher = XChaCha20Poly1305::new(&key.0.into());
    let ct = cipher
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad: &context.aad(),
            },
        )
        .expect("XChaCha20-Poly1305 encryption cannot fail for in-memory buffers");
    SealedBox {
        ct,
        key: key_id,
        nonce,
    }
}

/// Decrypts a sealed payload. Fails if the key is wrong, the ciphertext was changed, or
/// the payload is moved to another event, incident or compartment.
pub fn open(
    key: &DataKey,
    sealed: &SealedBox,
    context: SealContext<'_>,
) -> Result<Vec<u8>, CryptoError> {
    let cipher = XChaCha20Poly1305::new(&key.0.into());
    cipher
        .decrypt(
            &XNonce::from(sealed.nonce),
            Payload {
                msg: &sealed.ct,
                aad: &context.aad(),
            },
        )
        .map_err(|_| CryptoError::CannotOpen)
}
