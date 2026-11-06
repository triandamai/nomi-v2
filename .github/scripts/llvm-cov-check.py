#!/usr/bin/env python3
"""Fails when a crate's test coverage drops below its floor in backend/coverage-floors.json.

Each crate has a floor for lines and one for functions (the "lines" and "functions" of
`cargo llvm-cov report --json --summary-only`, summed over the crate's files). Floors are set at
the coverage the crate actually reached, so coverage can only go up: raise a crate's floor when
its tests improve. A crate without its own entry uses the "default" floor.

Usage: llvm-cov-check.py coverage.json coverage-floors.json [workspace-root]
"""

import json
import os
import sys
from collections import defaultdict

METRICS = ["lines", "functions"]


def crate_of(path):
    parts = path.split("/")
    return parts[1] if parts[0] == "crates" and len(parts) > 1 else parts[0]


def main():
    report = json.load(open(sys.argv[1]))["data"][0]
    floors = json.load(open(sys.argv[2]))
    root = os.path.abspath(sys.argv[3] if len(sys.argv) > 3 else ".")

    crates = defaultdict(lambda: {m: [0, 0] for m in METRICS})
    for f in report["files"]:
        crate = crate_of(os.path.relpath(f["filename"], root))
        for m in METRICS:
            crates[crate][m][0] += f["summary"][m]["covered"]
            crates[crate][m][1] += f["summary"][m]["count"]

    default = floors.get("default", {})
    failures = []
    for crate, metrics in sorted(crates.items()):
        floor = floors.get("crates", {}).get(crate, default)
        for m in METRICS:
            covered, count = metrics[m]
            if count == 0 or m not in floor:
                continue
            pct = 100 * covered / count
            if pct + 1e-9 < floor[m]:
                failures.append(f"{crate}: {m} {pct:.2f}% is below its floor of {floor[m]}%")

    if failures:
        print("Coverage dropped below its floor (backend/coverage-floors.json):")
        print("\n".join(f"  - {line}" for line in failures))
        sys.exit(1)
    print(f"Coverage is at or above every floor ({len(crates)} crates).")


if __name__ == "__main__":
    main()
