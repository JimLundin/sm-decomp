#!/usr/bin/env python3
"""Plan one ci.yml Verify run as a single job matrix.

SCOPE selects what runs:
  gate      every locked scenario engine-only against the oracle lock, lockstep
            for the scenarios the lock cannot hold, and the ROM import check.
            Three long-prefix rows await room-seed reseeding and stay in full.
  full      the gate plus lockstep of the whole corpus and both routes.
  rows      lockstep of the space-separated rows in SELECTED.
  coverage  instrumented lockstep of the corpus (and routes with COVER_ROUTES=1).
The gate and rows scopes pack their rows into jobs by measured seconds (pack.py),
so the longest row sets the wall time and every job starts at once.
"""
import json, os, pathlib, subprocess, sys, tomllib
from collections.abc import Callable, Collection
from typing import Any

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
sys.path.insert(0, str(ROOT / ".github/scripts"))
import scenario  # noqa: E402
import full_route  # noqa: E402
import lock_gate  # noqa: E402
import pack  # noqa: E402

CORPUS = ".github/scripts/plan_corpus.py"
ROUTE = ".github/scripts/full_route.py"
# Above this many prefix frames a rejected checkpoint costs more than the rest of
# the gate, and every state layout change rejects it. These rows run in full.
LONG_PREFIX = 20000
SHARD = ("kind", "seed_asset", "seed_sha256", "segments", "route", "inputs")
# One kind of work: its restore group ("corpus" or a route), its matrix mode and its row ids.
Work = tuple[str, str, list[str]]


def corpus_job(shard: dict[str, Any], mode: str, timeout: int) -> dict[str, Any]:
    """One lockstep or oracle-sample shard."""
    record = shard.get("record_oracle", False)
    return dict(name=shard["name"], mode="record" if record else mode, steps=shard["cost"],
                segments=shard["segments"], kinds=shard["kind"], timeout=timeout,
                shards=json.dumps([{key: shard.get(key, "") for key in SHARD}]))


def corpus_jobs(selected: list[str] | None, sample: bool, timeout: int = 65) -> list[dict[str, Any]]:
    if selected is not None and not selected and not sample:
        return []
    environment = {"SELECTED": " ".join(selected or ()), "ORACLE_SAMPLE": "1" if sample else ""}
    return [corpus_job(shard, "lockstep", timeout) for shard in lock_gate.matrix([CORPUS], environment)]


def room_ids(job: dict[str, Any], by_id: dict[str, dict[str, Any]], keep: Callable[[str], bool]) -> list[str]:
    """The rooms of one full_route.py job that this mode runs."""
    listed = subprocess.run(
        [sys.executable, ROUTE, "jobs"], cwd=ROOT, check=True, capture_output=True, text=True,
        env=dict(os.environ, FULL_ROUTE=job["route"])).stdout
    ids: list[str] = json.loads(listed)[job["index"]]["ids"]
    return [name for name in ids if "pending" not in by_id[name] and keep(name)]


def room_job(job: dict[str, Any], mode: str, by_id: dict[str, dict[str, Any]], keep: Callable[[str], bool],
             timeout: int) -> dict[str, Any] | None:
    ids = room_ids(job, by_id, keep)
    return dict(name=job["name"], mode=mode, steps=job["steps"], kinds="room", timeout=timeout,
                route=job["route"], asset=job["asset"], segments=" ".join(ids)) if ids else None


