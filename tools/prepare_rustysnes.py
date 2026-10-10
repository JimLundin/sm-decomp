#!/usr/bin/env python3
"""Prepare pinned RustySNES source and documented hardware timing patches."""
import argparse
import json
import subprocess
import sys

from setup import ROOT, digest, prepare

ORIGINAL = "caf0b70ac78a39a6a2bfa1e5389ee904d18535cdf1ebf17eb1cde7b38214fe75"
PATCHED = "c854a6c72ed20c5863819befa551f4b2f74d67656602c2a1a979785f9250b718"


def apply_patch():
    source = ROOT / "deps/rustysnes"
    target = source / "crates/rustysnes-core/src/dma.rs"
    identity = digest(target)
    interrupt_records = json.loads((ROOT / "tools/patches/rustysnes-clocked-interrupts.json").read_text())
    final = next(record["patched_sha256"] for record in interrupt_records if record["path"] == "crates/rustysnes-core/src/dma.rs")
    if identity in (PATCHED, final):
        print("Ready: RustySNES HDMA channel timing patch")
        return
    if identity != ORIGINAL:
        raise RuntimeError("RustySNES dma.rs differs from the pinned original and patch; preserve and review local edits")
    subprocess.run(["patch", "--batch", "-p1", "-i", str(ROOT / "tools/patches/rustysnes-hdma.patch")],
                   cwd=source, check=True)
    if digest(target) != PATCHED:
        raise RuntimeError("RustySNES HDMA patch checksum mismatch")


def apply_clocked_apu_patch():
    source = ROOT / "deps/rustysnes"
    records = json.loads((ROOT / "tools/patches/rustysnes-clocked-apu.json").read_text())
    identities = [digest(source / record["path"]) if (source / record["path"]).exists() else None
                  for record in records]
    dma = {record["path"]: record for record in
           json.loads((ROOT / "tools/patches/rustysnes-clocked-dma.json").read_text())}
    interrupts = {record["path"]: record for record in
                  json.loads((ROOT / "tools/patches/rustysnes-clocked-interrupts.json").read_text())}
    if all(identity == record["patched_sha256"] or
           (record["path"] in dma and identity == dma[record["path"]]["patched_sha256"]) or
           (record["path"] in interrupts and identity == interrupts[record["path"]]["patched_sha256"])
           for identity, record in zip(identities, records)):
        print("Ready: RustySNES clocked APU bus patch")
        return
    if identities != [record["original_sha256"] for record in records]:
        raise RuntimeError("RustySNES APU patch targets differ from pinned source; preserve and review local edits")
    subprocess.run(["patch", "--batch", "-p1", "-i", str(ROOT / "tools/patches/rustysnes-clocked-apu.patch")],
                   cwd=source, check=True)
    for record in records:
        if digest(source / record["path"]) != record["patched_sha256"]:
            raise RuntimeError("RustySNES clocked APU patch checksum mismatch")


def apply_clocked_dma_patch():
    source = ROOT / "deps/rustysnes"
    records = json.loads((ROOT / "tools/patches/rustysnes-clocked-dma.json").read_text())
    identities = [digest(source / record["path"]) for record in records]
    interrupts = {record["path"]: record for record in
                  json.loads((ROOT / "tools/patches/rustysnes-clocked-interrupts.json").read_text())}
    if all(identity == record["patched_sha256"] or
           (record["path"] in interrupts and identity == interrupts[record["path"]]["patched_sha256"])
           for identity, record in zip(identities, records)):
        print("Ready: RustySNES clocked DMA alignment patch")
        return
    if identities != [record["original_sha256"] for record in records]:
        raise RuntimeError("RustySNES DMA patch targets differ from pinned source; preserve and review local edits")
    subprocess.run(["patch", "--batch", "-p1", "-i", str(ROOT / "tools/patches/rustysnes-clocked-dma.patch")],
                   cwd=source, check=True)
    for record in records:
        if digest(source / record["path"]) != record["patched_sha256"]:
            raise RuntimeError("RustySNES clocked DMA patch checksum mismatch")


def apply_clocked_interrupts_patch():
    source = ROOT / "deps/rustysnes"
    records = json.loads((ROOT / "tools/patches/rustysnes-clocked-interrupts.json").read_text())
    identities = [digest(source / record["path"]) for record in records]
    if identities == [record["patched_sha256"] for record in records]:
        print("Ready: RustySNES clocked interrupt sampling patch")
        return
    if identities != [record["original_sha256"] for record in records]:
        raise RuntimeError("RustySNES interrupt patch targets differ from pinned source; preserve and review local edits")
    subprocess.run(["patch", "--batch", "-p1", "-i", str(ROOT / "tools/patches/rustysnes-clocked-interrupts.patch")],
                   cwd=source, check=True)
    for record in records:
        if digest(source / record["path"]) != record["patched_sha256"]:
            raise RuntimeError("RustySNES clocked interrupt patch checksum mismatch")


def apply_lsnes_reset_patch():
    source = ROOT / "deps/rustysnes"
    records = json.loads((ROOT / "tools/patches/rustysnes-lsnes-reset.json").read_text())
    identities = [digest(source / record["path"]) for record in records]
    if identities == [record["patched_sha256"] for record in records]:
        print("Ready: lsnes recording-core reset phase patch")
        return
    if identities != [record["original_sha256"] for record in records]:
        raise RuntimeError("RustySNES reset patch target differs from pinned source; preserve and review local edits")
    subprocess.run(["patch", "--batch", "-p1", "-i", str(ROOT / "tools/patches/rustysnes-lsnes-reset.patch")],
                   cwd=source, check=True)
    for record in records:
        if digest(source / record["path"]) != record["patched_sha256"]:
            raise RuntimeError("RustySNES recording-core reset patch checksum mismatch")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true")
    args = parser.parse_args()
    manifest = json.loads((ROOT / "tools/dependencies.json").read_text())
    dependency = next(record for record in manifest if record["name"] == "rustysnes")
    prepare(dependency, args.offline)
    apply_patch()
    apply_clocked_apu_patch()
    apply_clocked_dma_patch()
    apply_clocked_interrupts_patch()
    apply_lsnes_reset_patch()


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print("prepare-rustysnes: {}".format(error), file=sys.stderr)
        sys.exit(1)
