#!/usr/bin/env python3
"""Writes a minimal valid PMTiles v3 file (one empty vector tile at zoom 0) for the browser
test of the situation map (MAP-2): small enough to keep in the repository, real enough
for PMTiles and MapLibre to read. Spec: https://github.com/protomaps/PMTiles (v3).

    make_region.py region.pmtiles
"""

# [itest->dsn~situation-map~1]

import json
import struct
import sys


def varint(n: int) -> bytes:
    out = bytearray()
    while True:
        b = n & 0x7F
        n >>= 7
        out.append(b | (0x80 if n else 0))
        if not n:
            return bytes(out)


def main() -> None:
    tile = b""  # an empty Mapbox Vector Tile: no layers
    # Root directory with one entry: tile 0 (z0/x0/y0), run length 1, at offset 0.
    directory = varint(1) + varint(0) + varint(1) + varint(len(tile)) + varint(0 + 1)
    metadata = json.dumps({"name": "ORDO test region", "vector_layers": []}).encode()
    header_len = 127
    root_offset = header_len
    metadata_offset = root_offset + len(directory)
    tiles_offset = metadata_offset + len(metadata)
    e7 = lambda deg: int(round(deg * 10_000_000))
    header = b"PMTiles" + bytes([3]) + struct.pack(
        "<QQQQQQQQQQQ",
        root_offset, len(directory),
        metadata_offset, len(metadata),
        0, 0,                      # no leaf directories
        tiles_offset, len(tile),
        1, 1, 1,                   # addressed tiles, tile entries, tile contents
    ) + bytes([
        1,                         # clustered
        1,                         # internal compression: none
        1,                         # tile compression: none
        1,                         # tile type: MVT
        0, 0,                      # min and max zoom
    ]) + struct.pack("<iiii", e7(10.8), e7(48.3), e7(11.0), e7(48.45)) + bytes([0]) + struct.pack(
        "<ii", e7(10.9), e7(48.37)
    )
    assert len(header) == header_len, len(header)
    with open(sys.argv[1], "wb") as f:
        f.write(header + directory + metadata + tile)


if __name__ == "__main__":
    main()
