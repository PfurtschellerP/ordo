#!/usr/bin/env bash
# Local ORDO demo: Dex with demo accounts (Docker or Podman) and `ordo serve`. Not for real
# use: fixed passwords, demo certificates.
#
#   deploy/demo/demo.sh          start on this computer: http://127.0.0.1:18090
#   deploy/demo/demo.sh lan      start for devices in the local network: https://<ip>:18443
#                                (ORDO_DEMO_HOST=<ip> to choose the address)
#   deploy/demo/demo.sh stop     stop
#   deploy/demo/demo.sh reset    stop and delete all demo data
#
# Why HTTPS in the network: the app needs a secure context (WebCrypto for device keys and
# storage encryption, service worker, offline storage). Browsers grant it to
# http://localhost only, so other devices reach ORDO through Caddy with a local
# certificate authority, whose root certificate they install once (README.md).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
data="${ORDO_DEMO_DATA:-$here/.data}"
bin="${ORDO_BIN:-${CARGO_TARGET_DIR:-$repo/target}/debug/ordo}"
dist="${ORDO_APP_DIST:-$repo/platform/ordo-app/dist}"
docker="$(command -v docker || command -v podman)"
port=18090
tls_port=18443
ca_port=18080
# The Caddy of the Compose bundle (pinned there by digest, kept current by Dependabot).
caddy_image="$(sed -n 's/^ *image: \(docker.io\/library\/caddy:.*\)$/\1/p' "$repo/deploy/compose/minimal/compose.yaml")"

stop() {
  "$docker" rm -f ordo-demo-dex ordo-demo-caddy >/dev/null 2>&1 || true
  if [ -f "$data/ordo.pid" ]; then kill "$(cat "$data/ordo.pid")" 2>/dev/null || true; rm -f "$data/ordo.pid"; fi
}

mode=local
case "${1:-start}" in
  stop) stop; echo "Demo stopped."; exit 0 ;;
  reset) stop; rm -rf "$data"; echo "Demo data deleted."; exit 0 ;;
  start) ;;
  lan) mode=lan ;;
  *) echo "usage: $0 [start|lan|stop|reset]" >&2; exit 2 ;;
esac

if [ "$mode" = lan ]; then
  host="${ORDO_DEMO_HOST:-$(ip route get 1.1.1.1 2>/dev/null | awk '{for (i = 1; i <= NF; i++) if ($i == "src") print $(i + 1)}')}"
  [ -n "$host" ] || { echo "No network address found; set ORDO_DEMO_HOST." >&2; exit 1; }
  listen="0.0.0.0:$port"
  public="https://$host:$tls_port"
else
  host=127.0.0.1
  listen="127.0.0.1:$port"
  public="http://127.0.0.1:$port"
fi
issuer="http://$host:5556/dex"

stop
[ -x "$bin" ] || (cd "$repo" && cargo build -p ordo-server)
[ -f "$dist/index.html" ] || (cd "$repo/platform/ordo-app" && trunk build --release)
mkdir -p "$data"
[ -f "$data/master.key" ] || "$bin" generate-key > "$data/master.key"
printf ordo-test-secret > "$data/oidc-secret"
: > "$data/seed.toml"

# Dex announces the address the devices use, and sends them back to ORDO's.
sed -e "s#http://127.0.0.1:5556/dex#$issuer#" \
    -e "s#http://127.0.0.1:18090/auth/callback#$public/auth/callback#" \
    "$here/dex.yaml" > "$data/dex.yaml"
# Dex's API for local accounts (ADR-0060): certificates once, readable by Dex's user.
[ -f "$data/dex-api/ca.pem" ] || "$bin" generate-dex-certs "$data/dex-api" 127.0.0.1 localhost >/dev/null
chmod 755 "$data/dex-api" && chmod 644 "$data/dex-api"/*.pem
"$docker" run -d --name ordo-demo-dex -p 5556:5556 -p 127.0.0.1:5557:5557 \
  -v "$data/dex.yaml:/etc/dex/config.yaml:ro,Z" -v "$data/dex-api:/etc/dex/api:ro,Z" \
  ghcr.io/dexidp/dex:v2.46.1 dex serve /etc/dex/config.yaml >/dev/null
until curl -sf "$issuer/.well-known/openid-configuration" >/dev/null; do sleep 0.5; done

ORDO_ADMINS=dora ORDO_SERVER_NAME=localhost ORDO_LISTEN="$listen" ORDO_DATA_DIR="$data/data" \
ORDO_MASTER_KEY_FILE="$data/master.key" ORDO_SEED_FILE="$data/seed.toml" ORDO_APP_DIR="$dist" \
ORDO_OIDC_ISSUER="$issuer" ORDO_OIDC_CLIENT_ID=ordo \
ORDO_OIDC_CLIENT_SECRET_FILE="$data/oidc-secret" ORDO_PUBLIC_URL="$public" \
ORDO_DEX_API=https://127.0.0.1:5557 ORDO_DEX_API_CA_FILE="$data/dex-api/ca.pem" \
ORDO_DEX_API_CERT_FILE="$data/dex-api/ordo.pem" ORDO_DEX_API_KEY_FILE="$data/dex-api/ordo-key.pem" \
  nohup "$bin" serve >> "$data/server.log" 2>&1 &
echo $! > "$data/ordo.pid"
until curl -sf "http://127.0.0.1:$port/readyz" >/dev/null; do sleep 0.3; done

if [ "$mode" = lan ]; then
  # HTTPS for the network with Caddy's local certificate authority; its root certificate
  # (public, no key) is offered on a plain HTTP port for installing on the devices.
  mkdir -p "$data/caddy" "$data/caddy-config" "$data/ca"
  cat > "$data/Caddyfile" <<CADDY
{
	auto_https disable_redirects
	skip_install_trust
}
https://$host:$tls_port {
	tls internal
	reverse_proxy 127.0.0.1:$port
}
http://$host:$ca_port {
	root * /ca
	file_server
}
CADDY
  # As this user, so the certificate authority's files stay readable here.
  "$docker" run -d --name ordo-demo-caddy --network host --user "$(id -u):$(id -g)" \
    -v "$data/Caddyfile:/etc/caddy/Caddyfile:ro,Z" -v "$data/caddy:/data:Z" \
    -v "$data/caddy-config:/config:Z" -v "$data/ca:/ca:ro,Z" \
    "$caddy_image" >/dev/null
  root="$data/caddy/caddy/pki/authorities/local/root.crt"
  for _ in $(seq 60); do
    [ -f "$root" ] && curl -sf --cacert "$root" "$public/readyz" >/dev/null && break
    sleep 0.5
  done
  cp "$root" "$data/ca/ordo-demo-root.crt"
  cat <<INFO
ORDO demo running for the local network: $public/
On each device, first install and trust the demo's root certificate (once):
  http://$host:$ca_port/ordo-demo-root.crt   (steps: deploy/demo/README.md)
Firewall: allow TCP $tls_port, $ca_port and 5556 from the network, e.g.
  sudo firewall-cmd --add-port={$tls_port,$ca_port,5556}/tcp
INFO
else
  echo "ORDO demo running: $public/"
fi
cat <<INFO
Accounts (password "password"): dora@example.org (administrator: plans incidents, gives
roles), anna@example.org, ben@example.org, eva@example.org (roles come from the incident).
Use a separate browser profile or private window per person. Log: $data/server.log
Stop: $0 stop    Delete data: $0 reset
INFO
