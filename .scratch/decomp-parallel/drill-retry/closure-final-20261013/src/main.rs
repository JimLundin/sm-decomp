use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, env, fs, path::{Path, PathBuf}, process::Command, time::{SystemTime, UNIX_EPOCH}};

const STORE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry";
const FIXTURE: &str = "/tmp/sm-decomp-concurrency-drill-retry-20261013";
const B: &str = "01a13515-9a51-7af3-bc9f-a144afb06090";

fn parse(path: &Path) -> Result<Value> {
    Ok(serde_json::to_value(toml::from_str::<toml::Value>(&fs::read_to_string(path)?)?)?)
}
fn hash(path: &Path) -> Result<String> {
    ensure!(!fs::symlink_metadata(path)?.file_type().is_symlink(), "symlink: {}", path.display());
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
fn seal(path: &Path) -> Result<Value> { Ok(json!({"path": path, "sha256": hash(path)?})) }
fn inventory(path: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    ensure!(!fs::symlink_metadata(path)?.file_type().is_symlink(), "inventory symlink");
    if path.is_dir() {
        for entry in fs::read_dir(path)? { inventory(&entry?.path(), result)?; }
    } else { result.insert(path.display().to_string(), hash(path)?); }
    Ok(())
}
fn write(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, toml::to_string_pretty(&toml::Value::try_from(value)?)?)?;
    Ok(())
}
fn main() -> Result<()> {
    let store = Path::new(STORE);
    let closure = store.join("closure-final-20261013");
    let state = Path::new(FIXTURE).join(".scratch/drill-control");
    let receipts = state.join("receipts");
    let proof_path = store.join("post-publication-interruption.toml");
    ensure!(!proof_path.exists(), "final supplemental proof is immutable once written");
    let hold_launch = state.join("b-launch-1791896345.toml");
    let replay_launch = state.join("b-launch-1791896413.toml");
    let held = parse(&hold_launch)?;
    let replay = parse(&replay_launch)?;
    ensure!(held["session_id"] == B && replay["session_id"] == B, "wrong resumed B UUID");
    ensure!(held["pid"] == 516393 && held["process"]["start_ticks"] == 3126349, "wrong held process");
    ensure!(replay["pid"] == 517456 && replay["process"]["start_ticks"] == 3133121, "wrong replay process");
    ensure!(held["process"]["boot_id"] == replay["process"]["boot_id"], "boot identity changed");
    let live_path = state.join("b-process-1791896361.toml");
    let stopped_path = state.join("b-process-1791896409.toml");
    let terminal_path = state.join("b-process-1791896527.toml");
    let live = parse(&live_path)?;
    let stopped = parse(&stopped_path)?;
    let terminal = parse(&terminal_path)?;
    ensure!(live["pid"] == held["pid"] && live["status"] == "running"
        && live["start_ticks"] == held["process"]["start_ticks"]
        && live["boot_id"] == held["process"]["boot_id"], "held process identity changed");
    ensure!(stopped["pid"] == held["pid"] && stopped["status"] == "not-running", "missing observed interruption");
    ensure!(terminal["pid"] == replay["pid"] && terminal["status"] == "not-running", "replay not terminal");
    ensure!(1791893669 < 1791896361 && 1791896386 < 1791896409 && 1791896409 < 1791896413, "invalid order");
    for pid in [516393, 517456, 517220] { ensure!(!Path::new(&format!("/proc/{pid}")).exists(), "drill writer still exists: {pid}"); }
    ensure!(!state.join("post-publication-hold-release").exists(), "hold was released normally");
    let signal_event_path = closure.join("signal-controller-event.jsonl");
    let signal_events = fs::read_to_string(&signal_event_path)?;
    ensure!(signal_events.lines().count() == 1, "expected exact signal event");
    let signal_event: Value = serde_json::from_str(signal_events.trim())?;
    let signal_item = &signal_event["payload"]["item"];
    ensure!(signal_event["payload"]["thread_id"] == "01a13555-c722-7f00-9a28-d794ebf6ce9c"
        && signal_item["exit_code"] == 0
        && signal_item["command"].as_array().context("missing signal argv")?.iter().any(|s| s.as_str().is_some_and(|s| s.contains("decomp-drill signal b")))
        && signal_item["aggregated_output"].as_str().context("missing signal output")?.contains("517253-1791896386941824499.toml"), "signal tool event mismatch");
    for (launch, terminal_required) in [(&held, false), (&replay, true)] {
        let log = fs::read_to_string(launch["log"].as_str().context("log missing")?)?;
        let events = log.lines().map(serde_json::from_str::<Value>).collect::<std::result::Result<Vec<_>, _>>()?;
        ensure!(events[0]["type"] == "thread.started" && events[0]["thread_id"] == B, "actual resumed launch differs");
        if terminal_required { ensure!(events.last().context("empty replay log")?["type"] == "turn.completed", "replay log not complete"); }
        else { ensure!(events.last().context("empty held log")?["type"] == "item.started", "held log did not end in interrupted wait"); }
    }
    let snapshot_path = state.join("post-b-retry-receipts-before.toml");
    let snapshot = parse(&snapshot_path)?;
    let mut before = BTreeMap::new();
    for item in snapshot["artifacts"].as_array().context("snapshot missing")? {
        ensure!(before.insert(item["path"].as_str().context("snapshot path")?.to_owned(), item["sha256"].as_str().context("snapshot hash")?.to_owned()).is_none(), "duplicate snapshot path");
    }
    let mut after = BTreeMap::new();
    inventory(&Path::new(FIXTURE).join(".scratch/concurrency-drill-retry/receipts"), &mut after)?;
    ensure!(before == after && before.len() == 18, "maintained receipt bytes/paths changed");
    let ledger_path = Path::new(FIXTURE).join(".scratch/concurrency-drill-retry/ledger.toml");
    let ledger = parse(&ledger_path)?;
    for id in ["lane-a", "lane-b", "integrate-lane-a", "integrate-lane-b"] {
        ensure!(ledger["jobs"][id]["state"] == "complete" && ledger["jobs"][id]["stale"] == false, "job incomplete: {id}");
    }
    let mut pairs = Vec::new();
    for (id, original, retried) in [
        ("integrate-lane-a", "integrated-512471-1791893655590713716", "integrated-exact-retry-517916-1791896425217207597"),
        ("integrate-lane-b", "integrated-512471-1791893667278215825", "integrated-exact-retry-519119-1791896457302036951"),
    ] {
        let original_path = receipts.join(format!("{original}.execution.toml"));
        let retry_path = receipts.join(format!("{retried}.execution.toml"));
        let original = parse(&original_path)?;
        let retry = parse(&retry_path)?;
        ensure!(original["exit_code"] == 0 && retry["exit_code"] == 0 && retry["session_id"] == B, "failed/wrong retry");
        ensure!(original["request"]["sha256"] == retry["request"]["sha256"], "replay differs from durable publication request");
        let request = parse(Path::new(retry["request"]["path"].as_str().context("retry request missing")?))?;
        ensure!(request["queue"]["authority"]["session_id"] == B && request["queue"]["authority"]["generation"] == 2
            && request["queue"]["update"]["claim"]["id"] == id, "wrong exact authority/claim");
        pairs.push(json!({"claim_id":id,"original_command":seal(&original_path)?,"replay_command":seal(&retry_path)?,"request_bytes_identical":true}));
    }
    let head = Command::new("git").arg("-C").arg(FIXTURE).args(["rev-parse", "HEAD"]).output()?;
    ensure!(head.status.success(), "fixture HEAD unavailable");
    let head = String::from_utf8(head.stdout)?.trim().to_owned();
    ensure!(head == "2b1400ef758907c9aee50f997bb9bca05216ec52" && snapshot["source_commit"] == head, "fixture source changed");
    let worktrees = Command::new("git").arg("-C").arg(FIXTURE).args(["worktree", "list", "--porcelain"]).output()?;
    ensure!(worktrees.status.success() && String::from_utf8(worktrees.stdout)?.lines().filter(|s| s.starts_with("worktree ")).count() == 1, "implementer worktree remains");
    let proof = json!({
        "schema":"decomp-drill-post-publication-interruption-v1", "status":"pass",
        "observer_thread":env::var("CODEX_THREAD_ID").context("actual observer identity missing")?,
        "observed_at":SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs().to_string(),
        "session_id":B, "source_commit":head, "post_publication_controller_interrupted":true,
        "same_uuid_resumed":true, "exact_a_retry_passed":true, "exact_b_retry_passed":true,
        "publication_and_gate_bytes_unchanged":true, "maintained_receipt_count":18, "gate_reruns":0,
        "registered_implementer_worktrees":0,
        "held_launch":seal(&hold_launch)?, "held_log":seal(Path::new(held["log"].as_str().unwrap()))?,
        "held_stderr":seal(Path::new(held["stderr"].as_str().unwrap()))?, "live_observation":seal(&live_path)?,
        "signal_tool_event":seal(&signal_event_path)?,
        "signal_result":seal(&store.join("command-results/517253-1791896386941824499.toml"))?,
        "stopped_observation":seal(&stopped_path)?, "replay_launch":seal(&replay_launch)?,
        "replay_log":seal(Path::new(replay["log"].as_str().unwrap()))?,
        "replay_stderr":seal(Path::new(replay["stderr"].as_str().unwrap()))?,
        "terminal_observation":seal(&terminal_path)?, "receipt_snapshot":seal(&snapshot_path)?,
        "ledger_after":seal(&ledger_path)?, "retry_pairs":pairs,
        "cleanup_receipts":[seal(&closure.join("fixture-cleanup1.toml"))?,seal(&closure.join("fixture-cleanup2.toml"))?],
        "closure_reader":{"source":seal(&closure.join("src/main.rs"))?,"manifest":seal(&closure.join("Cargo.toml"))?,"lock":seal(&closure.join("Cargo.lock"))?,"binary":seal(&env::current_exe()?)?},
        "limitations":["Local source/build/hash identity is not platform attestation.","The interrupted held log ends during its bounded wait; exact stopped process observation establishes termination.","Infrastructure drill acceptance does not grant production activation or gameplay verification."]
    });
    write(&proof_path, &proof)?;
    let outer_path = store.join("concurrency-drill.toml");
    let mut outer = parse(&outer_path)?;
    outer["evidence"].as_array_mut().context("outer evidence missing")?.push(seal(&proof_path)?);
    write(&outer_path, &outer)?;
    println!("{}", serde_json::to_string_pretty(&json!({"status":"pass","proof":seal(&proof_path)?,"drill":seal(&outer_path)?,"maintained_receipts":18,"gate_reruns":0}))?);
    Ok(())
}
