//! Encrypted attachments (ADR-0033). Each attachment has its own random key; the
//! ciphertext is nonce ‖ XChaCha20-Poly1305 ciphertext, bound to the media type, and named
//! by its BLAKE3 hash. The object store only ever sees the ciphertext.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use ordo_core::AttachmentRef;

use crate::CryptoError;

/// Prefix of the additional authenticated data of attachments.
const ATTACHMENT_CONTEXT: &[u8] = b"ORDO attachment v1\0";

/// Bytes the encryption adds: nonce (24) and tag (16).
pub const OVERHEAD: u64 = 40;

fn aad(media_type: &str) -> Vec<u8> {
    [ATTACHMENT_CONTEXT, media_type.as_bytes()].concat()
}

// [impl->dsn~attachments~1]
/// Encrypts `plaintext` with a fresh `key` (random, used once) and returns the reference
/// for the event and the ciphertext for the object store.
pub fn seal(
    key: [u8; 32],
    nonce: [u8; 24],
    media_type: &str,
    plaintext: &[u8],
) -> (AttachmentRef, Vec<u8>) {
    let cipher = XChaCha20Poly1305::new(&key.into());
    let ct = cipher
        .encrypt(
            &XNonce::from(nonce),
            Payload {
                msg: plaintext,
                aad: &aad(media_type),
            },
        )
        .expect("XChaCha20-Poly1305 encryption cannot fail for in-memory buffers");
    let data = [nonce.as_slice(), &ct].concat();
    let reference = AttachmentRef {
        hash: hash(&data),
        size: data.len() as u64,
        media_type: media_type.to_owned(),
        key,
    };
    (reference, data)
}

/// The name of an attachment: the BLAKE3 hash of its ciphertext.
pub fn hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Whether `data` is the attachment `reference` names (size and hash).
pub fn matches(reference: &AttachmentRef, data: &[u8]) -> bool {
    data.len() as u64 == reference.size && hash(data) == reference.hash
}

/// Decrypts an attachment after checking that it is the one the reference names.
pub fn open(reference: &AttachmentRef, data: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if !matches(reference, data) || data.len() < 24 {
        return Err(CryptoError::CannotOpen);
    }
    let (nonce, ct) = data.split_at(24);
    let nonce: [u8; 24] = nonce.try_into().map_err(|_| CryptoError::CannotOpen)?;
    XChaCha20Poly1305::new(&reference.key.into())
        .decrypt(
            &XNonce::from(nonce),
            Payload {
                msg: ct,
                aad: &aad(&reference.media_type),
            },
        )
        .map_err(|_| CryptoError::CannotOpen)
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~attachments~1]
    #[test]
    fn attachments_open_only_unchanged_and_with_their_reference() {
        let (reference, data) = seal([7; 32], [9; 24], "image/jpeg", b"photo bytes");
        assert_eq!(reference.size, b"photo bytes".len() as u64 + OVERHEAD);
        assert_eq!(data.len() as u64, reference.size);
        assert!(!data.windows(5).any(|w| w == b"photo"), "ciphertext only");
        assert_eq!(open(&reference, &data).as_deref(), Ok(&b"photo bytes"[..]));

        let mut changed = data.clone();
        changed[30] ^= 1;
        assert_eq!(open(&reference, &changed), Err(CryptoError::CannotOpen));
        let other_type = AttachmentRef {
            media_type: "image/webp".into(),
            ..reference.clone()
        };
        assert_eq!(open(&other_type, &data), Err(CryptoError::CannotOpen));
        let other_key = AttachmentRef {
            key: [8; 32],
            ..reference.clone()
        };
        assert_eq!(open(&other_key, &data), Err(CryptoError::CannotOpen));
        assert!(!matches(&reference, &data[1..]));
    }
}
