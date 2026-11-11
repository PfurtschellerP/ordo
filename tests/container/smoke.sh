#!/bin/sh
# Smoke test of a server image (OPS-5): it starts read-only as a non-root user, answers
# health, readiness and metrics, serves the web app, and Docker reports it healthy. Run on every architecture
# the image is built for (amd64, arm64).
#
#   tests/container/smoke.sh IMAGE
#
# [itest->dsn~container-image~1]
set -eu
image="$1"
name="ordo-smoke-$$"
work=$(mktemp -d)
trap 'docker logs "$name" 2>&1 | tail -20 || true; docker rm -f "$name" >/dev/null 2>&1 || true; rm -rf "$work"' EXIT

docker run --rm --entrypoint /usr/local/bin/ordo "$image" generate-key > "$work/master.key"
chmod 644 "$work/master.key"
docker run -d --name "$name" --read-only -p 8080:8080 \
  -e ORDO_SERVER_NAME=localhost \
  -e ORDO_MASTER_KEY_FILE=/run/master.key \
  -v "$work/master.key:/run/master.key:ro" \
  "$image" >/dev/null

for _ in $(seq 30); do curl -fsS http://localhost:8080/readyz >/dev/null && break; sleep 1; done
curl -fsS http://localhost:8080/readyz
curl -fsS http://localhost:8080/healthz
curl -fsS http://localhost:8080/metrics | grep -q '^ordo_sessions 0$'
# The web app is in the image and served at / (with its WebAssembly).
curl -fsS http://localhost:8080/ | grep -q '<title>ORDO</title>'
wasm=$(curl -fsS http://localhost:8080/ | grep -o '/ordo-app-[0-9a-f]*_bg\.wasm' | head -1)
test -n "$wasm"
curl -fsS -o /dev/null "http://localhost:8080$wasm"
test "$(docker inspect --format '{{.Config.User}}' "$image")" = "65532:65532"
for _ in $(seq 30); do
  [ "$(docker inspect --format '{{.State.Health.Status}}' "$name")" = healthy ] && break
  sleep 2
done
test "$(docker inspect --format '{{.State.Health.Status}}' "$name")" = healthy
arch=$(docker image inspect --format '{{.Architecture}}' "$image")
size=$(docker image inspect --format '{{.Size}}' "$image" | numfmt --to=iec)
echo "Image $image: linux/$arch, $size, healthy"
