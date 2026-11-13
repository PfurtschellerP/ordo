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
# The client secret from .env, as a file for ORDO (Dex reads it from .env).
(. ./.env && printf %s "$ORDO_OIDC_CLIENT_SECRET" > oidc-client-secret)
sudo chown 65532:65532 oidc-client-secret && sudo chmod 400 oidc-client-secret

# Certificates for Dex's API, through which ORDO manages local accounts (ADR-0060).
mkdir dex-api && sudo chown 65532:65532 dex-api
docker compose run --rm --no-deps -v "$PWD/dex-api:/certs" ordo generate-dex-certs /certs dex
sudo chown 1001:1001 dex-api/dex-key.pem && sudo chmod 400 dex-api/dex-key.pem   # Dex runs as user 1001
sudo chmod 400 dex-api/ordo-key.pem

docker compose up -d
docker compose ps                 # ordo should become "healthy"
```

Endpoints (through Caddy): the app at `https://<domain>/`, login at `/dex`, sync at
`wss://<domain>/sync`, `/healthz`, `/readyz`, `/metrics`.

## Attachments in Garage (optional)

Photos and other attachments are stored encrypted, as ciphertext only
([ADR-0033](../../../docs/adr/0033-attachments-in-an-object-store.md)). By default they
live in the volume `ordo-data`. For more space, or to keep them on separate disks, put
them into [Garage](https://garagehq.deuxfleurs.fr), an S3-compatible store that is free
software as a whole:

```sh
openssl rand -hex 32 > garage-rpc.key
s3_secret=$(openssl rand -hex 32) && echo "$s3_secret" > s3-secret.key
sudo chown 0:0 garage-rpc.key && sudo chmod 400 garage-rpc.key        # Garage runs as root
sudo chown 65532:65532 s3-secret.key && sudo chmod 400 s3-secret.key  # ORDO as user 65532
echo "ORDO_S3_ACCESS_KEY_ID=GK$(openssl rand -hex 12)" >> .env
. ./.env
alias compose='docker compose -f compose.yaml -f compose.garage.yaml'
compose up -d garage
# Once: layout, bucket "ordo" and ORDO's key.
../../garage/setup.sh "docker compose -f compose.yaml -f compose.garage.yaml exec -T garage /garage" \
  ordo "$ORDO_S3_ACCESS_KEY_ID" "$s3_secret"
unset s3_secret
compose up -d
```

ORDO checks at start that the bucket answers and stops otherwise. Back up the volumes
`garage-meta` and `garage-data` as well. Moving existing attachments from `ordo-data`
into Garage is not automated yet.

## Login

ORDO stores no passwords and talks only to the bundled Dex
([ADR-0038](../../../docs/adr/0038-login-and-device-pairing.md),
[ADR-0039](../../../docs/adr/0039-dex-in-every-installation.md)). Two ways to fill in
`dex.yaml`:

- **Very small organisations: local accounts.** Administrators create, reset and delete
  them in ORDO (*Administration – Zugriff – Konten*); ORDO passes the change to Dex through its API
  and keeps no passwords ([ADR-0060](../../../docs/adr/0060-local-accounts-managed-in-ordo.md)).
  Accounts written in `dex.yaml` (`staticPasswords`) still work but can only be changed
  there. Field servers keep local accounts for people who join on scene.
- **Everyone else: your organisation's OpenID Connect provider** (Microsoft Entra ID,
  Keycloak, Authentik, Google, …) as a Dex connector. Administrators add it in ORDO
  (*Administration – Zugriff – Anmeldung*): ORDO writes it to Dex after a test login
  through it, and never shows the client secret again
  ([ADR-0063](../../../docs/adr/0063-upstream-login-configured-in-ordo.md)); Dex needs
  `DEX_API_CONNECTORS_CRUD=true`, as in `compose.yaml`. It can also be written into
  `dex.yaml` (commented example in `dex.yaml.example`). Register Dex at the provider with
  the redirect URI `https://<domain>/dex/callback`.

The identity provider only says who someone is. Roles are given per incident in ORDO
([ADR-0046](../../../docs/adr/0046-roles-granted-per-incident.md)): the administrators in
`ORDO_ADMINS` plan incidents and give roles (*Einsatzleitung – Personal*); whoever leads an
incident gives roles in it.

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

## Devices

- Open ORDO only over HTTPS: the app encrypts everything it stores on the device and
  refuses to store anything without a secure connection
  ([ADR-0041](../../../docs/adr/0041-encryption-at-rest-on-devices.md)).
- Enforce a screen lock and disk encryption on every tablet, phone and laptop (device
  management). The app's own encryption makes unpairing and remote wipe effective; against
  a stolen device that is switched off, the operating system's disk encryption protects.

## Notes

- The ORDO container is read-only, runs as a non-root user without capabilities; data
  lives in the volume `ordo-data`, Dex keeps its state in `dex-data`.
- ORDO trusts the system's certificate authorities. With your organisation's own
  certificate authority, mount its certificate into the ORDO container and set
  `SSL_CERT_FILE`.
- For development without login, `seed.toml.example` shows how to pair devices by hand.
- Administrators keep the organisation's vehicles in *Administration – Stammdaten*.
  If they are kept elsewhere, `vehicles.toml.example` shows a file for them
  (`ORDO_VEHICLES_FILE`); its vehicles are shown in ORDO but not changed there.
- Back up the volumes `ordo-data` and `dex-data` (with Garage also `garage-meta` and
  `garage-data`) **and** `master.key`.
