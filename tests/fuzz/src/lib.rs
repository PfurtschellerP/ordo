//! Shared checks of the fuzz targets (SEC-17, ADR-0059).

use ordo_core::codec;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Decodes `bytes` as `T`; if that works, the value must encode, decode again and encode
/// to the same bytes. Decoding must never panic, whatever the input.
pub fn round_trip<T: Serialize + DeserializeOwned>(bytes: &[u8]) {
    if let Ok(value) = codec::from_cbor::<T>(bytes) {
        let first = codec::to_cbor(&value).expect("a decoded value encodes");
        let again: T = codec::from_cbor(&first).expect("an encoded value decodes");
        let second = codec::to_cbor(&again).expect("and encodes again");
        assert_eq!(first, second, "encoding is stable");
    }
}
