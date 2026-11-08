---
status: proposed
date: 2026-11-08
---

# Cloud-ready modular monolith

## Context and Problem Statement

ORDO starts small but should run in containers and clusters (OPS-5…11).

## Considered Options

* Microservices from the start
* Single binary only
* Modular monolith following twelve-factor rules, container image and Compose from day one, Helm later

## Decision Outcome

Chosen option: "Modular monolith following twelve-factor rules", because one program serves a Raspberry Pi and a cluster.

### Consequences

* Good, because operation stays simple.
* Neutral: roles of the binary can be split later if measurements demand it.

## More Information

Details: [07-deployment-view.md](../architecture/07-deployment-view.md)
