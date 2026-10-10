#!/usr/bin/env python3
"""Check local inputs and tools without network access or generated-code changes."""

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
failures = []


def check(label, ok):
    print("{}: {}".format("OK" if ok else "MISSING", label))
    if not ok:
        failures.append(label)


for tool in ("rustup", "cargo", "cc", "g++", "make", "pkg-config", "python3", "curl", "tar", "patch", "unzip", "dpkg-deb", "just"):
    check(tool, shutil.which(tool) is not None)
if shutil.which("rustup"):
    channel = re.search(r'^channel = "([^"]+)"',
                        (root / "rust-toolchain.toml").read_text(), re.MULTILINE).group(1)
    installed = subprocess.run(["rustup", "run", channel, "rustc", "--version"],
                               capture_output=True, text=True)
    check("Rust " + channel, installed.returncode == 0 and "nightly" in installed.stdout)
    if installed.returncode == 0:
        print("    " + installed.stdout.strip())
rom = root / "rom/sm.sfc"
check("unheadered Super Metroid ROM (expected SHA-1)", rom.exists() and
      hashlib.sha1(rom.read_bytes()).hexdigest() == "da957f0d63d14cb441d215462904c4fa8519c613")
check("RustySNES sources", (root / "deps/rustysnes/crates/rustysnes-core/Cargo.toml").exists())
from prepare_rustysnes import PATCHED
source = root / "deps/rustysnes/crates/rustysnes-core/src/dma.rs"
interrupt_records = json.loads((root / "tools/patches/rustysnes-clocked-interrupts.json").read_text())
final_hdma = next(record["patched_sha256"] for record in interrupt_records if record["path"] == "crates/rustysnes-core/src/dma.rs")
check("RustySNES HDMA timing patch", source.exists() and hashlib.sha256(source.read_bytes()).hexdigest() in (PATCHED, final_hdma))
records = [record for name in ("rustysnes-clocked-apu", "rustysnes-clocked-dma", "rustysnes-clocked-interrupts", "rustysnes-lsnes-reset")
           for record in json.loads((root / "tools/patches" / (name + ".json")).read_text())]
final_hashes = {record["path"]: record["patched_sha256"] for record in records}
for relative, expected in final_hashes.items():
    path = root / "deps/rustysnes" / relative
    check("hardware profile patch " + relative, path.exists() and
          hashlib.sha256(path.read_bytes()).hexdigest() == expected)
check("snesrev symbols", (root / "out/symbols.toml").exists())
corpus = list((root / "deps/65816/v1").glob("*.n.json"))
check("65816 corpus (256 native-mode opcode files)", len(corpus) == 256)
check("MesenCE binary", (root / "deps/mesen/Mesen").exists())
check("original lsnes recording core", (root / "deps/lsnes/source/src/util/lsnes-dumpavi.util").exists())
for movie in json.loads((root / "movies/manifest.json").read_text()):
    path = root / "movies" / movie["file"]
    check("TAS movie " + path.name, path.exists() and
          hashlib.sha256(path.read_bytes()).hexdigest() == movie["sha256"])
print("\nFallback comparison: just diff-fallback movies/super-metroid-any-3653.lsmv")
print("Compiled comparison: just iterate (small validated instruction whitelist).")
print("TAS capture smoke: just sync-smoke. Acceptance gate: just sync-check.")
print("TAS prefix verified through 18000 frames; full replay status: docs/LSNES_BASELINE.md.")
sys.exit(bool(failures))
