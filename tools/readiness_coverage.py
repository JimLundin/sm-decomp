#!/usr/bin/env python3
"""Bind actual instruction coverage to preserved candidate and predecessor keys."""
import hashlib
import json
import re
from pathlib import Path

from readiness import digest, inventory_digest, write_json, read_verdict


def _table_rows(source):
    bodies = {(int(address, 16), int(mx)): body for address, mx, body in
              re.findall(r"fn step_([0-9a-f]+)_([0-3])\(m: &mut Machine\) \{(.*?)\n\}", source, re.S)}
    rows = []
    for address, mx, raw_bytes, symbol_address, symbol_mx in re.findall(
            r"\(0x([0-9a-f]+), ([0-3]), &\[([^\]]*)\], step_([0-9a-f]+)_([0-3])\)", source):
        key = (int(address, 16), int(mx))
        if key != (int(symbol_address, 16), int(symbol_mx)) or key not in bodies:
            raise RuntimeError("Generated table/body key mismatch")
        values = [int(value.strip()) for value in raw_bytes.split(",") if value.strip()]
        if not values or any(value < 0 or value > 255 for value in values):
            raise RuntimeError("Generated table has invalid instruction bytes")
        rows.append({"address": key[0], "mx": key[1], "opcode": values[0], "bytes": values,
                     "body_sha256": hashlib.sha256(bodies[key].encode()).hexdigest()})
    if len(rows) != len({(row["address"], row["mx"]) for row in rows}):
        raise RuntimeError("Generated table has duplicate keys")
    return rows


def generated_inventory(run, steps):
    """Preserve exact generated bodies and the maintained inputs defining them."""
    steps = Path(steps)
    source = steps.read_text()
    rows = _table_rows(source)
    match = re.search(r'ROM_SHA256:\s*Option<&str>\s*=\s*Some\("([0-9a-f]{64})"\)', source)
    if not match:
        raise RuntimeError("Generated table lacks ROM identity")
    config = run.path / "source/files/config/functions.toml"
    relevant = [entry for entry in run.source_files if entry["path"].startswith(
        ("crates/recomp/src/", "crates/runtime/src/", "crates/snes/src/")) or entry["path"] in
        ("Cargo.toml", "Cargo.lock", "crates/game/build.rs", "config/jump_tables.toml")]
    report = {"schema_version": 1, "rom_sha256": match.group(1), "steps_sha256": digest(steps),
              "steps_evidence": steps.relative_to(run.path).as_posix(), "config_sha256": digest(config), "execution_source_sha256": inventory_digest(relevant),
              "source_sha256": run.manifest["identities"]["source"]["sha256"], "keys": sorted(rows, key=lambda row: (row["address"], row["mx"]))}
    path = run.path / "generated/key-inventory.json"
    write_json(path, report)
    identity = run.record_artifact("generated-key-inventory", path)
    identity["evidence"] = path.relative_to(run.path).as_posix()
    run.save_manifest()
    run.guard_input(path)
    return path


def _predecessor(path):
    path = Path(path)
    manifest = json.loads((path / "manifest.json").read_text())
    if read_verdict(path)["status"] != "pass":
        raise RuntimeError("Predecessor lacks a completed passing verdict")
    identity = manifest["identities"]["generated-key-inventory"]
    relative = Path(identity["evidence"])
    if relative.is_absolute() or ".." in relative.parts:
        raise RuntimeError("Predecessor inventory path is unsafe")
    artifact = path / relative
    if not artifact.is_file() or digest(artifact) != identity["sha256"]:
        raise RuntimeError("Predecessor generated-key inventory is missing or corrupt")
    source = json.loads((path / "source/inventory.json").read_text())
    if source["sha256"] != inventory_digest(source["files"]) or source["sha256"] != manifest["identities"]["source"]["sha256"]:
        raise RuntimeError("Predecessor maintained source snapshot identity mismatch")
    for entry in source["files"]:
        if digest(path / "source/files" / entry["path"]) != entry["sha256"]:
            raise RuntimeError("Predecessor maintained source snapshot is corrupt")
    inventory = json.loads(artifact.read_text())
    steps_relative = Path(inventory["steps_evidence"])
    if steps_relative.is_absolute() or ".." in steps_relative.parts:
        raise RuntimeError("Predecessor generated steps path is unsafe")
    if digest(path / steps_relative) != inventory["steps_sha256"]:
        raise RuntimeError("Predecessor generated steps are corrupt")
    if inventory["keys"] != _table_rows((path / steps_relative).read_text()):
        raise RuntimeError("Predecessor inventory differs from actual generated steps")
    return manifest, inventory


