#!/usr/bin/env python3
"""Plan, prepare and run the full 100% route in lockstep against the original.

  plan                         write the chunk matrix to GITHUB_OUTPUT
  prepare                      restore the ROM, fetch the pinned 100% movie, write
                               movies/100.inputs (nothing ROM-derived is printed)
  checkpoints OUT_DIR          replay from boot, saving a checkpoint at every
                               chunk boundary (each produced by the original
                               from frame 0, so chunks never start from an
                               unverified state)
  chunk INDEX CHECKPOINT_DIR   replay one chunk from its boundary checkpoint

Every frame is compared exactly as in the corpus: WRAM, VRAM, CGRAM, OAM, PPU
state and rendered pixels.
"""
import base64, gzip, hashlib, io, json, os, pathlib, subprocess, sys, tomllib, urllib.request, zipfile

CHUNK_FRAMES = 20000  # ~60 fps lockstep on hosted runners: about 6 minutes per chunk.
INPUTS = pathlib.Path("movies/100.inputs")
INPUTS_SHA256 = "80aeef2665bac311"  # prefix of the canonical CRLF import; checked in prepare


def movie():
    movies = tomllib.loads(pathlib.Path("movies/manifest.toml").read_text())["movies"]
    return next(m for m in movies if m["category"] == "100%")


def chunks():
    total = movie()["frames"]
    return [(start, min(CHUNK_FRAMES, total - start)) for start in range(0, total, CHUNK_FRAMES)]


def plan():
    matrix = {"include": [{"index": i, "start": s, "frames": n, "name": f"100-{i:02d}-{s}"}
                          for i, (s, n) in enumerate(chunks())]}
    with open(os.environ["GITHUB_OUTPUT"], "a") as out:
        out.write(f"matrix={json.dumps(matrix)}\n")
    print(f"{len(matrix['include'])} chunks of up to {CHUNK_FRAMES} frames, {movie()['frames']} total")


def prepare():
    encoded = "".join(os.environ[f"ROM_{i:02d}"] for i in range(45))
    if not encoded:
        raise SystemExit("ROM secrets are missing")
    rom = gzip.decompress(base64.b64decode(encoded))
    if hashlib.sha1(rom).hexdigest() != "da957f0d63d14cb441d215462904c4fa8519c613":
        raise SystemExit("wrong ROM")
    pathlib.Path("rom").mkdir(exist_ok=True)
    pathlib.Path("rom/sm.smc").write_bytes(rom)
    entry = movie()
    path = pathlib.Path("movies", entry["file"])
    if path.exists():
        data = path.read_bytes()
    else:
        request = urllib.request.Request(entry["download"], headers={"User-Agent": "sm-decomp-ci"})
        with urllib.request.urlopen(request, timeout=300) as response:
            data = response.read()
        if hashlib.sha256(data).hexdigest() != entry["sha256"]:
            with zipfile.ZipFile(io.BytesIO(data)) as archive:
                data = archive.read(next(n for n in archive.namelist() if n.lower().endswith(".lsmv")))
    if hashlib.sha256(data).hexdigest() != entry["sha256"]:
        raise SystemExit("wrong 100% movie")
    path.write_bytes(data)
    subprocess.run([sys.executable, "tools/movie_inputs.py", str(path), str(INPUTS), "--rom", "rom/sm.smc"], check=True)
    canonical = INPUTS.read_bytes().replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    INPUTS.write_bytes(canonical)
    if not hashlib.sha256(canonical).hexdigest().startswith(INPUTS_SHA256):
        raise SystemExit("100% input import changed")


def verify(*extra):
    command = [sys.executable, "tools/verify_dll.py", "rom/sm.smc", str(INPUTS), "--no-build", "--direct", *extra]
    print("verify_dll", " ".join(extra), flush=True)
    subprocess.run(command, check=True)


def checkpoints(out_dir):
    out = pathlib.Path(out_dir)
    out.mkdir(parents=True, exist_ok=True)
    previous = None
    for index, (start, frames) in enumerate(chunks()[:-1]):
        target = out / f"100-{index + 1:02d}.smvchk"
        extra = ["--frames", str(frames), "--checkpoint-out", str(target)]
        if previous:
            extra[:0] = ["--checkpoint-in", str(previous)]
        verify(*extra)
        previous = target


def chunk(index, checkpoint_dir):
    index = int(index)
    start, frames = chunks()[index]
    extra = ["--frames", str(frames)]
    if index:
        extra = ["--checkpoint-in", str(pathlib.Path(checkpoint_dir) / f"100-{index:02d}.smvchk"), *extra]
    print(f"chunk {index}: frames {start}..{start + frames}", flush=True)
    verify(*extra)


if __name__ == "__main__":
    {"plan": plan, "prepare": prepare, "checkpoints": checkpoints, "chunk": chunk}[sys.argv[1]](*sys.argv[2:])
