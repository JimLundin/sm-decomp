"""Run crates' libtest suites as DLLs, for Windows machines that block test executables.

Defender quarantines freshly linked test `.exe` files but loads DLLs. Each enabled unit and
integration-test target is linked with `-shared`, so the generated libtest `main`
is an export of a DLL, and a child Python calls it through ctypes. libtest reads
its arguments from `GetCommandLineW`, so the child rewrites that buffer in place
before calling `main`.

    python test_dll.py <cargo> <out-dir> [--suite lib|test:NAME] <crate>... [-- <libtest args>]
    python test_dll.py --run <dll> [<libtest args>]

The wrapper exits 0 when all suites pass, 1 for ordinary libtest failures (child 101),
and 2 for build, launch or other process failures, which take precedence.
"""

import ctypes
import json
import os
import subprocess
import sys
from pathlib import Path


def run_dll(dll: str, args: list[str]) -> int:
    # The harness is for Windows only. WinDLL is looked up at run time, because mypy checks this
    # file against Linux's ctypes, which has no WinDLL.
    win_dll = getattr(ctypes, "WinDLL", None)
    if win_dll is None:
        sys.exit("test_dll: --run is for Windows; run cargo test instead")
    kernel32 = win_dll("kernel32")
    kernel32.GetCommandLineW.restype = ctypes.c_void_p
    line = kernel32.GetCommandLineW()
    new = subprocess.list2cmdline([dll, *args])
    # The new line quotes a subset of this process's arguments, so it fits.
    if len(new) > len(ctypes.wstring_at(line)):
        sys.exit("test_dll: libtest arguments do not fit the command line")
    ctypes.memmove(line, ctypes.create_unicode_buffer(new), (len(new) + 1) * 2)
    harness = ctypes.CDLL(dll)
    argv = (ctypes.c_char_p * 2)(dll.encode(), None)
    sys.stdout.flush()
    status: int = harness.main(1, argv)
    return status


def targets(cargo: str, crates: list[str], suite: str | None = None) -> list[tuple[str, str, str, Path]]:
    meta = json.loads(
        subprocess.run(
            [cargo, "metadata", "--no-deps", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    packages = {p["name"]: p for p in meta["packages"]}
    found: list[tuple[str, str, str, Path]] = []
    for crate in crates:
        if crate not in packages:
            sys.exit(f"test_dll: no package {crate}")
        package = packages[crate]
        root = Path(package["manifest_path"]).parent
        before = len(found)
        for target in package["targets"]:
            if not target.get("test", True):
                continue
            kinds = target["kind"]
            if "test" in kinds:
                found.append((crate, "--test", target["name"], root))
            elif "lib" in kinds or "proc-macro" in kinds:
                found.append((crate, "--lib", "", root))
            elif "bin" in kinds:
                found.append((crate, "--bin", target["name"], root))
        if len(found) == before:
            sys.exit(f"test_dll: no enabled test targets for {crate}")
    if suite is not None:
        if len(crates) != 1:
            sys.exit("test_dll: a selected suite requires one package")
        kind, name = ("--lib", "") if suite == "lib" else ("--test", suite.removeprefix("test:"))
        if suite != "lib" and (not suite.startswith("test:") or not name):
            sys.exit("test_dll: suite must be lib or test:NAME")
        found = [target for target in found if target[1:3] == (kind, name)]
        if len(found) != 1:
            sys.exit("test_dll: selected suite is absent, disabled or ambiguous")
    return found


def build_command(cargo: str, crate: str, kind: str, name: str, dll: Path) -> list[str]:
    build = [cargo, "rustc", "-q", "-p", crate, "--profile", "test", kind]
    build += [name] if name else []
    build += ["--"]
    return build + ["-C", "link-arg=-shared", "-C", "link-arg=-o", "-C", f"link-arg={dll.as_posix()}"]


def main() -> int:
    if sys.argv[1] == "--run":
        return run_dll(sys.argv[2], sys.argv[3:])
    cargo, out = sys.argv[1], Path(sys.argv[2])
    rest = sys.argv[3:]
    suite = None
    if rest and rest[0] == "--suite":
        if len(rest) < 3:
            sys.exit("test_dll: --suite needs a selector and package")
        suite, rest = rest[1], rest[2:]
    split = rest.index("--") if "--" in rest else len(rest)
    crates, libtest = rest[:split], rest[split + 1 :]
    if not crates:
        sys.exit("test_dll: name at least one package")
    try:
        out.mkdir(parents=True, exist_ok=True)
    except OSError:
        print("test_dll: output preparation failed")
        return 2
    failed = []
    infrastructure = []
    try:
        selected = targets(cargo, crates, suite)
    except (OSError, subprocess.CalledProcessError, ValueError, SystemExit):
        print("test_dll: target discovery failed")
        return 2
    for crate, kind, name, root in selected:
        label = f"{crate} {kind} {name}".strip()
        dll = out / f"{crate}-{name or 'lib'}.dll"
        build = build_command(cargo, crate, kind, name, dll)
        print(f"     Running {label} ({dll.name})", flush=True)
        try:
            build_status = subprocess.run(build).returncode
        except OSError:
            infrastructure.append(f"{label} (build launch)")
            continue
        if build_status != 0:
            infrastructure.append(f"{label} (build)")
            continue
        run = [sys.executable, "-I", __file__, "--run", str(dll), *libtest]
        try:
            status = subprocess.run(run, cwd=root).returncode
        except OSError:
            infrastructure.append(f"{label} (test launch)")
            continue
        # libtest's ordinary completed test failure exits 101. Other child statuses
        # include crashes and launch failures; they cannot be a model counterexample.
        if status == 101:
            failed.append(label)
        elif status != 0:
            infrastructure.append(f"{label} (test process)")
    if infrastructure:
        print("test_dll: infrastructure failed: " + ", ".join(infrastructure))
        return 2
    if failed:
        print("test_dll: FAILED " + ", ".join(failed))
        return 1
    print("test_dll: all targets passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
