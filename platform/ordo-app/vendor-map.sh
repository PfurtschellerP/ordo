#!/bin/sh
# Fetches the map libraries and label fonts into map/vendor and map/glyphs (ADR-0006).
# Libraries come from their npm packages, checked against the integrity npm records;
# fonts from Protomaps' basemap assets at a fixed commit. Run it after changing a version
# below, then review the diff.
set -eu
cd "$(dirname "$0")/map"
MAPLIBRE=6.17.0
PMTILES=4.5.0
BASEMAPS=5.7.2
ASSETS=028c18f713baecad011301ff7a69acc39bcc2ae7

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
(cd "$work" && npm pack --silent "maplibre-gl@$MAPLIBRE" "pmtiles@$PMTILES" "@protomaps/basemaps@$BASEMAPS" >/dev/null)
for t in "$work"/*.tgz; do
  name=$(basename "$t" .tgz)
  mkdir -p "$work/$name" && tar -xzf "$t" -C "$work/$name"
done
mkdir -p vendor
cp "$work/maplibre-gl-$MAPLIBRE/package/dist/maplibre-gl.mjs" \
   "$work/maplibre-gl-$MAPLIBRE/package/dist/maplibre-gl-worker.mjs" \
   "$work/maplibre-gl-$MAPLIBRE/package/dist/maplibre-gl.css" vendor/
cp "$work/pmtiles-$PMTILES/package/dist/pmtiles.js" vendor/
cp "$work/protomaps-basemaps-$BASEMAPS/package/dist/basemaps.js" vendor/

for font in "Noto Sans Regular" "Noto Sans Medium" "Noto Sans Italic"; do
  mkdir -p "glyphs/$font"
  for range in 0-255 256-511 8192-8447; do
    curl -sfL "https://raw.githubusercontent.com/protomaps/basemaps-assets/$ASSETS/fonts/$(printf %s "$font" | sed 's/ /%20/g')/$range.pbf" \
      -o "glyphs/$font/$range.pbf"
  done
done
echo "map libraries: maplibre-gl $MAPLIBRE, pmtiles $PMTILES, @protomaps/basemaps $BASEMAPS; fonts at $ASSETS"
