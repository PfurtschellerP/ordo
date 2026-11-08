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

- The app shell is kept on the device by a service worker (`sw.js`), so ORDO starts
  without a connection; it is installable via `manifest.webmanifest`.
- Pairing by typing the device key (from `ordo generate-device-key`); QR pairing follows.
- Browser storage (`localStorage`) holds the device key and data keys without extra
  protection.
- Texts are German in `src/text.rs`; they move to Fluent translation files later.

End-to-end UI test: Playwright in `tests/e2e/`, see [tests/README.md](../../tests/README.md#end-to-end-tests).
