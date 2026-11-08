use ordo_core::{CodecError, CompartmentId, IncidentId};

/// A replica could not complete an operation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SyncError {
    /// The device has not received its first welcome from the server yet.
    #[error("the device has not been connected to the server yet")]
    NotReady,
    /// The device is not subscribed to the stream, or the server denied it.
    #[error("no access to {compartment} of {incident}")]
    NoAccess {
        /// Incident.
        incident: IncidentId,
        /// Compartment.
        compartment: CompartmentId,
    },
    /// The compartment is protected and the device has no key for it.
    #[error("no key for {compartment} of {incident}")]
    NoKey {
        /// Incident.
        incident: IncidentId,
        /// Compartment.
        compartment: CompartmentId,
    },
    /// Received events don't continue the local sequence.
    #[error("expected sequence number {expected}, received {received}")]
    Gap {
        /// Next expected sequence number.
        expected: u64,
        /// Received sequence number.
        received: u64,
    },
    /// A received chain value doesn't match: an event was changed, removed or reordered.
    #[error("hash chain broken at sequence number {seq}")]
    BrokenChain {
        /// Sequence number of the first bad event.
        seq: u64,
    },
    /// The server refused the connection.
    #[error("the server refused the connection: {0:?}")]
    Refused(crate::message::Rejection),
    /// Encoding, decoding or decryption failed.
    #[error("invalid event: {0}")]
    Invalid(String),
}

impl From<CodecError> for SyncError {
    fn from(error: CodecError) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl From<ordo_crypto::CryptoError> for SyncError {
    fn from(error: ordo_crypto::CryptoError) -> Self {
        Self::Invalid(error.to_string())
    }
}
