#!/usr/bin/env python3
"""Turns `cargo llvm-cov report --json --summary-only` into Markdown for the GitHub job summary:
the workspace's totals, each crate's, then the least-covered files. The frontends' coverage
summaries (scripts/coverage-summary.mjs) look the same.

Usage: llvm-cov-summary.py coverage.json [workspace-root]
"""

import json
import os
import sys
from collections import defaultdict

METRICS = ["lines", "functions", "regions"]
LEAST_COVERED_COUNT = 15
GOOD_PCT = 90
FAIR_PCT = 75


def badge(pct):
    return "🟢" if pct >= GOOD_PCT else "🟡" if pct >= FAIR_PCT else "🔴"


def cell(metric):
    covered, count = metric["covered"], metric["count"]
    pct = 100 * covered / count if count else 0
    return f"{badge(pct)} {pct:.2f}% ({covered}/{count})"


def crate_of(path):
    """`nomi-server` for `crates/nomi-server/src/...`; anything else is its first folder."""
    parts = path.split("/")
    return parts[1] if parts[0] == "crates" and len(parts) > 1 else parts[0]


def main():
    report = json.load(open(sys.argv[1]))["data"][0]
    root = os.path.abspath(sys.argv[2] if len(sys.argv) > 2 else ".")
    files = [{**f, "path": os.path.relpath(f["filename"], root)} for f in report["files"]]

    out = [
        "## Backend: test coverage",
        "",
        "`cargo llvm-cov` over the workspace's tests (unit and integration).",
        "",
        "| Metric | Coverage |",
        "| --- | --- |",
        *[f"| {m.capitalize()} | {cell(report['totals'][m])} |" for m in METRICS],
        "",
    ]

    crates = defaultdict(lambda: {m: {"covered": 0, "count": 0} for m in METRICS})
    for f in files:
        for m in METRICS:
            crates[crate_of(f["path"])][m]["covered"] += f["summary"][m]["covered"]
            crates[crate_of(f["path"])][m]["count"] += f["summary"][m]["count"]
    out += [
        f"<details><summary>{len(crates)} crates</summary>",
        "",
        "| Crate | Lines | Functions |",
        "| --- | --- | --- |",
        *[f"| `{name}` | {cell(c['lines'])} | {cell(c['functions'])} |" for name, c in sorted(crates.items())],
        "",
        "</details>",
        "",
    ]

    least = sorted(
        (f for f in files if f["summary"]["lines"]["count"] > 0),
        key=lambda f: (f["summary"]["lines"]["percent"], -f["summary"]["lines"]["count"]),
    )[:LEAST_COVERED_COUNT]
    if least:
        out += [
            f"<details><summary>{len(least)} least-covered files</summary>",
            "",
            "| File | Lines | Functions |",
            "| --- | --- | --- |",
            *[f"| `{f['path']}` | {cell(f['summary']['lines'])} | {cell(f['summary']['functions'])} |" for f in least],
            "",
            "</details>",
            "",
        ]

    print("\n".join(out))


if __name__ == "__main__":
    main()
