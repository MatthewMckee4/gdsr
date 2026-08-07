#!/usr/bin/env python3
"""KLayout GDS write/read benchmark. Invoked per-operation by hyperfine.

Usage:
    python benchmark_klayout.py write
    python benchmark_klayout.py read

Requires: pip install klayout
"""

import math
import os
import sys
import tempfile

import klayout.db as pya

DB_UNITS = 1e-9
TMP_DIR = tempfile.gettempdir()


def build_library():
    layout = pya.Layout()
    layout.dbu = DB_UNITS / 1e-6

    leaf_cells = []
    for c in range(20):
        cell = layout.create_cell(f"leaf_{c}")
        leaf_cells.append(cell)

        for i in range(500):
            offset = i * 50
            layer = layout.layer((c * 3 + i) % 64, i % 8)
            npoints = 4 + (i % 5)
            points = []
            for p in range(npoints):
                angle = 2 * math.pi * p / npoints
                r = 20 + (i % 30)
                points.append(
                    pya.Point(
                        offset + int(r * math.cos(angle)),
                        int(r * math.sin(angle)),
                    )
                )
            cell.shapes(layer).insert(pya.Polygon(points))

        for i in range(250):
            y_base = c * 5000 + i * 80
            nsegs = 3 + (i % 8)
            points = [
                pya.Point(s * 150, y_base + (0 if s % 2 == 0 else 60))
                for s in range(nsegs)
            ]
            layer = layout.layer((c * 2 + i) % 64, i % 4)
            cell.shapes(layer).insert(pya.Path(points, (i % 30 + 1) * 10 * 2))

        for i in range(100):
            layer = layout.layer((c + i) % 64, i % 4)
            text = pya.Text(
                f"L{c}_text_{i:03}",
                pya.Trans(pya.Point(i * 400, c * 1000)),
            )
            cell.shapes(layer).insert(text)

    mid_cells = []
    for c in range(20):
        cell = layout.create_cell(f"mid_{c}")
        mid_cells.append(cell)
        for r in range(3):
            leaf = (c * 3 + r) % 20
            mag = 1.0 + (c % 5) * 0.1
            angle_deg = (c % 8) * 0.05 * 180 / math.pi
            mirror = c % 3 == 0
            trans = pya.CplxTrans(
                mag, angle_deg, mirror, pya.DPoint(r * 80_000, c * 40_000)
            )
            inst = pya.CellInstArray(
                leaf_cells[leaf].cell_index(),
                trans,
                pya.Vector(15_000, 0),
                pya.Vector(0, 12_000),
                10,
                8,
            )
            cell.insert(inst)

        for i in range(50):
            layer = layout.layer(i % 16, 0)
            points = [
                pya.Point(i * 2000, 0),
                pya.Point(i * 2000 + 1000, 0),
                pya.Point(i * 2000 + 1000, 1000),
                pya.Point(i * 2000, 1000),
            ]
            cell.shapes(layer).insert(pya.Polygon(points))

    for c in range(10):
        cell = layout.create_cell(f"top_{c}")

        for i in range(100):
            layer = layout.layer(i % 16, 0)
            points = [
                pya.Point(i * 1000, 0),
                pya.Point(i * 1000 + 500, 0),
                pya.Point(i * 1000 + 500, 500),
                pya.Point(i * 1000, 500),
            ]
            cell.shapes(layer).insert(pya.Polygon(points))

        for r in range(4):
            mid = (c * 4 + r) % 20
            mag = 0.9 + r * 0.1
            angle_deg = r * 0.03 * 180 / math.pi
            mirror = r % 2 == 1
            trans = pya.CplxTrans(mag, angle_deg, mirror, pya.DPoint(r * 300_000, 0))
            inst = pya.CellInstArray(
                mid_cells[mid].cell_index(),
                trans,
                pya.Vector(150_000, 0),
                pya.Vector(0, 120_000),
                5,
                4,
            )
            cell.insert(inst)

    return layout


COMMANDS = {
    "write": lambda lib, path: lib.write(path),
    "read": lambda lib, path: pya.Layout().read(path),
}

if __name__ == "__main__":
    if len(sys.argv) != 2 or sys.argv[1] not in COMMANDS:
        print(f"Usage: {sys.argv[0]} <{'|'.join(COMMANDS)}>", file=sys.stderr)
        sys.exit(1)

    cmd = sys.argv[1]
    path = os.path.join(TMP_DIR, "klayout_bench.gds")

    layout = build_library()

    if cmd == "read" and not os.path.exists(path):
        layout.write(path)

    COMMANDS[cmd](layout, path)
