#!/usr/bin/env python3
"""Build and cold-replay an isolated, identity-bound parity candidate."""
import argparse
import os
import json
import hashlib
import time
import sys
from pathlib import Path

from readiness import Run, preflight, write_json, digest
from readiness_coverage import generated_inventory, evaluate_coverage


def prepare_candidate(run, mode="generated", timeout=None):
    """Build exact candidate inputs without running or recording parity validation."""
    if mode not in ("fallback-only", "generated"):
        raise ValueError("Unknown candidate mode")
    generated = run.path / "generated" / mode
    if generated.exists():
        source = generated / "steps.rs"
        game = generated / "game.rs"
        if source.is_file() and game.is_file():
            run.check_sources()
            for name,path in (("steps",source),("game",game)):
                expected=run.manifest["identities"].get(mode+"-generated-"+name,{})
                if expected.get("sha256")!=digest(path):
                    raise RuntimeError("Prepared generated inputs lack matching build provenance")
            candidate=generated/"diff"
            if run.manifest["identities"].get(mode+"-binary",{}).get("sha256")!=digest(candidate):
                raise RuntimeError("Prepared candidate binary lacks matching build provenance")
            return {"status": "pass", "complete": True, "generated": generated,
                    "build": run.path / "build" / mode, "game": game, "steps": source, "timings": {},
                    "key_inventory": run.path / "generated/key-inventory.json" if mode == "generated" else None}
        raise RuntimeError("Incomplete candidate artifacts already exist")
    generated.mkdir()
    build = run.path / "build" / mode
    env = os.environ.copy()
    env["CARGO_TARGET_DIR"] = str(build)
    game_source, steps_source = generated / "game.rs", generated / "steps.rs"
    timings = {}
    frames = 0

    def stage(name, command, **kwargs):
        report = run.run_stage(mode + "-" + name, command, env=env, **kwargs)
        timings[name] = report["elapsed_seconds"]
        return report

    if mode == "generated":
        result = stage("generator-build", ["cargo", "build", "--release", "-p", "recomp", "--locked", "--offline"])
        if result["status"] != "pass":
            report = {"status": result["status"], "complete": False, "expected_frames": frames,
                      "completed_frames": 0, "reason": result.get("reason", "Required build/generation stage failed"),
                      "timings": timings, "failed_stage": result["command"]}
            return report
        binary = build / "release/recomp"
        run.record_artifact(mode + "-generator-binary", binary, "generated/" + mode + "/recomp")
        generator = generated / "recomp"
        run.guard_input(generator)
        result = stage("generation", [generator, "--rom", run.root / "rom/sm.sfc",
                        "--functions", run.root / "config/functions.toml", "--out", game_source,
                        "--steps-out", steps_source])
        if result["status"] != "pass":
            report = {"status": result["status"], "complete": False, "expected_frames": frames,
                      "completed_frames": 0, "reason": result.get("reason", "Required build/generation stage failed"),
                      "timings": timings, "failed_stage": result["command"]}
            return report
    else:
        # Untimed whole-subroutine output is not used to establish cycle evidence.
        game_source.write_text("pub static TABLE: &[runtime::TableEntry] = &[];\n")
        steps_source.write_text("pub static STEP_TABLE: &[runtime::StepEntry] = &[];\n"
                                "pub const ROM_SHA256: Option<&str> = None;\n")
        timings["generation"] = 0.0
    for name, path in (("game", game_source), ("steps", steps_source)):
        identity = run.record_artifact(mode + "-generated-" + name, path)
        identity["evidence"] = path.relative_to(run.path).as_posix()
        identity["execution"] = "untimed whole-subroutine output; not cycle evidence" if name == "game" else "resumable timed instruction table"
        run.guard_input(path)
    run.manifest.setdefault("parity_build_inputs", {})[mode] = {
        "game": game_source.relative_to(run.path).as_posix(), "steps": steps_source.relative_to(run.path).as_posix(),
        "target_directory": str(build), "environment": {"SM_RECOMP_GENERATED_GAME": str(game_source),
                                                         "SM_RECOMP_GENERATED_STEPS": str(steps_source)}}
    run.save_manifest()
    key_inventory = generated_inventory(run, steps_source) if mode == "generated" else None
    env["SM_RECOMP_GENERATED_GAME"] = str(game_source)
    env["SM_RECOMP_GENERATED_STEPS"] = str(steps_source)
    result = stage("build", ["cargo", "build", "--release", "-p", "diff", "--locked", "--offline"])
    if result["status"] != "pass":
        report = {"status": result["status"], "complete": False, "expected_frames": frames,
                  "completed_frames": 0, "reason": result.get("reason", "Required build stage failed"), "timings": timings}
        return report
    binary = build / "release/diff"
    run.record_artifact(mode + "-binary", binary, "generated/" + mode + "/diff")
    run.guard_input(generated / "diff")
    return {"status": "pass", "complete": True, "generated": generated, "build": build,
            "game": game_source, "steps": steps_source, "timings": timings, "key_inventory": key_inventory}


