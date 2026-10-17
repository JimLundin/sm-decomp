#!/usr/bin/env python3
"""Build the Windows verifier DLL with rustc, then run one replay via ctypes."""
import argparse
import ctypes
import os
from pathlib import Path
import shutil
import subprocess
import sys


def build(root, rustc):
    output = root / "target" / "verify-dll"
    output.mkdir(parents=True, exist_ok=True)
    library = output / "libsm.rlib"
    dll = output / "sm_verify.dll"
    common = [rustc, "--edition=2021", "-C", "opt-level=3", "-A", "warnings"]
    subprocess.run(
        common + ["--crate-name", "sm", "--crate-type", "rlib", "--cfg",
                  "sm_verify_dll", "sm/src/lib.rs", "-o", str(library)],
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
    linker = sysroot / "lib" / "rustlib" / host / "bin" / "rust-lld.exe"
    subprocess.run(
        common + ["--crate-name", "sm_verify", "--crate-type", "cdylib",
                  "-Z", "unstable-options", "-C", f"linker={linker}",
                  "-C", "linker-flavor=gnu-lld", "sm/src/verify_dll.rs",
                  "--extern", f"sm={library}", "-o", str(dll)],
        cwd=root, check=True,
    )
    return dll


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("rom", type=Path)
    parser.add_argument("inputs", type=Path)
    parser.add_argument("--frames", type=int)
    parser.add_argument("--no-build", action="store_true",
                        help="reuse target/verify-dll/sm_verify.dll")
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
    dll = root / "target" / "verify-dll" / "sm_verify.dll"
    if not args.no_build:
        dll = build(root, args.rustc)
    os.chdir(root)
    host = ctypes.CDLL(str(dll))
    host.sm_verify.argtypes = [ctypes.c_char_p, ctypes.c_char_p, ctypes.c_uint64]
    host.sm_verify.restype = ctypes.c_int
    return host.sm_verify(str(rom).encode("utf-8"), str(inputs).encode("utf-8"),
                          args.frames or 0)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"verify: {error}", file=sys.stderr)
        sys.exit(2)
