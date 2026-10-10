#!/usr/bin/env python3
"""Preserved failure experiments and matched diagnostic controls."""
import json
from pathlib import Path
from readiness import write_json, utc_now


def record_experiment(run, hypothesis, discriminator, result, discriminating=False, evidence=None):
    run.ensure_open()
    directory = run.path / "diagnostics/experiments"
    directory.mkdir(exist_ok=True)
    previous = [json.loads(path.read_text()) for path in sorted(directory.glob("*.json"))]
    record = {"schema_version": 1, "hypothesis": hypothesis, "predicted_discriminator": discriminator,
              "result": result, "new_discriminating_evidence": bool(discriminating), "evidence": evidence,
              "candidate_run_id": run.manifest["run_id"], "predecessor_run_id": run.manifest.get("accepted_parent"),
              "utc": utc_now()}
    path = directory / ("{:02d}.json".format(len(previous) + 1))
    if path.exists():
        raise RuntimeError("Experiment evidence cannot be overwritten")
    write_json(path, record)
    streak = 0
    for item in reversed(previous + [record]):
        if item["new_discriminating_evidence"]:
            break
        streak += 1
    report = dict(record, consecutive_without_evidence=streak, decision_required=streak >= 3,
                  choices=["Run an independent bus/IRQ hardware probe at the first mismatch",
                           "Review the preserved candidate and shared harness/cache assumptions",
                           "Defer this increment and retain the accepted predecessor"] if streak >= 3 else [])
    if report["decision_required"]:
        write_json(run.path / "diagnostics/decision.json", report)
        (run.path / "diagnostics/decision.md").write_text(
            "Decision required after three experiments without new evidence.\n\nCandidate: {}\nPredecessor: {}\n\n{}\n".format(
                report["candidate_run_id"], report["predecessor_run_id"], "\n".join("- " + choice for choice in report["choices"])))
    return report


def route_failure(failure, predecessor=None, disabled=None):
    if failure.get("stage") == "synchronization":
        return {"route": "independent hardware/input observation",
                "conclusion": "Shared-backend passing controls do not establish the recording core's hardware/input behavior",
                "next_experiment": "Capture original and candidate actual-fetch IRQ/DMA timing around the first checkpoint disagreement"}
    if failure.get("status") == "uncovered":
        return {"route": "coverage", "conclusion": "Required changed keys did not execute",
                "next_experiment": "Use a covered interval within the validated domain, or remove/defer unexecuted candidate keys"}
    passing = lambda report: report and report.get("status") == "pass" and report.get("complete") is True
    if passing(predecessor) and passing(disabled):
        return {"route": "increment", "conclusion": "Passing predecessor and disabled-entry controls supports attribution to the increment",
                "next_experiment": "Probe changed instruction semantics, width dispatch and ordered bus timing at the first disagreement"}
    return {"route": "shared paths/input/cache/harness", "conclusion": "At least one control failed or is incomplete; increment attribution is unresolved",
            "next_experiment": "Verify input/profile, preserved predecessor, cache fidelity and shared hardware at the first disagreement"}


