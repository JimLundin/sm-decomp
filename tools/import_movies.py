#!/usr/bin/env python3
"""Import the verified local TAS corpus with the Rust movie tool."""
import hashlib
import json
import subprocess
from pathlib import Path

root = Path(__file__).resolve().parent.parent
movies = root / "movies"
manifest = json.loads((movies / "manifest.json").read_text())
subprocess.run(["cargo", "build", "-p", "movie", "--locked", "--offline"], cwd=root, check=True)
for entry in manifest:
    source = movies / entry["file"]
    if hashlib.sha256(source.read_bytes()).hexdigest() != entry["sha256"]:
        raise SystemExit("movie checksum mismatch: {} (run just fetch-movies)".format(source))
    output = source.with_suffix(".inputs")
    subprocess.run([root / "target/debug/movie", source, output], cwd=root, check=True)
    if len(output.read_text().splitlines()) != entry["frames"]:
        raise SystemExit("frame count mismatch: {}".format(output))
