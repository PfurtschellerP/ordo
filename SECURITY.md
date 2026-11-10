# Security Policy

ORDO is software for emergency services. We take security problems seriously and
appreciate responsible disclosure.

## Reporting a vulnerability

Please **do not** open a public issue for security problems.

Report them privately through GitHub:
[report a vulnerability](https://github.com/PfurtschellerP/ordo/security/advisories/new)
(the repository's **Security** tab, **Report a vulnerability**). Please include:

- what is affected (component, version or commit),
- how to reproduce the problem,
- the possible impact, for example on patient data or on the integrity of incident logs.

We follow coordinated disclosure:

- we confirm receipt within **7 days** and tell you our assessment within **14 days**;
- we aim to publish a fix within **90 days**, sooner for problems that expose patient data
  or let someone change incident logs, and keep you informed on the way;
- we publish a [security advisory](https://github.com/PfurtschellerP/ordo/security/advisories)
  with the fix and credit you if you wish.

Please give us that time before you publish details.

## Supported versions

ORDO is in the planning stage (pre-alpha). Until the first release, only the latest
state of the `main` branch is supported.

## Measures in this repository

- Dependencies are checked for known vulnerabilities and licences on every change
  (`cargo deny check`), and Dependabot proposes updates weekly. Advisories that do not
  affect ORDO are listed with the reason in `deny.toml` and `osv-scanner.toml`.
- Static analysis with CodeQL on every change; the
  [OpenSSF Scorecard](https://scorecard.dev/viewer/?uri=github.com/PfurtschellerP/ordo)
  checks the repository's practices weekly.
- GitHub Actions are pinned to full commit hashes.
- ORDO's own code contains no `unsafe` Rust (`unsafe_code = "forbid"`).
- Every build makes a software bill of materials (CycloneDX); releases are signed keyless
  with cosign and carry build provenance ([how to verify](docs/architecture/07-deployment-view.md#77-releases-and-how-to-verify-them)).

## Cyber Resilience Act

ORDO is non-commercial free software; the project is not a manufacturer under the EU
Cyber Resilience Act. Whoever places ORDO on the market commercially, for example on a
preinstalled field server, is the manufacturer of that product. We support them with
this disclosure process, an SBOM and signed build provenance for every release. Details: [docs/standards.md](docs/standards.md#ordos-role-under-the-cyber-resilience-act).
