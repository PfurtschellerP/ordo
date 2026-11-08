---
status: accepted
date: 2026-11-08
---

# Dex in every installation; ORDO speaks only OpenID Connect

## Context and Problem Statement

[ADR-0038](0038-login-and-device-pairing.md) lets ORDO use the organisation's identity
provider, with Dex as an option for small installations. Field servers often have no
upstream internet; iPads reach them over local Wi-Fi. A login that depends on an external
provider fails there. Supporting many directory types (LDAP, SAML, …) in ORDO itself would
cost effort for a shrinking need.

## Considered Options

* ORDO talks to whatever provider the organisation runs; Dex optional
* Dex in every installation; ORDO talks only to it; upstream providers connect to Dex via OpenID Connect
* Own accounts in ORDO

## Decision Outcome

Chosen option: "Dex in every installation", because every ORDO server then has a login
that works offline, ORDO needs to know only one protocol, and organisations still use
their existing accounts.

* The Docker Compose bundle always includes Dex, reached at `https://<domain>/dex`. ORDO's
  issuer is fixed to it.
* **Very small organisations:** local accounts in Dex. **Everyone else:** their OpenID
  Connect provider (Microsoft Entra ID, Keycloak, …) as a Dex connector.
* **LDAP and SAML are not supported** by ORDO: Dex could reach them, but we neither
  document them as a path nor test them.
* Dex is a broker, not a cache: a new login with an upstream account needs the upstream
  provider. Offline, already paired devices keep working with their device keys, and
  people who join on scene use local accounts of the field server's Dex.
* Planned, after more important features: an already paired person with the right role
  approves a new device directly, without any identity provider ("substitution at
  half-time"; decided by Cedar).

### Consequences

* Good, because one login path for station and field, tested in CI with the same Dex.
* Good, because organisations keep their user management upstream.
* Bad, because local accounts in Dex have no management UI yet; a small admin page
  using Dex's API is planned.
* Neutral: field servers need certificates prepared in advance (documented in the bundle).

## More Information

Amends ADR-0038 (Dex is no longer optional). Bundle: `deploy/compose/minimal/`.
