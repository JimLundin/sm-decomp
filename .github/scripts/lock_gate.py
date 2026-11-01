#!/usr/bin/env python3
"""Plan the engine-only gate against the oracle digest lock.

Every pull request and push runs the engine alone over every scenario that
scenarios/lock.toml locks and compares its running digests with the faithful
oracle's (docs/SCENARIO_HARNESS.md, Decision 4; lock-gate.yml). Each seeded or
boot corpus row and each room of both routes is either locked or listed in
UNLOCKED with the LEGACY entry that keeps it on lockstep. The plan fails on a
stale lock (an entry whose start, inputs or budget no longer match its
scenario, an entry for no scenario, or a scenario neither locked nor listed);
it never re-records.

  check   check that the lock is current
  plan    check it and write the job matrix to GITHUB_OUTPUT

Recording (faithful oracle; docs/SCENARIO_HARNESS.md, "Oracle digest lock"):
  python tools/verify_dll.py rom/sm.smc --corpus movies/segments.toml \\
      --mode lock --oracle faithful --lock scenarios/lock.toml --segment ID...
  FULL_ROUTE=R python .github/scripts/full_route.py run JOB \\
      --mode lock --oracle faithful --lock scenarios/lock.toml
"""
import csv, json, os, pathlib, subprocess, sys, tempfile, tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
LOCK = ROOT / "scenarios/lock.toml"
sys.path.insert(0, str(ROOT / ".github/scripts"))
import full_route  # noqa: E402

# Scenarios that stay on patched lockstep, each with its docs/LEGACY.md entry
# ("Faithful-oracle known divergences"). The list only shrinks.
FAITHFUL = "a faithful-oracle known divergence"
ITEM_DRIFT = "WRAM drift lockstep masks, in the item and pickup rows (not yet identified)"
DRIFT = "drift lockstep masks (not yet identified)"
ENGINE_PANIC = "the engine alone panics; lockstep adopts the oracle's state first"
UNLOCKED = {
    "samus-scripted-intro": FAITHFUL,
    "pause-menu-tour": FAITHFUL,
    "boss-kraid-pause-400": FAITHFUL,
    "boss-kraid-pause-950": FAITHFUL,
    "boss-kraid-pause-1030": FAITHFUL,
    "enemy-fire-arc": FAITHFUL,
    "chain-0011-DF45": FAITHFUL,
    "chain-0211-C98E": FAITHFUL,
    "chain-any-0011-DF45": FAITHFUL,
    "item-bombs": ITEM_DRIFT,
    "item-grapple": ITEM_DRIFT,
    "item-gravity": ITEM_DRIFT,
    "item-hi-jump": ITEM_DRIFT,
    "item-ice": ITEM_DRIFT,
    "item-morph-ball": ITEM_DRIFT,
    "item-plasma": ITEM_DRIFT,
    "item-screw-attack": ITEM_DRIFT,
    "item-space-jump": ITEM_DRIFT,
    "item-spazer": ITEM_DRIFT,
    "item-speed-booster": ITEM_DRIFT,
    "item-spring-ball": ITEM_DRIFT,
    "item-varia": ITEM_DRIFT,
    "item-x-ray": ITEM_DRIFT,
    "pickup-energy": ITEM_DRIFT,
    "pickup-missile": ITEM_DRIFT,
    "pickup-power-bomb": ITEM_DRIFT,
    "pickup-super": ITEM_DRIFT,
    "samus-grapple-connect-swing": ITEM_DRIFT,
    "space-jump-bubble-word-counter": ITEM_DRIFT,
    "xray-activate-release": ITEM_DRIFT,
    "pickup-reserve": ITEM_DRIFT,
    "boss-kraid": DRIFT,
    "boss-kraid-pause-1960": DRIFT,
    "chain-0330-92FD": DRIFT,
    "chain-any-0079-ACF0": DRIFT,
    "chain-any-0196-92FD": DRIFT,
    "chain-0086-AA41": DRIFT,
    "chain-any-0074-AFA3": DRIFT,
    "boss-crocomire": DRIFT,
    "ice-return-business": DRIFT,
    "music-fir-selector10": DRIFT,
    "file-copy-clear": ENGINE_PANIC,
}


