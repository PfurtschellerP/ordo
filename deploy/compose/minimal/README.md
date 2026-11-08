# ORDO — minimal Docker Compose setup

One ORDO server with SQLite, [Dex](https://dexidp.io) for login and
[Caddy](https://caddyserver.com) for automatic HTTPS. Suitable for one organisation or
district, at a station with internet and on a field server without it.

## Start

```sh
cd deploy/compose/minimal
cp .env.example .env              # set ORDO_DOMAIN and ORDO_OIDC_CLIENT_SECRET
cp dex.yaml.example dex.yaml      # accounts, or the connector to your organisation's login

# Master key: encrypts all keys stored by the server. Keep a backup in a safe place —
# without it, stored clinical data cannot be read.
docker compose build
docker compose run --rm --no-deps --entrypoint /usr/local/bin/ordo ordo generate-key > master.key
sudo chown 65532:65532 master.key && sudo chmod 400 master.key   # the server runs as user 65532

docker compose up -d
docker compose ps                 # ordo should become "healthy"
```

Endpoints (through Caddy): the app at `https://<domain>/`, login at `/dex`, sync at
`wss://<domain>/sync`, `/healthz`, `/readyz`, `/metrics`.

## Login

ORDO stores no passwords and talks only to the bundled Dex
([ADR-0038](../../../docs/adr/0038-login-and-device-pairing.md),
[ADR-0039](../../../docs/adr/0039-dex-in-every-installation.md)). Two ways to fill in
`dex.yaml`:

- **Very small organisations: local accounts** (`staticPasswords`). Groups of an
  account become its ORDO roles, e.g. `medic`, `commander`.
- **Everyone else: your organisation's OpenID Connect provider** (Microsoft Entra ID,
  Keycloak, Authentik, Google, …) as a Dex connector; see the commented example in
  `dex.yaml.example`. Register Dex there with the redirect URI `https://<domain>/dex/callback`
  and let it send the groups that become ORDO roles.

ORDO itself speaks only OpenID Connect. Dex could also reach LDAP or SAML directories,
but ORDO does not support or test that; prefer your provider's OpenID Connect.

Changes to `dex.yaml` take effect after `docker compose restart dex`.

**Lost or stolen device:** its person revokes it under *Organisation – Geräte* on any of
their other devices; accounts in the group `admin` see and revoke every device. The device
is cut off at once and deletes its key and data as soon as it connects again.

## Field servers without internet

Devices that were paired before the deployment keep working without the identity provider
until their access timeout: each connection is proven with the device's own key, which the
field server checks itself. Plan it like this:

- **Pair the devices before you leave** (at the station, with internet): log in on every
  tablet or pair it by QR code.
- **Mind the access timeout:** a device login lasts 48 hours after its person was last
  verified by the identity provider, then the device locks itself until the person logs
  in again ([ADR-0040](../../../docs/adr/0040-access-ends-only-on-positive-evidence.md)).
  For longer deployments without internet, set `ORDO_ACCESS_TIMEOUT_HOURS` (up to 720 =
  30 days) before you leave. People with local accounts of the field server's Dex can log
  in again on site.
- **People who join on scene** log in with a local account of the field server's Dex
  (`staticPasswords`). An upstream provider cannot be reached offline; Dex does not cache
  its logins. (Planned: an already paired person with the right role approves a new device
  directly — like substituting a player at half-time.)
- **Certificates:** iPads and other browsers run the offline app and keep keys only over
  HTTPS. Without internet, Caddy cannot get or renew a Let's Encrypt certificate on site.
  Get it beforehand (e.g. a DNS challenge for `field1.ordo.your-org.example`, renewed at
  the station) or use your organisation's own certificate authority, installed on the
  devices by device management. The field router's DNS must resolve the domain to the
  field server.

## Notes

- The ORDO container is read-only, runs as a non-root user without capabilities; data
  lives in the volume `ordo-data`, Dex keeps its state in `dex-data`.
- ORDO trusts the system's certificate authorities. With your organisation's own
  certificate authority, mount its certificate into the ORDO container and set
  `SSL_CERT_FILE`.
- For development without login, `seed.toml.example` shows how to pair devices by hand.
- Back up the volumes `ordo-data` and `dex-data` **and** `master.key`.
