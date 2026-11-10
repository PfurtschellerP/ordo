// Generates the tactical symbols of a profile (MAP-3) from its symbols.json with
// @taktische-zeichen/core (MIT, DV 102): one SVG per symbol, without an embedded font, so
// the app stays small. With --check, compares instead of writing (CI).
//
//   node generate.mjs ../../profiles/de/symbols          write <id>.svg
//   node generate.mjs ../../profiles/de/symbols --check  fail if a file differs
//
// [impl->dsn~tactical-symbols~1]

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { erzeugeTaktischesZeichen } from "@taktische-zeichen/core";

const { version } = JSON.parse(
  readFileSync(new URL("node_modules/@taktische-zeichen/core/package.json", import.meta.url), "utf8"),
);
// The library's licence travels with every symbol, also inside the app.
const NOTICE = `<!-- Drawn by @taktische-zeichen/core ${version} (https://github.com/phjardas/taktische-zeichen). Copyright 2022-2026 Philipp Jardas. SPDX-License-Identifier: MIT -->`;

const [dir, flag] = process.argv.slice(2);
if (!dir) {
  console.error("usage: node generate.mjs <profile symbols dir> [--check]");
  process.exit(2);
}
const check = flag === "--check";
const { symbols } = JSON.parse(readFileSync(join(dir, "symbols.json"), "utf8"));
let differ = 0;
for (const { id, spec } of symbols) {
  const drawn = erzeugeTaktischesZeichen({ ...spec, skipFontRegistration: true }).toString();
  const svg = drawn.replace("?>\n", `?>\n${NOTICE}\n`) + "\n";
  const file = join(dir, `${id}.svg`);
  if (check) {
    if (!existsSync(file) || readFileSync(file, "utf8") !== svg) {
      console.error(`${file}: not what tools/symbols generates; run npm run generate`);
      differ++;
    }
  } else {
    writeFileSync(file, svg);
  }
}
if (differ) process.exit(1);
console.log(`${symbols.length} symbols ${check ? "up to date" : "written"} in ${dir}`);
