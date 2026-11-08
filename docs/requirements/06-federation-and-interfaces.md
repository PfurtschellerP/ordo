# ORDO Requirements — Functional requirements: federation and open interfaces

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

### 4.15 Federation

#### FED-1 — Federation without central server
`req~fed-1~1`

ORDO installations of different organisations can work together on one incident without a central server

Tags: could, federation

Needs: dsn

#### FED-2 — Closed by default
`req~fed-2~1`

Federation is **closed by default**: an installation only exchanges data with partners it has explicitly agreed with

Tags: could, federation

Needs: dsn

#### FED-3 — Federation agreements
`req~fed-3~1`

A federation agreement states which compartments and roles may cross to the partner

Tags: could, federation

Needs: dsn

#### FED-4 — Home installation per incident
`req~fed-4~1`

Each incident has a lead installation (*home*); the lead can be handed over (e.g. when a higher command level takes over); the handover is logged

Tags: could, federation

Needs: dsn

#### FED-5 — Partition tolerance
`req~fed-5~1`

If the connection between installations breaks, each side keeps working and they catch up automatically afterwards

Tags: could, federation

Needs: dsn

#### FED-6 — Verifiable origin and integrity
`req~fed-6~1`

When an installation receives an event, it shall be able to verify where the event comes from and that it is unchanged, also when another installation relayed it.

Tags: could, federation, m0

Needs: dsn

#### FED-7 — Globally unique identifiers
`req~fed-7~1`

ORDO shall make identifiers of users, organisations, incidents and events globally unique from the first release, so that installations can federate later without migrating data.

Tags: must, federation, m0

Needs: dsn

### 4.16 Open interfaces

#### API-1 — Open specifications
`req~api-1~1`

All external interfaces (client–server, sync protocol, federation, connector contracts, profile format) are published as open, versioned specifications

Tags: must, interfaces

Needs: dsn

#### API-2 — Independent spec versions
`req~api-2~1`

Specifications are versioned independently of the software; servers announce which versions they support

Tags: must, interfaces

Needs: dsn

#### API-3 — Permissive spec licence
`req~api-3~1`

Specifications are under a permissive licence — CC BY 4.0 for texts, Apache-2.0 for schemas and the conformance suite — so that other systems, including commercial ones, can implement them

Tags: must, interfaces

Needs: dsn

#### API-4 — Conformance test suite
`req~api-4~1`

A conformance test suite lets other implementations check that they follow the specification

Tags: should, interfaces

Needs: dsn
