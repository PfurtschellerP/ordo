# Security Policy

ORDO is software for emergency services. We take security problems seriously and
appreciate responsible disclosure.

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Report them privately through GitHub: open the repository's **Security** tab and choose
**Report a vulnerability**. Please include:

- what is affected (component, version or commit),
- how to reproduce the problem,
- the possible impact, for example on patient data or on the integrity of incident logs.

We will confirm receipt, keep you informed about the fix, and credit you in the advisory
if you wish.

## Supported versions

ORDO is in the planning stage (pre-alpha). Until the first release, only the latest
state of the `main` branch is supported.

## Measures in this repository

- Dependencies are checked for known vulnerabilities and licences on every change
  (`cargo deny check`), and Dependabot proposes updates weekly.
- GitHub Actions are pinned to full commit hashes.
- ORDO's own code contains no `unsafe` Rust (`unsafe_code = "forbid"`).

## Cyber Resilience Act

ORDO is non-commercial free software; the project is not a manufacturer under the EU
Cyber Resilience Act. Whoever places ORDO on the market commercially, for example on a
preinstalled field server, is the manufacturer of that product. We support them with
this disclosure process and, planned, an SBOM and signed build provenance for every
release. Details: [docs/standards.md](docs/standards.md#ordos-role-under-the-cyber-resilience-act).
