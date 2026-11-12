---
status: accepted
date: 2026-11-12
---

# Upstream logins are configured in ORDO, through Dex's connector API, after a test login

## Context and Problem Statement

Most organisations have their own identity provider (Microsoft Entra ID, Keycloak,
Authentik, Google, …). Dex connects ORDO to it ([ADR-0039](0039-dex-in-every-installation.md)),
but today an operator writes the connector into `dex.yaml` and restarts Dex. Local
accounts are already managed in the app through Dex's API
([ADR-0060](0060-local-accounts-managed-in-ordo.md)); administrators asked to connect their
provider the same way. A provider that is set up wrongly is noticed only at the next real
login, and a server that may change providers can redirect every login of the organisation.

## Considered Options

* Dex's connector API (`CreateConnector`, `UpdateConnector`, `DeleteConnector`,
  `ListConnectors`), driven by ORDO, with a test login before saving
* The same, saved without a test login
* Keep providers in `dex.yaml` only

## Decision Outcome

Chosen option: "Dex's connector API, with a test login before saving", decided on
2026-11-12 with the product owner.

* **Who:** the organisation permission `manage_accounts` (DE default: administrators), like
  local accounts. It may be split into a permission of its own later.
* **What:** several OpenID Connect providers, each a button on Dex's login page (e.g. a
  joint field server of two organisations). Name, issuer URL, client ID and client secret,
  and whether to skip the `email_verified` check (Entra ID does not send it). ORDO sets
  the redirect URI (`<Dex issuer>/callback`) and shows it, to be registered at the
  provider. LDAP, SAML and other connectors stay in `dex.yaml`; ORDO lists them but does
  not change them.
* **Test login first:** ORDO creates a temporary connector (`ordo-test-…`, at most
  10 minutes), the administrator logs in through it once in a new browser tab, and ORDO
  shows the user ID, name and e-mail address it would take from that login, without
  pairing a device. Only then can the administrator save; saving asks for a confirmation
  that logins through this provider will be accepted. While the test runs, Dex shows the
  temporary button to everyone; ORDO removes it after saving, after 10 minutes, or at
  start.
* **Secrets** are written to Dex and never sent back to the app. Changing a provider
  without a new secret keeps the old one; every change needs a new test login.
* **Logged:** the server logs every saved and deleted provider with who did it.
* **Enabling:** Dex needs `DEX_API_CONNECTORS_CRUD=true` (Dex 2.46) and persistent storage
  (the Compose bundle has SQLite). Without `ORDO_DEX_API`, the view is not offered.

### Consequences

* Good, because administrators connect their provider without a server shell, and a
  wrong issuer or secret shows up in the test, not at the next operation.
* Good, because the secret stays in Dex, which does the login.
* Bad, because the ORDO server can redirect every login of the organisation: a compromised
  server can add a provider of its own. The client key of Dex's API is a secret like the
  master key (as in ADR-0060), and every change is logged.
* Bad, because the temporary test button is visible on the login page for a few minutes.

## More Information

[Dex API](https://dexidp.io/docs/configuration/api/),
[Dex OpenID Connect connector](https://dexidp.io/docs/connectors/oidc/),
[api.proto](https://github.com/dexidp/dex/blob/master/api/v2/api.proto),
[spec/sync/protocol.md](../../spec/sync/protocol.md).
