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
| End-to-end tests in the browser | [`e2e/`](e2e/): real `ordo serve`, web app, headless Firefox | see below |

## End-to-end tests

`e2e/scenario.py` drives the web app in headless Firefox through Firefox's built-in
remote protocol (Marionette); it needs Python 3 and Firefox, nothing else.
`e2e/audit.py` checks the rendered pages in every theme and at two widths: text contrast,
table layout, touch-target size and reflow.

```sh
cargo build -p ordo-server
(cd platform/ordo-app && trunk build --release)
python3 tests/e2e/scenario.py target/debug/ordo platform/ordo-app/dist /tmp/ordo-e2e
```

Screenshots and the server log end up in the work directory (last argument).

## Traceability

Acceptance and end-to-end tests mark what they verify with `itest` coverage tags, unit
tests with `utest` (see [requirements](../docs/requirements/README.md#traceability-chain)).