def evaluate_coverage(run, profile_path, inventory_path, predecessor=None):
    """Record the strict changed-entry gate; initial readiness retains legacy debt."""
    profile_path, inventory_path = Path(profile_path), Path(inventory_path)
    profile = json.loads(profile_path.read_text())
    candidate = json.loads(inventory_path.read_text())
    if candidate.get("source_sha256") != run.manifest["identities"]["source"]["sha256"]:
        raise RuntimeError("Candidate key inventory source identity mismatch")
    if candidate.get("rom_sha256") != run.manifest.get("identities", {}).get("inputs", {}).get("rom_sha256", candidate.get("rom_sha256")):
        raise RuntimeError("Candidate key inventory ROM identity mismatch")
    steps_relative = Path(candidate["steps_evidence"])
    if steps_relative.is_absolute() or ".." in steps_relative.parts:
        raise RuntimeError("Candidate generated steps path is unsafe")
    if digest(run.path / steps_relative) != candidate["steps_sha256"]:
        raise RuntimeError("Candidate generated steps are corrupt")
    if candidate["keys"] != _table_rows((run.path / steps_relative).read_text()):
        raise RuntimeError("Candidate inventory differs from actual generated steps")
    identity = run.manifest["identities"]["generated-key-inventory"]
    if digest(inventory_path) != identity["sha256"]:
        raise RuntimeError("Candidate generated inventory artifact mismatch")
    if not profile.get("complete") or profile.get("expected_frames") != run.manifest["scope"]["frames"] \
            or profile.get("completed_frames") != profile["expected_frames"]:
        raise RuntimeError("Instruction coverage lacks complete matching replay scope")
    if profile.get("candidate_inventory_sha256") != digest(inventory_path):
        raise RuntimeError("Instruction profile candidate identity mismatch")
    if any(not isinstance(row.get("count"), int) or row["count"] < 0 for row in profile["interpreted"]):
        raise RuntimeError("Interpreted instruction profile has invalid counts")
    if sum(row["count"] for row in profile["interpreted"]) != profile["interpreted_instructions"]:
        raise RuntimeError("Interpreted instruction totals differ from profile counts")
    expected_config = run.path / "source/files/config/functions.toml"
    if expected_config.exists() and candidate.get("config_sha256") != digest(expected_config):
        raise RuntimeError("Candidate key inventory config identity mismatch")
    current = {(row["address"], row["mx"]): row for row in candidate["keys"]}
    counts = {(row["address"], row["mx"]): row["count"] for row in profile["compiled"]}
    if profile.get("schema_version") != 1 or candidate.get("schema_version") != 1:
        raise RuntimeError("Unsupported coverage or key inventory schema")
    if len(counts) != len(profile["compiled"]) or len(current) != len(candidate["keys"]):
        raise RuntimeError("Coverage has duplicate compiled keys")
    if any(row["count"] < 0 or row["opcode"] != current.get((row["address"], row["mx"]), {}).get("opcode")
           for row in profile["compiled"]):
        raise RuntimeError("Coverage opcode/count differs from generated inventory")
    if sum(counts.values()) != profile["generated_instructions"]:
        raise RuntimeError("Generated instruction totals differ from per-key counts")
    if set(current) != set(counts):
        raise RuntimeError("Executed table inventory differs from generated candidate keys")
    inputs = run.manifest.get("identities", {}).get("inputs", {})
    domain = {"rom": inputs.get("rom_sha256"), "movie": inputs.get("movie_sha256"),
              "input": inputs.get("input_sha256"), "profile": inputs.get("profile"),
              "seed": inputs.get("seed"), "controller": inputs.get("controller"),
              "reset": inputs.get("reset"), "start": 0, "frames": run.manifest["scope"].get("frames")}
    required = []
    parent_identity = None
    if predecessor is not None:
        manifest, previous = _predecessor(predecessor)
        if run.manifest.get("accepted_parent") not in (None, manifest["run_id"]):
            raise RuntimeError("Predecessor run differs from declared parent")
        old = {(row["address"], row["mx"]): row for row in previous["keys"]}
        parent_identity = {"run_id": manifest["run_id"], "inventory_sha256": manifest["identities"]["generated-key-inventory"]["sha256"],
                           "source_sha256": manifest["identities"]["source"]["sha256"]}
        shared_change = candidate.get("execution_source_sha256") != previous.get("execution_source_sha256")
        for key, row in sorted(current.items()):
            reason = "added" if key not in old else "body changed" if row != old[key] else "execution source changed" if shared_change else None
            if reason:
                required.append({"address": key[0], "mx": key[1], "reason": reason})
    uncovered = [{"address": row["address"], "mx": row["mx"]} for row in required
                 if counts[(row["address"], row["mx"])] == 0]
    executed = [row for row in profile["compiled"] if row["count"] > 0]
    unexecuted = [dict(row, validation="unvalidated baseline" if not required else "unvalidated")
                  for row in profile["compiled"] if row["count"] == 0]
    generated = profile["generated_instructions"]
    interpreted = profile["interpreted_instructions"]
    report = {"schema_version": 1, "status": "uncovered" if uncovered or not current or generated == 0 else "pass",
              "complete": True, "required_keys": required, "uncovered_keys": uncovered,
              "executed": executed, "unexecuted": unexecuted,
              "unsupported": [row for row in profile["interpreted"] if not row.get("supported", False)],
              "deferred": [row for row in profile["interpreted"] if row.get("supported", False)],
              "generated_instructions": generated, "interpreted_instructions": interpreted,
              "fallback_fraction": interpreted / (generated + interpreted) if generated + interpreted else 0,
              "validated_distinct_keys": len(executed), "predecessor": parent_identity,
              "candidate_inventory_sha256": digest(inventory_path),
              "candidate_source_sha256": run.manifest["identities"]["source"]["sha256"],
              "profile_sha256": digest(profile_path), "domain": domain, "discovery": "call-target misses are separate; not instruction counts"}
    if report["status"] == "uncovered":
        report["reason"] = "Changed compiled keys did not execute" if uncovered else "Empty table or no actual compiled execution"
    by_pattern = {}
    for kind, rows in (("generated", profile["compiled"]), ("interpreted", profile["interpreted"])):
        for row in rows:
            group = (row["opcode"], row.get("pattern", "unknown"), row.get("emulation", False))
            counts = by_pattern.setdefault(group, {"opcode": group[0], "pattern": group[1], "emulation": group[2],
                                                   "generated": 0, "interpreted": 0})
            counts[kind] += row["count"]
    report["opcode_patterns"] = [by_pattern[key] for key in sorted(by_pattern)]
    if predecessor is not None:
        old_coverage = Path(predecessor) / "coverage.json"
        parent_coverage_check = manifest.get("state") == "complete"
        parent_verdict = read_verdict(predecessor)
        expected_coverage = parent_verdict.get("checks", {}).get("changed-entry-coverage")
        if old_coverage.exists() and expected_coverage is not None and json.loads(old_coverage.read_text()) == expected_coverage:
            report["progress"] = comparable_progress(report, json.loads(old_coverage.read_text()))
        else:
            report["progress"] = {"comparable": False, "reason": "Predecessor lacks instruction coverage"}
    else:
        report["progress"] = {"comparable": False, "reason": "Initial readiness baseline"}
    write_json(run.path / "coverage.json", report)
    run.record_check("changed-entry-coverage", report)
    return report


def comparable_progress(candidate, previous):
    if candidate.get("domain") != previous.get("domain") or not candidate.get("domain"):
        return {"comparable": False, "reason": "ROM/movie/profile/seed/input interval differs or is absent"}
    return {"comparable": True,
            "generated_delta": candidate["generated_instructions"] - previous["generated_instructions"],
            "interpreted_delta": candidate["interpreted_instructions"] - previous["interpreted_instructions"],
            "validated_keys_delta": candidate.get("validated_distinct_keys", 0) - previous.get("validated_distinct_keys", 0)}
