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

1. The server sends `challenge` with 32 random bytes, new for every connection, and
   `versions`: the range of protocol versions it supports (§11).
2. The device sends `hello` with its user, device, the streams it wants (each with the
   last sequence number it already has), the protocol `version` it speaks (§11), and a
   `proof`: the Ed25519 signature with its
   device key over `"ORDO hello v1"`, a zero byte, the challenge, the 16-byte device ID
   and the UTF-8 user ID. The different prefix keeps proofs and event signatures apart.
3. The server MUST refuse (`refused: notAuthenticated`) a hello on a connection without an
   unused challenge, or with a proof that does not verify against the device's registered
   key; and (`refused: unknownDevice`) if the device is not paired with that user. A
   challenge is used once, so a recorded hello cannot be replayed. The server MUST refuse
   (`refused: unsupportedVersion`) a hello whose version is outside its range. Nothing is
   sent to a refused connection; the server closes it after the refusal.
4. The server answers `welcome`:
   - `granted`: streams the user may read. The server then sends every event the device
     is missing (`deliver`).
   - `denied`: streams the user may not read. The server MUST NOT send any event of a
     denied stream to this connection.
   - `protected`: compartments whose payloads must be sealed.
   - `keys`: data keys of granted protected streams.
   - `permissions` (optional): what the user may do beyond compartments, e.g.
     `manage_devices` (§6).
   - `version`: the protocol version agreed for this connection (§11).
   - `valid_until` (optional): the end of this device's login. The device locks itself
     afterwards, also offline, until the person logs in again. The server MUST refuse
     (`refused: loginExpired`) a hello of a device whose login has ended, and MUST end open
     connections of such devices with the same refusal.
5. The device then sends its outbox (`push`), oldest event first, in pushes of at most
   4 MiB of events each (at least one event); it sends the next push when the server has
   ordered the events of the last. The server MAY close a connection whose message is
   larger than 16 MiB. An event the server rejects for the moment (`notConnected`,
   `loginExpired`, `clockAhead`, and the other reasons not about the event itself) stays
   in the outbox for the next connection; one rejected for what it is (`malformed`,
   `wrongAuthor`, `badSignature`, `forbidden`, `wrongSealing`, `duplicateId`) leaves it.

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

`listVehicles` (version 1.1) asks for the installation's vehicles (master data: call sign,
type, organisation, service, default strength), offered when adding units. The server
answers `vehicles`, with the list only for a user with `manage_incidents` or whose roles
permit `edit_units` in some incident, and an empty list for everyone else. A session
agreed on 1.0 gets no answer.

`getMapRegion` (version 1.2) asks for the state of an incident's map region, which the
server offers at `/tiles/<incident ULID>.pmtiles`; `cutMapRegion` asks the server to cut a
new one for the incident around a centre with a radius of 1 to 50 km from its world map.
Both are answered with `mapRegion`, which names the incident. The server MUST cut only for
a user whose roles in that incident give the permission `manage_map`, at most one region
at a time on the server, and replace the incident's region file only once the new one is
complete; it answers anyone else with the state `none`. A device asks again while the
state is `running`. `mapTicket` asks for a ticket to read the incident's region; the
server answers `mapTicket`, with a ticket only for a user who may read the incident's
shared compartment. The device reads the region with HTTP range requests and
`Authorization: Bearer <ticket>`; the server MUST refuse a request without a valid ticket
for that incident (HTTP 403). A ticket is valid for 12 hours; a device asks for one on
every connection. A session agreed on 1.0 or 1.1 gets no answer.

`getOrganisationRoles` (version 1.2) asks for the organisation roles (`admin`, `planner`)
of all people; `setOrganisationRole` grants one, optionally until a time, or withdraws it.
Both are answered with `organisationRoles`, the full list, where roles from the server's
configuration are marked `fixed`. The server MUST change roles only for a user with the
organisation permission `manage_roles`, MUST NOT change fixed roles or let a user withdraw
their own `admin` role, and answers anyone else with an empty list. When a person's roles
change or a grant runs out, the server ends that person's connections with
`refused: rolesChanged`. A session agreed on 1.0 or 1.1 gets no answer.

`ping` (version 1.2) asks the server to answer `pong` at once, on any connection after
the challenge. A device uses it to notice a network that vanished without closing the
connection: having heard nothing for a while, it pings, and without an answer it gives the
connection up and connects anew. A session agreed on 1.0 or 1.1 gets no answer.

