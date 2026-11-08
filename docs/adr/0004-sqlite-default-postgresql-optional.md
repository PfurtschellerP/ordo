---
status: proposed
date: 2026-11-07
---

# SQLite by default, PostgreSQL optional

## Context and Problem Statement

Volunteer organisations need an installation without extra services (OPS-1); hosted setups need to scale (OPS-9).

## Considered Options

* PostgreSQL only
* SQLite only
* SQLite by default, PostgreSQL optional, via sqlx

## Decision Outcome

Chosen option: "SQLite by default, PostgreSQL optional, via sqlx", because small installations need no database server, large ones can scale out.

### Consequences

* Good, because one binary is a complete installation.
* Bad, because two database dialects must be tested.

## More Information

Details: [07-deployment-view.md](../architecture/07-deployment-view.md)
