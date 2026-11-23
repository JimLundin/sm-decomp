"""Run crates' libtest suites as DLLs, for Windows machines that block test executables.

Defender quarantines freshly linked test `.exe` files but loads DLLs. One `cargo test --no-run`
builds every enabled unit and integration-test target, so cargo schedules them all across every
core. Each harness is linked `-shared` and exports only the generated libtest `main` (explicit
exports avoid the PE ordinal limit when Rust's internal symbols exceed the export table). A hard
link names each harness `.dll` in the output directory, and a child Python calls its `main`
through ctypes. libtest reads its arguments from `GetCommandLineW`, so the child rewrites that
buffer in place before calling `main`. The suites run concurrently, at most one per core
(SM_TEST_JOBS overrides), and each one's output prints whole when it finishes; a lone suite
streams its output.

    python test_dll.py <cargo> <out-dir> [--suite lib|test:NAME] <crate>... [-- <libtest args>]
    python test_dll.py --run <dll> [<libtest args>]

The wrapper exits 0 when all suites pass, 1 for ordinary libtest failures (child 101),
and 2 for build, launch or other process failures, which take precedence.
"""

import ctypes
import json
import os
import shutil
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

# The link arguments are GNU ld's. Naming the target keeps the flags off build scripts and
# proc-macros, which cargo then builds for the host and runs as executables.
TRIPLE = "x86_64-pc-windows-gnu"

Target = tuple[str, str, str, Path]


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


def targets(cargo: str, crates: list[str], suite: str | None = None) -> list[Target]:
    meta = json.loads(
        subprocess.run(
            [cargo, "metadata", "--no-deps", "--format-version", "1"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout
    )
    packages = {p["name"]: p for p in meta["packages"]}
    found: list[Target] = []
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


def build_command(cargo: str, selected: list[Target], suite: str | None) -> list[str]:
    build = [cargo, "test", "--no-run", "--no-fail-fast", "-q", "--message-format=json-render-diagnostics"]
    build += ["--target", TRIPLE]
    for crate in dict.fromkeys(crate for crate, _, _, _ in selected):
        build += ["-p", crate]
    if suite is None:
        return build + ["--tests"]  # The targets with `test = true`, as targets() selects.
    _, kind, name, _ = selected[0]
    return build + [kind] + ([name] if name else [])


def build_env(definition: Path) -> dict[str, str]:
    """This environment, with each harness linked as a DLL that exports only `main`.

    Cargo prefers the encoded flags to RUSTFLAGS, so any RUSTFLAGS carry over into them; the
    encoding keeps a definition path with spaces in one argument."""
    env = dict(os.environ)
    encoded = env.get("CARGO_ENCODED_RUSTFLAGS")
    flags = encoded.split("\x1f") if encoded else env.get("RUSTFLAGS", "").split()
    flags += ["-C", "link-arg=-shared", "-C", f"link-arg={definition.as_posix()}"]
    env["CARGO_ENCODED_RUSTFLAGS"] = "\x1f".join(flags)
    return env


def export_definition(directory: Path) -> Path:
    """The definition file that disables GNU PE auto-export of Rust's internal symbols.

    Its path is in the flags, and cargo rebuilds every crate when the flags change, so every
    lane's output directory shares the one in their parent: a lane whose target/ is a copy of
    another's build (worktree.sh) then reuses it."""
    definition = directory / "exports.def"
    content = b"EXPORTS\nmain\n"
    if definition.is_file() and definition.read_bytes() == content:
        return definition
    partial = directory / f"exports.def.{os.getpid()}"
    partial.write_bytes(content)
    os.replace(partial, definition)
    return definition


def artifacts(stdout: str, selected: list[Target]) -> dict[Target, Path]:
    """The harness built for each selected target, from cargo's JSON messages."""
    by_source = {(target[3], target[1], target[2]): target for target in selected}
    built: dict[Target, Path] = {}
    for line in stdout.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") != "compiler-artifact" or not message.get("executable"):
            continue
        if not message.get("profile", {}).get("test"):
            continue  # A binary an integration test depends on, not a harness.
        kinds, name = message["target"]["kind"], message["target"]["name"]
        kind = "--test" if "test" in kinds else "--bin" if "bin" in kinds else "--lib"
        source = (Path(message["manifest_path"]).parent, kind, "" if kind == "--lib" else name)
        if source in by_source:
            built[by_source[source]] = Path(message["executable"])
    return built


def place(harness: Path, dll: Path) -> None:
    """Names the harness `.dll` for the loader, by hard link where the file system allows."""
    if dll.exists() and os.path.samefile(harness, dll):
        return
    dll.unlink(missing_ok=True)
    try:
        os.link(harness, dll)
    except OSError:
        shutil.copy2(harness, dll)


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
    try:
        selected = targets(cargo, crates, suite)
    except (OSError, subprocess.CalledProcessError, ValueError, SystemExit):
        print("test_dll: target discovery failed")
        return 2
    try:
        definition = export_definition(out.parent)
    except OSError:
        print("test_dll: infrastructure failed: export preparation")
        return 2
    try:
        build = subprocess.run(build_command(cargo, selected, suite), stdout=subprocess.PIPE,
                               text=True, env=build_env(definition))
        built = artifacts(build.stdout or "", selected)
    except (OSError, ValueError, KeyError, TypeError):
        print("test_dll: infrastructure failed: build launch")
        return 2
    failed: list[str] = []
    infrastructure: list[str] = [] if build.returncode == 0 else ["build"]
    runnable: list[tuple[str, Path, Path]] = []
    for target in selected:
        crate, kind, name, root = target
        label = f"{crate} {kind} {name}".strip()
        dll = out / f"{crate}-{name or 'lib'}.dll"
        if target not in built:
            infrastructure.append(f"{label} (build)")
            continue
        try:
            place(built[target], dll)
        except OSError:
            infrastructure.append(f"{label} (placement)")
            continue
        runnable.append((label, dll, root))
    alone = len(runnable) == 1
    lock = threading.Lock()

    def run(label: str, dll: Path, root: Path) -> None:
        command = [sys.executable, "-I", __file__, "--run", str(dll), *libtest]
        try:
            if alone:
                print(f"     Running {label} ({dll.name})", flush=True)
                result = subprocess.run(command, cwd=root)
            else:
                result = subprocess.run(command, cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        except OSError:
            with lock:
                infrastructure.append(f"{label} (test launch)")
            return
        with lock:
            if not alone:
                print(f"     Running {label} ({dll.name})")
                print((result.stdout or b"").decode("utf-8", "replace"), end="", flush=True)
            # libtest's ordinary completed test failure exits 101. Other child statuses
            # include crashes and launch failures; they cannot be a model counterexample.
            if result.returncode == 101:
                failed.append(label)
            elif result.returncode != 0:
                infrastructure.append(f"{label} (test process)")

    jobs = max(1, int(os.environ.get("SM_TEST_JOBS") or os.cpu_count() or 1))
    with ThreadPoolExecutor(max_workers=jobs) as pool:
        for future in [pool.submit(run, *suite_run) for suite_run in runnable]:
            future.result()
    if infrastructure:
        print("test_dll: infrastructure failed: " + ", ".join(sorted(infrastructure)))
        return 2
    if failed:
        print("test_dll: FAILED " + ", ".join(sorted(failed)))
        return 1
    print("test_dll: all targets passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
