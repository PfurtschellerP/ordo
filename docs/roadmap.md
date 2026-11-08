# ORDO Roadmap and open questions

> Moved from the requirements. Phases are planning, not commitments.

## Roadmap (draft)

| Phase | Content |
|---|---|
| **0 — Foundation** | Repository, CI, container image, Docker Compose, architecture decisions, platform core (events, sync, compartments, modules, profiles, global IDs, signed events), sync prototype, first spec drafts, design tokens and themes, accessibility testing in CI |
| **1 — MVP (Med)** | Shared modules (incidents, units, command, tasks, map, log, org) + **Med package: patients and triage, UHS, transport planning, hospitals** (no ePCR yet); planned operations; offline; one organisation; field tests at planned events |
| **2 — Fire** | Fire package; first connectors between Fire and Med; field tests with fire brigades |
| **3 — Patient records** | ePCR with clinical compartment, PDF handover; data-protection review |
| **4 — Connected** | Federation between installations, alert import, local server in the command vehicle, breathing-apparatus monitoring, conformance tests |
| **5 — Professional** | Single sign-on, Helm chart, hosted service, control-centre interfaces (EMSI), hospital interfaces (FHIR, IVENA), Matrix adapter, police modules with partners |

## Open questions

1. Which open tactical-symbol set (DV 102) can be used, and under which licence?
2. Which organisations in Germany could be early test partners — first medical services
   for planned events (e.g. DRK, ASB, JUH, MHD), then fire brigades?
3. Which DIVI MIND version should the ePCR be based on, and is its use licence-free?
4. Who can review data protection and medical-device questions before field use?

## Next steps


1. Phase 0: set up the Cargo workspace, CI (build, tests, multi-arch container image)
   and the minimal Compose setup; write ADRs 001–021.
2. Design tokens (primitive, semantic, safety) with day, night and high-contrast
   themes; `ordo-tokens` with contrast checks in CI.
3. Draft the event envelope and sync protocol as the first specs (CDDL + AsyncAPI).
4. Sync prototype covering the risky parts: `ordo-core` + `ordo-sync` + `ordo-crypto`,
   **two compartments** (`incident`, `med.clinical`), signed events with qualified IDs,
   one shared module (`units`), one package module (`med.patients`), one connector
   (`med.patientCounts.v1`), a minimal DE profile, minimal server and Leptos app, run
   via Docker Compose. Test: a command tablet sees counts but never receives patient
   data; network on/off; conflicting edits; two server instances on PostgreSQL. Measure
   size and memory.
5. Design the main tablet screens for the Med MVP (patient registration, UHS board,
   transport board) with volunteers.
