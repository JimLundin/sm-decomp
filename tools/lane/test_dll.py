"""Run crates' libtest suites as DLLs, for Windows machines that block test executables.

Defender quarantines freshly linked test `.exe` files but loads DLLs. Each lib and
integration-test target is linked with `-shared`, so the generated libtest `main`
is an export of a DLL, and a child Python calls it through ctypes. libtest reads
its arguments from `GetCommandLineW`, so the child rewrites that buffer in place
before calling `main`.

    python test_dll.py <cargo> <out-dir> <crate>... [-- <libtest args>]
    python test_dll.py --run <dll> [<libtest args>]
"""

import ctypes
import json
import os
import subprocess
import sys
from pathlib import Path


def run_dll(dll: str, args: list[str]) -> int:
    # The harness only exists for Windows; the guard also tells mypy on Linux that WinDLL exists.
    if sys.platform != "win32":
        sys.exit("test_dll: --run is for Windows; run cargo test instead")
    kernel32 = ctypes.WinDLL("kernel32")
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


def targets(cargo: str, crates: list[str]) -> list[tuple[str, str, str, Path]]:
    meta = json.loads(
        subprocess.run(
            [cargo, "metadata", "--no-deps", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    packages = {p["name"]: p for p in meta["packages"]}
    found = []
    for crate in crates:
        if crate not in packages:
            sys.exit(f"test_dll: no package {crate}")
        package = packages[crate]
        root = Path(package["manifest_path"]).parent
        for target in package["targets"]:
            kinds = target["kind"]
            if "test" in kinds:
                found.append((crate, "--test", target["name"], root))
            elif "lib" in kinds and target.get("test", True):
                found.append((crate, "--lib", "", root))
    return found


def main() -> int:
    if sys.argv[1] == "--run":
        return run_dll(sys.argv[2], sys.argv[3:])
    cargo, out = sys.argv[1], Path(sys.argv[2])
    rest = sys.argv[3:]
    split = rest.index("--") if "--" in rest else len(rest)
    crates, libtest = rest[:split], rest[split + 1 :]
    out.mkdir(parents=True, exist_ok=True)
    failed = []
    for crate, kind, name, root in targets(cargo, crates):
        label = f"{crate} {kind} {name}".strip()
        dll = out / f"{crate}-{name or 'lib'}.dll"
        build = [cargo, "rustc", "-q", "-p", crate, "--profile", "test", kind]
        build += [name] if name else []
        build += ["--", "-C", "link-arg=-shared", "-C", "link-arg=-o", "-C", f"link-arg={dll.as_posix()}"]
        print(f"     Running {label} ({dll.name})", flush=True)
        if subprocess.run(build).returncode != 0:
            failed.append(f"{label} (build)")
            continue
        run = [sys.executable, "-I", __file__, "--run", str(dll), *libtest]
        if subprocess.run(run, cwd=root).returncode != 0:
            failed.append(label)
    if failed:
        print("test_dll: FAILED " + ", ".join(failed))
        return 1
    print("test_dll: all targets passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
