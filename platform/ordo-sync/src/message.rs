//! Protocol messages between devices and the server (spec/sync/protocol.cddl).
//!
//! Messages are encoded as CBOR; enums use the externally tagged form `{"name": …}`.

use ordo_core::codec::{self, CodecError, fixed_bytes};
use ordo_core::{
    CompartmentId, DeviceId, EventId, IncidentId, KeyId, SequencedEvent, SignedEvent, UserId,
};
use serde::{Deserialize, Serialize};

/// From a device to the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientMessage {
    /// First message after connecting.
    Hello(Hello),
    /// Events from the device's outbox.
    Push(Vec<SignedEvent>),
    /// Asks for the paired devices: one's own, or all with permission `manage_devices`.
    ListDevices,
    /// Revokes a device (one's own, or any with permission `manage_devices`).
    RevokeDevice(DeviceId),
}

/// A paired device, as listed for its person or an admin (SEC-4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// The device.
    pub device: DeviceId,
    /// Its person.
    pub user: UserId,
    /// When it was paired (ms), if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paired: Option<u64>,
    /// End of its login (ms), if limited.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<u64>,
}

/// Identifies the device, proves its key and asks for streams.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    /// The user working on the device.
    pub user: UserId,
    /// The device.
    pub device: DeviceId,
    /// Streams the device wants, with the last sequence number it already has.
    pub subscriptions: Vec<Subscription>,
    /// Ed25519 signature of the server's challenge with the device key
    /// (`ordo_crypto::hello_message`).
    #[serde(with = "fixed_bytes")]
    pub proof: [u8; 64],
}

/// A random challenge the server sends right after the connection opens; the device
/// answers with a [`Hello`] that signs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Challenge {
    /// 32 random bytes, used once.
    #[serde(with = "fixed_bytes")]
    pub nonce: [u8; 32],
}

/// A stream the device wants, and how far it already is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subscription {
    /// Incident.
    pub incident: IncidentId,
    /// Compartment.
    pub compartment: CompartmentId,
    /// Last sequence number the device has (0 if none).
    pub after: u64,
}

/// From the server to a device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServerMessage {
    /// First message on every connection.
    Challenge(Challenge),
    /// Answer to [`Hello`].
    Welcome(Welcome),
    /// The server refused the connection.
    Refused(Rejection),
    /// Newly ordered events of one stream.
    Deliver(Delivery),
    /// An event from the device was not accepted.
    Rejected(Rejected),
    /// The paired devices the user may see (answer to `listDevices` and `revokeDevice`).
    Devices(Vec<DeviceInfo>),
}

/// Which streams the device gets, and the keys it needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Welcome {
    /// Streams the device will receive.
    pub granted: Vec<Subscription>,
    /// Streams the user may not read. The device receives nothing of them.
    pub denied: Vec<StreamRef>,
    /// Compartments whose payloads must be sealed.
    pub protected: Vec<CompartmentId>,
    /// Data keys for protected compartments the user may read.
    pub keys: Vec<KeyGrant>,
    /// What the user may do beyond reading and writing compartments, e.g.
    /// `manage_devices`; the app shows or hides functions accordingly (ADR-0035).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<String>,
    /// Until when (ms since the Unix epoch) the login of this device is valid; the device
    /// locks itself afterwards, also offline, until the person logs in again (ADR-0040).
    /// `None`: no limit (devices paired by hand for development).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<u64>,
}

/// An incident compartment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamRef {
    /// Incident.
    pub incident: IncidentId,
    /// Compartment.
    pub compartment: CompartmentId,
}

/// A compartment data key for one device.
///
/// Prototype: the key travels inside the (TLS-protected) session. Later it is
/// encrypted for the device's own key (X25519), see architecture chapter 8.2.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyGrant {
    /// Incident.
    pub incident: IncidentId,
    /// Compartment.
    pub compartment: CompartmentId,
    /// Name of the key.
    pub key_id: KeyId,
    /// The key.
    #[serde(with = "fixed_bytes")]
    pub key: [u8; 32],
}

impl core::fmt::Debug for KeyGrant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KeyGrant")
            .field("incident", &self.incident)
            .field("compartment", &self.compartment)
            .field("key_id", &self.key_id)
            .finish_non_exhaustive()
    }
}

/// Newly ordered events of one stream, in sequence order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Delivery {
    /// Incident.
    pub incident: IncidentId,
    /// Compartment.
    pub compartment: CompartmentId,
    /// The events.
    pub events: Vec<SequencedEvent>,
}

/// An event that was not accepted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rejected {
    /// The event.
    pub event: EventId,
    /// Why.
    pub reason: Rejection,
}

/// Why the server did not accept something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Rejection {
    /// The event body could not be decoded.
    Malformed,
    /// The device is not registered for this user.
    UnknownDevice,
    /// The hello did not prove the device key (missing challenge or wrong proof).
    NotAuthenticated,
    /// The login of this device has expired or was ended; the person must log in again.
    LoginExpired,
    /// The device was revoked (lost, stolen, retired); it deletes its local data.
    Revoked,
    /// Author in the event doesn't match the connected user and device.
    WrongAuthor,
    /// The signature is not valid.
    BadSignature,
    /// The user may not write to this compartment.
    Forbidden,
    /// The compartment is protected and the payload is not sealed with its current key,
    /// or it is unprotected and the payload is sealed.
    WrongSealing,
    /// The event's clock is too far ahead of the server's.
    ClockAhead,
    /// The client sent a push before hello.
    NotConnected,
    /// An event with this ID but different content was already ordered.
    DuplicateId,
}

/// Encodes a message as CBOR.
pub fn encode<T: Serialize>(message: &T) -> Result<Vec<u8>, CodecError> {
    codec::to_cbor(message)
}

/// Decodes a message from CBOR.
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    codec::from_cbor(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ordo_core::Ulid;

    // [utest->dsn~device-revocation~1]
    #[test]
    fn device_messages_match_the_schema() {
        // A unit variant is a plain text string in CBOR: "listDevices" (11 bytes).
        let mut expected = vec![0x6b];
        expected.extend_from_slice(b"listDevices");
        assert_eq!(encode(&ClientMessage::ListDevices).unwrap(), expected);

        let device = DeviceId(Ulid::from_parts(1, 2).unwrap());
        let revoke = ClientMessage::RevokeDevice(device);
        assert_eq!(
            decode::<ClientMessage>(&encode(&revoke).unwrap()).unwrap(),
            revoke
        );
        let list = ServerMessage::Devices(vec![DeviceInfo {
            device,
            user: "anna@ordo.example.org".parse().unwrap(),
            paired: Some(1),
            valid_until: None,
        }]);
        assert_eq!(
            decode::<ServerMessage>(&encode(&list).unwrap()).unwrap(),
            list
        );
        let revoked = ServerMessage::Refused(Rejection::Revoked);
        assert_eq!(
            decode::<ServerMessage>(&encode(&revoked).unwrap()).unwrap(),
            revoked
        );
    }
}
