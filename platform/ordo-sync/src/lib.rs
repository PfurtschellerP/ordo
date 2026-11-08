//! ORDO sync (architecture chapters 8.1–8.3, spec/sync).
//!
//! Pure logic without I/O: the [`Hub`] is the server side that orders, stores and
//! distributes events; a [`Replica`] is a device's local copy with its outbox. Both
//! exchange [`message`]s, which a transport (WebSocket, or a simulated network in tests)
//! carries as CBOR. Time and randomness are passed in, which keeps everything
//! deterministic and testable.

mod applied;
mod error;
mod hub;
pub mod message;
pub mod policy;
mod replica;

pub use applied::{AppliedEvent, Derived, Deriver, Entropy, StreamKey};
pub use error::SyncError;
pub use hub::{Hub, HubConfig, Outgoing, SessionId};
pub use policy::{AccessPolicy, StaticPolicy};
pub use replica::Replica;
