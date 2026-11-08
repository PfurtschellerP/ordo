# 11. Risks and technical debt

> Part of the [ORDO architecture](README.md) (arc42).

| Risk | Mitigation |
|---|---|
| **Building a platform instead of a product** (over-engineering) | Extract the platform from real modules, not the other way round; keep the module API internal until two packages use it; field-test early |
| Offline sync with compartments is complex | Prototype first, with two compartments from day one |
| ePCR brings legal duties (GDPR Art. 9, retention, confidentiality) | Compartments from the start; data-protection review before field use; ePCR in its own phase |
| Feature creep into medical-device territory | Hard rule: document and organise, no decision support ([constraints](../requirements/08-constraints.md)) |
| Rust + WASM has fewer UI contributors than JS | Small design system; good build docs |
| SQLite in WASM / browser storage limits (iOS) | Verify in prototype; persistent storage; Tauri shell later |
| Prototype keeps the device replica in `localStorage` (about 5 MB, rewritten on every change); photos fill it quickly | Base64 instead of hex halves the size; a visible warning when saving fails; next: IndexedDB with append-only writes ([ADR-0028](../adr/0028-patient-photos-in-sealed-events.md)) |
| Map is JavaScript, not Rust | Thin binding layer (`ordo-map`) |
| The map canvas is not accessible | Accessible list view with the same actions; keyboard access to map objects |
| Accessible Leptos component libraries are young | Own components following the WAI-ARIA APG; adopt a library only after evaluation |
| Organisation branding breaks contrast | Themes checked automatically; safety tokens can't be overridden |
| Federation is a large undertaking | Only the foundations now (IDs, signatures, home concept); the rest in phase 4 |
| Maintaining open specs costs time | Spec-first only for sync, federation and connectors; the admin API spec is generated from code |
| Volunteer project, limited time | Small MVP; planned operations (events) as low-risk field tests |
| Safety-critical features (BA monitoring) | Own requirements and tests; never the only safeguard |
