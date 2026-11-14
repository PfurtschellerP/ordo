---
status: accepted
date: 2026-11-14
---

# A guest's one-time code lasts 30 minutes, set per installation

Partly supersedes [ADR-0077](0077-guest-access-by-one-time-qr-code.md): how long a
guest's code works. Everything else in ADR-0077 stands.

## Context and Problem Statement

ADR-0077 let a guest's one-time code work for 15 minutes. At a registration desk the
command often makes the code, then registers the next helpers while the first one finds
their phone, gets onto the network and scans it. The product owner decided on 2026-11-14:
30 minutes by default, and each installation sets its own time.

## Considered Options

* 30 minutes by default, set by the server's configuration
* A fixed 30 minutes
* A time chosen with each code

## Decision Outcome

Chosen option: "30 minutes by default, set by the server's configuration":
`ORDO_GUEST_CODE_MINUTES`, from 5 to 1440 minutes (a day). Whoever runs the installation
weighs convenience against the risk of an unused code being seen; a single code cannot be
used twice, and access still ends with the duty. A time per code would add a field to a
form that should stay short.

### Consequences

* Good, because a helper has time to scan the code at a busy desk.
* Bad, because an unused code that someone else has seen works for longer; a new code
  makes it invalid at once.

## More Information

Design item *Guest access* in [chapter 8](../architecture/08-concepts.md#guest-access).
