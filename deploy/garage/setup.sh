#!/bin/sh
# Prepares a fresh single-node Garage for ORDO: layout, bucket and access key.
# Usage: setup.sh <garage command> <bucket> <key id> <secret key>
#   e.g. setup.sh "docker exec ordo-garage /garage" ordo GK… <64 hex>
# The key ID is "GK" and 24 hex characters, the secret 64 hex characters
# (`openssl rand -hex 12`, `openssl rand -hex 32`). Running it again changes nothing.
set -eu
garage=$1 bucket=$2 key_id=$3 secret=$4

i=0
until $garage status >/dev/null 2>&1; do
  i=$((i + 1)); [ "$i" -lt 30 ] || { echo "Garage does not answer" >&2; exit 1; }
  sleep 1
done
node=$($garage node id -q 2>/dev/null | cut -d@ -f1)
if ! $garage layout show 2>/dev/null | grep -q "^Current cluster layout version: [1-9]"; then
  $garage layout assign -z dc1 -c 1G "$node"
  $garage layout apply --version 1
fi
$garage bucket info "$bucket" >/dev/null 2>&1 || $garage bucket create "$bucket"
$garage key info "$key_id" >/dev/null 2>&1 || $garage key import --yes -n ordo "$key_id" "$secret"
$garage bucket allow --read --write "$bucket" --key "$key_id"
