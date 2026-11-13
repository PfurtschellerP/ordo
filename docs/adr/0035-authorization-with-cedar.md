---
status: accepted
date: 2026-11-08
---

# Roles and permissions as Cedar policies

## Context and Problem Statement

Access must follow roles, never individual people ([ADR-0031](0031-command-structure-from-doctrine.md)).
Organisations must be able to define their own roles without code. The rules must be
enforced on the server for every event and every stream, also for offline devices that
sync later, and they must be auditable.

## Considered Options

* Own rule code in the hub
* Open Policy Agent (Rego; Go service or WebAssembly module)
* Cedar (Apache-2.0, written in Rust, CNCF Sandbox since October 2025)

## Decision Outcome

Chosen option: "Cedar", because it is an established open-source policy engine that
embeds directly in ORDO's Rust server (no extra service, fits the single binary), has a
small, readable language designed for authorisation, and validates policies against a
schema, so mistakes in organisation roles are caught before use.

* **Model.** Principal: a user in an incident, member of roles. Roles come from positions
  in the command structure and from organisational functions. Actions: read a
  compartment, record an event type, open a workspace, administrative actions.
  Resources: compartments, event types, workspaces of an incident.
* **Policies as data, in layers.** Packages ship default roles and policies (e.g. the Med
  package: medics may read and write `med.clinical`). Profiles and organisations add or
  change roles and policies in their layer (architecture 8.8). The schema comes from the
  packages; every layer is validated in CI and when loaded.
* **One enforcement point: the server.** The hub asks Cedar before it grants a stream,
  hands out a compartment key, or accepts an event. Rejections carry a reason.
* **Devices only mirror.** The server sends each device its effective permissions; the
  app uses them to show or disable menus and commands (ADR-0025). Cedar is not part of
  the WebAssembly bundle. Offline input that turns out not to be allowed is rejected when
  it syncs, with the reason shown.
* **Audit.** Each incident records the hash of the policy set it runs with; decisions can
  be replayed later.
* Open Policy Agent stays an option at the edges, for operators who already use it
  (e.g. Kubernetes admission), not inside the sync path.

### Consequences

* Good, because organisations get custom roles without code, and policies can be analysed.
* Good, because people never hold permissions directly; handing over a position moves
  its rights.
* Bad, because contributors learn a small policy language.

## More Information

Implemented with the login work (roadmap item 1). Sources: [Cedar](https://github.com/cedar-policy/cedar),
[CNCF project page](https://www.cncf.io/projects/cedar/).

### Implementation notes (2026-11-10)

* Write checks pass the event type next to the module (context `event_type`), so a
  policy can allow a module but restrict one kind of event in it: medical orders
  (`orders.given`) only from physicians and paramedics (physicians only since ADR-0053),
  acknowledging and completing them from the whole crew (architecture 8, "Order
  permission"). Reading and the view permissions pass an empty event type.
