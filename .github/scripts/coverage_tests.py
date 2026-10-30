#!/usr/bin/env python3
"""Build and run the target packages' unit and integration tests instrumented.

Usage: coverage_tests.py OUT_DIR

Run with RUSTFLAGS carrying -C instrument-coverage. Builds every lib and
tests/*.rs test binary of sm_data, sm_game and sm_runtime, copies them to
OUT_DIR/bin (llvm-cov needs them as objects) and runs each as `cargo test`
would, from its package root, with LLVM_PROFILE_FILE under OUT_DIR/profiles.
OUT_DIR/objects.txt names the copied binaries. Doc-tests are not
instrumented and are left to CI's test step. Exits non-zero if a binary fails.
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
    built = subprocess.run(command, check=True, stdout=subprocess.PIPE, text=True).stdout
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
    failed = []
    for name, binary, package in binaries:
        environment = dict(os.environ, CARGO_MANIFEST_DIR=str(package),
                           LLVM_PROFILE_FILE=str(out / "profiles" / f"{name}-%p-%m.profraw"))
        print(f"== {name}", flush=True)
        if subprocess.run([str(binary)], cwd=package, env=environment).returncode != 0:
            failed.append(name)
    names = [binary.name for _, binary, _ in binaries]
    (out / "objects.txt").write_text("".join(f"{name}\n" for name in names), encoding="utf-8")
    print(f"{len(binaries)} test binaries, {len(failed)} failed", flush=True)
    if failed:
        raise SystemExit("failed: " + ", ".join(failed))


if __name__ == "__main__":
    main()
