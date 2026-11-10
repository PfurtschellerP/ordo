---
status: accepted
date: 2026-11-10
---

# OpenSpec is not adopted; requirements stay in OpenFastTrace

## Context and Problem Statement

[OpenSpec](https://openspec.dev) (Fission AI, MIT) is a spec-driven development framework
for AI coding assistants: current specifications in `openspec/specs/<area>/spec.md`
("Requirement … SHALL …" with Given/When/Then scenarios), and per change a folder with
`proposal.md`, `design.md`, `tasks.md` and specification deltas (added, modified, removed
requirements) that are merged into the specifications when the change is archived. Does
it help ORDO, which is largely built with AI assistants (ADR-0032: reuse before building)?

## Considered Options

* Adopt OpenSpec for requirements and changes
* Adopt only its change workflow (proposal, design, tasks) next to the existing documents
* Keep the existing chain: requirements (EARS, OpenFastTrace) → design items (arc42) → code
  and tests, decisions as MADR, roadmap

## Decision Outcome

Chosen option: "Keep the existing chain", because ORDO already does spec-driven
development, with more than OpenSpec checks:

* **Traceability in CI.** Every requirement of a milestone must be covered by a design item,
  code (`impl`) and tests (`utest`, `itest`); OpenFastTrace fails the build otherwise.
  OpenSpec's verification is a review by the assistant, not a gate.
* **One source of truth.** OpenSpec specifications would duplicate the requirements and
  design items; two stores of the same requirements drift apart.
* **Decisions and their reasons** live in ADRs (MADR), which OpenSpec's `design.md` per
  change would scatter.
* **No extra toolchain** (a Node CLI) in the documentation path.

What ORDO takes from it: a larger change starts with its requirement and design item (or
an ADR) before the code, and AGENTS.md tells assistants so. Revisit if the project grows
many parallel contributors who need per-change proposals.

### Consequences

* Good, because requirements stay single-sourced and traced in CI.
* Bad, because there is no per-change folder that collects proposal and tasks; the commit
  history and the roadmap serve that purpose.
