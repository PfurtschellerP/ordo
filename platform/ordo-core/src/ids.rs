//! Identifiers (spec/sync/event.md §2.2).
//!
//! Text identifiers are validated when they are created, so a value of one of these
//! types is always well-formed. Identifiers that leave a server (users, incidents, keys)
//! are qualified with the server name, which makes them globally unique and allows
//! federation later.

use core::fmt;
use core::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use crate::ulid::Ulid;

/// An identifier was not well-formed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {kind} {value:?}: {reason}")]
pub struct IdError {
    kind: &'static str,
    value: String,
    reason: &'static str,
}

impl IdError {
    /// Which kind of identifier was invalid, e.g. `"server name"`.
    pub fn kind(&self) -> &'static str {
        self.kind
    }

    /// Why the value was rejected.
    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

// ---------------------------------------------------------------------------
// Binary ULID identifiers
// ---------------------------------------------------------------------------

macro_rules! ulid_id {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub Ulid);

        impl $name {
            /// The underlying ULID.
            pub const fn ulid(self) -> Ulid {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({})"), self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_bytes(&self.0.to_bytes())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                crate::codec::fixed_bytes::deserialize::<D, 16>(deserializer)
                    .map(|bytes| Self(Ulid::from_bytes(bytes)))
            }
        }
    };
}

ulid_id! {
    /// Globally unique ID of an event (ULID, 16 bytes on the wire).
    EventId
}

ulid_id! {
    /// ID of a device (tablet, phone, laptop, server process), ULID.
    DeviceId
}

// ---------------------------------------------------------------------------
// Text identifiers
// ---------------------------------------------------------------------------

// [impl->dsn~qualified-identifiers~1]
macro_rules! text_id {
    ($(#[$meta:meta])* $name:ident, $kind:literal, $check:path) => {
        $(#[$meta])*
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// The identifier as text.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdError;

            fn try_from(value: String) -> Result<Self, IdError> {
                match $check(&value) {
                    Ok(()) => Ok(Self(value)),
                    Err(reason) => Err(IdError { kind: $kind, value, reason }),
                }
            }
        }

        impl FromStr for $name {
            type Err = IdError;

            fn from_str(s: &str) -> Result<Self, IdError> {
                Self::try_from(s.to_owned())
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> String {
                id.0
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, concat!(stringify!($name), "({:?})"), self.0)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let text = String::deserialize(deserializer)?;
                Self::try_from(text).map_err(de::Error::custom)
            }
        }
    };
}

text_id! {
    /// Name of an ORDO server: a lower-case DNS name, optionally with a port,
    /// e.g. `ordo.drk-musterstadt.de`.
    ServerName, "server name", check_server_name
}

text_id! {
    /// A user, qualified with their server: `anna@ordo.drk-musterstadt.de`.
    UserId, "user ID", check_user_id
}

text_id! {
    /// An incident, qualified with the server where it was created: `ULID@server`.
    IncidentId, "incident ID", check_incident_id
}

text_id! {
    /// A signing or data key held by a server: `name@server`.
    KeyId, "key ID", check_key_id
}

text_id! {
    /// A data compartment, e.g. `incident` or `med.clinical`.
    CompartmentId, "compartment ID", check_dotted_name
}

text_id! {
    /// A module, e.g. `units` or `med.uhs`.
    ModuleId, "module ID", check_dotted_name
}

text_id! {
    /// A namespaced event type, e.g. `uhs.patientAdmitted`.
    EventType, "event type", check_event_type
}

impl UserId {
    /// Builds a user ID from a local part and a server name.
    pub fn new(localpart: &str, server: &ServerName) -> Result<Self, IdError> {
        format!("{localpart}@{server}").parse()
    }

    /// The part before the `@`.
    pub fn localpart(&self) -> &str {
        split_qualified(&self.0).0
    }

    /// The server the user belongs to.
    pub fn server(&self) -> &str {
        split_qualified(&self.0).1
    }
}

impl IncidentId {
    /// Builds an incident ID from a ULID and the server where the incident is created.
    pub fn new(ulid: Ulid, server: &ServerName) -> Self {
        // Both parts are valid by construction.
        Self(format!("{ulid}@{server}"))
    }

    /// The ULID part.
    pub fn ulid(&self) -> Ulid {
        split_qualified(&self.0)
            .0
            .parse()
            .expect("incident IDs are validated on creation")
    }

    /// The server where the incident was created.
    pub fn server(&self) -> &str {
        split_qualified(&self.0).1
    }
}

impl KeyId {
    /// Builds a key ID from a key name and the server that holds the key.
    pub fn new(name: &str, server: &ServerName) -> Result<Self, IdError> {
        format!("{name}@{server}").parse()
    }

    /// The key name (part before the `@`).
    pub fn name(&self) -> &str {
        split_qualified(&self.0).0
    }

    /// The server that holds the key.
    pub fn server(&self) -> &str {
        split_qualified(&self.0).1
    }
}

/// Splits a validated `local@server` value. The local part never contains `@`.
fn split_qualified(value: &str) -> (&str, &str) {
    value.split_once('@').unwrap_or((value, ""))
}

// ---------------------------------------------------------------------------
// Validation rules (must match spec/sync/event.cddl)
// ---------------------------------------------------------------------------

type Check = Result<(), &'static str>;

fn check_server_name(value: &str) -> Check {
    let (host, port) = match value.rsplit_once(':') {
        Some((host, port)) => (host, Some(port)),
        None => (value, None),
    };
    if let Some(port) = port {
        if port.is_empty() || port.len() > 5 || !port.bytes().all(|b| b.is_ascii_digit()) {
            return Err("the port must have 1 to 5 digits");
        }
        match port.parse::<u32>() {
            Ok(1..=65535) => {}
            _ => return Err("the port must be between 1 and 65535"),
        }
    }
    if host.is_empty() || host.len() > 253 {
        return Err("the host name must have 1 to 253 characters");
    }
    for label in host.split('.') {
        if label.is_empty() || label.len() > 63 {
            return Err("each part of the host name must have 1 to 63 characters");
        }
        if !label
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("the host name may only contain a-z, 0-9, '-' and '.'");
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err("parts of the host name must not start or end with '-'");
        }
    }
    Ok(())
}

