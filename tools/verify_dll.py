#!/usr/bin/env python3
"""Build the Windows verifier DLL, then run one replay via ctypes."""
import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time


def source_hash(root):
    """Bind cached replays to source contents, including added/deleted modules."""
    paths = set((root / "sm").rglob("*.rs"))
    paths.update(root / name for name in (
        "Cargo.toml", "Cargo.lock", "sm/Cargo.toml", "rust-toolchain.toml",
        "tools/verify_dll.py"))
    paths.update(path for path in (root / ".cargo").glob("*") if path.is_file())
    digest = hashlib.sha256()
    for path in sorted(paths):
        data = path.read_bytes()
        digest.update(path.relative_to(root).as_posix().encode("utf-8") + b"\0")
        digest.update(len(data).to_bytes(8, "little"))
        digest.update(data)
    return digest.hexdigest()


def check_source(host, expected):
    if not hasattr(host, "sm_verify_source_hash"):
        raise RuntimeError("cached DLL has no source stamp; run without --no-build")
    host.sm_verify_source_hash.argtypes = []
    host.sm_verify_source_hash.restype = ctypes.c_void_p
    stamp = host.sm_verify_source_hash()
    if not stamp or ctypes.string_at(stamp, 64) != expected.encode("ascii"):
        raise RuntimeError("cached DLL does not match current source; run without --no-build")


def load_host(dll):
    # Keep SDK search directories alive for as long as this DLL is loaded.
    directories = []
    for path in os.environ.get("LIBRARY_PATH", "").split(os.pathsep):
        if path:
            for directory in (Path(path), Path(path).parent / "bin"):
                if directory.is_dir():
                    directories.append(os.add_dll_directory(str(directory.resolve())))
    host = ctypes.CDLL(str(dll))
    host._sdk_directories = directories
    return host


def check_library(host, root):
    """Compare the rlib's compiled source bytes, not the host's environment."""
    if not hasattr(host, "sm_verify_source_file"):
        raise RuntimeError("cached DLL has no game-library source data; rebuild")

    class SourceFile(ctypes.Structure):
        _fields_ = [("path", ctypes.c_void_p), ("path_len", ctypes.c_size_t),
                    ("data", ctypes.c_void_p), ("data_len", ctypes.c_size_t)]

    host.sm_verify_source_file.argtypes = [ctypes.c_size_t]
    host.sm_verify_source_file.restype = SourceFile
    expected = {path.relative_to(root).as_posix()
                for path in (root / "sm" / "src").rglob("*.rs")}
    expected.difference_update(("sm/src/main.rs", "sm/src/verify.rs",
                                "sm/src/verify_dll.rs", "sm/src/verify/checkpoint.rs"))
    for index in range(len(expected) + 1):
        source = host.sm_verify_source_file(index)
        if source.path_len == 0:
            break
        path = ctypes.string_at(source.path, source.path_len).decode("utf-8")
        if path not in expected:
            raise RuntimeError(f"unexpected or repeated game-library source: {path}; rebuild")
        expected.remove(path)
        if ctypes.string_at(source.data, source.data_len) != (root / path).read_bytes():
            raise RuntimeError(f"stale game-library source: {path}; rebuild")
    if expected:
        raise RuntimeError("game-library source list incomplete; update sm/src/verify_source.rs: "
                           + ", ".join(sorted(expected)))


