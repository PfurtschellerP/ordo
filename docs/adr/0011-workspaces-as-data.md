---
status: proposed
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

## More Information

Details: [05-building-block-view.md#53-service-packages-and-workspaces](../architecture/05-building-block-view.md#53-service-packages-and-workspaces)
