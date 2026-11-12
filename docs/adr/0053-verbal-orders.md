---
status: accepted
date: 2026-11-12
---

# Orders given by voice or radio are recorded by the crew and confirmed later

## Context and Problem Statement

[ORDER-1 to ORDER-4](../requirements/04-med.md#4111-med-package-medical-orders) let
physicians and paramedics give orders on the patient record; the server accepts
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

Chosen option: the second one, decided on 2026-11-12 with the product owner after a
practitioner's test of the demo: anyone documenting a medication could type any physician
as "Angeordnet durch", so documented orders did not show who really gave them.

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
  orders (Cedar, like `orders.given`). The app offers it to the person named in
  `ordered_by` when that person uses ORDO; the record shows who confirmed and when.
  Unconfirmed recorded orders stay marked, and the lead's overview counts them. The
  server cannot check that the confirmer is the person named, since it cannot read the
  payload; the record shows both names, so a mismatch is visible.
* `orders.given` and its rule stay unchanged: giving an order in ORDO yourself still needs
  the role.
* **Who ordered a documented medication** comes only from an order, given or recorded: the
  entry keeps the order's ID, and the form's field for who ordered is read-only
  ([ORDER-7](../requirements/04-med.md#order-7--who-ordered-comes-from-the-order)).
* ORDO records and shows; it never judges whether an order was permitted, needed or
  correct (EPCR-9).

Still to do with practitioners and an organisation's medical director (ÄLRD): confirm the
fields and the confirmation step, and check them against the rules for delegated measures
(in Germany e.g. § 4 NotSanG); measures a paramedic takes on their own authority are
documented without an order.

### Consequences

* Good, because spoken orders are documented when they happen, with who gave them, how,
  and whether they were read back.
* Good, because the rule that only physicians and paramedics give orders in ORDO stays
  intact, and confirmation uses the same rule.
* Bad, because there are two ways an order comes into ORDO, which the app and the
  overview must show apart.

### Implementation

[ORDER-6](../requirements/04-med.md#order-6--record-spoken-orders) and ORDER-7, design
items *Recorded orders* and *Order documentation* in
[chapter 8](../architecture/08-concepts.md#recorded-orders), next to *Order permission*.
