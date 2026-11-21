"""Build and run the verification crate's required ignored witnesses."""

import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib


ROOT = Path(__file__).resolve().parents[2]
VERIFY = ROOT / "crates/verify"
MANIFEST = VERIFY / "required-witnesses.toml"
NAME = re.compile(r"[A-Za-z_]\w*(?:::[A-Za-z_]\w*)+")
SUMMARY = re.compile(
    r"^test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
    r"(\d+) measured; \d+ filtered out;.*$", re.MULTILINE
)
LIST_SUMMARY = re.compile(r"^(\d+) tests?, \d+ benchmarks?$", re.MULTILINE)


def libtest(executable: str, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run([executable, *args], cwd=VERIFY, encoding="utf-8", errors="replace",
                          capture_output=True)


def build() -> str | None:
    result = subprocess.run(
        ["cargo", "test", "--locked", "--profile", "verify", "-p", "verify",
         "--lib", "--no-run", "--message-format=json-render-diagnostics"],
        cwd=ROOT, encoding="utf-8", stdout=subprocess.PIPE,
    )
    if result.returncode:
        print(f"BUILD FAILURE (cargo exit {result.returncode})", file=sys.stderr)
        return None
    executables: list[str] = []
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if (message.get("reason") == "compiler-artifact"
                and message["target"]["name"] == "verify"
                and message["profile"]["test"]
                and message["executable"]):
            executables.append(message["executable"])
    if len(executables) != 1:
        raise ValueError("cargo did not report exactly one verify test executable")
    return executables[0]


def required() -> list[str]:
    groups = tomllib.loads(MANIFEST.read_text(encoding="utf-8"))
    if not groups or any(not isinstance(group, list) or not group for group in groups.values()):
        raise ValueError("required witness groups must be nonempty arrays")
    names = [name for group in groups.values() for name in group]
    if any(not isinstance(name, str) or not NAME.fullmatch(name) for name in names):
        raise ValueError("required witnesses must be exact libtest names")
    if len(names) != len(set(names)):
        raise ValueError("duplicate required witness")
    return names


def completed(executable: str, name: str) -> bool:
    result = libtest(executable, name, "--exact", "--ignored", "--color", "never")
    # Panic and process output can contain private fixture values. Report only the test ID
    # and classify completion from libtest's own result, never from its panic text.
    summary = SUMMARY.findall(result.stdout)
    for code, status, counts, label in [
        (0, "ok", ("1", "0", "0", "0"), "PASS"),
        (101, "FAILED", ("0", "1", "0", "0"), "TEST FAILURE"),
    ]:
        if (result.returncode == code and summary == [(status, *counts)]
                and f"test {name} ... {status}" in result.stdout.splitlines()):
            print(f"{label} {name}", flush=True)
            return code == 0
    raise ValueError(f"{name}: libtest did not complete the required test (exit {result.returncode})")


def main() -> int:
    try:
        names = required()
        executable = build()
        if executable is None:
            return 2
        listing = libtest(executable, "--list", "--ignored")
        if listing.returncode:
            raise ValueError(f"libtest discovery failed (exit {listing.returncode})")
        available = {line.removesuffix(": test") for line in listing.stdout.splitlines()
                     if line.endswith(": test")}
        if LIST_SUMMARY.findall(listing.stdout) != [str(len(available))]:
            raise ValueError("libtest did not report a complete ignored test list")
        missing = [name for name in names if name not in available]
        if missing:
            for name in missing:
                print(f"MISSING REQUIRED WITNESS {name}", file=sys.stderr)
            return 4
        results = [completed(executable, name) for name in names]
    except (OSError, ValueError, KeyError) as error:
        print(f"INFRASTRUCTURE FAILURE: {error}", file=sys.stderr)
        return 3
    print(f"Required witnesses passed: {sum(results)}/{len(names)}")
    return 0 if all(results) else 5


if __name__ == "__main__":
    sys.exit(main())
