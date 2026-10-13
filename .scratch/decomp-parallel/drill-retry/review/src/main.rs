//! Read-only inspection: stdout is the sole output; callers retain reports.
use anyhow::{Context, Result, ensure};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn parse(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)?;
    if let Ok(value) = serde_json::from_str(&text) {
        return Ok(value);
    }
    Ok(serde_json::to_value(toml::from_str::<toml::Value>(&text)?)?)
}

fn hash(path: &Path) -> Result<String> {
    ensure!(!fs::symlink_metadata(path)?.file_type().is_symlink(), "symlink artifact: {}", path.display());
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn authenticate(value: &Value, checked: &mut BTreeMap<String, String>) -> Result<()> {
    match value {
        Value::Object(map) => {
            if let (Some(path), Some(expected)) =
                (map.get("path").and_then(Value::as_str), map.get("sha256").and_then(Value::as_str))
            {
                let actual = hash(Path::new(path)).with_context(|| format!("artifact {path}"))?;
                ensure!(actual == expected, "seal mismatch: {path}: {actual} != {expected}");
                if let Some(old) = checked.insert(path.to_owned(), actual) {
                    ensure!(old == expected, "inconsistent references: {path}");
                } else if let Ok(dependency) = parse(Path::new(path)) {
                    authenticate(&dependency, checked)?;
                }
            }
            for child in map.values() {
                authenticate(child, checked)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                authenticate(child, checked)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn files(root: &Path, result: &mut BTreeMap<String, String>) -> Result<()> {
    ensure!(!fs::symlink_metadata(root)?.file_type().is_symlink(), "inventory symlink: {}", root.display());
    if root.is_dir() {
        for item in fs::read_dir(root)? {
            files(&item?.path(), result)?;
        }
    } else {
        result.insert(root.display().to_string(), hash(root)?);
    }
    Ok(())
}

fn retry(store: &Path, fixture: &Path, checked: &mut BTreeMap<String, String>) -> Result<Value> {
    let proof = parse(&store.join("post-b-retry.toml"))?;
    ensure!(
        proof["both_integrations_complete"] == true
            && proof["exact_a_retry_passed"] == true
            && proof["exact_b_retry_passed"] == true
            && proof["publication_and_gate_bytes_unchanged"] == true
            && proof["gate_reruns"] == 0,
        "retry proof incomplete"
    );
    authenticate(&proof, checked)?;
    let snapshot = parse(Path::new(proof["receipt_snapshot"]["path"].as_str().context("retry snapshot missing")?))?;
    let mut before = BTreeMap::new();
    for item in snapshot["artifacts"].as_array().context("snapshot artifacts missing")? {
        let path = item["path"].as_str().context("snapshot path missing")?;
        ensure!(
            before
                .insert(path.to_owned(), item["sha256"].as_str().context("snapshot hash missing")?.to_owned())
                .is_none(),
            "duplicate snapshot path"
        );
    }
    let mut after = BTreeMap::new();
    files(&fixture.join(".scratch/concurrency-drill-retry/receipts"), &mut after)?;
    ensure!(before == after, "maintained receipt inventory differs from pre-retry snapshot");
    let head = Command::new("git").arg("-C").arg(fixture).args(["rev-parse", "HEAD"]).output()?;
    ensure!(head.status.success(), "cannot inspect shared HEAD");
    let head = String::from_utf8(head.stdout)?.trim().to_owned();
    ensure!(proof["source_commit"] == head && snapshot["source_commit"] == head, "final shared HEAD mismatch");
    let ledger = parse(&fixture.join(".scratch/concurrency-drill-retry/ledger.toml"))?;
    for id in ["integrate-lane-a", "integrate-lane-b", "lane-a", "lane-b"] {
        ensure!(
            ledger["jobs"][id]["state"] == "complete" && ledger["jobs"][id]["stale"] == false,
            "incomplete/stale job: {id}"
        );
    }
    Ok(json!({"source_commit":head,"receipt_count":before.len(),"inventory_equal":true,"jobs_complete":true}))
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(args.len() >= 3, "usage: REVIEW STORE FIXTURE ROOT_ARTIFACT...");
    let store = PathBuf::from(&args[0]);
    let fixture = PathBuf::from(&args[1]);
    let mut checked = BTreeMap::new();
    for argument in &args[2..] {
        let path = Path::new(argument);
        let root = json!({"path":path,"sha256":hash(path)?});
        authenticate(&root, &mut checked)?;
    }
    let retry = if store.join("post-b-retry.toml").exists() {
        retry(&store, &fixture, &mut checked)?
    } else {
        json!({"status":"pending"})
    };
    let required = [
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
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let execution = if store.join("execution.toml").exists() {
        let value = parse(&store.join("execution.toml"))?;
        let observed = value["controls"]
            .as_object()
            .context("controls missing")?
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        ensure!(observed == required, "control set differs from required eleven");
        ensure!(
            value["integrations"].as_array().context("integrations missing")?.len() == 2,
            "need exactly two integrations"
        );
        for session in value["sessions"].as_array().context("sessions missing")? {
            let launch = parse(Path::new(session["launch"]["path"].as_str().context("launch path missing")?))?;
            let log = fs::read_to_string(session["log"]["path"].as_str().context("log path missing")?)?;
            let start: Value = serde_json::from_str(log.lines().next().context("empty session log")?)?;
            ensure!(
                start["type"] == "thread.started" && start["thread_id"] == launch["session_id"],
                "actual launch UUID mismatch"
            );
        }
        json!({"controls":required,"integrations":2,"session_launches":value["sessions"].as_array().unwrap().len(),"launch_ids_match":true})
    } else {
        json!({"status":"pending"})
    };
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"schema":"decomp-drill-independent-inspection-v1","authenticated":true,"checked_artifacts":checked.len(),"retry":retry,"execution":execution,"artifacts":checked})
        )?
    );
    Ok(())
}
