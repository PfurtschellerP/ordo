# ORDO Sync Protocol

> Specification **draft 1** · 2026-11-08 · Schema: [`protocol.cddl`](protocol.cddl) ·
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

1. The server sends `challenge` with 32 random bytes, new for every connection.
2. The device sends `hello` with its user, device, the streams it wants (each with the
   last sequence number it already has), and a `proof`: the Ed25519 signature with its
   device key over `"ORDO hello v1"`, a zero byte, the challenge, the 16-byte device ID
   and the UTF-8 user ID. The different prefix keeps proofs and event signatures apart.
3. The server MUST refuse (`refused: notAuthenticated`) a hello on a connection without an
   unused challenge, or with a proof that does not verify against the device's registered
   key; and (`refused: unknownDevice`) if the device is not paired with that user. A
   challenge is used once, so a recorded hello cannot be replayed. Nothing is sent to a
   refused connection; the server closes it after the refusal.
4. The server answers `welcome`:
   - `granted`: streams the user may read. The server then sends every event the device
     is missing (`deliver`).
   - `denied`: streams the user may not read. The server MUST NOT send any event of a
     denied stream to this connection.
   - `protected`: compartments whose payloads must be sealed.
   - `keys`: data keys of granted protected streams.
   - `permissions` (optional): what the user may do beyond compartments, e.g.
     `manage_devices` (§6).
   - `valid_until` (optional): the end of this device's login. The device locks itself
     afterwards, also offline, until the person logs in again. The server MUST refuse
     (`refused: loginExpired`) a hello of a device whose login has ended, and MUST end open
     connections of such devices with the same refusal.
5. The device then sends its whole outbox (`push`).

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

## 6. Devices

After the welcome, a device MAY send `listDevices`; the server answers `devices` with the
devices of the connection's user, or of all users if the user has the permission
`manage_devices`. `revokeDevice` revokes a device under the same rule; otherwise the server
MUST refuse it (`refused: forbidden`). On a revocation the server MUST:

- store it before answering, so it survives a restart;
- end every open connection of the revoked device with `refused: revoked`;
- answer the requester with the new `devices` list (unless it revoked itself);
- refuse every later hello of that device with `refused: revoked`, before checking its
  proof.

A device that receives `refused: revoked` MUST delete its device key and all local data
of ORDO, and MUST NOT reconnect. Pairing it again creates a new device.

## 7. Incidents and roles

Incidents and their rosters are events of the module `incidents` in the incident's
`incident` compartment ([ADR-0046](../../docs/adr/0046-roles-granted-per-incident.md)):
`incidents.planned` (`name`), `incidents.stateChanged` (`state`: planned, active, closed),
`incidents.roleGranted` and `incidents.roleWithdrawn` (`user`, `role`). The server MUST:

- accept `incidents.planned` only from a user with the permission `manage_incidents`, and
  only as the first one of that incident;
- accept the other incident events only for a planned incident, from a user with
  `manage_incidents` or whose roles in that incident permit `assign_role`;
- reject incident events with an invalid payload (`malformed`);
- decide every access with the user's roles in that incident (roster) plus their
  organisation roles;
- end every connection of a user whose roles changed with `refused: rolesChanged`.

A device receiving `rolesChanged` reconnects; for every stream listed as `denied` in the
new welcome it MUST delete that stream's events and key. `listIncidents` returns the
incidents in which the user has a role, or all of them for `manage_incidents`.

## 8. Profile pictures

`getAvatars` asks for the pictures of several people; `setAvatar` sets (with `image`) or
removes the user's own picture. The server answers `avatars`. It resolves each picture in
the order ORDO, identity provider, Gravatar
([ADR-0047](../../docs/adr/0047-profile-pictures.md)), and MUST accept only JPEG, PNG or
WebP images of at most 128 KiB.

## 9. Changes and open points

Draft 1: the connection is authenticated with a challenge signed by the device key
(§2, [ADR-0038](../../docs/adr/0038-login-and-device-pairing.md)); device logins expire
(`valid_until`, `loginExpired`, [ADR-0040](../../docs/adr/0040-access-ends-only-on-positive-evidence.md));
devices are listed and revoked over the connection (§6, `permissions`, `revoked`);
incidents and roles per incident (§7, `listIncidents`, `rolesChanged`); profile pictures
(§8).

Open for draft 2:

- Key grants encrypted for the device key (X25519); key rotation.
- Verifying signatures of other devices' events on the device (key directory).
- AsyncAPI description of the WebSocket channel.
