# Acceptance scenarios

Key workflows described in Gherkin (Given / When / Then), so that domain experts can read
and check them. Today they are implemented by hand as Rust tests in
[`../tests/`](../tests/) and as the browser scenario in [`../../e2e/`](../../e2e/); with
the MVP a Gherkin runner (Rust `cucumber` crate) will execute them directly and they get
OpenFastTrace IDs.
