# 7. Deployment view

> Part of the [ORDO architecture](README.md) (arc42).

ORDO stays **one program** (a modular monolith), but is built from the start to run
well in containers and orchestrators. Small setups get one file; large setups get the
same program, scaled out.

## 7.1 Cloud-ready by design

| Principle | How ORDO does it |
|---|---|
| Configuration | Environment variables or a config file; nothing baked into the image |
| State | Only in the database and file/object storage; the process itself is stateless |
| Logs | Structured JSON to stdout |
| Health | `/healthz` (alive), `/readyz` (database reachable, migrations done) |
| Observability | Prometheus metrics at `/metrics`; optional OpenTelemetry tracing |
| Shutdown | Graceful on `SIGTERM`: stop accepting, tell devices to reconnect elsewhere, finish writes |
| Migrations | Automatic on start (single instance) or as a separate `ordo migrate` job (cluster) |
| Files | Map tiles, attachments, backups: local directory **or** S3-compatible object storage |
| Secrets | Environment or mounted files (Docker / Kubernetes secrets); external key management later |

## 7.2 Container image

- Static binary in a minimal image (`scratch` / distroless): no shell, non-root,
  read-only file system, roughly 20–30 MB.
- Multi-architecture: `linux/amd64` and `linux/arm64` (Raspberry Pi, ARM servers).
- Published with every release, signed (cosign), with a software bill of materials.

## 7.3 Deployment options

| Option | Setup | Typical use |
|---|---|---|
| **Single binary** | `ordo serve` + SQLite file; systemd unit | One organisation, mini PC / Raspberry Pi |
| **Compose — minimal** | ORDO (SQLite on a volume) + Caddy for automatic TLS | One organisation or district |
| **Compose — full** | ORDO + PostgreSQL + Caddy | District / county, several organisations |
| **Kubernetes** (Helm, later) | Several ORDO replicas + PostgreSQL (e.g. CloudNativePG) + ingress + object storage | Hosted service, large regions |
| **Edge** (later) | Same binary on a mini PC in the command vehicle | Scenes without internet |

## 7.4 Scaling out

- **SQLite mode:** exactly one instance, by design.
- **PostgreSQL mode:** any number of stateless instances behind a load balancer.
  Devices connect to any instance. New events are passed between instances via
  PostgreSQL `LISTEN/NOTIFY`, so no extra message broker is needed. A broker (e.g. NATS)
  only if measurements ever show a need.
- **Ordering stays correct** with many instances: sequence numbers are assigned inside a
  database transaction per incident.
- **Splitting later without rewriting:** the binary can run selected roles
  (`ordo serve --roles api,sync`, `ordo worker` for reports, retention, adapters). We
  split into separate services only if measurements show it is worth it.

Every record carries an `organisation_id` from day one (multi-tenancy).

## 7.5 Current implementation

| Part | State |
|---|---|
| `ordo serve` (`platform/ordo-server`) | WebSocket sync at `/sync`, SQLite storage, `/healthz`, `/readyz`, `/metrics`, JSON logs, graceful shutdown |
| Container image (`deploy/Containerfile`) | Static binary on distroless, non-root (65532), read-only file system possible, built-in health check |
| Compose — minimal (`deploy/compose/minimal/`) | ORDO + Caddy (automatic HTTPS) |
| Compose — full, Helm, multi-arch release images | Planned |

### 7.5.1 Design items

#### Environment configuration
`dsn~environment-configuration~1`

`ordo serve` reads all settings from environment variables (`ORDO_SERVER_NAME`, `ORDO_LISTEN`, `ORDO_DATA_DIR`, `ORDO_MASTER_KEY_FILE`, `ORDO_SEED_FILE`, `ORDO_PROTECTED`, `ORDO_LOG_FORMAT`); secrets are read from files so they can be mounted as container secrets. Nothing is baked into the image.

Covers:

* [req~ops-7~1](../requirements/07-quality.md#ops-7--external-configuration)

Needs: impl, itest

Tags: must, m2

#### Store before send
`dsn~store-before-send~1`

For every message the server lets the hub decide, stores all resulting changes (new events, new keys) in one SQLite transaction, and only then sends the resulting messages — while the hub is still locked, so every device receives deliveries in sequence order. If storing fails, nothing is sent and the server stops so that it is restarted; devices still have the events in their outbox and send them again.

Covers:

* [req~off-3~1](../requirements/07-quality.md#off-3--automatic-sync)

Needs: impl, itest

Tags: must, m2

#### Wrapped keys at rest
`dsn~wrapped-keys-at-rest~1`

The server stores data keys and its own signing key only wrapped with the master key (XChaCha20-Poly1305, labelled per key), and events exactly as ordered, so clinical payloads stay sealed in the database. The master key comes from a file outside the database; without it, stored keys cannot be opened and the server refuses to start.

Covers:

* [req~sec-10~2](../requirements/07-quality.md#sec-10--health-data-protection)

Needs: impl, itest

Tags: must, m2

