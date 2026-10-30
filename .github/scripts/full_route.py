#!/usr/bin/env python3
"""Verify the full 100% route as a chain of original-emulator room seeds.

A replay of the movie from boot cannot follow the route: the verifier takes one
input per native controller read, and lag frames shift the movie (run
41047290647 passed lockstep while looping in Ceres). Instead every room of the
100% capture starts from its captured original state (target/seeds/
100-input-phase) with lag-aligned inputs that run into the next room, and must
reach it. A room whose capture contains a no-update wait (the item fanfare's
399 lag NMIs) stops at that wait: the original-emulator wait length differs
from the oracle's, so later inputs land early and the room never ends (72 of
79 local failures). Such a row only expects gameplay (state 8). Every frame compares WRAM, VRAM, CGRAM, OAM, PPU state and pixels
against the original, as corpus rows do.

Local (needs the ROM, the movie and the original capture with its lag logs):
  inputs                 write movies/chain-100-NNNN.inputs and the pinned
                         movies/full-route-chain.csv (steps, input SHA-256)
  pack BUNDLE            encrypt the seeds and inputs with the ROM-derived key
CI:
  plan                   write the job matrix to GITHUB_OUTPUT
  restore BUNDLE         restore the pinned bundle (ROM in rom/sm.smc)
  run JOB                replay that job's rooms through verify_dll --corpus

Nothing ROM-derived is committed: the CSVs hold room ids, step counts, frame
numbers and SHA-256 digests only.
"""
import csv, hashlib, io, json, os, pathlib, re, subprocess, sys, tarfile

CAPTURE = "100-input-phase"
SEEDS = pathlib.Path("target/seeds") / CAPTURE
ROOMS = pathlib.Path("movies/room-seeds-100-input-phase.csv")
CHAIN = pathlib.Path("movies/full-route-chain.csv")
RECIPE = pathlib.Path("movies/full-route-chain.generated.toml")
MOVIE = pathlib.Path("movies/super-metroid-100-4010.lsmv")
MOVIE_FRAMES = 222788
JOBS = 12
REGIONS = ("wram", "sram", "vram", "cgram", "oam", "apuram")
# Encrypted release asset (ci-data) holding the seeds and inputs; set when published.
BUNDLE_ASSET = None
BUNDLE_SHA256 = None

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "tools"))


def rooms():
    with ROOMS.open(newline="") as stream:
        return list(csv.DictReader(stream))


def chain():
    with CHAIN.open(newline="") as stream:
        return list(csv.DictReader(stream))


def input_name(index):
    return f"movies/chain-100-{index:04d}.inputs"


def inputs():
    import movie_inputs
    source = movie_inputs.load(MOVIE, pathlib.Path("rom/sm.smc"))
    with (SEEDS / "rooms.csv").open(newline="") as stream:
        files = {row["room_index"]: row["file"] for row in csv.DictReader(stream) if row["region"] == "wram"}
    with (SEEDS / "lag.csv.events.csv").open(newline="") as stream:
        waits = sorted(int(event["movie_frame"]) for event in csv.DictReader(stream)
                       if event["kind"] == "no_update_enter")
    entries, rows = [], rooms()
    for position, row in enumerate(rows):
        index = int(row["room_index"])
        following = rows[position + 1] if position + 1 < len(rows) else None
        # Steps never exceed movie frames, so the span to the next seed always
        # runs into the next room; the captured end of the movie bounds the last.
        start = int(row["movie_frame"])
        end = int(following["movie_frame"]) if following else MOVIE_FRAMES
        wait = next((frame for frame in waits if start < frame < end), None)
        ends = "no-update-wait" if wait else ("next-room" if following else "capture-end")
        count = (wait or end) - start
        while True:
            try:
                words = movie_inputs.seeded_wait_slice(source, SEEDS / "lag.csv", SEEDS / files[row["room_index"]], count)
                break
            except ValueError as error:
                if ends != "capture-end" or "capture ends" not in str(error) or count == 1:
                    raise
                count -= 1
        data = "".join(f"{word:04X}\r\n" for word in words).encode("ascii")
        pathlib.Path(input_name(index)).write_bytes(data)
        entries.append({"room_index": index, "room": row["room"], "next_room": following["room"] if following else "",
                        "ends": ends, "steps": len(words), "input_sha256": hashlib.sha256(data).hexdigest()})
        print(f"{index:4d} {row['room']} -> {entries[-1]['next_room'] or 'end'} ({ends}): {len(words)} steps", flush=True)
    with CHAIN.open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(entries[0]), lineterminator="\n")
        writer.writeheader()
        writer.writerows(entries)


def bundle_files():
    names = [f"{SEEDS.as_posix()}/rooms.csv"]
    for entry in chain():
        filename = f"{int(entry['room_index']):04d}-{entry['room']}-wram.bin"
        names += [f"{SEEDS.as_posix()}/{filename.replace('-wram.bin', f'-{region}.bin')}" for region in REGIONS]
        names += [f"{SEEDS.as_posix()}/{filename.replace('-wram.bin', '-registers.csv')}", input_name(int(entry["room_index"]))]
    return names


