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
| Fuzzing of all network input | [`fuzz/`](fuzz/): cargo-fuzz targets, nightly Rust, run by ClusterFuzzLite in CI (ADR-0059) | `cargo +nightly fuzz run --fuzz-dir tests/fuzz client_message` |

Note on `tests/fuzz`: its `rust-toolchain.toml` asks for nightly Rust, so any `cargo` or
`rustup` command run *inside* that directory installs the nightly toolchain. Run the
targets from the repository root as shown, or in a container like CI does.

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

Devices are paired for real: Anna logs in with [Dex](https://dexidp.io), Ben's tablet is
paired by QR code from a second browser ("phone"). The test starts Dex in Docker
(`support/dex.ts`, configuration `support/dex.yaml`); Docker or Podman must be available.

```sh
cargo build -p ordo-server
(cd platform/ordo-app && trunk build --release)
cd tests/e2e
npm ci
npx playwright install --with-deps firefox chromium webkit
npx playwright test                    # all three browsers
npx playwright test --project=firefox  # one browser
```

`ORDO_BIN` and `ORDO_APP_DIST` override the paths of the server and the app. The tests
use ports 18090 (ORDO) and 5556 (Dex), the same as the local demo; while the demo runs,
set `ORDO_E2E_PORT` and `ORDO_E2E_DEX_PORT` (e.g. 18190 and 5566). The tests refuse to
start against an ORDO that is already listening. The app's build needs `cargo-about` for
the licence page (`cargo install --locked cargo-about --version 0.9.3 --features cli`);
without it the page says the crates' notices are left out, and this test fails. Screenshots go to `tests/e2e/screenshots/<browser>/`, failure traces to
`tests/e2e/test-results/` (`npx playwright show-trace …`).

## Traceability

Acceptance and end-to-end tests mark what they verify with `itest` coverage tags, unit
tests with `utest` (see [requirements](../docs/requirements/README.md#traceability-chain)).
