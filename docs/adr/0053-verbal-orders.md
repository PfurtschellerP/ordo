---
status: proposed
date: 2026-11-10
---

# Orders given by voice or radio are recorded by the crew and confirmed later

## Context and Problem Statement

[ORDER-1 to ORDER-4](../requirements/04-med.md#4111-med-package-medical-orders) let
physicians and paramedics give orders on the patient record; the server accepts
`orders.given` only from these roles ([ORDER-3](../requirements/04-med.md#order-3--only-permitted-roles-give-orders),
`dsn~order-permission~1`). In practice many orders are spoken: the emergency physician at
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

Proposed option: the second one.

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
* ORDO records and shows; it never judges whether an order was permitted, needed or
  correct (EPCR-9).

Before acceptance, practitioners and the organisation's medical director (ÄLRD) should
confirm the fields and the confirmation step, and check them against the rules for
delegated measures (in Germany e.g. § 4 NotSanG).

### Consequences

* Good, because spoken orders are documented when they happen, with who gave them, how,
  and whether they were read back.
* Good, because the rule that only physicians and paramedics give orders in ORDO stays
  intact, and confirmation uses the same rule.
* Bad, because there are two ways an order comes into ORDO, which the app and the
  overview must show apart.

### If accepted

* New requirement ORDER-6 (EARS): "When a crew member records an order given by voice,
  radio, phone or telemedicine, ORDO shall record who gave it, by which channel and
  whether it was read back, mark it until the person who gave it confirms it, and accept
  the confirmation only from roles that give orders."
* Design item in chapter 8 next to *Medical orders*, Cedar rule next to *Order
  permission*.
