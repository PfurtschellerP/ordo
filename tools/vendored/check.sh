#!/bin/sh
# Compares the JavaScript libraries and typefaces the web app vendors (vendor-map.sh,
# vendor-fonts.sh) with the newest versions on npm, which Dependabot cannot see because
# the versions live in shell scripts. Prints a Markdown table; exit code 2 if any is
# behind, so the weekly workflow opens an issue. See docs/dependencies.md.
set -eu
app="$(dirname "$0")/../../platform/ordo-app"
version() { sed -n "s/^$1=//p" "$2"; }
behind=0
echo "| Package | Vendored | Newest on npm |"
echo "|---|---|---|"
while read -r package variable script; do
  ours=$(version "$variable" "$app/$script")
  newest=$(npm view "$package" version)
  mark=""
  if [ "$ours" != "$newest" ]; then mark=" (behind)"; behind=1; fi
  echo "| $package | $ours | $newest$mark |"
done <<LIST
maplibre-gl MAPLIBRE vendor-map.sh
pmtiles PMTILES vendor-map.sh
@protomaps/basemaps BASEMAPS vendor-map.sh
terra-draw TERRA_DRAW vendor-map.sh
terra-draw-maplibre-gl-adapter TERRA_DRAW_MAPLIBRE vendor-map.sh
@fontsource/atkinson-hyperlegible-next VERSION vendor-fonts.sh
@fontsource/atkinson-hyperlegible-mono VERSION vendor-fonts.sh
LIST
[ "$behind" -eq 0 ] || exit 2
