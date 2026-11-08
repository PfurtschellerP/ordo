# ORDO — minimal Docker Compose setup

One ORDO server with SQLite, behind [Caddy](https://caddyserver.com) for automatic
HTTPS. Suitable for one organisation or district.

## Start

```sh
cd deploy/compose/minimal
cp .env.example .env              # set ORDO_DOMAIN
cp seed.toml.example seed.toml    # users and devices paired by hand (development)
openssl rand -hex 32 > oidc-client-secret   # needed even without login, see below

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

## Login (optional)

ORDO stores no passwords; people log in with an OpenID Connect identity provider
([ADR-0038](../../../docs/adr/0038-login-and-device-pairing.md)). Every setup needs the
secret file, even without login:

```sh
openssl rand -hex 32 > oidc-client-secret
```

- **Your organisation's provider** (Keycloak, Authentik, Microsoft Entra ID, …): register
  ORDO as a client with the redirect URI `https://<domain>/auth/callback` and, for QR
  pairing, the device authorization grant; put its issuer URL into `ORDO_OIDC_ISSUER` in
  `.env` and the client secret into `oidc-client-secret`. Groups in the `groups` claim
  become ORDO roles.
- **Bundled Dex** with local accounts: `cp dex.yaml.example dex.yaml`, edit it (domain,
  secret, accounts), set `ORDO_OIDC_ISSUER=https://<domain>/dex` in `.env`, then
  `docker compose --profile login up -d`.

The ORDO container trusts the system's certificate authorities. For a provider with your
organisation's own certificate authority, mount its certificate and set `SSL_CERT_FILE`.

## Notes

- The ORDO container is read-only, runs as a non-root user without capabilities; data
  lives in the volume `ordo-data`.
- Changes to `seed.toml` take effect after `docker compose restart ordo`.
- Back up the volume `ordo-data` **and** `master.key`.
