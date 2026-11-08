# ORDO Sync Protocol

> Specification **draft 0** · 2026-11-08 · Schema: [`protocol.cddl`](protocol.cddl) ·
> Events: [`event.md`](event.md)

The key words MUST, MUST NOT, SHOULD and MAY are to be interpreted as described in
RFC 2119.

## 1. Model

- A **stream** is one compartment of one incident. It is the unit of ordering, access
  and encryption.
- The incident's **home server** orders each stream (`seq` 1, 2, 3, … and hash chain).
- A **device** keeps a local copy of the streams its user may read, plus an **outbox**
  of events it created that the server has not yet ordered.

Messages are CBOR maps; the transport (draft 0: WebSocket, binary frames) carries one
message per frame.

## 2. Connecting

1. The device sends `hello` with its user, device and the streams it wants, each with the
   last sequence number it already has.
2. The server MUST refuse (`refused: unknownDevice`) if the device is not paired with
   that user.
3. The server answers `welcome`:
   - `granted`: streams the user may read. The server then sends every event the device
     is missing (`deliver`).
   - `denied`: streams the user may not read. The server MUST NOT send any event of a
     denied stream to this connection.
   - `protected`: compartments whose payloads must be sealed.
   - `keys`: data keys of granted protected streams.
4. The device then sends its whole outbox (`push`).

## 3. Pushing events

For each pushed event the server MUST check, in this order, and reject otherwise:

| Check | Rejection |
|---|---|
| The body decodes | `malformed` |
| Author user and device equal the connection's | `wrongAuthor` |
| The signature is valid for the device's key | `badSignature` |
| The user may write to the stream | `forbidden` |
| Protected streams: sealed with the current key; other streams: plain | `wrongSealing` |
| The event's clock is not too far ahead | `clockAhead` |

An event the server has already ordered (same event ID, same body) MUST be ignored,
without rejection; resending is therefore always safe. An event with a known ID but a
different body MUST be rejected with `duplicateId`.

Accepted events are ordered and delivered (`deliver`) to every connection granted the
stream — including the sender, which then removes them from its outbox.

## 4. Receiving events

A device MUST check that delivered events continue its local sequence and that each
chain value matches ([event.md §6](event.md)). A gap or a broken chain is an error; the
device reconnects and catches up. The device MUST discard deliveries for streams it was
not granted.

## 5. Derived events

The server MAY publish events it derives from a stream, e.g. connector events such as
`med.patientCounts.v1` ([ADR-0024](../../docs/adr/0024-server-derived-connectors.md)).
They are signed by the server and ordered like any other event.

## 6. Open points for draft 1

- Authentication of the connection (token or challenge with the device key).
- Key grants encrypted for the device key (X25519); key rotation.
- Verifying signatures of other devices' events on the device (key directory).
- AsyncAPI description of the WebSocket channel.
