---
status: accepted
date: 2026-11-10
---

# Post-quantum readiness: hybrid key exchange now, replaceable algorithms throughout

## Context and Problem Statement

Clinical data recorded today must stay confidential for decades. An attacker can record
encrypted traffic now and decrypt it once a large quantum computer exists ("harvest now,
decrypt later"). That threatens key exchange (TLS, key grants), not symmetric encryption
with 256-bit keys or 256-bit hashes. Signatures are threatened later, by forgery, which
matters for the long-term evidence of the incident log. ORDO's algorithms today:

| Use | Algorithm | Quantum risk |
|---|---|---|
| TLS between devices and the server (Compose: Caddy) | TLS 1.3; key exchange per client | key exchange: harvest now, decrypt later |
| Event and hello signatures | Ed25519, versioned contexts (`ORDO event v1`, `ORDO hello v1`) | forgery once a quantum computer exists |
| Sealed payloads and attachments | XChaCha20-Poly1305, 256-bit data keys | none known that matters |
| Device storage | AES-256-GCM (WebCrypto), key wrapped per device (ADR-0041) | none known that matters |
| Hash chain, attachment names | BLAKE3, 256 bit | none known that matters |
| Key grants to devices (planned, draft 2) | X25519 planned | key exchange: harvest now, decrypt later |
| Outgoing TLS of the server (OIDC, object store) | rustls with *ring*, X25519 | key exchange; inside the operator's network |

The standards are now final: ML-KEM (FIPS 203) and ML-DSA (FIPS 204). National bodies such
as the BSI recommend using them in combination with a classical algorithm (hybrid) for
now. Browsers and current TLS servers (Go 1.24 and later, therefore Caddy; OpenSSL 3.5)
offer the hybrid group X25519MLKEM768 by default.

## Considered Options

* Wait until quantum computers are closer
* Post-quantum algorithms alone
* Hybrid: classical and post-quantum together, first where the risk is now (key exchange),
  with every algorithm replaceable

## Decision Outcome

Chosen option: hybrid, step by step.

1. **TLS now.** The TLS endpoint offers X25519MLKEM768 first, then classical groups for
   older clients. The Compose bundle sets it in its Caddyfile, and CI checks that a client
   offering only X25519MLKEM768 gets it (`tests/tls/check_hybrid_tls.py`, OpenSSL 3.5).
   Operators with their own proxy run the same check.
2. **Key grants hybrid from the start.** When key grants are encrypted for the device
   (protocol draft 2), they use X25519 together with ML-KEM-768 (e.g. the X-Wing
   combination), never X25519 alone.
3. **Signatures later, hybrid.** Ed25519 stays until a reviewed ML-DSA implementation
   for Rust and the browser is mature. Then events carry Ed25519 and ML-DSA-65 signatures
   in a new envelope version. Already stored events keep their signatures; the hash chain
   protects their order and content in the meantime.
4. **Crypto agility.** Every signature, key and sealed payload names its algorithm or
   carries a versioned context, so an algorithm is replaced by a new version instead of a
   silent change. This inventory is kept current in this ADR and in architecture 8.5.
5. **Outgoing TLS of the server** moves to hybrid when its TLS library supports it
   without new build requirements.

### Consequences

* Good, because the most urgent risk, recorded traffic, is covered now at no cost to
  older clients, and CI keeps it covered.
* Good, because later changes are planned as new versions, not emergencies.
* Bad, because hybrid key exchange makes handshakes a little larger (about 1 KB more).
* Neutral: the security depends on the proxy in front of ORDO; the check makes that
  visible.

## More Information

* NIST FIPS 203 (ML-KEM) and FIPS 204 (ML-DSA), August 2024
* BSI, Technische Richtlinie TR-02102-1 (current edition): recommendations on
  cryptographic mechanisms, including hybrid post-quantum key exchange
* IETF draft "Hybrid key exchange in TLS 1.3" and the group X25519MLKEM768
