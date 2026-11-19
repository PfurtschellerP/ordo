---
status: accepted
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

### Design (accepted 2026-11-19)

The product owner accepted this decision on 2026-11-19, with a registry as its design, so
that a new package changes no shared file (MOD-3):

* **Event kinds.** Each module declares every kind of event it writes once, next to the
  payload: compartment, module, event type and the payload's schema version
  (`ordo_sync::EventKind`). Writers record by kind; no shared table lists them.
* **Packages in the app.** Each package describes itself in one entry of the app's
  registry: its workspaces and views, the page of each view, its quick actions, its bell
  items, its log lines and its texts. The shell loops over the registered packages. The
  app's source is in folders (`shell/`, `shared/`, one per package); a crate per package
  follows once a second package exists.
* **Packages on the server.** Each package brings its policies, derivers and write guards;
  the server loops over the packages.

Design item: [Package registry](../architecture/05-building-block-view.md#package-registry).

## More Information

Details: [05-building-block-view.md](../architecture/05-building-block-view.md)
