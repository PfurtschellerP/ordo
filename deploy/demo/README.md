# Local demo

Try ORDO on your own computer: one server, Dex with demo accounts, the web app.

```sh
deploy/demo/demo.sh          # start: http://127.0.0.1:18090/
deploy/demo/demo.sh stop     # stop (data stays in deploy/demo/.data)
deploy/demo/demo.sh reset    # stop and delete the demo data
```

Needs Docker or Podman (for Dex), Rust, and for the first start `trunk` and
`wasm-bindgen-cli` to build the app. Accounts, password `password`:

| Account | Roles | Sees |
|---|---|---|
| `anna@example.org` | medic | Sanität – UHS, patient records |
| `ben@example.org` | commander | Einsatzleitung: counts, units — no patient data |
| `dora@example.org` | admin | Organisation – Geräte: all devices |
| `eva@example.org` | medic, commander | both |

Open each person in its own browser profile or private window: every browser profile is
one device. The first person creates an incident ("Neuen Einsatz anlegen"); others enter
its ID (shown in the header; the full ID is in its tooltip) to join. Pair a second device
by QR code: "Mit QR-Code koppeln", then open the shown address in another window.

Not for real use: fixed passwords, plain HTTP on `localhost`.
