//! CBOR encoding and decoding.

use std::io::Cursor;

use serde::Serialize;
use serde::de::DeserializeOwned;

/// Encoding or decoding failed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CodecError {
    /// The value could not be encoded.
    #[error("could not encode CBOR: {0}")]
    Encode(String),
    /// The bytes are not valid CBOR for the expected structure.
    #[error("could not decode CBOR: {0}")]
    Decode(String),
    /// There are bytes left after the encoded value.
    #[error("unexpected bytes after the end of the CBOR value")]
    TrailingBytes,
    /// The encoded value exceeds a size limit.
    #[error("{len} bytes exceed the limit of {max} bytes")]
    TooLarge {
        /// Actual size.
        len: usize,
        /// Allowed size.
        max: usize,
    },
    /// The envelope format version is not supported.
    #[error("envelope version {0} is not supported")]
    UnsupportedVersion(u8),
}

/// Encodes a value as CBOR.
pub fn to_cbor<T: Serialize + ?Sized>(value: &T) -> Result<Vec<u8>, CodecError> {
    let mut out = Vec::new();
    ciborium::into_writer(value, &mut out).map_err(|e| CodecError::Encode(format!("{e:?}")))?;
    Ok(out)
}

/// Decodes exactly one CBOR value; trailing bytes are an error.
pub fn from_cbor<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    let mut cursor = Cursor::new(bytes);
    let value =
        ciborium::from_reader(&mut cursor).map_err(|e| CodecError::Decode(format!("{e:?}")))?;
    if cursor.position() != bytes.len() as u64 {
        return Err(CodecError::TrailingBytes);
    }
    Ok(value)
}

/// Serde helpers for fixed-size byte arrays encoded as CBOR byte strings.
///
/// Use with `#[serde(with = "crate::codec::fixed_bytes")]` on `[u8; N]` fields.
pub mod fixed_bytes {
    use core::fmt;

    use serde::{Deserializer, Serializer, de};

    /// Serializes `[u8; N]` as a byte string.
    pub fn serialize<S: Serializer, const N: usize>(
        bytes: &[u8; N],
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(bytes)
    }

    /// Deserializes a byte string of exactly `N` bytes.
    pub fn deserialize<'de, D: Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<[u8; N], D::Error> {
        deserializer.deserialize_bytes(Visitor::<N>)
    }

    struct Visitor<const N: usize>;

    impl<const N: usize> de::Visitor<'_> for Visitor<N> {
        type Value = [u8; N];

        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "a byte string of {N} bytes")
        }

        fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<[u8; N], E> {
            v.try_into().map_err(|_| E::invalid_length(v.len(), &self))
        }
    }
}
