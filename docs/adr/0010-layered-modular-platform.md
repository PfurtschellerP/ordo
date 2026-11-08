---
status: proposed
date: 2026-11-08
---

# Platform, shared modules and service packages

## Context and Problem Statement

Fire, Med and Police must share one platform but stay independently extensible (MOD-1…7).

## Considered Options

* One application with feature flags
* Microservices per service
* Layered modular monolith: platform → shared modules → service packages; compiled-in modules

## Decision Outcome

Chosen option: "Layered modular monolith with compiled-in modules", because it keeps one binary for small installations while enforcing module boundaries.

### Consequences

* Good, because dependencies only point downwards.
* Bad, because runtime plugins by third parties are not possible yet.

## More Information

Details: [05-building-block-view.md](../architecture/05-building-block-view.md)
