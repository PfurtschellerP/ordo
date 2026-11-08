---
status: superseded by ADR-0033
date: 2026-11-08
---

# Patient photos as small images inside sealed events

## Context and Problem Statement

A photo helps to identify patients in stressful situations, especially unknown or
unconscious ones (PAT-8). Photos are health-related personal data (GDPR Art. 9). They must
work offline, reach only authorised devices and never leave the clinical compartment.

## Considered Options

* Photos inside events of the protected compartment, downscaled on the device
* A separate encrypted file store with its own transfer protocol, referenced by hash from events
* No photos; a free-text description only

## Decision Outcome

Chosen option: "Photos inside events, downscaled on the device", because it reuses
sealing, need-to-know sync, offline storage and history without new infrastructure.

* The device downscales the image (longest side at most 800 px) and re-encodes it as
  JPEG. This removes camera and location metadata (EXIF).
* A photo event holds at most 160 KiB; the module rejects larger ones and active formats
  such as SVG.
* Photos are taken with the standard HTML media capture (`capture` attribute): no camera
  permission, also works as an upload on laptops.
* The record shows the latest photo; earlier photos stay in the history.

### Consequences

* Good, because photos are sealed, synced and kept like every other clinical entry.
* Bad, because every device with clinical access receives every photo of the incident;
  this limits the number of photos per incident.
* Bad, because the prototype's browser storage (localStorage, about 5 MB) fills up
  quickly. The app now warns when storage fails; moving local storage to IndexedDB is
  the next step (roadmap).
* Revisit with larger media (e.g. ECG strips, documents): then a separate file store with
  lazy loading is the better option.

## More Information

Module: `packages/med/patients/src/record.rs` (`PhotoAdded`), app:
`platform/ordo-app/src/patients.rs` (`downscale`).