def cargo_host(root, rustc, linker, output, source, retry=True):
    cargo = shutil.which("cargo") or str(Path(rustc).with_name("cargo.exe"))
    sdk = [arg for path in os.environ.get("LIBRARY_PATH", "").split(os.pathsep)
           if path for arg in ("-L", f"native={path}")]
    # Backdated files in another checkout can look fresh in a shared target
    # directory. Keep this game's and host's artifacts with their source.
    result = subprocess.run(
        [cargo, "rustc", "-p", "sm", "--release", "--example", "sm_verify", "--locked",
         "--target-dir", str(root / "target"), "--message-format=json",
         "--", "-C", f"linker={linker}",
         "-C", "linker-flavor=ld", "-C", "debuginfo=0", "-A", "warnings"] + sdk,
        cwd=root, env=dict(os.environ, RUSTC=rustc, SM_VERIFY_SOURCE_HASH=source),
        text=True, encoding="utf-8",
        stdout=subprocess.PIPE,
    )
    dll = None
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message["reason"] == "compiler-message":
            if message["message"]["level"] in ("error", "failure-note"):
                print(message["message"]["rendered"] or "", end="", file=sys.stderr)
        elif (message["reason"] == "compiler-artifact"
              and message["target"]["name"] == "sm_verify"):
            dll = next((Path(name) for name in message["filenames"]
                        if name.endswith(".dll")), None)
    result.check_returncode()
    if dll is None:
        raise RuntimeError("Cargo did not report the verifier DLL")
    # Inspect in a separate process so a stale DLL is unloaded before rebuilding.
    checked = subprocess.run(
        [sys.executable, str(root / "tools/verify_dll.py"), "--check-library", str(dll)],
        cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
    )
    if checked.returncode:
        if not retry:
            raise RuntimeError(checked.stderr.strip() or "unable to inspect game library")
        print(checked.stderr.strip(), file=sys.stderr)
        print("Rebuilding sm after game-library source check failed", file=sys.stderr)
        # Invalidate Cargo's timestamp check without deleting artifacts or
        # changing source contents. The crate root is always a library input.
        # This also works with pre-existing/seeded targets lacking CACHEDIR.TAG.
        (root / "sm/src/lib.rs").touch()
        return cargo_host(root, rustc, linker, output, source, retry=False)
    # Preserve --no-build's existing path. An unchanged DLL can still be loaded
    # by another replay; do not overwrite it merely to refresh its timestamp.
    cached = output / "sm_verify.dll"
    if not cached.exists() or cached.read_bytes() != dll.read_bytes():
        shutil.copyfile(dll, cached)
    return cached


