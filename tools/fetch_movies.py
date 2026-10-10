#!/usr/bin/env python3
"""Fetch published TAS inputs, unwrap TASVideos's ZIP, and verify movie/ROM identity."""

import argparse
import hashlib
import io
import json
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true", help="only verify existing movie files")
    args = parser.parse_args()
    manifest = json.loads((ROOT / "movies/manifest.json").read_text())
    rom = ROOT / "rom/sm.sfc"
    rom_hash = hashlib.sha256(rom.read_bytes()).hexdigest() if rom.exists() else None
    for entry in manifest:
        destination = ROOT / "movies" / entry["file"]
        if not destination.exists():
            if args.offline:
                raise RuntimeError("missing movie: " + entry["file"])
            downloads = ROOT / "deps/downloads/tas"
            downloads.mkdir(parents=True, exist_ok=True)
            archive = downloads / (entry["file"] + ".zip")
            subprocess.run(["curl", "-L", "--fail", "--silent", "--show-error", "--retry", "3",
                            "--connect-timeout", "30", entry["download"], "-o", str(archive)], check=True)
            with zipfile.ZipFile(archive) as outer:
                names = [name for name in outer.namelist() if name.endswith(".lsmv")]
                if len(names) != 1:
                    raise RuntimeError("expected one LSMV in TASVideos download")
                data = outer.read(names[0])
        else:
            data = destination.read_bytes()
        if hashlib.sha256(data).hexdigest() != entry["sha256"]:
            raise RuntimeError("movie checksum mismatch: " + entry["file"])
        with zipfile.ZipFile(io.BytesIO(data)) as movie:
            expected_rom = movie.read("rom.sha256").decode().strip()
            if expected_rom != entry["rom_sha256"] or (rom_hash is not None and rom_hash != expected_rom):
                raise RuntimeError("ROM checksum mismatch: " + entry["file"])
        if not destination.exists():
            destination.write_bytes(data)
        print("Ready: {} ({} frames; {})".format(entry["file"], entry["frames"], entry["category"]))
    print("Run just import-movies, then just diff-fallback; see docs/TAS_MOVIES.md for synchronization limits.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.CalledProcessError, zipfile.BadZipFile, KeyError) as error:
        print("movies: {}".format(error), file=sys.stderr)
        sys.exit(1)
