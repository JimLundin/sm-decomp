#!/usr/bin/env python3
"""Plan one verify.yml run as a single job matrix.

SCOPE selects what runs:
  gate      every locked scenario engine-only against the oracle lock, lockstep
            for the scenarios the lock cannot hold, and the ROM import check.
            Three long-prefix rows await room-seed reseeding and stay in full.
  full      the gate plus lockstep of the whole corpus and both routes.
  rows      lockstep of the space-separated rows in SELECTED.
  coverage  instrumented lockstep of the corpus (and routes with COVER_ROUTES=1).
"""
import json, os, pathlib, sys, tomllib

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / ".github/scripts"))
import scenario  # noqa: E402
import lock_gate  # noqa: E402

CORPUS = ".github/scripts/plan_corpus.py"
ROUTE = ".github/scripts/full_route.py"
# Above this many prefix frames a rejected checkpoint costs more than the rest of
# the gate, and every state layout change rejects it. These rows run in full.
LONG_PREFIX = 20000
SHARD = ("kind", "seed_asset", "seed_sha256", "segments", "route", "inputs")


def corpus_job(shard, mode, timeout):
    """One lockstep or oracle-sample shard."""
    record = shard.get("record_oracle", False)
    return dict(name=shard["name"], mode="record" if record else mode, steps=shard["cost"],
                segments=shard["segments"], kinds=shard["kind"], timeout=timeout,
                shards=json.dumps([{key: shard.get(key, "") for key in SHARD}]))


def corpus_jobs(selected, sample, timeout=65):
    if selected is not None and not selected and not sample:
        return []
    environment = {"SELECTED": " ".join(selected or ()), "ORACLE_SAMPLE": "1" if sample else ""}
    return [corpus_job(shard, "lockstep", timeout) for shard in lock_gate.matrix([CORPUS], environment)]


def room_ids(job, by_id, keep):
    """The rooms of one full_route.py job that this mode runs."""
    listed = lock_gate.subprocess.run(
        [sys.executable, ROUTE, "jobs"], cwd=ROOT, check=True, capture_output=True, text=True,
        env=dict(os.environ, FULL_ROUTE=job["route"])).stdout
    ids = json.loads(listed)[job["index"]]["ids"]
    return [name for name in ids if "pending" not in by_id[name] and keep(name)]


def room_job(job, mode, by_id, keep, timeout):
    ids = room_ids(job, by_id, keep)
    return dict(name=job["name"], mode=mode, steps=job["steps"], kinds="room", timeout=timeout,
                route=job["route"], asset=job["asset"], segments=" ".join(ids)) if ids else None


def main():
    scope = os.environ["SCOPE"]
    rows = scenario.rows(ROOT)
    by_id = {row["id"]: row for row in rows}
    locked = scenario.check(ROOT)
    room_timeout = 150 if scope == "coverage" else 60
    routes = lock_gate.matrix([ROUTE, "plan"], {})
    jobs = []
    if scope in ("gate", "full"):
        for job in lock_gate.jobs():
            job = dict(job, mode="lock", timeout=60)
            if job["kind"] == "room":
                job = room_job(dict(job, name=job["name"].removeprefix("route-")), "lock", by_id,
                               locked.__contains__, 60)
                if job:
                    job["name"] = "lock-" + job["name"]
            else:
                job["name"] = "lock-" + job["name"]
            if job:
                jobs.append(job)
        jobs.append(dict(name="rom-import", mode="rom-import", steps=0, kinds="", timeout=30))
    long_rows = [row["id"] for row in rows if "chain" not in row and "pending" not in row
                 and row.get("start_frame", 0) > LONG_PREFIX]
    if scope == "gate":
        # The lock cannot hold these: unlocked scenarios and checkpoint-backed rows.
        selected = [row["id"] for row in rows if "chain" not in row and "pending" not in row
                    and row["id"] not in locked and row["id"] not in long_rows]
        jobs += corpus_jobs(selected, False)
        for route in {job["route"]: job for job in routes}.values():
            ids = [row["id"] for row in rows if row.get("chain") == route["route"]
                   and "pending" not in row and row["id"] not in locked]
            if ids:
                jobs.append(dict(name=f"lockstep-{route['route']}-unlocked-{len(ids)}-rooms", mode="lockstep",
                                 steps=sum(by_id[name].get("frames", 0) for name in ids), kinds="room",
                                 timeout=60, route=route["route"], asset=route["asset"],
                                 segments=" ".join(ids)))
    elif scope == "full":
        sample = os.environ.get("ORACLE_SAMPLE") == "1"
        heavy = set(long_rows)
        for shard in lock_gate.matrix([CORPUS], {"SELECTED": "", "ORACLE_SAMPLE": "1" if sample else ""}):
            job = corpus_job(shard, "lockstep", 65)
            if heavy & set(shard["segments"].split()):
                job["timeout"] = 240
            jobs.append(job)
        jobs += filter(None, (room_job(job, "lockstep", by_id, lambda _: True, 60) for job in routes))
    elif scope == "rows":
        selected = os.environ.get("SELECTED", "").split()
        unknown = set(selected) - set(by_id)
        if not selected or unknown:
            raise SystemExit(f"scope rows needs known row ids; unknown: {sorted(unknown)}")
        heavy = set(selected) & set(long_rows)
        corpus = [name for name in selected if "chain" not in by_id[name]]
        jobs += corpus_jobs(corpus, False, 240 if heavy else 65)
        for route in {job["route"]: job for job in routes}.values():
            ids = [name for name in selected if by_id[name].get("chain") == route["route"]]
            if ids:
                jobs.append(dict(name=f"lockstep-{route['route']}-{len(ids)}-rooms", mode="lockstep",
                                 steps=sum(by_id[name].get("frames", 0) for name in ids), kinds="room",
                                 timeout=60, route=route["route"], asset=route["asset"],
                                 segments=" ".join(ids)))
    elif scope == "coverage":
        jobs += corpus_jobs(None, False, 300)
        if os.environ.get("COVER_ROUTES") == "1":
            jobs += filter(None, (room_job(job, "lockstep", by_id, lambda _: True, room_timeout)
                                  for job in routes))
    else:
        raise SystemExit(f"unknown scope {scope}")
    jobs.sort(key=lambda job: -job["steps"])
    # Names label checks and artifacts, so each one must be unique.
    seen = {}
    for job in jobs:
        count = seen[job["name"]] = seen.get(job["name"], 0) + 1
        if count > 1:
            job["name"] = f"{job['name']}-{count}"
    movie = next(m for m in tomllib.load((ROOT / "movies/manifest.toml").open("rb"))["movies"]
                 if m["category"] == "any%")
    with open(os.environ["GITHUB_OUTPUT"], "a") as out:
        out.write(f"matrix={json.dumps({'include': jobs})}\n")
        out.write(f"movie_file={movie['file']}\nmovie_sha256={movie['sha256']}\n")
    with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as out:
        out.write(f"Scope {scope}: {len(jobs)} jobs\n\n")
        if scope == "gate":
            out.write("PR lock gate; three long-prefix rows remain pending room-seed reseeding.\n\n")
            for name in long_rows:
                if name not in locked:
                    out.write(f"Excluded from fast gate: {name}; pending room-seed reseeding; retained in nightly full cold replay.\n")
    print(f"{scope}: {len(jobs)} jobs: " + ", ".join(job["name"] for job in jobs))


if __name__ == "__main__":
    main()
