//! Retain exact observed drill artifacts without changing the frozen subject.
use anyhow::{Context, Result, ensure};
use serde_json::{Value as J, json};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
const ROOT: &str = "/tmp/sm-decomp-concurrency-drill-retry-20261013";
const STORE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry";
fn state() -> PathBuf {
    Path::new(ROOT).join(".scratch/drill-control")
}
fn read(p: &Path) -> Result<J> {
    Ok(serde_json::to_value(toml::from_str::<toml::Value>(&fs::read_to_string(p)?)?)?)
}
fn write(p: &Path, v: &J) -> Result<()> {
    fs::create_dir_all(p.parent().unwrap())?;
    fs::write(p, toml::to_string_pretty(&toml::Value::try_from(v)?)?)?;
    Ok(())
}
fn seal(p: &Path) -> Result<J> {
    Ok(json!({"path":p,"sha256":format!("{:x}",Sha256::digest(fs::read(p)?))}))
}
fn now() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs().to_string()
}
fn identity() -> Result<String> {
    let id = std::env::var("CODEX_THREAD_ID").context("actual Codex caller required")?;
    ensure!(id.len() == 36, "actual identity required");
    Ok(id)
}
fn process(pid: u32) -> Result<J> {
    let dir = PathBuf::from(format!("/proc/{pid}"));
    let stat = fs::read_to_string(dir.join("stat"))?;
    let fields: Vec<_> = stat.rsplit_once(')').unwrap().1.split_whitespace().collect();
    let argv: Vec<String> = fs::read(dir.join("cmdline"))?
        .split(|b| *b == 0)
        .filter(|b| !b.is_empty())
        .map(|b| String::from_utf8_lossy(b).into())
        .collect();
    Ok(
        json!({"pid":pid,"boot_id":fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim(),"start_ticks":fields[19].parse::<u64>()?,"kernel_state":fields[0],"argv":argv,"cwd":fs::read_link(dir.join("cwd"))?,"observed_at":now(),"status":"running"}),
    )
}
fn prepare_ticket() -> Result<J> {
    let held = read(&Path::new(ROOT).join(".scratch/concurrency-drill-retry/ledger.toml"))?;
    let holder = &held["jobs"]["lane-b"];
    ensure!(
        ["reserved", "running", "submitted"].contains(&holder["state"].as_str().unwrap_or_default())
            && holder["stale"] == false,
        "exact B holder required"
    );
    let mut scope = read(&state().join("scope-duplicate-ticket.toml"))?;
    scope["job"]["ticket"] = holder["job"]["ticket"].clone();
    scope["job"]["writable_paths"] = json!(["docs/drill-duplicate.md"]);
    let path = state().join("scope-ticket-disjoint.toml");
    ensure!(!path.exists(), "immutable competing scope already exists");
    write(&path, &scope)?;
    let mut scopes = read(&state().join("scopes.toml"))?;
    for entry in scopes["scopes"].as_array_mut().unwrap() {
        if entry["path"] == state().join("scope-duplicate-ticket.toml").display().to_string() {
            *entry = seal(&path)?;
        }
    }
    write(&state().join("scopes.toml"), &scopes)?;
    let observation = state().join("ticket-disjoint-holder.toml");
    write(
        &observation,
        &json!({"schema":"decomp-drill-ticket-holder-v1","claim":holder["claim"],"job":holder["job"],"state":holder["state"],"stale":holder["stale"],"candidate_scope":seal(&path)?,"observed_at":now()}),
    )?;
    seal(&observation)
}
fn pure_next(token: &str, generation: u64, overlap: bool) -> Result<J> {
    next_check(token,generation,if overlap {"overlap"}else{"ticket"})
}
fn next_check(token: &str, generation: u64, expected: &str) -> Result<J> {
    let id = identity()?;
    let req = json!({"operation":"infrastructure-next","queue":{"operation":"next","checkout":ROOT,"effort":"concurrency-drill-retry","coordinator":token,"capacity":3,"available_slots":0,"authority":{"session_id":id,"generation":generation}},"admission":{"domain":"infrastructure","scopes":read(&state().join("scopes.toml"))?["scopes"]}});
    let n = format!(
        "ticket-disjoint-next-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    let dir = state().join("receipts");
    let request = dir.join(format!("{n}.request.toml"));
    write(&request, &req)?;
    let engine = fs::read_to_string(state().join("engine-path"))?;
    let child = Command::new(engine.trim())
        .arg("decomp-session")
        .current_dir(ROOT)
        .env("SM_PIPELINE_ROOT", format!("{ROOT}/out/pipeline"))
        .env("SM_PIPELINE_WORKERS", "4")
        .stdin(File::open(&request)?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let observed = process(child.id())?;
    let out = child.wait_with_output()?;
    let stdout = dir.join(format!("{n}.stdout"));
    let stderr = dir.join(format!("{n}.stderr"));
    fs::write(&stdout, &out.stdout)?;
    fs::write(&stderr, &out.stderr)?;
    let receipt = dir.join(format!("{n}.execution.toml"));
    write(
        &receipt,
        &json!({"schema":"decomp-drill-command-v1","session_id":id,"pid":observed["pid"],"process":observed,"route":"decomp-session","request":seal(&request)?,"stdout":seal(&stdout)?,"stderr":seal(&stderr)?,"exit_code":out.status.code().unwrap_or(-1),"observed_at":now()}),
    )?;
    ensure!(out.status.success(), "competing next failed: {}", String::from_utf8_lossy(&out.stderr));
    let output = read(&stdout)?;
    if expected == "overlap" {
        ensure!(output.get("assignment").is_none()
            && output["deferred"]["control-overlap"].as_str().is_some_and(|reason| reason.contains("writable ownership")),
            "need exact writable ownership deferral");
    } else if expected == "ticket" {
        ensure!(output.get("assignment").is_none() && output["deferred"]["control-duplicate-ticket"] == "ticket is claimed",
            "need exact pure duplicate-ticket rejection");
    } else {
        ensure!(output.get("assignment").is_none(),"seed control cannot reserve work");
    }
    Ok(json!({"command":seal(&receipt)?,"output":output}))
}
fn pure_controls(token: &str, generation: u64) -> Result<J> {
    next_check(token,generation,"overlap")?;
    prepare_ticket()?;
    ticket_seal(true)?;
    next_check(token,generation,"seed")?;
    ticket_seal(false)?;
    let result = next_check(token,generation,"ticket")?;
    write(&state().join("pure-controls-complete.toml"),&json!({"schema":"decomp-drill-pure-controls-v1","status":"complete","command":result["command"],"observed_at":now()}))?;
    Ok(result)
}
fn commands() -> Result<Vec<(PathBuf, J, J, J)>> {
    let mut result = Vec::new();
    for entry in fs::read_dir(state().join("receipts"))? {
        let p = entry?.path();
        if !p.file_name().unwrap().to_string_lossy().ends_with(".execution.toml") {
            continue;
        }
        let receipt = read(&p)?;
        if !["decomp-authority", "decomp-session"].contains(&receipt["route"].as_str().unwrap_or_default()) {
            continue;
        }
        let request = read(Path::new(receipt["request"]["path"].as_str().unwrap()))?;
        let out = read(Path::new(receipt["stdout"]["path"].as_str().unwrap())).unwrap_or(J::Null);
        result.push((p, receipt, request, out));
    }
    result.sort_by(|a, b| a.1["observed_at"].as_str().cmp(&b.1["observed_at"].as_str()).then(a.0.cmp(&b.0)));
    Ok(result)
}
fn start_request(claim: &str) -> Result<J> {
    let mut requests = fs::read_dir(state().join("receipts"))?
        .filter_map(|entry| entry.ok()).map(|entry| entry.path())
        .filter(|path| path.file_name().is_some_and(|name| name.to_string_lossy().starts_with("start-"))
            && path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".request.toml")))
        .collect::<Vec<_>>();
    requests.sort();
    let mut req = requests.iter().rev().find_map(|path| {
        let value = read(path).ok()?;
        (value["handoff"]["claim"]["id"] == claim).then_some(value)
    }).context("exact original integration Start request missing")?;
    let ledger = read(&Path::new(ROOT).join(".scratch/concurrency-drill-retry/ledger.toml"))?;
    let id = req["handoff"]["job"]["inputs"]["submission-id"].as_str().context("submission id missing")?;
    req["handoff"]["submitted_commit"] = ledger["jobs"][id]["result"]["commit"].clone();
    ensure!(req["handoff"]["submitted_commit"].as_str().is_some_and(|commit| commit.len() == 40), "full submitted commit required");
    req["queue"]["authority"]["session_id"] = json!(identity()?);
    let path = state().join(format!("{claim}-corrected-start.request.toml"));
    ensure!(!path.exists(), "corrected Start request already sealed");
    write(&path, &req)?;
    Ok(json!({"request":seal(&path)?}))
}
fn ticket_seal(seed: bool) -> Result<J> {
    let disjoint = state().join("scope-ticket-disjoint.toml");
    let target = if seed { state().join("scope-ticket-seed.toml") } else { disjoint.clone() };
    if seed {
        let mut candidate = read(&disjoint)?;
        candidate["job"]["ticket"] = json!(".scratch/concurrency-drill-retry/issues/overlap.md");
        if target.exists() {
            ensure!(read(&target)? == candidate, "existing seed scope differs");
        } else {
            write(&target, &candidate)?;
        }
    } else {
        let ledger = read(&Path::new(ROOT).join(".scratch/concurrency-drill-retry/ledger.toml"))?;
        let candidate = &ledger["jobs"]["control-duplicate-ticket"];
        ensure!(candidate["state"] == "ready" && candidate["attempts"] == 0
            && candidate.get("claim").is_none(), "exact never-launched seeded candidate required");
    }
    let mut list = read(&state().join("scopes.toml"))?;
    let mut matches = 0;
    for artifact in list["scopes"].as_array_mut().context("scopes missing")? {
        let path = Path::new(artifact["path"].as_str().context("scope path missing")?);
        if read(path)?["job"]["id"] == "control-duplicate-ticket" {
            *artifact = seal(&target)?;
            matches += 1;
        }
    }
    ensure!(matches == 1, "exactly one duplicate-ticket candidate seal required");
    write(&state().join("scopes.toml"), &list)?;
    Ok(json!({"scope":seal(&target)?,"operation":if seed {"seed-ticket"} else {"switch-ticket"}}))
}
fn remember(claim: &str) -> Result<J> {
    let mut receipts = commands()?.into_iter().filter(|(_, receipt, request, _)| {
        receipt["exit_code"] == 0 && request["operation"] == "start" && request["handoff"]["claim"]["id"] == claim
    }).collect::<Vec<_>>();
    receipts.sort_by_key(|(_, receipt, _, _)| receipt["observed_at"].as_str().unwrap_or("").to_owned());
    let (_, _, _, result) = receipts.last().context("successful corrected Start missing")?;
    write(&state().join(format!("handoff-{claim}.toml")), result)?;
    Ok(result.clone())
}
fn manifest() -> Result<J> {
    let post_b = Path::new(STORE).join("post-b-retry.toml");
    let retry = read(&post_b)?;
    ensure!(retry["exact_a_retry_passed"] == true && retry["exact_b_retry_passed"] == true
        && retry["publication_and_gate_bytes_unchanged"] == true && retry["gate_reruns"] == 0,
        "fresh actual post-B exact integrated retries and immutable receipt bytes required");
    let mut sessions = Vec::new();
    for entry in fs::read_dir(state())? {
        let p = entry?.path();
        let name = p.file_name().unwrap().to_string_lossy();
        if !name.contains("-launch-") || p.extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let launch = read(&p)?;
        if launch["schema"] != "decomp-drill-session-launch-v1" {
            continue;
        }
        let log = Path::new(launch["log"].as_str().unwrap());
        let stderr = Path::new(launch["stderr"].as_str().unwrap());
        sessions.push(json!({"launch":seal(&p)?,"log":seal(log)?,"stderr":seal(stderr)?}));
    }
    sessions.sort_by_key(|s| s["launch"]["path"].as_str().unwrap().to_owned());
    let all = commands()?;
    let mut controls = serde_json::Map::new();
    for name in [
        "independent-sessions",
        "writable-overlap",
        "duplicate-ticket",
        "crash-restart",
        "surviving-child",
        "stopped-transfer",
        "stale-authority",
        "same-attempt-retry",
        "shared-capacity",
        "cleanup",
        "integration-success",
    ] {
        let mut selected = Vec::new();
        for (p, r, q, o) in &all {
            let op = q["operation"].as_str().unwrap_or_default();
            let good = r["exit_code"] == 0;
            let next = op == "infrastructure-next";
            let assignment = o.get("assignment");
            let record = op == "infrastructure-record";
            let result = q["queue"]["update"]["result"]["result"].as_str().unwrap_or_default();
            let deferred = o["deferred"].as_object();
            let matches = match name {
                "independent-sessions" => {
                    good && next
                        && assignment.is_some_and(|a| {
                            ["lane-a", "lane-b"].contains(&a["claim"]["id"].as_str().unwrap_or_default())
                                && a["claim"]["attempt"] == 1
                        })
                }
                "writable-overlap" => {
                    good && next
                        && assignment.is_none()
                        && deferred.is_some_and(|d| {
                            d.values().any(|v| v.as_str().is_some_and(|v| v.contains("writable ownership")))
                        })
                }
                "duplicate-ticket" => {
                    (good
                        && next
                        && assignment.is_some_and(|a| a["claim"]["id"] == "lane-b" && a["claim"]["attempt"] == 1))
                        || p.file_name().unwrap().to_string_lossy().starts_with("ticket-disjoint-next")
                }
                "crash-restart" => good && op == "resume" && o["status"] == "reviewed-resume",
                "surviving-child" => good && p.file_name().unwrap().to_string_lossy().starts_with("survival-inspect"),
                "stopped-transfer" => good && op == "transfer",
                "stale-authority" => {
                    !good
                        && fs::read_to_string(r["stderr"]["path"].as_str().unwrap())?
                            .contains("stale session authority")
                }
                "same-attempt-retry" => good && record && result == "submitted",
                "shared-capacity" => {
                    good && next
                        && assignment.is_none()
                        && o["jobs"].as_array().is_some_and(|jobs| {
                            jobs.iter()
                                .filter(|j| ["reserved", "running"].contains(&j["state"].as_str().unwrap_or_default()))
                                .count()
                                == 3
                        })
                }
                "cleanup" | "integration-success" => {
                    good && ((record && result == "integrated") || r["route"] == "decomp-authority")
                }
                _ => false,
            };
            if matches {
                selected.push(seal(p)?);
            }
        }
        ensure!(!selected.is_empty(), "control {name} not yet observed");
        let mut observations = Vec::new();
        if name == "surviving-child" {
            observations.push(seal(&state().join("surviving-child.toml"))?);
        }
        if name == "duplicate-ticket" {
            for p in ["scope-ticket-disjoint.toml", "ticket-disjoint-holder.toml"] {
                observations.push(seal(&state().join(p))?);
            }
        }
        if name == "integration-success" || name == "same-attempt-retry" {
            observations.push(seal(&post_b)?);
        }
        controls.insert(name.to_owned(), json!({"commands":selected,"observations":observations}));
    }
    let receipts = Path::new(ROOT).join(".scratch/concurrency-drill-retry/receipts");
    let mut integrations = Vec::new();
    for entry in fs::read_dir(&receipts)? {
        let p = entry?.path();
        if !p.file_name().unwrap().to_string_lossy().ends_with("-infrastructure-result.toml") {
            continue;
        }
        let publication = read(&p)?;
        let result = &publication["admitted"]["result"];
        if result["result"] != "integrated" {
            continue;
        }
        integrations
            .push(json!({"result":seal(&p)?,"authentication":result["authentication"],"cleanup":result["cleanup"]}));
    }
    ensure!(integrations.len() == 2, "exactly two sequential authenticated integrations required");
    integrations.sort_by_key(|v| v["result"]["path"].as_str().unwrap().to_owned());
    let execution = Path::new(STORE).join("execution.toml");
    write(
        &execution,
        &json!({"schema":"decomp-drill-execution-v1","frozen_input":seal(&state().join("frozen-input.toml"))?,"isolated_bridge":seal(&state().join("bridge.toml"))?,"sessions":sessions,"controls":controls,"integrations":integrations,"worker_budget":seal(&state().join("worker-budget.toml"))?}),
    )?;
    let booleans: serde_json::Map<String, J> = controls.keys().map(|name| (name.clone(), J::Bool(true))).collect();
    let outer = Path::new(STORE).join("concurrency-drill.toml");
    let frozen = read(&state().join("frozen-input.toml"))?;
    write(
        &outer,
        &json!({"schema":"decomp-concurrency-drill-v1","checkout":"/home/jim/sm-decomp","source_checkout":frozen["engine_source_checkout"],"isolated_checkout":ROOT,"bridge":seal(Path::new("/home/jim/sm-decomp/.scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml"))?,"head":frozen["engine_source_commit"],"source_fingerprint":frozen["engine_source_fingerprint"],"controls":booleans,"evidence":[seal(&execution)?,seal(&post_b)?],"execution":seal(&execution)?}),
    )?;
    Ok(json!({"execution":seal(&execution)?,"drill":seal(&outer)?}))
}
fn authenticate() -> Result<J> {
    fn walk(value: &J, checked: &mut std::collections::BTreeSet<PathBuf>) -> Result<()> {
        match value {
            J::Object(map) => {
                if let (Some(path), Some(hash)) =
                    (map.get("path").and_then(J::as_str), map.get("sha256").and_then(J::as_str))
                {
                    let path = PathBuf::from(path);
                    let current = seal(&path)?;
                    ensure!(current["sha256"] == hash, "retained seal mismatch: {}", path.display());
                    if checked.insert(path.clone()) {
                        if let Ok(v) = read(&path) {
                            walk(&v, checked)?;
                        }
                    }
                }
                for v in map.values() {
                    walk(v, checked)?;
                }
            }
            J::Array(list) => {
                for v in list {
                    walk(v, checked)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    let mut checked = std::collections::BTreeSet::new();
    for p in [
        Path::new(STORE).join("concurrency-drill.toml"),
        Path::new(STORE).join("final-cleanup1.toml"),
        Path::new(STORE).join("final-cleanup2.toml"),
    ] {
        walk(&seal(&p)?, &mut checked)?;
    }
    let result = json!({"schema":"decomp-drill-retained-authentication-v1","authenticated":true,"checked_artifacts":checked.len(),"observed_at":now(),"artifacts":checked.iter().map(|p|seal(p)).collect::<Result<Vec<_>>>()?});
    write(&Path::new(STORE).join("retained-authentication.toml"), &result)?;
    Ok(result)
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let value = match args.get(1).map(String::as_str) {
        Some("prepare-ticket") => prepare_ticket()?,
        Some("pure-next") => pure_next(&args[2], args[3].parse()?, args.get(4).is_some())?,
        Some("pure-controls") => pure_controls(&args[2], args[3].parse()?)?,
        Some("manifest") => manifest()?,
        Some("authenticate") => authenticate()?,
        Some("start-request") => start_request(&args[2])?,
        Some("remember") => remember(&args[2])?,
        Some("seed-ticket") => ticket_seal(true)?,
        Some("switch-ticket") => ticket_seal(false)?,
        _ => anyhow::bail!("usage: prepare-ticket | pure-next TOKEN GENERATION | manifest"),
    };
    let path = Path::new(STORE).join("command-results").join(format!(
        "evidence-{}-{}.toml",std::process::id(),SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()));
    write(&path,&value)?;
    let mut compact = json!({"result":seal(&path)?});
    for key in ["command","request","execution","drill","authenticated","checked_artifacts","status","scope","operation"] {
        if let Some(value)=value.get(key){compact[key]=value.clone();}
    }
    println!("{}", toml::to_string_pretty(&toml::Value::try_from(compact)?)?);
    Ok(())
}
