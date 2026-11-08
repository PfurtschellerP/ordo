# ORDO Requirements

What ORDO must do and how well. Every requirement is a **specification item** in the
[OpenFastTrace](https://github.com/itsallcode/openfasttrace) (OFT) Markdown format, so it
can be traced to the design that covers it and to the code and tests that implement it.

| File | Content |
|---|---|
| [01-vision-and-scope.md](01-vision-and-scope.md) | Vision, guiding principles, scope, users and roles |
| [02-shared.md](02-shared.md) | Shared modules: incidents, units, command, tasks, map, log, organisation |
| [03-fire.md](03-fire.md) | Fire package |
| [04-med.md](04-med.md) | Med package: patients and triage, UHS, ePCR, transport planning |
| [05-police-and-connectors.md](05-police-and-connectors.md) | Police package; connectors between packages |
| [06-federation-and-interfaces.md](06-federation-and-interfaces.md) | Federation; open interfaces |
| [07-quality.md](07-quality.md) | Usability, offline, performance, security, operation, internationalisation, modularity, accessibility, design system |
| [08-constraints.md](08-constraints.md) | Licence, law, regulation and other constraints |

Planning lives in the [roadmap](../roadmap.md), decisions in the [ADRs](../adr/README.md),
the solution in the [architecture](../architecture/README.md), terms in the
[glossary](../glossary.md).

## How a requirement looks

Schematic (placeholders in angle brackets; a real example is
[PAT-3](04-med.md#pat-3--re-triage-with-history)):

```markdown
#### <LABEL> — <Short title>
`req~<label in lower case>~<revision>`

<Requirement text in EARS form.>

Tags: <priority>, <area>[, <milestone>]

Needs: dsn
```

Do not write complete example IDs in documentation: OpenFastTrace would import them as
real items.

| Part | Meaning |
|---|---|
| Heading | Human-readable label (`PAT-3`) and a short title |
| ID line | `req~<name>~<revision>`: the OFT ID. The name is the label in lower case. Increase the revision when the meaning changes, so that everything that covered the old revision is flagged for review |
| Text | The requirement, written in EARS (below) |
| `Tags:` | Priority (`must`, `should`, `could`), area (`shared`, `fire`, `med`, `police`, `connectors`, `federation`, `interfaces`, `quality`) and milestone (`m0`, …) |
| `Needs:` | Which artifact types must cover it: normally `dsn` (a design item in the architecture) |

## Writing requirements: EARS

New and changed requirements use the **EARS** sentence patterns (*Easy Approach to
Requirements Syntax*). They make requirements unambiguous for people and machines.

| Pattern | Template | Example |
|---|---|---|
| Always | ORDO shall … | ORDO shall keep a list of hospitals with … |
| Event | **When** …, ORDO shall … | When a patient is re-triaged, ORDO shall … |
| State | **While** …, ORDO shall … | While an incident has registered patients, ORDO shall … |
| Unwanted | **If** …, **then** ORDO shall … | If a theme fails a contrast pair, then ORDO shall reject it |
| Optional | **Where** …, ORDO shall … | Where material tracking is switched on, ORDO shall … |

The Med requirements for the MVP (`PAT`, `UHS`, `TRANS`), the compartment rules and the
milestone-0 items are already in EARS form. Others are rephrased when they are next
changed.

## Traceability chain

```text
req  (requirements)      what is needed               Needs: dsn
 └─ dsn (architecture)   how ORDO solves it            Needs: impl, utest
     ├─ impl (code)      // [impl->dsn~<name>~<revision>]
     ├─ utest (unit tests)        // [utest->dsn~<name>~<revision>]
     └─ itest (acceptance tests)  // [itest->dsn~<name>~<revision>]
```

- Code and tests point to **design items**, not directly to requirements.
- Acceptance scenarios for key workflows are written in **Gherkin**
  ([acceptance/](acceptance/)) and become executable tests with the MVP.

## Milestones and CI

CI traces all items tagged with a milestone reached so far, plus everything without
tags (the coverage tags in code). Every item of the milestone must be fully covered, otherwise
the build fails.

| Milestone | Tag | Content |
|---|---|---|
| 0 — Foundation | `m0` | Globally unique IDs (FED-7), verifiable events (FED-6), design tokens and themes (DS-2, DS-3, DS-4, DS-6) |
| 1 — Sync prototype | `m1` | Compartment-scoped sync (SEC-9), sealed health data (SEC-10), automatic sync (OFF-3), no lost changes (OFF-4), triage history (PAT-3), patient counts for command (PAT-6), minimum connector data (CONN-3) |
| 2 — Server | `m2` | Configuration from the environment (OPS-7); store-before-send and wrapped keys strengthen OFF-3 and SEC-10 |

To add a requirement to the current milestone, add the milestone tag to it **and** to
the design items that cover it. Requirements and design items must always carry at
least a priority tag, so that items outside the milestone are not traced by accident.

Run locally (needs Java 17 or newer):

```sh
java -jar openfasttrace-4.11.0.jar trace -t _,m0,m1,m2 docs platform modules packages acceptance
```
