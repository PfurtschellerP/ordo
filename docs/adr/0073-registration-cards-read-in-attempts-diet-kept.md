---
status: accepted
date: 2026-11-13
---

# Registration cards are read in several attempts, and their diet is kept

Partly supersedes [ADR-0072](0072-registration-cards-read-with-rqrr.md): how the photo is
prepared, and that the diet is not taken. rqrr and the format as profile data stand.

## Context and Problem Statement

The product owner tried reading a Meldekarte on 2026-11-13 by photographing it on a
screen, and it failed. The two photos (12 and 24 megapixels, from two angles) were not
read by rqrr at 1600 pixels, the one size ADR-0072 chose. The QR code was sharp and whole,
but a photo of a screen carries the display's pixel grid, which turns into moiré when the
photo is shrunk. Which size and smoothing read a photo differed from photo to photo.

The product owner also wants the diet: the food station on site plans with it. And a card
is a convenience to register helpers quickly, and to see who is still out (each picks up
their card when released); a card that is not read now and then is acceptable, a card
read wrongly is not.

## Considered Options

* rqrr, the photo prepared in several attempts
* rxing (ZXing port), its QR reader alone
* rqrr with ORDO's own thresholding (black and white by local mean)

## Decision Outcome

Chosen option: "rqrr, the photo prepared in several attempts". Tested with the two photos
and a synthetic photo, each also turned (−12° to 45°, 90°), cut, and at half size: of the
variants where the code was whole, rqrr read all real ones, within one to four attempts
and under 0.4 s natively. rxing read the same ones in fewer attempts and costs 0.3 MB
(compressed) more now that only its QR reader is linked (ADR-0072 measured the whole
crate). But it pulls in about 25 crates, among them `encoding`, which RustSec lists as
unmaintained (RUSTSEC-2021-0153); `cargo deny` refuses it. Our own thresholding did not
read more than rqrr's own.

* **The app shrinks the photo, not the browser:** it takes the photo from a canvas at full
  size, a strip of rows at a time (Safari limits a canvas to about 16 megapixels), and
  averages blocks of k × k pixels, k the smallest whole number that brings the longer edge
  to at most 3000 pixels. Browsers shrink with too few samples per pixel, even with
  `imageSmoothingQuality` "high": the browser test's photo of a screen was not read after
  Chromium or Firefox had shrunk it, nor after shrinking by sampling in a native test, and
  was read after averaging.
* **Attempts** (longest edge, box blur radius): 3000 and 1, 2000 and 1, 2400 and 0, 1600
  and 1, 1200 and 0, until one finds a code. Shrinking averages pixels (no moiré from
  skipped ones), enlarging interpolates (a code small in the photo gets enough pixels per
  module), the blur smooths the pixel grid away. In this order every test photo that can be
  read at all is read, most of them at the first or second attempt. Between attempts the
  page stays responsive.
* **Conservative parsing:** a card with another number of fields (a field holding the
  separator, a changed format) is not taken at all; a name needs a letter; a single value
  that does not fit (a date in another form, longer than the register keeps, control
  characters) is left out and the rest taken. The command checks before saving.
* **The diet is kept** (HELP-1): a field of the register, in the sealed compartment
  `people` with everything else, deleted with it after the retention period
  ([ADR-0071](0071-helper-register-in-its-own-compartment.md)). A diet can say something
  about health (e.g. lactose intolerance); it is kept because catering on site needs it,
  only the incident command reads it, and the overview counts the diets of those still on
  duty rather than listing who eats what. Schema version 2 of the helper events adds it.

### Consequences

* Good, because photos of cards on screens and on paper are read, offline, with a small
  dependency (rqrr needs only `g2p`, for its Galois field arithmetic).
* Good, because a misread value is left out rather than taken.
* Bad, because a photo without a code takes all five attempts (0.3 s natively, longer in
  a tablet's browser) before it is reported as not recognised.
* Bad, because the attempts were tuned on few photos; field use will show more. The test
  keeps a synthetic photo of a screen that the old single attempt does not read.

## More Information

Design item *Registration cards* in
[chapter 5](../architecture/05-building-block-view.md#registration-cards); the roadmap's
dependency analysis, which this decision is an example of.
