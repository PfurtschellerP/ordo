//! Encryption at rest on the device (ADR-0041): envelope encryption.
//!
//! - **Key-encryption key:** AES-256-GCM, created on the device as a non-extractable
//!   WebCrypto key and kept in IndexedDB. Scripts can use it but never read it.
//! - **Storage key:** 32 random bytes, stored only wrapped by the key-encryption key, and
//!   unwrapped into memory when the app starts.
//! - **Items** (pairing with the device key, a pending pairing, the replica with its data
//!   keys) are encrypted with the storage key (XChaCha20-Poly1305, the item's name as
//!   associated data, so items cannot be swapped). Nothing is stored in plain text.
//!
//! Unpairing and revocation replace the storage key: whatever was stored before, also
//! leftovers in the browser's storage files, cannot be decrypted any more
//! (crypto-shredding). The key-encryption key lies in the browser profile, so this does not
//! replace the operating system's disk encryption; a key bound to the person (PIN or
//! biometrics, SEC-13) is meant to be added as a second key-encryption key.

use std::cell::RefCell;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use ordo_crypto::seal::{self, DataKey};

/// Prefix of an encrypted item (format version 1: Base64 of nonce ‖ ciphertext).
const ITEM_PREFIX: &str = "enc1:";
/// Associated data of the wrapped storage key.
const STORAGE_KEY_AAD: &[u8] = b"ORDO storage key v1";
/// Where the wrapped storage key is kept (Base64 of IV ‖ AES-GCM ciphertext).
const STORAGE_KEY_ITEM: &str = "ordo.storage-key";
const DATABASE: &str = "ordo";
const STORE: &str = "keys";
const KEK_ID: &str = "kek-v1";

thread_local! {
    /// The storage key while the app runs.
    static STORAGE_KEY: RefCell<Option<DataKey>> = const { RefCell::new(None) };
}

// [impl->dsn~device-storage-encryption~1]
/// Encrypts `bytes` of the item `label` with `key`: `enc1:` and Base64 of nonce ‖
/// ciphertext.
pub fn seal_item(key: &DataKey, nonce: [u8; 24], label: &str, bytes: &[u8]) -> String {
    let sealed = seal::wrap(key, nonce, &format!("storage/{label}"), bytes);
    format!("{ITEM_PREFIX}{}", BASE64.encode(sealed))
}

// [impl->dsn~device-storage-encryption~1]
/// Decrypts an item made by [`seal_item`]; `None` if it is not encrypted, belongs to
/// another item or key, or was changed.
pub fn open_item(key: &DataKey, label: &str, text: &str) -> Option<Vec<u8>> {
    let sealed = BASE64.decode(text.strip_prefix(ITEM_PREFIX)?).ok()?;
    seal::unwrap(key, &format!("storage/{label}"), &sealed).ok()
}

/// Whether a stored text is encrypted (older versions stored plain text).
pub fn is_encrypted(text: &str) -> bool {
    text.starts_with(ITEM_PREFIX)
}

/// Encrypts an item with the storage key; `None` before [`open`] succeeded.
pub fn encrypt(label: &str, bytes: &[u8]) -> Option<String> {
    let mut nonce = [0u8; 24];
    random(&mut nonce);
    STORAGE_KEY.with_borrow(|key| key.as_ref().map(|k| seal_item(k, nonce, label, bytes)))
}

/// Decrypts an item with the storage key.
pub fn decrypt(label: &str, text: &str) -> Option<Vec<u8>> {
    STORAGE_KEY.with_borrow(|key| open_item(key.as_ref()?, label, text))
}

/// Opens the vault when the app starts: loads or creates the key-encryption key and the
/// storage key. Fails without a secure context (HTTPS) or without IndexedDB; the app then
/// refuses to store anything.
pub async fn open() -> Result<(), String> {
    let subtle = web::subtle()?;
    let kek = match web::load_kek().await? {
        Some(kek) => kek,
        None => {
            let kek = web::generate_kek(&subtle).await?;
            web::store_kek(&kek).await?;
            kek
        }
    };
    let wrapped = web::local_storage().and_then(|s| s.get_item(STORAGE_KEY_ITEM).ok().flatten());
    let unwrapped = match wrapped {
        Some(text) => web::unwrap_storage_key(&subtle, &kek, &text).await,
        None => None,
    };
    let key = match unwrapped {
        Some(key) => key,
        // First start, or the key-encryption key was lost (browser data partly cleared):
        // a new storage key; items of the old one cannot be read any more.
        None => {
            let key = new_storage_key();
            web::save_storage_key(&subtle, &kek, &key).await?;
            key
        }
    };
    STORAGE_KEY.set(Some(key));
    web::KEK.set(Some(kek));
    Ok(())
}

