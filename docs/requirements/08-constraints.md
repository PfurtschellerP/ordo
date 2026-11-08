# ORDO Requirements — Constraints

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

## 6. Constraints

- Licence: **AGPL-3.0-or-later**. Modified versions must stay open, including when they
  are offered as a hosted service.
- Implemented in Rust; client runs as WebAssembly (see [architecture](../architecture/README.md)).
- Only open-licensed map data, symbols and fonts (e.g. OpenStreetMap).
- No dependency on a single commercial cloud provider.
- **Accessibility law:** EN 301 549 / WCAG 2.2 AA ([§5.8](07-quality.md#58-accessibility)). Public bodies using ORDO must
  publish an accessibility statement; ORDO supplies the conformance report for it.
- ORDO is a support tool. Responsibility stays with the people in charge, and
  procedures must work without it (fallback: paper, e.g. paper triage tags).
- **Medical-device regulation (EU MDR):** ORDO documents and organises; it does not
  diagnose, recommend treatment or make triage decisions. Any feature that would do so
  needs a regulatory assessment first, because it could make ORDO a medical device.
- **Health data:** GDPR Art. 9 and national rules on medical documentation (DE: e.g.
  § 630f BGB retention, professional confidentiality) apply to the ePCR. Requires review
  by a data-protection expert before the first field use.
