# Local demo

Try ORDO on your own computer: one server, Dex with demo accounts, the web app.

```sh
deploy/demo/demo.sh          # start on this computer: http://127.0.0.1:18090/
deploy/demo/demo.sh lan      # start for phones and tablets in the local network (below)
deploy/demo/demo.sh stop     # stop (data stays in deploy/demo/.data)
deploy/demo/demo.sh reset    # stop and delete the demo data
```

Needs Docker or Podman (for Dex), Rust, and for the first start `trunk` and
`wasm-bindgen-cli` to build the app. Accounts, password `password`:

| Account | In ORDO |
|---|---|
| `dora@example.org` | administrator (`ORDO_ADMINS=dora`): plans incidents, gives roles, sees all devices |
| `anna@example.org`, `ben@example.org`, `eva@example.org` | whatever role the incident gives them |

Roles are given per incident ([ADR-0046](../../docs/adr/0046-roles-granted-per-incident.md)):

1. Log in as Dora, plan an incident ("Einsatz anlegen"); you lead it.
2. Under *Einsatzleitung – Führung*, give roles to `anna@localhost` (Sanitätsdienst) and
   `ben@localhost` (Einsatzleitung); "Einsatz beginnen" starts it.
3. Log in as Anna or Ben in another browser profile or private window (every profile is
   one device) and open the incident from the list.

For the situation map, log in as Dora and open *Menü – Kartengrundlage*. Enter a centre
(e.g. `48,37` and `10,90`) and a radius, then "Kartengrundlage erstellen". The demo server
downloads that region from the Protomaps world map, so the computer needs internet access;
a radius of 20 km can take a few minutes. Every device then sees the map under
*Lage*, and can keep it offline there.

Pair a shared tablet by QR code: "Mit QR-Code koppeln", then open the shown address in
another window and log in there.

## On devices in the local network

`demo.sh lan` lets phones and tablets in the same network use the demo. ORDO listens on all
interfaces (`0.0.0.0`), and the addresses Dex and ORDO announce are this computer's network
address (found automatically; `ORDO_DEMO_HOST=192.168.1.20 deploy/demo/demo.sh lan` sets
it).

The app needs a *secure context* (WebCrypto for the device key and the encrypted storage,
the service worker, offline storage). Browsers grant it to `http://localhost`, not to
`http://192.168.…`, so in the network ORDO is reached over HTTPS: Caddy with its own local
certificate authority, at `https://<address>:18443/`. Each device trusts that authority
once:

1. Open `http://<address>:18080/ordo-demo-root.crt` on the device (only the public root
   certificate is offered there).
2. Install and trust it:
   - **Android:** Settings → Security → More security settings → Encryption & credentials
     → Install a certificate → CA certificate (the wording varies by vendor).
   - **iPhone, iPad:** allow the downloaded profile, install it under Settings → General →
     VPN & Device Management, then switch it on under Settings → General → About →
     Certificate Trust Settings.
   - **Firefox** (its own certificate store): Settings → Privacy & Security → Certificates
     → View Certificates → Authorities → Import, "trust this CA to identify websites".
   - **Chrome, Edge, Safari on a computer:** add it to the system's trusted roots.
3. Open `https://<address>:18443/` and log in.

The firewall of this computer must let the devices reach TCP 18443 (ORDO), 18080 (the
certificate) and 5556 (login), e.g. `sudo firewall-cmd --add-port={18443,18080,5556}/tcp`
on Fedora (until the next reboot). The certificate authority stays in
`deploy/demo/.data/caddy` and is reused at the next start, so devices trust it once;
`demo.sh reset` deletes it. Remove the certificate from the devices when the test is over.

Not for real use: fixed passwords, demo certificates, the login page (Dex) in plain HTTP.
