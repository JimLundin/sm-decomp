#!/usr/bin/env python3
"""Build and run the target packages' unit and integration tests instrumented.

Usage: coverage_tests.py OUT_DIR

Run with RUSTFLAGS carrying -C instrument-coverage. Builds every lib and
tests/*.rs test binary of sm_data, sm_game and sm_runtime, copies them to
OUT_DIR/bin (llvm-cov needs them as objects) and runs each as `cargo test`
would, from its package root, with LLVM_PROFILE_FILE under OUT_DIR/profiles.
OUT_DIR/objects.txt names the binaries that ran; one that Windows cannot
start is skipped with a warning. Doc-tests are not
instrumented and are left to CI's test step. Exits non-zero if a binary fails.

On Windows the executables link with Rust's bundled lld, as the verifier DLL
does. The GNU linker keeps only the first of the per-function coverage
records (.lcovfun), all of which share one section name, and places the
flagless coverage sections at address 0, below the image base, so the
executable is not a valid PE ("%1 is not a valid Win32 application").
"""
import json
import os
import pathlib
import shutil
import subprocess
import sys

PACKAGES = ["sm_data", "sm_game", "sm_runtime"]


def main():
    out = pathlib.Path(sys.argv[1]).resolve()
    (out / "bin").mkdir(parents=True, exist_ok=True)
    (out / "profiles").mkdir(parents=True, exist_ok=True)
    command = ["cargo", "test", "--locked", "--lib", "--tests", "--no-run", "--message-format=json",
               "--target-dir", "target/coverage-tests"]
    for package in PACKAGES:
        command += ["-p", package]
    build_environment = dict(os.environ)
    if os.name == "nt":
        host = next(line.split(": ", 1)[1] for line in subprocess.check_output(
            ["rustc", "-vV"], text=True).splitlines() if line.startswith("host: "))
        sysroot = pathlib.Path(subprocess.check_output(["rustc", "--print", "sysroot"], text=True).strip())
        linker = sysroot / "lib" / "rustlib" / host / "bin" / "gcc-ld" / "ld.lld.exe"
        build_environment[f"CARGO_TARGET_{host.upper().replace('-', '_')}_LINKER"] = str(linker)
        build_environment["RUSTFLAGS"] = " ".join(
            filter(None, [os.environ.get("RUSTFLAGS"), "-C linker-flavor=ld"]))
    built = subprocess.run(command, check=True, stdout=subprocess.PIPE, text=True, env=build_environment).stdout
    binaries = []
    for line in built.splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") == "compiler-artifact" and message.get("executable") and message["profile"]["test"]:
            source = pathlib.Path(message["executable"])
            target = out / "bin" / source.name
            shutil.copy2(source, target)
            package = pathlib.Path(message["manifest_path"]).parent
            binaries.append((message["target"]["name"], target, package))
    if not binaries:
        raise SystemExit("no test binaries were built")
    failed, ran = [], []
    for name, binary, package in binaries:
        environment = dict(os.environ, CARGO_MANIFEST_DIR=str(package),
                           LLVM_PROFILE_FILE=str(out / "profiles" / f"{name}-%p-%m.profraw"))
        print(f"== {name}", flush=True)
        try:
            result = subprocess.run([str(binary)], cwd=package, env=environment)
        except OSError as error:
            # A binary Windows cannot start is left out of the report.
            print(f"::warning::{binary.name} could not start: {error}", flush=True)
            continue
        ran.append(binary.name)
        if result.returncode != 0:
            failed.append(name)
    (out / "objects.txt").write_text("".join(f"{name}\n" for name in ran), encoding="utf-8")
    print(f"{len(binaries)} test binaries, {len(ran)} ran, {len(failed)} failed", flush=True)
    if failed:
        raise SystemExit("failed: " + ", ".join(failed))


if __name__ == "__main__":
    main()