// [impl->dsn~device-storage-encryption~1]
/// Replaces the storage key (unpairing, revocation): everything encrypted before can no
/// longer be read. The old wrapped key is removed at once; the new one is stored shortly
/// after (if the page closes before that, the next start simply makes another key).
pub fn shred() {
    if let Some(storage) = web::local_storage() {
        let _ = storage.remove_item(STORAGE_KEY_ITEM);
    }
    let key = new_storage_key();
    let copy = key.clone();
    STORAGE_KEY.set(Some(key));
    leptos::task::spawn_local(async move {
        let Ok(subtle) = web::subtle() else { return };
        if let Some(kek) = web::KEK.with_borrow(Clone::clone) {
            let _ = web::save_storage_key(&subtle, &kek, &copy).await;
        }
    });
}

fn new_storage_key() -> DataKey {
    let mut bytes = [0u8; 32];
    random(&mut bytes);
    DataKey::from_bytes(bytes)
}

fn random(buffer: &mut [u8]) {
    if let Some(crypto) = web_sys::window().and_then(|w| w.crypto().ok()) {
        let _ = crypto.get_random_values_with_u8_array(buffer);
    }
}

/// Browser APIs: WebCrypto and IndexedDB.
mod web {
    use std::cell::RefCell;

    use idb::{DatabaseEvent, Factory, ObjectStoreParams, TransactionMode};
    use js_sys::{Array, Uint8Array};
    use ordo_crypto::seal::DataKey;
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{AesGcmParams, AesKeyGenParams, CryptoKey, SubtleCrypto};

    use super::{BASE64, DATABASE, Engine, KEK_ID, STORAGE_KEY_AAD, STORAGE_KEY_ITEM, STORE};

    thread_local! {
        /// The key-encryption key while the app runs (a handle; its bytes stay hidden).
        pub(super) static KEK: RefCell<Option<CryptoKey>> = const { RefCell::new(None) };
    }

    const NO_SECURE_CONTEXT: &str =
        "Sichere Speicherung nicht verfügbar: Die App muss über HTTPS geöffnet werden.";

    pub(super) fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    pub(super) fn subtle() -> Result<SubtleCrypto, String> {
        let window = web_sys::window().ok_or(NO_SECURE_CONTEXT)?;
        if !window.is_secure_context() {
            return Err(NO_SECURE_CONTEXT.to_owned());
        }
        Ok(window.crypto().map_err(|_| NO_SECURE_CONTEXT)?.subtle())
    }

    fn error(e: impl std::fmt::Debug) -> String {
        format!("Sichere Speicherung: {e:?}")
    }

    async fn database() -> Result<idb::Database, String> {
        let factory = Factory::new().map_err(error)?;
        let mut request = factory.open(DATABASE, Some(1)).map_err(error)?;
        request.on_upgrade_needed(|event| {
            if let Ok(database) = event.database() {
                let _ = database.create_object_store(STORE, ObjectStoreParams::new());
            }
        });
        request.await.map_err(error)
    }

    pub(super) async fn load_kek() -> Result<Option<CryptoKey>, String> {
        let database = database().await?;
        let transaction = database
            .transaction(&[STORE], TransactionMode::ReadOnly)
            .map_err(error)?;
        let store = transaction.object_store(STORE).map_err(error)?;
        let value = store
            .get(JsValue::from_str(KEK_ID))
            .map_err(error)?
            .await
            .map_err(error)?;
        transaction.await.map_err(error)?;
        database.close();
        Ok(value.and_then(|v| v.dyn_into::<CryptoKey>().ok()))
    }

