# ORDO web app (prototype)

Leptos, client-side rendered, compiled to WebAssembly with [Trunk](https://trunkrs.dev).
Served by `ordo serve` when `ORDO_APP_DIR` points to the build output.

```sh
cargo install --locked trunk wasm-bindgen-cli   # wasm-bindgen-cli in the version from Cargo.lock
sudo dnf install clang                          # or: apt install clang (compiles SQLite for the browser)
cd platform/ordo-app && trunk build --release   # output: dist/
```

The build also produces the device storage worker from `platform/ordo-store`
(`ordo-store_loader.js`, `ordo-store.js`, `ordo-store_bg.wasm`): SQLite on the browser's
file system, in its own Cargo workspace (ADR-0034).

Workspaces: **Sanität** (register patients, UHS board, re-triage) and **Einsatzleitung**
(patient counts per UHS). The device works offline; input is kept in the browser and
sent when the connection returns.

Prototype limitations:

- The app shell is kept on the device by a service worker (`sw.js`), so ORDO starts
  without a connection; it is installable via `manifest.webmanifest`.
- Pairing by logging in or by QR code; typing the device key (from
  `ordo generate-device-key`) remains for development.
- Device data is encrypted (ADR-0041) and kept in SQLite on OPFS (ADR-0034); without OPFS
  in workers the app falls back to `localStorage`.
- Texts are German in `src/text.rs`; they move to Fluent translation files later.

End-to-end UI test: Playwright in `tests/e2e/`, see [tests/README.md](../../tests/README.md#end-to-end-tests).
