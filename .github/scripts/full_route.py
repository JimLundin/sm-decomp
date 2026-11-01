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

Each room is a scenario in scenarios/route-R.toml; scenarios/lock.toml pins
its inputs and seed and the ci-data bundle that holds them (tools/scenario.py).

Local (needs the ROM, the movie and the original capture with its lag logs):
  inputs                 write movies/chain-100-NNNN.inputs, the route's
                         scenarios (steps, next room) and their pins
  pack BUNDLE            encrypt the seeds and inputs with the ROM-derived key
                         and pin the bundle
CI:
  plan                   write the job matrix to GITHUB_OUTPUT
  restore BUNDLE         restore the pinned bundle (ROM in rom/sm.smc)
  run JOB [ARGS...]      replay that job's rooms through verify_dll --corpus,
                         passing ARGS (e.g. --mode check-lock --lock F, which
                         replays only the rooms F locks)

Nothing ROM-derived is committed: the scenarios and the room-seed CSVs hold
room ids, step counts, frame numbers and SHA-256 digests only.
"""
import csv, hashlib, io, json, os, pathlib, re, subprocess, sys, tarfile

ROUTE = os.environ.get("FULL_ROUTE", "100")
REGIONS = ("wram", "sram", "vram", "cgram", "oam", "apuram")
# Per route: capture, committed room-seed index, movie and its length.
ROUTES = {
    "100": dict(capture="100-input-phase", rooms="movies/room-seeds-100-input-phase.csv",
                movie="movies/super-metroid-100-4010.lsmv", frames=222788),
    "any": dict(capture="any-boundary", rooms="movies/room-seeds-any-boundary.csv",
                movie="movies/super-metroid-any-3653.lsmv", frames=129712),
}
_route = ROUTES[ROUTE]
CAPTURE = _route["capture"]
SEEDS = pathlib.Path("target/seeds") / CAPTURE
ROOMS = pathlib.Path(_route["rooms"])
MOVIE = pathlib.Path(_route["movie"])
MOVIE_FRAMES = _route["frames"]
# Keep sixteen balanced jobs across the published routes. The workflow
# independently bounds how many jobs run at once.
TARGET_JOBS = 16

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import scenario  # noqa: E402


def rooms():
    with ROOMS.open(newline="") as stream:
        return list(csv.DictReader(stream))


def chain(route=ROUTE):
    """The route's rooms in order, from their scenarios and pins."""
    return [dict(id=row["id"], room_index=row["seed"].split(":")[1], room=row["id"].rsplit("-", 1)[1],
                 steps=row["frames"], input_sha256=row["input_sha256"], bundle=row.get("bundle"),
                 pending=row.get("pending"), regression=row.get("regression"))
            for row in scenario.rows(ROOT) if row.get("chain") == route]


def published(route=ROUTE):
    """The route's ci-data asset and its pinned SHA-256; None until published."""
    names = {entry["bundle"] for entry in chain(route)}
    if len(names) != 1 or None in names:
        return None, None
    name = names.pop()
    return name, scenario.read_lock(ROOT).bundles[name]


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
    notes, lock = scenario.load(ROOT), scenario.read_lock(ROOT)
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
        name = room_id(ROUTE, index, row["room"])
        # A room that runs into the next must reach it; one that stops at a wait only expects gameplay.
        expect = f"room:{following['room']}" if ends == "next-room" else "state:8"
        entries.append({key: value for key, value in notes.get(name, {}).items()
                        if key in ("pending", "unlocked", "regression")}
                       | {"id": name, "start": f"seed:{row['observation_sha256']}", "ticks": len(words),
                          "facts": [expect]})
        pin = lock.pins.setdefault(name, {"id": name}) | {"seed": f"{CAPTURE}:{index}"}
        lock.pins[name] = pin | {"inputs": pathlib.Path(input_name(index)).name,
                                 "input_sha256": hashlib.sha256(data).hexdigest()}
        print(f"{index:4d} {row['room']} -> {expect} ({ends}): {len(words)} steps", flush=True)
    scenario.write_route(ROOT, ROUTE, entries)
    scenario.write_lock(ROOT / scenario.LOCK, lock)


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
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    lock = scenario.read_lock(ROOT)
    for entry in chain():
        lock.pins[entry["id"]]["bundle"] = pathlib.Path(bundle).name
    lock.bundles[pathlib.Path(bundle).name] = digest
    lock.bundles = {name: sha for name, sha in lock.bundles.items()
                    if any(pin.get("bundle") == name for pin in lock.pins.values())}
    scenario.write_lock(ROOT / scenario.LOCK, lock)
    print(f"Encrypted {len(names)} files; ciphertext SHA-256 {digest}; pinned in {scenario.LOCK}")


