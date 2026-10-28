#!/usr/bin/env python3
"""Read one Windows replay jobs root; verify receipts/logs without running jobs."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import sys
import time

VERIFIED = re.compile(r"^Verified (\d+) frames: (\d+) failing frames(?: |$)")
ROW = re.compile(r"^(PASS|FAIL) ([\w.-]+)(?:: .*)?$")
SOURCE = re.compile(r"^Verifier build/check: .*; source ([0-9a-f]{64})$")
IDENTITY = ("source", "dll_sha256", "codec")


def utc():
    return datetime.now(timezone.utc).isoformat()


def read_json(path, files, missing):
    if path.is_symlink():
        raise ValueError(f"receipt is a symlink: {path}")
    if not path.is_file():
        missing.append(str(path))
        return {}
    data = path.read_bytes()
    files[str(path)] = hashlib.sha256(data).hexdigest()
    value = json.loads(data.decode("utf-8-sig"))
    if not isinstance(value, dict):
        raise ValueError(f"{path}: expected a JSON object")
    return value


def phase(start, end, elapsed):
    wall = None
    if start and end:
        first, last = (datetime.fromisoformat(v.replace("Z", "+00:00"))
                       for v in (start, end))
        if first.tzinfo is None or last.tzinfo is None or last < first:
            raise ValueError(f"invalid START/EXIT interval: {start}, {end}")
        wall = (last - first).total_seconds()
    return {"start_utc": start, "exit_utc": end, "timestamp_wall_seconds": wall,
            "recorded_elapsed_seconds": elapsed}


def read_log(path):
    if path.is_symlink():
        raise ValueError(f"log is a symlink: {path}")
    digest, passes, failed_rows, sources = hashlib.sha256(), [], [], set()
    comparisons = failures = summaries = 0
    # Hash and parse the same bytes. Progress lines are deliberately not totals.
    with path.open("rb") as stream:
        for raw in stream:
            digest.update(raw)
            line = raw.decode("utf-8").strip()
            match = VERIFIED.match(line)
            if match:
                comparisons += int(match[1])
                failures += int(match[2])
                summaries += 1
            match = ROW.fullmatch(line)
            if match:
                (passes if match[1] == "PASS" else failed_rows).append(match[2])
            match = SOURCE.fullmatch(line)
            if match:
                sources.add(match[1])
    return {"sha256": digest.hexdigest(), "pass_rows": passes,
            "failed_rows": failed_rows, "verified_comparisons": comparisons,
            "failing_frames": failures, "verified_summaries": summaries,
            "log_sources": sorted(sources)}


def collect(root):
    began, started = time.perf_counter(), utc()
    files, missing, errors, jobs, passes = {}, [], [], {}, Counter()
    run_start = read_json(root / "RUN-START.json", files, missing)
    run_exit = read_json(root / "RUN-EXIT.json", files, missing)
    attribution = {key: run_start.get(key) for key in IDENTITY}
    for key, value in attribution.items():
        if not value:
            missing.append(f"RUN-START {key}")
        elif run_exit and run_exit.get(key) != value:
            errors.append(f"RUN-EXIT {key} differs from RUN-START")
    if run_exit and run_exit.get("start_utc") != run_start.get("utc"):
        errors.append("RUN-EXIT START differs from RUN-START utc")
    names = set(run_exit.get("jobs", {}))
    # Only immediate job directories in the exact supplied root; no recursive scan.
    names.update(p.name for p in root.iterdir() if p.is_dir())
    for name in sorted(names):
        if not re.fullmatch(r"[A-Za-z0-9_-]+", name):
            errors.append(f"invalid job directory name: {name!r}")
            continue
        job_root = root / name
        if job_root.is_symlink() or job_root.resolve().parent != root:
            errors.append(f"job directory leaves supplied root: {name}")
            continue
        error_count = len(errors)
        start = read_json(job_root / "START.json", files, missing)
        end = read_json(job_root / "EXIT.json", files, missing)
        recorded = run_exit.get("jobs", {}).get(name)
        if run_exit and recorded != end:
            errors.append(f"{name}: per-job EXIT differs from RUN-EXIT jobs")
        for receipt, label in ((start, "START"), (end, "EXIT")):
            if not receipt:
                continue
            if receipt.get("id") != name:
                errors.append(f"{name}: {label} id differs")
            # DLL is recorded in START; existing EXIT receipts omit that field.
            for key in IDENTITY if label == "START" else ("source", "codec"):
                if receipt.get(key) != attribution[key]:
                    errors.append(f"{name}: {label} {key} differs from RUN-START")
        if start and end and start.get("utc") != end.get("start_utc"):
            errors.append(f"{name}: START utc differs from EXIT start_utc")
        logs, valid = {}, bool(start and end and recorded == end and len(errors) == error_count)
        for channel in ("stdout", "stderr"):
            path = job_root / f"{channel}.log"
            if not path.is_file():
                missing.append(str(path))
                valid = False
                continue
            log = read_log(path)
            files[str(path)] = log["sha256"]
            logs[channel] = log
            expected = end.get(f"{channel}_sha256")
            if not expected:
                missing.append(f"{name}: EXIT {channel}_sha256")
                valid = False
            elif log["sha256"] != expected:
                errors.append(f"{name}: {channel} SHA256 mismatch")
                valid = False
            if any(source != attribution["source"] for source in log["log_sources"]):
                errors.append(f"{name}: {channel} source stamp differs")
                valid = False
        # The verifier's terminal summaries and PASS rows are stdout protocol.
        output = logs.get("stdout", {})
        count = output.get("verified_comparisons", 0) if valid else 0
        failing = output.get("failing_frames", 0) if valid else 0
        if end and end.get("exit") != 0:
            errors.append(f"{name}: process exit {end.get('exit')}")
        if valid and not output["verified_summaries"]:
            missing.append(f"{name}: stdout has no terminal Verified summary")
        if any(log["failing_frames"] or log["failed_rows"] for log in logs.values()):
            errors.append(f"{name}: recorded failing frames or FAIL rows")
        if valid:
            passes.update(output["pass_rows"])
        jobs[name] = {"phase": phase(end.get("start_utc", start.get("utc")),
                                     end.get("exit_utc"), end.get("seconds")),
                      "exit": end.get("exit"), "logs_hash_verified": valid,
                      "pass_rows": output.get("pass_rows", []) if valid else [],
                      "verified_comparisons": count, "failing_frames": failing,
                      "admission_released": end.get("admission_released"),
                      "budget_GiB": start.get("budget_GiB"),
                      "log_sha256": {key: value["sha256"] for key, value in logs.items()}}
    gaps = []
    for before, after in (
        ("fresh-initialized329", "fresh-initialized-resume5"),
        ("fresh-save-before-kago55650", "fresh-reserve147273-shared-prefix"),
        ("fresh-save-before-kago55650", "after-save-capture-both-kago"),
        ("fresh-reserve147273-shared-prefix", "after-reserve-capture"),
    ):
        first, last = jobs.get(before, {}).get("phase", {}), jobs.get(after, {}).get("phase", {})
        interval = phase(first.get("exit_utc"), last.get("start_utc"), None)
        gaps.append({"before": before, "after": after,
                     "seconds": interval["timestamp_wall_seconds"]})
    if not jobs:
        missing.append("no recorded jobs")
    result = {"jobs_root": str(root), "status": "FAIL" if errors else "INCOMPLETE" if missing else "PASS",
              "attribution": attribution,
              "run_phase": phase(run_start.get("utc"), run_exit.get("exit_utc"), run_exit.get("seconds")),
              "groups": len(jobs), "pass_rows": sorted(passes), "unique_pass_rows": len(passes),
              "duplicate_pass_rows": {name: count for name, count in passes.items() if count > 1},
              "verified_comparisons": sum(job["verified_comparisons"] for job in jobs.values()),
              "failing_frames": sum(job["failing_frames"] for job in jobs.values()),
              "jobs": jobs, "sequential_gaps": gaps,
              "run_admission_released": run_exit.get("admission_released"),
              "minimum_free_RAM_bytes": run_exit.get("minimum_free_RAM_bytes"),
              "minimum_free_disk_bytes": run_exit.get("minimum_free_disk_bytes"),
              "actual_file_sha256": files, "errors": errors, "missing_evidence": missing,
              "unknown_boundaries": ["source/review/build/landing START and EXIT are outside this jobs root",
                                     "recorded admission release does not inspect live process handles",
                                     "source/DLL/codec stamps are matched, external artifacts are not rehashed",
                                     "comparison totals count actual Verified summaries, not unique game frames"],
              "collection": {"start_utc": started, "exit_utc": utc(),
                             "seconds": time.perf_counter() - began}}
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("jobs_root", type=Path, help="exact directory containing RUN-START.json and RUN-EXIT.json")
    parser.add_argument("--format", choices=("text", "json"), default="text")
    args = parser.parse_args()
    try:
        result = collect(args.jobs_root.resolve(strict=True))
    except (OSError, ValueError, TypeError, KeyError) as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    if args.format == "json":
        print(json.dumps(result, indent=2))
    else:
        print(f"{result['status']}: {result['groups']} groups, {result['unique_pass_rows']} unique PASS rows, "
              f"{result['verified_comparisons']} verified comparisons / {result['failing_frames']} failing frames")
        print("Attribution: " + "; ".join(f"{key} {value}" for key, value in result["attribution"].items()))
        print("Run: " + json.dumps(result["run_phase"]))
        for name, job in result["jobs"].items():
            print(f"{name}: " + json.dumps(job))
        print("Sequential gaps: " + json.dumps(result["sequential_gaps"]))
        print(f"Run admission released: {result['run_admission_released']}")
        print("Rows: " + ", ".join(result["pass_rows"]))
        for label in ("errors", "missing_evidence", "unknown_boundaries"):
            print(f"{label}: " + json.dumps(result[label]))
        print("Collection: " + json.dumps(result["collection"]))
    return 0 if result["status"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
