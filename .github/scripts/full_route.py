#!/usr/bin/env python3
"""Verify a full TAS route as a chain of original-emulator room seeds.

FULL_ROUTE selects the route: 100 (default, the 100% TAS and its
100-input-phase capture) or any (the any% TAS and its any-boundary capture).

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
  run JOB [ARGS...]      replay that job's rooms through verify_dll --corpus,
                         passing ARGS (e.g. --mode check-lock --lock F, which
                         replays only the rooms F locks)

Nothing ROM-derived is committed: the CSVs hold room ids, step counts, frame
numbers and SHA-256 digests only.
"""
import csv, hashlib, io, json, os, pathlib, re, subprocess, sys, tarfile

ROUTE = os.environ.get("FULL_ROUTE", "100")
REGIONS = ("wram", "sram", "vram", "cgram", "oam", "apuram")
# Per route: capture, committed room-seed index, pinned chain CSV, movie and its
# length, and the encrypted ci-data asset with the seeds and inputs (None until
# published).
ROUTES = {
    "100": dict(capture="100-input-phase", rooms="movies/room-seeds-100-input-phase.csv",
                chain="movies/full-route-chain.csv", movie="movies/super-metroid-100-4010.lsmv",
                frames=222788, asset="full-route-chain-v1-72e0f64e0d47b53d.tar.gz.enc",
                sha256="72e0f64e0d47b53d5110cb8c31e13571c14bc95684696aa8ce7f62c05a6f6dc1"),
    "any": dict(capture="any-boundary", rooms="movies/room-seeds-any-boundary.csv",
                chain="movies/full-route-chain-any.csv", movie="movies/super-metroid-any-3653.lsmv",
                frames=129712, asset="full-route-chain-any-v1-5d139284193bc614.tar.gz.enc",
                sha256="5d139284193bc61443855b0fa82128abf44e66f2ad03a7dd5360bbbbef582f03"),
}
_route = ROUTES[ROUTE]
CAPTURE = _route["capture"]
SEEDS = pathlib.Path("target/seeds") / CAPTURE
ROOMS = pathlib.Path(_route["rooms"])
CHAIN = pathlib.Path(_route["chain"])
# One generated recipe per job, so jobs can run side by side in one checkout.
RECIPE = "movies/full-route-chain-{route}-{job}.generated.toml"
MOVIE = pathlib.Path(_route["movie"])
MOVIE_FRAMES = _route["frames"]
BUNDLE_ASSET, BUNDLE_SHA256 = _route["asset"], _route["sha256"]
# The published routes share one matrix of about WAVES * PARALLEL jobs of
# similar step totals; full-route.yml runs PARALLEL of them at a time.
PARALLEL, WAVES = 8, 2
# Rooms that stay in lockstep but whose aligned inputs never leave the room;
# they are reported, not replayed, until the cause is known. Every other room
# must pass.
KNOWN_FAILURES = {}
# Rooms that once exposed sm_game bugs; they must keep passing.
REGRESSIONS = {} if ROUTE != "100" else {
    6: "Ceres Ridley's acceleration carry chain times the getaway as on hardware",
    246: "D387 out-of-room level word wraps like $94's long indexed read",
    311: "Tourian statue init passes $D7B9 and returned handles as $1993",
    312: "Tourian statue dust clouds pass $20 and returned handles as $1993",
    324: "the original sound-clear delay times the escape door as on hardware",
    328: "escape wall explosion passes A = 1 as $1993",
    329: "escape wall explosion passes A = 1 as $1993",
}
assert not set(KNOWN_FAILURES) & set(REGRESSIONS)

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / "tools"))


def rooms():
    with ROOMS.open(newline="") as stream:
        return list(csv.DictReader(stream))


def chain():
    with CHAIN.open(newline="") as stream:
        return list(csv.DictReader(stream))


def room_id(route, index, room):
    """A route room's corpus and lock scenario id."""
    return f"chain-{index:04d}-{room}" if route == "100" else f"chain-{route}-{index:04d}-{room}"


