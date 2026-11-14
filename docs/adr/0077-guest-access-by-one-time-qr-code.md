---
status: accepted
date: 2026-11-14
---

# Helpers without an account get access by a one-time QR code, until their duty ends

## Context and Problem Statement

Spontaneous helpers and helpers of other organisations have no account with the
organisation's identity provider, yet some of them need ORDO in the incident: a medic of a
neighbouring organisation on a medical team, for instance, who should see the team's
patient cases (REP-3). The product owner decided on 2026-11-14
([ADR-0076](0076-people-and-roles-in-one-list.md) is the first part): their role is chosen
at registration, never *Einsatzleitung*; they log in with a QR code the incident command
makes for them in this incident; their access ends when their duty ends.

Devices are paired by logging in at the identity provider (OpenID Connect, RFC 8628 for a
second device; [ADR-0038](0038-login-and-device-pairing.md)). A guest has no login there. Creating
an account at the provider for each guest would need an administrator, a password the
guest has to type on a phone, and someone to delete the account afterwards.

## Considered Options

* A one-time pairing code from ORDO, for a guest identity that exists only in ORDO
* A local account at the identity provider (Dex) per guest, with a generated password
* A shared "guest" account per incident

## Decision Outcome

Chosen option: "A one-time pairing code from ORDO". It replaces only the login step of
pairing: the device still makes its own key pair, and the server registers its public key
as for a login.

* **Guest identity:** the server makes a user ID `guest-<ULID>@<server>` for one incident.
  Names that the identity provider maps to start with `guest-` get a prefix (`u-`), so no
  login can take a guest's ID. The guest's name is the helper's name in the register,
  shown as their display name.
* **Making the code:** the incident command registers the helper without an account,
  chooses the role and the end of duty, and asks for access. The server checks that the
  person may grant roles in the incident, makes the guest ID and a code of 256 random bits
  (base64url), keeps only its SHA-256 hash, valid once for 15 minutes, and returns it with
  a QR code of the link `https://<server>/?guest=<code>`. The command's device grants the
  role to the guest ID (`roleGranted`) and links the helper to it, as for an account. A new
  code can be made at any time; it makes the earlier unused ones invalid.
* **Using the code:** the helper scans it with their phone. The app makes a device key and
  sends code, device ID, public key and device name to `POST /auth/guest`. The server
  looks up the hash; an unknown, used or expired code is refused the same way, without
  saying which. On success the code is used up, and the device is paired with the guest's
  ID, its login ending at the end of duty.
* **Access ends with duty:** the end of a guest device's login is the helper's end of
  duty, which the command sets with the code and changes with `setGuestAccess` when it
  changes the end of duty or ends the duty (*Dienst beenden*: now). The server ends the
  device's connections when its login ends (`loginExpired`, [ADR-0040](0040-access-ends-only-on-positive-evidence.md));
  there is no renewal, since there is no identity provider to ask. A lost phone is
  blocked like any device ([ORG-12](../requirements/02-shared.md#org-12--devices-in-the-incident)).
* **Never *Einsatzleitung*:** the Cedar context carries `guest`, and a `forbid` policy
  takes every permission from a guest who holds `commander`, so neither a mistake in the
  app nor an organisation's own policy can make a guest lead. The app does not offer the
  role for guests.
* **Security** (OWASP ASVS 5.0 level 2, authentication and session management): the code
  has 256 bits from the operating system's random source, is used once, lives 15 minutes,
  is stored only as a hash, and is never logged; refusals are counted, and the server
  answers `429 Too Many Requests` after 20 refused codes in a minute. Codes travel only
  over TLS. Every code made and used is logged with who made it and the guest's ID, never
  the code. The pairing gives no more than the role in this one incident.

A local account per guest needs administrator rights (`manage_accounts`) and a password
the guest types; a shared account per incident cannot end one helper's access without the
others' and cannot say who did what.

### Consequences

* Good, because a helper without an account is on ORDO within a minute, with exactly the
  role and the time the command decided, and their actions carry their own ID.
* Good, because access ends by itself with the duty, also if nobody remembers to withdraw
  it.
* Bad, because ORDO keeps a second way to pair, which needs its own tests and review.
* Bad, because whoever holds an unused code for its 15 minutes can pair with it; the QR
  code is shown only on the command's device, and a new one makes it invalid.

## More Information

Design item *Guest access* in [chapter 8](../architecture/08-concepts.md#guest-access);
requirement HELP-7. Sync protocol 1.3 (`createGuest`, `guest`, `setGuestAccess`), HTTP
API 1.1 (`POST /auth/guest`).
