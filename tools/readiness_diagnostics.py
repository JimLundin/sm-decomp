#!/usr/bin/env python3
"""Identity-bound cold focused instruction diagnosis for a preserved candidate."""
import argparse
import json
import os
import time
from pathlib import Path
from readiness import Run,preflight
from readiness_parity import run_parity


def diagnose(run,start_input=0,instructions=100,mode="generated",timeout=None,start_instruction=0,cache_path=None):
    """Use a preserved parity candidate when available; otherwise build it in isolation."""
    started=time.monotonic()
    generated=run.path/"generated"/mode
    source=generated/"steps.rs"
    game=generated/"game.rs"
    if not source.exists():
        # Build/generation is shared with cold parity, using a minimal bounded check
        # only to prepare a candidate; its scope cannot certify readiness.
        report=run_parity(run,mode,1,timeout)
        if report["status"]!="pass":
            result={"status":report["status"],"complete":False,"reason":"Candidate preparation failed"}
            run.record_check("instruction-diagnosis",result);return result
    environment=dict(os.environ,CARGO_TARGET_DIR=str(run.path/"build"/mode),
                     SM_RECOMP_GENERATED_GAME=str(game),SM_RECOMP_GENERATED_STEPS=str(source))
    stage=run.run_stage("focused-build",["cargo","build","--release","-p","diff","--bin","focused-replay","--locked","--offline"],env=environment,timeout=timeout)
    if stage["status"]!="pass":
        result={"status":stage["status"],"complete":False,"reason":"Focused binary build failed"}
        run.record_check("instruction-diagnosis",result);return result
    binary=run.path/"build"/mode/"release/focused-replay"
    run.record_artifact("focused-binary",binary,"generated/focused-replay");candidate=run.path/"generated/focused-replay";run.guard_input(candidate)
    output=run.path/"diagnostics/instructions"
    command=[candidate,run.root/"movies/super-metroid-any-3653.lsmv","--out",output,"--start-input",str(start_input),"--start-instruction",str(start_instruction),"--instructions",str(instructions),"--rom",run.root/"rom/sm.sfc"]
    if mode=="fallback-only":command.append("--fallback-only")
    if cache_path is not None:
        import hashlib
        full=run.manifest["identities"]
        identities={key:full["inputs"][key] for key in ("rom_sha256","movie_sha256","input_sha256")}
        identities.update(source_sha256=full["source"]["sha256"],hardware_sha256=full["hardware"]["source_sha256"],
                          profile="lsnes-profile",serializer="paired-v1",compiler=full["compiler"]["version"],
                          features=full["hardware"]["features"],settings={"seed":0,"mode":mode,"candidate_sha256":hashlib.sha256(candidate.read_bytes()).hexdigest()})
        identity_path=run.path/"generated/checkpoint-identities.json";identity_path.write_text(json.dumps(identities,sort_keys=True));run.guard_input(identity_path)
        command.extend(["--cache",str(cache_path),"--identities",str(identity_path)])
    stage=run.run_stage("focused-replay",command,timeout=timeout,exit_statuses={1:"divergence",2:"error"})
    path=output/"instruction.json"
    if not path.is_file():result={"status":stage["status"] if stage["status"]!="pass" else "incomplete","complete":False,"reason":"No focused completion report"}
    else:
        result=json.loads(path.read_text());identities=run.manifest["identities"]["inputs"]
        valid=(result.get("schema_version")==1 and result.get("expected_instructions")==instructions
               and result.get("start_input_index")==start_input and result.get("replay")==("warm" if cache_path is not None else "cold")
               and result.get("start_instruction")==start_instruction
               and all(result.get("identities",{}).get(key)==identities[key] for key in ("rom_sha256","movie_sha256","input_sha256")))
        if cache_path is not None:
            cache=result.get("cache",{});validation=cache.get("validation",{})
            valid=valid and cache.get("status")=="pass" and cache.get("complete") is True and validation.get("completed_frames")==1 and validation.get("observed_rollovers")==2 and validation.get("complete") is True
        if not valid:result.update(status="error",complete=False,reason="Focused identity/scope mismatch")
        elif stage["status"] not in ("pass","divergence"):result.update(status=stage["status"],complete=False,reason=stage.get("reason"))
        elif result.get("status")=="pass" and (stage["returncode"]!=0 or result.get("completed_instructions")!=instructions or result.get("complete") is not True):result.update(status="incomplete",complete=False,reason="Focused scope did not complete")
        result["evidence"]=path.relative_to(run.path).as_posix()
        run.record_artifact("focused-report",path)
    if cache_path is not None and result.get("cache",{}).get("status")=="pass":
        for name in ("pair.json","validation.json","cache.json"):
            run.record_artifact("checkpoint-"+name,Path(cache_path)/name,"diagnostics/cache/"+name)
    result["total_seconds"]=time.monotonic()-started
    result["feedback_target_seconds"]=60
    result["feedback_target_overrun"]=result["total_seconds"]>60
    run.check_sources();run.record_check("instruction-diagnosis",result);return result


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument("--start-input",type=int,default=0)
    parser.add_argument("--instructions",type=int,default=100);parser.add_argument("--fallback-only",action="store_true")
    parser.add_argument("--start-instruction",type=int,default=0)
    parser.add_argument("--cache",type=Path);parser.add_argument("--timeout",type=float);args=parser.parse_args()
    run=Run.create(Path(__file__).resolve().parent.parent,"diagnostic",{"start_input":args.start_input,"instructions":args.instructions})
    print("Run: "+str(run.path),flush=True);print("Brief: "+str(run.path/"brief.md"),flush=True)
    try:
        failures=preflight(run)
        if failures:return run.finalize("error","preflight","; ".join(failures))
        report=diagnose(run,args.start_input,args.instructions,"fallback-only" if args.fallback_only else "generated",args.timeout,args.start_instruction,args.cache)
        return run.finalize(report["status"],"instruction-diagnosis",report.get("reason","Focused scope complete; readiness requires other gates"),["preflight","instruction-diagnosis"])
    except KeyboardInterrupt:return run.finalize("incomplete","instruction-diagnosis","Interrupted")
    except Exception as error:return run.finalize("error","instruction-diagnosis",str(error))


if __name__=="__main__":raise SystemExit(main())
