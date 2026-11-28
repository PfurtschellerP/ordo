---
status: accepted
date: 2026-11-28
---

# The Auftrag card

## Context and Problem Statement

An Auftrag on *Disposition*, and a transport on *Transporte*, unfolded several things
inside their card: "Pin zeigen" opened a small map, and a second click closed it again;
"Pin verschieben", "Weiteres Einsatzmittel schicken" and "Ziel ändern" opened forms; the
"⋯" menu opened as a box in the card's flow. Each made the card jump, on a phone a card
with an open map filled the screen, and "Pin zeigen" and "Pin verschieben" were two buttons
for one thing (product owner, 2026-11-28). *Disposition* also listed the units with their
status, which *Lage – Einsatzmittel* shows with more since ADR-0099.

## Considered Options

1. One anatomy for every Auftrag and transport card: head, one line of facts, the objects
   as chips that open dialogs, one next step and a floating "⋯" menu; nothing opens inside
   the card.
2. Keep the unfolding parts and only tidy their buttons.

## Decision Outcome

Chosen: option 1, decided by the product owner on 2026-11-28 (concept *Struktur, Mittel,
Auftrag*, decision 4).

* **Chips:** 📍 the pin, 🏥 the destination. A chip shows the value and opens its dialog;
  dashed, it says what to set ("Pin setzen").
* **One pin dialog** shows the pin on a small map and moves or removes it; "Pin zeigen" and
  "Pin verschieben" become one. The crew keeps its small map in the card, read only, with
  the link to the maps app.
* **Assigning** ("Zuweisen", "Weiteres Einsatzmittel schicken") opens the units present
  that are not on the Auftrag, the available first, one button each.
* **The foot** has the one next step; the others are in the "⋯" menu, a popover over the
  page (a sheet at the bottom on phones); ending steps come last, set apart.
* **One dialog per list**, not per card, as for every edit dialog (chapter 8.6).
* *Disposition* gets the tabs *Aufträge* and *Personal anfordern*; the unit list leaves it,
  their status is set in *Lage*.
* The transport card follows the same anatomy in a second step: the destination as a chip
  with a dialog listing the hospitals with their reported free places.

### Consequences

* Good: a card keeps its size; one way to see and change each object; fewer buttons.
* Neutral: setting a unit's status needs *Lage* instead of *Disposition*.
* Bad: the popover needs a browser with the Popover API (all supported ones have it).

## More Information

* Concept *Struktur, Mittel, Auftrag* (2026-11-28), decision 4.
* Design items *Auftrag card*, *Tasks* and *Reports* (chapter 5).
* Builds on [ADR-0102](0102-building-blocks-screen-space-and-calm-pages.md) and
  [ADR-0085](0085-several-units-on-one-einsatzauftrag.md).
