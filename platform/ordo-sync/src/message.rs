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
    /// Asks for the incidents the user takes part in (all with `manage_incidents`).
    ListIncidents,
    /// Asks for the people the server knows, to suggest them when granting a role.
    ListPeople,
    /// Asks for the profile pictures of these people (ADR-0047).
    GetAvatars(Vec<UserId>),
    /// Sets (or, without image, removes) the user's own profile picture in ORDO.
    SetAvatar(AvatarImage),
    /// Asks for a ticket to upload or download an attachment (ADR-0033).
    BlobTicket(BlobRequest),
}

/// A request for an attachment ticket. Attachments are encrypted on the device and named
/// by the BLAKE3 hash of their ciphertext; each belongs to a stream (incident and
/// compartment), whose need-to-know rules decide (ADR-0033).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobRequest {
    /// The incident.
    pub incident: IncidentId,
    /// The compartment of the event that refers to the attachment.
    pub compartment: CompartmentId,
    /// BLAKE3 hash of the ciphertext.
    #[serde(with = "fixed_bytes")]
    pub hash: [u8; 32],
    /// Size in bytes (for uploads; ignored for downloads).
    pub size: u64,
    /// Upload (`true`) or download.
    pub upload: bool,
}

/// The answer to a [`BlobRequest`]: a ticket for `PUT`/`GET /blobs/<hash>` (header
/// `Authorization: Bearer <ticket>`), or none if not allowed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlobTicket {
    /// The attachment.
    #[serde(with = "fixed_bytes")]
    pub hash: [u8; 32],
    /// Upload or download.
    pub upload: bool,
    /// The ticket, valid for a few minutes; `None`: not allowed (or too large).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ticket: Option<String>,
}

/// An uploaded profile picture: JPEG, PNG or WebP, at most [`MAX_AVATAR_BYTES`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvatarImage {
    /// The image; `None` removes the picture uploaded in ORDO.
    #[serde(with = "serde_bytes", default, skip_serializing_if = "Option::is_none")]
    pub image: Option<Vec<u8>>,
}

/// Largest profile picture in bytes.
pub const MAX_AVATAR_BYTES: usize = 128 * 1024;

/// The profile picture of a person, resolved by the server: uploaded in ORDO, else from
/// the identity provider, else from Gravatar (ADR-0047).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Avatar {
    /// The person.
    pub user: UserId,
    /// The image, if there is one.
    #[serde(with = "serde_bytes", default, skip_serializing_if = "Option::is_none")]
    pub image: Option<Vec<u8>>,
}

/// Whether `bytes` is a JPEG, PNG or WebP image of acceptable size.
pub fn is_avatar_image(bytes: &[u8]) -> bool {
    bytes.len() <= MAX_AVATAR_BYTES
        && (bytes.starts_with(&[0xff, 0xd8, 0xff])
            || bytes.starts_with(b"\x89PNG\r\n\x1a\n")
            || (bytes.len() > 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP"))
}

/// The app's views and functions a user may use in one incident.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentPermissions {
    /// The incident.
    pub incident: IncidentId,
    /// Permission names, e.g. `view_situation`, `edit_units`.
    pub permissions: Vec<String>,
}

/// An incident as listed for a person (ADR-0046).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncidentSummary {
    /// The incident.
    pub incident: IncidentId,
    /// Its name.
    pub name: String,
    /// Planned, active or closed.
    pub state: crate::incidents::IncidentState,
    /// The person's roles in it.
    pub roles: Vec<String>,
    /// Whether the person may grant roles and change the state.
    pub manage: bool,
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
    /// The protocol version the device speaks on this connection; none is `1.0`
    /// (ADR-0052).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<crate::version::Version>,
}

/// A random challenge the server sends right after the connection opens; the device
/// answers with a [`Hello`] that signs it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Challenge {
    /// 32 random bytes, used once.
    #[serde(with = "fixed_bytes")]
    pub nonce: [u8; 32],
    /// The protocol versions the server supports (ADR-0052).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub versions: Option<crate::version::VersionRange>,
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
    /// The incidents the user may open (answer to `listIncidents`).
    Incidents(Vec<IncidentSummary>),
    /// The people the server knows (answer to `listPeople`): empty for those who may not
    /// grant roles anywhere.
    People(Vec<UserId>),
    /// Profile pictures (answer to `getAvatars` and `setAvatar`).
    Avatars(Vec<Avatar>),
    /// Answer to `blobTicket`.
    BlobTicket(BlobTicket),
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
    /// Per subscribed incident: the app's views and functions the user's roles allow there
    /// (e.g. `view_situation`, `edit_units`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incident_permissions: Vec<IncidentPermissions>,
    /// Until when (ms since the Unix epoch) the login of this device is valid; the device
    /// locks itself afterwards, also offline, until the person logs in again (ADR-0040).
    /// `None`: no limit (devices paired by hand for development).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<u64>,
    /// The protocol version agreed for this connection (ADR-0052).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<crate::version::Version>,
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
    /// The person's roles changed: the device reconnects to get what it may read now
    /// (ADR-0046). Not an error.
    RolesChanged,
    /// The hello named a protocol version outside the range the server supports
    /// (ADR-0052); the challenge named the range.
    UnsupportedVersion,
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
    fn avatar_images_are_checked() {
        assert!(is_avatar_image(&[0xff, 0xd8, 0xff, 0xe0]));
        assert!(is_avatar_image(b"\x89PNG\r\n\x1a\nrest"));
        assert!(is_avatar_image(b"RIFF\0\0\0\0WEBPVP8 "));
        assert!(!is_avatar_image(b"<svg xmlns=..."), "no SVG (scripts)");
        assert!(!is_avatar_image(&vec![0xff; MAX_AVATAR_BYTES + 1]));
        let set = ClientMessage::SetAvatar(AvatarImage {
            image: Some(vec![0xff, 0xd8, 0xff]),
        });
        assert_eq!(
            decode::<ClientMessage>(&encode(&set).unwrap()).unwrap(),
            set
        );
    }

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
