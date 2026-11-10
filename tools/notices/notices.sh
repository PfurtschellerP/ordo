#!/bin/sh
# Writes the licence notices of the Rust crates ORDO ships (LIC-1, ADR-0055) as an HTML
# fragment for the app's page "Lizenzen und Quellcode": the server and app workspace,
# then the storage worker (its own workspace). Run by the web app's Trunk hook.
#
#   tools/notices/notices.sh OUT_FILE
#
# Without cargo-about the fragment stays empty and the page says so, unless
# ORDO_NOTICES_REQUIRED=1 (CI and release builds), which makes it an error.
set -eu
out="$1"
repo="$(cd "$(dirname "$0")/../.." && pwd)"
mkdir -p "$(dirname "$out")"
if ! command -v cargo-about >/dev/null 2>&1; then
  if [ "${ORDO_NOTICES_REQUIRED:-0}" = 1 ]; then
    echo "notices: cargo-about is missing (cargo install --locked cargo-about@0.9.3 --features cli)" >&2
    exit 1
  fi
  echo "notices: cargo-about not installed; the licence notices of the crates are left out" >&2
  : > "$out"
  exit 0
fi
# Up to date: nothing the list depends on is newer.
if [ -s "$out" ] && [ -z "$(find "$repo/Cargo.lock" "$repo/platform/ordo-store/Cargo.lock" "$repo/about.toml" "$repo/tools/notices/crates.hbs" -newer "$out")" ]; then
  exit 0
fi
tmp="$out.tmp"
cargo-about generate --fail -c "$repo/about.toml" -m "$repo/Cargo.toml" "$repo/tools/notices/crates.hbs" > "$tmp"
cargo-about generate --fail -c "$repo/about.toml" -m "$repo/platform/ordo-store/Cargo.toml" "$repo/tools/notices/crates.hbs" >> "$tmp"
mv "$tmp" "$out"