`getAccounts` (version 1.2) asks for the local login accounts of the installation's
identity provider (Dex); `changeAccount` creates one with a first password, sets a new
password or deletes one. Both are answered with `accounts`: `managed` false and no list for
a user without the organisation permission `manage_accounts` or when the server has no
connection to Dex's API, else the list and, after a change that was not made, the reason.
The server MUST NOT store passwords. Deleting an account MUST end the person's logins
([ADR-0060](../../docs/adr/0060-local-accounts-managed-in-ordo.md)). A session agreed on
1.0 or 1.1 gets no answer.

`listPeople` asks for the people the server knows, to suggest them when granting a role:
the users with a paired device or a role in any incident. The server answers `people`
(a list of user IDs, sorted). It MUST answer with the list only a user with
`manage_incidents` or with a role that permits `assign_role` in some incident, and with an
empty list everyone else.

## 8. Profile pictures

`getAvatars` asks for the pictures of several people; `setAvatar` sets (with `image`) or
removes the user's own picture. The server answers `avatars`. It resolves each picture in
the order ORDO, identity provider, Gravatar
([ADR-0047](../../docs/adr/0047-profile-pictures.md)), and MUST accept only JPEG, PNG or
WebP images of at most 128 KiB.

## 9. Attachments

Attachments are encrypted on the device and named by the BLAKE3 hash of their ciphertext
([ADR-0033](../../docs/adr/0033-attachments-in-an-object-store.md)). A device asks for a
ticket with `blobTicket`. The server MUST grant an upload ticket only if the user may write
the given stream and the size is within its limit, and a download ticket only if the user
may read the stream and the attachment was uploaded to it. The bytes travel over HTTP:
`PUT /blobs/<hash>` and `GET /blobs/<hash>` (lower-case hex) with
`Authorization: Bearer <ticket>`. The server MUST refuse an upload whose size or BLAKE3
hash differs from the name (HTTP 422), and use an upload ticket only once.

## 10. Changes and open points

Draft 1: the connection is authenticated with a challenge signed by the device key
(§2, [ADR-0038](../../docs/adr/0038-login-and-device-pairing.md)); device logins expire
(`valid_until`, `loginExpired`, [ADR-0040](../../docs/adr/0040-access-ends-only-on-positive-evidence.md));
devices are listed and revoked over the connection (§6, `permissions`, `revoked`);
incidents and roles per incident (§7, `listIncidents`, `rolesChanged`, `listPeople`);
profile pictures (§8); attachments (§9).

Open for draft 2:

- Key grants encrypted for the device key (X25519); key rotation.
- Verifying signatures of other devices' events on the device (key directory).
- AsyncAPI description of the WebSocket channel.

## 11. Versions

Each specification has its own version, `major.minor`
([ADR-0052](../../docs/adr/0052-api-versions.md)). A change that adds something a client
may ignore (a new message, an optional field) raises `minor`; a change that breaks older
clients raises `major`.

- **Sync protocol:** the server names its range in `challenge` (`versions`: `min`, `max`).
  The device SHOULD send the newest version both support in `hello` (`version`). A hello
  without a version is version `1.0`. The server MUST refuse a version outside its range
  (`unsupportedVersion`), MUST confirm the agreed version in `welcome`, and MUST NOT send
  messages or fields newer than that version on the connection.
- **HTTP API** (login, attachments): a request names its version in the header
  `ORDO-API-Version`; without it the oldest supported version applies. The server MUST
  answer a version outside its range with `406 Not Acceptable` and a JSON body naming the
  range (`{"http": {"min": "1.0", "max": "1.0"}}`), and MUST name the version it answers
  in on every other response in the same header.
- **Discovery:** `GET /.well-known/ordo` returns the supported range of each
  specification as JSON, e.g. `{"sync": {"min": "1.0", "max": "1.0"}, "http": {…}}`.
- Event payloads are versioned apart, by `schema_version` per event type ([`event.md`](event.md)).

| Version | Sync protocol | HTTP API |
|---|---|---|
| 1.0 | Draft 1 (§2–§9), with `versions`, `version`, `unsupportedVersion` | `/auth/*`, `/blobs/*` as of 2026-11-10 |
| 1.1 | `listVehicles` and `vehicles` (§7) | unchanged |
| 1.2 | `getMapRegion`, `cutMapRegion`, `mapRegion` and `mapTicket` (§7) | `/tiles/<incident>.pmtiles` with a ticket |

