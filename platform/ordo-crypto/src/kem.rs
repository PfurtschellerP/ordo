//! Data keys sealed for one device (ADR-0080, SEC-19).
//!
//! The server sends a device the data keys of the compartments it may read sealed for that
//! device alone, with the hybrid key encapsulation X-Wing (X25519 and ML-KEM-768,
//! ADR-0042): a proxy that ends TLS, or a recording of the traffic, never sees a data key.
//!
//! The device's X-Wing key is derived from its device seed (the Ed25519 signing key's
//! seed) with BLAKE3 in key-derivation mode, so nothing new is stored and pairing stays as
//! it is. A sealed key is like HPKE's base mode with one message: X-Wing gives a shared
//! secret; BLAKE3 derives a key and a nonce from it; XChaCha20-Poly1305 encrypts the
//! 32-byte data key, bound by its associated data to the device, incident, compartment and
//! key name.

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use core::convert::Infallible;
use x_wing::{
    DecapsulationKeyRejectNonContrib, Decapsulator, Encapsulate, EncapsulationKey, KeyExport,
    TryCryptoRng, TryDecapsulate,
};
use zeroize::Zeroize;

use crate::CryptoError;

/// Size of a device's public X-Wing key, in bytes.
pub const PUBLIC_KEY_SIZE: usize = x_wing::ENCAPSULATION_KEY_SIZE;
/// Size of the X-Wing encapsulation in a sealed key, in bytes.
pub const ENCAPSULATED_SIZE: usize = x_wing::CIPHERTEXT_SIZE;

/// Derivation contexts (BLAKE3 `derive_key`): globally unique, never reused.
const SEED_CONTEXT: &str = "ORDO 2026-11-17 device X-Wing key v1";
const KEY_CONTEXT: &str = "ORDO 2026-11-17 key grant encryption key v1";
const NONCE_CONTEXT: &str = "ORDO 2026-11-17 key grant nonce v1";

/// A device's X-Wing key, derived from its device seed. Wiped from memory when dropped.
pub struct DeviceKem(DecapsulationKeyRejectNonContrib);

impl DeviceKem {
    // [impl->dsn~sealed-key-grants~1]
    /// The X-Wing key of the device whose Ed25519 seed is `device_seed`.
    pub fn from_device_seed(device_seed: &[u8; 32]) -> Self {
        let mut seed = blake3::derive_key(SEED_CONTEXT, device_seed);
        let key = DecapsulationKeyRejectNonContrib::from(seed);
        seed.zeroize();
        Self(key)
    }

    /// The public key the device sends with its hello, `PUBLIC_KEY_SIZE` bytes.
    pub fn public_key(&self) -> Vec<u8> {
        self.0.encapsulation_key().to_bytes().to_vec()
    }

    // [impl->dsn~sealed-key-grants~1]
    /// Opens a data key sealed for this device with `aad`, the grant it belongs to.
    ///
    /// # Errors
    ///
    /// [`CryptoError::KeyGrant`] if the key was sealed for another device or grant, or was
    /// changed.
    pub fn open(&self, sealed: &SealedKey, aad: &[u8]) -> Result<[u8; 32], CryptoError> {
        let encapsulated = x_wing::Ciphertext::try_from(sealed.encapsulated.as_slice())
            .map_err(|_| CryptoError::KeyGrant)?;
        let mut shared = self
            .0
            .try_decapsulate(&encapsulated)
            .map_err(|_| CryptoError::KeyGrant)?;
        let (cipher, nonce) = cipher(&shared);
        shared.zeroize();
        let mut plain = cipher
            .decrypt(
                &nonce,
                Payload {
                    msg: &sealed.ciphertext,
                    aad,
                },
            )
            .map_err(|_| CryptoError::KeyGrant)?;
        let key: [u8; 32] = plain
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::KeyGrant)?;
        plain.zeroize();
        Ok(key)
    }
}

impl core::fmt::Debug for DeviceKem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("DeviceKem(…)")
    }
}

/// A data key sealed for one device.
#[derive(Clone, PartialEq, Eq)]
pub struct SealedKey {
    /// The X-Wing encapsulation, `ENCAPSULATED_SIZE` bytes.
    pub encapsulated: Vec<u8>,
    /// The data key, encrypted: 32 bytes and a 16-byte tag.
    pub ciphertext: Vec<u8>,
}

impl core::fmt::Debug for SealedKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SealedKey").finish_non_exhaustive()
    }
}

/// The cipher and nonce of one sealed key, from the X-Wing shared secret.
fn cipher(shared: &[u8]) -> (XChaCha20Poly1305, XNonce) {
    let mut key = blake3::derive_key(KEY_CONTEXT, shared);
    let nonce_bytes = blake3::derive_key(NONCE_CONTEXT, shared);
    let cipher = XChaCha20Poly1305::new(&key.into());
    key.zeroize();
    let mut nonce = [0u8; 24];
    nonce.copy_from_slice(&nonce_bytes[..24]);
    (cipher, XNonce::from(nonce))
}

