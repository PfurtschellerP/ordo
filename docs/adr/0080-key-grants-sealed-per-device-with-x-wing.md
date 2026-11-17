---
status: accepted
date: 2026-11-17
---

# Key grants are sealed for each device with X-Wing (X25519 and ML-KEM-768)

## Context and Problem Statement

The server gives each device the data keys of the protected compartments its user may read
(`med.clinical`, `people`; [8.2](../architecture/08-concepts.md#82-data-compartments)).
Until now the key travelled in plain text inside the `welcome`, protected only by TLS.
TLS often ends before ORDO: at Caddy in the Compose bundle, at an operator's reverse
proxy, or at a load balancer. Whoever controls or records traffic there gets every data
key and so every clinical record. Recorded TLS traffic is also exposed to later
decryption by a quantum computer, where the endpoint does not use a hybrid group.
[ADR-0042](0042-post-quantum-readiness.md) step 2 decided that key grants, once encrypted
for the device, use X25519 together with ML-KEM-768, never X25519 alone. SEC-19 asks for
this.

## Considered Options

* X-Wing (X25519 and ML-KEM-768, IETF draft `draft-connolly-cfrg-xwing-kem`) from RustCrypto
  (`x-wing`), with BLAKE3 and XChaCha20-Poly1305 around it
* The same with `libcrux-kem` (Cryspen; formally verified ML-KEM)
* HPKE (RFC 9180) with a hybrid KEM (`hpke` crate)
* Keep the plain key inside TLS

## Decision Outcome

Chosen option: X-Wing from RustCrypto, because it is the hybrid ADR-0042 names, is pure
Rust without `unsafe` code in ORDO and builds for WebAssembly, and comes from the same
project as ORDO's other primitives (`chacha20poly1305`, `ed25519-dalek` via
`curve25519-dalek`).

* **Device key.** A device's X-Wing key is derived from its Ed25519 device seed with
  BLAKE3 (`derive_key`, context `ORDO 2026-11-17 device X-Wing key v1`). Nothing new is
  stored, pairing stays as it is, and the key is the same on every connection.
* **Hello.** The device sends its public X-Wing key (`kem_key`) with every hello. The
  proof signs it (context `ORDO hello v2`), so no one in between can swap in a key of
  their own.
* **Sealing.** For each grant the server encapsulates to `kem_key`, derives an
  XChaCha20-Poly1305 key and nonce from the shared secret with BLAKE3 (one message per
  encapsulation, like HPKE's base mode), and encrypts the 32-byte data key. The
  associated data binds the device ID, incident, compartment and key name, so a grant
  cannot be moved to another device or stream. The decapsulation key refuses
  non-contributory ciphertexts.
* **No plain keys.** Protocol 1.4 carries only sealed grants. Versions 1.0–1.3 carried
  plain keys; a proxy could force them by editing the unauthenticated version range in the
  `challenge`. So the server's range now starts at 1.4. This drops versions before the
  first release, when the server and the web app it serves are the only clients
  ([ADR-0052](0052-api-versions.md) asks to announce dropped versions; there was no one
  to announce it to).
* Spec: [`spec/sync/protocol.md`](../../spec/sync/protocol.md) §2 and §11,
  [`protocol.cddl`](../../spec/sync/protocol.cddl) (`hello`, `key-grant`).

### Consequences

* Good, because a TLS-terminating proxy, a load balancer or a recording no longer sees a
  data key, and recorded grants stay closed even to a later quantum computer as long as
  either X25519 or ML-KEM-768 holds.
* Good, because the device needs no new storage, pairing, or key exchange round trip.
* Bad, because `x-wing` 0.1 is not audited and X-Wing is still an IETF draft (draft 11,
  September 2026); the construction may change before it is final. Its parts (ML-KEM,
  X25519, SHA3) are standard, and the contexts are versioned, so a final version is a
  new context, not a silent change.
* Bad, because every hello is about 1.2 KB larger and every grant about 1.2 KB; a device
  gets two or three grants per incident.
* Neutral: the server still knows the data keys (trust model of 8.2). This protects the
  way between server and device, not the server.

### Confirmation

`ordo-crypto` tests that only the device opens a key sealed for it, and that another
grant, a changed byte or a short encapsulation is refused. The acceptance test
`key_grants` checks that a welcome contains no plain key, that a changed grant is
refused by the device, and that a hello with a swapped or missing `kem_key` is refused
by the server.

## Pros and Cons of the Options

### X-Wing from RustCrypto

* Good, because it is pure Rust, small, WebAssembly-ready and from RustCrypto.
* Bad, because version 0.1, not audited.

### libcrux-kem

* Good, because its ML-KEM is formally verified.
* Bad, because it is pre-1.0 (0.0.x) with an API that still changes, and adds a second
  family of primitives next to RustCrypto.

### HPKE with a hybrid KEM

* Good, because HPKE is an RFC with a fixed key schedule.
* Bad, because the post-quantum and hybrid KEMs for HPKE are still IETF drafts, so there
  is no standard HPKE suite with X-Wing yet. ORDO's construction is HPKE base mode in
  spirit and can move to HPKE once such a suite is final.

### Plain key inside TLS

* Bad, because whoever ends TLS reads every data key.

## More Information

* IETF draft `draft-connolly-cfrg-xwing-kem` (X-Wing)
* NIST FIPS 203 (ML-KEM)
* RFC 9180 (HPKE)
* [ADR-0042](0042-post-quantum-readiness.md), [ADR-0041](0041-encryption-at-rest-on-devices.md)
