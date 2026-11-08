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
