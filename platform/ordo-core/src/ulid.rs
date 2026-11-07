//! ULIDs: 128-bit identifiers that sort by creation time.
//!
//! ORDO implements only the part of the ULID specification it needs, without a
//! dependency: construction from a timestamp and randomness, the 16-byte binary form
//! and the canonical 26-character Crockford base32 text form.

use core::fmt;
use core::str::FromStr;

const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const TEXT_LEN: usize = 26;
const TIMESTAMP_MAX: u64 = (1 << 48) - 1;
const RANDOM_MASK: u128 = (1 << 80) - 1;

/// A ULID: 48 bits of millisecond timestamp followed by 80 random bits.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ulid(u128);

/// Why a ULID could not be created or parsed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum UlidError {
    /// The text form does not have 26 characters.
    #[error("a ULID has exactly 26 characters")]
    Length,
    /// The text form contains a character outside canonical Crockford base32.
    #[error("a ULID may only contain 0-9 and upper-case A-Z without I, L, O and U")]
    Character,
    /// The text form encodes a value larger than 128 bits.
    #[error("the first character of a ULID must be 0-7")]
    Overflow,
    /// The timestamp does not fit into 48 bits.
    #[error("the timestamp does not fit into 48 bits")]
    Timestamp,
}

impl Ulid {
    /// Builds a ULID from a Unix timestamp in milliseconds and random bits (only the
    /// lowest 80 bits of `random` are used).
    pub fn from_parts(timestamp_ms: u64, random: u128) -> Result<Self, UlidError> {
        if timestamp_ms > TIMESTAMP_MAX {
            return Err(UlidError::Timestamp);
        }
        Ok(Self(
            (u128::from(timestamp_ms) << 80) | (random & RANDOM_MASK),
        ))
    }

    /// The timestamp part, in milliseconds since the Unix epoch.
    pub const fn timestamp_ms(self) -> u64 {
        (self.0 >> 80) as u64
    }

    /// Creates a ULID from its 16-byte big-endian binary form.
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(u128::from_be_bytes(bytes))
    }

    /// The 16-byte big-endian binary form.
    pub const fn to_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }
}

impl fmt::Display for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut text = [0u8; TEXT_LEN];
        for (i, c) in text.iter_mut().enumerate() {
            // 26 characters × 5 bits = 130 bits; the first character carries 3 bits.
            let shift = 125 - 5 * i as u32;
            *c = ALPHABET[((self.0 >> shift) & 0x1f) as usize];
        }
        // The alphabet is ASCII, so this cannot fail.
        f.write_str(core::str::from_utf8(&text).map_err(|_| fmt::Error)?)
    }
}

impl fmt::Debug for Ulid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Ulid({self})")
    }
}

impl FromStr for Ulid {
    type Err = UlidError;

    fn from_str(s: &str) -> Result<Self, UlidError> {
        let bytes = s.as_bytes();
        if bytes.len() != TEXT_LEN {
            return Err(UlidError::Length);
        }
        let mut value: u128 = 0;
        for (i, &c) in bytes.iter().enumerate() {
            let digit = ALPHABET
                .iter()
                .position(|&a| a == c)
                .ok_or(UlidError::Character)? as u128;
            if i == 0 && digit > 7 {
                return Err(UlidError::Overflow);
            }
            value = (value << 5) | digit;
        }
        Ok(Self(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trip() {
        let ulid = Ulid::from_parts(1_762_600_000_000, 0x1234_5678_9abc_def0_1234).unwrap();
        let text = ulid.to_string();
        assert_eq!(text.len(), 26);
        assert_eq!(text.parse::<Ulid>().unwrap(), ulid);
    }

    #[test]
    fn known_values() {
        assert_eq!(
            Ulid::from_bytes([0; 16]).to_string(),
            "00000000000000000000000000"
        );
        assert_eq!(
            Ulid::from_bytes([0xff; 16]).to_string(),
            "7ZZZZZZZZZZZZZZZZZZZZZZZZZ"
        );
        // Example from the ULID specification.
        let spec_example: Ulid = "01ARZ3NDEKTSV4RRFFQ69G5FAV".parse().unwrap();
        assert_eq!(spec_example.timestamp_ms(), 1_469_922_850_259);
    }

    #[test]
    fn sorts_by_time() {
        let earlier = Ulid::from_parts(1000, u128::MAX).unwrap();
        let later = Ulid::from_parts(1001, 0).unwrap();
        assert!(earlier < later);
        assert!(earlier.to_string() < later.to_string());
    }

    #[test]
    fn rejects_invalid_text() {
        assert_eq!("0".parse::<Ulid>(), Err(UlidError::Length));
        assert_eq!(
            "0000000000000000000000000I".parse::<Ulid>(),
            Err(UlidError::Character)
        );
        assert_eq!(
            "0000000000000000000000000a".parse::<Ulid>(),
            Err(UlidError::Character)
        );
        assert_eq!(
            "80000000000000000000000000".parse::<Ulid>(),
            Err(UlidError::Overflow)
        );
    }

    #[test]
    fn rejects_large_timestamp() {
        assert_eq!(Ulid::from_parts(1 << 48, 0), Err(UlidError::Timestamp));
    }

    #[test]
    fn binary_round_trip() {
        let ulid = Ulid::from_parts(42, 7).unwrap();
        assert_eq!(Ulid::from_bytes(ulid.to_bytes()), ulid);
    }
}
