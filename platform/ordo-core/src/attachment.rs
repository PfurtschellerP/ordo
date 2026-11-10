//! References to attachments (ADR-0033): photos and other large content travel outside
//! the events, encrypted, named by the hash of their ciphertext.

use serde::{Deserialize, Serialize};

use crate::codec::fixed_bytes;

/// An encrypted attachment, as an event payload refers to it. The key opens the
/// attachment; it travels only inside the (sealed) event, never to the object store.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentRef {
    /// BLAKE3 hash of the ciphertext; the attachment's name on the server.
    #[serde(with = "fixed_bytes")]
    pub hash: [u8; 32],
    /// Size of the ciphertext in bytes.
    pub size: u64,
    /// Media type of the plaintext, e.g. `image/jpeg`.
    pub media_type: String,
    /// The attachment's own key (XChaCha20-Poly1305).
    #[serde(with = "fixed_bytes")]
    pub key: [u8; 32],
}

impl core::fmt::Debug for AttachmentRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AttachmentRef")
            .field("hash", &self.hash)
            .field("size", &self.size)
            .field("media_type", &self.media_type)
            .finish_non_exhaustive()
    }
}
