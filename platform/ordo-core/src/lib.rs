//! ORDO platform core.
//!
//! This crate defines what every other part of ORDO builds on: identifiers, the hybrid
//! logical clock and the event envelope, as specified in `spec/sync/event.md`.
//!
//! It does no I/O and has no knowledge of fire, medical or police specifics, so it
//! compiles both natively (server) and to WebAssembly (app).

pub mod attachment;
pub mod codec;
pub mod event;
pub mod hlc;
pub mod ids;
pub mod ulid;

pub use attachment::AttachmentRef;
pub use codec::CodecError;
pub use event::{Author, EventBody, Payload, SealedBox, SequencedEvent, Signature, SignedEvent};
pub use hlc::{Hlc, HlcClock, HlcError};
pub use ids::{
    CompartmentId, DeviceId, EventId, EventType, GUEST_PREFIX, IdError, IncidentId, KeyId,
    ModuleId, ServerName, UserId,
};
pub use ulid::{Ulid, UlidError};
