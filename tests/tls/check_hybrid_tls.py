#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Checks that a TLS endpoint negotiates the hybrid post-quantum key exchange
X25519MLKEM768 (ADR-0042). Needs OpenSSL 3.5 or newer.

    check_hybrid_tls.py host:port [server-name]

Exit code 0 if the endpoint agreed on X25519MLKEM768 when the client offered only that
group, 1 otherwise. The certificate is not checked here (a local CA is fine).
"""

# [itest->dsn~pq-transport~1]

import re
import subprocess
import sys

GROUP = "X25519MLKEM768"


def negotiated_group(target: str, name: str) -> str | None:
    """The key exchange group the endpoint agreed on, offering only GROUP."""
    result = subprocess.run(
        ["openssl", "s_client", "-connect", target, "-servername", name, "-groups", GROUP],
        input="",
        capture_output=True,
        text=True,
        timeout=30,
        check=False,
    )
    found = re.search(r"Negotiated TLS1\.3 group: (\S+)", result.stdout + result.stderr)
    return found.group(1) if found else None


def main() -> int:
    if len(sys.argv) not in (2, 3):
        print(__doc__, file=sys.stderr)
        return 2
    target = sys.argv[1]
    name = sys.argv[2] if len(sys.argv) == 3 else target.rsplit(":", 1)[0]
    version = subprocess.run(["openssl", "version"], capture_output=True, text=True, check=False).stdout.strip()
    group = negotiated_group(target, name)
    if group == GROUP:
        print(f"{target}: hybrid post-quantum key exchange {GROUP} ({version})")
        return 0
    print(f"{target}: no {GROUP} (negotiated: {group or 'none'}; {version})", file=sys.stderr)
    return 1


if __name__ == "__main__":
    sys.exit(main())
