---
status: accepted
date: 2026-11-11
---

# Local accounts are managed in ORDO, through Dex's API

## Context and Problem Statement

Every installation has Dex ([ADR-0039](0039-dex-in-every-installation.md)); very small
organisations, and people who join a field server on scene, log in with Dex's local
accounts. Those accounts live in `dex.yaml` (`staticPasswords`, bcrypt hashes), so adding
a helper means editing a file on the server and restarting Dex. Administrators need to do
it in ORDO (ORG-10).

## Considered Options

* Dex's own interface for local accounts
* ORDO manages the accounts through Dex's gRPC API
* Own accounts in ORDO (rejected in ADR-0039)
* A separate identity provider with an admin interface (Keycloak, Authentik, …)

## Decision Outcome

Chosen option: "ORDO manages the accounts through Dex's gRPC API", decided on 2026-11-11.
Dex has no interface for its accounts: they are kept in its configuration or changed
through its gRPC API (`CreatePassword`, `UpdatePassword`, `DeletePassword`,
`ListPasswords`, `RevokeRefresh`). ORDO uses that API and keeps Dex the only place that
stores passwords. A separate provider would mean another service with its own database in
every installation, against the aim of one small field server.

* **Connection:** Dex's API has no authentication of its own, only TLS with client
  certificates. ORDO connects with a client certificate, Dex accepts only certificates of
  the installation's own CA, and the API listens only on the internal network.
  `ordo generate-dex-certs` creates the CA and both certificates (rcgen). Without
  `ORDO_DEX_API` the view is not offered, e.g. where accounts come from an upstream
  provider.
* **Client:** tonic over rustls (ring), with the few messages of Dex's `api.proto` written
  as prost structs, so the build needs no `protoc`.
* **Passwords:** an administrator sets the first password and resets it; ORDO hashes it
  with bcrypt (as Dex expects) and forgets it. At least 12 characters, at most 72 bytes
  (bcrypt's limit).
* **Who:** the organisation permission `manage_accounts` (DE default: administrators).
* **Deleting an account** removes it in Dex and revokes its renewal tokens. ORDO finds
  the person by the login's subject and ends their login at once, so their devices lock
  ([ADR-0040](0040-access-ends-only-on-positive-evidence.md)): the deletion is the positive
  evidence.
* **User IDs:** accounts made through the API have no preferred user name, so the ORDO
  user ID comes from the e-mail address (`eva@…` → `eva`), as for other providers.
* Accounts in `dex.yaml` stay possible; Dex lists them, but they can only be changed there.

### Consequences

* Good, because administrators add helpers on scene without a server shell.
* Good, because passwords stay in Dex, which does the login.
* Bad, because the ORDO server can change every local account: a compromised server can
  take over local accounts (not upstream ones). The client key is a secret like the
  master key.
* Bad, because one more set of certificates has to be created and mounted.

## More Information

[Dex API](https://dexidp.io/docs/configuration/api/),
[Dex local connector](https://dexidp.io/docs/connectors/local/),
[api.proto](https://github.com/dexidp/dex/blob/master/api/v2/api.proto).
