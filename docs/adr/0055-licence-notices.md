---
status: accepted
date: 2026-11-10
---

# Licence notices: `cargo-about` for the crates, a list for the other parts

## Context and Problem Statement

ORDO ships hundreds of Rust crates in the server and the web app, plus map libraries,
fonts, icons, tactical symbols and map data. Most of their licences (MIT, BSD, Apache-2.0,
OFL) require their copyright and licence notice to go with every copy. The AGPL asks ORDO
to offer its source code to everyone who uses it over a network (section 13). Users need
one place for both (LIC-1).

## Considered Options

* `cargo-about` at build time, plus a short hand-kept list of the non-Rust parts
* A list written by hand
* The CycloneDX bill of materials ([ADR-0043](0043-supply-chain.md)) as the source

## Decision Outcome

Chosen option: "`cargo-about` plus a short list", because `cargo-about` (MIT/Apache-2.0,
maintained by Embark Studios) is the established tool: it reads the dependency tree and
collects each crate's own licence text, copyright lines included. A hand-written list
would go stale with every update. The bill of materials names licences but carries no
texts.

* `about.toml` accepts the same licences as `deny.toml`. It leaves out development and
  build dependencies and ORDO's own crates.
* `tools/notices/notices.sh` runs `cargo-about` for the main workspace and for the storage worker.
  The web app's pre-build step calls it, and CI and release builds fail without it. Without
  it, a local build gets an empty list, and the page says so.
* The non-Rust parts are a list in the app, kept next to the vendoring scripts. Their
  licence texts come from `LICENSES/`, which the app serves.

### Consequences

* Good, because the notices follow every dependency update by themselves.
* Good, because the notices load only when the page opens.
* Bad, because a full build needs one more pinned tool.
* Bad, because the list of non-Rust parts must be updated by hand when one is added.

## More Information

[cargo-about](https://github.com/EmbarkStudios/cargo-about),
[08-concepts.md](../architecture/08-concepts.md),
[GNU AGPL section 13](https://www.gnu.org/licenses/agpl-3.0.html#section13).
