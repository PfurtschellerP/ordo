# ORDO Requirements — Quality requirements

> Part of the [ORDO requirements](README.md). Format: OpenFastTrace items, see the README.

## 5. Non-functional requirements

### 5.1 Usability

#### UX-1 — Three taps
`req~ux-1~1`

Main functions reachable in ≤ 3 taps from the workspace overview

Tags: must, quality

Needs: dsn

#### UX-2 — Large touch targets
`req~ux-2~1`

Touch targets ≥ 12 mm; usable with gloves on a 10" tablet

Tags: must, quality

Needs: dsn

#### UX-3 — Day and night mode
`req~ux-3~1`

Day mode with high contrast for sunlight; night mode that doesn't dazzle

Tags: must, quality

Needs: dsn

#### UX-4 — No training needed
`req~ux-4~1`

A new user can do the core task of their role without training

Tags: must, quality

Needs: dsn

#### UX-5 — No lost input
`req~ux-5~1`

No data entry is lost when the app is closed, the device sleeps or the network drops

Tags: must, quality, m3

Needs: dsn

#### UX-6 — Tablet, laptop, phone
`req~ux-6~1`

Works on tablet (main device), laptop and phone (reduced view)

Tags: must, quality

Needs: dsn

#### UX-7 — Accessible to everyone
`req~ux-7~1`

