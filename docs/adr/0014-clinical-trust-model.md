---
status: proposed
date: 2026-11-08
---

# Trust model for clinical data

## Context and Problem Statement

Clinical data is encrypted per compartment; the question is whether the server may decrypt it.

## Considered Options

* Server-trusted: server can decrypt, logs every decryption
* End-to-end: only devices can decrypt

## Decision Outcome

Chosen option: "Server-trusted encryption first; end-to-end later with the same structure (open question)", because reports, PDF handover, FHIR export and backups need server access.

### Consequences

* Good, because server-side features work.
* Bad, because a compromised server exposes clinical data; to be reviewed with data-protection experts.

## More Information

Details: [08-concepts.md#82-data-compartments](../architecture/08-concepts.md#82-data-compartments)
