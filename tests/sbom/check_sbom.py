#!/usr/bin/env python3
"""Checks a CycloneDX software bill of materials of an ORDO component (ADR-0043): the
expected format and version, the component itself, and dependencies with versions and
package URLs, among them the crates ORDO's security rests on.

    check_sbom.py <file.cdx.json> <component> [required crate ...]
"""

# [itest->dsn~sbom~1]

import json
import sys


def problems(bom: dict, component: str, required: list[str]) -> list[str]:
    """What is wrong with the bill of materials; empty if nothing."""
    found = []
    if bom.get("bomFormat") != "CycloneDX":
        found.append("not CycloneDX")
    if bom.get("specVersion") != "1.6":
        found.append(f"spec version {bom.get('specVersion')}, expected 1.6")
    if bom.get("metadata", {}).get("component", {}).get("name") != component:
        found.append(f"the bill is not about {component}")
    components = bom.get("components", [])
    if not components:
        found.append("no dependencies listed")
    incomplete = [c.get("name", "?") for c in components if not c.get("version") or not c.get("purl")]
    if incomplete:
        found.append(f"without version or package URL: {', '.join(incomplete[:5])}")
    names = {c.get("name") for c in components}
    found += [f"{crate} missing" for crate in required if crate not in names]
    return found


def main() -> int:
    if len(sys.argv) < 3:
        print(__doc__, file=sys.stderr)
        return 2
    path, component, required = sys.argv[1], sys.argv[2], sys.argv[3:]
    with open(path, encoding="utf-8") as f:
        bom = json.load(f)
    found = problems(bom, component, required)
    if found:
        print(f"{path}: " + "; ".join(found), file=sys.stderr)
        return 1
    print(f"{component}: CycloneDX {bom['specVersion']}, {len(bom['components'])} components")
    return 0


if __name__ == "__main__":
    sys.exit(main())
