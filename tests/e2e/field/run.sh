#!/bin/sh
# Field-near test (dsn~field-test~1): starts the Compose bundle with two tablets, runs
# field.test.ts against it and writes report.md to the work directory.
#
#   tests/e2e/field/run.sh                # image ordo:local (docker build -f deploy/Containerfile -t ordo:local .)
#   ORDO_FIELD_IMAGE=ordo:ci tests/e2e/field/run.sh
#
# FIELD_WORK (default: a new temporary directory) keeps certificates, configuration, the
# report and the containers' logs. KEEP=1 leaves the containers running afterwards.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
export FIELD_WORK="${FIELD_WORK:-$(mktemp -d "${TMPDIR:-/tmp}/ordo-field-XXXXXX")}"
export ORDO_FIELD_IMAGE="${ORDO_FIELD_IMAGE:-ordo:local}"
export ORDO_DOMAIN=ordo.internal ORDO_OIDC_CLIENT_SECRET=field-test-secret ORDO_ADMINS=dora
export FIELD_ANDROID_PORT="${FIELD_ANDROID_PORT:-3201}" FIELD_IPAD_PORT="${FIELD_IPAD_PORT:-3202}"
compose() {
  docker compose --project-directory "$repo/deploy/compose/minimal" \
    -f "$repo/deploy/compose/minimal/compose.yaml" -f "$here/compose.field.yaml" "$@"
}

# The organisation's own certificate authority and the site's certificate.
mkdir -p "$FIELD_WORK/tls"
cd "$FIELD_WORK/tls"
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -days 7 \
  -subj "/CN=ORDO field test CA" -keyout ca-key.pem -out ca.pem \
  -addext basicConstraints=critical,CA:TRUE -addext keyUsage=critical,keyCertSign 2>/dev/null
openssl req -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -subj "/CN=$ORDO_DOMAIN" \
  -keyout site-key.pem -out site.csr 2>/dev/null
printf 'subjectAltName=DNS:%s\nextendedKeyUsage=serverAuth\n' "$ORDO_DOMAIN" > site.ext
openssl x509 -req -in site.csr -CA ca.pem -CAkey ca-key.pem -CAcreateserial -days 7 \
  -extfile site.ext -out site.pem 2>/dev/null
rm -f ca-key.pem site.csr site.ext
chmod 644 ./*.pem
# Chromium trusts the site's key (a managed Android tablet would trust the CA).
openssl x509 -in site.pem -pubkey -noout | openssl pkey -pubin -outform der |
  openssl dgst -sha256 -binary | base64 > site-spki.txt

# The bundle's configuration, as the README makes it.
cd "$FIELD_WORK"
cp "$here/Caddyfile.field" Caddyfile
docker run --rm "$ORDO_FIELD_IMAGE" generate-key > master.key && chmod 644 master.key
printf %s "$ORDO_OIDC_CLIENT_SECRET" > oidc-client-secret && chmod 644 oidc-client-secret
mkdir -p dex-api && chmod 777 dex-api
docker run --rm -v "$FIELD_WORK/dex-api:/certs:Z" "$ORDO_FIELD_IMAGE" generate-dex-certs /certs dex >/dev/null
chmod 755 dex-api
# Dex: the bundle's template, with the field test's people (password "password").
{
  cat "$repo/deploy/compose/minimal/dex.yaml.example"
  printf '\nstaticPasswords:\n'
  n=1
  for p in "dora:Dora Klein" "anna:Anna Schmidt" "ben:Ben Weber"; do
    printf '  - email: "%s@example.org"\n    hash: "$2a$10$2b2cU8CPhOTaGrs1HRQuAueS7JTT5ZHsHSzYiFPm1leZck7Mc8T4W"\n' "${p%%:*}"
    printf '    username: "%s"\n    preferredUsername: "%s"\n    userID: "00000000-0000-4000-8000-00000000000%s"\n' "${p#*:}" "${p%%:*}" "$n"
    n=$((n + 1))
  done
} > dex.yaml

cleanup() {
  compose logs --no-color > "$FIELD_WORK/compose.log" 2>&1 || true
  [ "${KEEP:-0}" = 1 ] || compose down -v --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT
compose up -d --wait --no-build
echo "field test: bundle up, work directory $FIELD_WORK"

cd "$repo/tests/e2e"
npx playwright test -c playwright.field.config.ts
