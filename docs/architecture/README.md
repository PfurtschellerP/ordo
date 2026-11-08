# ORDO Architecture

The architecture follows the [arc42](https://arc42.org) template (widely used in
German-speaking countries), one file per chapter. Diagrams follow the
[C4 model](https://c4model.com) and are written in Mermaid, which GitHub renders directly.

| Chapter | Content |
|---|---|
| [1. Introduction and goals](01-introduction-and-goals.md) | Quality goals, stakeholders, architecture drivers |
| [2. Constraints](02-architecture-constraints.md) | Licence, law, technology constraints |
| [3. Context and scope](03-context-and-scope.md) | External partners; devices and servers |
| [4. Solution strategy](04-solution-strategy.md) | Key ideas, technology choices, **open standards in use** |
| [5. Building block view](05-building-block-view.md) | Layers, components, packages and workspaces, repository |
| [6. Runtime view](06-runtime-view.md) | Offline sync, connectors, concurrent changes |
| [7. Deployment view](07-deployment-view.md) | Single binary, Compose, Kubernetes, scaling |
| [8. Crosscutting concepts](08-concepts.md) | Events, compartments, connectors, federation, security, design system and accessibility, open specifications, regional profiles |
| [9. Architecture decisions](09-architecture-decisions.md) | Pointer to the [ADRs](../adr/README.md) |
| [10. Quality requirements](10-quality-requirements.md) | Quality scenarios, resource budget |
| [11. Risks and technical debt](11-risks-and-technical-debt.md) | Known risks and mitigations |
| [12. Glossary](12-glossary.md) | Pointer to the [glossary](../glossary.md) |

## Design items

Where a concept is specified precisely enough to be implemented, it contains
**design items** in OpenFastTrace format (artifact type `dsn`). Each design item covers
one or more requirements and needs coverage by code (`impl`) and tests (`utest`); see
[requirements/README.md](../requirements/README.md#traceability-chain).
