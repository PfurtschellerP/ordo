---
status: accepted; partly superseded by ADR-0084
date: 2026-11-18
---

# A guest's code works as long as the access it gives

Supersedes [ADR-0078](0078-guest-codes-last-30-minutes-set-per-installation.md). Partly
supersedes [ADR-0077](0077-guest-access-by-one-time-qr-code.md): how long a guest's code
and access last. Everything else in ADR-0077 stands.

## Context and Problem Statement

ADR-0078 let a guest's one-time code work for 30 minutes, set per installation, and
ADR-0077 required an end of duty for every guest. In the demo of 2026-11-17 the product
owner found both getting in the way. Helpers get their code at the desk and scan it
later, and a helper often starts without a known end of duty. Starting a duty cleared its
end, so the QR code could not be made again at all.

## Considered Options

* The code works until the guest's access ends: the end of duty, or the close of the
  incident where no end is known
* Keep 30 minutes, but allow a guest without an end of duty
* Keep ADR-0077 and ADR-0078 and fix only the cleared end of duty

## Decision Outcome

Chosen option: "The code works until the guest's access ends", decided by the product
owner on 2026-11-17.

* A guest's access ends at their end of duty. Without one, it lasts until the incident is
  closed (`UNTIL_CLOSED` in the sync protocol, the largest time the server stores). The
  server ends the access of every guest of a closed incident within a minute.
* A code works until it is used or the access ends. A new code still makes the unused
  one invalid at once. `ORDO_GUEST_CODE_MINUTES` is gone.
* Ending the duty ends the access, as before. Starting a new duty gives it back, until
  the new end or the close of the incident.

### Consequences

* Good, because a helper can scan their code whenever they are ready.
* Good, because helpers without a planned end of duty get access too.
* Bad, because an unused code that someone else has seen works for longer. Making a new
  code, or ending the duty, makes it invalid. Codes are never logged, and the code is
  shown only on the command's device until closed.

## More Information

[HELP-7](../requirements/02-shared.md#help-7--access-without-an-account), design item
*Guest access* in [chapter 8](../architecture/08-concepts.md#guest-access).
