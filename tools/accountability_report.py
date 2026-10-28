#!/usr/bin/env python3
"""Print at most six explicit local receipt rows; never infer execution from utc."""
import argparse
from datetime import datetime
import json
from pathlib import Path
import time


def pick(data, *keys):
    return next((data[k] for k in keys if data.get(k) is not None), None)


def stamp(value):
    if not value:
        return None
    result = datetime.fromisoformat(value.replace(" UTC", "+00:00").replace("Z", "+00:00"))
    if result.tzinfo is None:
        raise ValueError("timestamp has no timezone")
    return result


def cell(value):
    text = "UNKNOWN" if value is None else str(value)
    return text.replace("|", "\\|").replace("\n", " ").replace("\r", " ")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", nargs=3, action="append", required=True,
                        metavar=("PHASE", "OWNER", "PATH"))
    parser.add_argument("--utc-is-push", action="append", default=[], metavar="PHASE")
    args = parser.parse_args()
    if len(args.receipt) > 6:
        parser.error("at most six receipt rows")
    began, intervals, errors = time.perf_counter(), [], []
    print("| Phase | Owner | Actual START | First code | EXIT/push | Wall s | Observation | Result |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- |")
    for phase, owner, path in args.receipt:
        try:
            data = json.loads(Path(path).read_text(encoding="utf-8-sig"))
            owner = data.get("owner", owner)
            start = pick(data, "actual_start_utc", "start_utc", "started_utc", "commit_push_start_utc")
            first = pick(data, "first_code_utc", "coherent_code_utc", "firstcode_utc", "first_code_at")
            end = pick(data, "actual_exit_utc", "exit_utc", "push_utc", "pushed_utc", "published_utc")
            observed = pick(data, "observed_utc", "collected_utc", "utc")
            if phase in args.utc_is_push and end is None:
                end = data.get("utc")
                observed = pick(data, "observed_utc", "collected_utc")
            left, right = stamp(start), stamp(end)
            wall = None if left is None or right is None else (right-left).total_seconds()
            if wall is not None and wall < 0:
                raise ValueError("EXIT precedes START")
            if wall is not None:
                intervals.append((phase, left, right))
            result = pick(data, "status", "result")
            if data.get("commit"):
                result = f"{result or ''} commit {data['commit'][:8]}".strip()
            reported = pick(data, "commit_push_seconds", "build_seconds", "seconds")
            if reported is not None:
                result = f"{result or 'UNKNOWN'}; reported {reported:.3f}s"
            row = [phase, owner, start, first, end, None if wall is None else f"{wall:.3f}", observed, result]
        except (OSError, ValueError, TypeError, AttributeError) as error:
            errors.append(f"{phase}: {error}")
            row = [phase, owner, None, None, None, None, None, f"ERROR: {error}"]
        print("| " + " | ".join(cell(v) for v in row) + " |")
    overlaps = []
    for i, (phase, left, right) in enumerate(intervals):
        for other, begin, end in intervals[i+1:]:
            seconds = (min(right, end)-max(left, begin)).total_seconds()
            if seconds > 0:
                overlaps.append(f"{phase}/{other} {seconds:.3f}s")
    print("Overlap: " + (", ".join(overlaps) or "none between complete recorded intervals") +
          ("; UNKNOWN for rows missing START/EXIT" if len(intervals) < len(args.receipt) else ""))
    print(f"Observations are metadata, not execution. Collection {time.perf_counter()-began:.6f}s; no durations summed.")
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
