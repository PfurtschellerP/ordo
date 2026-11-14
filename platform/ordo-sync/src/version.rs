//! Versions of ORDO's specifications ([ADR-0052](../../../docs/adr/0052-api-versions.md)):
//! `major.minor`, written as text (`"1.0"`). A change a client may ignore raises `minor`,
//! a breaking change raises `major`.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// A version of a specification, e.g. `1.4`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version {
    /// Raised by changes that break older clients.
    pub major: u16,
    /// Raised by changes older clients may ignore.
    pub minor: u16,
}

impl Version {
    /// A version.
    pub const fn new(major: u16, minor: u16) -> Self {
        Self { major, minor }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// Text that is not `major.minor`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a version (major.minor): {0:?}")]
pub struct VersionError(String);

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let error = || VersionError(s.to_owned());
        let (major, minor) = s.split_once('.').ok_or_else(error)?;
        let number = |part: &str| {
            (!part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
                .then(|| part.parse::<u16>().ok())
                .flatten()
                .ok_or_else(error)
        };
        Ok(Self::new(number(major)?, number(minor)?))
    }
}

impl Serialize for Version {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// The versions a server supports, from `min` to `max`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionRange {
    /// Oldest supported version; also the version of a request that names none.
    pub min: Version,
    /// Newest supported version.
    pub max: Version,
}

impl VersionRange {
    // [impl->dsn~protocol-versions~1]
    /// Whether `version` is supported.
    pub fn contains(&self, version: Version) -> bool {
        self.min <= version && version <= self.max
    }

    /// The version to speak with a server that supports `server`, if this side supports
    /// `self`: the newest both support.
    pub fn agree(&self, server: &VersionRange) -> Option<Version> {
        let newest = self.max.min(server.max);
        (newest >= self.min.max(server.min)).then_some(newest)
    }
}

/// The versions of the sync protocol this implementation speaks (spec/sync/protocol.md §11).
pub const SYNC: VersionRange = VersionRange {
    min: Version::new(1, 0),
    max: Version::new(1, 2),
};

/// Sync protocol 1.2: `getMapRegion`, `cutMapRegion`, `mapRegion` and `mapTicket` (MAP-8).
pub const SYNC_MAP_REGION: Version = Version::new(1, 2);

/// Sync protocol 1.2: `geocode` and `geocoded`, the address search (ADR-0066).
pub const SYNC_GEOCODE: Version = Version::new(1, 2);

/// Sync protocol 1.2: `ping` and `pong`, to notice a vanished network.
pub const SYNC_LIVENESS: Version = Version::new(1, 2);

/// Sync protocol 1.2: `listIncidentDevices`, `blockDevice` and `incidentDevices` (ORG-12).
pub const SYNC_INCIDENT_DEVICES: Version = Version::new(1, 2);

/// The versions of the client–server HTTP API (login, attachments) this implementation
/// speaks; a request names its version in the header [`HTTP_HEADER`].
pub const HTTP: VersionRange = VersionRange {
    min: Version::new(1, 0),
    max: Version::new(1, 0),
};

/// The HTTP header that names the API version of a request and of its response.
pub const HTTP_HEADER: &str = "ORDO-API-Version";

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~protocol-versions~1]
    #[test]
    fn versions_are_major_dot_minor_and_ordered_by_number() {
        let v: Version = "1.10".parse().unwrap();
        assert_eq!(v, Version::new(1, 10));
        assert_eq!(v.to_string(), "1.10");
        assert!(Version::new(1, 9) < v, "by number, not as text");
        assert!(Version::new(2, 0) > v);
        for bad in ["1", "1.", ".1", "1.2.3", "v1.0", "1.-1", " 1.0", "1.0 "] {
            assert!(bad.parse::<Version>().is_err(), "{bad:?}");
        }
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, "\"1.10\"");
        assert_eq!(serde_json::from_str::<Version>(&json).unwrap(), v);
    }

    // [utest->dsn~protocol-versions~1]
    #[test]
    fn ranges_contain_their_ends_and_agree_on_the_newest_shared_version() {
        let server = VersionRange {
            min: Version::new(1, 2),
            max: Version::new(1, 5),
        };
        assert!(server.contains(Version::new(1, 2)) && server.contains(Version::new(1, 5)));
        assert!(!server.contains(Version::new(1, 1)) && !server.contains(Version::new(2, 0)));
        let older = VersionRange {
            min: Version::new(1, 0),
            max: Version::new(1, 3),
        };
        assert_eq!(older.agree(&server), Some(Version::new(1, 3)));
        let newer = VersionRange {
            min: Version::new(1, 4),
            max: Version::new(2, 1),
        };
        assert_eq!(newer.agree(&server), Some(Version::new(1, 5)));
        let too_old = VersionRange {
            min: Version::new(1, 0),
            max: Version::new(1, 1),
        };
        assert_eq!(too_old.agree(&server), None, "no shared version");
    }
}
