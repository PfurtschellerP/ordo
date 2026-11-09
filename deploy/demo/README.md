# Local demo

Try ORDO on your own computer: one server, Dex with demo accounts, the web app.

```sh
deploy/demo/demo.sh          # start: http://127.0.0.1:18090/
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
2. Under *Einsatzleitung – Personal*, give roles to `anna@localhost` (Sanitätsdienst) and
   `ben@localhost` (Einsatzleitung); "Einsatz beginnen" starts it.
3. Log in as Anna or Ben in another browser profile or private window (every profile is
   one device) and open the incident from the list.

Pair a shared tablet by QR code: "Mit QR-Code koppeln", then open the shown address in
another window and log in there.

Not for real use: fixed passwords, plain HTTP on `localhost`.
