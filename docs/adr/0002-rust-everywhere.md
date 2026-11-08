---
status: accepted
date: 2026-11-07
---

# Use Rust for server and client

## Context and Problem Statement

ORDO should waste no resources (PERF-1, PERF-4, PERF-5) and client and server must agree on the state.

## Considered Options

* TypeScript on client and server
* Rust on the server, TypeScript in the client
* Rust everywhere; client compiled to WebAssembly

## Decision Outcome

Chosen option: "Rust everywhere; client compiled to WebAssembly", because it is fast and memory-efficient without a garbage collector, and the domain code is written once for client and server.

### Consequences

* Good, because domain logic, sync and validation are shared crates.
* Good, because memory safety without a runtime fits small devices.
* Bad, because fewer contributors know Rust UI development than JavaScript.

## More Information

Details: [04-solution-strategy.md](../architecture/04-solution-strategy.md)
