---
status: accepted
date: 2026-11-19
---

# One guest code pairs all of the guest's devices

Partly supersedes [ADR-0083](0083-guest-codes-last-as-long-as-the-access.md): a code no
longer works only once. Everything else in ADR-0083 stands.

## Context and Problem Statement

Under ADR-0083 a guest's code works until it is used or the access ends. In the demo of
2026-11-18 the product owner asked that a helper can pair all of their devices with one
QR code, for example a phone and a tablet. With a code that works once, the command has to
make a new code for every further device, and the new code makes the first one invalid
while the helper may still be scanning it.

## Considered Options

* A code pairs any number of the guest's devices until the access ends
* A code pairs up to a fixed number of devices
* Keep one device per code; the command makes a code per device

## Decision Outcome

Chosen option: "A code pairs any number of the guest's devices until the access ends",
decided by the product owner on 2026-11-19.

* Every device that pairs with the code gets the guest's ID, role and end of access.
* A new code makes the earlier one invalid, used or not. Devices paired before keep their
  access.
* Ending the duty, or closing the incident, ends the access of all the guest's devices,
  and the code stops working with it.
* The server still records when a code was first used.

### Consequences

* Good, because a helper pairs a phone and a tablet with the code they were given.
* Good, because the command does not have to make codes one by one.
* Bad, because a code that someone else has seen pairs further devices until the access
  ends. The command sees every paired device under *Geräte im Einsatz* in
  *Vorbereitung – Personal* and can block it; a new code stops the old one at once. Codes are still never logged,
  and the code is shown only on the command's device until closed.
* Neutral: no fixed number of devices, because any number would be arbitrary and a leaked
  code is better stopped by a new code than by a count.

## More Information

[HELP-7](../requirements/02-shared.md#help-7--access-without-an-account), design item
*Guest access* in [chapter 8](../architecture/08-concepts.md#guest-access).
