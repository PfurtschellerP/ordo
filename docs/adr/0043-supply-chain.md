---
status: accepted
date: 2026-11-10
---

# Supply chain: bills of materials now, signed releases with provenance at the first release

## Context and Problem Statement

Emergency services run ORDO on their own servers and devices. They need to know what is
in it (to answer "are we affected?" when a dependency has a vulnerability) and that what
they run is what this repository built. The EU Cyber Resilience Act asks for a software
bill of materials and for secure updates; procurement increasingly asks for them too.

ORDO already has: dependencies pinned in `Cargo.lock`; `cargo deny` for licences and
advisories; Dependabot for crates, actions, images and npm; GitHub Actions pinned by
commit hash; container base images pinned by digest; REUSE for licences per file.

## Considered Options

* SPDX or CycloneDX as the format of the bill of materials
* Signing with long-lived keys, or keyless signing with Sigstore (cosign) and a public
  transparency log
* Build provenance as SLSA attestations (GitHub artifact attestations), or none

## Decision Outcome

* **Bill of materials: CycloneDX 1.6**, made with `cargo-cyclonedx` for what ships: the
  server and the web app. The container workflow makes them on every build, checks them
  (`tests/sbom/check_sbom.py`: format, the component, every dependency with version and
  package URL, the crates ORDO's security rests on) and keeps them as a build artifact.
  CycloneDX because security tools (e.g. Dependency-Track, OSV-Scanner) read it and it
  carries vulnerability information (VEX) as well.
* **At the first release:** images and binaries are signed keyless with cosign (Sigstore),
  with an SLSA build provenance attestation, and published with their bills of
  materials; the OpenSSF Scorecard runs on the repository. These publish entries to public
  services (the Sigstore transparency log, the Scorecard API), so they start when the
  repository becomes public and the project lead agrees; until then nothing leaves the
  private repository.
* Releases are built only by CI from a tag, never on a developer's machine.

### Consequences

* Good, because every build can say what it contains, and the check catches a broken or
  empty bill before anyone relies on it.
* Good, because operators will be able to verify a release without managing keys.
* Bad, because keyless signing ties releases to GitHub and Sigstore as services; the
  signature and the provenance can still be checked offline once downloaded.

## More Information

* [CycloneDX](https://cyclonedx.org) (ECMA-424), `cargo-cyclonedx`
* [Sigstore cosign](https://docs.sigstore.dev), [SLSA](https://slsa.dev),
  [OpenSSF Scorecard](https://scorecard.dev)
* Related: [ADR-0042](0042-post-quantum-readiness.md), [ADR-0032](0032-reuse-open-source-first.md)
