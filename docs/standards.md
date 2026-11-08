# Standards, doctrine and conventions

ORDO follows published standards and professional doctrine wherever they exist and fit
([ADR-0022](adr/0022-open-standards-first.md)). This page lists each one with its edition,
how ORDO uses it, and where. Technical formats are summarised in
[architecture 4.3](architecture/04-solution-strategy.md#43-open-standards-in-use).

**Status:** *followed* = ORDO implements it; *profile* = it shapes a regional profile
(data, not code); *planned* = intended, not built yet; *reference* = read for design
decisions, not implemented.

Some documents are not freely licensed. ORDO links to them and describes what it takes
from them in its own words; it does not copy their text. Where we only found a secondary
source, it is marked *(secondary)*; replace it when the official source is known.

## Command and control

| Standard | Publisher, edition | Status | Use in ORDO |
|---|---|---|---|
| [FwDV 100 "Führung und Leitung im Einsatz"](https://www.lfs.sachsen.de/download/fwdv100.pdf) | AFKzV, Stand 10 March 1999; adopted by the German states and, as DV 100, by aid organisations | profile, planned | Command organisation (incident command, sectors, sub-sectors), command levels A–D, staff functions S1–S6, specialist advisers and liaison officers, incident log and situation map; [ADR-0031](adr/0031-command-structure-from-doctrine.md) |
| [SKKM-Richtlinie für das Führen im Katastropheneinsatz](https://www.bmi.gv.at/204/download/files/skkm-richtlinie-fuer-das-fuehren-im-katastropheneinsatz_bf_20260820.pdf) | Austrian Federal Ministry of the Interior (SKKM), 2nd revised edition June 2026 | reference, later AT profile | Each authority and organisation keeps its own command organisation, linked through interfaces; staff functions S1–S6 (S7 optional); incident log kept by S3 |
| [ISO 22320:2018](https://www.iso.org/standard/67851.html) Emergency management — Guidelines for incident management | ISO, 2018, confirmed 2024 | reference | Roles and responsibilities, chain and unity of command, joint command, span of control: the core meanings behind regional command structures |
| [NIMS / Incident Command System](https://www.fema.gov/emergency-managers/nims) | FEMA (US), NIMS 3rd edition, October 2017 | reference | Modular structure that grows with the incident, unified command, span of control (optimum 1:5); a future international profile |
| [JESIP Joint Doctrine](https://jesip.org.uk/what-is-the-joint-doctrine) | JESIP (UK), edition 3.1 | reference | Joint working principles, command levels (strategic, tactical, operational), M/ETHANE message format |
| PDV 100 "Führung und Einsatz der Polizei" | German police, classified *VS – Nur für den Dienstgebrauch* | not available | Police command structure; police modules only with police partners |
| [Taktische Zeichen im Bevölkerungsschutz](https://lv-saarland.drk.de/fileadmin/user_upload/Begleitende_Hinweise_zur_%C3%9Cberarbeitung.pdf) (recommendation for FwDV/DV 102) | AFKzV/SKK, revised edition 2024 *(secondary)* | profile, planned | Tactical map symbols; open question on licensing ([roadmap](roadmap.md)) |
| Unit status codes (FMS/TMS), e.g. [Bavarian FMS guideline](https://www.gesetze-bayern.de/Content/Document/BayVwV156097), [Baden-Württemberg tactical status messages](https://lfs-bw.de/fileadmin/LFS-BW/themen/funk/digitalfunk/regelungenbetriebshandbuch/dokumente/Taktische_Statusmeldungen_022025.pdf) | German states; meanings of 1–9 largely uniform, details differ by state | profile | Status codes 1–9 of the DE profile, mapped to core meanings; prototype list in `platform/ordo-app/src/text.rs` |

## Medical response and patient records

| Standard | Publisher, edition | Status | Use in ORDO |
|---|---|---|---|
| [DIN 13050 Begriffe im Rettungswesen](https://www.dinmedia.de/de/norm/din-13050/343530475) | DIN, 2021-10 (paid) | reference | Terms: medical incident command (LNA, OrgL), treatment area, patient collection point; [glossary](glossary.md) |
| Sichtungskategorien SK I–IV and the triage tag (PAK) | BBK consensus conferences on triage; state rules, e.g. [Bavarian MANV guideline 2025](https://kvb.de/fileadmin/kvb/Mitglieder/Versorgung/Notarztdienst/STMI-MAN-RL-2025-Ersteinschaetzung-Vorsichtung-Kennzeichnung-Registrierung.pdf) | followed (profile) | Triage categories with colour, label and shape (PAT-2, A11Y-3). ORDO records the category; it never computes one (EPCR-9) |
| Sanitätseinsatzleitung and MANV sections (patient collection point, treatment area, transport organisation) | State law and MANV concepts, e.g. Saarland *Rahmenkonzept MANV* (11/2025) | reference, planned | Medical branch of the command structure ([ADR-0031](adr/0031-command-structure-from-doctrine.md)) |
| DIVI MIND (minimal emergency dataset) and DIVI emergency protocol | DIVI; version 7.0 according to a [December 2025 statement](https://www.bundesgesundheitsministerium.de/fileadmin/Dateien/3_Downloads/Gesetze_und_Verordnungen/Stellungnahmen_WP21/Notfallreform/AKTIN__DNVF__GMDS__TMF_Stellungnahme_RefE_Notfallreform.pdf) *(secondary)* | profile | Section structure of the ePCR forms in `profiles/de/med/epcr-forms.json` (EPCR-2); field-level alignment and licence are open questions |
| [HL7 FHIR R4 Questionnaire](https://hl7.org/fhir/R4/questionnaire.html) | HL7 International, R4 (4.0.1), CC0 | followed | Format of the ePCR forms in a documented subset ([ADR-0037](adr/0037-epcr-forms-as-fhir-questionnaire.md), [spec](../spec/profiles/questionnaire-subset.md)); questions name FHIR elements for the later handover (EPCR-6) |
| [HL7 FHIR SDC](https://hl7.org/fhir/uv/sdc/) (Structured Data Capture) | HL7 International | followed (one extension) | `calculatedExpression`, restricted to sums of documented values |
| [UCUM](https://ucum.org) | Regenstrief Institute, free licence with notice | profile | Units of numbers in the ePCR forms; the notice is in the profile |
| [LOINC](https://loinc.org) | Regenstrief Institute, free licence with notice | profile | Vital-sign questions carry LOINC codes; the notice is in the profile (`copyright`) |
| [OASIS EDXL-TEP 1.1](https://docs.oasis-open.org/emergency/edxl-tep/v1.1/edxl-tep-v1.1.html) Tracking of Emergency Patients | OASIS, CS02, September 2018 | planned | Candidate for exchanging patient tracking data with control centres and hospitals |
| [EU Medical Device Regulation 2017/745](https://eur-lex.europa.eu/eli/reg/2017/745/oj) | EU | followed | ORDO documents and organises, it never gives clinical decision support ([constraints](requirements/08-constraints.md)) |
| [GDPR](https://eur-lex.europa.eu/eli/reg/2016/679/oj), Art. 5 and 9 | EU | followed | Data minimisation, health data in the protected compartment, photos only for identification and re-encoded without metadata ([ADR-0028](adr/0028-patient-photos-in-sealed-events.md)) |

## Accessibility and user interface

| Standard | Publisher, edition | Status | Use in ORDO |
|---|---|---|---|
| [EN 301 549](https://www.etsi.org/deliver/etsi_en/301500_301599/301549/03.02.01_60/en_301549v030201p.pdf) | ETSI/CEN/CENELEC, V3.2.1 (2021-03) | followed | Accessibility target ([ADR-0020](adr/0020-accessibility-target.md)) |
| [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | W3C Recommendation, level AA | followed | Checked in CI with axe-core and our own checks (`tests/e2e/support/audit.ts`) |
| [BITV 2.0](https://www.gesetze-im-internet.de/bitv_2_0/) | Germany | followed | Accessibility duty of public bodies in Germany; met through EN 301 549 |
| [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/) | W3C | followed | Disclosure (menu), [combobox](https://www.w3.org/WAI/ARIA/apg/patterns/combobox/) and [modal dialog](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/) (command palette, [ADR-0030](adr/0030-command-palette.md)) |
| [HTML Media Capture](https://www.w3.org/TR/html-media-capture/) | W3C | followed | Taking a patient photo with the device camera, without extra permissions |
| Mobile navigation patterns: [Android adaptive navigation](https://developer.android.com/develop/ui/compose/layouts/adaptive/build-adaptive-navigation) (Material 3), Apple Human Interface Guidelines (tab bars) | Google, Apple | followed | Bottom bar on phones, rail on wider screens ([ADR-0026](adr/0026-navigation-at-the-bottom-on-phones.md)) |
| [Modal and nonmodal dialogs](https://nngroup.com/articles/modal-nonmodal-dialog/) | Nielsen Norman Group | reference | Records are edited in place, not in modal dialogs ([ADR-0029](adr/0029-edit-records-in-place.md)) |
| [axe-core](https://github.com/dequelabs/axe-core) | Deque, 4.14 | followed | Automated WCAG 2.2 A/AA checks of every view in the browser tests |
| [W3C Design Tokens Format](https://www.designtokens.org/) | W3C Community Group, 2025.10 | followed | Design system ([ADR-0021](adr/0021-design-system.md)) |

## Data, security and engineering

These are listed with their purpose in [architecture 4.3](architecture/04-solution-strategy.md#43-open-standards-in-use):
CBOR (RFC 8949), CDDL (RFC 8610), Ed25519 (RFC 8032), XChaCha20-Poly1305, BLAKE3,
ULID, WebSocket (RFC 6455), URI fragments and percent-encoding (RFC 3986), JPEG, OCI
images, SPDX and REUSE, arc42, MADR 4, EARS, Gherkin, OpenFastTrace, AGENTS.md.

## Keeping this list current

- Add a row when ORDO starts to follow, or plans to follow, a standard; link the ADR,
  requirement or file that uses it.
- Record the edition. When a new edition appears, check what changes and note it here.
- Prefer the publisher's own page. Mark secondary sources.
