# ORDO web app (prototype)

Leptos, client-side rendered, compiled to WebAssembly with [Trunk](https://trunkrs.dev).
Served by `ordo serve` when `ORDO_APP_DIR` points to the build output.

```sh
cargo install --locked trunk wasm-bindgen-cli   # wasm-bindgen-cli in the version from Cargo.lock
cd platform/ordo-app && trunk build --release   # output: dist/
```

Workspaces: **Sanität** (register patients, UHS board, re-triage) and **Einsatzleitung**
(patient counts per UHS). The device works offline; input is kept in the browser and
sent when the connection returns.

Prototype limitations:

- Pairing by typing the device key (from `ordo generate-device-key`); QR pairing follows.
- Browser storage (`localStorage`) holds the device key and data keys without extra
  protection.
- Texts are German in `src/text.rs`; they move to Fluent translation files later.

End-to-end UI test: `python3 acceptance/ui/scenario.py target/debug/ordo platform/ordo-app/dist /tmp/ordo-ui`.
