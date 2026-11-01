#!/usr/bin/env python3
"""Plan the engine-only gate against the oracle digest lock.

Every pull request and push runs the engine alone over every scenario that
scenarios/lock.toml locks and compares its running digests with the faithful
oracle's (docs/SCENARIO_HARNESS.md, Decision 4; lock-gate.yml). The plan first
runs `tools/scenario.py lock --check`, which fails on a stale lock (an entry
whose start, inputs or budget no longer match its scenario, an entry for no
scenario, or a lockable scenario neither locked nor `unlocked`); it never
re-records.

  plan    check the lock and write the job matrix to GITHUB_OUTPUT

Recording (faithful oracle; docs/SCENARIO_HARNESS.md, "Oracle digest lock"):
  python tools/scenario.py lock --record
"""
import json, os, pathlib, subprocess, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import scenario  # noqa: E402


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
    locked = scenario.check(ROOT)
    corpus = sorted(locked & {row["id"] for row in scenario.rows(ROOT) if "chain" not in row})
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
    {"plan": plan}[sys.argv[1]]()
