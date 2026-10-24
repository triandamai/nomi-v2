#!/usr/bin/env python3
"""Turns Gradle's JUnit XML results into a Markdown test report for the job summary.

The native counterpart to cargo-test-summary.py: totals per module, every failure with its message and the top of
its stack trace, and the slowest test classes. Reads every TEST-*.xml under the given directory.

Usage: junit-summary.py <root> <title>   (prints Markdown; exits 0 even when tests failed — the Gradle step fails
the job, this only reports)
"""

import sys
import xml.etree.ElementTree as ET
from collections import defaultdict
from pathlib import Path

SLOWEST_SHOWN = 10
FAILURE_LINES = 15
MESSAGE_CHARS = 300


def module_of(path: Path, root: Path) -> str:
    """`shared/feature/chat` from `<root>/shared/feature/chat/build/test-results/desktopTest/TEST-x.xml`."""
    parts = path.relative_to(root).parts
    return "/".join(parts[: parts.index("build")]) if "build" in parts else str(path.parent)


def target_of(path: Path) -> str:
    """The test task the file came from (desktopTest, iosSimulatorArm64Test, testDebugUnitTest…)."""
    return path.parent.name


def main() -> None:
    root = Path(sys.argv[1]).resolve()
    title = sys.argv[2] if len(sys.argv) > 2 else "Native tests"
    files = sorted(root.glob("**/build/test-results/**/TEST-*.xml"))

    totals = defaultdict(lambda: {"tests": 0, "failed": 0, "skipped": 0, "time": 0.0})
    failures = []
    suites = []
    for file in files:
        try:
            tree = ET.parse(file)
        except ET.ParseError:
            continue
        for suite in tree.getroot().iter("testsuite"):
            key = (module_of(file, root), target_of(file))
            row = totals[key]
            row["tests"] += int(suite.get("tests", 0))
            row["failed"] += int(suite.get("failures", 0)) + int(suite.get("errors", 0))
            row["skipped"] += int(suite.get("skipped", 0))
            row["time"] += float(suite.get("time", 0) or 0)
            suites.append((float(suite.get("time", 0) or 0), suite.get("name", "?"), key[0]))
            for case in suite.iter("testcase"):
                problem = case.find("failure")
                if problem is None:
                    problem = case.find("error")
                if problem is not None:
                    failures.append((key, case.get("classname", ""), case.get("name", ""), problem))

    tests = sum(r["tests"] for r in totals.values())
    failed = sum(r["failed"] for r in totals.values())
    skipped = sum(r["skipped"] for r in totals.values())
    icon = "❌" if failed else "✅"
    out = [f"## {icon} {title}", ""]
    if not files:
        out += ["No test results were found (the build may have failed before any test ran).", ""]
        print("\n".join(out))
        return
    out += [f"**{tests - failed - skipped} passed**, **{failed} failed**, {skipped} skipped — {tests} tests in {len(totals)} test tasks.", ""]

    out += ["| Module | Task | Tests | Failed | Skipped | Time |", "|---|---|---:|---:|---:|---:|"]
    for (module, target), row in sorted(totals.items()):
        mark = " ❌" if row["failed"] else ""
        out.append(f"| `{module}` | {target} | {row['tests']} | {row['failed']}{mark} | {row['skipped']} | {row['time']:.1f}s |")
    out.append("")

    if failures:
        out += ["### Failures", ""]
        for (module, target), cls, name, problem in failures:
            message = (problem.get("message") or "").strip().replace("\n", " ")[:MESSAGE_CHARS]
            trace = "\n".join((problem.text or "").strip().splitlines()[:FAILURE_LINES])
            out += [
                f"<details><summary><code>{cls.split('.')[-1]}.{name}</code> — {module} ({target})</summary>",
                "",
                f"**{message}**" if message else "",
                "",
                "```",
                trace,
                "```",
                "</details>",
                "",
            ]

    slow = sorted(suites, reverse=True)[:SLOWEST_SHOWN]
    if slow:
        out += ["<details><summary>Slowest test classes</summary>", "", "| Class | Module | Time |", "|---|---|---:|"]
        out += [f"| `{name.split('.')[-1]}` | `{module}` | {time:.1f}s |" for time, name, module in slow]
        out += ["", "</details>", ""]

    print("\n".join(out))


if __name__ == "__main__":
    main()
