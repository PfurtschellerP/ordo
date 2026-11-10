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
| FwDV 100 "Führung und Leitung im Einsatz" ([copy of the Baden-Württemberg fire service school](https://www.lfs-bw.de/fileadmin/LFS-BW/themen/gesetze_vorschriften/fwdv/dokumente/FwDV_100.pdf)) | AFKzV, Stand 10 March 1999; **introduced by each state**, partly as an extended DV 100 (e.g. Rhineland-Palatinate 2000, Saxony-Anhalt 2011), and by aid organisations; per state: [German state sources](standards-de-states.md) | profile, planned | Command organisation (incident command, sectors, sub-sectors), command levels A–D, staff functions S1–S6, specialist advisers and liaison officers, incident log and situation map; [ADR-0031](adr/0031-command-structure-from-doctrine.md) |
| [SKKM-Richtlinie für das Führen im Katastropheneinsatz](https://www.bmi.gv.at/204/download/files/skkm-richtlinie-fuer-das-fuehren-im-katastropheneinsatz_bf_20260820.pdf) | Austrian Federal Ministry of the Interior (SKKM), 2nd revised edition June 2026 | reference, later AT profile | Each authority and organisation keeps its own command organisation, linked through interfaces; staff functions S1–S6 (S7 optional); incident log kept by S3 |
| [ISO 22320:2018](https://www.iso.org/standard/67851.html) Emergency management — Guidelines for incident management | ISO, 2018, confirmed 2024 | reference | Roles and responsibilities, chain and unity of command, joint command, span of control: the core meanings behind regional command structures |
| [NIMS / Incident Command System](https://www.fema.gov/emergency-managers/nims) | FEMA (US), NIMS 3rd edition, October 2017 | reference | Modular structure that grows with the incident, unified command, span of control (optimum 1:5); a future international profile |
| [JESIP Joint Doctrine](https://jesip.org.uk/what-is-the-joint-doctrine) | JESIP (UK), edition 3.1 | reference | Joint working principles, command levels (strategic, tactical, operational), M/ETHANE message format |
| PDV 100 "Führung und Einsatz der Polizei" | German police, classified *VS – Nur für den Dienstgebrauch* | not available | Police command structure; police modules only with police partners |
| [Taktische Zeichen im Bevölkerungsschutz – Empfehlungen zur Einführung einer FwDV 102/DV 102](https://www.kfv-ab.de/images/KFV/2025/250228_TZ/Taktische_Zeichen_im_Bevoelkerungsschutz.pdf) | BBK, 1st edition May 2024, ISBN 978-3-949117-29-9; recommended to the states by the AFKzV on 11/12 March 2024; replaces the SKK recommendation of 2010 (copy hosted by a district fire brigade association; graphics on the [BABZ learning platform](https://www.lernplattform-babz-bund.de)) | profile (DE), in use | Tactical map symbols, built from frame, basic sign, organisation colour and additional signs. The DE profile's symbols are generated from these parameters ([ADR-0054](adr/0054-tactical-symbols-generated.md)); to be compared with this edition before field use |
| Unit status codes (FMS/TMS), e.g. [Bavarian FMS guideline](https://www.gesetze-bayern.de/Content/Document/BayVwV156097), [Baden-Württemberg tactical status messages](https://www.lfs-bw.de/fileadmin/LFS-BW/themen/technik/funk/digitalfunk/regelungenbetriebshandbuch/dokumente/Taktische_Statusmeldungen.pdf) | German states; meanings of 1–9 largely uniform, details differ by state | profile | Status codes 1–9 of the DE profile, mapped to core meanings; prototype list in `platform/ordo-app/src/text.rs` |

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

## Identity and access

| Standard | Publisher, edition | Status | Use in ORDO |
|---|---|---|---|
| [Cedar](https://github.com/cedar-policy/cedar) policy language | Cedar project, CNCF Sandbox (since October 2025), Apache-2.0 | followed | Roles and permissions ([ADR-0035](adr/0035-authorization-with-cedar.md)); schema and default policies in `platform/ordo-server/policies/`, `packages/*/policies/` |
| [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html) with [PKCE (RFC 7636)](https://www.rfc-editor.org/rfc/rfc7636) | OpenID Foundation, IETF | followed | Login with the organisation's identity provider; ORDO is the relying party, the browser sees no token ([ADR-0038](adr/0038-login-and-device-pairing.md)) |
| [OAuth 2.0 Device Authorization Grant (RFC 8628)](https://www.rfc-editor.org/rfc/rfc8628) | IETF | followed | Pairing shared tablets and phones by QR code |
| [OAuth 2.0 DPoP (RFC 9449)](https://www.rfc-editor.org/rfc/rfc9449) | IETF | planned | Access tokens bound to the device key for HTTP APIs |
| [Dex](https://dexidp.io) | CNCF, Apache-2.0, v2.46 | followed | Login of every installation: local accounts or the organisation's OpenID Connect provider as connector ([ADR-0039](adr/0039-dex-in-every-installation.md)); also in the browser tests |
| Ed25519 challenge–response ([RFC 8032](https://www.rfc-editor.org/rfc/rfc8032)) | IETF | followed | Every sync connection proves the device key ([protocol §2](../spec/sync/protocol.md)) |

## Engineering: architecture, requirements, code

| Standard | Publisher, edition | Status | Use in ORDO |
|---|---|---|---|
| [arc42](https://arc42.org) | arc42 (Starke, Hruschka), template with 12 chapters | followed | Architecture documentation, [docs/architecture](architecture/README.md); design items in chapter 8 |
| [MADR](https://adr.github.io/madr/) (Markdown Architecture Decision Records) | MADR 4 | followed | One decision per file, [docs/adr](adr/README.md); accepted ADRs stand, a change needs a superseding ADR |
| [EARS](https://alistairmavin.com/ears/) (Easy Approach to Requirements Syntax) | Mavin et al., IEEE RE 2009 | followed | Sentence patterns of new and changed requirements, [requirements README](requirements/README.md) |
| [OpenFastTrace](https://github.com/itsallcode/openfasttrace) | itsallcode, 4.11.0 (GPL-3.0, used as a tool) | followed | Requirements → design items → code and tests (`impl`, `utest`, `itest` tags), traced in CI per milestone |
| [Gherkin](https://cucumber.io/docs/gherkin/reference/) | Cucumber | followed | Acceptance scenarios, [tests/acceptance/features](../tests/acceptance/features/) |
| RFC 2119 / [RFC 8174](https://www.rfc-editor.org/rfc/rfc8174) | IETF (BCP 14) | followed | MUST/SHOULD/MAY in the interface specifications ([spec/](../spec/)) |
| [CBOR (RFC 8949)](https://www.rfc-editor.org/rfc/rfc8949) and [CDDL (RFC 8610)](https://www.rfc-editor.org/rfc/rfc8610) | IETF | followed | Encoding and schema of events and the sync protocol ([spec/sync](../spec/sync/protocol.md)) |
| Docs-as-code | community practice | followed | Documentation next to the code, reviewed and checked like code ([ADR-0023](adr/0023-docs-as-code.md)) |
| [AGENTS.md](https://agents.md) | open format (Linux Foundation, Agentic AI Foundation) | followed | Instructions for AI coding agents ([AGENTS.md](../AGENTS.md)) |
| [Rust Style Guide](https://doc.rust-lang.org/style-guide/) (rustfmt) and [Clippy](https://doc.rust-lang.org/clippy/) | Rust project | followed | Formatting checked in CI; Clippy with warnings as errors; no `unsafe` in ORDO's own code |
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | Rust project | reference | Naming and public interfaces of the crates |
| [How to write a Git commit message](https://cbea.ms/git-commit/) | Chris Beams | followed | Imperative subject line; the body says why |
| [REUSE](https://reuse.software/spec/) and [SPDX licence identifiers](https://spdx.org/licenses/) | FSFE, REUSE 3.3; SPDX | followed | Licence of every file (`REUSE.toml`), checked in CI |
| [Mermaid](https://mermaid.js.org) | Mermaid project | followed | Diagrams in the documentation, as text |
| [OWASP ASVS](https://github.com/OWASP/ASVS) | OWASP, 5.0 (May 2025) | followed (target) | Security requirements: level 2 overall, selected level-3 requirements for keys and clinical data (decision of 2026-11-08) |
| [BSI TR-03161](https://www.bsi.bund.de/EN/Themen/Unternehmen-und-Organisationen/Standards-und-Zertifizierung/Technische-Richtlinien/TR-nach-Thema-sortiert/tr03161/tr-03161.html) parts 2 and 3 | BSI | reference | Security of health applications (web, backend) |
| [OpenSpec](https://openspec.dev) | Fission AI, MIT | considered, not adopted | Spec-driven change workflow for AI assistants; ORDO's requirements, design items and ADRs already cover it with CI tracing ([ADR-0048](adr/0048-openspec-not-adopted.md)) |

Data and security formats are listed with their purpose in
[architecture 4.3](architecture/04-solution-strategy.md#43-open-standards-in-use):
CBOR, CDDL, Ed25519 (RFC 8032), XChaCha20-Poly1305, BLAKE3, ULID, WebSocket (RFC 6455), URI
fragments and percent-encoding (RFC 3986), JPEG, OCI images. For attachments: the
Amazon S3 API as implemented by fully open-source stores, tested with
[Garage](https://garagehq.deuxfleurs.fr) 2.1 (AGPL-3.0, [ADR-0033](adr/0033-attachments-in-an-object-store.md)).

## Regulation and product security

| Standard | Publisher, edition | Status | Use in ORDO |
|---|---|---|---|
| [Cyber Resilience Act](https://eur-lex.europa.eu/eli/reg/2024/2847/oj), Regulation (EU) 2024/2847 | EU; in force since 10 December 2024, reporting duties since 11 September 2026, other duties from 11 December 2027; Commission guidance of 27 July 2026, C(2026) 5252 *(date from secondary sources)* | followed (role below) | ORDO is non-commercial free software; we prepare what manufacturers need: SBOM, vulnerability handling, signed releases |
| [NIS2 Directive](https://eur-lex.europa.eu/eli/dir/2022/2555/oj), (EU) 2022/2555 | EU | reference | Operators such as rescue services may be regulated entities and must check their suppliers; the CRA preparations cover what they ask for |

### ORDO's role under the Cyber Resilience Act

*Our reading, not legal advice. To be checked by a lawyer before the first release and
whenever ORDO's governance or funding changes.*

- **The ORDO project is not a manufacturer.** ORDO is free and open-source software
  (AGPL-3.0-or-later), developed without commercial activity: no paid edition, no
  updates or features tied to payment. Publishing the code is not placing it on the
  market. Donations, grants or public funding do not change this as long as the software
  stays freely available on the same terms.
- **Whoever monetises ORDO is the manufacturer of that product.** For example, someone
  who sells a field server with ORDO preinstalled, offers a paid edition, or otherwise
  places ORDO on the market as part of a commercial activity carries the manufacturer's
  duties for it: risk assessment, SBOM, vulnerability handling and reporting, security
  updates for the support period, CE marking.
- **Open-source steward, possibly later.** If a legal entity (for example a foundation or
  association) is founded to support ORDO systematically, and ORDO is meant for
  commercial use by others, that entity can become an *open-source software steward*
  (Article 24) with lighter duties: a documented cybersecurity policy, cooperation with
  market surveillance authorities, and reporting of actively exploited vulnerabilities as
  far as it is involved. This will be an ADR when the governance is decided
  ([roadmap](roadmap.md)).
- **CRA-ready anyway.** So that operators, integrators and future manufacturers can adopt
  ORDO easily, the project provides what the CRA asks of products: coordinated
  vulnerability disclosure ([SECURITY.md](../SECURITY.md)), dependency checks on every
  change, and, planned, an SBOM, build provenance and signatures for every release.

## Keeping this list current

- Add a row when ORDO starts to follow, or plans to follow, a standard; link the ADR,
  requirement or file that uses it.
- Record the edition. When a new edition appears, check what changes and note it here.
- Prefer the publisher's own page. Mark secondary sources.
- State-specific doctrine and rules go to [German state sources](standards-de-states.md)
  and the state's profile.
