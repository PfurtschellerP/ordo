---
status: accepted
date: 2026-11-08
---

# Encryption at rest on devices: envelope encryption with a device-bound key

## Context and Problem Statement

The web app kept its pairing (with the device's private key) and its replica (with the
data keys of protected compartments) in `localStorage` as plain text (SEC-1 not met).
Browsers offer non-extractable WebCrypto keys, but no key storage that is protected by the
operating system: a non-extractable key only cannot be *exported by scripts*; the browser
still writes it into its IndexedDB files in the profile. Whoever copies an unencrypted
profile from disk therefore gets the key with the data.

Threats on a device, and what helps against each:

| Threat | Effective control |
|---|---|
| Lost or stolen device, switched off or locked | Disk encryption of the operating system (iPadOS, Android with a screen lock; BitLocker, LUKS on laptops), required by the organisation's device management |
| Device left unlocked, or taken while the app runs | Device lock with PIN or biometrics (SEC-13), revocation (SEC-4), access timeout (SEC-12) |
| Script in the page (XSS, malicious extension) | Content Security Policy, few dependencies; keys that scripts cannot read |
| Leftovers after unpairing or a remote wipe | Destroying the key the data was encrypted with (crypto-shredding) |
| Copy of the browser profile from an unencrypted disk | Only a key bound to the person (SEC-13) |

## Considered Options

* Plain storage; rely on disk encryption only
* Envelope encryption: a storage key wrapped by a non-extractable WebCrypto key in IndexedDB
* A key derived from a PIN for everything
* Non-extractable WebCrypto keys for signing (Ed25519) and for every encryption

## Decision Outcome

Chosen option: "Envelope encryption", because it removes plain text and readable keys
from the device storage now, makes unpairing and remote wipe effective, and gives the
device lock (SEC-13) a place to add a key bound to the person, without changing the
storage format again.

* **Key-encryption key:** AES-256-GCM, created on the device, non-extractable, kept in
  IndexedDB (`ordo` / `keys` / `kek-v1`).
* **Storage key:** 32 random bytes, stored only wrapped by the key-encryption key
  (`ordo.storage-key`), unwrapped into memory at start.
* **Items** (pairing, pending pairing, replica): XChaCha20-Poly1305 with the storage key,
  the item's name as associated data; stored as `enc1:` and Base64 of nonce and
  ciphertext. Older plain-text items are encrypted on first load. Without a working vault
  (no HTTPS, no IndexedDB) the app stores nothing and says why.
* **Unpairing and revocation replace the storage key** (crypto-shredding).
* **Disk encryption stays required.** The operating guide asks organisations to enforce a
  screen lock and disk encryption on every device.
* **Later, with SEC-13:** a second key-encryption key from WebAuthn PRF (biometrics,
  platform authenticator) or a PIN with rate limiting; the storage key is then wrapped by
  it, and the app opens only after unlocking. Signing keys as non-extractable WebCrypto
  keys come with the post-quantum migration, which changes signing anyway (ADR-0042).

### Consequences

* Good, because nothing in the device storage is readable without the key-encryption key,
  and a wipe destroys access to every leftover copy.
* Good, because the format allows more key-encryption keys (PIN, biometrics) later.
* Bad, because the device-bound key alone does not protect a copied profile from an
  unencrypted disk; this ADR says so instead of promising it.
* Bad, because the storage key and the data keys are in the app's memory while it runs,
  as before.
* Neutral, because the app now needs a secure context (HTTPS or `localhost`), as the
  offline app shell already did.

## More Information

Implementation: `platform/ordo-app/src/vault.rs`; IndexedDB through the
[`idb`](https://github.com/devashishdxt/idb) crate (MIT/Apache-2.0, ADR-0032). Research
and the plan for post-quantum cryptography and the supply chain: ADR-0042 and ADR-0043
(planned). Device storage moves to SQLite on OPFS later
([ADR-0034](0034-device-storage-sqlite-opfs.md)); the same storage key then encrypts it.