def _predecessor_run(parent, predecessor):
    """Rebuild from intact preserved source in an isolated checkout."""
    import subprocess
    from readiness import Run, read_verdict, inventory_digest, digest
    predecessor=Path(predecessor)
    if read_verdict(predecessor)["status"]!="pass":
        raise RuntimeError("Selected predecessor must have a complete passing verdict")
    manifest=json.loads((predecessor/"manifest.json").read_text())
    inventory=json.loads((predecessor/"source/inventory.json").read_text())
    if inventory_digest(inventory["files"])!=manifest["identities"]["source"]["sha256"]:
        raise RuntimeError("Selected predecessor source inventory is corrupt")
    saved=Run(parent.root,predecessor,manifest)
    checkout=parent.path/"diagnostics/predecessor-checkout"
    saved.restore_source(checkout)
    subprocess.run(["git","init","-q",str(checkout)],check=True)
    subprocess.run(["git","-C",str(checkout),"add","."],check=True)
    subprocess.run(["git","-C",str(checkout),"-c","user.name=Readiness","-c","user.email=readiness@local.invalid",
                    "commit","-qm","Preserved predecessor source"],check=True)
    (checkout/"rom").symlink_to(parent.root/"rom",target_is_directory=True)
    (checkout/"deps").mkdir()
    for material in (parent.root/"deps").iterdir():
        if material.name!="rustysnes":
            (checkout/"deps"/material.name).symlink_to(material,target_is_directory=material.is_dir())
    subprocess.run(["python3",str(checkout/"tools/prepare_rustysnes.py"),"--offline"],cwd=checkout,check=True,
                   stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    for movie in (parent.root/"movies").iterdir():
        if movie.suffix in (".lsmv",".inputs"):
            (checkout/"movies"/movie.name).symlink_to(movie)
    (checkout/"out").mkdir(exist_ok=True)
    if (parent.root/"out/symbols.toml").exists():
        (checkout/"out/symbols.toml").symlink_to(parent.root/"out/symbols.toml")
    child=Run.create(checkout,"control",parent.manifest["scope"],manifest["run_id"])
    from readiness import preflight
    failures=preflight(child)
    if failures:raise RuntimeError("Predecessor control preflight failed: "+"; ".join(failures))
    if child.manifest["identities"]["source"]["sha256"]!=manifest["identities"]["source"]["sha256"]:
        raise RuntimeError("Rebuilt predecessor source identity differs from preserved snapshot")
    return child


def diagnose_failure(run, failure, predecessor=None, intervals=None, timeout=None):
    """Preserve candidate, disabled and predecessor controls over identical intervals."""
    from readiness_diagnostics import diagnose
    from readiness_parity import prepare_candidate
    result={"schema_version":1,"status":failure.get("status","incomplete"),"complete":False,
            "candidate_run_id":run.manifest["run_id"],"predecessor":str(predecessor) if predecessor else None}
    if failure.get("stage")=="synchronization" or failure.get("status")=="uncovered":
        result.update(route_failure(failure));result["complete"]=True
        write_json(run.path/"diagnostics/failure-controls.json",result);run.record_check("failure-controls",result);return result
    preparation=prepare_candidate(run,"generated",timeout)
    if preparation["status"]!="pass":
        result.update(status=preparation["status"],reason="Candidate preparation failed")
        run.record_check("failure-controls",result);return result
    coverage=run.checks.get("changed-entry-coverage",{})
    keys=[{"address":row["address"],"mx":row["mx"]} for row in coverage.get("required_keys",[])]
    if not keys and coverage.get("executed"):
        keys=[{"address":row["address"],"mx":row["mx"]} for row in coverage["executed"]]
    if not keys:
        inventory=json.loads((run.path/"generated/key-inventory.json").read_text())
        keys=[{"address":row["address"],"mx":row["mx"]} for row in inventory["keys"]]
    if intervals is None:
        from readiness_diagnostics import discover_intervals
        discovery=discover_intervals(run,keys,18000,timeout)
        if discovery["status"] not in ("pass","uncovered"):
            result.update(status=discovery["status"],complete=False,reason="Instruction discovery failed",discovery=discovery)
            run.record_check("failure-controls",result);return result
        if discovery["status"]=="uncovered":
            result.update(status="uncovered",complete=True,discovery=discovery,**route_failure({"status":"uncovered"}))
            run.record_check("failure-controls",result);return result
        intervals=discovery.get("intervals",[])
    if not intervals:
        result.update(status="uncovered",complete=True,**route_failure({"status":"uncovered"}))
        run.record_check("failure-controls",result);return result
    reports=[]
    child=None
    if predecessor:
        try:child=_predecessor_run(run,predecessor)
        except Exception as error:result["predecessor_error"]=str(error)
    for index,interval in enumerate(intervals):
        prefix="failure-{:02d}".format(index)
        args=(interval["start_input"],interval["instructions"],"generated",timeout,interval.get("start_instruction",0))
        candidate=diagnose(run,*args,cache_path=run.path/"cache"/(prefix+"-candidate"),namespace=prefix+"-candidate")
        disabled=diagnose(run,*args,cache_path=run.path/"cache"/(prefix+"-disabled"),namespace=prefix+"-disabled",disabled_keys=keys)
        old={"status":"incomplete","complete":False,"reason":result.get("predecessor_error","No preserved predecessor selected")}
        if child:
            old=diagnose(child,*args,cache_path=child.path/"cache"/(prefix+"-predecessor"),namespace=prefix+"-predecessor")
        reports.append({"interval":interval,"candidate":candidate,"disabled":disabled,"predecessor":old,
                        "routing":route_failure(failure,old,disabled)})
    if reports:
        route=reports[0]["routing"]
        record_experiment(run,"Candidate increment changes the first observed execution effect",
                          "Predecessor and disabled-entry controls both pass over the same input interval",
                          {"controls":reports,"routing":route},discriminating=route["route"]=="increment",
                          evidence=route["conclusion"])
    if child:
        child.finalize("pass",reason="Matched predecessor control scopes complete")
        result["predecessor_control_run"]=str(child.path)
    result.update(complete=all(all(control.get("complete") is True for control in (item["candidate"],item["disabled"],item["predecessor"])) for item in reports),controls=reports,routing=reports[0]["routing"] if reports else route_failure(failure))
    write_json(run.path/"diagnostics/failure-controls.json",result);run.record_check("failure-controls",result)
    return result
