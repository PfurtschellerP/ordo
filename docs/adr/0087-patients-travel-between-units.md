---
status: accepted
date: 2026-11-20
---

# A patient is registered where they are found and handed over between units

Supersedes in part [ADR-0082](0082-only-a-uhss-staff-writes-it.md) and
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

Chosen option: "One holder at a time", the product owner's direction of 2026-11-19,
accepted with the answers of 2026-11-20.

* **Registration.** A patient is registered with a holder: the UHS the person works at,
  or the unit whose crew names them (UNIT-5), e.g. from a patient Auftrag (REP-3).
* **Writing.** The holder's people write the record: the staff list of a UHS, the crew
  of a unit, judged at the event's clock as today (ADR-0056); roles allowed at every UHS
  (`all_uhs`) still pass. Earlier holders keep reading it.
* **Handover.** `patients.handedOver` (to a UHS, a unit, a transport, or outside ORDO,
  with an optional note). The receiving side confirms with `patients.taken`; until then
  both see "Übergabe offen", so responsibility never falls between two. A move between
  UHS becomes a handover.
* **Quick handover by QR code** (product owner, 2026-11-20). The holder chooses
  *Schnellübergabe*: the device records an open handover to nobody in particular and
  shows a QR code. It carries only the incident, the triage tag and the handover's ID,
  never clinical data. The receiver photographs it in ORDO (rqrr, as the registration
  cards, ADR-0073), chooses where the patient goes among the UHS and units they may write
  for, and confirms; that records `patients.taken` with the chosen holder. Their device
  must already hold the incident and the clinical compartment; the code grants nothing by
  itself. It works offline on both sides; the server checks the taking when it arrives.
* **Outside ORDO.** A handover document, made on the device, also offline: a PDF/A-3
  that people read, with the same data embedded machine-readably as an HL7 FHIR bundle,
  following DIVI MIND where it fits; to be checked against what hospitals and control
  centres accept before it is built.
* **Who hands over outside ORDO.** Every holder (a field crew may hand a patient straight
  to an ambulance from outside the incident). Proposed by the developers on 2026-11-20,
  confirmed by the product owner on 2026-11-21, with PDF/A-3; the document's template is
  [ADR-0089](0089-handover-document-from-typst-templates.md).
* **Counts.** The patient counts for the command count patients per holder: per UHS as
  today, and "bei Einsatzmitteln" for patients with crews.

### Consequences

* Good, because nobody documents on paper first and types it in later.
* Good, because the record follows the patient, with every holder in its history.
* Bad, because the write guard and the counts must know units' crews as well as UHS
  staff lists.
* Bad, because events of today's patients (`uhs` field) need a reading as "held by that
  UHS"; no release has shipped them.
* Bad, because a QR code shown on a screen can be photographed by anyone nearby; it
  only lets someone who may already write patients take this one, and the log shows who
  took it.

## More Information

Concept with screens: patient record overview, timeline and handover
(https://claude.ai/artifact/DuACJqM6EXsZKuXRh81xd6, 2026-11-19). [UHS-10](../requirements/04-med.md#uhs-10--only-a-uhss-staff-writes-its-patients),
[REP-3](../requirements/02-shared.md#rep-3--patient-from-a-patient-case), [roadmap, phase 3](../roadmap.md).
