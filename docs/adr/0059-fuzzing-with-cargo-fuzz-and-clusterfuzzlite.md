---
status: accepted
date: 2026-11-11
---

# Fuzzing with cargo-fuzz, run continuously by ClusterFuzzLite

## Context and Problem Statement

The server decodes events and protocol messages from every device, and devices decode
what the server and other devices send. A decoder that panics, loops or allocates without
bound on hostile input would take a server down during an incident (SEC-17, ASVS V5). Unit
tests cover the inputs we think of; fuzzing finds the ones we do not.

## Considered Options

* cargo-fuzz (libFuzzer) targets, run in CI by ClusterFuzzLite
* Property tests only (proptest) on stable Rust
* OSS-Fuzz (Google's hosted fuzzing for open source)

## Decision Outcome

Chosen option: "cargo-fuzz, run by ClusterFuzzLite", decided on 2026-11-11, because:

* cargo-fuzz is the Rust project's standard (rust-fuzz, MIT/Apache-2.0).
* ClusterFuzzLite (Google, Apache-2.0) runs the same targets in GitHub Actions. It keeps a
  corpus between runs and reports crashes with the input. It needs no account or outside
  service.
* OpenSSF Scorecard recognises it.
* OSS-Fuzz needs an application and suits projects with many users; it can follow later
  with the same targets.

Property tests would only check what we describe and would not grow a corpus.

* **Targets** in `tests/fuzz/` (its own workspace, nightly Rust): the protocol messages in both
  directions, signed events and their bodies, and the payloads of every event type.
  Without panics is not enough: every decoded value must encode again.
* **When:** a short run on every push to `main` and on pull requests, and a longer batch
  run every day. A crash fails the run and the input is kept as an artifact.
* **Seeds:** valid messages and events written by a small program from the real types, so
  fuzzing starts from realistic input.

### Consequences

* Good, because the decoders are checked continuously against input no one wrote by hand.
* Bad, because fuzzing needs a nightly toolchain, limited to the `tests/fuzz/` workspace.

## More Information

[cargo-fuzz](https://github.com/rust-fuzz/cargo-fuzz),
[ClusterFuzzLite](https://google.github.io/clusterfuzzlite/),
[08-concepts.md](../architecture/08-concepts.md).
