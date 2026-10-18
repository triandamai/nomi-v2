#!/usr/bin/env python3
"""Turns a saved `cargo test` log into a Markdown report for the GitHub Actions job summary.

`cargo test` has no stable machine-readable output, so this reads its plain output: each test
binary's "Running ..." header, its `test <name> ... ok|FAILED|ignored` lines, its "test result:"
line, the panic output under "failures:", and libtest's "has been running for over 60 seconds"
warnings. Usage: cargo-test-summary.py <log> [>> "$GITHUB_STEP_SUMMARY"]
"""

import re
import sys

ANSI = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
RUNNING = re.compile(r"^\s*Running (?:unittests )?(\S+) \((?:[^)]*/)?deps/([A-Za-z0-9_]+?)-[0-9a-f]+\)")
DOC_TESTS = re.compile(r"^\s*Doc-tests (\S+)")
RESULT = re.compile(
    r"^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; \d+ measured; \d+ filtered out; finished in ([\d.]+)s"
)
FAILED_TEST = re.compile(r"^test (\S+) \.\.\. FAILED")
SLOW = re.compile(r"^test (\S+) has been running for over (\d+) seconds")
FAILURE_OUTPUT = re.compile(r"^---- (\S+) stdout ----")


def parse(lines):
    binaries, slow, failure_output = [], [], {}
    current, capturing = None, None
    for raw in lines:
        line = ANSI.sub("", raw.rstrip("\n"))
        if m := RUNNING.match(line):
            current = {"name": m.group(2), "path": m.group(1), "failed_tests": []}
            capturing = None
        elif m := DOC_TESTS.match(line):
            current = {"name": f"{m.group(1)} (doc-tests)", "path": "", "failed_tests": []}
            capturing = None
        elif m := FAILURE_OUTPUT.match(line):
            capturing = m.group(1)
            failure_output[capturing] = []
        elif capturing and (line.startswith("failures:") or line.startswith("test result:")):
            capturing = None
        elif capturing is not None:
            failure_output[capturing].append(line)
            continue
        if m := FAILED_TEST.match(line):
            if current is not None:
                current["failed_tests"].append(m.group(1))
        if m := SLOW.match(line):
            slow.append((m.group(1), current["name"] if current else "?", int(m.group(2))))
        if (m := RESULT.match(line)) and current is not None:
            current.update(
                ok=m.group(1) == "ok",
                passed=int(m.group(2)),
                failed=int(m.group(3)),
                ignored=int(m.group(4)),
                seconds=float(m.group(5)),
            )
            binaries.append(current)
            current = None
    return binaries, slow, failure_output


def main():
    with open(sys.argv[1], encoding="utf-8", errors="replace") as log:
        binaries, slow, failure_output = parse(log)

    ran = [b for b in binaries if b["passed"] or b["failed"] or b["ignored"]]
    passed = sum(b["passed"] for b in binaries)
    failed = sum(b["failed"] for b in binaries)
    ignored = sum(b["ignored"] for b in binaries)
    seconds = sum(b["seconds"] for b in binaries)
    bad_binaries = sum(1 for b in ran if not b["ok"])

    out = ["## Cargo Test Report", ""]
    if not binaries:
        out += ["No test results found in the log: the build likely failed before any test ran.", ""]
        print("\n".join(out))
        return

    out += [
        "| | Passed | Failed | Ignored | Total | Time in tests |",
        "|---|---:|---:|---:|---:|---:|",
        f"| **Test binaries** | {len(ran) - bad_binaries} | {bad_binaries} | | {len(ran)} | |",
        f"| **Tests** | {passed} | {failed} | {ignored} | {passed + failed + ignored} | {seconds:.1f}s |",
        "",
    ]

    if failed:
        out += ["### Failed tests", ""]
        for b in binaries:
            for test in b["failed_tests"]:
                out += [f"<details><summary><code>{test}</code> in <code>{b['name']}</code></summary>", "", "```"]
                out += failure_output.get(test, ["(no output captured)"])[-60:]
                out += ["```", "", "</details>", ""]

    if slow:
        out += ["### Slow tests", "", "Ran for over 60 seconds (libtest's warning):", ""]
        out += [f"- `{test}` in `{binary}`" for test, binary, _ in slow]
        out += [""]

    slowest = sorted(ran, key=lambda b: b["seconds"], reverse=True)
    out += [
        "<details><summary>Every test binary, slowest first</summary>",
        "",
        "| Binary | Passed | Failed | Ignored | Time |",
        "|---|---:|---:|---:|---:|",
    ]
    for b in slowest:
        mark = "" if b["ok"] else " (failed)"
        out.append(f"| `{b['name']}`{mark} | {b['passed']} | {b['failed']} | {b['ignored']} | {b['seconds']:.2f}s |")
    out += ["", "</details>", ""]
    print("\n".join(out))


if __name__ == "__main__":
    main()