def packed(work: list[Work], by_id: dict[str, dict[str, Any]], assets: dict[str, str],
           long_rows: Collection[str] = ()) -> list[dict[str, Any]]:
    """Jobs for the work, packed by measured seconds to one budget (pack.py).

    A route job restores its route's bundle; a corpus job restores each seed
    bundle or movie route its rows need once, as plan_corpus.py shards them."""
    merged: dict[tuple[str, str], list[str]] = {}
    for group, mode, ids in work:
        merged.setdefault((group, mode), []).extend(ids)
    merged = {key: list(dict.fromkeys(ids)) for key, ids in merged.items()}
    shard_of: dict[str, dict[str, Any]] = {}
    corpus = [name for (group, _), ids in merged.items() if group == "corpus" for name in ids]
    if corpus:
        # plan_corpus.py drops pending rows, so they never reach a job.
        for shard in lock_gate.matrix([CORPUS], {"SELECTED": " ".join(sorted(set(corpus))), "ORACLE_SAMPLE": ""}):
            shard_of.update((name, shard) for name in shard["segments"].split())
        for key, ids in merged.items():
            if key[0] == "corpus":
                ids[:] = [name for name in ids if name in shard_of]
    limit = pack.budget(((ids, mode) for (_, mode), ids in merged.items()), by_id)
    jobs: list[dict[str, Any]] = []
    for (group, mode), ids in merged.items():
        for index, segments in enumerate(pack.pack(ids, by_id, mode, limit) if ids else []):
            job: dict[str, Any] = dict(mode=mode, steps=sum(by_id[name].get("frames", 0) for name in segments),
                                       seconds=round(pack.estimate(segments, by_id, mode)),
                                       segments=" ".join(segments))
            if group == "corpus":
                restores: dict[tuple[str, ...], dict[str, Any]] = {}
                for name in segments:
                    shard = shard_of[name]
                    identity = tuple(str(shard.get(key, "")) for key in SHARD if key != "segments")
                    entry = restores.setdefault(identity, {key: shard.get(key, "") for key in SHARD} | {"segments": ""})
                    entry["segments"] = " ".join(entry["segments"].split() + [name])
                job.update(name=f"{mode}-corpus-{index:02d}-{len(segments)}-rows",
                           kinds=" ".join(sorted({entry["kind"] for entry in restores.values()})),
                           timeout=240 if set(segments) & set(long_rows) else 65,
                           shards=json.dumps(list(restores.values())))
            else:
                job.update(name=f"{mode}-{group}-{index:02d}-{len(segments)}-rooms", kinds="room", timeout=60,
                           route=group, asset=assets[group])
            jobs.append(job)
    return jobs


def main() -> None:
    scope = os.environ["SCOPE"]
    rows = scenario.rows(ROOT)
    by_id = {row["id"]: row for row in rows}
    locked = scenario.check(ROOT)
    room_timeout = 150 if scope == "coverage" else 60
    routes = lock_gate.matrix([ROUTE, "plan"], {}) if scope in ("full", "coverage") else []
    assets = {route: asset for route in full_route.ROUTES
              for asset in [full_route.published(route)[0]] if asset is not None}
    jobs: list[dict[str, Any]] = []
    long_rows = [row["id"] for row in rows if "chain" not in row and "pending" not in row
                 and row.get("start_frame", 0) > LONG_PREFIX]
    if scope == "gate":
        runnable = [row for row in rows if "pending" not in row]
        corpus = [row["id"] for row in runnable if "chain" not in row]
        work: list[Work] = [("corpus", "lock", [name for name in corpus if name in locked]),
                            # The lock cannot hold these: unlocked scenarios and checkpoint-backed rows.
                            ("corpus", "lockstep", [name for name in corpus if name not in locked
                                                    and name not in long_rows])]
        for route in assets:
            chain = [row["id"] for row in runnable if row.get("chain") == route]
            work += [(route, "lock", [name for name in chain if name in locked]),
                     (route, "lockstep", [name for name in chain if name not in locked])]
        jobs += packed(work, by_id, assets)
        jobs.append(dict(name="rom-import", mode="rom-import", steps=0, seconds=0, kinds="", timeout=30))
    elif scope == "full":
        for job in lock_gate.jobs():
            job = dict(job, mode="lock", timeout=60)
            planned: dict[str, Any] | None = job
            if job["kind"] == "room":
                planned = room_job(dict(job, name=job["name"].removeprefix("route-")), "lock", by_id,
                                   locked.__contains__, 60)
                if planned:
                    planned["name"] = "lock-" + planned["name"]
            else:
                job["name"] = "lock-" + job["name"]
            if planned:
                jobs.append(planned)
        jobs.append(dict(name="rom-import", mode="rom-import", steps=0, kinds="", timeout=30))
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
        jobs += packed([(by_id[name].get("chain", "corpus"), "lockstep", [name]) for name in selected],
                       by_id, assets, long_rows)
    elif scope == "coverage":
        jobs += corpus_jobs(None, False, 300)
        if os.environ.get("COVER_ROUTES") == "1":
            jobs += filter(None, (room_job(job, "lockstep", by_id, lambda _: True, room_timeout)
                                  for job in routes))
    else:
        raise SystemExit(f"unknown scope {scope}")
    # Start the longest jobs first: by estimated seconds where packed, else by frames.
    jobs.sort(key=lambda job: -job.get("seconds", job["steps"]))
    # Names label checks and artifacts, so each one must be unique.
    seen: dict[str, int] = {}
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
