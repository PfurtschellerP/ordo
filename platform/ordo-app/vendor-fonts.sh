#!/bin/sh
# Fetches the app's typefaces into fonts/ (design tokens: Atkinson Hyperlegible Next and
# Mono, SIL OFL 1.1) from their Fontsource npm packages, checked against the integrity npm
# records: regular and bold, Latin and Latin Extended, WOFF2 only, plus fonts.css and the
# licences. Run it after changing a version below, then review the diff.
set -eu
cd "$(dirname "$0")"
VERSION=5.3.0

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
(cd "$work" && npm pack --silent "@fontsource/atkinson-hyperlegible-next@$VERSION" \
  "@fontsource/atkinson-hyperlegible-mono@$VERSION" >/dev/null)
rm -rf fonts && mkdir fonts
{
  echo "/* Typefaces of the design tokens (vendor-fonts.sh, Fontsource $VERSION, SIL OFL 1.1). */"
  for family in next mono; do
    pkg="$work/fontsource-atkinson-hyperlegible-$family-$VERSION/package"
    [ -d "$pkg" ] || { mkdir -p "$pkg"; tar -xzf "$work/fontsource-atkinson-hyperlegible-$family-$VERSION.tgz" -C "$pkg" --strip-components=1; }
    cp "$pkg/LICENSE" "fonts/OFL-atkinson-hyperlegible-$family.txt"
    for weight in 400 700; do
      for subset in latin-ext latin; do
        file="atkinson-hyperlegible-$family-$subset-$weight-normal.woff2"
        cp "$pkg/files/$file" fonts/
        # The package's rule for this file, with the WOFF2 source only, served from /fonts/.
        awk -v f="$file" 'BEGIN { RS = "}" } index($0, f) { sub(/^[^@]*/, ""); print $0 "}" }' "$pkg/$weight.css" \
          | sed -e "s#src: url(./files/$file) format('woff2'), url([^)]*) format('woff');#src: url(/fonts/$file) format('woff2');#"
      done
    done
  done
} > fonts/fonts.css
echo "fonts: Atkinson Hyperlegible Next and Mono $VERSION"
