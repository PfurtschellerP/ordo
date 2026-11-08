---
status: accepted
date: 2026-11-08
---

# Login: OpenID Connect, device keys and QR pairing

## Context and Problem Statement

The prototype paired devices from a seed file, and the sync connection was not
authenticated: a client only *named* its user and device. ORDO needs a login that works
with the identity systems organisations already run, that is safe on shared tablets, and
that keeps working at a scene without internet.

## Considered Options

* Own user accounts and passwords in ORDO
* OpenID Connect with the organisation's identity provider for people; a key per device for connections
* Bearer tokens (OIDC access tokens) for every connection

## Decision Outcome

Chosen option: "OpenID Connect for people, a key per device for connections", because ORDO
then stores no passwords and reuses existing identity providers (ADR-0032), while devices
stay usable offline.

* **People log in at the identity provider** of their organisation via OpenID Connect
  (authorization code flow with PKCE): Keycloak, Authentik, Zitadel, Microsoft Entra ID,
  … ORDO is only a relying party. For small installations the Docker Compose bundle ships
  [Dex](https://dexidp.io) (CNCF, Apache-2.0), with local users or as a bridge to LDAP,
  Active Directory or other providers.
* **Roles, not people, get permissions.** Group claims of the identity provider are
  mapped to ORDO roles in the organisation layer; positions in the command structure add
  incident roles (ADR-0031). Cedar decides (ADR-0035).
* **Every device has its own key pair** (Ed25519), created on the device. The private key
  never leaves it; in browsers it is a non-extractable WebCrypto key where supported.
  Pairing registers the public key for a user.
* **Pairing a device**
  * with a keyboard: the person logs in in the browser; the app registers its key;
  * shared tablet or phone: the OAuth 2.0 Device Authorization Grant (RFC 8628). The
    device shows a QR code; a logged-in person scans it on their own phone and approves.
* **Every connection proves the device key** (sync protocol draft 1): the server sends a
  random challenge; the device signs it, together with its user and device ID. No bearer
  token travels over the sync connection, and nothing has to be reachable at the scene
  except the ORDO server.
* **Offline.** Once paired, a device works without the identity provider. A field server
  checks device keys locally. People log in again after a configurable time (e.g. per
  incident or every 7 days) or when the identity provider is reachable; revoked devices
  are refused at the next connection.
* **HTTP APIs** (later): access tokens bound to the device key with DPoP (RFC 9449), so a
  stolen token is useless without the key.
* The seed file stays for development and tests only.

### Consequences

* Good, because organisations keep their user management, and ORDO keeps no passwords.
* Good, because a lost tablet is revoked by removing its key; no shared passwords.
* Bad, because a test identity provider (Dex) is needed in CI and in the Compose bundle.

## More Information

Built in steps: (1) device proof on every connection and roles with Cedar; (2) OpenID
Connect login, key registration and QR pairing with Dex; (3) protected key storage in the
browser. Protocol: [spec/sync/protocol.md](../../spec/sync/protocol.md).