    pub(super) async fn store_kek(kek: &CryptoKey) -> Result<(), String> {
        let database = database().await?;
        let transaction = database
            .transaction(&[STORE], TransactionMode::ReadWrite)
            .map_err(error)?;
        let store = transaction.object_store(STORE).map_err(error)?;
        store
            .put(kek, Some(&JsValue::from_str(KEK_ID)))
            .map_err(error)?
            .await
            .map_err(error)?;
        transaction.commit().map_err(error)?.await.map_err(error)?;
        database.close();
        Ok(())
    }

    pub(super) async fn generate_kek(subtle: &SubtleCrypto) -> Result<CryptoKey, String> {
        let usages = Array::of2(&"encrypt".into(), &"decrypt".into());
        let params = AesKeyGenParams::new("AES-GCM", 256);
        // Not extractable: scripts can use the key, never read it.
        let promise = subtle
            .generate_key_with_object(&params, false, &usages)
            .map_err(error)?;
        JsFuture::from(promise)
            .await
            .map_err(error)?
            .dyn_into::<CryptoKey>()
            .map_err(error)
    }

    fn gcm(iv: &[u8]) -> AesGcmParams {
        let params = AesGcmParams::new("AES-GCM", &Uint8Array::from(iv));
        params.set_additional_data_u8_array(&Uint8Array::from(STORAGE_KEY_AAD));
        params
    }

    pub(super) async fn save_storage_key(
        subtle: &SubtleCrypto,
        kek: &CryptoKey,
        key: &DataKey,
    ) -> Result<(), String> {
        let mut iv = [0u8; 12];
        super::random(&mut iv);
        let promise = subtle
            .encrypt_with_object_and_u8_array(&gcm(&iv), kek, key.as_bytes())
            .map_err(error)?;
        let ciphertext = Uint8Array::new(&JsFuture::from(promise).await.map_err(error)?).to_vec();
        let text = BASE64.encode([iv.as_slice(), &ciphertext].concat());
        local_storage()
            .ok_or("Kein Gerätespeicher")?
            .set_item(STORAGE_KEY_ITEM, &text)
            .map_err(error)
    }

    pub(super) async fn unwrap_storage_key(
        subtle: &SubtleCrypto,
        kek: &CryptoKey,
        text: &str,
    ) -> Option<DataKey> {
        let bytes = BASE64.decode(text).ok()?;
        if bytes.len() < 12 {
            return None;
        }
        let (iv, ciphertext) = bytes.split_at(12);
        let promise = subtle
            .decrypt_with_object_and_u8_array(&gcm(iv), kek, ciphertext)
            .ok()?;
        let plain = Uint8Array::new(&JsFuture::from(promise).await.ok()?).to_vec();
        Some(DataKey::from_bytes(plain.try_into().ok()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~device-storage-encryption~1]
    #[test]
    fn items_are_encrypted_and_bound_to_their_name_and_key() {
        let key = DataKey::from_bytes([7; 32]);
        let text = seal_item(
            &key,
            [1; 24],
            "ordo.pairing",
            b"{\"private_key\":\"secret\"}",
        );
        assert!(is_encrypted(&text));
        assert!(!text.contains("secret"), "no plain text");
        assert_eq!(
            open_item(&key, "ordo.pairing", &text).as_deref(),
            Some(b"{\"private_key\":\"secret\"}".as_slice())
        );
        assert_eq!(
            open_item(&key, "ordo.replica", &text),
            None,
            "items cannot be swapped"
        );
        assert_eq!(
            open_item(&DataKey::from_bytes([8; 32]), "ordo.pairing", &text),
            None,
            "a replaced storage key cannot read old items (crypto-shredding)"
        );
        let mut changed = text.clone().into_bytes();
        let last = changed.len() - 2;
        changed[last] = if changed[last] == b'A' { b'B' } else { b'A' };
        assert_eq!(
            open_item(&key, "ordo.pairing", &String::from_utf8(changed).unwrap()),
            None,
            "changes are detected"
        );
        assert!(!is_encrypted("{\"user\":\"anna\"}"));
        assert_eq!(open_item(&key, "ordo.pairing", "b64:AAAA"), None);
    }
}
