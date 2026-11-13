---
status: accepted
date: 2026-11-13
---

# Registration cards are read from a photo with rqrr; their format is profile data

## Context and Problem Statement

Helpers of German aid organisations often bring a *Meldekarte* (meldekarte.de, run by
volunteers of the BRK Kreisverband Augsburg-Land): a printed card with their personal data
and a QR code. The incident command wants to register helpers by scanning it (roadmap
item 9, [HELP-5](../requirements/02-shared.md#help-5--registration-card-read-by-camera)).

The QR format is not published. The product owner provided a sample card on 2026-11-13;
its QR code is UTF-8 text, 16 fields separated by `;`: family name, given name, date of
birth (`DD.MM.YYYY`), gender (`m`, `w`, `d`), postcode, town, nationality, street, mobile
number, district branch, organisation and unit (joined by a space), place of action,
formation, start and end of action (`DD.MM.YYYY - hh:mm`), diet. How a `;` inside a field
is written is unknown.

ORDO's web app must read it offline, in every supported browser (Chromium, Firefox,
WebKit), on tablets and phones, without growing much: it is downloaded and kept by every
device.

## Considered Options

* `rqrr` (Rust, MIT or Apache-2.0, QR codes only) in the app, the photo decoded by the
  browser's canvas
* `rxing` (Rust port of ZXing, Apache-2.0, all barcode formats)
* The browser's `BarcodeDetector`
* `jsQR` (JavaScript, Apache-2.0)

## Decision Outcome

Chosen option: "`rqrr`", because it reads the sample card, also from a blurred, rotated,
noisy photo, and adds about 0.2 MB (0.1 MB compressed) to the app. `rxing` read it too but
adds 1.35 MB (0.58 MB compressed), 40 % of the app, for formats ORDO does not need.
`BarcodeDetector` is missing in Firefox and in Safari on iPads. `jsQR` has had no release
since 2020 and would be ORDO's first JavaScript dependency in the app.

* **A photo, not a live video:** a file input with `capture` opens the device's camera
  app, which focuses and exposes well; it needs no camera permission and works the same
  everywhere. The photo is drawn on a canvas at most 1600 pixels wide, turned into grey
  values and read by rqrr on the device. It is not stored and not sent.
* **The format is profile data** (rule 6): `helpers/registration-cards.toml` names the
  separator, the number of fields and what goes where, with `{n}` for field n and date
  patterns. The shared module `helpers` reads any such card; it knows nothing of the
  Meldekarte.
* **Only what the register keeps:** name, date of birth, address, phone, organisation and
  duty (from start and end of action). Gender, nationality, diet (which can be health
  data) and the place of action are not taken.
* **The command checks:** the card fills the form; nothing is saved until the command
  presses *Erfassen*.

### Consequences

* Good, because helpers are registered in seconds, offline, on any device.
* Good, because other card formats are a profile change.
* Bad, because the format may change without notice; a card that does not fit is reported
  as not recognised, and the form stays as it was. The source is marked for manual checks.
* Neutral: live scanning of several cards in a row may come later, with the same reader.

## More Information

Design item *Registration cards* in
[chapter 5](../architecture/05-building-block-view.md#registration-cards);
[ADR-0071](0071-helper-register-in-its-own-compartment.md) for the register.
