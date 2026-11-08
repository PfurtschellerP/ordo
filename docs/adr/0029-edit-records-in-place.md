---
status: accepted
date: 2026-11-08
---

# Edit records in place, not in modal dialogs

## Context and Problem Statement

Registering patients and documenting records are the main data-entry tasks. Many apps
open modal dialogs for such forms. ORDO is used under stress, with interruptions, on
phones and tablets, and by people using assistive technology.

## Considered Options

* Modal dialogs for every form
* Records as pages with their own address; forms open in place (inline), master–detail on wide screens
* Side sheets next to the list

## Decision Outcome

Chosen option: "Records as pages, forms in place", because:

* Modal dialogs block the context people need while documenting (the list, other
  sections of the record), and an accidental tap outside or a back gesture can discard
  input. Usability guidance reserves modal dialogs for short, critical interruptions.
* Long forms in dialogs are hard to use on phones and with screen magnification.
* A record with its own address (`#/med.uhs/patients/<tag>`) works with the back button and
  can be reopened after a reload.

Rules:

* Wide screens: list and record side by side (master–detail). Narrow screens: the record
  is a page of its own with a link back to the list.
* A form section shows its documented values; "Edit" or "New entry" opens the form in
  place; Save and Cancel close it and return focus to the button.
* Unsaved input stays per patient and form while the app runs, also when switching views
  (UX-5). It is not written to browser storage, because that storage is not encrypted.
* Quick registration stays a short form at the top of the list (PAT-1: about 10 seconds).
* Modal dialogs only for short tasks that need the whole attention: the command palette
  ([ADR-0030](0030-command-palette.md)) and, later, confirmations of irreversible actions.

### Consequences

* Good, because nothing is lost by a stray tap, and the back button behaves as expected.
* Good, because the same structure works on phone, tablet and laptop.
* Neutral: long records need a section index; the record page has one.

## More Information

Sources: [standards register](../standards.md#accessibility-and-user-interface).
App: `platform/ordo-app/src/patients.rs`.
