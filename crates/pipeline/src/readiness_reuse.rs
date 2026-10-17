use crate::atomic_toml;
use crate::digest;
use crate::lock;
use crate::readiness_contract as contract;
use crate::readiness_origins as origins;
use crate::readiness_run::Run;
use crate::readiness_run::identity;
use crate::readiness_run::safe_artifact;
use crate::readiness_run::source_inventory;
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use toml::Value;
fn executable(path: &str) -> bool {
    let p = Path::new(path);
    !([Some("md"), Some("rst")].contains(&p.extension().and_then(|s| s.to_str()))
        && (path.starts_with("docs/")
            || path.starts_with("workflows/")
            || p.file_name().is_some_and(|n| n == "README.md" || n == "NOTES.md")))
}
pub fn policy(manifest: &Value, files: &Value, source: &Path) -> Result<Value> {
    let entries = files.as_array().context("source dependency inventory missing")?;
    let docs =
        entries.iter().filter(|entry| entry["path"].as_str().is_some_and(|p| !executable(p))).collect::<Vec<_>>();
    let mut referenced = BTreeSet::new();
    for entry in entries {
        let path = crate::documents::field(entry, "path")?;
        if !executable(path) {
            continue;
        }
        let text = String::from_utf8_lossy(&fs::read(safe_artifact(source, path)?)?).into_owned();
        for document in &docs {
            let p = crate::documents::field(document, "path")?;
            let filename = Path::new(p).file_name().and_then(|s| s.to_str()).context("document filename")?;
            if text.contains(p) || text.contains(filename) {
                referenced.insert(p.to_owned());
            }
        }
    }
    let mut inputs = manifest["identities"]["inputs"].clone();
    for name in ["input_evidence", "compiled_table_rom", "prior_compiled_table_sha256"] {
        inputs.as_table_mut().context("dependency input identities missing")?.remove(name);
    }
    let settings = crate :: report ! ({ "stage_commands" : crate :: report ! ({ "check" : crate :: report ! (vec ! ["just" , "check"]) , "lint" : crate :: report ! (vec ! ["just" , "lint"]) , }) , "inputs" : inputs , "hardware" : manifest ["identities"] ["hardware"] , "compiler" : manifest ["identities"] ["compiler"] , "build_settings" : manifest ["build_settings"] , });
    let mut policies = BTreeMap::new();
    for name in contract::REUSABLE {
        let files = entries
            .iter()
            .filter(|entry| {
                let path = entry["path"].as_str().unwrap();
                let shared = executable(path)
                    && !["config/functions.toml", "config/jump_tables.toml"].contains(&path)
                    && !path.starts_with("crates/recomp/src/");
                referenced.contains(path)
                    || if ["synchronization", "fallback-only-parity"].contains(&name) {
                        shared
                    } else {
                        executable(path)
                    }
            })
            .cloned()
            .collect::<Vec<_>>();
        policies.insert(
            name,
            crate :: report ! ({ "files" : files , "files_sha256" : identity (& files) ? , "settings" : settings , }),
        );
    }
    Ok(
        crate :: report ! ({ "schema_version" : 1 , "contract_version" : contract :: VERSION , "source_sha256" : manifest ["identities"] ["source"] ["sha256"] , "checks" : policies , "candidate_checks_always_fresh" : contract :: CANDIDATE , "cache_policy" : "exact complete identities and cold-history validation; changed candidate rebuilds cold" , }),
    )
}
pub fn record_policy(run: &mut Run) -> Result<Value> {
    run.check_sources()?;
    let files = origins::document(&run.path.join("source/inventory.toml"))?;
    let report = policy(&Value::try_from(&run.manifest)?, &files["files"], &run.path.join("source/files"))?;
    let path = run.path.join("checks/dependency-policy.toml");
    atomic_toml(&path, &report)?;
    run.record_artifact("dependency-policy", &path, None)?;
    run.manifest.identities.get_mut("dependency-policy").context("dependency identity missing")?["evidence"] =
        "checks/dependency-policy.toml".into();
    run.save()?;
    Ok(report)
}
fn verify_policy(owner: &Path, manifest: &Value, inventory: &Value) -> Result<Value> {
    let path = origins::artifact(owner, "dependency-policy", manifest)?;
    let report = origins::document(&path)?;
    ensure!(
        report == policy(manifest, &inventory["files"], &owner.join("source/files"))?,
        "dependency policy differs from actual source/settings"
    );
    Ok(report)
}
pub fn pointer(owner: &Path, manifest: &Value) -> Result<Value> {
    Ok(
        crate :: report ! ({ "schema_version" : 1 , "format" : "sm-native-readiness-pointer-v1" , "run_id" : manifest ["run_id"] , "run_path" : fs :: canonicalize (owner) ? , "manifest_sha256" : digest (& owner . join ("manifest.toml")) ? , "verdict_sha256" : digest (& owner . join ("verdict.toml")) ? , "source_inventory" : "source/inventory.toml" , "source_sha256" : manifest ["identities"] ["source"] ["sha256"] , "contract_version" : contract :: VERSION , "domain" : contract :: domain () , }),
    )
}
pub fn verify_accepted(owner: &Path, require_pointer: bool) -> Result<Value> {
    let owner = fs::canonicalize(owner)?;
    let (typed_manifest, typed_inventory) = origins::snapshot_typed(&owner)?;
    let typed_verdict: pipeline_model::readiness::Verdict = pipeline_model::read(&owner.join("verdict.toml"))?;
    ensure!(
        typed_manifest.state == "complete"
            && ["readiness", "iteration"].contains(&typed_manifest.kind.as_str())
            && typed_manifest.contract_version == Some(contract::VERSION)
            && typed_verdict.run_id == typed_manifest.run_id
            && typed_verdict.status == "pass"
            && typed_verdict.acceptance_eligible,
        "origin is not complete accepted readiness run"
    );
    let actual = &typed_verdict.mandatory_checks;
    let mut expected = contract::MANDATORY.map(str::to_owned).into_iter().collect::<BTreeSet<_>>();
    if typed_manifest.kind == "iteration" {
        ensure!(
            typed_manifest.accepted_parent.as_deref().is_some_and(|parent| !parent.is_empty()),
            "accepted iteration predecessor missing"
        );
        expected.insert("increment-progress".into());
    }
    ensure!(
        actual.len() == expected.len() && actual.iter().cloned().collect::<BTreeSet<_>>() == expected,
        "accepted origin lacks complete mandatory registry"
    );
    ensure!(
        typed_manifest.scope.get("domain").and_then(Value::as_str) == Some("Japan/USA any%")
            && typed_manifest.scope.get("frames").and_then(Value::as_integer) == Some(18000),
        "accepted origin domain differs"
    );
    ensure!(
        typed_verdict.checks.values().all(|r| r.get("status").and_then(Value::as_str) == Some("pass")
            && r.get("complete").and_then(Value::as_bool) == Some(true)),
        "accepted origin contains failed/incomplete checks"
    );
    for name in contract::MANDATORY {
        let report = typed_verdict.checks.get(name).context("accepted mandatory check missing")?;
        contract::validate(name, report)?;
        let entry = typed_manifest.check_evidence.get(name).context("accepted check seal missing")?;
        let path = safe_artifact(&owner, &entry.path)?;
        ensure!(
            digest(&path)? == entry.sha256 && origins::document(&path)? == *report,
            "accepted check differs from terminal/digest evidence: {name}"
        );
    }
    let manifest = Value::try_from(&typed_manifest)?;
    let inventory = Value::try_from(&typed_inventory)?;
    let verdict = Value::try_from(&typed_verdict)?;
    origins::owned(&owner, &manifest)?;
    verify_policy(&owner, &manifest, &inventory)?;
    let declaration = crate::pattern_contract::load(&owner.join("source/files"), None, None)?;
    for name in ["instruction", "timing"] {
        let report = typed_verdict.checks.get(name).context("accepted pattern check missing")?;
        ensure!(
            report.get("declaration") == Some(&Value::try_from(&declaration)?)
                && report.get("scope").and_then(Value::as_str) == Some("complete production declaration")
                && report.get("prospective_opcode").is_none(),
            "accepted pattern declaration differs from source"
        );
    }
    if require_pointer {
        let expected = pointer(&owner, &manifest)?;
        let root = owner
            .parent()
            .context("run parent missing")?
            .parent()
            .context("runs parent missing")?
            .parent()
            .context("checkout parent missing")?;
        let mut paths = vec![root.join("out/accepted.toml")];
        let history = root.join("out/accepted-history");
        if history.exists() {
            paths.extend(fs::read_dir(history)?.filter_map(|v| v.ok().map(|e| e.path())));
        }
        ensure!(
            paths.iter().any(|path| origins::document(path).is_ok_and(|actual| actual == expected)),
            "accepted pointer lineage missing or corrupt"
        );
    }
    Ok(crate :: report ! ({ "manifest" : manifest , "inventory" : inventory , "verdict" : verdict , }))
}
pub fn publish(owner: &Path, root: &Path) -> Result<Value> {
    fs::create_dir_all(root.join("out"))?;
    let _lock = lock(&root.join("out/acceptance.lock"))?;
    let verified = verify_accepted(owner, false)?;
    if let Some(expected) = verified["manifest"].get("expected_pointer").and_then(Value::as_str) {
        ensure!(pointer_token(root)? == expected, "accepted pointer changed during readiness recovery");
    }
    crate::readiness::verify_origins(owner, &verified)?;
    ensure!(
        crate::readiness_metadata::value(&source_inventory(root)?).expect("validated native metadata")
            == verified["inventory"]["files"],
        "candidate source differs from current checkout"
    );
    if verified["manifest"]["type"]
        == crate::readiness_metadata::value(&"iteration").expect("validated native metadata")
    {
        let current = origins::document(&root.join("out/accepted.toml"))?;
        ensure!(
            crate::documents::field(&current, "run_id")?
                == crate::documents::field(&verified["manifest"], "accepted_parent")?,
            "competing acceptance changed predecessor before publication"
        );
    }
    let pointer = pointer(owner, &verified["manifest"])?;
    let history = root.join("out/accepted-history");
    fs::create_dir_all(&history)?;
    let retained = history.join(format!("{}.toml", crate::documents::field(&verified["manifest"], "run_id")?));
    if retained.exists() {
        ensure!(origins::document(&retained)? == pointer, "accepted history immutable identity differs");
    } else {
        atomic_toml(&retained, &pointer)?;
    }
    atomic_toml(
        &owner.join("acceptance-result.toml"),
        &crate :: report ! ({ "status" : "pass" , "accepted" : true , "pointer" : pointer , }),
    )?;
    atomic_toml(&root.join("out/accepted.toml"), &pointer)?;
    Ok(pointer)
}
pub(crate) fn pointer_token(root: &Path) -> Result<String> {
    match fs::read(root.join("out/accepted.toml")) {
        Ok(bytes) => {
            use sha2::{Digest, Sha256};
            Ok(format!("{:x}", Sha256::digest(bytes)))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok("absent".into()),
        Err(error) => Err(error.into()),
    }
}
pub fn plan(run: &Run, previous: Option<&Path>) -> Result<Value> {
    let mut result = crate::report!({});
    for name in contract::CANDIDATE {
        {
            let value: toml::Value =
                crate :: report ! ({ "action" : "fresh" , "reason" : "candidate-specific evidence always required" , });
            result.as_table_mut().context("native metadata table missing")?.insert((name).into(), value);
        };
    }
    for name in contract::REUSABLE {
        {
            let value: toml::Value =
                crate :: report ! ({ "action" : "fresh" , "reason" : "no verified accepted predecessor" , });
            result.as_table_mut().context("native metadata table missing")?.insert((name).into(), value);
        };
    }
    let Some(owner) = previous else {
        return Ok(result);
    };
    let verified = match verify_accepted(owner, true) {
        Ok(v) => v,
        Err(error) => {
            for name in contract::REUSABLE {
                result[name]["reason"] = format!("accepted origin cannot be verified: {error:#}").into();
            }
            return Ok(result);
        }
    };
    if let Err(error) = crate::readiness::verify_origins(owner, &verified) {
        for name in contract::REUSABLE {
            result[name]["reason"] = format!("accepted complete origins cannot be verified: {error:#}").into();
        }
        return Ok(result);
    }
    let old = verify_policy(owner, &verified["manifest"], &verified["inventory"])?;
    let inventory = origins::document(&run.path.join("source/inventory.toml"))?;
    let current = verify_policy(&run.path, &Value::try_from(&run.manifest)?, &inventory)?;
    for name in contract::REUSABLE {
        if old["checks"][name] != current["checks"][name] {
            result[name]["reason"] = "executable/build dependencies or settings changed".into();
            continue;
        }
        match crate::readiness::verify_check(owner, name, &verified) {
            Ok(report) => {
                {
                    let value: toml::Value = crate :: report ! ({ "action" : "reuse" , "reason" : "verified accepted scope and unchanged dependencies" , "run_id" : verified ["manifest"] ["run_id"] , "dependency_sha256" : identity (& old ["checks"] [name]) ? , "report" : report , "path" : owner , });
                    result.as_table_mut().context("native metadata table missing")?.insert((name).into(), value);
                };
            }
            Err(error) => {
                result[name]["reason"] = format!("accepted check origin cannot be verified: {error:#}").into()
            }
        }
    }
    Ok(result)
}
pub fn apply(run: &mut Run, name: &str, plan: &Value) -> Result<Value> {
    ensure!(
        contract::REUSABLE.contains(&name)
            && plan["action"] == crate::readiness_metadata::value(&"reuse").expect("validated native metadata"),
        "check is not eligible for reuse"
    );
    let owner = Path::new(crate::documents::field(plan, "path")?);
    let verified = verify_accepted(owner, true)?;
    let report = crate::readiness::verify_check(owner, name, &verified)?;
    ensure!(
        crate::readiness_metadata::value(&report).expect("validated native metadata") == plan["report"]
            && verified["manifest"]["run_id"] == plan["run_id"],
        "reuse origin changed since planning"
    );
    let old = verify_policy(owner, &verified["manifest"], &verified["inventory"])?;
    let inventory = origins::document(&run.path.join("source/inventory.toml"))?;
    let current = verify_policy(&run.path, &Value::try_from(&run.manifest)?, &inventory)?;
    ensure!(
        old["checks"][name] == current["checks"][name]
            && identity(&old["checks"][name])? == crate::documents::field(plan, "dependency_sha256")?,
        "reuse dependencies differ"
    );
    let mut report = report;
    {
        let value: toml::Value = crate :: report ! ({ "path" : owner , "run_id" : verified ["manifest"] ["run_id"] , "dependency_sha256" : plan ["dependency_sha256"] , });
        report.as_table_mut().context("native metadata table missing")?.insert(("reused_from").into(), value);
    };
    run.record_check(name, report.clone())?;
    Ok(report)
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mut run = Run::create(&root, "reuse-plan", contract::domain(), None)?;
    let outcome = (|| -> Result<Value> {
        let failures = crate::preflight::verify(&mut run)?;
        ensure!(failures.is_empty(), "preflight failed: {}", failures.join("; "));
        record_policy(&mut run)?;
        plan(&run, request.get("predecessor").and_then(Value::as_str).map(Path::new))
    })();
    match outcome {
        Ok(plan) => {
            atomic_toml(&run.path.join("reuse.toml"), &plan)?;
            run.record_check(
                "reuse-plan",
                crate :: report ! ({ "status" : "pass" , "complete" : true , "plan" : plan , }),
            )?;
            run.finalize(
                "pass",
                Some("reuse-plan"),
                Some("verified reuse plan; no acceptance"),
                &["preflight".into(), "reuse-plan".into()],
            )
        }
        Err(error) => run.finalize("error", Some("reuse-plan"), Some(&format!("{error:#}")), &[]),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn project() -> Temp {
        let root = std::env::temp_dir().join(format!(
            "sm-native-reuse-controls-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        assert!(std::process::Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        for (name, text) in [
            ("config/functions.toml", "# routines"),
            ("crates/recomp/src/step.rs", "// emitter"),
            ("crates/runtime/src/machine.rs", "// runtime"),
            ("docs/control.md", "domain"),
            ("tools/control.rs", "// reads docs/control.md"),
            ("docs/unreferenced.md", "prose"),
        ] {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Temp(root)
    }
    fn policies(root: &Path) -> Value {
        let manifest = crate :: report ! ({ "identities" : crate :: report ! ({ "source" : crate :: report ! ({ "sha256" : "candidate" , }) , "inputs" : crate :: report ! ({ "rom_sha256" : "rom" , }) , "hardware" : crate :: report ! ({ }) , "compiler" : crate :: report ! ({ }) , }) , "build_settings" : crate :: report ! ({ }) , });
        let mut files = source_inventory(root).unwrap();
        for name in ["docs/control.md", "docs/unreferenced.md"] {
            files.push(pipeline_model::readiness::SourceFile {
                path: name.into(),
                sha256: digest(&root.join(name)).unwrap(),
                mode: 420,
            });
        }
        policy(&manifest, &Value::try_from(&files).unwrap(), root).unwrap()
    }
    #[test]
    fn compiler_changes_refresh_candidate_checks_and_shared_changes_refresh_reusable_checks() {
        let root = project();
        let before = policies(&root.0);
        fs::write(root.0.join("config/functions.toml"), "# candidate routines changed").unwrap();
        fs::write(root.0.join("crates/recomp/src/step.rs"), "// emitter changed").unwrap();
        let compiler = policies(&root.0);
        for name in ["fallback-only-parity", "synchronization"] {
            assert_eq!(before["checks"][name], compiler["checks"][name]);
        }
        assert_ne!(before["checks"]["check"], compiler["checks"]["check"]);
        fs::write(root.0.join("crates/runtime/src/machine.rs"), "// observer changed").unwrap();
        let shared = policies(&root.0);
        for name in contract::REUSABLE {
            assert_ne!(compiler["checks"][name], shared["checks"][name]);
        }
        fs::write(root.0.join("docs/unreferenced.md"), "new prose").unwrap();
        let prose = policies(&root.0);
        assert_eq!(shared["checks"], prose["checks"]);
        fs::write(root.0.join("docs/control.md"), "new executed contract").unwrap();
        let contract = policies(&root.0);
        for name in super::contract::REUSABLE {
            assert_ne!(prose["checks"][name], contract["checks"][name]);
        }
    }
    #[test]
    fn scoped_pass_cannot_publish_or_replace_predecessor() {
        let root = project();
        fs::create_dir_all(root.0.join("out")).unwrap();
        let pointer = root.0.join("out/accepted.toml");
        fs::write(&pointer, "prior pointer").unwrap();
        let mut run = Run::create(&root.0, "patterns", contract::domain(), None).unwrap();
        run.capture_source().unwrap();
        run.record_check("instruction", crate :: report ! ({ "status" : "pass" , "complete" : true , })).unwrap();
        run.finalize("pass", None, None, &["instruction".into()]).unwrap();
        assert!(publish(&run.path, &root.0).is_err());
        assert_eq!(fs::read_to_string(pointer).unwrap(), "prior pointer");
    }
}
