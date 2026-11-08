# 1. Introduction and goals

> Part of the [ORDO architecture](README.md) (arc42).

ORDO is an open-source platform for on-scene coordination of emergency services. What it
must do is described in the [requirements](../requirements/README.md); this chapter
summarises what shapes the architecture.

## 1.1 Quality goals

| Priority | Quality goal | Scenario (short) | Requirements |
|---|---|---|---|
| 1 | **Works offline** | A medic registers patients without network; nothing is lost and it syncs later | OFF-1…5, UX-5 |
| 2 | **Need to know** | A fire tablet shows patient counts but never receives patient records | SEC-8…10, CONN-3 |
| 3 | **Usable and accessible under stress** | A volunteer uses the core task of their role without training, with gloves, at night, with a screen reader | UX-1…9, A11Y-1…11 |
| 4 | **Trustworthy record** | Any change to a logged event is detectable | LOG-3, FED-6 |
| 5 | **Resource-efficient** | Runs on a 5-year-old tablet and a Raspberry Pi | PERF-1…6 |
| 6 | **Extensible** | A new service package or country profile needs no change to existing code | MOD-1…7, I18N-4…8 |

## 1.2 Stakeholders

| Stakeholder | Expectation |
|---|---|
| Responders and leads (fire, med, police) | Fast, reliable, accessible tool that works offline ([roles](../requirements/01-vision-and-scope.md#3-users-and-roles)) |
| Volunteer organisations | Easy installation and operation, no IT staff needed |
| Public bodies (municipalities, counties) | GDPR, accessibility (EN 301 549), open licence, conformance report |
| Data-protection officers, auditors | Traceable requirements, compartments, audit trail |
| Developers and contributors | Clear module boundaries, open standards, documented decisions |
| Other system vendors | Open, versioned interface specifications |

## 1.3 Architecture drivers

These requirements shape the architecture the most:

| Driver | From | Consequence |
|---|---|---|
| Several services on one platform | MOD-1…5, CONN-1…5 | Layers: platform → shared modules → service packages; connectors between packages |
| Patient data must stay with medical staff | SEC-8…10, EPCR-7 | **Data compartments** with own keys, access rules, sync scope and retention |
| Works fully offline, devices sync later | OFF-1…5 | Local-first clients: every device has its own copy of the data it may see |
| Tamper-evident, append-only records | LOG-3, EPCR-4 | Store changes as **events**; logs and records are derived from them |
| Resource-efficient everywhere | PERF-1, PERF-4, PERF-5 | **Rust** on server and client (WebAssembly); compact wire format |
| Easy to run for volunteers | OPS-1, UX-9 | **One static binary**, SQLite by default, ready-made packages |
| Self-hosted *and* later hosted | OPS-4 | Multi-tenancy designed in from the start |
| Tablet, laptop, phone | UX-6 | One cross-platform client (installable web app, later native shell) |
| Small first, cloud-native later | OPS-1, OPS-5…11 | Modular monolith; stateless process; containers and Compose from day one; Helm later |
| Federation between organisations | FED-1…7 | Server-qualified IDs and signed events from day one; same sync protocol for devices and servers |
| Open interfaces | API-1…4 | Spec-first, published specifications with conformance tests |
| Regional profiles (DE first) | I18N-2, I18N-4…8 | Terminology, symbols, codes and forms are layered data packages |
