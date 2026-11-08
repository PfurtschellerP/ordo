---
status: proposed
date: 2026-11-08
---

# Access ends only on positive evidence, never on unreachability

## Context and Problem Statement

Paired devices work without the identity provider (ADR-0038, ADR-0039). Organisations
still want logins re-verified from time to time, and roles kept in line with their
directory. A field server's Dex can be reachable while its upstream provider is not. Dex
answers a failed renewal of an upstream identity with the same generic server error
(HTTP 500) whether the upstream was unreachable or rejected the person; only its own
decisions (expired renewal token, ended session) come back as `invalid_grant`. A rule like
"re-login once the login service is reachable" would therefore lock people out in the
field.

## Considered Options

* Forced re-login after a fixed time, when the login service seems reachable
* Background re-verification on the server; access ends only on positive evidence
* No re-verification at all

## Decision Outcome

Proposed option: "Background re-verification; access ends only on positive evidence",
because emergency work must not stop for an unreachable service, while revocations and
directory changes still take effect.

* **Ends access:** an admin revokes the device or person in ORDO (also locally on a field
  server); or a *successful* renewal shows that the person no longer holds any ORDO group,
  so their roles become empty and Cedar denies everything.
* **Never ends access:** no network, Dex or the upstream unreachable, a generic error, an
  expired renewal token. These only leave "last verified at …" unchanged.
* **Re-verification runs on the server**, in the background: it keeps Dex's renewal token
  per person (wrapped with the master key) and renews it from time to time. Success updates
  the roles; failure keeps the last known roles. Nobody is asked to log in again while
  working.
* **Admins see "last verified"** per device and person and can be alerted after a
  configurable time. An automatic lockout for that reason is an explicit organisation
  setting and never applies during an active incident.
* **Probing the upstream** is only used to explain the status ("login service of the
  organisation not reachable – devices keep working"), never to decide access.

### Consequences

* Good, because a disconnected field deployment keeps working, and revocations still work
  locally.
* Bad, because a person disabled upstream keeps access until an admin revokes them or the
  upstream can be asked again; compensated by device keys that cannot be copied, the screen
  lock with PIN or biometrics on devices with clinical data (SEC-4), local revocation and
  the audit trail.

## More Information

Built with login step 3 (roadmap). Dex behaviour: `server/grants/refresh.go` in Dex v2.46.
