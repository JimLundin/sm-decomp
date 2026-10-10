#!/usr/bin/env python3
"""Run the patched APU unit tests in a minimal isolated workspace.

This uses the source selected by the root lockfile without resolving the
unrelated RustySNES GUI/dev dependencies. No commercial ROM is required.
"""
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    with tempfile.TemporaryDirectory(prefix="sm-hardware-tests-") as directory:
        workspace = Path(directory)
        for name in ("rustysnes-apu", "rustysnes-savestate"):
            shutil.copytree(ROOT / "deps/rustysnes/crates" / name, workspace / name)
        (workspace / "Cargo.toml").write_text('''[workspace]
resolver = "3"
members = ["rustysnes-apu", "rustysnes-savestate"]
[workspace.package]
edition = "2024"
rust-version = "1.96"
license = "MIT OR Apache-2.0"
authors = []
[workspace.lints.rust]
unsafe_code = "forbid"
[workspace.lints.clippy]
all = "warn"
''')
        # External/path package identities change in the isolated workspace.
        # Retain every registry package's locked version and checksum.
        lock = (ROOT / "Cargo.lock").read_text()
        header, *packages = lock.split("[[package]]")
        (workspace / "Cargo.lock").write_text(header + "".join(
            "[[package]]" + package for package in packages if 'source = "registry+' in package or any(
                'name = "' + name + '"' in package for name in ("rustysnes-apu", "rustysnes-savestate"))))
        env = os.environ.copy()
        env["CARGO_TARGET_DIR"] = str(ROOT / "target/hardware-tests")
        subprocess.run(["cargo", "test", "--manifest-path", str(workspace / "Cargo.toml"),
                        "-p", "rustysnes-apu", "--lib", "--features", "clocked-smp", "--offline"],
                       cwd=ROOT, env=env, check=True)


if __name__ == "__main__":
    main()
