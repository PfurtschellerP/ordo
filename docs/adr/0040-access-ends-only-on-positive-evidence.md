---
status: accepted
date: 2026-11-08
---

# Device logins expire after a timeout; unreachability alone ends nothing early

## Context and Problem Statement

Paired devices work without the identity provider (ADR-0038, ADR-0039). Access must not
last forever, yet a field deployment must survive some days without internet. A field
server's Dex can be reachable while its upstream provider is not, and Dex answers a failed
renewal of an upstream identity with the same generic server error (HTTP 500) whether the
upstream was unreachable or rejected the person; only its own decisions (expired renewal
token, ended session) come back as `invalid_grant`. A rule like "re-login once the login
service is reachable" would lock people out in the field.

## Considered Options

* Access without a time limit, ended only by an admin
* A timeout since the last successful verification, renewed in the background; a definitive refusal ends access at once
* Forced re-login after a fixed time, when the login service seems reachable

## Decision Outcome

Chosen option: "Timeout since the last successful verification", decided on 2026-11-08.

* **Every device login has an end**: 48 hours after the last successful verification by
  default, configurable from 1 to 30 days (`ORDO_ACCESS_TIMEOUT_HOURS`, 24–720) for long
  deployments without internet. We adjust the default when experience shows it is too
  short.
* **Verification** happens at every login on any of the person's devices, and in the
  background: the server keeps the identity provider's renewal token per person (wrapped
  with the master key) and renews it every 15 minutes. A confirmation moves the end of
  all the person's device logins to "now + timeout" and updates their roles.
* **Access ends** when the timeout passes, or at once when the identity provider
  definitively refuses a renewal (`invalid_grant`), or when an admin ends it.
* **Undecided answers end nothing early**: no network, Dex or the upstream unreachable,
  generic errors. The timeout limits how long such a state can last.
* **Enforcement:** the server refuses expired devices at connect and cuts open
  connections (`loginExpired`); every welcome tells the device when its login ends, so the
  device locks itself at that time, also offline. Logging in again with the same device key
  unlocks it; the data on the device stays.
* Devices paired by hand from the seed file (development only) have no end.

### Consequences

* Good, because access is always bounded, and a field deployment works for the configured
  time without internet.
* Good, because revocations and refusals by the identity provider still take effect.
* Bad, because a person disabled upstream keeps access until the timeout if the upstream
  cannot be asked; compensated by the bounded timeout, device keys that cannot be copied,
  the screen lock on devices with clinical data (SEC-13), local revocation and the audit
  trail.

## More Information

Dex behaviour: `server/grants/refresh.go` in Dex v2.46. Protocol: welcome `valid_until`,
rejection `loginExpired` ([spec/sync/protocol.md](../../spec/sync/protocol.md)).
