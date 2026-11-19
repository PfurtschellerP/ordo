---
status: proposed
date: 2026-11-19
---

# A patient is registered where they are found and handed over between units

Would supersede in part [ADR-0082](0082-only-a-uhss-staff-writes-it.md) and
[ADR-0086](0086-the-incident-command-sets-up-the-uhs.md): who writes a patient.

## Context and Problem Statement

Today every patient belongs to a UHS from registration on, and only that UHS's staff
writes them (UHS-10). A crew that takes over a patient from an Auftrag in the field is on
no UHS staff list, so it cannot register the patient it is treating. The product owner
decided the direction on 2026-11-19: the person in the field registers the patient and
later hands them over, to a UHS, a transport or someone else. The patient record is one
record for the whole incident and travels with the patient. Within the incident the
handover happens in ORDO; to anyone outside it, ORDO produces a standardised handover
document.

## Considered Options

* The patient is with one **holder** at a time (a UHS, a unit's crew, a transport); the
  holder's people write the record; a handover moves the patient to the next holder
* Every medical role writes every patient (as before ADR-0056)
* Keep UHS-only patients and let crews register on behalf of a UHS

## Decision Outcome

Proposed: "One holder at a time", the product owner's direction of 2026-11-19.

* **Registration.** A patient is registered with a holder: the UHS the person works at,
  or the unit whose crew names them (UNIT-5), e.g. from a patient Auftrag (REP-3).
* **Writing.** The holder's people write the record: the staff list of a UHS, the crew
  of a unit, judged at the event's clock as today (ADR-0056); roles allowed at every UHS
  (`all_uhs`) still pass. Earlier holders keep reading it.
* **Handover.** `patients.handedOver` (to a UHS, a unit, a transport, or outside ORDO,
  with an optional note). The receiving side confirms with `patients.taken`; until then
  both see "Übergabe offen" (open question 1). A move between UHS becomes a handover.
* **Outside ORDO.** A handover document, made on the device, also offline (open question
  2).
* **Counts.** The patient counts for the command count patients per holder: per UHS as
  today, and "bei Einsatzmitteln" for patients with crews.

### Consequences

* Good, because nobody documents on paper first and types it in later.
* Good, because the record follows the patient, with every holder in its history.
* Bad, because the write guard and the counts must know units' crews as well as UHS
  staff lists.
* Bad, because events of today's patients (`uhs` field) need a reading as "held by that
  UHS"; no release has shipped them.

## Open questions for the product owner

1. Does the receiving side confirm a handover, or does the handover alone move the
   patient? Proposed: the receiver confirms, so responsibility never falls between two.
2. Format of the handover document: proposed a PDF/A-3 that people read, with the same
   data embedded machine-readably as an HL7 FHIR bundle, following DIVI MIND where it
   fits (roadmap, open question 3); to be checked against what hospitals and control
   centres accept.
3. Who may hand over to "outside ORDO": every holder, or only roles that end care (e.g.
   the transport's crew at the hospital)?

## More Information

Concept with screens: patient record overview, timeline and handover
(https://claude.ai/artifact/DuACJqM6EXsZKuXRh81xd6, 2026-11-19). [UHS-10](../requirements/04-med.md#uhs-10--only-a-uhss-staff-writes-its-patients),
[REP-3](../requirements/02-shared.md#rep-3--patient-from-a-patient-case), [roadmap, phase 3](../roadmap.md).