def input_name(index):
    return f"movies/chain-{ROUTE}-{index:04d}.inputs"


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
    """Pack this route's rooms into groups of similar step totals, heaviest group first.

    Every published route's groups target the same size, a WAVES * PARALLEL
    share of all their steps, so the combined matrix runs in WAVES waves.
    Each room replays from its own seed, so a group need not be contiguous:
    rooms go heaviest first into the lightest group.
    """
    entries = chain()
    total = 0
    for settings in ROUTES.values():
        if settings["asset"] is not None:
            with open(settings["chain"], newline="") as stream:
                total += sum(int(row["steps"]) for row in csv.DictReader(stream))
    target = total / (WAVES * PARALLEL)
    groups = [[] for _ in range(max(1, round(sum(int(e["steps"]) for e in entries) / target)))]
    for entry in sorted(entries, key=lambda entry: -int(entry["steps"])):
        min(groups, key=lambda group: sum(int(e["steps"]) for e in group)).append(entry)
    for group in groups:
        group.sort(key=lambda entry: int(entry["room_index"]))
    return sorted((group for group in groups if group), key=lambda group: -sum(int(e["steps"]) for e in group))


def plan():
    """One matrix over every published route; each job names its route and asset."""
    include = []
    for route, settings in ROUTES.items():
        if settings["asset"] is None:
            print(f"route {route}: chain bundle not published yet; skipped")
            continue
        planned = subprocess.run([sys.executable, __file__, "jobs"], env=dict(os.environ, FULL_ROUTE=route),
                                 check=True, capture_output=True, text=True).stdout
        include += [dict(job, route=route, asset=settings["asset"]) for job in json.loads(planned)]
    if not include:
        raise SystemExit("no chain bundle is published; see ROUTES in this script")
    # Start the heaviest jobs first across both routes.
    include.sort(key=lambda job: -job["steps"])
    with open(os.environ["GITHUB_OUTPUT"], "a") as out:
        out.write(f"matrix={json.dumps({'include': include})}\n")
    print(f"{len(include)} jobs: " + ", ".join(job["name"] for job in include))


def job_list():
    print(json.dumps([{"index": i, "name": f"{ROUTE}-{i:02d}-{len(group)}-rooms",
                       "steps": sum(int(entry["steps"]) for entry in group)} for i, group in enumerate(jobs())]))


def locked(extra):
    """The scenario ids an oracle digest lock passed as --lock F holds, if any."""
    if "--lock" not in extra:
        return None
    import tomllib
    path = pathlib.Path(extra[extra.index("--lock") + 1])
    if not path.exists():
        return set()
    with path.open("rb") as stream:
        return {entry["id"] for entry in tomllib.load(stream).get("scenario", [])}


def run(job, *extra):
    observation = {row["room_index"]: row["observation_sha256"] for row in rooms()}
    group = jobs()[int(job)]
    # Checking a lock replays only the rooms it holds; recording one, every room.
    lock = locked(extra) if "check-lock" in extra else None
    ids, lines = [], []
    for entry in group:
        index = int(entry["room_index"])
        name = room_id(ROUTE, index, entry["room"])
        if index in KNOWN_FAILURES:
            print(f"KNOWN {name}: {KNOWN_FAILURES[index]}", flush=True)
            continue
        if lock is not None and name not in lock:
            print(f"UNLOCKED {name}: not in the lock; stays on lockstep", flush=True)
            continue
        if index in REGRESSIONS:
            print(f"REGRESSION {name}: {REGRESSIONS[index]}", flush=True)
        ids.append(name)
        expect = f"room:{entry['next_room']}" if entry["ends"] == "next-room" else "state:8"
        lines += ["[[segment]]", f'id = "{name}"', f'seed = "{CAPTURE}:{index}"',
                  f'seed_sha256 = "{observation[entry["room_index"]]}"',
                  f'inputs = "{pathlib.Path(input_name(index)).name}"', f'input_sha256 = "{entry["input_sha256"]}"',
                  f"frames = {entry['steps']}", f'expect = ["{expect}"]', ""]
    header = ["# Generated by .github/scripts/full_route.py run; not committed.", "version = 1",
              "required = [" + ", ".join(f'"{name}"' for name in ids) + "]", ""]
    recipe = pathlib.Path(RECIPE.format(route=ROUTE, job=job))
    recipe.write_text("\n".join(header + lines))
    print(f"job {job}: rooms {', '.join(entry['room_index'] for entry in group)}, "
          f"{sum(int(entry['steps']) for entry in group)} steps", flush=True)
    subprocess.run([sys.executable, "tools/verify_dll.py", "rom/sm.smc", "--corpus", str(recipe),
                    "--no-build", "--direct", *extra], check=True)


if __name__ == "__main__":
    {"inputs": inputs, "pack": pack, "plan": plan, "jobs": job_list, "restore": restore, "run": run}[sys.argv[1]](*sys.argv[2:])
