---
status: accepted
date: 2026-11-08
---

# Role-based workspaces defined as data

## Context and Problem Statement

Each role should see only what it needs, in its service's wording (UX-8).

## Considered Options

* Fixed screens per role in code
* Workspaces defined as data in packages and profiles

## Decision Outcome

Chosen option: "Workspaces defined as data", because organisations can adapt them without code.

### Consequences

* Good, because new roles need no release.
* Bad, because the workspace format needs validation.

### Design (accepted 2026-11-19)

Accepted by the product owner on 2026-11-19. A package's workspaces and views are data in
its entry of the app's registry ([ADR-0010](0010-layered-modular-platform.md)): ID, label,
icon, the compartments and permission they need, their views in order. The shell builds
the launcher, the navigation and the routes from them. Profiles may override labels and
order later; then the format gets its validation in `ordo-profiles`.

## More Information

Details: [05-building-block-view.md#53-service-packages-and-workspaces](../architecture/05-building-block-view.md#53-service-packages-and-workspaces)
