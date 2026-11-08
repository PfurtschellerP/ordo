# ORDO — minimal Docker Compose setup

One ORDO server with SQLite, behind [Caddy](https://caddyserver.com) for automatic
HTTPS. Suitable for one organisation or district.

## Start

```sh
cd deploy/compose/minimal
cp .env.example .env              # set ORDO_DOMAIN
cp seed.toml.example seed.toml    # users and devices

# Master key: encrypts all keys stored by the server. Keep a backup in a safe place —
# without it, stored clinical data cannot be read.
docker compose build
docker compose run --rm --no-deps --entrypoint /usr/local/bin/ordo ordo generate-key > master.key
sudo chown 65532:65532 master.key && sudo chmod 400 master.key   # the server runs as user 65532

docker compose up -d
docker compose ps                 # ordo should become "healthy"
```

Endpoints (through Caddy): `wss://<domain>/sync`, `https://<domain>/healthz`,
`/readyz`, `/metrics`.

## Notes

- The ORDO container is read-only, runs as a non-root user without capabilities; data
  lives in the volume `ordo-data`.
- Changes to `seed.toml` take effect after `docker compose restart ordo`.
- Back up the volume `ordo-data` **and** `master.key`.
