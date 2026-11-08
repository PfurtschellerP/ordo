# ORDO Requirements — Functional requirements: Police package and connectors

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

### 4.13 Police package

#### POL-1 — Liaison view
`req~pol-1~1`

Liaison view: shared situation picture, cordons, access and exit routes, staging areas

Tags: should, police

Needs: dsn

#### POL-2 — Information for police
`req~pol-2~1`

Receive the information other packages share with police via connectors (e.g. number of injured, scene access)

Tags: should, police

Needs: dsn

#### POL-3 — Dedicated police modules
`req~pol-3~1`

Dedicated police modules only together with police partners

Tags: could, police

Needs: dsn

### 4.14 Connectors between service packages

#### CONN-1 — Only via connectors
`req~conn-1~1`

Service packages exchange information **only** through defined, versioned connectors; never by reading each other's internal data

Tags: must, connectors

Needs: dsn

#### CONN-2 — Connectors are documented
`req~conn-2~1`

Each connector states what data it shares, in which direction, and why; this list is visible to administrators

Tags: must, connectors

Needs: dsn

#### CONN-3 — Minimum necessary data
`req~conn-3~1`

Connectors shall share only the minimum data necessary; clinical details shall never leave the Med package.

Tags: must, connectors, m1

Needs: dsn

#### CONN-4 — Cross-service requests
`req~conn-4~1`

Requests across services (e.g. fire asks for medical care; med asks for technical rescue) are tasks with status visible to both sides

Tags: must, connectors

Needs: dsn

#### CONN-5 — First connectors
`req~conn-5~1`

First connectors: patient counts and transport status (Med → command); responder injured (Fire → Med); sectors, danger zones, staging areas (command → all); cordons and routes (command/police → all)

Tags: must, connectors

Needs: dsn