def run_parity(run, mode, frames=18000, timeout=None, predecessor=None):
    """Run one cold parity gate within an open, preflight-verified parent run."""
    if mode not in ("fallback-only", "generated") or frames <= 0:
        raise ValueError("Parity requires a valid mode and positive exact frame count")
    started = time.monotonic()
    inputs = run.manifest["identities"]["inputs"]
    if frames > inputs["frames"]:
        report = {"status": "incomplete", "complete": False, "expected_frames": frames,
                  "completed_frames": 0, "reason": "Verified movie is shorter than requested scope"}
        run.record_check(mode + "-parity", report)
        return report
    preparation = prepare_candidate(run, mode, timeout)
    if preparation["status"] != "pass":
        run.record_check(mode + "-parity", preparation)
        return preparation
    generated, build = preparation["generated"], preparation["build"]
    game_source, steps_source = preparation["game"], preparation["steps"]
    key_inventory = preparation["key_inventory"]
    timings = preparation["timings"]
    env = dict(os.environ, CARGO_TARGET_DIR=str(build), SM_RECOMP_GENERATED_GAME=str(game_source),
               SM_RECOMP_GENERATED_STEPS=str(steps_source))
    def stage(name, command, **kwargs):
        report = run.run_stage(mode + "-" + name, command, env=env, **kwargs)
        timings[name] = report["elapsed_seconds"]
        return report
    binary = build / "release/diff"
    identity = run.manifest["identities"][mode + "-binary"]
    candidate = generated / "diff"
    run.guard_input(candidate)
    output = run.path / "diagnostics" / (mode + "-frame-parity")
    result = stage("replay", [candidate, run.root / "movies/super-metroid-any-3653.lsmv",
                    "--" + mode, "--frames", str(frames), "--rom", run.root / "rom/sm.sfc", "--out", output],
                   timeout=timeout, exit_statuses={1: "divergence", 2: "error"})
    raw_report = output / "parity.json"
    if result["status"] == "incomplete" or result["status"] == "error" and (result["returncode"] != 2 or not raw_report.exists()):
        report = {"status": result["status"], "complete": False, "expected_frames": frames,
                  "completed_frames": 0, "reason": result.get("reason", "Replay did not complete")}
        if raw_report.exists():
            partial = json.loads(raw_report.read_text())
            report["completed_frames"] = partial.get("completed_frames", 0)
            report["generated_steps"] = partial.get("generated_steps", 0)
            report["interpreted_steps"] = partial.get("interpreted_steps", 0)
            report["partial_evidence"] = raw_report.relative_to(run.path).as_posix()
    elif not raw_report.is_file():
        report = {"status": "incomplete", "complete": False, "expected_frames": frames,
                  "completed_frames": 0, "reason": "Replay produced no structured completion evidence"}
    else:
        report = json.loads(raw_report.read_text())
        run.record_artifact(mode + "-parity-report", raw_report)
        selected = (run.path / inputs["input_evidence"]).read_bytes().splitlines(keepends=True)[:frames]
        expected = {"rom_sha256": inputs["rom_sha256"], "movie_sha256": inputs["movie_sha256"],
                    "input_sha256": inputs["input_sha256"], "binary_sha256": identity["sha256"],
                    "selected_input_sha256": hashlib.sha256(b"".join(selected)).hexdigest()}
        valid = (report.get("schema_version") == 1 and report.get("mode") == mode
                 and report.get("expected_frames") == frames and report.get("available_frames") == inputs["frames"]
                 and report.get("seed") == 0 and report.get("controller") == "P1"
                 and report.get("profile") == "lsnes-profile"
                 and report.get("regions") == ["CPU+clocks", "WRAM", "framebuffer"]
                 and all(report.get("identities", {}).get(name) == value for name, value in expected.items()))
        if mode == "generated":
            valid = valid and report.get("identities", {}).get("compiled_rom_sha256") == inputs["rom_sha256"]
        if not valid:
            report.update(status="error", complete=False, reason="Replay evidence identity or scope mismatch")
        elif report.get("status") == "pass" and not (
                result["returncode"] == 0 and report.get("complete") is True
                and report.get("completed_frames") == frames and report.get("matched_frames") == frames
                and (mode != "generated" or report.get("compiled_table_entries", 0) > 0
                     and report.get("generated_steps", 0) > 0)):
            report.update(status="incomplete", complete=False, reason="Successful replay did not complete its mandatory scope")
        report["evidence"] = raw_report.relative_to(run.path).as_posix()
    report["timings"] = timings
    report["total_seconds"] = time.monotonic() - started
    report["feedback_target_seconds"] = 1200
    report["feedback_target_overrun"] = timings.get("replay", 0) > 1200
    run.check_sources()
    coverage_path = output / "coverage.json"
    if mode == "generated" and coverage_path.is_file() and report.get("status") in ("pass", "divergence", "uncovered"):
        if report.get("coverage_sha256") != digest(coverage_path):
            raise RuntimeError("Instruction profile digest differs from replay report")
        coverage_identity = run.record_artifact("generated-instruction-profile", coverage_path)
        coverage_identity["evidence"] = coverage_path.relative_to(run.path).as_posix()
        run.save_manifest()
        gate_profile = json.loads(coverage_path.read_text())
        gate_profile.update(complete=report.get("complete") is True, expected_frames=frames,
                            completed_frames=report.get("completed_frames", 0),
                            candidate_inventory_sha256=run.manifest["identities"]["generated-key-inventory"]["sha256"])
        bound_profile = run.path / "checks/generated-profile-bound.json"
        write_json(bound_profile, gate_profile)
        if report.get("complete") is True:
            gate = evaluate_coverage(run, bound_profile, key_inventory, predecessor)
        else:
            gate = {"status": "incomplete", "complete": False, "reason": "Replay incomplete; instruction coverage is partial"}
            run.record_check("changed-entry-coverage", gate)
        report["coverage"] = "coverage.json"
        if report["status"] == "pass" and gate["status"] != "pass":
            report.update(status=gate["status"], reason=gate.get("reason"))
    elif mode == "generated" and report.get("status") == "pass":
        report.update(status="incomplete", complete=False, reason="Actual instruction coverage evidence is absent")
    run.record_check(mode + "-parity", report)
    return report


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parent.parent)
    parser.add_argument("--mode", choices=("fallback-only", "generated"), default="generated")
    parser.add_argument("--frames", type=int, default=18000)
    parser.add_argument("--timeout", type=float)
    parser.add_argument("--predecessor", type=Path)
    args = parser.parse_args(argv)
    parent_id = None
    if args.predecessor is not None:
        parent_id = args.predecessor.name
    run = Run.create(args.root, "parity", {"domain": "Japan/USA any%", "frames": args.frames,
                                          "mode": args.mode, "seed": 0, "controller": "P1"}, parent_id)
    print("Run: " + str(run.path), flush=True)
    print("Brief: " + str(run.path / "brief.md"), flush=True)
    try:
        failures = preflight(run)
        if failures:
            return run.finalize("error", "preflight", "; ".join(failures))
        report = run_parity(run, args.mode, args.frames, args.timeout, args.predecessor)
        status = report["status"]
        return run.finalize(status, args.mode + "-parity", report.get("reason", "Cold parity scope complete; other readiness gates remain pending"),
                            mandatory_checks=["preflight", args.mode + "-parity"])
    except KeyboardInterrupt:
        return run.finalize("incomplete", "parity", "Parity command interrupted")
    except Exception as error:
        return run.finalize("error", "parity", str(error))


if __name__ == "__main__":
    sys.exit(main())