def build(root, rustc, use_cargo=False):
    source = source_hash(root)
    output = root / "target" / "verify-dll"
    output.mkdir(parents=True, exist_ok=True)
    library = output / "libsm.rlib"
    dll = output / "sm_verify.dll"
    # Retain 16 codegen units; incremental compilation otherwise defaults to 256.
    common = [rustc, "--edition=2021", "-C", "opt-level=3",
              "-C", "codegen-units=16", "-A", "warnings"]
    sysroot = Path(subprocess.check_output(
        [rustc, "--print", "sysroot"], cwd=root, text=True).strip())
    host = next(line.removeprefix("host: ") for line in subprocess.check_output(
        [rustc, "-vV"], cwd=root, text=True).splitlines() if line.startswith("host: "))
    if host != "x86_64-pc-windows-gnu":
        raise RuntimeError("DLL host requires the pinned x86_64-pc-windows-gnu toolchain")
    # The bundled GNU linker cannot read bigobj emitted for the large game;
    # Rust's bundled lld handles those object files without external tools.
    linker = sysroot / "lib" / "rustlib" / host / "bin" / "gcc-ld" / "ld.lld.exe"
    if use_cargo:
        dll = cargo_host(root, rustc, linker, output, source)
        if source_hash(root) != source:
            raise RuntimeError("source changed during build; rebuild before replay")
        return dll
    subprocess.run(
        common + ["--crate-name", "sm", "--crate-type", "rlib", "--cfg",
                  "sm_verify_dll", "-C", f"incremental={output / 'incremental-sm'}",
                  "sm/src/lib.rs", "-o", str(library)],
        cwd=root, check=True,
    )
    subprocess.run(
        common + ["--crate-name", "sm_verify", "--crate-type", "cdylib",
                  "-C", f"linker={linker}", "-C", "linker-flavor=ld",
                  "-C", f"incremental={output / 'incremental-host'}", "sm/src/verify_dll.rs",
                  "--extern", f"sm={library}", "-o", str(dll)],
        cwd=root, env=dict(os.environ, SM_VERIFY_SOURCE_HASH=source), check=True,
    )
    if source_hash(root) != source:
        raise RuntimeError("source changed during build; rebuild before replay")
    return dll


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rom", type=Path)
    parser.add_argument("inputs", type=Path)
    parser.add_argument("--frames", type=int)
    parser.add_argument("--checkpoint-in", type=Path,
                        help="resume a local verifier checkpoint; --frames counts from that point")
    parser.add_argument("--checkpoint-out", type=Path,
                        help="save a new local verifier checkpoint after the last frame")
    parser.add_argument("--require-hit", action="append", default=[], metavar="PROBE",
                        help="require an explicit routine/branch probe to execute (repeatable)")
    parser.add_argument("--after-hits", type=int, metavar="N",
                        help="stop N additional frames after all required probes first execute; "
                             "fail if the movie or --frames limit ends earlier")
    parser.add_argument("--no-build", action="store_true",
                        help="reuse target/verify-dll/sm_verify.dll if its source still matches")
    parser.add_argument("--cargo", action="store_true",
                        help="reuse Cargo release compilation (requires the SDL2 environment)")
    parser.add_argument("--rustc", default=shutil.which("rustc") or str(
        Path.home() / ".cargo" / "bin" / "rustc.exe"))
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("use cargo run --release -- verify on Linux")
    if args.frames is not None and not 1 <= args.frames <= 2**64 - 1:
        parser.error("--frames must be a positive u64")
    if args.after_hits is not None:
        if not 0 <= args.after_hits <= 2**64 - 1:
            parser.error("--after-hits must be a nonnegative u64")
        if not args.require_hit:
            parser.error("--after-hits requires at least one --require-hit")
    root = Path(__file__).resolve().parents[1]
    started = time.perf_counter()
    source = source_hash(root)
    # Paths are resolved before changing cwd so invocation works from anywhere.
    rom, inputs = args.rom.resolve(), args.inputs.resolve()
    replay_args = [str(rom), str(inputs)]
    if args.frames is not None:
        replay_args.extend(["--frames", str(args.frames)])
    for probe in args.require_hit:
        replay_args.extend(["--require-hit", probe])
    if args.after_hits is not None:
        replay_args.extend(["--after-hits", str(args.after_hits)])
    for option, path in (("--checkpoint-in", args.checkpoint_in),
                         ("--checkpoint-out", args.checkpoint_out)):
        if path is not None:
            replay_args.extend([option, str(path.resolve())])
    dll = root / "target" / "verify-dll" / "sm_verify.dll"
    if not args.no_build:
        dll = build(root, args.rustc, args.cargo)
    os.chdir(root)
    host = load_host(dll)
    check_source(host, source)
    check_library(host, root)
    if source_hash(root) != source:
        raise RuntimeError("source changed before replay; rebuild before replay")
    print(f"Verifier build/check: {time.perf_counter() - started:.2f}s; source {source}",
          file=sys.stderr, flush=True)
    if args.checkpoint_in is not None or args.checkpoint_out is not None or args.require_hit:
        if not hasattr(host, "sm_verify_args"):
            raise RuntimeError("cached DLL predates segment options; run without --no-build")
        host.sm_verify_args.argtypes = [ctypes.c_uint32, ctypes.POINTER(ctypes.c_char_p)]
        host.sm_verify_args.restype = ctypes.c_int
        argv = (ctypes.c_char_p * len(replay_args))(
            *(arg.encode("utf-8") for arg in replay_args))
        result = host.sm_verify_args(len(replay_args), argv)
    else:
        host.sm_verify.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint64]
        host.sm_verify.restype = ctypes.c_int
        result = host.sm_verify(str(rom).encode("utf-8"), str(inputs).encode("utf-8"),
                                args.frames or 0)
    if source_hash(root) != source:
        raise RuntimeError("source changed during replay; result applies only to the printed source")
    print(f"Verifier total: {time.perf_counter() - started:.2f}s", file=sys.stderr)
    return result


if __name__ == "__main__":
    try:
        if len(sys.argv) == 3 and sys.argv[1] == "--check-library":
            check_library(load_host(sys.argv[2]), Path(__file__).resolve().parents[1])
            sys.exit(0)
        sys.exit(main())
    except (OSError, RuntimeError, ValueError, subprocess.CalledProcessError) as error:
        print(f"verify: {error}", file=sys.stderr)
        sys.exit(2)
