# ORDO

**O**perations, **R**esponse, **D**ispatch & **O**rganisation

ORDO is an open-source platform for on-scene coordination of emergency services.
Fire services, emergency medical services and police share one platform, and each
gets its own package with dedicated views, connected to the others through defined
interfaces.

- **Works offline**: every device keeps working without network and syncs later.
- **Need to know**: patient data stays with medical staff and never reaches devices
  that don't need it.
- **Easy to run**: one binary or one `docker compose up`, from a Raspberry Pi to a
  cluster.
- **Open**: open source, open specifications, federation between organisations.

Built first for volunteer organisations in Germany, designed to be international.

> **Status: planning (pre-alpha).** There is no usable software yet.

## Documentation

- [Requirements](docs/requirements.md)
- [Architecture](docs/architecture.md)
- [Specifications](spec/README.md)

## Licence

- ORDO software: [GNU Affero General Public License v3.0 or later](LICENSE)
- Documentation: [CC BY 4.0](LICENSES/CC-BY-4.0.txt)
- Specifications: [CC BY 4.0](LICENSES/CC-BY-4.0.txt) (texts) and
  [Apache-2.0](LICENSES/Apache-2.0.txt) (schemas, conformance tests)

Details per file are in [`REUSE.toml`](REUSE.toml).
