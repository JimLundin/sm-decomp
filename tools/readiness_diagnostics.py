#!/usr/bin/env python3
"""Identity-bound cold focused instruction diagnosis for a preserved candidate."""
import argparse
import json
import os
import time
from pathlib import Path
from readiness import Run,preflight
from readiness_parity import prepare_candidate


def diagnose(run,start_input=0,instructions=100,mode="generated",timeout=None,start_instruction=0,cache_path=None, namespace="", disabled_keys=None, until_frame_end=False):
    """Use a preserved parity candidate when available; otherwise build it in isolation."""
    started=time.monotonic()
    if until_frame_end and (cache_path is not None or start_instruction!=0):
        raise ValueError('Failed-frame localization requires a cold frame boundary without skipped instructions')
    prefix=(namespace+"-") if namespace else ""
    disabled_keys=sorted(disabled_keys or [], key=lambda key:(key["address"],key["mx"]))
    generated=run.path/"generated"/mode
    source=generated/"steps.rs"
    game=generated/"game.rs"
    if not source.exists():
        report=prepare_candidate(run,mode,timeout)
        if report["status"]!="pass":
            result={"status":report["status"],"complete":False,"reason":"Candidate preparation failed"}
            run.record_check(prefix+"instruction-diagnosis",result);return result
    environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/"build"/mode),
                     SM_DECOMP_GENERATED_GAME=str(game),SM_DECOMP_GENERATED_STEPS=str(source))
    focused_shared=run.path/"build"/mode/"release/focused-replay"
    stage={"status":"pass"}
    if not focused_shared.exists() or "focused-prepared-binary" not in run.manifest["identities"]:
        stage=run.run_stage(prefix+"focused-build",["cargo","build","--release","-p","diff","--bin","focused-replay","--locked","--offline"],env=environment,timeout=timeout)
    if stage["status"]!="pass":
        result={"status":stage["status"],"complete":False,"reason":"Focused binary build failed"}
        run.record_check(prefix+"instruction-diagnosis",result);return result
    binary=run.path/"build"/mode/"release/focused-replay"
    if "focused-prepared-binary" not in run.manifest["identities"]:
        run.record_artifact("focused-prepared-binary",binary,"generated/focused-prepared")
    else:
        from readiness import digest
        if digest(binary)!=run.manifest["identities"]["focused-prepared-binary"]["sha256"]:
            raise RuntimeError("Focused build binary changed after preparation")
    candidate=run.path/"generated"/(prefix+"focused-replay")
    if not candidate.exists():run.record_artifact(prefix+"focused-binary",binary,"generated/"+prefix+"focused-replay")
    else:run.record_artifact(prefix+"focused-binary",candidate)
    run.guard_input(candidate)
    output=run.path/"diagnostics"/(prefix+"instructions")
    command=[candidate,run.root/"movies/super-metroid-any-3653.lsmv","--out",output,"--start-input",str(start_input),"--start-instruction",str(start_instruction),"--instructions",str(instructions),"--rom",run.root/"rom/sm.sfc"]
    if until_frame_end:command.append('--until-frame-end')
    if mode=="fallback-only":command.append("--fallback-only")
    if disabled_keys:
        disabled_path=run.path/"generated"/(prefix+"disabled-keys.json")
        disabled_path.write_text(json.dumps(disabled_keys,sort_keys=True));run.guard_input(disabled_path)
        command.extend(["--disabled-keys",str(disabled_path)])
    if cache_path is not None:
        import hashlib
        full=run.manifest["identities"]
        identities={key:full["inputs"][key] for key in ("rom_sha256","movie_sha256","input_sha256")}
        identities.update(source_sha256=full["source"]["sha256"],hardware_sha256=full["hardware"]["source_sha256"],
                          profile="lsnes-profile",serializer="paired-v1",compiler=full["compiler"]["version"],
                          features=full["hardware"]["features"],settings={"seed":0,"mode":mode,"candidate_sha256":hashlib.sha256(candidate.read_bytes()).hexdigest(),"disabled_keys":disabled_keys})
        identity_path=run.path/"generated"/(prefix+"checkpoint-identities.json");identity_path.write_text(json.dumps(identities,sort_keys=True));run.guard_input(identity_path)
        command.extend(["--cache",str(cache_path),"--identities",str(identity_path)])
    stage=run.run_stage(prefix+"focused-replay",command,timeout=timeout,exit_statuses={1:"divergence",2:"error"})
    path=output/"instruction.json"
    if not path.is_file():result={"status":stage["status"] if stage["status"]!="pass" else "incomplete","complete":False,"reason":"No focused completion report"}
    else:
        result=json.loads(path.read_text());identities=run.manifest["identities"]["inputs"]
        valid=(result.get("schema_version")==1 and (result.get("expected_instructions")==instructions if not until_frame_end else result.get('scope')=='until-frame-end' and result.get('expected_instructions') is None and result.get('instruction_budget')==2000000)
               and result.get("start_input_index")==start_input and result.get("replay")==("warm" if cache_path is not None else "cold")
               and result.get("start_instruction")==start_instruction
               and result.get("binary_sha256")==__import__("hashlib").sha256(candidate.read_bytes()).hexdigest()
               and result.get("disabled_keys")==disabled_keys
               and all(result.get("identities",{}).get(key)==identities[key] for key in ("rom_sha256","movie_sha256","input_sha256")))
        if cache_path is not None:
            cache=result.get("cache",{});validation=cache.get("validation",{})
            valid=valid and cache.get("status")=="pass" and cache.get("complete") is True and validation.get("completed_frames")==1 and validation.get("observed_rollovers")==2 and validation.get("complete") is True
        if not valid:result.update(status="error",complete=False,reason="Focused identity/scope mismatch")
        elif stage["status"] not in ("pass","divergence"):result.update(status=stage["status"],complete=False,reason=stage.get("reason"))
        elif result.get("status")=="pass" and (stage["returncode"]!=0 or (result.get("completed_instructions")!=instructions if not until_frame_end else result.get('observed_rollovers')!=1) or result.get("complete") is not True):result.update(status="incomplete",complete=False,reason="Focused scope did not complete")
        result["evidence"]=path.relative_to(run.path).as_posix()
        run.record_artifact(prefix+"focused-report",path)
    if cache_path is not None and result.get("cache",{}).get("status")=="pass":
        for name in ("pair.json","validation.json","cache.json"):
            run.record_artifact(prefix+"checkpoint-"+name,Path(cache_path)/name,"diagnostics/"+prefix+"cache/"+name)
    result["disabled_keys"]=disabled_keys
    result["warm_target_overrun"]=result.get("warm_replay_seconds",0)>60
    result["total_seconds"]=time.monotonic()-started
    result["feedback_target_seconds"]=60
    result["feedback_target_overrun"]=result["total_seconds"]>60
    run.check_sources();run.record_check(prefix+"instruction-diagnosis",result);return result


