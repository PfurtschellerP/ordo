# 10. Quality requirements

> Part of the [ORDO architecture](README.md) (arc42).

Quality goals: [chapter 1.1](01-introduction-and-goals.md#11-quality-goals). All quality
requirements: [requirements/07-quality.md](../requirements/07-quality.md).

## 10.1 Quality scenarios

| Quality | Scenario | Expected response | Requirements |
|---|---|---|---|
| Offline | A medic registers 20 patients while the tablet has no network for 30 minutes | All registrations are stored locally; after reconnecting they sync without user action; no data is lost | OFF-1, OFF-3, OFF-4, UX-5 |
| Offline | Two devices change the same unit's status while disconnected | Last writer wins per field (by HLC); the conflict is visible in the log | OFF-4 |
| Need to know | A fire command tablet joins an incident with patients in UHS | It shows patient counts per category; its local storage contains no clinical events | SEC-9, CONN-3, PAT-6 |
| Integrity | Someone changes a stored event on the server | Signature check and hash chain detect the change | LOG-3, FED-6 |
| Performance | A change is made on one tablet while others are connected | It appears on the other devices within 2 s | PERF-2 |
| Accessibility | A user with a screen reader registers a patient | Every control is labelled and reachable by keyboard; the triage category is announced as text | A11Y-3, A11Y-4 |
| Accessibility | An organisation uploads a theme with low contrast | The theme is rejected with the failing pairs | DS-4 |
| Extensibility | A new country profile is added | No code change or recompiling | I18N-5 |

## 10.2 Resource budget

- **WASM size:** `opt-level = "z"`, LTO, `wasm-opt`; packages and rarely used views are
  loaded on demand, so a fire tablet never downloads the ePCR code. Initial load
  < 1 MB compressed.
- **DOM work:** fine-grained reactivity; long lists (patients, log) virtualised.
- **Battery:** no polling (WebSocket push), batched sync, GPS only when needed.
- **Server:** one async process; idle memory target < 50 MB.
- **Measured in CI:** benchmarks for projections, sync and encryption; budgets checked
  on the reference tablet.
