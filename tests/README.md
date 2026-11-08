# Tests

ORDO follows the usual Rust layout. Tests close to the code stay with the code; tests that
span several crates or the running system live here.

| Kind | Where | Run |
|---|---|---|
| Unit tests | next to the code (`#[cfg(test)]` modules) | `cargo test --workspace` |
| Integration tests of one crate | `<crate>/tests/`, e.g. `platform/ordo-server/tests/` | `cargo test --workspace` |
| Benchmarks | `<crate>/benches/`, e.g. `platform/ordo-server/benches/` | `cargo bench -p ordo-server` |
| Acceptance tests across crates | [`acceptance/`](acceptance/): server and devices on a simulated network | `cargo test -p ordo-acceptance` |
| Acceptance scenarios (Gherkin) | [`acceptance/features/`](acceptance/features/) | become executable with the MVP |
| End-to-end tests in the browser | [`e2e/`](e2e/): real `ordo serve`, web app, Firefox, Chromium and WebKit | see below |

## End-to-end tests

[`e2e/scenario.spec.ts`](e2e/scenario.spec.ts) drives the web app with
[Playwright](https://playwright.dev) in Firefox, Chromium and WebKit (the engine of
Safari and of every browser on iPads), against a real `ordo serve`. It uses real key
presses, file uploads, phone-sized viewports and the browser's back button.

[`e2e/support/audit.ts`](e2e/support/audit.ts) checks every view in every theme at phone
and laptop width: [axe-core](https://github.com/dequelabs/axe-core) for WCAG 2.2 A/AA,
plus our own checks for what axe does not cover (44 px touch targets, reflow, broken
table rows, fixed bars covering content). Playwright's Firefox build crashes inside axe
on some pages, so axe runs in Chromium and WebKit, and Firefox runs our own checks.

```sh
cargo build -p ordo-server
(cd platform/ordo-app && trunk build --release)
cd tests/e2e
npm ci
npx playwright install --with-deps firefox chromium webkit
npx playwright test                    # all three browsers
npx playwright test --project=firefox  # one browser
```

`ORDO_BIN` and `ORDO_APP_DIST` override the paths of the server and the app;
screenshots go to `tests/e2e/screenshots/<browser>/`, failure traces to
`tests/e2e/test-results/` (`npx playwright show-trace …`).

## Traceability

Acceptance and end-to-end tests mark what they verify with `itest` coverage tags, unit
tests with `utest` (see [requirements](../docs/requirements/README.md#traceability-chain)).
