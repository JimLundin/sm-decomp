use crate::digest;
use crate::lock;
use crate::preflight;
use crate::readiness;
use crate::readiness_contract as contract;
use crate::readiness_metadata::toml_value_decode;
use crate::readiness_origins as origins;
use crate::readiness_reuse as reuse;
use crate::readiness_run::Run;
use crate::readiness_run::safe_artifact;
use crate::readiness_run::source_inventory;
use anyhow::{Context, Result, ensure};
use pipeline_model::{BoundSelection, Candidate, GeneratedKey, KeyInventory, SelectionAction, SelectionOutcome};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};
use toml::Value;
use toml_edit::{Array, DocumentMut, Table, value};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    pub address: u32,
    pub mx: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Proposal {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub selection_sha256: String,
    pub prior_config_sha256: String,
    pub proposed_config_sha256: String,
    pub steps_sha256: String,
    pub actual_new_keys: Vec<Key>,
    pub deferred: Vec<GeneratedKey>,
    pub preserved_baseline_keys: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Increment {
    pub schema_version: u32,
    pub kind: String,
    pub baseline_path: PathBuf,
    pub selection_path: PathBuf,
    pub selection_sha256: String,
    pub preparation_path: PathBuf,
    pub chosen: Candidate,
    pub deferred: Vec<GeneratedKey>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<crate::iteration_pattern::Promotion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    pub schema_version: u32,
    pub status: String,
    pub complete: bool,
    pub kind: String,
    pub generated_instructions: u64,
    pub interpreted_instructions: u64,
    pub comparable: bool,
    pub generated_delta: i64,
    pub interpreted_delta: i64,
    pub validated_keys_delta: i64,
    pub reason: String,
}
fn rows(path: &Path) -> Result<Vec<GeneratedKey>> {
    toml_value_decode(crate::coverage::table_rows(&fs::read_to_string(path)?)?)
}
fn index(rows: &[GeneratedKey]) -> BTreeMap<(u32, u8), &GeneratedKey> {
    rows.iter().map(|r| ((r.address, r.mx), r)).collect()
}
fn address(table: &Table) -> Result<u32> {
    let text = table.get("addr").and_then(|i| i.as_str()).context("configuration address missing")?;
    Ok(u32::from_str_radix(&text.chars().filter(|c| c.is_ascii_hexdigit()).collect::<String>(), 16)?)
}
fn config(source: &str, chosen: &Candidate) -> Result<String> {
    chosen.validate()?;
    let mut doc = source.parse::<DocumentMut>()?;
    if doc.get("function").is_none() {
        doc["function"] = toml_edit::Item::ArrayOfTables(Default::default());
    }
    let functions = doc["function"].as_array_of_tables_mut().context("function declarations must be tables")?;
    let mut found = false;
    for table in functions.iter_mut() {
        if address(table)? != chosen.address {
            continue;
        }
        ensure!(!found, "duplicate reviewed routine address");
        found = true;
        let variants = table["variants"].as_array_mut().context("reviewed routine lacks explicit widths")?;
        let present = variants.iter().any(|v| v.as_str() == Some(&chosen.variant));
        ensure!(
            present == (chosen.action == SelectionAction::EnableDeferredKeys),
            "selected routine/width action differs from reviewed configuration"
        );
        if !present {
            variants.push(&chosen.variant);
        }
    }
    if !found {
        ensure!(chosen.action == SelectionAction::AddRoutineWidth, "deferred revisit requires reviewed routine");
        let mut table = Table::new();
        table["addr"] = value(format!("{:06X}", chosen.address));
        let mut widths = Array::new();
        widths.push(&chosen.variant);
        table["variants"] = value(widths);
        functions.push(table);
    }
    let desired = chosen.new_executed_keys.iter().map(|r| (r.address, r.mx)).collect::<BTreeSet<_>>();
    let mut comments = String::new();
    if let Some(exclusions) = doc.get_mut("excluded_step").and_then(|i| i.as_array_of_tables_mut()) {
        let mut kept = toml_edit::ArrayOfTables::new();
        for table in exclusions.iter() {
            let width = table.get("variant").and_then(|i| i.as_str()).context("exclusion width missing")?;
            let mx = match width {
                "m0x0" => 0,
                "m0x1" => 1,
                "m1x0" => 2,
                "m1x1" => 3,
                _ => anyhow::bail!("invalid exclusion width"),
            };
            if desired.contains(&(address(table)?, mx)) {
                for line in table.to_string().lines().filter(|l| l.trim_start().starts_with('#')) {
                    comments.push_str(line);
                    comments.push('\n');
                }
                if let Some(prefix) = table.decor().prefix().and_then(|s| s.as_str()) {
                    comments.push_str(prefix);
                }
            } else {
                kept.push(table.clone());
            }
        }
        *exclusions = kept;
    }
    Ok(format!("{doc}\n{comments}"))
}
pub(crate) fn generator(run: &mut Run, timeout: u64) -> Result<PathBuf> {
    let root = run.root.clone();
    let target = run.path.join("build/proposal-generator");
    let env = BTreeMap::from([
        ("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned()),
        ("CARGO_INCREMENTAL".into(), "0".into()),
    ]);
    let stage = run.stage(
        "proposal-generator-build",
        &["cargo", "build", "--release", "-p", "recomp", "--locked", "--offline", "--jobs", "1"].map(str::to_owned),
        &root,
        &env,
        Duration::from_secs(timeout),
        &BTreeMap::new(),
    )?;
    ensure!(
        stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "proposal generator build failed"
    );
    run.record_artifact("proposal-generator", &target.join("release/recomp"), Some("generated/recomp"))?;
    let path = run.path.join("generated/recomp");
    run.guard(&path)?;
    Ok(path)
}
#[cfg(test)]
fn generate(run: &mut Run, generator: &Path, config: &Path, name: &str, timeout: u64) -> Result<PathBuf> {
    generate_pattern(run, generator, config, name, timeout, None)
}
fn generate_pattern(
    run: &mut Run,
    generator: &Path,
    config: &Path,
    name: &str,
    timeout: u64,
    prospective: Option<&str>,
) -> Result<PathBuf> {
    let root = run.root.clone();
    let steps = run.path.join(format!("generated/{name}-steps.rs"));
    let mut command = vec![
        generator.to_string_lossy().into_owned(),
        "--rom".into(),
        root.join("rom/sm.sfc").to_string_lossy().into_owned(),
        "--functions".into(),
        config.to_string_lossy().into_owned(),
        "--out".into(),
        run.path.join(format!("generated/{name}-game.rs")).to_string_lossy().into_owned(),
        "--steps-out".into(),
        steps.to_string_lossy().into_owned(),
    ];
    if let Some(opcode) = prospective {
        command.extend(["--prospective-pattern".into(), opcode.into()]);
    }
    let stage = run.stage(
        &format!("{name}-generation"),
        &command,
        &root,
        &BTreeMap::new(),
        Duration::from_secs(timeout),
        &BTreeMap::new(),
    )?;
    ensure!(
        stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "{name} generation failed"
    );
    Ok(steps)
}
pub(crate) fn propose(
    run: &mut Run,
    baseline: &KeyInventory,
    chosen: &Candidate,
    generator: &Path,
    selection_hash: &str,
    timeout: u64,
) -> Result<PathBuf> {
    propose_pattern(run, baseline, chosen, generator, selection_hash, timeout, None)
}
pub(crate) fn propose_pattern(
    run: &mut Run,
    baseline: &KeyInventory,
    chosen: &Candidate,
    generator: &Path,
    selection_hash: &str,
    timeout: u64,
    prospective: Option<&str>,
) -> Result<PathBuf> {
    baseline.validate()?;
    chosen.validate()?;
    run.guard(generator)?;
    let current = run.root.join("config/functions.toml");
    let proposal = run.path.join("generated/functions.toml");
    fs::write(&proposal, config(&fs::read_to_string(&current)?, chosen)?)?;
    let first = generate_pattern(run, generator, &proposal, "unpruned", timeout, prospective)?;
    let all = rows(&first)?;
    let previous = index(&baseline.keys);
    let actual = index(&all);
    let desired = chosen.new_executed_keys.iter().map(|r| ((r.address, r.mx), r)).collect::<BTreeMap<_, _>>();
    ensure!(previous.keys().all(|key| actual.contains_key(key)), "proposal removes accepted baseline key");
    for (key, row) in &desired {
        ensure!(
            !previous.contains_key(key)
                && actual.get(key).is_some_and(|a| a.bytes == row.bytes && a.opcode == row.opcode),
            "selected observed key differs from generated CFG"
        );
    }
    let deferred = all
        .iter()
        .filter(|r| !previous.contains_key(&(r.address, r.mx)) && !desired.contains_key(&(r.address, r.mx)))
        .cloned()
        .collect::<Vec<_>>();
    let mut source = fs::read_to_string(&proposal)?;
    for row in &deferred {
        source . push_str (& format ! ("\n[[excluded_step]]\naddr = \"{:06X}\"\nvariant = \"m{}x{}\"\nreason = \"deferred unexecuted proposed key\"\n" , row . address , row . mx >> 1 , row . mx & 1)) ;
    }
    fs::write(&proposal, source)?;
    let final_steps = generate_pattern(run, generator, &proposal, "proposed", timeout, prospective)?;
    let final_rows = rows(&final_steps)?;
    ensure!(
        index(&final_rows).keys().copied().collect::<BTreeSet<_>>()
            == previous.keys().chain(desired.keys()).copied().collect(),
        "pruned table differs from baseline plus measured slice"
    );
    let report = Proposal {
        schema_version: 1,
        status: "pass".into(),
        complete: true,
        selection_sha256: selection_hash.into(),
        prior_config_sha256: digest(&current)?,
        proposed_config_sha256: digest(&proposal)?,
        steps_sha256: digest(&final_steps)?,
        actual_new_keys: desired.keys().map(|&(address, mx)| Key { address, mx }).collect(),
        deferred,
        preserved_baseline_keys: previous.len(),
    };
    run.record_typed_check("candidate-proposal", &report)?;
    run.record_artifact("proposed-config", &proposal, None)?;
    run.record_artifact("proposed-steps", &final_steps, None)?;
    Ok(proposal)
}
fn selection(owner: &Path) -> Result<BoundSelection> {
    selection_for(owner, SelectionOutcome::SupportedRoutine)
}
pub(crate) fn selection_for(owner: &Path, outcome: SelectionOutcome) -> Result<BoundSelection> {
    let (manifest, _) = origins::snapshot(owner)?;
    origins::owned(owner, &manifest)?;
    let verdict = origins::document(&owner.join("verdict.toml"))?;
    ensure!(
        manifest["type"]
            == crate::readiness_metadata::value(&"iteration-selection").expect("validated native metadata")
            && manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && verdict["acceptance_eligible"]
                == crate::readiness_metadata::value(&false).expect("validated native metadata"),
        "selection is not a complete measured run"
    );
    let report: BoundSelection = pipeline_model::read(&origins::artifact(owner, "selection-result", &manifest)?)?;
    report.ranking.validate()?;
    ensure!(report.ranking.outcome == outcome, "selection requires a different route");
    let ranking: pipeline_model::SelectionReport = pipeline_model::read(&owner.join("checks/selection.toml"))?;
    ensure!(
        report.ranking == ranking
            && crate::readiness_metadata::value(&report.source_sha256).expect("validated native metadata")
                == manifest["identities"]["source"]["sha256"]
            && crate::readiness_metadata::value(&report.baseline.run_id).expect("validated native metadata")
                == manifest["accepted_parent"],
        "selection ranking/source/baseline differs"
    );
    for (name, hash) in [
        ("selection-profile", &report.profile_sha256),
        ("selection-baseline-inventory", &report.baseline_inventory_sha256),
        ("selection-discovery", &report.discovery_sha256),
    ] {
        ensure!(digest(&origins::artifact(owner, name, &manifest)?)? == *hash, "selection input differs: {name}");
    }
    let process = origins::process(owner, "selection-ranking", &manifest, None, &[0])?;
    let executable = origins::artifact(owner, "selection-generator", &manifest)?;
    ensure!(
        process["command"][0]
            == crate::readiness_metadata::value(&executable.to_string_lossy().as_ref())
                .expect("validated native metadata"),
        "selection executable differs"
    );
    let baseline = reuse::verify_accepted(&report.baseline.run_path, true)?;
    readiness::verify_origins(&report.baseline.run_path, &baseline)?;
    ensure!(
        baseline["manifest"]["run_id"]
            == crate::readiness_metadata::value(&report.baseline.run_id).expect("validated native metadata")
            && baseline["manifest"]["identities"]["source"]["sha256"]
                == crate::readiness_metadata::value(&report.baseline.source_sha256).expect("validated native metadata"),
        "selection accepted baseline differs"
    );
    Ok(report)
}
pub(crate) fn prepare(root: &Path, owner: &Path, timeout: u64) -> Result<PathBuf> {
    let owner = fs::canonicalize(owner)?;
    let mut run = Run::create(root, "increment-preparation", contract::domain(), None)?;
    eprintln!("Preparation: {}", run.path.display());
    let result = (|| -> Result<()> {
        run.capture_source()?;
        let selected = selection(&owner)?;
        ensure!(
            run.manifest.identities["source"]["sha256"]
                == crate::readiness_metadata::value(&selected.source_sha256).expect("validated native metadata"),
            "candidate changed since selection"
        );
        run.manifest.accepted_parent = selected.baseline.run_id.clone().into();
        run.save()?;
        ensure!(preflight::verify_existing(&mut run)?.is_empty(), "candidate preflight failed");
        let source = owner.join("selection.toml");
        let hash = digest(&source)?;
        run.record_artifact("measured-selection", &source, Some("generated/selection.toml"))?;
        let baseline: KeyInventory = pipeline_model::read(&owner.join("generated/selection-baseline-inventory.toml"))?;
        let executable = generator(&mut run, timeout)?;
        propose(
            &mut run,
            &baseline,
            selected.ranking.chosen.as_ref().context("selection chosen key missing")?,
            &executable,
            &hash,
            timeout,
        )?;
        let proposal: Proposal = pipeline_model::read(&run.path.join("checks/candidate-proposal.toml"))?;
        let increment = Increment {
            schema_version: 1,
            kind: "supported-expansion".into(),
            baseline_path: selected.baseline.run_path,
            selection_path: owner,
            selection_sha256: hash,
            preparation_path: run.path.clone(),
            chosen: selected.ranking.chosen.unwrap(),
            deferred: proposal.deferred,
            pattern: None,
            session_id: None,
            increment_id: None,
        };
        let path = run.path.join("increment.toml");
        pipeline_model::write(&path, &increment)?;
        run.record_artifact("increment", &path, None)?;
        run.check_sources()?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            run.finalize(
                "pass",
                Some("candidate-proposal"),
                Some("bounded proposal prepared; reviewed configuration preserved"),
                &["preflight".into(), "candidate-proposal".into()],
            )?;
            Ok(run.path)
        }
        Err(error) => {
            run.finalize("error", Some("candidate-proposal"), Some(&format!("{error:#}")), &[])?;
            Err(error)
        }
    }
}
pub(crate) fn apply(root: &Path, owner: &Path) -> Result<Increment> {
    if owner.join("checks/pattern-promotion.toml").exists() {
        return crate::iteration_pattern::apply(root, owner, false);
    }
    apply_inner(root, owner, false)
}
pub(crate) fn recover_application(root: &Path, owner: &Path) -> Result<Increment> {
    if owner.join("checks/pattern-promotion.toml").exists() {
        return crate::iteration_pattern::apply(root, owner, true);
    }
    apply_inner(root, owner, true)
}
fn apply_inner(root: &Path, owner: &Path, recovery: bool) -> Result<Increment> {
    let (manifest, inventory) = origins::snapshot(owner)?;
    origins::owned(owner, &manifest)?;
    let verdict = origins::document(&owner.join("verdict.toml"))?;
    ensure!(
        manifest["type"]
            == crate::readiness_metadata::value(&"increment-preparation").expect("validated native metadata")
            && manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "preparation did not complete"
    );
    let increment: Increment = pipeline_model::read(&origins::artifact(owner, "increment", &manifest)?)?;
    let proposal: Proposal = pipeline_model::read(&owner.join("checks/candidate-proposal.toml"))?;
    ensure!(proposal.status == "pass" && proposal.complete, "proposal did not pass");
    let _guard = lock(&root.join("out/increment-apply.lock"))?;
    let pointer = origins::document(&root.join("out/accepted.toml"))?;
    ensure!(pointer["run_id"] == manifest["accepted_parent"], "accepted predecessor changed before application");
    let config = root.join("config/functions.toml");
    let proposed = origins::artifact(owner, "proposed-config", &manifest)?;
    ensure!(digest(&proposed)? == proposal.proposed_config_sha256, "prepared configuration checksum differs");
    if recovery && digest(&config)? == proposal.proposed_config_sha256 {
        let mut expected = inventory["files"].clone();
        let row = expected
            .as_array_mut()
            .context("preparation source files missing")?
            .iter_mut()
            .find(|r| {
                r["path"]
                    == crate::readiness_metadata::value(&"config/functions.toml").expect("validated native metadata")
            })
            .context("preparation config missing")?;
        row["sha256"] = proposal.proposed_config_sha256.into();
        ensure!(
            toml::Value::try_from(source_inventory(root)?)? == expected,
            "source differs from interrupted bounded application"
        );
        return Ok(increment);
    }
    ensure!(
        crate::readiness_metadata::value(&source_inventory(root)?).expect("validated native metadata")
            == inventory["files"],
        "user source/config changed since preparation"
    );
    ensure!(
        digest(&config)? == proposal.prior_config_sha256 && digest(&proposed)? == proposal.proposed_config_sha256,
        "prepared configuration checksum differs"
    );
    let temp = root.join(format!("config/.functions.increment-{}.tmp", std::process::id()));
    let result = (|| -> Result<()> {
        let mut output = OpenOptions::new().write(true).create_new(true).open(&temp)?;
        output.write_all(&fs::read(proposed)?)?;
        output.sync_all()?;
        fs::set_permissions(&temp, config.metadata()?.permissions())?;
        ensure!(
            crate::readiness_metadata::value(&source_inventory(root)?).expect("validated native metadata")
                == inventory["files"],
            "source changed during application"
        );
        fs::rename(&temp, &config)?;
        File::open(config.parent().unwrap())?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result?;
    Ok(increment)
}
pub(crate) fn progress(coverage: &Value, kind: &str) -> Result<Progress> {
    let counts = &coverage["progress"];
    let comparable =
        counts["comparable"] == crate::readiness_metadata::value(&true).expect("validated native metadata");
    let generated_delta = counts["generated_delta"].as_integer().unwrap_or(0);
    let pass = matches!(kind, "supported-expansion" | "timed-pattern-expansion")
        && coverage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
        && coverage["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
        && comparable
        && generated_delta > 0;
    Ok (Progress { schema_version : 1 , status : if pass { "pass" } else { "uncovered" } . into () , complete : true , kind : kind . into () , generated_instructions : coverage ["generated_instructions"] . as_integer () . and_then (| n | u64 :: try_from (n) . ok ()) . context ("generated count missing") ? , interpreted_instructions : coverage ["interpreted_instructions"] . as_integer () . and_then (| n | u64 :: try_from (n) . ok ()) . context ("interpreted count missing") ? , comparable , generated_delta , interpreted_delta : counts ["interpreted_delta"] . as_integer () . unwrap_or (0) , validated_keys_delta : counts ["validated_keys_delta"] . as_integer () . unwrap_or (0) , reason : if pass { "comparable observed instruction progress verified" } else { "expansion requires positive observed gain in the identical validated domain; other kinds require independent proof" } . into () })
}
pub(crate) fn progress_gate(run: &mut Run) -> Result<()> {
    let increment: Increment =
        toml_value_decode(Value::Table(run.manifest.increment.clone().context("increment missing")?))?;
    let report = progress(run.checks.get("changed-entry-coverage").context("coverage missing")?, &increment.kind)?;
    run.record_typed_check("increment-progress", &report)
}
pub(crate) fn verify_progress(owner: &Path, verified: &Value) -> Result<()> {
    if verified["manifest"]["type"]
        != crate::readiness_metadata::value(&"iteration").expect("validated native metadata")
    {
        return Ok(());
    }
    let increment: Increment = toml_value_decode(verified["manifest"]["increment"].clone())?;
    ensure!(
        increment.schema_version == 1
            && matches!(increment.kind.as_str(), "supported-expansion" | "timed-pattern-expansion"),
        "unsupported increment proof kind"
    );
    let coverage = &verified["verdict"]["checks"]["changed-entry-coverage"];
    let actual = progress(coverage, &increment.kind)?;
    let report: Progress = toml_value_decode(verified["verdict"]["checks"]["increment-progress"].clone())?;
    let persisted: Progress = pipeline_model::read(&owner.join("checks/increment-progress.toml"))?;
    ensure!(
        report == actual && report == persisted && report.status == "pass",
        "iteration lacks verified positive progress"
    );
    let baseline = reuse::verify_accepted(&increment.baseline_path, true)?;
    ensure!(
        baseline["manifest"]["run_id"] == verified["manifest"]["accepted_parent"]
            && baseline["manifest"]["run_id"] == coverage["predecessor"]["run_id"],
        "iteration predecessor identity differs"
    );
    ensure!(
        crate::readiness_metadata::value(&crate::coverage::progress(
            coverage,
            &baseline["verdict"]["checks"]["changed-entry-coverage"]
        )?)
        .expect("validated native metadata")
            == coverage["progress"],
        "progress delta differs from accepted actual counts"
    );
    let (preparation, _) = origins::snapshot(&increment.preparation_path)?;
    origins::owned(&increment.preparation_path, &preparation)?;
    let verdict = origins::document(&increment.preparation_path.join("verdict.toml"))?;
    ensure!(
        verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && preparation["accepted_parent"] == verified["manifest"]["accepted_parent"],
        "bounded preparation identity differs"
    );
    let stored: Increment =
        pipeline_model::read(&origins::artifact(&increment.preparation_path, "increment", &preparation)?)?;
    let mut candidate = increment.clone();
    candidate.session_id = None;
    candidate.increment_id = None;
    ensure!(candidate == stored, "accepted increment differs from sealed proposal");
    if increment.kind == "timed-pattern-expansion" {
        crate::iteration_pattern::verify_promotion(owner, verified, &increment)?;
    } else {
        ensure!(increment.pattern.is_none(), "supported increment carries a pattern promotion");
    }
    let proposal: Proposal = pipeline_model::read(&increment.preparation_path.join("checks/candidate-proposal.toml"))?;
    ensure!(
        digest(&safe_artifact(&owner.join("source/files"), "config/functions.toml")?)?
            == proposal.proposed_config_sha256,
        "accepted config differs from bounded proposal"
    );
    let expected = rows(&origins::artifact(&increment.preparation_path, "proposed-steps", &preparation)?)?;
    let current: KeyInventory =
        pipeline_model::read(&origins::artifact(owner, "generated-key-inventory", &verified["manifest"])?)?;
    let prior: KeyInventory = pipeline_model::read(&origins::artifact(
        &increment.baseline_path,
        "generated-key-inventory",
        &baseline["manifest"],
    )?)?;
    let expected = index(&expected);
    let current = index(&current.keys);
    let prior = index(&prior.keys);
    let desired = increment.chosen.new_executed_keys.iter().map(|r| (r.address, r.mx)).collect::<BTreeSet<_>>();
    ensure!(
        current.keys().eq(expected.keys())
            && prior.keys().all(|k| current.contains_key(k))
            && current.keys().filter(|k| !prior.contains_key(k)).copied().collect::<BTreeSet<_>>() == desired,
        "accepted table exceeds selected measured slice"
    );
    Ok(())
}
pub(crate) fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let timeout =
        request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400);
    let owner = if let Some(path) = request.get("preparation").and_then(Value::as_str) {
        PathBuf::from(path)
    } else {
        prepare(&root, Path::new(crate::documents::field(&request, "selection")?), timeout)?
    };
    if request.get("prepare_only").and_then(toml::Value::as_bool) == Some(true) {
        return Ok(crate :: report ! ({ "status" : "pass" , "preparation_path" : owner , }));
    }
    let increment = apply(&root, &owner)?;
    readiness::operate(
        crate :: report ! ({ "checkout" : root , "predecessor" : increment . baseline_path , "original_run" : request . get ("original_run") . and_then (Value :: as_str) , "timeout" : timeout , "increment" : increment , }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    use pipeline_model::{Complexity, SelectedKey};
    use std::{
        os::unix::fs::PermissionsExt,
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };
    struct Temp(PathBuf);
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn root() -> Temp {
        let path = std::env::temp_dir()
            .join(format!("sm-native-increment-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir(&path).unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&path).status().unwrap().success());
        fs::create_dir(path.join("config")).unwrap();
        fs::create_dir(path.join("rom")).unwrap();
        let mut rom = vec![0xff; 3 * 1024 * 1024];
        rom[..4].copy_from_slice(&[0xea, 0xca, 0xea, 0x60]);
        rom[32..34].copy_from_slice(&[0xea, 0x60]);
        fs::write(path.join("rom/sm.sfc"), rom).unwrap();
        Temp(path)
    }
    fn chosen(address: u32, opcode: u8, action: SelectionAction) -> Candidate {
        Candidate {
            address: 0x808000,
            mx: 3,
            variant: "m1x1".into(),
            action,
            estimated_compiled_step_gain: 20,
            complexity: Complexity { blocks: 1, instruction_keys: 4 },
            new_executed_keys: vec![SelectedKey {
                address,
                mx: 3,
                opcode,
                bytes: vec![opcode],
                pattern: "observed".into(),
                count: 20,
            }],
            deferred_keys: vec![],
            overlapping_keys: vec![],
        }
    }
    fn executable() -> PathBuf {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let target = root.join("target");
        let output = Command::new("cargo")
            .args(["build", "-p", "recomp", "--locked", "--offline", "--jobs", "1"])
            .env("CARGO_TARGET_DIR", &target)
            .current_dir(root)
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        target.join("debug/recomp")
    }
    #[test]
    fn measured_proposal_and_deferred_revisit_physically_prune_only_unselected_keys() {
        let executable = executable();
        for revisit in [false, true] {
            let root = root();
            let current = root.0.join("config/functions.toml");
            let source = if revisit {
                "# Reviewed routine\n[[function]]\naddr = \"808000\"\nname = \"Existing\"\nvariants = [\"m1x1\", \"m0x0\"]\n\n# Now observed key\n[[excluded_step]]\naddr = \"808001\"\nvariant = \"m1x1\"\nreason = \"deferred\"\n\n# Keep this deferred key\n[[excluded_step]]\naddr = \"808002\"\nvariant = \"m1x1\"\nreason = \"deferred\"\n"
            } else {
                "# User comment retained\n[[function]]\naddr = \"808020\"\nname = \"Legacy\"\nvariants = [\"m1x1\"]\n"
            };
            fs::write(&current, source).unwrap();
            let mut run = Run::create(&root.0, "increment-preparation", crate::report!({}), None).unwrap();
            run.capture_source().unwrap();
            let initial = generate(&mut run, &executable, &current, "baseline", 60).unwrap();
            let baseline = KeyInventory { schema_version: 1, keys: rows(&initial).unwrap() };
            let choice = chosen(
                if revisit { 0x808001 } else { 0x808000 },
                if revisit { 0xca } else { 0xea },
                if revisit { SelectionAction::EnableDeferredKeys } else { SelectionAction::AddRoutineWidth },
            );
            let proposal = propose(&mut run, &baseline, &choice, &executable, "selection", 60).unwrap();
            assert_eq!(fs::read_to_string(&current).unwrap(), source);
            let final_rows = rows(&run.path.join("generated/proposed-steps.rs")).unwrap();
            assert_eq!(final_rows.len(), baseline.keys.len() + 1);
            assert!(final_rows.iter().any(|r| r.address == choice.new_executed_keys[0].address && r.mx == 3));
            assert!(!final_rows.iter().any(|r| r.address == 0x808002 && r.mx == 3));
            let text = fs::read_to_string(proposal).unwrap();
            if revisit {
                assert!(text.contains("# Now observed key"));
                assert!(text.contains("# Keep this deferred key"));
                assert!(text.contains("name = \"Existing\""));
            } else {
                assert!(text.contains("# User comment retained"));
            }
            run.check_sources().unwrap();
        }
    }
    #[test]
    fn progress_rejects_zero_negative_uncomparable_and_self_certified_repairs() {
        let mut coverage = crate :: report ! ({ "status" : "pass" , "complete" : true , "generated_instructions" : 20 , "interpreted_instructions" : 80 , "progress" : crate :: report ! ({ "comparable" : true , "generated_delta" : 10 , "interpreted_delta" : - 10 , "validated_keys_delta" : 1 , }) , });
        assert_eq!(progress(&coverage, "supported-expansion").unwrap().status, "pass");
        assert_eq!(progress(&coverage, "correctness-fix").unwrap().status, "uncovered");
        for delta in [0, -1] {
            coverage["progress"]["generated_delta"] = delta.into();
            assert_eq!(progress(&coverage, "supported-expansion").unwrap().status, "uncovered");
        }
        coverage["progress"]["generated_delta"] = 10.into();
        coverage["progress"]["comparable"] = false.into();
        assert_eq!(progress(&coverage, "supported-expansion").unwrap().status, "uncovered");
    }
    #[test]
    fn atomic_application_preserves_mode_and_rejects_tampered_proposal_or_changed_source() {
        let root = root();
        let config = root.0.join("config/functions.toml");
        fs::write(&config, "# user config\n").unwrap();
        fs::set_permissions(&config, fs::Permissions::from_mode(0o640)).unwrap();
        let mut run = Run::create(&root.0, "increment-preparation", crate::report!({}), Some("parent")).unwrap();
        run.capture_source().unwrap();
        let proposed = run.path.join("generated/functions.toml");
        let bytes = b"# user config\n[[function]]\naddr = \"808000\"\nvariants = [\"m1x1\"]\n";
        fs::write(&proposed, bytes).unwrap();
        let report = Proposal {
            schema_version: 1,
            status: "pass".into(),
            complete: true,
            selection_sha256: "selection".into(),
            prior_config_sha256: digest(&config).unwrap(),
            proposed_config_sha256: digest(&proposed).unwrap(),
            steps_sha256: "steps".into(),
            actual_new_keys: vec![Key { address: 0x808000, mx: 3 }],
            deferred: vec![],
            preserved_baseline_keys: 0,
        };
        run.record_typed_check("candidate-proposal", &report).unwrap();
        run.record_artifact("proposed-config", &proposed, None).unwrap();
        let increment = Increment {
            schema_version: 1,
            kind: "supported-expansion".into(),
            baseline_path: "parent".into(),
            selection_path: "selection".into(),
            selection_sha256: "selection".into(),
            preparation_path: run.path.clone(),
            chosen: chosen(0x808000, 0xea, SelectionAction::AddRoutineWidth),
            pattern: None,
            deferred: vec![],
            session_id: None,
            increment_id: None,
        };
        pipeline_model::write(&run.path.join("increment.toml"), &increment).unwrap();
        run.record_artifact("increment", &run.path.join("increment.toml"), None).unwrap();
        run.finalize("pass", None, None, &["candidate-proposal".into()]).unwrap();
        crate::atomic_toml(&root.0.join("out/accepted.toml"), &crate :: report ! ({ "run_id" : "parent" , })).unwrap();
        let pointer = fs::read(root.0.join("out/accepted.toml")).unwrap();
        fs::write(&proposed, "tampered").unwrap();
        assert!(apply(&root.0, &run.path).is_err());
        assert_eq!(fs::read_to_string(&config).unwrap(), "# user config\n");
        fs::write(&proposed, bytes).unwrap();
        fs::write(&config, "user edit").unwrap();
        assert!(apply(&root.0, &run.path).is_err());
        fs::write(&config, "# user config\n").unwrap();
        assert_eq!(apply(&root.0, &run.path).unwrap(), increment);
        assert_eq!(fs::read(config.clone()).unwrap(), bytes);
        assert_eq!(config.metadata().unwrap().permissions().mode() & 0o777, 0o640);
        assert_eq!(fs::read(root.0.join("out/accepted.toml")).unwrap(), pointer);
        assert_eq!(fs::read_dir(root.0.join("config")).unwrap().count(), 1);
    }
}