def cipher(decrypt, bundle):
    import argparse, shutil
    import ci_seed_data
    openssl = shutil.which("openssl") or str(pathlib.Path(os.environ.get("ProgramFiles", "C:/Program Files")) / "Git/mingw64/bin/openssl.exe")
    return ci_seed_data.cipher(argparse.Namespace(rom=pathlib.Path("rom/sm.smc"), openssl=openssl,
                                                  bundle=pathlib.Path(bundle)), decrypt=decrypt)


def pack(bundle):
    names = bundle_files()
    command, env = cipher(False, bundle)
    with open(bundle, "xb") as output:
        process = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=output, env=env)
        with tarfile.open(fileobj=process.stdin, mode="w|gz") as archive:
            for name in names:
                info = archive.gettarinfo(name, arcname=name)
                info.mode, info.uid, info.gid, info.uname, info.gname, info.mtime = 0o600, 0, 0, "", "", 0
                with open(name, "rb") as stream:
                    archive.addfile(info, stream)
        process.stdin.close()
        if process.wait():
            raise SystemExit("bundle encryption failed")
    with open(bundle, "rb") as stream:
        print(f"Encrypted {len(names)} files; ciphertext SHA-256 {hashlib.file_digest(stream, 'sha256').hexdigest()}")


def restore(bundle):
    import ci_seed_data
    with open(bundle, "rb") as stream:
        if not BUNDLE_SHA256 or hashlib.file_digest(stream, "sha256").hexdigest() != BUNDLE_SHA256:
            raise SystemExit("chain bundle SHA-256 mismatch")
    wanted = set(bundle_files())
    command, env = cipher(True, bundle)
    process = subprocess.Popen(command, stdout=subprocess.PIPE, env=env)
    restored = set()
    with tarfile.open(fileobj=process.stdout, mode="r|gz") as archive:
        for member in archive:
            if member.name not in wanted or not member.isfile() or not ci_seed_data.valid_name(member.name):
                raise SystemExit("unexpected chain bundle member")
            path = pathlib.Path(member.name)
            path.parent.mkdir(parents=True, exist_ok=True)
            with archive.extractfile(member) as source, path.open("xb") as output:
                output.write(source.read())
            restored.add(member.name)
    while process.stdout.read(1 << 20):
        pass
    if process.wait() or restored != wanted:
        raise SystemExit("chain bundle is incomplete or failed to decrypt")
    for entry in chain():
        if hashlib.sha256(pathlib.Path(input_name(int(entry["room_index"]))).read_bytes()).hexdigest() != entry["input_sha256"]:
            raise SystemExit(f"chain input {entry['room_index']} does not match its pin")
    print(f"Restored {len(restored)} chain files")


def jobs():
    """Split the chain into JOBS contiguous groups of similar step totals."""
    entries = chain()
    total = sum(int(entry["steps"]) for entry in entries)
    groups, current, steps = [], [], 0
    for entry in entries:
        current.append(entry)
        steps += int(entry["steps"])
        if steps >= total * (len(groups) + 1) / JOBS and len(groups) < JOBS - 1:
            groups.append(current)
            current = []
    return groups + ([current] if current else [])


def plan():
    if BUNDLE_ASSET is None:
        raise SystemExit("the chain bundle is not published yet; see BUNDLE_ASSET in this script")
    matrix = {"include": [{"index": i, "name": f"100-{i:02d}-rooms-{group[0]['room_index']}-{group[-1]['room_index']}",
                           "steps": sum(int(entry["steps"]) for entry in group)}
                          for i, group in enumerate(jobs())]}
    with open(os.environ["GITHUB_OUTPUT"], "a") as out:
        out.write(f"matrix={json.dumps(matrix)}\nasset={BUNDLE_ASSET}\n")
    print(f"{len(matrix['include'])} jobs over {len(chain())} rooms, "
          f"{sum(int(entry['steps']) for entry in chain())} steps")


def run(job):
    observation = {row["room_index"]: row["observation_sha256"] for row in rooms()}
    group = jobs()[int(job)]
    ids, lines = [], []
    for entry in group:
        index = int(entry["room_index"])
        name = f"chain-{index:04d}-{entry['room']}"
        ids.append(name)
        expect = f"room:{entry['next_room']}" if entry["ends"] == "next-room" else "state:8"
        lines += ["[[segment]]", f'id = "{name}"', f'seed = "{CAPTURE}:{index}"',
                  f'seed_sha256 = "{observation[entry["room_index"]]}"',
                  f'inputs = "{pathlib.Path(input_name(index)).name}"', f'input_sha256 = "{entry["input_sha256"]}"',
                  f"frames = {entry['steps']}", f'expect = ["{expect}"]', ""]
    header = ["# Generated by .github/scripts/full_route.py run; not committed.", "version = 1",
              "required = [" + ", ".join(f'"{name}"' for name in ids) + "]", ""]
    RECIPE.write_text("\n".join(header + lines))
    print(f"job {job}: rooms {group[0]['room_index']}..{group[-1]['room_index']}, "
          f"{sum(int(entry['steps']) for entry in group)} steps", flush=True)
    subprocess.run([sys.executable, "tools/verify_dll.py", "rom/sm.smc", "--corpus", str(RECIPE),
                    "--no-build", "--direct"], check=True)


if __name__ == "__main__":
    {"inputs": inputs, "pack": pack, "plan": plan, "restore": restore, "run": run}[sys.argv[1]](*sys.argv[2:])
