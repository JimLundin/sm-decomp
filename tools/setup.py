#!/usr/bin/env python3
"""Download pinned dependencies and build the workspace and reference runner."""

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEPS = ROOT / "deps"


def run(*args):
    subprocess.run([str(arg) for arg in args], cwd=ROOT, check=True)


def digest(path):
    result = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            result.update(block)
    return result.hexdigest()


def prepare(dep, offline):
    archive = DEPS / "downloads" / dep["archive"]
    archive.parent.mkdir(parents=True, exist_ok=True)
    if not archive.exists():
        if offline:
            raise RuntimeError("missing cached download: {}".format(archive))
        partial = archive.with_suffix(archive.suffix + ".part")
        print("Downloading {} ({})".format(dep["name"], dep["revision"]), flush=True)
        run("curl", "-L", "--fail", "--silent", "--show-error", "--retry", "3",
            "--connect-timeout", "30", dep["url"], "-o", partial)
        if digest(partial) != dep["sha256"]:
            raise RuntimeError("checksum mismatch: {}".format(partial))
        partial.replace(archive)
    if digest(archive) != dep["sha256"]:
        raise RuntimeError("checksum mismatch: {}".format(archive))

    kind = dep["kind"]
    destination = DEPS / ("mesen-runtime" if kind == "runtime-deb" else dep["name"])
    stamp = destination / (".setup-" + dep["name"])
    if stamp.exists() and stamp.read_text().strip() == dep["sha256"]:
        print("Ready: {}".format(dep["name"]), flush=True)
        return
    destination.mkdir(parents=True, exist_ok=True)
    if kind in ("source", "native-tests"):
        command = ["tar", "-xzf", archive, "--strip-components=1", "-C", destination]
        if kind == "native-tests":
            command += ["--wildcards", "*/README.md", "*/v1/*.n.json"]
        run(*command)
    elif kind == "zip":
        run("unzip", "-qo", archive, "-d", destination)
    else:
        run("dpkg-deb", "-x", archive, destination)
    stamp.write_text(dep["sha256"] + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true", help="use cached downloads and Cargo crates")
    parser.add_argument("--skip-mesen", action="store_true", help="prepare only the Rust tools, sources and corpus")
    args = parser.parse_args()
    required = ["rustup", "cargo", "cc", "curl", "tar", "patch"]
    if not args.skip_mesen:
        if (platform.system(), platform.machine()) != ("Linux", "x86_64"):
            parser.error("the Mesen runtime is Linux x64; use --skip-mesen on other hosts")
        required += ["unzip", "dpkg-deb"]
    missing = [tool for tool in required if shutil.which(tool) is None]
    if missing:
        raise RuntimeError("install required tools first: " + ", ".join(missing))
    # Cargo selects the exact toolchain from rust-toolchain.toml.
    run("cargo", "--version")
    dependencies = json.loads((ROOT / "tools/dependencies.json").read_text())
    for dep in dependencies:
        if args.skip_mesen and dep["kind"] not in ("source", "native-tests"):
            continue
        prepare(dep, args.offline)
        if dep["name"] == "rustysnes":
            from prepare_rustysnes import apply_patch, apply_clocked_apu_patch, apply_clocked_dma_patch, apply_clocked_interrupts_patch, apply_lsnes_reset_patch, apply_readiness_observation_patch
            apply_patch()
            apply_clocked_apu_patch()
            apply_clocked_dma_patch()
            apply_clocked_interrupts_patch()
            apply_lsnes_reset_patch()
            apply_readiness_observation_patch()
    if not args.skip_mesen:
        libraries = DEPS / "mesen-runtime/lib/x86_64-linux-gnu"
        run(DEPS / "patchelf/usr/bin/patchelf", "--set-interpreter",
            libraries / "ld-linux-x86-64.so.2", "--set-rpath", libraries,
            DEPS / "mesen/Mesen")
        (DEPS / "mesen/Mesen").chmod(0o755)
    run(sys.executable, ROOT / "tools/symbols.py")
    flags = ["--locked"]
    if args.offline:
        flags += ["--offline"]
    run("cargo", "fetch", *flags)
    run("cargo", "build", "--workspace", *flags)
    print("Setup complete. Run `just doctor`, `just check`, and `just reference-smoke`.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print("setup: {}".format(error), file=sys.stderr)
        sys.exit(1)