def scenarios():
    """Every lockable corpus row's and route room's id, with the key its lock entry carries."""
    corpus, rooms = {}, {}
    with (ROOT / "movies/segments.toml").open("rb") as stream:
        for row in tomllib.load(stream)["segment"]:
            # Checkpoint starts need an oracle-side prefix first; pending rows have no data.
            if "pending" in row or row.get("start_frame", 0) or "checkpoint" in row:
                continue
            start = f"seed:{row['seed_sha256']}" if "seed" in row else "boot"
            corpus[row["id"]] = dict(start=start, inputs=row["input_sha256"], frames=row["frames"])
    for route, settings in full_route.ROUTES.items():
        with (ROOT / settings["rooms"]).open(newline="") as stream:
            seeds = {row["room_index"]: row["observation_sha256"] for row in csv.DictReader(stream)}
        with (ROOT / settings["chain"]).open(newline="") as stream:
            for entry in csv.DictReader(stream):
                rooms[full_route.room_id(route, int(entry["room_index"]), entry["room"])] = dict(
                    start=f"seed:{seeds[entry['room_index']]}", inputs=entry["input_sha256"],
                    frames=int(entry["steps"]))
    return corpus, rooms


def check():
    """The locked ids; exits naming every stale entry or unlisted scenario."""
    with LOCK.open("rb") as stream:
        entries = {entry["id"]: entry for entry in tomllib.load(stream).get("scenario", [])}
    corpus, rooms = scenarios()
    keys, problems = corpus | rooms, []
    for name, entry in sorted(entries.items()):
        if name not in keys:
            problems.append(f"{name}: locked, but no such scenario")
        elif name in UNLOCKED:
            problems.append(f"{name}: both locked and listed as unlocked")
        elif entry["oracle"] != "faithful" or any(entry[field] != value for field, value in keys[name].items()):
            problems.append(f"{name}: stale lock entry (start, inputs or budget changed); re-record it")
    for name in sorted(set(UNLOCKED) - set(keys)):
        problems.append(f"{name}: listed as unlocked, but no such scenario")
    for name in sorted(set(keys) - set(entries) - set(UNLOCKED)):
        problems.append(f"{name}: neither locked nor listed as unlocked")
    if problems:
        raise SystemExit("Oracle digest lock is not current:\n" + "\n".join(problems))
    print(f"Lock current: {len(entries)} scenarios locked, {len(UNLOCKED)} unlocked")
    return set(entries)


def matrix(command, environment):
    """The job matrix one of the lockstep workflows' planners writes."""
    with tempfile.TemporaryDirectory() as work:
        output, summary = pathlib.Path(work, "output"), pathlib.Path(work, "summary")
        subprocess.run([sys.executable, *command], cwd=ROOT, check=True, stdout=subprocess.DEVNULL,
                       env=dict(os.environ, GITHUB_OUTPUT=str(output),
                                GITHUB_STEP_SUMMARY=str(summary), **environment))
        line = next(line for line in output.read_text().splitlines() if line.startswith("matrix="))
        return json.loads(line.removeprefix("matrix="))["include"]


def plan():
    locked = check()
    corpus = sorted(locked & set(scenarios()[0]))
    # Rooms run as the route jobs do (a room job checks only the rooms the lock
    # holds). Corpus rows shard by seed bundle as the lockstep corpus does, and
    # the shards pack into jobs of about a room job's frames, each restoring its
    # shards' bundles in turn.
    rooms = [dict(job, kind="room", name=f"route-{job['name']}")
             for job in matrix([".github/scripts/full_route.py", "plan"], {})]
    shards = matrix([".github/scripts/plan_corpus.py"], {"SELECTED": " ".join(corpus)})
    size = max(job["steps"] for job in rooms)
    bins = [[] for _ in range(max(1, round(sum(shard["cost"] for shard in shards) / size)))]
    for shard in sorted(shards, key=lambda shard: -shard["cost"]):
        min(bins, key=lambda group: sum(shard["cost"] for shard in group)).append(shard)
    jobs = rooms + [
        dict(kind="corpus", name=f"corpus-{index:02d}-{sum(len(s['segments'].split()) for s in group)}-rows",
             steps=sum(shard["cost"] for shard in group),
             segments=" ".join(shard["segments"] for shard in group),
             kinds=" ".join(sorted({shard["kind"] for shard in group})),
             shards=json.dumps([{key: shard[key] for key in ("kind", "seed_asset", "seed_sha256",
                                                             "segments", "route", "inputs")}
                                for shard in group]))
        for index, group in enumerate(bins) if group]
    jobs.sort(key=lambda job: -job["steps"])
    with open(os.environ["GITHUB_OUTPUT"], "a") as out:
        out.write(f"matrix={json.dumps({'include': jobs})}\n")
    print(f"{len(jobs)} jobs: " + ", ".join(job["name"] for job in jobs))


if __name__ == "__main__":
    {"check": check, "plan": plan}[sys.argv[1]]()
