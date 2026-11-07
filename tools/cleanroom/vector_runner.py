"""Run a clean-room export's tests, the one Python a confined implementer may run.

Defender quarantines freshly linked test `.exe` files but loads DLLs, so `cargo test` cannot
run in a clean-room session on Windows. This links each lib and integration-test target of the
clean crates (`clean_types`, `clean_model_*`) in the current directory's workspace as a DLL and
calls its libtest `main` in a child interpreter, as tools/lane/test_dll.py does; libtest reads
its arguments from `GetCommandLineW`, so the child rewrites that buffer first.

It takes no arguments: launch.sh copies it into the lane directory, out of the session's reach,
allows exactly `<python> -I <that copy>` and checks its sha256 after the session.
"""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

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
    crates = workspace / "crates"
    found = []
    for package in meta["packages"]:
        name = package["name"]
        manifest = Path(package["manifest_path"]).resolve()
        if not (name == "clean_types" or name.startswith("clean_model_")):
            continue
        if crates not in manifest.parents:
            continue
        for target in package["targets"]:
            kinds = target["kind"]
            if "test" in kinds:
                found.append((name, "--test", target["name"], manifest.parent))
            elif "lib" in kinds and target.get("test", True):
                found.append((name, "--lib", "", manifest.parent))
    return Path(meta["target_directory"]), found


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
