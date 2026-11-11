#!/usr/bin/env python3
"""Writes minimal valid PMTiles v3 files for the browser tests: small enough to keep in the
repository, real enough for PMTiles and MapLibre to read. Spec:
https://github.com/protomaps/PMTiles (v3).

    make_region.py region.pmtiles          one empty vector tile at zoom 0 (MAP-2)
    make_region.py --world world.pmtiles   a "world map", zoom 0 to 3, every tile an empty
                                           gzip-compressed vector tile, to cut regions
                                           from (MAP-8)
"""

# [itest->dsn~situation-map~1]
# [itest->dsn~map-region~1]

import gzip
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
    world = sys.argv[1] == "--world"
    out = sys.argv[2] if world else sys.argv[1]
    if world:
        # An empty Mapbox Vector Tile, gzip-compressed (as Protomaps serves tiles), with a
        # fixed time so the file stays the same; z0..z3 are tile IDs 0..84, one run.
        tile = gzip.compress(b"", mtime=0)
        tiles, max_zoom, compression = 85, 3, 2
    else:
        tile = b""  # an empty Mapbox Vector Tile: no layers
        tiles, max_zoom, compression = 1, 0, 1
    # Root directory with one entry: from tile 0, run length `tiles`, at offset 0.
    directory = varint(1) + varint(0) + varint(tiles) + varint(len(tile)) + varint(0 + 1)
    name = "ORDO test world" if world else "ORDO test region"
    metadata = json.dumps({"name": name, "vector_layers": []}).encode()
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
        tiles, 1, 1,               # addressed tiles, tile entries, tile contents
    ) + bytes([
        1,                         # clustered
        1,                         # internal compression: none
        compression,               # tile compression: none (1) or gzip (2)
        1,                         # tile type: MVT
        0, max_zoom,               # min and max zoom
    ]) + (
        struct.pack("<iiii", e7(-180), e7(-85), e7(180), e7(85)) if world
        else struct.pack("<iiii", e7(10.8), e7(48.3), e7(11.0), e7(48.45))
    ) + bytes([0]) + struct.pack(
        "<ii", e7(10.9), e7(48.37)
    )
    assert len(header) == header_len, len(header)
    with open(out, "wb") as f:
        f.write(header + directory + metadata + tile)


if __name__ == "__main__":
    main()
