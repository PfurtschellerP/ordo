---
status: accepted
date: 2026-11-09
---

# Triage algorithms are shown as reference, never computed

## Context and Problem Statement

Practitioners asked for the triage catalogues and algorithms (e.g. the BBK's pre-triage
algorithm PRIOR, mSTaRT) in the triage workflow. Software that derives a triage category
from entered findings supports a clinical decision; under the EU Medical Device Regulation
(Rule 11) it would be a medical device, probably class IIa, needing a quality management
system, a notified body and certification. ORDO's first rule is "no clinical decision
support" (AGENTS.md, [constraints](../requirements/08-constraints.md)).

## Considered Options

* A: the algorithm as a reference next to the input; the person chooses the category
* B: an interactive algorithm that proposes the category (ORDO becomes a medical device)

## Decision Outcome

Chosen option: "A", decided by the project lead on 2026-11-09, because it speeds up the
workflow without making ORDO a medical device.

* The profile names the algorithms in use (per state or organisation) and provides each
  as a reference (the official chart, with source and edition), shown next to the triage
  input and opened with one tap.
* The person chooses the category; ORDO never pre-selects, suggests or checks it.
* Each triage event records which algorithm was used ("Vorsichtung nach PRIOR"), as
  documentation.

### Consequences

* Good, because the workflow gets faster and better documented while ORDO stays outside
  the MDR.
* Bad, because ORDO cannot point out an inconsistent category; that stays the person's
  responsibility, as on paper.
