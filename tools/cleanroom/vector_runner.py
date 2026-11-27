"""Run a clean-room export's tests, the one Python a confined implementer may run.

It runs each lib and integration-test target of the clean crates (those under `crates/clean/`)
in the current workspace with `cargo test`.

Before the tests it runs the shape rules import.sh refuses a crate for (shape_scan.awk: G6, a
struct that only carries one call's arguments, and G7, a cycle of module references) over each
clean crate's library sources, as code_check.sh does, and fails on a finding. DESIGN.md states both rules; the scan reads only the export's own sources, and its
findings name only their files, lines and modules.

It takes no arguments: launch.sh copies it and the two scan scripts it runs into the lane
directory, out of the session's reach, writes beside them the absolute path of the awk it runs
(awk.path) and checks their sha256 after the session.
"""

import json
import shutil
import subprocess
import sys
from pathlib import Path

# The scripts of code_check.sh's shape rules, beside this runner in the repository and in a lane.
SHAPE_SCAN = ("strip_rust.awk", "shape_scan.awk")
AWK_PIN = "awk.path"
HERE = Path(__file__).resolve().parent


def clean_targets(cargo: str, workspace: Path) -> list[tuple[str, str, str, Path]]:
    meta = json.loads(
        subprocess.run(
            [cargo, "metadata", "--no-deps", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
            cwd=workspace,
        ).stdout
    )
    clean = workspace / "crates" / "clean"
    found = []
    for package in meta["packages"]:
        name = package["name"]
        manifest = Path(package["manifest_path"]).resolve()
        if clean not in manifest.parents:
            continue
        for target in package["targets"]:
            kinds = target["kind"]
            if "test" in kinds:
                found.append((name, "--test", target["name"], manifest.parent))
            elif "lib" in kinds and target.get("test", True):
                found.append((name, "--lib", "", manifest.parent))
    return found


def library_sources(crate: Path) -> list[str]:
    """The crate's library sources as code_check.sh takes them: src/ less its test files."""
    found = []
    for path in sorted((crate / "src").rglob("*.rs")):
        relative = path.relative_to(crate)
        if path.is_symlink() or path.name == "tests.rs" or "tests" in relative.parts[1:-1]:
            continue
        found.append(relative.as_posix())
    return found


def shape_findings(awk: str, crates: list[Path]) -> list[str]:
    """code_check.sh's shape findings in each crate, one line each, as `PATH: TAG: DETAIL`."""
    scripts = [arg for name in SHAPE_SCAN for arg in ("-f", (HERE / name).as_posix())]
    findings = []
    for crate in crates:
        sources = library_sources(crate)
        if not sources:
            continue
        scan = subprocess.run([awk, *scripts, *sources], cwd=crate, capture_output=True,
                              encoding="utf-8", errors="replace", check=False)
        if scan.returncode != 0:
            findings.append(f"crates/clean/{crate.name}: the shape scan could not run")
            continue
        for line in scan.stdout.splitlines():
            tag, path, detail = line.split("\t", 2)
            findings.append(f"crates/clean/{crate.name}/src/{path}: {tag}: {detail}")
    return findings


def pinned_awk(workspace: Path) -> str:
    """The awk awk.path names, an absolute path to a file outside the workspace, once the scan
    scripts it runs are beside the runner."""
    try:
        awk = Path((HERE / AWK_PIN).read_text(encoding="utf-8").strip())
    except OSError:
        sys.exit(f"vector_runner: no {AWK_PIN} beside the runner to name the awk it runs")
    if not awk.is_absolute() or not awk.is_file() or awk.resolve().is_relative_to(workspace):
        sys.exit(f"vector_runner: the pinned awk {awk} is not a file outside the workspace")
    if not all((HERE / name).is_file() for name in SHAPE_SCAN):
        sys.exit("vector_runner: no shape scan scripts for the module and argument rules")
    return str(awk)


def shape_failures(awk: str, crates: list[Path]) -> list[str]:
    """Prints each shape finding; the run's failed entry for them, if any."""
    shapes = shape_findings(awk, crates)
    for finding in shapes:
        print(f"vector_runner: {finding}", flush=True)
    return [f"shapes ({len(shapes)} findings)"] if shapes else []


def run_target(cargo: str, workspace: Path, target: tuple[str, str, str, Path]) -> str | None:
    """Run one test target and return its label on failure."""
    crate, kind, name, _ = target
    label = f"{crate} {kind} {name}".strip()
    test = [cargo, "test", "-p", crate, kind, *([name] if name else [])]
    print(f"     Running {label}", flush=True)
    return label if subprocess.run(test, cwd=workspace).returncode != 0 else None


def run(cargo: str, workspace: Path) -> int:
    """The shape rules, then every clean crate test target; 1 when anything fails."""
    awk = pinned_awk(workspace)
    targets = clean_targets(cargo, workspace)
    if not targets:
        sys.exit("vector_runner: no clean crate test targets here")
    failed = shape_failures(awk, list(dict.fromkeys(root for *_, root in targets)))
    for target in targets:
        if (failure := run_target(cargo, workspace, target)) is not None:
            failed.append(failure)
    if failed:
        print("vector_runner: FAILED " + ", ".join(failed), flush=True)
        return 1
    print("vector_runner: all targets passed", flush=True)
    return 0


def main() -> int:
    if len(sys.argv) > 1:
        sys.exit("vector_runner: takes no arguments")
    cargo = shutil.which("cargo")
    if cargo is None:
        sys.exit("vector_runner: no cargo on PATH")
    return run(cargo, Path.cwd().resolve())


if __name__ == "__main__":
    sys.exit(main())
