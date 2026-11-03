#!/usr/bin/env python3
"""Pack replay rows into Verify jobs by their measured seconds.

replay_job.py runs one job's rows in WORKERS verifier processes; rows with an
input prefix share the first process (partition). The planner gives every job
one budget, the longest single row in the plan, and packs each job's processes
to finish within it. The longest row sets the run's wall time anyway, so the
other jobs fill up to it, and the plan needs as few runners as it can: all of
them start at once, and fewer jobs pay the per-job checkout, artifact, ROM and
bundle setup. If that would take more than RUNNERS jobs, the budget grows.

row_seconds.toml holds each row's measured seconds per verifier mode
(row_seconds.sh regenerates it from CI logs); a row it lacks is estimated
conservatively from its frames.
"""
import math, pathlib, tomllib
from collections.abc import Iterable, Mapping
from typing import Any

# ubuntu-24.04 runners for public repositories have four cores (replay_job.py uses them all).
WORKERS = 4
# ci.yml runs at most 18 replay jobs at once; leave room for rom-import and the small lockstep jobs.
RUNNERS = 14
# A process of several rows is packed to this share of the budget, a margin for estimate error.
FILL = 0.9
# Below this a job spends a large share of its time on setup.
SHORTEST = 300.0
# Leaves a margin under ci.yml's 60-minute job timeouts for estimate error.
LONGEST = 1800.0
# Unmeasured rows: a row measured in the other mode takes about as long (lockstep
# ran 0.76 times its lock check where both were measured); otherwise lock checks
# run 6.7 frames a second or faster for nine rows in ten, and lockstep 8.3. A
# checkpoint row may regenerate its boot prefix about five times faster.
FPS = {"lock": 5.0, "lockstep": 5.0}
MEASURED: dict[str, dict[str, float]] = tomllib.loads(
    pathlib.Path(__file__).with_name("row_seconds.toml").read_text())


def seconds(row: Mapping[str, Any], mode: str) -> float:
    """A row's replay seconds in one matrix mode (lock, lockstep or record)."""
    table = "lock" if mode == "lock" else "lockstep"
    other = "lockstep" if table == "lock" else "lock"
    measured = MEASURED.get(table, {}).get(row["id"], MEASURED.get(other, {}).get(row["id"]))
    if measured is not None:
        return float(measured)
    prefix: int = row.get("start_frame", 0)
    frames: int = row.get("frames", 0) + (prefix // 5 if "checkpoint" in row else prefix)
    return max(1.0, frames / FPS[table])


def partition(ids: list[str], rows: Mapping[str, Mapping[str, Any]], mode: str,
              workers: int = WORKERS) -> list[list[str]]:
    """Split one job's rows into balanced processes, keeping prefix rows together."""
    prefixed = [name for name in ids if rows[name].get("start_frame", 0)]
    groups: list[list[str]] = [[] for _ in range(max(1, min(workers, len(ids))))]
    loads = [0.0] * len(groups)
    if prefixed:
        groups[0].extend(prefixed)
        loads[0] = sum(seconds(rows[name], mode) for name in prefixed)
    for name in sorted(set(ids) - set(prefixed), key=lambda name: (-seconds(rows[name], mode), name)):
        index = loads.index(min(loads))
        groups[index].append(name)
        loads[index] += seconds(rows[name], mode)
    return [group for group in groups if group]


def estimate(ids: list[str], rows: Mapping[str, Mapping[str, Any]], mode: str) -> float:
    """A job's estimated replay seconds: its slowest process."""
    return max(sum(seconds(rows[name], mode) for name in group) for group in partition(ids, rows, mode))


def budget(work: Iterable[tuple[Iterable[str], str]], rows: Mapping[str, Mapping[str, Any]]) -> float:
    """One plan's job budget over its (row ids, mode) work."""
    costs = [seconds(rows[name], mode) for ids, mode in work for name in ids] or [0.0]
    longest = max(costs)
    wanted = max(longest, sum(costs) / (WORKERS * FILL * RUNNERS), SHORTEST)
    return max(longest, min(wanted, LONGEST))


def pack(ids: list[str], rows: Mapping[str, Mapping[str, Any]], mode: str, limit: float) -> list[list[str]]:
    """The fewest jobs whose processes each fit the budget, longest rows first.

    A process of one row may take the whole budget; one of several rows
    keeps to FILL of it, and never past LONGEST. Each job is checked with
    partition, as it will run."""
    def cost(name: str) -> float:
        return seconds(rows[name], mode)

    order = sorted(ids, key=lambda name: (-cost(name), name))
    def fits(job: list[str]) -> bool:
        return all(len(group) == 1 or sum(map(cost, group)) <= FILL * min(limit, LONGEST)
                   for group in partition(job, rows, mode))

    count = max(1, math.ceil(sum(map(cost, order)) / (WORKERS * FILL * min(limit, LONGEST))))
    while True:
        jobs: list[list[str]] = [[] for _ in range(count)]
        loads = [[0.0] * WORKERS for _ in range(count)]
        for name in order:
            # Prefix rows run in the first process of their job.
            cores = range(1) if rows[name].get("start_frame", 0) else range(WORKERS)
            job, core = min(((job, core) for job in range(count) for core in cores),
                            key=lambda place: loads[place[0]][place[1]])
            jobs[job].append(name)
            loads[job][core] += cost(name)
        packed = [job for job in jobs if job]
        if count >= len(order) or all(map(fits, packed)):
            return packed
        count += 1
