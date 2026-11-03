#!/usr/bin/env python3
"""Split one Verify job's rows into its worker processes by their measured seconds.

replay_job.py runs one job's rows in its `workers` verifier processes; rows with
an input prefix share the first process (partition). The planner
(crates/sm_tools/src/bin/sm_pipeline, pack.rs) packed each job with the same
split, so both must change together.

row_seconds.toml holds each row's measured seconds per verifier mode
(row_seconds.sh regenerates it from CI logs); a row it lacks is estimated
conservatively from its frames.
"""
import pathlib, tomllib
from collections.abc import Mapping
from typing import Any

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
