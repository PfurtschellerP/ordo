#!/usr/bin/env bash
# Local ORDO demo: Dex with demo accounts (Docker or Podman) and `ordo serve` on
# http://127.0.0.1:18090. Not for real use: fixed passwords, plain HTTP on localhost.
#
#   deploy/demo/demo.sh          start (builds server and app if needed)
#   deploy/demo/demo.sh stop     stop
#   deploy/demo/demo.sh reset    stop and delete all demo data
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
data="${ORDO_DEMO_DATA:-$here/.data}"
bin="${ORDO_BIN:-${CARGO_TARGET_DIR:-$repo/target}/debug/ordo}"
dist="${ORDO_APP_DIST:-$repo/platform/ordo-app/dist}"
docker="$(command -v docker || command -v podman)"
port=18090

stop() {
  "$docker" rm -f ordo-demo-dex >/dev/null 2>&1 || true
  if [ -f "$data/ordo.pid" ]; then kill "$(cat "$data/ordo.pid")" 2>/dev/null || true; rm -f "$data/ordo.pid"; fi
}

case "${1:-start}" in
  stop) stop; echo "Demo stopped."; exit 0 ;;
  reset) stop; rm -rf "$data"; echo "Demo data deleted."; exit 0 ;;
  start) ;;
  *) echo "usage: $0 [start|stop|reset]" >&2; exit 2 ;;
esac

stop
[ -x "$bin" ] || (cd "$repo" && cargo build -p ordo-server)
[ -f "$dist/index.html" ] || (cd "$repo/platform/ordo-app" && trunk build --release)
mkdir -p "$data"
[ -f "$data/master.key" ] || "$bin" generate-key > "$data/master.key"
printf ordo-test-secret > "$data/oidc-secret"
: > "$data/seed.toml"

"$docker" run -d --name ordo-demo-dex -p 5556:5556 \
  -v "$here/dex.yaml:/etc/dex/config.yaml:ro,Z" ghcr.io/dexidp/dex:v2.46.1 dex serve /etc/dex/config.yaml >/dev/null
until curl -sf http://127.0.0.1:5556/dex/.well-known/openid-configuration >/dev/null; do sleep 0.5; done

ORDO_ADMINS=dora ORDO_SERVER_NAME=localhost ORDO_LISTEN="127.0.0.1:$port" ORDO_DATA_DIR="$data/data" \
ORDO_MASTER_KEY_FILE="$data/master.key" ORDO_SEED_FILE="$data/seed.toml" ORDO_APP_DIR="$dist" \
ORDO_OIDC_ISSUER=http://127.0.0.1:5556/dex ORDO_OIDC_CLIENT_ID=ordo \
ORDO_OIDC_CLIENT_SECRET_FILE="$data/oidc-secret" ORDO_PUBLIC_URL="http://127.0.0.1:$port" \
  nohup "$bin" serve >> "$data/server.log" 2>&1 &
echo $! > "$data/ordo.pid"
until curl -sf "http://127.0.0.1:$port/readyz" >/dev/null; do sleep 0.3; done

cat <<INFO
ORDO demo running: http://127.0.0.1:$port/
Accounts (password "password"): dora@example.org (administrator: plans incidents, gives
roles), anna@example.org, ben@example.org, eva@example.org (roles come from the incident).
Use a separate browser profile or private window per person. Log: $data/server.log
Stop: $0 stop    Delete data: $0 reset
INFO
