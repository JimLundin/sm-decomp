#!/usr/bin/env python3
"""Pack replay rows into Verify jobs by their measured seconds.

replay_job.py runs one job's rows in its `workers` verifier processes; rows with
an input prefix share the first process (partition). The longest row in a plan
sets its wall time, so it is the budget: every other job's processes are packed
to finish well within it, and the plan needs as few runners as it can, all
started at once. If that would take more than RUNNERS jobs, the budget grows.

The runners' four cores are two physical cores with two threads each, so a
row's speed depends on what else its runner replays (runs 41638757665 and
41645738482, against the old plan's mostly part-idle runners):
- the Mother Brain room took 850 to 1080 s with its runner otherwise idle and
  1470 s with three other processes beside it. A floor row (within FIRST of
  the budget) therefore gets a job of two processes, itself and one process of
  shorter rows from its group;
- lockstep rows took 1.6 to 1.9 times their measured seconds four to a runner,
  lock checks 0.6 to 1.4 times. Lockstep jobs run two processes.

row_seconds.toml holds each row's measured seconds per verifier mode
(row_seconds.sh regenerates it from CI logs); a row it lacks is estimated
conservatively from its frames.
"""
import math, pathlib, tomllib
from collections.abc import Iterable, Mapping
from typing import Any

# Processes per job: one per vCPU for lock checks, one per physical core for lockstep.
WORKERS = {"lock": 4, "lockstep": 2}
# verify.yml runs at most 18 replay jobs at once; the short lockstep route jobs and
# rom-import fill in after the first to finish.
RUNNERS = 17
# A process of several rows is packed to this share of the budget: a margin for
# estimate error and for runners that run slower.
FILL = 0.8
# A row within this share of the plan's longest row is a floor row.
FIRST = 0.9
# Below this a job spends a large share of its time on setup.
SHORTEST = 300.0
# Leaves a margin under verify.yml's 60-minute job timeouts for estimate error.
LONGEST = 1800.0
# Unmeasured rows: a row measured in the other mode takes about as long (lockstep
# ran 0.76 times its lock check where both were measured); otherwise lock checks
# run 6.7 frames a second or faster for nine rows in ten, and lockstep 8.3. A
# checkpoint row may regenerate its boot prefix about five times faster.
FPS = {"lock": 5.0, "lockstep": 5.0}
MEASURED: dict[str, dict[str, float]] = tomllib.loads(
    pathlib.Path(__file__).with_name("row_seconds.toml").read_text())
Rows = Mapping[str, Mapping[str, Any]]


def table(mode: str) -> str:
    """The row_seconds.toml table of a matrix mode (lock, lockstep or record)."""
    return "lock" if mode == "lock" else "lockstep"


def workers(mode: str) -> int:
    return WORKERS[table(mode)]


def seconds(row: Mapping[str, Any], mode: str) -> float:
    """A row's replay seconds in one matrix mode."""
    other = "lockstep" if table(mode) == "lock" else "lock"
    measured = MEASURED.get(table(mode), {}).get(row["id"], MEASURED.get(other, {}).get(row["id"]))
    if measured is not None:
        return float(measured)
    prefix: int = row.get("start_frame", 0)
    frames: int = row.get("frames", 0) + (prefix // 5 if "checkpoint" in row else prefix)
    return max(1.0, frames / FPS[table(mode)])


def partition(ids: list[str], rows: Rows, mode: str, count: int) -> list[list[str]]:
    """Split one job's rows into `count` balanced processes, keeping prefix rows together."""
    prefixed = [name for name in ids if rows[name].get("start_frame", 0)]
    groups: list[list[str]] = [[] for _ in range(max(1, min(count, len(ids))))]
    loads = [0.0] * len(groups)
    if prefixed:
        groups[0].extend(prefixed)
        loads[0] = sum(seconds(rows[name], mode) for name in prefixed)
    for name in sorted(set(ids) - set(prefixed), key=lambda name: (-seconds(rows[name], mode), name)):
        index = loads.index(min(loads))
        groups[index].append(name)
        loads[index] += seconds(rows[name], mode)
    return [group for group in groups if group]


def estimate(ids: list[str], rows: Rows, mode: str, count: int) -> float:
    """A job's estimated replay seconds: its slowest process."""
    return max(sum(seconds(rows[name], mode) for name in group) for group in partition(ids, rows, mode, count))


def budget(work: Iterable[tuple[Iterable[str], str]], rows: Rows) -> tuple[float, float]:
    """One plan's job budget over its (row ids, mode) work, and its longest row."""
    costs = [(seconds(rows[name], mode), workers(mode)) for ids, mode in work for name in ids] or [(0.0, 1)]
    longest = max(cost for cost, _ in costs)
    wanted = max(longest, sum(cost / count for cost, count in costs) / (FILL * RUNNERS), SHORTEST)
    return max(longest, min(wanted, LONGEST)), longest


def floor(ids: list[str], rows: Rows, mode: str, longest: float) -> tuple[list[list[str]], list[str]]:
    """Two-process jobs for the floor rows among ids, and the ids left.

    Beside each floor row runs one process of the longest other rows that fit
    within FILL of it; prefix rows stay out, since they would share its process."""
    def cost(name: str) -> float:
        return seconds(rows[name], mode)

    order = sorted(ids, key=lambda name: (-cost(name), name))
    floors = [name for name in order if cost(name) >= FIRST * longest]
    left = [name for name in order if name not in floors]
    jobs: list[list[str]] = []
    for name in floors:
        job, load = [name], 0.0
        for other in list(left):
            if not rows[other].get("start_frame", 0) and load + cost(other) <= FILL * cost(name):
                job.append(other)
                left.remove(other)
                load += cost(other)
        jobs.append(job)
    return jobs, left


def pack(ids: list[str], rows: Rows, mode: str, limit: float) -> list[list[str]]:
    """The fewest jobs whose processes each fit the budget, longest rows first.

    A process of one row may take the whole budget; one of several rows
    keeps to FILL of it, and never past LONGEST. Each job is checked with
    partition, as it will run."""
    def cost(name: str) -> float:
        return seconds(rows[name], mode)

    count = workers(mode)
    cap = FILL * min(limit, LONGEST)
    order = sorted(ids, key=lambda name: (-cost(name), name))
    def fits(job: list[str]) -> bool:
        return all(len(group) == 1 or sum(map(cost, group)) <= cap for group in partition(job, rows, mode, count))

    total = max(1, math.ceil(sum(map(cost, order)) / (count * cap)))
    while True:
        jobs: list[list[str]] = [[] for _ in range(total)]
        loads = [[0.0] * count for _ in range(total)]
        for name in order:
            # Prefix rows run in the first process of their job.
            cores = range(1) if rows[name].get("start_frame", 0) else range(count)
            job, core = min(((job, core) for job in range(total) for core in cores),
                            key=lambda place: loads[place[0]][place[1]])
            jobs[job].append(name)
            loads[job][core] += cost(name)
        packed = [job for job in jobs if job]
        if total >= len(order) or all(map(fits, packed)):
            return packed
        total += 1
