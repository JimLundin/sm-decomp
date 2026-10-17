use crate::atomic_toml;
use crate::diagnostics;
use crate::digest;
use crate::parity;
use crate::patterns;
use crate::preflight;
use crate::readiness_contract as contract;
use crate::readiness_feedback;
use crate::readiness_origins as origins;
use crate::readiness_reuse as reuse;
use crate::readiness_run::Run;
use crate::readiness_run::identity;
use crate::readiness_run::safe_artifact;
use crate::readiness_sync;
use anyhow::{Context, Result, ensure};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use toml::Value;
fn missing(run: &mut Run, name: &str, reason: &str) -> Result<Value> {
    if let Some(report) = run.checks.get(name) {
        return Ok(report.clone());
    }
    let report = crate :: report ! ({ "status" : if run . was_interrupted () { "incomplete" } else { "error" } , "complete" : false , "reason" : reason , });
    run.record_check(name, report.clone())?;
    Ok(report)
}
fn record_acceptance_failure(
    owner: &Path,
    verdict: &mut Value,
    error: &anyhow::Error,
    publication: bool,
) -> Result<()> {
    let mut terminal: pipeline_model::readiness::Verdict = verdict.clone().try_into()?;
    terminal.status = "error".into();
    terminal.acceptance_eligible = false;
    terminal.reason = Some(format!(
        "{} failed: {error:#}",
        if publication { "acceptance publication" } else { "origin verification" }
    ));
    atomic_toml(&owner.join("verdict.toml"), &terminal)?;
    if publication {
        atomic_toml(
            &owner.join("acceptance-result.toml"),
            &crate::report!({"status":"error","accepted":false,"reason":format!("{error:#}")}),
        )?;
    }
    *verdict = Value::try_from(terminal)?;
    Ok(())
}
fn complete_report(run: &Run, name: &str) -> Result<Value> {
    run.checks.get(name).cloned().context("gate completion report missing")
}
pub fn verify_check(owner: &Path, name: &str, verified: &Value) -> Result<Value> {
    verify_check_chain(owner, name, verified, &mut BTreeSet::new())
}
fn verify_check_chain(owner: &Path, name: &str, verified: &Value, visited: &mut BTreeSet<PathBuf>) -> Result<Value> {
    let owner = fs::canonicalize(owner)?;
    ensure!(visited.insert(owner.clone()), "cyclic reused evidence origin");
    let manifest = &verified["manifest"];
    let report = &verified["verdict"]["checks"][name];
    contract::validate(name, report)?;
    if let Some(prior) = report.get("reused_from") {
        ensure!(contract::REUSABLE.contains(&name), "candidate-specific check cannot reuse predecessor evidence");
        let previous = Path::new(crate::documents::field(prior, "path")?);
        let accepted = reuse::verify_accepted(previous, true)?;
        let before = crate::documents::field(&accepted["manifest"], "started_utc")?;
        let current = crate::documents::field(manifest, "started_utc")?;
        ensure!(
            before < current && accepted["manifest"]["run_id"] == prior["run_id"],
            "reused evidence must precede current run and match origin"
        );
        let old = origins::document(&origins::artifact(previous, "dependency-policy", &accepted["manifest"])?)?;
        let current = origins::document(&origins::artifact(&owner, "dependency-policy", manifest)?)?;
        ensure!(
            old["checks"][name] == current["checks"][name]
                && identity(&old["checks"][name])? == crate::documents::field(prior, "dependency_sha256")?,
            "reused check dependencies differ"
        );
        let actual = verify_check_chain(previous, name, &accepted, visited)?;
        ensure!(
            normalized_report(report)? == normalized_report(&actual)?,
            "reused check differs from original evidence"
        );
        return Ok(report.clone());
    }
    if ["check", "lint"].contains(&name) {
        let process = origins::process(&owner, name, manifest, Some(&crate::report!(vec!["just", name])), &[0])?;
        ensure!(process == *report, "successful check process differs from gate");
    } else if ["fallback-only-parity", "generated-parity"].contains(&name) {
        let mode = if name == "fallback-only-parity" { "fallback-only" } else { "generated" };
        let raw_path = safe_artifact(&owner, crate::documents::field(report, "evidence")?)?;
        let raw = origins::document(&raw_path)?;
        let saved = origins::artifact(&owner, &format!("{mode}-parity-report"), manifest)?;
        ensure!(saved == raw_path, "parity report provenance differs");
        contract::validate(name, &raw)?;
        let binary = origins::artifact(&owner, &format!("{mode}-binary"), manifest)?;
        let inputs = &manifest["identities"]["inputs"];
        ensure!(
            raw["mode"] == crate::readiness_metadata::value(&mode).expect("validated native metadata")
                && raw["seed"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
                && raw["controller"] == crate::readiness_metadata::value(&"P1").expect("validated native metadata")
                && raw["profile"]
                    == crate::readiness_metadata::value(&"lsnes-profile").expect("validated native metadata")
                && raw["regions"] == crate::report!(vec!["CPU+clocks", "WRAM", "framebuffer"])
                && raw["available_frames"] == inputs["frames"]
                && raw["identities"]["binary_sha256"]
                    == crate::readiness_metadata::value(&digest(&binary)?).expect("validated native metadata"),
            "parity actual executable/scope differs"
        );
        let canonical = safe_artifact(&owner, crate::documents::field(inputs, "input_evidence")?)?;
        ensure!(
            digest(&canonical)? == crate::documents::field(inputs, "input_sha256")?,
            "parity canonical input differs"
        );
        let selected =
            fs::read_to_string(&canonical)?.lines().take(18000).map(|line| format!("{line}\n")).collect::<String>();
        use sha2::{Digest, Sha256};
        ensure!(
            raw["identities"]["selected_input_sha256"]
                == crate::readiness_metadata::value(&format!("{:x}", Sha256::digest(selected.as_bytes())))
                    .expect("validated native metadata"),
            "selected parity input differs"
        );
        for key in ["rom_sha256", "movie_sha256", "input_sha256"] {
            ensure!(raw["identities"][key] == inputs[key], "parity input identity differs");
        }
        if mode == "generated" {
            ensure!(raw["identities"]["compiled_rom_sha256"] == inputs["rom_sha256"], "compiled ROM identity differs");
        }
        let process = origins::process(&owner, &format!("{mode}-replay"), manifest, None, &[0])?;
        let command = crate::pattern_contract::strings(&process["command"])?;
        let cwd = Path::new(crate::documents::field(&process, "cwd")?);
        let expected = vec![
            binary.to_string_lossy().into_owned(),
            cwd.join("movies/super-metroid-any-3653.lsmv").to_string_lossy().into_owned(),
            format!("--{mode}"),
            "--frames".into(),
            "18000".into(),
            "--rom".into(),
            cwd.join("rom/sm.sfc").to_string_lossy().into_owned(),
            "--out".into(),
            raw_path.parent().unwrap().to_string_lossy().into_owned(),
        ];
        ensure!(command == expected, "cold parity process invocation differs");
        for item in ["game", "steps"] {
            let path = origins::artifact(&owner, &format!("{mode}-generated-{item}"), manifest)?;
            if mode == "fallback-only" {
                let expected = if item == "steps" {
                    "pub static STEP_TABLE: &[runtime::StepEntry] = &[];\npub const ROM_SHA256: Option<&str> = None;\n"
                } else {
                    "pub static TABLE: &[runtime::TableEntry] = &[];\n"
                };
                ensure!(fs::read_to_string(path)? == expected, "fallback contains compiled entries");
            }
        }
        for field in [
            "status",
            "complete",
            "expected_frames",
            "completed_frames",
            "matched_frames",
            "generated_steps",
            "interpreted_steps",
        ] {
            ensure!(raw[field] == report[field], "parity gate summary differs from raw completion");
        }
    } else if name == "synchronization" {
        let original = if owner.join("generated/original.csv").exists() {
            owner.join("generated/original.csv")
        } else {
            let reused = crate::documents::array(manifest, "reused_evidence")?
                .iter()
                .find(|r| {
                    r["kind"]
                        == crate::readiness_metadata::value(&"original-oracle").expect("validated native metadata")
                })
                .context("original reuse origin missing")?;
            Path::new(crate::documents::field(reused, "path")?).join("generated/original.csv")
        };
        let computed = readiness_sync::compare(
            &original,
            &owner.join("generated/candidate.csv"),
            &manifest["identities"]["inputs"],
            18000,
        )?;
        for field in [
            "status",
            "complete",
            "events",
            "expected_frames",
            "completed_frames",
            "original_sha256",
            "candidate_sha256",
        ] {
            ensure!(computed[field] == report[field], "synchronization differs from independent captures");
        }
    }
    Ok(report.clone())
}
fn normalized_report(report: &Value) -> Result<Value> {
    let mut summary = report.clone();
    summary.as_table_mut().context("check report must be a table")?.remove("reused_from");
    Ok(summary)
}
fn focus_gate(run: &mut Run, timeout: u64) -> Result<Value> {
    let coverage = complete_report(run, "changed-entry-coverage")?;
    ensure!(
        coverage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "coverage must pass before mandatory focus"
    );
    let keys = if crate::documents::array(&coverage, "required_keys")?.is_empty() {
        let row = crate::documents::array(&coverage, "executed")?.first().context("no compiled key executed")?;
        crate::report!(vec![crate :: report ! ({ "address" : row ["address"] , "mx" : row ["mx"] , })])
    } else {
        Value::Array(
            crate::documents::array(&coverage, "required_keys")?
                .iter()
                .map(|r| crate :: report ! ({ "address" : r ["address"] , "mx" : r ["mx"] , }))
                .collect::<Vec<_>>(),
        )
    };
    let discovery = diagnostics::discover(run, &keys, 18000, Duration::from_secs(timeout))?;
    ensure!(
        discovery["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "covered instruction discovery failed"
    );
    let mut intervals = crate::documents::array(&discovery, "intervals")?.clone();
    intervals.sort_by_key(|i| {
        (
            i["start_input"].as_integer().and_then(|n| u64::try_from(n).ok()),
            i["start_instruction"].as_integer().and_then(|n| u64::try_from(n).ok()),
        )
    });
    let mut merged: Vec<Value> = Vec::new();
    for mut interval in intervals {
        {
            let value: toml::Value = crate::report!(vec![interval["key"]]);
            interval.as_table_mut().context("native metadata table missing")?.insert(("keys").into(), value);
        };
        if let Some(last) = merged.last_mut()
            && last["start_input"] == interval["start_input"]
            && interval["start_instruction"]
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .context("interval start")?
                <= last["start_instruction"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap()
                    + last["instructions"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap()
        {
            let end = (last["start_instruction"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap()
                + last["instructions"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap())
            .max(
                interval["start_instruction"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap()
                    + interval["instructions"]
                        .as_integer()
                        .and_then(|n| u64::try_from(n).ok())
                        .context("interval extent")?,
            );
            last["instructions"] = Value::Integer(i64::try_from(
                end - last["start_instruction"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap(),
            )?);
            last["keys"].as_array_mut().unwrap().extend(crate::documents::array(&interval, "keys")?.clone());
        } else {
            merged.push(interval);
        }
    }
    let mut reports = Vec::new();
    let mut validations = Vec::new();
    for (index, interval) in merged.iter().enumerate() {
        let namespace = format!("mandatory-focus-{index}-");
        let request = crate :: report ! ({ "start_input" : interval ["start_input"] , "start_instruction" : interval ["start_instruction"] , "instructions" : interval ["instructions"] , "timeout" : timeout , "cache" : run . path . join ("cache") . join (format ! ("mandatory-focus-{index}")) , });
        let report = diagnostics::diagnose(run, &request, &namespace)?;
        if report.get("cache").and_then(|cache| cache.get("status")).and_then(Value::as_str) == Some("pass") {
            validations.push(report["cache"]["validation"].clone());
        }
        reports.push(report);
    }
    ensure!(!reports.is_empty(), "mandatory focus lacks covered intervals");
    let status = reports
        .iter()
        .find(|r| r["status"] != crate::readiness_metadata::value(&"pass").expect("validated native metadata"))
        .and_then(|r| r["status"].as_str())
        .unwrap_or("pass");
    let expected = reports
        .iter()
        .map(|r| {
            r.get("expected_instructions").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(0)
        })
        .sum::<u64>();
    let completed = reports
        .iter()
        .map(|r| {
            r.get("completed_instructions").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(0)
        })
        .sum::<u64>();
    let report = crate :: report ! ({ "schema_version" : 1 , "status" : status , "complete" : reports . iter () . all (| r | r ["complete"] == Value :: Boolean (true)) , "expected_instructions" : expected , "completed_instructions" : completed , "intervals" : merged , "reports" : reports , "discovery_evidence" : "diagnostics/discovery/discovery.toml" , "replay" : "warm" , "warm_replay_seconds" : reports . iter () . map (| r | r.get("warm_replay_seconds").and_then(Value::as_float) . unwrap_or (0.0)) . sum :: < f64 > () , "total_seconds" : reports . iter () . map (| r | r.get("total_seconds").and_then(Value::as_float) . unwrap_or (0.0)) . sum :: < f64 > () , "evidence" : "diagnostics/focused-summary.toml" , });
    atomic_toml(&run.path.join("diagnostics/focused-summary.toml"), &report)?;
    run.record_check("instruction-diagnosis", report.clone())?;
    if validations.len() == reports.len() {
        let mut validation = validations[0].clone();
        {
            let value: toml::Value = Value::Array(validations);
            validation.as_table_mut().context("native metadata table missing")?.insert(("validations").into(), value);
        };
        {
            let value: toml::Value = Value::Array(merged);
            validation.as_table_mut().context("native metadata table missing")?.insert(("intervals").into(), value);
        };
        {
            let value: toml::Value = "diagnostics/mandatory-focus-0-cache/validation.toml".into();
            validation.as_table_mut().context("native metadata table missing")?.insert(("evidence").into(), value);
        };
        run.record_check("checkpoint-validation", validation)?;
    }
    Ok(report)
}
fn verify_focus(
    owner: &Path,
    manifest: &Value,
    binary: &Path,
    process: &Value,
    actual: &Value,
    interval: &Value,
    cache: &Path,
) -> Result<()> {
    let start = interval["start_input"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .context("focus interval input missing")?;
    let skip = interval["start_instruction"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .context("focus interval instruction missing")?;
    let count = interval["instructions"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .filter(|n| *n > 0)
        .context("focus interval count missing")?;
    ensure!(
        actual["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && actual["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && actual["scope"]
                == crate::readiness_metadata::value(&"instruction-count").expect("validated native metadata")
            && actual["start_input_index"]
                == crate::readiness_metadata::value(&start).expect("validated native metadata")
            && actual["start_instruction"]
                == crate::readiness_metadata::value(&skip).expect("validated native metadata")
            && actual["expected_instructions"]
                == crate::readiness_metadata::value(&count).expect("validated native metadata")
            && actual["completed_instructions"]
                == crate::readiness_metadata::value(&count).expect("validated native metadata"),
        "focused completed interval differs from measured discovery"
    );
    let command = crate::pattern_contract::strings(&process["command"])?;
    ensure!(command.len() == 16, "focused invocation has unexpected flags");
    let cwd = Path::new(crate::documents::field(process, "cwd")?);
    ensure!(
        command[0] == binary.to_string_lossy()
            && command[1] == cwd.join("movies/super-metroid-any-3653.lsmv").to_string_lossy()
            && command[2] == "--out"
            && command[4] == "--start-input"
            && command[5] == start.to_string()
            && command[6] == "--start-instruction"
            && command[7] == skip.to_string()
            && command[8] == "--instructions"
            && command[9] == count.to_string()
            && command[10] == "--rom"
            && command[11] == cwd.join("rom/sm.sfc").to_string_lossy()
            && command[12] == "--cache"
            && command[14] == "--identities",
        "focused actual invocation differs"
    );
    ensure!(
        origins::document(&Path::new(&command[3]).join("instruction.toml"))? == *actual,
        "focused output differs from executed command"
    );
    let identity_relative =
        Path::new(&command[15]).strip_prefix(owner)?.to_str().context("focused identity path invalid")?;
    let name = cache
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.strip_suffix("-cache"))
        .context("focused cache namespace missing")?;
    ensure!(command[13] == owner.join("cache").join(name).to_string_lossy(), "focused cache invocation differs");
    let supplied: pipeline_model::CheckpointIdentities =
        pipeline_model::read(&safe_artifact(owner, identity_relative)?)?;
    let mut validation: pipeline_model::PairValidation = pipeline_model::read(&cache.join("validation.toml"))?;
    validation.identities.dispatch_sha256 = None;
    ensure!(validation.identities == supplied, "focused checkpoint identities differ from supplied invocation");
    let ids = &manifest["identities"];
    let inputs = &ids["inputs"];
    ensure!(
        supplied.source_sha256 == crate::documents::field(&ids["source"], "sha256")?
            && supplied.hardware_sha256 == crate::documents::field(&ids["hardware"], "source_sha256")?
            && supplied.compiler == crate::documents::field(&ids["compiler"], "version")?
            && supplied.features == crate::pattern_contract::strings(&ids["hardware"]["features"])?
            && supplied.settings.seed == 0
            && supplied.settings.mode.as_deref() == Some("generated")
            && supplied.settings.disabled_keys.is_empty()
            && supplied.settings.candidate_sha256 == Some(digest(binary)?),
        "focused source/hardware/settings differ"
    );
    for (key, value) in [
        ("rom_sha256", &supplied.rom_sha256),
        ("movie_sha256", &supplied.movie_sha256),
        ("input_sha256", &supplied.input_sha256),
    ] {
        ensure!(Some(value.as_str()) == inputs[key].as_str(), "focused checkpoint input differs");
    }
    Ok(())
}
pub fn verify_origins(owner: &Path, verified: &Value) -> Result<()> {
    let manifest = &verified["manifest"];
    preflight::authenticate(owner, manifest)?;
    let checks = &verified["verdict"]["checks"];
    for name in contract::MANDATORY {
        contract::validate(name, &checks[name])?;
    }
    ensure!(
        checks["preflight"]["failures"] == crate::report!(vec![])
            && checks["input-identity"]["status"]
                == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && checks["hardware-identity"]["status"]
                == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "preflight material identity absent"
    );
    origins::process(owner, "doctor", manifest, Some(&crate::report!(vec!["just", "doctor"])), &[0])?;
    let compiler = origins::process(
        owner,
        "compiler",
        manifest,
        Some(&crate::report!(vec!["rustc", "--version", "--verbose"])),
        &[0],
    )?;
    ensure!(
        fs::read_to_string(owner.join(crate::documents::field(&compiler, "stdout")?))?.trim()
            == crate::documents::field(&manifest["identities"]["compiler"], "version")?,
        "compiler identity differs"
    );
    for name in ["check", "lint", "fallback-only-parity", "generated-parity", "synchronization"] {
        verify_check(owner, name, verified)?;
    }
    let patterns = patterns::authenticate(
        crate :: report ! ({ "output" : owner . join ("checks/patterns") , "run" : owner , "timing" : true , }),
    )?;
    ensure!(
        patterns["source_sha256"] == manifest["identities"]["source"]["sha256"],
        "pattern candidate source differs"
    );
    let coverage = &checks["changed-entry-coverage"];
    let previous = if coverage.get("predecessor").is_none_or(|v| v.as_table().is_some_and(|t| t.is_empty())) {
        None
    } else {
        let path = PathBuf::from(crate::documents::field(&coverage["predecessor"], "run_path")?);
        let accepted = reuse::verify_accepted(&path, true)?;
        ensure!(
            accepted["manifest"]["run_id"] == coverage["predecessor"]["run_id"],
            "coverage predecessor lineage differs"
        );
        Some(path)
    };
    let profile = owner.join("checks/generated-profile-bound.toml");
    let inventory = owner.join("generated/key-inventory.toml");
    ensure!(
        crate::coverage::derive(owner, manifest, &profile, &inventory, previous.as_deref())? == *coverage,
        "coverage gate differs from independently derived actual keys/counts"
    );
    ensure!(
        digest(&profile)? == crate::documents::field(coverage, "profile_sha256")?
            && digest(&inventory)? == crate::documents::field(coverage, "candidate_inventory_sha256")?
            && coverage["candidate_source_sha256"] == manifest["identities"]["source"]["sha256"],
        "coverage profile/inventory/source differs"
    );
    let inventory_doc = origins::document(&inventory)?;
    let steps = safe_artifact(owner, crate::documents::field(&inventory_doc, "steps_evidence")?)?;
    ensure!(
        digest(&steps)? == crate::documents::field(&inventory_doc, "steps_sha256")?
            && crate::readiness_metadata::value(&crate::coverage::table_rows(&fs::read_to_string(steps)?)?)
                .expect("validated native metadata")
                == inventory_doc["keys"],
        "coverage actual table/body inventory differs"
    );
    let raw_profile = owner.join("diagnostics/generated-frame-parity/coverage.toml");
    let raw = origins::document(&owner.join("diagnostics/generated-frame-parity/parity.toml"))?;
    ensure!(
        digest(&raw_profile)? == crate::documents::field(&raw, "coverage_sha256")?
            && origins::document(&raw_profile)?["generated_instructions"] == coverage["generated_instructions"],
        "coverage raw replay totals differ"
    );
    let discovery = origins::document(&safe_artifact(
        owner,
        crate::documents::field(&checks["instruction-diagnosis"], "discovery_evidence")?,
    )?)?;
    ensure!(
        discovery["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && discovery["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && discovery["unexecuted"] == crate::report!(vec![]),
        "focused key discovery incomplete"
    );
    for key in ["rom_sha256", "movie_sha256", "input_sha256"] {
        ensure!(
            discovery["identities"][key] == manifest["identities"]["inputs"][key],
            "focused discovery input differs"
        );
    }
    let reports = crate::documents::array(&checks["instruction-diagnosis"], "reports")?;
    let intervals = crate::documents::array(&checks["instruction-diagnosis"], "intervals")?;
    ensure!(!reports.is_empty() && reports.len() == intervals.len(), "focus lacks covered intervals");
    for (index, report) in reports.iter().enumerate() {
        let actual = origins::document(&safe_artifact(owner, crate::documents::field(report, "evidence")?)?)?;
        let binary = origins::artifact(owner, &format!("mandatory-focus-{index}-focused-binary"), manifest)?;
        let process =
            origins::process(owner, &format!("mandatory-focus-{index}-focused-replay"), manifest, None, &[0])?;
        ensure!(
            process["command"][0]
                == crate::readiness_metadata::value(&binary.to_string_lossy().as_ref())
                    .expect("validated native metadata")
                && actual["binary_sha256"]
                    == crate::readiness_metadata::value(&digest(&binary)?).expect("validated native metadata")
                && actual["disabled_keys"] == crate::report!(vec![])
                && actual["replay"] == crate::readiness_metadata::value(&"warm").expect("validated native metadata")
                && actual["completed_instructions"] == report["completed_instructions"],
            "focused actual candidate execution differs"
        );
        for key in ["rom_sha256", "movie_sha256", "input_sha256"] {
            ensure!(actual["identities"][key] == manifest["identities"]["inputs"][key], "focus input identity differs");
        }
        for key in crate::documents::array(&intervals[index], "keys")? {
            ensure!(
                crate::documents::array(&actual, "executed_generated_keys")?
                    .contains(&crate::report!(vec![key["address"], key["mx"]])),
                "focused interval did not execute target key"
            );
        }
        let cache = owner.join(format!("diagnostics/mandatory-focus-{index}-cache"));
        crate::paired_verify::verify_files(&cache, &actual)?;
        verify_focus(owner, manifest, &binary, &process, &actual, &intervals[index], &cache)?;
    }
    readiness_feedback::verify(owner, manifest, &checks["feedback-demonstrations"])?;
    crate::iteration_increment::verify_progress(owner, verified)?;
    Ok(())
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let previous = request.get("predecessor").and_then(Value::as_str).map(PathBuf::from);
    let parent = previous.as_ref().and_then(|p| p.file_name()).and_then(|s| s.to_str());
    let iteration = request.get("increment").is_some();
    let run = Run::create(&root, if iteration { "iteration" } else { "readiness" }, contract::domain(), parent)?;
    operate_run(request, run)
}
pub(crate) fn operate_run(request: Value, mut run: Run) -> Result<Value> {
    let root = run.root.clone();
    let previous = request.get("predecessor").and_then(Value::as_str).map(PathBuf::from);
    let iteration = request.get("increment").is_some();
    if iteration {
        run.manifest.increment = Some(request["increment"].as_table().context("increment table missing")?.clone());
    }
    if let Some(expected) = request.get("expected_pointer").and_then(Value::as_str) {
        run.manifest.expected_pointer = Some(expected.into());
    }
    run.manifest.contract_version = Some(contract::VERSION);
    run.save()?;
    eprintln!("Run: {}", run.path.display());
    let timeout =
        request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400);
    let outcome = (|| -> Result<()> {
        let failures = preflight::verify(&mut run)?;
        ensure!(failures.is_empty(), "preflight failed: {}", failures.join("; "));
        if let Some(owner) = &previous {
            reuse::verify_accepted(owner, true)?;
        }
        reuse::record_policy(&mut run)?;
        let plan = reuse::plan(&run, previous.as_deref())?;
        atomic_toml(&run.path.join("checks/reuse-decisions.toml"), &plan)?;
        for name in ["check", "lint"] {
            if plan[name]["action"] == crate::readiness_metadata::value(&"reuse").expect("validated native metadata") {
                reuse::apply(&mut run, name, &plan[name])?;
            } else {
                let root = run.root.clone();
                run.stage(
                    name,
                    &["just".into(), name.into()],
                    &root,
                    &BTreeMap::new(),
                    Duration::from_secs(timeout),
                    &BTreeMap::new(),
                )?;
            }
        }
        let output = run.path.join("checks/patterns");
        fs::create_dir_all(&output)?;
        let mut instruction = crate::report!({});
        patterns::produce(
            &mut run,
            &output,
            &crate :: report ! ({ "timing" : true , "batch_size" : 10000 , }),
            &mut instruction,
        )?;
        let report = patterns::authenticate(crate :: report ! ({ "output" : output , "timing" : true , }))?;
        run.record_check("instruction", report)?;
        run.record_check("timing", origins::document(&output.join("timing.toml"))?)?;
        for mode in ["fallback-only", "generated"] {
            let name = format!("{mode}-parity");
            if plan[&name]["action"] == crate::readiness_metadata::value(&"reuse").expect("validated native metadata") {
                reuse::apply(&mut run, &name, &plan[&name])?;
            } else {
                parity::replay(
                    &mut run,
                    mode,
                    18000,
                    Duration::from_secs(timeout),
                    if mode == "generated" { previous.as_deref() } else { None },
                )?;
            }
        }
        if plan["synchronization"]["action"]
            == crate::readiness_metadata::value(&"reuse").expect("validated native metadata")
        {
            reuse::apply(&mut run, "synchronization", &plan["synchronization"])?;
        } else {
            readiness_sync::synchronize(
                &mut run,
                request.get("original_run").and_then(Value::as_str).map(Path::new),
                18000,
            )?;
        }
        readiness_feedback::demonstrations(&mut run, timeout)?;
        focus_gate(&mut run, timeout)?;
        if iteration {
            crate::iteration_increment::progress_gate(&mut run)?;
        }
        Ok(())
    })();
    if let Err(error) = outcome {
        for name in contract::MANDATORY {
            missing(&mut run, name, &format!("{error:#}"))?;
        }
    }
    for name in contract::MANDATORY {
        missing(&mut run, name, "mandatory gate did not complete")?;
    }
    let success = run.checks.values().all(|r| {
        r["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && r["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
    }) && contract::MANDATORY.iter().all(|name| contract::validate(name, &run.checks[*name]).is_ok());
    let mut mandatory = contract::MANDATORY.map(str::to_owned).to_vec();
    if iteration {
        mandatory.push("increment-progress".into());
    }
    let status = if success {
        "pass"
    } else if run.was_interrupted() {
        "incomplete"
    } else {
        run.checks
            .values()
            .filter_map(|r| r["status"].as_str())
            .find(|s| ["divergence", "uncovered"].contains(s))
            .unwrap_or("error")
    }
    .to_owned();
    let mut verdict = run.finalize(&status, Some("readiness"), None, &mandatory)?;
    if success {
        let verified = crate :: report ! ({ "manifest" : run . manifest , "inventory" : origins :: document (& run . path . join ("source/inventory.toml")) ? , "verdict" : verdict , });
        match verify_origins(&run.path, &verified) {
            Ok(()) => {
                verdict["acceptance_eligible"] = true.into();
                atomic_toml(&run.path.join("verdict.toml"), &verdict)?;
                match reuse::publish(&run.path, &root) {
                    Ok(pointer) => {
                        atomic_toml(
                            &run.path.join("acceptance-result.toml"),
                            &crate :: report ! ({ "status" : "pass" , "accepted" : true , "pointer" : pointer , }),
                        )?;
                    }
                    Err(error) => {
                        record_acceptance_failure(&run.path, &mut verdict, &error, true)?;
                    }
                }
            }
            Err(error) => {
                record_acceptance_failure(&run.path, &mut verdict, &error, false)?;
            }
        }
    }
    Ok(verdict)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn final_origin_and_publication_refusals_retain_durable_error_without_acceptance() {
        let root = std::env::temp_dir().join(format!(
            "sm-readiness-final-refusal-{}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        assert!(std::process::Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        fs::create_dir(root.join("crates")).unwrap();
        fs::write(root.join("crates/control.rs"), "// private final refusal fixture\n").unwrap();
        for publication in [false, true] {
            let mut run = Run::create(&root, "readiness", contract::domain(), None).unwrap();
            run.manifest.contract_version = Some(contract::VERSION);
            run.capture_source().unwrap();
            let mut verdict = run.finalize("pass", Some("readiness"), None, &[]).unwrap();
            assert!(verdict.get("reason").is_none());
            if publication {
                verdict["acceptance_eligible"] = true.into();
                atomic_toml(&run.path.join("verdict.toml"), &verdict).unwrap();
            }
            let (manifest, inventory) = origins::snapshot(&run.path).unwrap();
            origins::owned(&run.path, &manifest).unwrap();
            let error = if publication {
                reuse::publish(&run.path, &root).unwrap_err()
            } else {
                verify_origins(
                    &run.path,
                    &crate::report!({"manifest":manifest,"inventory":inventory,"verdict":verdict}),
                )
                .unwrap_err()
            };
            if publication {
                assert!(error.to_string().contains("mandatory registry"));
            } else {
                assert!(
                    error
                        .downcast_ref::<std::io::Error>()
                        .is_some_and(|error| { error.kind() == std::io::ErrorKind::NotFound })
                );
            }
            record_acceptance_failure(&run.path, &mut verdict, &error, publication).unwrap();
            let saved: pipeline_model::readiness::Verdict =
                pipeline_model::read(&run.path.join("verdict.toml")).unwrap();
            assert_eq!(saved.status, "error");
            assert!(!saved.acceptance_eligible);
            assert!(saved.reason.as_deref().is_some_and(|reason| reason.contains(&error.to_string())));
            assert_eq!(verdict, Value::try_from(&saved).unwrap());
            let manifest: pipeline_model::readiness::RunManifest =
                pipeline_model::read(&run.path.join("manifest.toml")).unwrap();
            assert_eq!(manifest.state, "complete");
            assert!(!root.join("out/accepted.toml").exists());
            if publication {
                let result = origins::document(&run.path.join("acceptance-result.toml")).unwrap();
                assert_eq!(result.get("accepted").and_then(Value::as_bool), Some(false));
            }
        }
        fs::remove_dir_all(root).unwrap();
    }
}
