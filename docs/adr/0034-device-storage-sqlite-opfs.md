---
status: proposed
date: 2026-11-08
---

# Device storage: SQLite on the browser's file system, in a worker

## Context and Problem Statement

The prototype writes the whole replica as one string to `localStorage` after every
change. That storage holds about 5 MB, is synchronous and rewrites everything each time.
Devices must keep a whole incident, with attachments, for days, offline.

## Considered Options

* `localStorage` (prototype)
* IndexedDB through a maintained crate
* SQLite compiled to WebAssembly (`sqlite-wasm-rs`) on the Origin Private File System (OPFS), in a Web Worker; blobs as OPFS files

## Decision Outcome

Proposed option: "SQLite on OPFS in a worker", because it reuses SQLite, the same
database the server uses by default: same schema and queries, append-only writes,
storage in the gigabytes, and a mature engine instead of our own snapshot format.

* A dedicated Web Worker owns the replica: storage, sync connection and encryption. The
  UI talks to it by messages, so heavy work never blocks the screen.
* SQLite runs with the OPFS "sync access handle pool" VFS, which needs no special server
  headers. Where OPFS is missing, the IndexedDB VFS of the same crate is the fallback.
* Attachments are OPFS files, named by their hash ([ADR-0033](0033-attachments-in-an-object-store.md)).
* The app asks for persistent storage (`navigator.storage.persist()`) and shows free space.

A short spike decides before the switch: bundle size, Safari/iPadOS support, write
speed, and the worker boundary with Leptos. If the spike fails, IndexedDB is used.

### Consequences

* Good, because storage no longer limits incident size, and nothing is rewritten.
* Good, because data at rest can be encrypted page by page later, with keys from WebCrypto.
* Bad, because the worker boundary adds message passing between UI and replica.

## More Information

Roadmap item 1. Replaces the local-persistence design of the prototype
(architecture 5.5.3).
