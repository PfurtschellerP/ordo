---
status: accepted
date: 2026-11-13
---

# Orders given by voice or radio are recorded by the crew and confirmed later

## Context and Problem Statement

[ORDER-1 to ORDER-4](../requirements/04-med.md#4111-med-package-medical-orders) let
physicians and paramedics give orders on the patient record; the server accepted
`orders.given` only from these roles ([ORDER-3](../requirements/04-med.md#order-3--only-permitted-roles-give-orders),
design item *Order permission*). In practice many orders are spoken: the emergency physician at
the stretcher with gloves on, the medical lead (LNA) or a tele-emergency physician by
radio or phone. The crew carries them out at once and must document them with who gave
them. Today the crew cannot record such an order, because the server rejects an
`orders.given` from a medic, and a physician outside ORDO cannot write one at all.

The server cannot read clinical payloads (they are sealed, design item *Sealed payloads*); it
decides on the author's roles and the event type only.

## Considered Options

* Let medics write `orders.given` on behalf of the physician
* A second event type for orders the crew records, with who gave them and how, confirmed
  later by the person who gave them where that person uses ORDO
* Leave verbal orders to free text in the record

## Decision Outcome

Chosen option: the second one, chosen on 2026-11-12 with the product owner after a
practitioner's test of the demo, where anyone documenting a medication could type any
physician as "Angeordnet durch", so documented orders did not show who really gave them.
Accepted on 2026-11-13, after the legal check below, with the product owner's answers to
its two open points (*Decisions on the open points*).

* **`orders.recorded`** (module `med.orders`, compartment `med.clinical`): the same
  content as `orders.given` (what, for which patient, note) plus `ordered_by` (a person
  of the incident or free text, e.g. "Telenotarzt Dr. M."), `channel` (in person, radio,
  phone, telemedicine) and `read_back` (whether the order was repeated back and
  confirmed, closed-loop communication). Every crew role that writes clinical data may
  write it; the author is the person who recorded it.
* **The order then follows the same steps** (acknowledged, done, withdrawn), is listed
  with the open orders, and can be taken into the documentation (ORDER-5). The app shows
  it apart: "mündlich angeordnet von … · dokumentiert von …", with the channel.
* **Confirmation:** `orders.confirmed` (order ID) may be written only by roles that give
  orders (Cedar, like `orders.given`; by default physicians, see below). The app offers it to the person named in
  `ordered_by` when that person uses ORDO; the record shows who confirmed and when.
  Unconfirmed recorded orders stay marked, and the lead's overview counts them. The
  server cannot check that the confirmer is the person named, since it cannot read the
  payload; the record shows both names, so a mismatch is visible.
* `orders.given` keeps its event and its rule: giving an order in ORDO yourself still
  needs a role that gives orders.
* **Who ordered a documented medication** comes only from an order, given or recorded: the
  entry keeps the order's ID, and the form's field for who ordered is read-only
  ([ORDER-7](../requirements/04-med.md#order-7--who-ordered-comes-from-the-order)).
* ORDO records and shows; it never judges whether an order was permitted, needed or
  correct (EPCR-9).

### Legal check (Germany, federal law, 2026-11-12)

Checked against the current federal texts; not legal advice. State law (rescue service
acts, the ÄLRD's instructions) can add rules and is still to be checked with an ÄLRD.

A paramedic (Notfallsanitäter) carries out a medical measure on one of three grounds:

1. **On their own responsibility** until a physician arrives or other medical care,
   telemedical included, begins (§ 2a and § 4 (2) no. 1 c NotSanG). No order exists; the
   measure is documented without one. This fits ORDER-7.
2. **Ordered by a physician in the individual case** ("ärztlich veranlasst", § 4 (2)
   no. 2 b NotSanG). The order may come from the emergency physician on scene or from a
   tele-emergency physician; the latter is a delegation like one on scene, with
   supervision by continuous contact (guidance of the Bavarian interior ministry). This is
   what `orders.given` and `orders.recorded` document; the channel covers telemedicine.
3. **Standing orders of the ÄLRD** (SOPs, § 4 (2) no. 2 c NotSanG; for narcotics § 13 (1b)
   BtMG, the SOP in text form). No individual order exists; the ground is the SOP.

Findings:

* **Recording spoken orders fits the law.** Delegation by voice, radio or telemedicine is
  lawful, and the delegating physician keeps the responsibility for the order, the crew
  for carrying it out. Documenting who ordered, how, and whether it was read back serves
  both.
* **Documentation:** records are kept close in time to the treatment, and changes keep
  the original visible with their date (model: § 630f BGB). Immutable events and
  correction events meet this.
* **Confirmation is not required by federal law.** No federal rule asks the physician to
  countersign a spoken order. The confirmation step is an extra safeguard; it must never
  hold up carrying out an order, and it does not.
* **Open: who gives and confirms orders.** Under federal law a physician (on scene, by
  telemedicine, or the ÄLRD by SOP) delegates medical measures; a paramedic does not
  delegate them to others. The default rule lets physicians *and paramedics* give and
  confirm orders (ORDER-3). That fits orders that are not medical measures (e.g.
  transport, positioning), but not medication ordered by a paramedic for another crew
  member. To be decided: physicians only by default, a profile setting, or kinds of
  orders.
* **Open: SOP as the ground.** A measure under an ÄLRD SOP is documented without an
  order; the record cannot yet say that its ground is an SOP (and which one) rather than
  the paramedic's own responsibility. To be decided with the profile's ÄLRD.
* **Narcotics:** the narcotics register (stock and use, §§ 13–14 BtMVV) is kept by the
  physician appointed for the rescue service. ORDO's patient record does not replace it.

### Decisions on the open points (2026-11-13)

Decided by the product owner:

* **Only physicians give and confirm orders** by default. The Med policies accept
  `orders.given` and `orders.confirmed` only from the role `physician`; paramedics, like
  the rest of the crew, record spoken orders (`orders.recorded`), acknowledge and carry
  out orders. This follows federal law: a physician delegates medical measures, a
  paramedic does not. Orders that are not medical measures (transport, positioning) are
  not split off as their own kind for now. An organisation whose state law or ÄLRD allows
  more changes its own policy files ([ADR-0035](0035-authorization-with-cedar.md)); ORDO
  has no setting per kind of order.
* **No SOP catalogue.** Keeping every area's SOPs in ORDO, with who may do what, when and
  where, is not worth it. A measure under an SOP or on the paramedic's own responsibility
  is documented without an order, as before; the record does not name its ground, and the
  note can.

Still to do with practitioners and an ÄLRD: the fields and the confirmation step, and the
state law of the first users.

### Consequences

* Good, because spoken orders are documented when they happen, with who gave them, how,
  and whether they were read back.
* Good, because one rule decides who gives orders in ORDO (physicians, by default), and
  confirmation uses the same rule.
* Bad, because a paramedic can no longer give an order in ORDO for a non-medical step
  (e.g. transport); they tell the crew directly, or the organisation widens its policy.
* Bad, because there are two ways an order comes into ORDO, which the app and the
  overview must show apart.

### Implementation

[ORDER-6](../requirements/04-med.md#order-6--record-spoken-orders) and ORDER-7, design
items *Recorded orders* and *Order documentation* in
[chapter 8](../architecture/08-concepts.md#recorded-orders), next to *Order permission*.
