#!/usr/bin/env python3
"""Compare lsnes and reference checkpoints at the same game-loop instruction.

An explicit constant frame offset can account for different emulator frame
numbering. No per-frame resynchronization or value tolerance is performed.
"""
import argparse
import csv
import hashlib
import json
from pathlib import Path

FIELDS = ("game_state", "room", "x", "y", "health", "items",
          "time_frames", "time_seconds", "time_minutes", "time_hours")


def coverage(path, rows):
    """A completed, hash-bound capture may outlast the last gameplay visit."""
    sidecar = path.with_suffix(".capture.json")
    if not sidecar.exists():
        return rows[-1]["frame"]
    record = json.loads(sidecar.read_text())
    if record["checkpoint_sha256"] != hashlib.sha256(path.read_bytes()).hexdigest():
        raise ValueError("capture checksum mismatch: {}".format(path))
    return int(record["completed_frames"])


def read(path):
    with path.open() as source:
        reader = csv.DictReader(source)
        if reader.fieldnames != ["frame", "event", *FIELDS]:
            raise ValueError("unexpected checkpoint columns in {}".format(path))
        rows = [{key: int(value, 16 if key in FIELDS else 10) for key, value in row.items()} for row in reader]
    if not rows:
        raise ValueError("empty checkpoints: {}".format(path))
    for index, row in enumerate(rows):
        if row["event"] != index + 1 or (index and row["frame"] < rows[index - 1]["frame"]):
            raise ValueError("missing/reordered events in {}".format(path))
    return rows


def compare(baseline, candidate, start_frame, end_frame, frame_offset):
    # Filter by a common time window. Multiple game-loop visits in a video
    # frame are retained, in order; dropped/extra visits cause a failure.
    expected = [row for row in baseline if start_frame <= row["frame"] <= end_frame]
    actual = [row for row in candidate if start_frame <= row["frame"] - frame_offset <= end_frame]
    if not expected or not actual:
        return {"status": "FAIL", "reason": "selected interval has no checkpoints"}
    for index, (left, right) in enumerate(zip(expected, actual)):
        differences = {}
        if left["frame"] != right["frame"] - frame_offset:
            differences["frame"] = [left["frame"], right["frame"] - frame_offset]
        for field in FIELDS:
            if left[field] != right[field]:
                differences[field] = [left[field], right[field]]
        if differences:
            return {"status": "FAIL", "matched_events": index, "baseline_event": left["event"],
                    "candidate_event": right["event"], "baseline_frame": left["frame"], "differences": differences}
    if len(expected) != len(actual):
        return {"status": "FAIL", "reason": "checkpoint counts differ", "baseline_events": len(expected),
                "candidate_events": len(actual)}
    return {"status": "PASS", "events": len(expected), "start_frame": start_frame,
            "end_frame": end_frame, "candidate_frame_offset": frame_offset}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("baseline", type=Path)
    parser.add_argument("candidate", type=Path)
    parser.add_argument("--start-frame", type=int, required=True,
                        help="start after the compared game fields have been initialized")
    parser.add_argument("--end-frame", type=int, required=True)
    parser.add_argument("--candidate-frame-offset", type=int, default=0)
    parser.add_argument("--out", type=Path, default=Path("out/sync/report.json"))
    args = parser.parse_args()
    if args.start_frame < 1 or args.end_frame < args.start_frame:
        parser.error("require 1 <= start-frame <= end-frame")
    baseline, candidate = read(args.baseline), read(args.candidate)
    if coverage(args.baseline, baseline) < args.end_frame or coverage(args.candidate, candidate) - args.candidate_frame_offset < args.end_frame:
        raise SystemExit("checkpoint captures do not cover --end-frame")
    report = compare(baseline, candidate, args.start_frame, args.end_frame, args.candidate_frame_offset)
    report.update(baseline=str(args.baseline), candidate=str(args.candidate), checkpoint_pc="828948")
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    raise SystemExit(0 if report["status"] == "PASS" else 1)


if __name__ == "__main__":
    main()
