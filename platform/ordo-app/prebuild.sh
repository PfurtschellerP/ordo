#!/bin/sh
# Runs before every Trunk build, one step after the other (Trunk starts hooks of one stage
# at the same time, and two first runs of cargo would race to install the toolchain):
# the design-token style sheet with its contrast check, then the licence notices of the
# shipped crates (LIC-1, ADR-0055).
set -eu
cd "$(dirname "$0")"
cargo run --quiet --locked -p ordo-tokens -- build --design ../../design --out generated
sh ../../tools/notices/notices.sh generated/third-party/crates.html