fn check_qualified(value: &str, check_local: fn(&str) -> Check) -> Check {
    let (local, server) = value
        .split_once('@')
        .ok_or("must have the form name@server")?;
    check_local(local)?;
    check_server_name(server)
}

fn check_user_id(value: &str) -> Check {
    check_qualified(value, |local| {
        let valid = (1..=64).contains(&local.len())
            && local.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'_' | b'-')
            });
        if valid {
            Ok(())
        } else {
            Err("the user name must have 1 to 64 characters from a-z, 0-9, '.', '_' and '-'")
        }
    })
}

fn check_incident_id(value: &str) -> Check {
    check_qualified(value, |local| {
        local
            .parse::<Ulid>()
            .map(|_| ())
            .map_err(|_| "the part before '@' must be a ULID")
    })
}

fn check_key_id(value: &str) -> Check {
    check_qualified(value, |local| {
        let valid = (1..=64).contains(&local.len())
            && local
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'));
        if valid {
            Ok(())
        } else {
            Err("the key name must have 1 to 64 characters from A-Z, a-z, 0-9, '_' and '-'")
        }
    })
}

fn check_segments(value: &str, min_segments: usize, allow_upper: bool) -> Check {
    let mut count = 0;
    for segment in value.split('.') {
        count += 1;
        let mut bytes = segment.bytes();
        match bytes.next() {
            Some(b'a'..=b'z') => {}
            _ => return Err("each part must start with a lower-case letter a-z"),
        }
        let valid_rest = bytes.all(|b| {
            b.is_ascii_lowercase() || b.is_ascii_digit() || (allow_upper && b.is_ascii_uppercase())
        });
        if !valid_rest {
            return Err(if allow_upper {
                "parts may only contain letters and digits"
            } else {
                "parts may only contain a-z and 0-9"
            });
        }
    }
    if count < min_segments {
        return Err("must have at least two parts separated by '.'");
    }
    Ok(())
}

fn check_dotted_name(value: &str) -> Check {
    check_segments(value, 1, false)
}

fn check_event_type(value: &str) -> Check {
    check_segments(value, 2, true)
}

// [utest->dsn~qualified-identifiers~1]
#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> ServerName {
        "ordo.drk-musterstadt.de".parse().unwrap()
    }

    #[test]
    fn server_names() {
        for ok in [
            "ordo.drk-musterstadt.de",
            "localhost",
            "localhost:8443",
            "a1.b2",
        ] {
            assert!(ok.parse::<ServerName>().is_ok(), "{ok} should be valid");
        }
        for bad in [
            "",
            "Ordo.example.org",
            "-ordo.example.org",
            "ordo..example.org",
            "ordo.example.org:",
            "ordo.example.org:0",
            "ordo.example.org:70000",
            "ordo_example.org",
        ] {
            assert!(
                bad.parse::<ServerName>().is_err(),
                "{bad} should be invalid"
            );
        }
    }

    #[test]
    fn user_ids() {
        let user = UserId::new("anna.muster", &server()).unwrap();
        assert_eq!(user.as_str(), "anna.muster@ordo.drk-musterstadt.de");
        assert_eq!(user.localpart(), "anna.muster");
        assert_eq!(user.server(), "ordo.drk-musterstadt.de");
        assert!("Anna@ordo.example.org".parse::<UserId>().is_err());
        assert!("anna".parse::<UserId>().is_err());
        assert!("anna@".parse::<UserId>().is_err());
        assert!("a@b@ordo.example.org".parse::<UserId>().is_err());
    }

    #[test]
    fn incident_ids() {
        let ulid = Ulid::from_parts(1_762_600_000_000, 99).unwrap();
        let incident = IncidentId::new(ulid, &server());
        assert_eq!(incident.ulid(), ulid);
        assert_eq!(incident.server(), "ordo.drk-musterstadt.de");
        assert_eq!(incident.as_str().parse::<IncidentId>().unwrap(), incident);
        assert!("not-a-ulid@ordo.example.org".parse::<IncidentId>().is_err());
    }

    #[test]
    fn key_ids() {
        let key = KeyId::new("device-01JC7", &server()).unwrap();
        assert_eq!(key.name(), "device-01JC7");
        assert!("bad key@ordo.example.org".parse::<KeyId>().is_err());
    }

    #[test]
    fn dotted_names() {
        assert!("incident".parse::<CompartmentId>().is_ok());
        assert!("med.clinical".parse::<CompartmentId>().is_ok());
        assert!("med.Clinical".parse::<CompartmentId>().is_err());
        assert!("med..clinical".parse::<ModuleId>().is_err());
        assert!("1med".parse::<ModuleId>().is_err());
    }

    #[test]
    fn event_types() {
        assert!("uhs.patientAdmitted".parse::<EventType>().is_ok());
        assert!("patientAdmitted".parse::<EventType>().is_err());
        assert!("uhs.PatientAdmitted".parse::<EventType>().is_err());
        assert!("uhs.patient-admitted".parse::<EventType>().is_err());
    }

    #[test]
    fn error_explains_problem() {
        let err = "Ordo".parse::<ServerName>().unwrap_err();
        assert_eq!(err.kind(), "server name");
        assert!(err.to_string().contains("a-z"));
    }
}
