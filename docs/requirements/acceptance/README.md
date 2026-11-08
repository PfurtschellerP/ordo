# Acceptance scenarios

Key workflows described as Gherkin scenarios (Given / When / Then). Domain experts can
read and check them; with the MVP they become executable tests (Rust `cucumber` crate)
and get OpenFastTrace IDs, so that they trace to the requirements they verify.

Until then the scenarios are not executable and carry no OpenFastTrace IDs.
