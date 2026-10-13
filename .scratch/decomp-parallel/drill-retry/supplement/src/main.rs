//! Seals observed supplemental replay and repairs only unsealed final metadata.
use anyhow::{Result, ensure, Context};
use serde_json::{Value as J, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::{Path,PathBuf}, process::Command};
const STORE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry";
const FIXTURE: &str = "/tmp/sm-decomp-concurrency-drill-retry-20261013";
const B: &str = "01a13515-9a51-7af3-bc9f-a144afb06090";
fn read(path: &Path)->Result<J>{Ok(serde_json::to_value(toml::from_str::<toml::Value>(&fs::read_to_string(path)?)?)?)}
fn write(path: &Path,value: &J)->Result<()>{fs::write(path,toml::to_string_pretty(&toml::Value::try_from(value)?)?)?;Ok(())}
fn seal(path: &Path)->Result<J>{ensure!(path.is_absolute() && !path.is_symlink(),"absolute regular artifact required");Ok(json!({"path":path,"sha256":format!("{:x}",Sha256::digest(fs::read(path)?))}))}
fn time(value: &J)->Result<u64>{Ok(value["observed_at"].as_str().context("observation time missing")?.parse()?)}
fn files(root:&Path,map:&mut BTreeMap<String,String>)->Result<()>{ensure!(!root.is_symlink(),"symlink receipt");if root.is_dir(){for e in fs::read_dir(root)?{files(&e?.path(),map)?}}else{map.insert(root.display().to_string(),seal(root)?["sha256"].as_str().unwrap().into());}Ok(())}
fn git(args:&[&str])->Result<String>{let o=Command::new("git").args(args).current_dir(FIXTURE).output()?;ensure!(o.status.success(),"git observation failed");Ok(String::from_utf8(o.stdout)?.trim().into())}
fn normalized_launch(original:&Path,output:&Path)->Result<J>{let mut v=read(original)?;let prompt=Path::new(v["prompt"]["path"].as_str().unwrap());let absolute=if prompt.is_absolute(){prompt.to_path_buf()}else{Path::new("/home/jim/sm-decomp").join(prompt)};ensure!(seal(&absolute)?["sha256"]==v["prompt"]["sha256"],"prompt changed during path normalization");v["prompt"]["path"]=json!(absolute);write(output,&v)?;Ok(v)}
fn main()->Result<()>{
 let store=Path::new(STORE);let state=Path::new(FIXTURE).join(".scratch/drill-control");let supplement=store.join("supplement");
 let held_original=state.join("b-launch-1791896345.toml");let replay_original=state.join("b-launch-1791896413.toml");
 let held_path=supplement.join("b-held-launch-absolute.toml");let replay_path=supplement.join("b-replay-launch-absolute.toml");
 let held=normalized_launch(&held_original,&held_path)?;let replay=normalized_launch(&replay_original,&replay_path)?;
 ensure!(held["session_id"]==B && replay["session_id"]==B && held["pid"]==516393 && replay["pid"]==517456,"actual resumed session changed");
 let live_path=state.join("b-process-1791896361.toml");let signal_path=store.join("command-results/517253-1791896386941824499.toml");
 let stopped_path=state.join("b-process-1791896409.toml");let terminal_path=state.join("b-process-1791896527.toml");
 let live=read(&live_path)?;let signal=read(&signal_path)?;let stopped=read(&stopped_path)?;let terminal=read(&terminal_path)?;
 for observed in [&live,&signal]{ensure!(observed["status"]=="running" && observed["pid"]==held["pid"] && observed["start_ticks"]==held["process"]["start_ticks"] && observed["boot_id"]==held["process"]["boot_id"],"hold/signal process identity mismatch");}
 ensure!(stopped["status"]=="not-running" && stopped["pid"]==held["pid"] && terminal["status"]=="not-running" && terminal["pid"]==replay["pid"],"controller termination not observed");
 ensure!(time(&held)?<=time(&live)? && time(&live)?<=time(&signal)? && time(&signal)?<=time(&stopped)? && time(&stopped)?<time(&replay)? && time(&replay)?<time(&terminal)?,"interruption/replay chronology invalid");
 let held_log=PathBuf::from(held["log"].as_str().unwrap());let replay_log=PathBuf::from(replay["log"].as_str().unwrap());
 let held_events=fs::read_to_string(&held_log)?.lines().map(serde_json::from_str::<J>).collect::<std::result::Result<Vec<_>,_>>()?;
 let replay_events=fs::read_to_string(&replay_log)?.lines().map(serde_json::from_str::<J>).collect::<std::result::Result<Vec<_>,_>>()?;
 for events in [&held_events,&replay_events]{let starts:Vec<_>=events.iter().filter(|e|e["type"]=="thread.started").collect();ensure!(starts.len()==1 && starts[0]["thread_id"]==B,"actual launch JSONL start mismatch");}
 ensure!(held_events.iter().any(|e|e["item"]["command"].as_str().is_some_and(|x|x.contains("wait post-publication-hold-release 40")) && e["type"]=="item.started"),"held controller never entered bounded wait");
 ensure!(!state.join("post-publication-hold-release").exists(),"hold released instead of interrupted");
 ensure!(replay_events.iter().any(|e|e["type"]=="turn.completed"),"replay controller lacks terminal log");
 let snapshot_path=state.join("post-b-retry-receipts-before.toml");let snapshot=read(&snapshot_path)?;
 let before:BTreeMap<String,String>=snapshot["artifacts"].as_array().unwrap().iter().map(|v|(v["path"].as_str().unwrap().to_owned(),v["sha256"].as_str().unwrap().to_owned())).collect();
 let mut after=BTreeMap::new();files(&Path::new(FIXTURE).join(".scratch/concurrency-drill-retry/receipts"),&mut after)?;
 ensure!(before.len()==18 && before==after,"all18 maintained publications/gates/authentication must remain unchanged");
 let head=git(&["rev-parse","HEAD"])?;ensure!(head=="2b1400ef758907c9aee50f997bb9bca05216ec52" && snapshot["source_commit"]==head && git(&["status","--porcelain"])?.is_empty(),"shared source identity changed");
 let ledger=read(&Path::new(FIXTURE).join(".scratch/concurrency-drill-retry/ledger.toml"))?;for id in ["lane-a","lane-b","integrate-lane-a","integrate-lane-b"]{ensure!(ledger["jobs"][id]["state"]=="complete" && ledger["jobs"][id]["stale"]==false,"completed lineage changed");}
 let mut retries=Vec::new();
 for (id,new,original,previous_retry) in [("integrate-lane-a","integrated-exact-retry-517916-1791896425217207597","integrated-512471-1791893655590713716","integrated-exact-retry-512471-1791893669337460552"),("integrate-lane-b","integrated-exact-retry-519119-1791896457302036951","integrated-512471-1791893667278215825","integrated-exact-retry-512471-1791893671067873059")]{
  let new_req=state.join("receipts").join(format!("{new}.request.toml"));let old_req=state.join("receipts").join(format!("{original}.request.toml"));let command_path=state.join("receipts").join(format!("{new}.execution.toml"));let command=read(&command_path)?;let req=read(&new_req)?;
  ensure!(fs::read(&new_req)?==fs::read(&old_req)?,"completed retry must be byte-identical to original request");
  ensure!(command["exit_code"]==0 && command["session_id"]==B && req["queue"]["authority"]["session_id"]==B && req["queue"]["authority"]["generation"]==2 && req["queue"]["update"]["claim"]["id"]==id,"actual successful exact B generation2 retry missing");
  ensure!(time(&command)?>time(&stopped)? && time(&command)?<time(&terminal)?,"retry not after interruption");
  let old_command=read(&state.join("receipts").join(format!("{original}.execution.toml")))?;ensure!(time(&old_command)?<time(&held)?,"interruption not after existing durable publication");
  let previous_command_path=state.join("receipts").join(format!("{previous_retry}.execution.toml"));let previous_command=read(&previous_command_path)?;
  ensure!(command["stdout"]["sha256"]==previous_command["stdout"]["sha256"],"post-B exact retry response changed");
  retries.push(json!({"claim":id,"original_request":seal(&old_req)?,"original_publication_command":seal(&state.join("receipts").join(format!("{original}.execution.toml")))?,"prior_post_b_retry_command":seal(&previous_command_path)?,"exact_retry_request":seal(&new_req)?,"command":seal(&command_path)?,"request_bytes_identical":true,"post_b_retry_response_bytes_identical":true,"actual_generation":2}));
 }
 let controller_rollout=Path::new("/home/jim/.codex/sessions/2026/10/13/rollout-2026-10-13T14-56-16-01a13555-c722-7f00-9a28-d794ebf6ce9c.jsonl");
 let events=fs::read_to_string(controller_rollout)?.lines().map(serde_json::from_str::<J>).collect::<std::result::Result<Vec<_>,_>>()?;
 let call=events.iter().find(|e|e["type"]=="response_item" && e["payload"]["type"]=="function_call" && e["payload"]["arguments"].as_str().is_some_and(|a|a.contains("decomp-drill signal b"))).context("controller signal tool call missing")?;
 let call_id=call["payload"]["call_id"].clone();let output=events.iter().find(|e|e["payload"]["type"]=="function_call_output" && e["payload"]["call_id"]==call_id).context("controller signal tool output missing")?;
 let excerpt=supplement.join("controller-signal-events.jsonl");fs::write(&excerpt,format!("{}\n{}\n",serde_json::to_string(call)?,serde_json::to_string(output)?))?;
 let cleanup_paths=[store.join("final-cleanup1.toml"),store.join("final-cleanup2.toml")];for p in &cleanup_paths{ensure!(read(p)?["apply"]==true,"final scoped cleanup did not apply");}
 let proof_path=store.join("post-publication-interruption.toml");
 let proof=json!({"schema":"decomp-drill-post-publication-interruption-v1","status":"complete","actual_session_id":B,"actual_integrator_generation":2,"source_commit":head,"durable_publication_precedes_hold":true,"live_controller_interrupted_after_publication":true,"same_actual_uuid_resumed_after_observed_stop":true,"both_exact_replays_passed":true,"original_requests_byte_identical":true,"publication_and_gate_bytes_unchanged":true,"gate_reruns":0,"maintained_receipt_count":18,"original_receipt_snapshot":seal(&snapshot_path)?,"original_post_b_retry":seal(&store.join("post-b-retry.toml"))?,"hold":{"original_launch":seal(&held_original)?,"launch_absolute_prompt_metadata":seal(&held_path)?,"log":seal(&held_log)?,"stderr":seal(Path::new(held["stderr"].as_str().unwrap()))?,"observed_live":seal(&live_path)?,"checked_signal_result":seal(&signal_path)?,"controller_signal_events":seal(&excerpt)?,"signal":"SIGINT","observed_stopped":seal(&stopped_path)?},"replay":{"original_launch":seal(&replay_original)?,"launch_absolute_prompt_metadata":seal(&replay_path)?,"log":seal(&replay_log)?,"stderr":seal(Path::new(replay["stderr"].as_str().unwrap()))?,"observed_terminal":seal(&terminal_path)?,"retries":retries},"prior_thread_inspection": [seal(Path::new("/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/final-recovery-20261013/harness-thread-inspection.json"))?,seal(Path::new("/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/final-recovery-20261013/b-thread-before-replay.json"))?],"final_cleanup":cleanup_paths.iter().map(|p|seal(p)).collect::<Result<Vec<_>>>()?,"supplement_reader":{"source":seal(&supplement.join("src/main.rs"))?,"manifest":seal(&supplement.join("Cargo.toml"))?,"lock":seal(&supplement.join("Cargo.lock"))?,"binary":seal(&supplement.join("bin/decomp-drill-supplement"))?,"build_log":seal(&supplement.join("build.log"))?},"limitations":["Observed local process and actual Codex JSONL identities; no platform attestation.","Only the observed same B controller was signaled; no production/gameplay mutation.","Original new launch metadata retained. Normalized copies convert relative prompt paths to matching absolute artifacts without changing prompt bytes.","Retained hold stderr is a termination-time write_stdin unknown-process diagnostic; the controller was observed stopped before replay."]});
 write(&proof_path,&proof)?;
 let execution_path=store.join("execution.toml");let mut execution=read(&execution_path)?;
 for session in execution["sessions"].as_array_mut().context("sessions missing")?{let path=session["launch"]["path"].as_str().unwrap();let replacement=if path==held_original.to_str().unwrap(){Some(&held_path)}else if path==replay_original.to_str().unwrap(){Some(&replay_path)}else{None};if let Some(replacement)=replacement{session["launch"]=seal(replacement)?;}}
 for control in ["same-attempt-retry","crash-restart","integration-success"]{execution["controls"][control]["observations"].as_array_mut().unwrap().push(seal(&proof_path)?);}
 write(&execution_path,&execution)?;
 let drill_path=store.join("concurrency-drill.toml");let mut drill=read(&drill_path)?;drill["execution"]=seal(&execution_path)?;drill["evidence"]=json!([seal(&execution_path)?,seal(&store.join("post-b-retry.toml"))?,seal(&proof_path)?]);write(&drill_path,&drill)?;
 println!("{}",toml::to_string_pretty(&toml::Value::try_from(json!({"proof":seal(&proof_path)?,"execution":seal(&execution_path)?,"drill":seal(&drill_path)?}))?)?);Ok(())
}
