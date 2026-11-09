---
status: accepted
date: 2026-11-09
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

Chosen option (accepted by the project lead on 2026-11-09): "SQLite on OPFS in a worker", because it reuses SQLite, the same
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

### Spike results (2026-11-09)

* **Building blocks** (all open source): [`rusqlite`](https://github.com/rusqlite/rusqlite)
  0.41 (MIT) on `wasm32-unknown-unknown` via
  [`sqlite-wasm-rs`](https://github.com/Spxg/sqlite-wasm-rs) 0.6 (MIT, SQLite compiled to
  WebAssembly) and `sqlite-wasm-vfs` 0.3 with the `sahpool` VFS (OPFS sync access handles,
  in a dedicated worker; no special HTTP headers needed). `sqlite-wasm-rs` can also build
  SQLite3 Multiple Ciphers (MIT) to encrypt the whole database with the storage key of
  ADR-0041.
* **Works** in Firefox and Chromium: a worker opens a database in OPFS, writes, and finds
  the rows again after a reload. WebKit (Safari, iPadOS) is to be confirmed in CI, where
  WebKit runs.
* **Size:** SQLite adds 1.08 MB of WebAssembly (484 KB compressed) to today's app (1.35 MB,
  567 KB compressed): about 1 MB compressed in total, cached once by the offline app.
* **Build:** compiling SQLite for WebAssembly needs `clang`; GitHub's runners have it,
  developers install it (`dnf install clang`, `apt install clang`).
* **Open point:** `sqlite-wasm-rs` 0.6.2 bundles SQLite 3.53.4, before the fix of
  RUSTSEC-2026-0459. On the device the database lives in the app's own OPFS area, which
  only the app's own code can write; update as soon as a release bundles 3.54.
* **Decision:** go ahead with SQLite on OPFS (no IndexedDB fallback needed for the
  browsers tested).

