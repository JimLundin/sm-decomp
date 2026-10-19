#!/usr/bin/env python3
"""Build the Windows verifier DLL, then run one replay via ctypes."""
import argparse
import ctypes
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys


def cargo_library(root, rustc):
    env = dict(os.environ, RUSTC=rustc)
    cargo = shutil.which("cargo") or str(Path(rustc).with_name("cargo.exe"))
    # Cargo fingerprints can reuse stale package artifacts across checkouts.
    result = subprocess.run(
        [cargo, "build", "--release", "--lib", "--locked",
         "--target-dir", str(root / "target"), "--message-format=json"],
        cwd=root, env=env, text=True, encoding="utf-8", stdout=subprocess.PIPE,
    )
    library = None
    search_paths = []
    for line in result.stdout.splitlines():
        message = json.loads(line)
        if message["reason"] == "compiler-message":
            # Like the standalone build, omit warnings but retain build errors.
            if message["message"]["level"] in ("error", "failure-note"):
                print(message["message"]["rendered"] or "", end="", file=sys.stderr)
        elif message["reason"] == "build-script-executed":
            search_paths.extend(message["linked_paths"])
        elif (message["reason"] == "compiler-artifact"
              and Path(message["manifest_path"]) == root / "sm" / "Cargo.toml"):
            library = next((Path(name) for name in message["filenames"]
                            if name.endswith(".rlib")), None)
    result.check_returncode()
    if library is None:
        raise RuntimeError("Cargo did not report the sm release library")
    dependencies = library.parent
    if dependencies.name != "deps":
        dependencies = dependencies / "deps"
    search_paths.append(f"dependency={dependencies}")
    search_paths.extend(f"native={path}" for path in
                        os.environ.get("LIBRARY_PATH", "").split(os.pathsep) if path)
    return library, [arg for path in search_paths for arg in ("-L", path)]


def build(root, rustc, use_cargo=False):
    output = root / "target" / "verify-dll"
    output.mkdir(parents=True, exist_ok=True)
    library = output / "libsm.rlib"
    dll = output / "sm_verify.dll"
    # Retain 16 codegen units; incremental compilation otherwise defaults to 256.
    common = [rustc, "--edition=2021", "-C", "opt-level=3",
              "-C", "codegen-units=16", "-A", "warnings"]
    link_paths = []
    if use_cargo:
        library, link_paths = cargo_library(root, rustc)
    else:
        subprocess.run(
            common + ["--crate-name", "sm", "--crate-type", "rlib", "--cfg",
                      "sm_verify_dll", "-C", f"incremental={output / 'incremental-sm'}",
                      "sm/src/lib.rs", "-o", str(library)],
            cwd=root, check=True,
        )
    sysroot = Path(subprocess.check_output(
        [rustc, "--print", "sysroot"], cwd=root, text=True).strip())
    host = next(line.removeprefix("host: ") for line in subprocess.check_output(
        [rustc, "-vV"], cwd=root, text=True).splitlines() if line.startswith("host: "))
    if host != "x86_64-pc-windows-gnu":
        raise RuntimeError("DLL host requires the pinned x86_64-pc-windows-gnu toolchain")
    # The bundled GNU linker cannot read bigobj emitted for the large game;
    # Rust's bundled lld handles those object files without external tools.
    linker = sysroot / "lib" / "rustlib" / host / "bin" / "gcc-ld" / "ld.lld.exe"
    subprocess.run(
        common + ["--crate-name", "sm_verify", "--crate-type", "cdylib",
                  "-C", f"linker={linker}", "-C", "linker-flavor=ld",
                  "-C", f"incremental={output / 'incremental-host'}", "sm/src/verify_dll.rs",
                  "--extern", f"sm={library}", "-o", str(dll)] + link_paths,
        cwd=root, check=True,
    )
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
    parser.add_argument("--no-build", action="store_true",
                        help="reuse target/verify-dll/sm_verify.dll")
    parser.add_argument("--cargo", action="store_true",
                        help="reuse Cargo release compilation (requires the SDL2 environment)")
    parser.add_argument("--rustc", default=shutil.which("rustc") or str(
        Path.home() / ".cargo" / "bin" / "rustc.exe"))
    args = parser.parse_args()
    if os.name != "nt":
        parser.error("use cargo run --release -- verify on Linux")
    if args.frames is not None and not 1 <= args.frames <= 2**64 - 1:
        parser.error("--frames must be a positive u64")
    root = Path(__file__).resolve().parents[1]
    # Paths are resolved before changing cwd so invocation works from anywhere.
    rom, inputs = args.rom.resolve(), args.inputs.resolve()
    replay_args = [str(rom), str(inputs)]
    if args.frames is not None:
        replay_args.extend(["--frames", str(args.frames)])
    for probe in args.require_hit:
        replay_args.extend(["--require-hit", probe])
    for option, path in (("--checkpoint-in", args.checkpoint_in),
                         ("--checkpoint-out", args.checkpoint_out)):
        if path is not None:
            replay_args.extend([option, str(path.resolve())])
    dll = root / "target" / "verify-dll" / "sm_verify.dll"
    if not args.no_build:
        dll = build(root, args.rustc, args.cargo)
    os.chdir(root)
    # Keep SDK search directories alive throughout the native replay.
    dll_directories = []
    # MinGW SDKs place runtime DLLs beside lib/ in bin/.
    for path in os.environ.get("LIBRARY_PATH", "").split(os.pathsep):
        if path:
            for directory in (Path(path), Path(path).parent / "bin"):
                if directory.is_dir():
                    dll_directories.append(os.add_dll_directory(str(directory.resolve())))
    host = ctypes.CDLL(str(dll))
    if args.checkpoint_in is not None or args.checkpoint_out is not None or args.require_hit:
        if not hasattr(host, "sm_verify_args"):
            raise RuntimeError("cached DLL predates segment options; run without --no-build")
        host.sm_verify_args.argtypes = [ctypes.c_uint32, ctypes.POINTER(ctypes.c_char_p)]
        host.sm_verify_args.restype = ctypes.c_int
        argv = (ctypes.c_char_p * len(replay_args))(
            *(arg.encode("utf-8") for arg in replay_args))
        return host.sm_verify_args(len(replay_args), argv)
    host.sm_verify.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint64]
    host.sm_verify.restype = ctypes.c_int
    return host.sm_verify(str(rom).encode("utf-8"), str(inputs).encode("utf-8"),
                          args.frames or 0)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, ValueError, subprocess.CalledProcessError) as error:
        print(f"verify: {error}", file=sys.stderr)
        sys.exit(2)
