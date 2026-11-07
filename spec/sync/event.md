# ORDO Event Envelope

> Specification **draft 0** · 2026-11-08 · Schema: [`event.cddl`](event.cddl)

The key words MUST, MUST NOT, SHOULD and MAY are to be interpreted as described in
RFC 2119.

## 1. Purpose

Every change in ORDO is an immutable **event**. This document defines how an event is
structured, signed, encrypted and ordered, so that devices and servers — including
servers of different organisations and other implementations — can exchange events and
verify them independently.

Events are encoded in **CBOR** (RFC 8949). The exact structures are defined in CDDL
(RFC 8610) in [`event.cddl`](event.cddl).

## 2. Structure

An event travels as a **signed event**: a two-element array of

1. **body** — the CBOR-encoded event body, as a byte string, and
2. **signature** — who signed it and the signature value.

The body is a byte string *inside* the outer structure on purpose. Receivers MUST keep
and forward the body bytes exactly as received. This way:

- a signature can be verified without re-encoding anything;
- a server can store and forward events of modules or envelope extensions it doesn't
  understand, and the signature stays valid.

### 2.1 Body fields

| Key | Meaning |
|---|---|
| `v` | Envelope format version. This document defines version `1`. Receivers MUST reject bodies with a version they don't support. |
| `id` | Event ID: a ULID (16 bytes). Globally unique. |
| `sv` | Schema version of the event type's payload. |
| `hlc` | Hybrid logical clock timestamp (§3). |
| `inc` | Incident the event belongs to (`ULID@server`; the server is where the incident was created). |
| `mod` | Module that defines the event type (e.g. `med.uhs`). |
| `comp` | Compartment the event belongs to (e.g. `incident`, `med.clinical`). |
| `type` | Event type, namespaced (e.g. `uhs.patientAdmitted`). |
| `author` | User and device that created the event. |
| `origin` | Server on which the event was created. |
| `payload` | The module-specific content, either plain or sealed (§5). |

Receivers MUST ignore keys they don't know. Producers SHOULD use deterministic
encoding (RFC 8949 §4.2.1), so that equal content gives equal bytes; correctness does
not depend on it, because signatures and hashes are computed over the bytes as sent.

### 2.2 Identifiers

| Identifier | Form | Example |
|---|---|---|
| Server name | lower-case DNS name, optional port | `ordo.drk-musterstadt.de` |
| User ID | `localpart@server` | `anna@ordo.drk-musterstadt.de` |
| Incident ID | `ULID@server` | `01JC8Z5R6V3N2P1Q0S9T8W7X6Y@ordo.drk-musterstadt.de` |
| Key ID | `name@server` | `device-01JC7…@ordo.drk-musterstadt.de` |
| Event ID, device ID | ULID, 16 bytes binary | — |
| Compartment, module | dotted lower-case | `med.clinical` |
| Event type | dotted, ≥ 2 segments, camelCase | `uhs.patientAdmitted` |

ULIDs in text form use canonical upper-case Crockford base32 (26 characters).
Qualifying identifiers with their server makes them globally unique, which is required
for federation.

## 3. Hybrid logical clock

The `hlc` is `[wall, counter, node]`: wall-clock milliseconds (UTC), a counter for
events within the same millisecond, and the creating device. Timestamps are ordered by
wall, then counter, then node (bytewise).

A clock MUST NOT produce a timestamp lower than or equal to any timestamp it produced
or observed before. When the physical clock does not advance, the counter is
incremented. A receiver SHOULD reject events whose wall time is more than a configured
offset ahead of its own physical clock.

The HLC gives a consistent order across devices for conflict resolution. The
authoritative order of an incident is assigned by its home server (§6).

## 4. Signatures

- Algorithm: **Ed25519** (RFC 8032), verified in strict mode.
- Signed message: the ASCII bytes `ORDO event v1`, one zero byte, then the body bytes.
- `kid` names the signing key. Normally this is the creating device's key, which is
  certified by its server; the key directory is defined in the federation
  specification (to come).

Receivers MUST verify the signature before accepting an event.

## 5. Payload

- **Plain:** `{"plain": bstr}` — the module's payload, CBOR-encoded.
- **Sealed:** `{"sealed": {"ct", "key", "nonce"}}` — the CBOR-encoded payload encrypted
  with **XChaCha20-Poly1305** using the compartment data key named in `key` and a
  random 24-byte `nonce`. The additional authenticated data is the 16 event-ID bytes,
  followed by the UTF-8 incident ID, a zero byte and the UTF-8 compartment ID. This binds
  the ciphertext to its event.

Events in protected compartments (e.g. `med.clinical`) MUST be sealed.

## 6. Ordering and hash chain

The incident's **home server** orders events per incident and compartment and wraps
each in a **sequenced event**:

- `seq`: 1, 2, 3, … without gaps.
- `chain`: `H_chain(previous chain || event hash)`, with 32 zero bytes as the previous
  chain value for `seq = 1`.

Hashes use **BLAKE3** in key-derivation mode for domain separation:

| Value | Definition |
|---|---|
| event hash | BLAKE3 derive-key with context `ORDO event hash v1` over the body bytes |
| chain | BLAKE3 derive-key with context `ORDO chain v1` over previous chain (32 bytes) ‖ event hash (32 bytes) |

Anyone holding the sequence can recompute the chain and detect a changed, removed or
reordered event.

## 7. Limits

- A signed event MUST NOT exceed 1 MiB when encoded. Larger content (e.g. photos) is
  stored as attachments and referenced from the payload.

## 8. Open points for draft 1

- Integer map keys instead of text keys to save space (to be decided after measuring).
- Key directory, device certificates and key rotation (federation specification).
- Test vectors for the conformance suite.