/// Randomness for the encapsulation, from the caller's source (the server's entropy).
struct Entropy<'a>(&'a mut dyn FnMut(&mut [u8]));

impl x_wing::kem::common::rand_core::TryRng for Entropy<'_> {
    type Error = Infallible;

    fn try_next_u32(&mut self) -> Result<u32, Infallible> {
        let mut b = [0u8; 4];
        (self.0)(&mut b);
        Ok(u32::from_le_bytes(b))
    }

    fn try_next_u64(&mut self) -> Result<u64, Infallible> {
        let mut b = [0u8; 8];
        (self.0)(&mut b);
        Ok(u64::from_le_bytes(b))
    }

    fn try_fill_bytes(&mut self, dst: &mut [u8]) -> Result<(), Infallible> {
        (self.0)(dst);
        Ok(())
    }
}

impl TryCryptoRng for Entropy<'_> {}

// [impl->dsn~sealed-key-grants~1]
/// Seals `key` for the device with the public X-Wing key `public`, bound to `aad`.
/// `entropy` must fill buffers with cryptographically secure random bytes.
///
/// # Errors
///
/// [`CryptoError::KeyGrant`] if `public` is not an X-Wing public key.
pub fn seal_for_device(
    public: &[u8],
    key: &[u8; 32],
    aad: &[u8],
    entropy: &mut dyn FnMut(&mut [u8]),
) -> Result<SealedKey, CryptoError> {
    let device = EncapsulationKey::try_from(public).map_err(|_| CryptoError::KeyGrant)?;
    let (encapsulated, mut shared) = device.encapsulate_with_rng(&mut Entropy(entropy));
    let (cipher, nonce) = cipher(&shared);
    shared.zeroize();
    let ciphertext = cipher
        .encrypt(&nonce, Payload { msg: key, aad })
        .map_err(|_| CryptoError::KeyGrant)?;
    Ok(SealedKey {
        encapsulated: encapsulated.to_vec(),
        ciphertext,
    })
}

/// What a sealed key is bound to: the device it is for and the grant (spec/sync
/// protocol.md §2): 16 device-ID bytes, then the UTF-8 incident ID, compartment ID and key
/// name, each followed by a zero byte.
pub fn grant_aad(device: &[u8; 16], incident: &str, compartment: &str, key_id: &str) -> Vec<u8> {
    let mut aad = Vec::with_capacity(16 + incident.len() + compartment.len() + key_id.len() + 3);
    aad.extend_from_slice(device);
    for part in [incident, compartment, key_id] {
        aad.extend_from_slice(part.as_bytes());
        aad.push(0);
    }
    aad
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entropy(seed: u8) -> impl FnMut(&mut [u8]) {
        let mut state = u64::from(seed) | 1;
        move |buf: &mut [u8]| {
            for b in buf {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                *b = (state >> 24) as u8;
            }
        }
    }

    // [utest->dsn~sealed-key-grants~1]
    #[test]
    fn only_the_device_opens_a_key_sealed_for_it() {
        let device = DeviceKem::from_device_seed(&[7; 32]);
        assert_eq!(device.public_key().len(), PUBLIC_KEY_SIZE);
        assert_eq!(
            DeviceKem::from_device_seed(&[7; 32]).public_key(),
            device.public_key(),
            "derived from the device seed, the same every time"
        );
        let aad = grant_aad(
            &[1; 16],
            "01J@ordo.example.org",
            "med.clinical",
            "k1@ordo.example.org",
        );
        let sealed =
            seal_for_device(&device.public_key(), &[42; 32], &aad, &mut entropy(3)).unwrap();
        assert_eq!(sealed.encapsulated.len(), ENCAPSULATED_SIZE);
        assert_eq!(sealed.ciphertext.len(), 48);
        assert_eq!(device.open(&sealed, &aad).unwrap(), [42; 32]);

        // Another device, another grant, a changed byte: nothing.
        let other = DeviceKem::from_device_seed(&[8; 32]);
        assert!(other.open(&sealed, &aad).is_err());
        let elsewhere = grant_aad(
            &[1; 16],
            "01J@ordo.example.org",
            "people",
            "k1@ordo.example.org",
        );
        assert!(device.open(&sealed, &elsewhere).is_err());
        let mut changed = sealed.clone();
        changed.ciphertext[0] ^= 1;
        assert!(device.open(&changed, &aad).is_err());
        let mut short = sealed;
        short.encapsulated.pop();
        assert!(device.open(&short, &aad).is_err());
        assert!(seal_for_device(&[0; 10], &[42; 32], &aad, &mut entropy(3)).is_err());
    }

    #[test]
    fn the_aad_keeps_its_parts_apart() {
        assert_ne!(
            grant_aad(&[0; 16], "a", "bc", "d"),
            grant_aad(&[0; 16], "ab", "c", "d")
        );
    }
}