def restore(bundle):
    import ci_seed_data
    pinned = published()[1]
    with open(bundle, "rb") as stream:
        if not pinned or hashlib.file_digest(stream, "sha256").hexdigest() != pinned:
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

    Every published route's groups target one of TARGET_JOBS equal shares
    of all their steps; concurrency is bounded by the workflow.
    Each room replays from its own seed, so a group need not be contiguous:
    rooms go heaviest first into the lightest group.
    """
    entries = chain()
    total = sum(int(entry["steps"]) for route in ROUTES if published(route)[0] for entry in chain(route))
    target = total / TARGET_JOBS
    groups = [[] for _ in range(max(1, round(sum(int(e["steps"]) for e in entries) / target)))]
    for entry in sorted(entries, key=lambda entry: -int(entry["steps"])):
        min(groups, key=lambda group: sum(int(e["steps"]) for e in group)).append(entry)
    for group in groups:
        group.sort(key=lambda entry: int(entry["room_index"]))
    return sorted((group for group in groups if group), key=lambda group: -sum(int(e["steps"]) for e in group))


def plan():
    """One matrix over every published route; each job names its route and asset."""
    include = []
    for route in ROUTES:
        asset = published(route)[0]
        if asset is None:
            print(f"route {route}: chain bundle not published yet; skipped")
            continue
        planned = subprocess.run([sys.executable, __file__, "jobs"], env=dict(os.environ, FULL_ROUTE=route),
                                 check=True, capture_output=True, text=True).stdout
        include += [dict(job, route=route, asset=asset) for job in json.loads(planned)]
    if not include:
        raise SystemExit(f"no chain bundle is published; see the route pins in {scenario.LOCK}")
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
    return set(scenario.read_lock(ROOT, pathlib.Path(extra[extra.index("--lock") + 1])).entries)


def run(job, *extra):
    group = jobs()[int(job)]
    # Checking a lock replays only the rooms it holds; recording one, every room.
    lock = locked(extra) if "check-lock" in extra else None
    ids = []
    for entry in group:
        name = entry["id"]
        if entry["pending"]:
            print(f"KNOWN {name}: {entry['pending']}", flush=True)
            continue
        if lock is not None and name not in lock:
            print(f"UNLOCKED {name}: not in the lock; stays on lockstep", flush=True)
            continue
        if entry["regression"]:
            print(f"REGRESSION {name}: {entry['regression']}", flush=True)
        ids.append(name)
    print(f"job {job}: rooms {', '.join(entry['room_index'] for entry in group)}, "
          f"{sum(int(entry['steps']) for entry in group)} steps", flush=True)
    if ids:
        subprocess.run([sys.executable, "tools/verify_dll.py", "rom/sm.smc", "--corpus", "scenarios",
                        "--no-build", "--direct", *[part for name in ids for part in ("--segment", name)],
                        *extra], check=True)


if __name__ == "__main__":
    {"inputs": inputs, "pack": pack, "plan": plan, "jobs": job_list, "restore": restore, "run": run}[sys.argv[1]](*sys.argv[2:])
