#!/usr/bin/env python3
"""Summarise a `cargo test` log per test binary and require every expected one.

Usage: test_summary.py LOG PACKAGE_DIR...

Each PACKAGE_DIR (e.g. crates/sm_game) must have run its library unit tests,
every tests/*.rs integration test and its doc-tests. A test file that is not
built, or a binary that stops before its result line, fails the step instead
of passing silently. The table goes to stdout and, in GitHub Actions, to the
job summary.
"""
import os
import pathlib
import re
import sys

RUNNING = re.compile(r"^\s*Running (unittests )?(\S+) \((?:.*[\\/])?deps[\\/]([A-Za-z0-9_]+)-[0-9a-f]+(?:\.exe)?\)")
DOCTESTS = re.compile(r"^\s*Doc-tests (\S+)")
RESULT = re.compile(r"^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out")


def binaries(log):
    """(name, result) in log order; result is None if the binary gave none."""
    current, found = None, []
    for line in log.splitlines():
        running = RUNNING.match(line)
        doctests = DOCTESTS.match(line)
        if running or doctests:
            if current is not None:
                found.append((current, None))
            if running:
                unit, source, crate = running.groups()
                current = f"{crate} (unit)" if unit else source.replace("\\", "/")
            else:
                current = f"{doctests.group(1)} (doc)"
            continue
        result = RESULT.match(line)
        if result and current is not None:
            found.append((current, result.groups()))
            current = None
    if current is not None:
        found.append((current, None))
    return found


def main():
    log_path, *packages = sys.argv[1:]
    log = pathlib.Path(log_path).read_text(encoding="utf-8", errors="replace")
    found = binaries(log)
    ran = {name for name, _ in found}
    expected = []
    for package in packages:
        root = pathlib.Path(package)
        crate = root.name
        expected.append(f"{crate} (unit)")
        expected.append(f"{crate} (doc)")
        # Integration tests are reported by their path inside the package.
        expected += [f"tests/{path.name}" for path in sorted((root / "tests").glob("*.rs"))]
    rows = ["| Test binary | Result | Passed | Failed | Ignored | Filtered |", "| --- | --- | --- | --- | --- | --- |"]
    failed = []
    for name, result in found:
        if result is None:
            rows.append(f"| `{name}` | **no result** | | | | |")
            failed.append(name)
        else:
            status, passed, fails, ignored, _, filtered = result
            rows.append(f"| `{name}` | {'ok' if status == 'ok' else '**' + status + '**'} | {passed} | {fails} | {ignored} | {filtered} |")
            if status != "ok":
                failed.append(name)
    missing = [name for name in expected if name not in ran]
    for name in missing:
        rows.append(f"| `{name}` | **not run** | | | | |")
    report = "\n".join(["### Target package tests", "", *rows, ""])
    print(report)
    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as handle:
            handle.write(report + "\n")
    if missing:
        print("expected test binaries that did not run: " + ", ".join(missing), file=sys.stderr)
    if failed:
        print("test binaries that failed or gave no result: " + ", ".join(failed), file=sys.stderr)
    return 1 if missing or failed else 0


if __name__ == "__main__":
    sys.exit(main())
