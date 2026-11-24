"""Run a clean-room export's tests, the one Python a confined implementer may run.

Defender quarantines freshly linked test `.exe` files but loads DLLs, so `cargo test` cannot
run in a clean-room session on Windows. This links each lib and integration-test target of the
clean crates (those under `crates/clean/`) in the current directory's workspace as a DLL and
calls its libtest `main` in a child interpreter, as tools/lane/test_dll.py does; libtest reads
its arguments from `GetCommandLineW`, so the child rewrites that buffer first.

Before the tests it runs the shape rules import.sh refuses a crate for (shape_scan.awk: G6, a
struct that only carries one call's arguments, and G7, a cycle of module references) over each
clean crate's library sources, as code_check.sh does, and fails on a finding. DESIGN.md states both rules; the scan reads only the export's own sources, and its
findings name only their files, lines and modules.

It takes no arguments: launch.sh copies it and the two scan scripts it runs into the lane
directory, out of the session's reach, allows exactly `<python> -I <that copy>` and checks their
sha256 after the session.
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

# The scripts of code_check.sh's shape rules, beside this runner in the repository and in a lane.
SHAPE_SCAN = ("strip_rust.awk", "shape_scan.awk")

# The child: rewrite the command line to the DLL alone, then call its libtest main.
CHILD = """
import ctypes, subprocess, sys
dll = sys.argv[1]
kernel32 = ctypes.WinDLL("kernel32")
kernel32.GetCommandLineW.restype = ctypes.c_void_p
line = kernel32.GetCommandLineW()
new = subprocess.list2cmdline([dll])
if len(new) > len(ctypes.wstring_at(line)):
    sys.exit("vector_runner: the DLL path does not fit the command line")
ctypes.memmove(line, ctypes.create_unicode_buffer(new), (len(new) + 1) * 2)
harness = ctypes.CDLL(dll)
argv = (ctypes.c_char_p * 2)(dll.encode(), None)
sys.stdout.flush()
sys.exit(harness.main(1, argv))
"""


def clean_targets(cargo: str, workspace: Path) -> tuple[Path, list[tuple[str, str, str, Path]]]:
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
    return Path(meta["target_directory"]), found


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
    here = Path(__file__).resolve().parent
    scripts = [arg for name in SHAPE_SCAN for arg in ("-f", (here / name).as_posix())]
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


def main() -> int:
    if len(sys.argv) > 1:
        sys.exit("vector_runner: takes no arguments")
    if os.name != "nt":
        sys.exit("vector_runner: for Windows; run cargo test instead")
    cargo = shutil.which("cargo")
    if cargo is None:
        sys.exit("vector_runner: no cargo on PATH")
    workspace = Path.cwd().resolve()
    target_dir, targets = clean_targets(cargo, workspace)
    if not targets:
        sys.exit("vector_runner: no clean crate test targets here")
    out = target_dir / "vector-runner"
    out.mkdir(parents=True, exist_ok=True)
    failed = []
    awk = shutil.which("awk")
    here = Path(__file__).resolve().parent
    if awk is None or not all((here / name).is_file() for name in SHAPE_SCAN):
        sys.exit("vector_runner: no awk or shape scan scripts for the module and argument rules")
    shapes = shape_findings(awk, list(dict.fromkeys(root for *_, root in targets)))
    for finding in shapes:
        print(f"vector_runner: {finding}", flush=True)
    if shapes:
        failed.append(f"shapes ({len(shapes)} findings)")
    for crate, kind, name, root in targets:
        label = f"{crate} {kind} {name}".strip()
        dll = out / f"{crate}-{name or 'lib'}.dll"
        build = [cargo, "rustc", "-q", "-p", crate, "--profile", "test", kind]
        build += [name] if name else []
        build += ["--", "-C", "link-arg=-shared", "-C", "link-arg=-o", "-C", f"link-arg={dll.as_posix()}"]
        print(f"     Running {label} ({dll.name})", flush=True)
        if subprocess.run(build, cwd=workspace).returncode != 0:
            failed.append(f"{label} (build)")
            continue
        # cargo test runs a target in its package's directory.
        run = [sys.executable, "-I", "-c", CHILD, str(dll)]
        if subprocess.run(run, cwd=root).returncode != 0:
            failed.append(label)
    if failed:
        print("vector_runner: FAILED " + ", ".join(failed), flush=True)
        return 1
    print("vector_runner: all targets passed", flush=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
