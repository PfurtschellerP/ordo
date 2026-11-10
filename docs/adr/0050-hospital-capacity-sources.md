---
status: proposed
date: 2026-11-10
---

# Hospital capacity: entered by hand now, interfaces later

## Context and Problem Statement

The hospital list (TRANS-2, [design item](../architecture/08-concepts.md#hospital-list))
records the capacity a hospital reports per triage category. The incident command enters
it by hand. The project lead asked whether an open source of hospital capacity exists, so
ORDO could fetch it instead. ORDO never infers capacity from the transports it sent: a
hospital also takes walk-in patients, regular ambulance runs and patients of other
incidents, so "reported minus sent" would look exact and be wrong. The board shows both
numbers side by side instead.

What we found (research on 2026-11-10):

* **IVENA eHealth** (mainis IT-Service GmbH) is the capacity system most German regions
  use. Hospitals enter their treatment capacity, and dispatch centres see it in real time.
  A MANV module shows actual and target capacities. IVENA has an API: the vendor's change
  history names API versions, sessions created "via web or API", and a separate MANV
  capacity export. Its documentation is not public, and access needs an agreement with the
  vendor and the region. Third parties do connect to it; DIVERA 24/7, for example, sells
  an IVENA connection for hospital alerting.
* **DIVI Intensive Care Register** (DIVI, RKI): it covers intensive care beds only, and
  hospitals report to it once a day through a web form or an API. Public data is
  aggregated by district, state and country, with daily time series on GitHub and Zenodo.
  We found no public API that answers per hospital. It is too coarse and too slow for a
  MANV.
* **OASIS EDXL-HAVE 2.0** (Hospital AVailability Exchange) is an open XML standard for
  exactly this: facility status, capacity per service and emergency department state in
  emergencies. It was published jointly with HL7 as Committee Specification 02 (March
  2019). It is not an OASIS Standard, and we found no German use.
* **HL7 FHIR SANER** reports bed capacity as `MeasureReport` over `Location` (bed status
  occupied, available, not available). It came out of the US COVID-19 response, is not an
  authorised publication, and is continued in the US-only SAFR guide. It counts beds,
  not patients per triage category.
* **Hospital master data:** every licensed German hospital site has a six-digit site ID
  in the InEK site register (§ 293 (6) SGB V), kept on behalf of GKV-Spitzenverband and
  DKG. We found no stated licence for reuse of the register.

## Considered Options

* Keep entering capacity by hand, and plan interfaces for later
* Fetch from an open source now
* Estimate capacity from the transports ORDO sent

## Decision Outcome

Chosen option: keep entering by hand, and plan interfaces for later. There is no open,
live source of hospital capacity in Germany to fetch from. Estimating from transports is
ruled out (see the context). Later work, not a priority now:

1. **One event, many sources.** Any interface writes the same `hospitals.capacitySet` as
   the hand entry, from a technical user of the interface, so the board shows who
   reported the capacity and when. ORDO keeps its own model: patients per triage category,
   as MANV planning in Germany counts them.
2. **IVENA first.** It is the source the dispatch centres already trust. The interface is
   a connector of the Med package. It is built only once an organisation has an IVENA
   agreement that covers it and the API documentation is available. Its MANV capacities
   are the ones to read.
3. **Exchange format: EDXL-HAVE.** When ORDO exchanges hospital availability with other
   systems or federation partners, it uses EDXL-HAVE 2.0 rather than a format of its own,
   unless a partner already speaks FHIR. We confirm this choice with the first real
   partner, since the standard is still a Committee Specification.
4. **Master data.** Hospital entries get an optional identifier: the InEK site ID in
   Germany, from the profile. A hospital list kept by the organisation can then be
   imported, as HL7 FHIR `Organization`, `Location` and `HealthcareService` or as
   EDXL-HAVE, instead of being entered for every incident. We check the register's licence
   before importing from it.

### Consequences

* Good, because the board works today without any agreement, and an interface later
  changes no event, view or policy.
* Good, because no number on the board is a guess.
* Bad, because during a large incident someone must type the capacities that hospitals
  report by phone or that IVENA shows, until the interface exists.

## More Information

* IVENA eHealth: [modules](https://ivena.de/page.php?k1=main&k2=ueber&k3=pakete),
  [change history](https://ivena.de/download.php?k1=main&k2=download&k3=historie),
  [Lower Saxony ministry](https://www.ms.niedersachsen.de/krankenhaeuser/ivena_ehealth/ivena-ehealth-215526.html),
  [DIVERA IVENA connection](https://www.divera247.com/loesungen/im-gesundheitswesen/klinik/patientenzuweisung)
* DIVI Intensive Care Register: [documentation of the public data (Zenodo)](https://zenodo.org/records/13847967)
* [EDXL-HAVE 2.0, Committee Specification 02](https://docs.oasis-open.org/emergency/edxl-have/v2.0/edxl-have-v2.0.html)
* [HL7 FHIR SANER](https://hl7.org/fhir/uv/saner/introduction.html),
  [US SAFR](https://hl7.org/fhir/us/safr)
* [Hospital site register (GKV-Spitzenverband)](https://www.gkv-spitzenverband.de/krankenversicherung/krankenhaeuser/qualitaet_1/kh_standortverzeichnis/krankenhaus_standortverzeichnis.jsp)
* Related: [ADR-0022](0022-open-standards-first.md), [ADR-0024](0024-server-derived-connectors.md), TRANS-6 (no destination without a person's decision)