Accessible to everyone, including people with disabilities ([§5.8](07-quality.md#58-accessibility))

Tags: must, quality

Needs: dsn

#### UX-8 — Role workspaces
`req~ux-8~1`

Each role opens in its **workspace**: only the views and actions for that role, in that service's wording

Tags: must, quality, m5

Needs: dsn

#### UX-9 — Works with defaults
`req~ux-9~1`

Ready-made packages work with defaults; no configuration needed to start

Tags: must, quality

Needs: dsn

#### UX-10 — Keyboard command palette
`req~ux-10~1`

When a user presses Ctrl+K (⌘K on Apple devices), ORDO shall open a search for views, patients, units and commands, from which the chosen entry runs with the keyboard alone.

Tags: should, quality, m6

Needs: dsn

#### UX-11 — Header and reach for both hands
`req~ux-11~1`

ORDO shall show, on wide screens, the menu, the page heading, the search in the middle, the connection state and the signed-in person in the header from left to right, with the person's settings in a menu of their own; when a person chooses left-handed use on a device, ORDO shall put the menu of the bottom bar at its left end on that device.

Tags: should, quality, m16

Needs: dsn

#### UX-12 — Compact navigation on phones
`req~ux-12~2`

ORDO shall show every view with an icon and a short label, the same in the rail and in the bottom bar, and at most four views plus the menu in the bottom bar; on every screen ORDO shall show the connection state as an icon whose shape tells the state and whose colour repeats it, with the number of unsent inputs, and the full state when the person opens it; errors stay visible.

Tags: should, quality, m16

Needs: dsn

#### UX-13 — Suggest people in person fields
`req~ux-13~1`

When a person fills in a field that names a member of the crew (who carried out a measure, ordered or gave a medication) or grants a role, ORDO shall suggest the people it knows for that purpose and shall accept any other name as typed.

Tags: should, quality, m20

Needs: dsn

#### UX-14 — Move through lists with the keyboard
`req~ux-14~1`

While the focus is on an item of a list, ORDO shall move it to the next item with `j` or Arrow Down and to the previous item with `k` or Arrow Up; in the command palette, ORDO shall move the selection with Ctrl+J and Ctrl+K besides the arrow keys.

Tags: should, quality, m23

Needs: dsn

### 5.2 Offline and reliability

#### OFF-1 — Fully offline
`req~off-1~1`

All functions needed on scene work without any network connection

Tags: must, quality

Needs: dsn

#### OFF-2 — Local network sync
`req~off-2~1`

Devices at the same scene can share data over a local network without internet

Tags: must, quality

Needs: dsn

#### OFF-3 — Automatic sync
`req~off-3~1`

When the connection returns, changes sync automatically without user action

Tags: must, quality, m1

Needs: dsn

#### OFF-4 — No lost changes
`req~off-4~1`

Concurrent changes never lose data; conflicts are resolved automatically and visibly

Tags: must, quality, m1

Needs: dsn

#### OFF-5 — Visible sync state
`req~off-5~1`

The device shows clearly whether it is in sync, and since when it isn't

Tags: must, quality

Needs: dsn

### 5.3 Performance

#### OFF-6 — Attachments
`req~off-6~1`

When a device adds an attachment (photo, document, recording), ORDO shall encrypt it on the device, store it on the server only as ciphertext named by its hash, and hand it only to devices whose users may read the stream it belongs to; large attachments shall not travel with the events.

Tags: must, quality, m13

Needs: dsn

#### OFF-7 — Device storage
`req~off-7~1`

ORDO shall keep an incident's data on the device in a database that is not limited to a few megabytes and is written change by change, encrypted with the device's storage key; where the browser cannot run that database, ORDO shall keep working with its simpler storage.

Tags: must, quality, m13

Needs: dsn

#### PERF-1 — Older tablets
`req~perf-1~1`

Runs smoothly on a mid-range Android tablet that is 5 years old

Tags: must, quality

Needs: dsn

#### PERF-2 — Fast propagation
`req~perf-2~1`

Changes appear on other connected devices within 2 s

Tags: must, quality

Needs: dsn

#### PERF-3 — Large incidents
`req~perf-3~1`

Handles an incident with 100 units, 50 sectors, 500 patients and 10,000 log entries

Tags: must, quality

Needs: dsn

#### PERF-4 — Small hardware
`req~perf-4~1`

A small installation runs on a Raspberry Pi 5 or a mini PC

Tags: must, quality

Needs: dsn

#### PERF-5 — Resource efficiency
`req~perf-5~1`

Resource-efficient by design: low memory and CPU use, small downloads, minimal battery drain

Tags: must, quality

Needs: dsn

#### PERF-6 — Fast start
`req~perf-6~1`

App starts and shows the active incident in < 2 s on the reference tablet, also offline

Tags: must, quality

Needs: dsn

### 5.4 Security and data protection

#### SEC-1 — Encryption in transit and at rest
`req~sec-1~2`

ORDO shall encrypt all network traffic with TLS, and shall store data on devices only
encrypted, with a key that is destroyed when the device is unpaired or revoked.

Tags: must, quality, m10

Needs: dsn

#### SEC-14 — Protection against later decryption
`req~sec-14~1`

ORDO shall protect network traffic against decryption by future quantum computers with a hybrid key exchange (classical and post-quantum) wherever the TLS endpoint it ships supports one, and shall keep every cryptographic algorithm it uses documented and replaceable by a new version.

Tags: must, quality, m27

Needs: dsn

#### SEC-15 — Software bill of materials
`req~sec-15~1`

For every build of what ORDO ships (server, web app), ORDO's build shall produce a software bill of materials in CycloneDX that lists every dependency with its version, and shall fail if it is missing or incomplete.

Tags: must, quality, m28

Needs: dsn

#### SEC-16 — Signed releases with provenance
`req~sec-16~1`

ORDO shall publish every release with signed container images and binaries, a build provenance attestation and its software bills of materials, so operators can verify what they run ([ADR-0043](../adr/0043-supply-chain.md)).

Tags: should, quality

Needs: dsn

#### SEC-17 — Robust against malformed input
`req~sec-17~1`

When the server or a device receives malformed or hostile data (events, protocol messages, payloads, attachments' metadata), ORDO shall refuse it without crashing, hanging or exhausting memory; the decoders of all data from the network shall be fuzzed continuously in CI.

Tags: must, quality, m42

Needs: dsn

#### SEC-2 — Role-based access
`req~sec-2~1`

ORDO shall grant access to compartments by the roles a user holds in the organisation and the incident, never to individual users. Organisations shall be able to define their own roles and permissions without changing code.

Tags: must, quality, m7

Needs: dsn

#### SEC-11 — Authenticated connections
`req~sec-11~1`

When a device connects, ORDO shall verify that the device holds the private key registered for its user and device before it sends any data or key; a device that cannot prove this shall receive nothing, and a recorded connection shall not be replayable.

Tags: must, quality, m7

Needs: dsn

#### SEC-12 — Access timeout
`req~sec-12~1`

ORDO shall end the access of a paired device when its person has not been verified by the identity provider for the access timeout (default 48 hours, configurable from 1 to 30 days), or at once when the identity provider definitively refuses the person; the device shall then lock itself, also offline, until the person logs in again. An unreachable identity provider alone shall not end access before the timeout.

Tags: must, quality, m9

Needs: dsn

#### SEC-3 — Login and single sign-on
`req~sec-3~2`

When a person logs in, ORDO shall authenticate them with their organisation's identity provider via OpenID Connect and shall store no passwords itself. For organisations without an identity provider, the installation bundle shall include one with local accounts.

Tags: must, quality, m8

Needs: dsn

#### SEC-4 — Device revocation
`req~sec-4~2`

When a person revokes one of their devices, or an administrator revokes any device, ORDO shall cut the device's connections at once, refuse it from then on, and make it delete its key and data at its next connection.

Tags: must, quality, m9

Needs: dsn

#### SEC-13 — Device lock
`req~sec-13~1`

While a device holds clinical data and is not used, ORDO shall lock it automatically and unlock it only with a PIN or biometrics. (Split from SEC-4.)

Tags: must, quality

Needs: dsn

#### SEC-5 — GDPR
`req~sec-5~1`

GDPR: data minimisation, configurable retention and deletion, export of personal data

Tags: must, quality

Needs: dsn

#### SEC-6 — Audit trail
`req~sec-6~1`

Audit trail of all access to and changes of incident data

Tags: must, quality

Needs: dsn

#### SEC-7 — Security process
`req~sec-7~1`

Regular dependency updates and a documented security-reporting process

Tags: must, quality

Needs: dsn

#### SEC-8 — Data compartments
`req~sec-8~1`

ORDO shall separate data into compartments (e.g. shared incident data, clinical data, police-restricted data), each with its own access rules, encryption key, sync scope and retention.

Tags: must, quality

Needs: dsn

#### SEC-9 — Compartment-scoped sync
`req~sec-9~1`

ORDO shall send a device only data from compartments its user may read. A device whose user may not read a compartment shall never receive or store that compartment's data (e.g. a fire tablet never stores patient records).

Tags: must, quality, m1

Needs: dsn

#### SEC-10 — Health data protection
`req~sec-10~2`

ORDO shall treat health data as special-category data (GDPR Art. 9) and store it only encrypted per compartment, on the server and on devices. Access logging is covered by SEC-6 and EPCR-7.

Tags: must, quality, m1

Needs: dsn

### 5.5 Operation

#### OPS-1 — One-command install
`req~ops-1~1`

Install a self-hosted server with one command (single binary or container image) and a setup wizard

Tags: must, quality

Needs: dsn

#### OPS-2 — Safe updates
`req~ops-2~1`

Updates without data loss; automatic database migrations

Tags: must, quality

Needs: dsn

#### OPS-3 — Backup and restore
`req~ops-3~1`

Built-in backup and restore, including encrypted compartments

Tags: must, quality

Needs: dsn

#### OPS-4 — Hosted multi-tenant
`req~ops-4~1`

The same software can later run as a hosted multi-tenant service

Tags: must, quality

Needs: dsn

#### OPS-5 — Container image
`req~ops-5~1`

An official container image (multi-architecture: amd64, arm64) is published with every release

Tags: must, quality, m41

Needs: dsn

#### OPS-6 — Docker Compose setups
`req~ops-6~1`

Ready-made Docker Compose setups: minimal (ORDO + automatic TLS) and full (ORDO + PostgreSQL + TLS)

Tags: must, quality

Needs: dsn

#### OPS-7 — External configuration
`req~ops-7~1`

All configuration via environment variables or a config file; nothing baked into the image

Tags: must, quality, m2

Needs: dsn

#### OPS-8 — Health, logs, metrics
`req~ops-8~1`

Health and readiness endpoints, structured logs to stdout, metrics (Prometheus) and optional tracing (OpenTelemetry)

Tags: must, quality

Needs: dsn

#### OPS-9 — Horizontal scaling
`req~ops-9~1`

With PostgreSQL, several server instances can run in parallel behind a load balancer

Tags: must, quality

Needs: dsn

#### OPS-10 — Graceful shutdown
`req~ops-10~1`

Graceful shutdown: connected devices reconnect to another instance without losing data

Tags: must, quality

Needs: dsn

#### OPS-11 — Helm chart
`req~ops-11~1`

A Helm chart for Kubernetes (later)

Tags: could, quality

Needs: dsn

### 5.6 Internationalisation

#### I18N-1 — German and English
`req~i18n-1~1`

User interface in German and English at launch; all texts translatable

Tags: must, quality

Needs: dsn

#### I18N-2 — Regional profiles
`req~i18n-2~1`

Regional profiles bundle terminology, unit types, status codes, tactical symbols, incident keywords, triage categories and patient-record forms. **The first profile is Germany (DE)**: FwDV 100, DV 102, DIN/EN vehicle types, status 1–9, *Sichtungskategorien*, DIVI MIND. Austria and Switzerland follow.

Tags: must, quality

Needs: dsn

#### I18N-3 — Time and units
`req~i18n-3~1`

Times stored in UTC and shown in local time; units of measurement configurable

Tags: must, quality

Needs: dsn

#### I18N-4 — No regional code in core
`req~i18n-4~1`

The core contains **no country-specific code or data**; everything regional comes from profiles

Tags: must, quality

Needs: dsn

#### I18N-5 — Profiles without recompiling
`req~i18n-5~1`

A new country, state or organisation profile can be added without changing or recompiling ORDO

Tags: must, quality

Needs: dsn

#### I18N-6 — Layered profiles
`req~i18n-6~1`

Profiles are layered: a profile can extend another and add, override or hide entries (e.g. DE → Bavaria → own organisation)

Tags: must, quality

Needs: dsn

#### I18N-7 — Profiles for non-programmers
`req~i18n-7~1`

Profiles can be written and validated by non-programmers (documented text format + validation tool)

Tags: must, quality

Needs: dsn

#### I18N-8 — Profile format versions
`req~i18n-8~1`

Each profile declares which version of the profile format it uses; ORDO rejects incompatible profiles with a clear message

Tags: must, quality

Needs: dsn

### 5.7 Modularity

#### MOD-1 — Layered structure
`req~mod-1~1`

ORDO is layered: platform → shared modules → service packages, with connectors between packages and adapters to external systems

Tags: must, quality

Needs: dsn

#### MOD-2 — Modules
`req~mod-2~1`

Every functional area is a module with its own events, state, permissions, views and settings

Tags: must, quality

Needs: dsn

#### MOD-3 — Extensible without changes
`req~mod-3~1`

A new module or service package can be added without changing existing ones

Tags: must, quality

Needs: dsn

#### MOD-4 — Switch packages and modules
`req~mod-4~1`

Service packages and modules can be switched on or off per organisation

Tags: must, quality

Needs: dsn

#### MOD-5 — Published interfaces only
`req~mod-5~1`

Modules depend only on the platform and on other modules' published interfaces, never on their internals

Tags: must, quality

Needs: dsn

#### MOD-6 — Adapters
`req~mod-6~1`

Interfaces to external systems are adapters behind a common interface

Tags: must, quality

Needs: dsn

#### MOD-7 — Replaceable data
`req~mod-7~1`

Map symbol sets, forms, report templates and translations are replaceable data, not code

Tags: must, quality

Needs: dsn

### 5.8 Accessibility

Responders, command assistants and control-centre staff include people with visual,
hearing, motor or cognitive impairments — and everyone is impaired at a scene at night,
in the rain, with gloves, under stress. Accessibility is a core requirement, not an
add-on.

#### A11Y-1 — EN 301 549 and WCAG 2.2 AA
`req~a11y-1~1`

ORDO conforms to **EN 301 549**, the European standard behind the European Accessibility Act, the EU Web Accessibility Directive and, in Germany, BITV 2.0 and the state regulations that apply to municipal bodies such as fire brigades. Target: **WCAG 2.2 Level AA** (as in EN 301 549 v4.1.1), which also covers v3.2.1 (WCAG 2.1 AA), the version currently cited as harmonised standard

Tags: must, quality

Needs: dsn

#### A11Y-2 — Applies to every part
`req~a11y-2~1`

This applies to every part: app, admin interface, setup wizard, generated documents (PDF/UA-1 tagged PDFs, EN 301 549 clause 10) and the documentation website

Tags: must, quality

Needs: dsn

#### A11Y-3 — Not colour alone
`req~a11y-3~1`

Colour is never the only carrier of meaning (WCAG 1.4.1): triage categories, unit status and alarms always combine colour with text and a shape or pattern — about 8 % of men have a colour-vision deficiency

Tags: must, quality, m3

Needs: dsn

#### A11Y-4 — Keyboard and screen readers
`req~a11y-4~1`

Fully operable by keyboard alone and with screen readers (NVDA, JAWS, VoiceOver, TalkBack, Orca): semantic HTML first, WAI-ARIA only where needed, following the WAI-ARIA Authoring Practices

Tags: must, quality

Needs: dsn

#### A11Y-5 — Accessible map alternative
`req~a11y-5~1`

Everything on the map is also available in an accessible list view with the same actions, because a map canvas is not accessible to screen readers

Tags: must, quality

Needs: dsn

#### A11Y-6 — Zoom, reflow and OS settings
`req~a11y-6~1`

Works with 200 % text size and 400 % zoom (reflow at 320 CSS px) without losing content or functions; follows the operating system's settings for text size, dark mode, increased contrast, reduced motion and forced colours

Tags: must, quality

Needs: dsn

#### A11Y-7 — Multi-sensory alarms
`req~a11y-7~1`

Alarms and time-critical notices use several senses: visual plus sound plus vibration where available; never sound only (noise at the scene); nothing flashes more than three times per second (WCAG 2.3.1)

Tags: must, quality

Needs: dsn

#### A11Y-8 — Accessible locking
`req~a11y-8~1`

Automatic locking (SEC-13) warns beforehand, never loses data, and unlocking needs no memory or puzzle test (WCAG 2.2.1, 3.3.8)

Tags: must, quality

Needs: dsn

#### A11Y-9 — Plain language
`req~a11y-9~1`

Clear, short wording and consistent terms in the UI; texts shown to patients or the public (e.g. refusal of treatment) are available in plain language (*Einfache / Leichte Sprache*) and several languages

Tags: must, quality

Needs: dsn

#### A11Y-10 — Continuous accessibility testing
`req~a11y-10~1`

Tested continuously: automated checks in CI, manual screen-reader tests for every release, and an expert audit (e.g. BITV-Test) before the first release for public bodies

Tags: must, quality

Needs: dsn

#### A11Y-11 — Conformance report
`req~a11y-11~1`

Every release comes with an accessibility conformance report against EN 301 549, so that public bodies can write their accessibility statement (*Erklärung zur Barrierefreiheit*); ORDO has a built-in way to report barriers

Tags: must, quality

Needs: dsn

### 5.9 Design system

#### DS-1 — One design system
`req~ds-1~1`

One consistent design system for all packages: components, layout, icons, motion and wording rules, documented with live examples

Tags: must, quality

Needs: dsn

#### DS-2 — Design tokens
`req~ds-2~1`

ORDO shall define all visual values as design tokens in the open W3C Design Tokens format (DTCG 2025.10), in three layers: primitive → semantic → component.

Tags: must, quality, m0

Needs: dsn

#### DS-3 — Themes
`req~ds-3~1`

ORDO shall ship the themes *day*, *night* and *high contrast*, and themes shall change semantic tokens only. Organisations can add their own branding as an additional theme layer.

Tags: must, quality, m0

Needs: dsn

#### DS-4 — Themes cannot break accessibility or safety
`req~ds-4~1`

If a theme fails any WCAG contrast pair (1.4.3, 1.4.11) or changes a safety-relevant token (triage categories, alarms, status), then ORDO shall reject the theme.

Tags: must, quality, m0

Needs: dsn

#### DS-5 — Accessible components
`req~ds-5~1`

Components are accessible by default (semantics, focus handling, keyboard, labels), so module developers get accessibility without extra effort

Tags: must, quality

Needs: dsn

#### DS-6 — Layered themes
`req~ds-6~1`

ORDO shall treat themes as data that can be layered like profiles (e.g. ORDO default theme → organisation theme).

Tags: must, quality, m0

Needs: dsn

#### DS-7 — Open-source building blocks
`req~ds-7~1`

Uses existing open-source building blocks where suitable (token tooling, fonts, icons, test tools), under licences compatible with AGPL-3.0

Tags: must, quality

Needs: dsn