def discover_intervals(run,keys,frames=18000,timeout=None):
    preparation=prepare_candidate(run,"generated",timeout)
    if preparation["status"]!="pass":return preparation
    environment=dict(os.environ,CARGO_TARGET_DIR=str(preparation["build"]),SM_DECOMP_GENERATED_GAME=str(preparation["game"]),SM_DECOMP_GENERATED_STEPS=str(preparation["steps"]))
    binary=preparation["build"]/"release/focused-replay"
    if not binary.exists() or "focused-prepared-binary" not in run.manifest["identities"]:
        stage=run.run_stage("discovery-build",["cargo","build","--release","-p","diff","--bin","focused-replay","--locked","--offline"],env=environment,timeout=timeout)
        if stage["status"]!="pass":return stage
    if "focused-prepared-binary" not in run.manifest["identities"]:
        run.record_artifact("focused-prepared-binary",binary,"generated/focused-prepared")
    frozen=run.path/"generated/discovery-replay"
    if not frozen.exists():run.record_artifact("discovery-binary",binary,"generated/discovery-replay")
    run.guard_input(frozen)
    keys_path=run.path/"generated/discovery-keys.json";keys_path.write_text(json.dumps(keys,sort_keys=True));run.guard_input(keys_path)
    output=run.path/"diagnostics/discovery"
    stage=run.run_stage("instruction-discovery-process",[frozen,run.root/"movies/super-metroid-any-3653.lsmv","--discover-keys",keys_path,"--frames",str(frames),"--rom",run.root/"rom/sm.sfc","--out",output],timeout=timeout,exit_statuses={1:"uncovered",2:"error"})
    if not (output/"discovery.json").exists():return dict(stage,complete=False)
    report=json.loads((output/"discovery.json").read_text());run.record_artifact("instruction-discovery-report",output/"discovery.json")
    import hashlib
    valid=report.get("schema_version")==1 and report.get("expected_frames")==frames and report.get("keys")==keys and report.get("complete") is True
    valid=valid and report.get("binary_sha256")==hashlib.sha256(frozen.read_bytes()).hexdigest()
    valid=valid and all(report.get("identities",{}).get(key)==run.manifest["identities"]["inputs"][key] for key in ("rom_sha256","movie_sha256","input_sha256"))
    if not valid:report.update(status="error",complete=False,reason="Discovery identity/scope mismatch")
    elif stage["status"] not in ("pass","uncovered"):report.update(status=stage["status"],complete=False,reason=stage.get("reason"))
    run.record_check("instruction-discovery",report)
    return report


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--start-input",type=int,default=0)
    parser.add_argument("--instructions",type=int,default=100);parser.add_argument("--fallback-only",action="store_true")
    parser.add_argument("--start-instruction",type=int,default=0)
    parser.add_argument('--until-frame-end',action='store_true')
    parser.add_argument("--cache",type=Path);parser.add_argument("--timeout",type=float)
    parser.add_argument("--failure",type=Path);parser.add_argument("--predecessor",type=Path);args=parser.parse_args()
    run=Run.create(Path(__file__).resolve().parent.parent,"diagnostic",{"start_input":args.start_input,"instructions":args.instructions})
    print("Run: "+str(run.path),flush=True);print("Brief: "+str(run.path/"brief.md"),flush=True)
    try:
        failures=preflight(run)
        if failures:return run.finalize("error","preflight","; ".join(failures))
        if args.failure is not None:
            from readiness_controls import diagnose_failure
            report=diagnose_failure(run,json.loads(args.failure.read_text()),args.predecessor,
                [{"start_input":args.start_input,"start_instruction":args.start_instruction,"instructions":args.instructions}],args.timeout)
            return run.finalize(report["status"],"failure-controls",report.get("routing",{}).get("conclusion","Failure evidence preserved"))
        report=diagnose(run,args.start_input,args.instructions,"fallback-only" if args.fallback_only else "generated",args.timeout,args.start_instruction,args.cache,until_frame_end=args.until_frame_end)
        return run.finalize(report["status"],"instruction-diagnosis",report.get("reason","Focused scope complete; readiness requires other gates"),["preflight","instruction-diagnosis"])
    except KeyboardInterrupt:return run.finalize("incomplete","instruction-diagnosis","Interrupted")
    except Exception as error:return run.finalize("error","instruction-diagnosis",str(error))


if __name__=="__main__":raise SystemExit(main())
