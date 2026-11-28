import argparse
import subprocess
import sys
from pathlib import Path

from ordered import FrozenList

from sm_tools.ci.core.required_witnesses import (
    BUILD,
    DESCRIPTION,
    NAME,
    NAME_ERROR,
    TEST_HELP,
    available,
    build_failure,
    discovery_failure,
    executable,
    inclusion,
    missing,
    passed,
    required,
    single_test,
    verdict_without_output,
)

ROOT = Path(__file__).resolve().parents[5]
VERIFY = ROOT / "crates/verify"
MANIFEST = VERIFY / "required-witnesses.toml"


class Arguments(argparse.Namespace):
    test: str | None = None


def libtest(binary: str, *args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        (binary, *args),
        cwd=VERIFY,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )


def run(target: str) -> int:
    selected = bool(target)
    names = (
        FrozenList((target,))
        if selected
        else required(MANIFEST.read_text(encoding="utf-8"))
    )
    included = inclusion(selected=selected)
    built = subprocess.run(
        BUILD, cwd=ROOT, encoding="utf-8", stdout=subprocess.PIPE, check=False
    )
    if built.returncode:
        print(build_failure(built.returncode), file=sys.stderr)
        return 2
    binary = executable(built.stdout)
    listing = libtest(binary, "--list", included)
    if listing.returncode:
        raise ValueError(discovery_failure(listing.returncode))
    absent = missing(names, available(listing.stdout), selected=selected)
    if absent:
        print("\n".join(absent), file=sys.stderr)
        return 4
    results: list[bool] = []
    for name in names:
        result = libtest(binary, *single_test(name, included))
        done = verdict_without_output(name, result.returncode, result.stdout)
        print("\n".join(done.lines), flush=True)
        results.append(done.passed)
    print(passed(FrozenList(results), selected=selected))
    return 0 if all(results) else 5


def main() -> int:
    parser = argparse.ArgumentParser(description=DESCRIPTION)
    _ = parser.add_argument("test", nargs="?", help=TEST_HELP)
    target = parser.parse_args(namespace=Arguments()).test or ""
    if target and not NAME.fullmatch(target):
        parser.error(NAME_ERROR)
    try:
        return run(target)
    except (OSError, ValueError, KeyError) as error:
        print(f"INFRASTRUCTURE FAILURE: {error}", file=sys.stderr)
        return 3
