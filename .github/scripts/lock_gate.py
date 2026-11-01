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
  record-oracle    re-record SEGMENTS privately and compare their committed entries

Recording (faithful oracle; docs/SCENARIO_HARNESS.md, "Oracle digest lock"):
  python tools/scenario.py lock --record
"""
import datetime, hashlib, json, os, pathlib, subprocess, sys, tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import scenario  # noqa: E402


def oracle_sample(rows, locked, slot, minimum=30):
    """Rotate equally through the corpus and each route every three-hour UTC slot."""
    pools = {}
    for row in rows:
        if row["id"] in locked:
            pools.setdefault(row.get("chain", "corpus"), []).append(row["id"])
    if sum(map(len, pools.values())) < minimum:
        raise ValueError(f"oracle sample needs at least {minimum} locked scenarios")
    selected = []
    for index, (_, pool) in enumerate(sorted(pools.items())):
        pool.sort()
        quota = minimum // len(pools) + (index < minimum % len(pools))
        start = (slot * quota) % len(pool)
        selected.extend(pool[(start + i) % len(pool)] for i in range(min(quota, len(pool))))
    # Small pools contribute all their rows; fill the remaining places from the others.
    remaining = sorted((set(locked) & {row["id"] for row in rows}) - set(selected))
    short = minimum - len(selected)
    if short:
        start = (slot * short) % len(remaining)
        selected.extend(remaining[(start + i) % len(remaining)] for i in range(short))
    return selected


def oracle_slot():
    return int(datetime.datetime.now(datetime.timezone.utc).timestamp()) // (3 * 60 * 60)


def compare_oracle(locked, recorded, selected):
    """Exact entries include identity, format, ticks, every domain and every mark."""
    failures = []
    if set(recorded) != set(selected):
        failures.append("recorded scenario ids differ from the selected ids")
    for name in selected:
        expected, actual = locked.get(name), recorded.get(name)
        if expected is None or actual is None:
            failures.append(f"{name}: missing oracle entry")
            continue
        for field in sorted(expected.keys() | actual.keys()):
            if expected.get(field) != actual.get(field):
                if field in ("digests", "marks"):
                    before, after = expected.get(field, {}), actual.get(field, {})
                    changed = sorted(key for key in before.keys() | after.keys()
                                     if before.get(key) != after.get(key))
                    failures.append(f"{name}: {field} differ in {', '.join(changed)}")
                else:
                    failures.append(f"{name}: {field} differs")
    return failures


def record_oracle():
    """Re-record into a private temporary lock; never write the committed lock."""
    selected = os.environ["SEGMENTS"].split()
    locked = scenario.check(ROOT)
    if not selected or len(selected) != len(set(selected)) or set(selected) - locked:
        raise SystemExit("oracle sample must name distinct already locked scenarios")
    committed = ROOT / scenario.LOCK
    before = hashlib.sha256(committed.read_bytes()).digest()
    expected = scenario.read_lock(ROOT).entries
    with tempfile.TemporaryDirectory(prefix="oracle-sample-") as directory:
        recorded = pathlib.Path(directory, "lock.toml")
        # Raw verifier output stays private on the runner, including on failure.
        with pathlib.Path(directory, "replay.log").open("w") as log:
            result = subprocess.run(
                [sys.executable, str(ROOT / "tools/verify_dll.py"), str(ROOT / "rom/sm.smc"),
                 "--corpus", str(ROOT / "scenarios"), "--no-build", "--direct",
                 "--mode", "lock", "--oracle", "faithful", "--lock", str(recorded),
                 *[part for name in selected for part in ("--segment", name)]],
                cwd=ROOT, stdout=log, stderr=subprocess.STDOUT)
        if hashlib.sha256(committed.read_bytes()).digest() != before:
            raise SystemExit("FAIL: committed oracle lock changed during recording")
        if result.returncode:
            raise SystemExit(f"FAIL: faithful oracle recording exited {result.returncode}")
        failures = compare_oracle(expected, scenario.read_lock(ROOT, recorded).entries, selected)
    for failure in failures:
        print(f"FAIL {failure}", flush=True)
    if failures:
        raise SystemExit(1)
    for name in selected:
        entry = expected[name]
        print(f"PASS {name}: oracle lock matches; {entry['ticks']} ticks; "
              f"domains {', '.join(sorted(entry['digests']))}; hash {entry['combined']}", flush=True)


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
    {"plan": plan, "record-oracle": record_oracle}[sys.argv[1]]()
