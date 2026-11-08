---
status: accepted
date: 2026-11-08
---

# Docs-as-code with arc42, MADR and OpenFastTrace

## Context and Problem Statement

Requirements, architecture and decisions must be usable by people (contributors, test
partners, auditors) and by AI agents, and must stay in step with the code.

## Considered Options

* Two free-form Markdown documents
* Wiki or external tools (Confluence, Notion, Jira, DOORS)
* Docs-as-code in the repository: arc42 architecture, MADR decisions, OpenFastTrace requirements written in EARS, Gherkin acceptance scenarios, a glossary and AGENTS.md

## Decision Outcome

Chosen option: "Docs-as-code in the repository", because every change is reviewed in
pull requests, CI checks that requirements are designed, implemented and tested, and
agents load exactly the small, structured files they need.

### Consequences

* Good, because traceability (requirement → design → code → test) is checked in CI.
* Good, because the formats are open and widely known (arc42, MADR, EARS, Gherkin, AGENTS.md).
* Bad, because tracing needs Java in CI (OpenFastTrace).
* Neutral: StrictDoc remains an option if requirements must be exchanged as ReqIF.

## More Information

Details: [requirements/README.md](../requirements/README.md), [architecture/README.md](../architecture/README.md)
