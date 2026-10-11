use crate::readiness_metadata::toml_value_decode;
use crate::{
    digest, family, preflight, readiness, readiness_contract as contract, readiness_origins as origins,
    readiness_reuse as reuse,
    readiness_run::{Run, safe_artifact},
};
use anyhow::{Context, Result, ensure};
use pipeline_model::{
    AcceptedBaseline, BoundSelection, InputDomain, InstructionProfile, KeyInventory, ProfileIdentities,
    ProfileProvenance, SelectionReport,
};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Duration,
};
use toml::Value;
fn bind_profile(
    run: &mut Run,
    owner: &Path,
    manifest: &Value,
    report_path: &Path,
    profile_path: &Path,
) -> Result<PathBuf> {
    let report = origins::document(report_path)?;
    let profile: InstructionProfile = pipeline_model::read(profile_path)?;
    profile.validate()?;
    let inputs = &manifest["identities"]["inputs"];
    let binary = &manifest["identities"]["generated-binary"];
    let canonical = safe_artifact(owner, family::field(inputs, "input_evidence")?)?;
    ensure!(digest(&canonical)? == family::field(inputs, "input_sha256")?, "baseline canonical input differs");
    let selected = fs::read_to_string(&canonical)?.lines().take(18000).map(|l| format!("{l}\n")).collect::<String>();
    let expected = ProfileIdentities {
        rom_sha256: family::field(inputs, "rom_sha256")?.into(),
        movie_sha256: family::field(inputs, "movie_sha256")?.into(),
        input_sha256: family::field(inputs, "input_sha256")?.into(),
        binary_sha256: family::field(binary, "sha256")?.into(),
        compiled_rom_sha256: family::field(inputs, "rom_sha256")?.into(),
        selected_input_sha256: format!("{:x}", Sha256::digest(selected.as_bytes())),
    };
    let expected_value = toml::Value::try_from(&expected)?;
    ensure!(
        report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && report["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && report["expected_frames"]
                == crate::readiness_metadata::value(&18000).expect("validated native metadata")
            && report["completed_frames"]
                == crate::readiness_metadata::value(&18000).expect("validated native metadata")
            && report["matched_frames"] == crate::readiness_metadata::value(&18000).expect("validated native metadata")
            && report["mode"] == crate::readiness_metadata::value(&"generated").expect("validated native metadata")
            && report["profile"] == inputs["profile"]
            && report["seed"] == inputs["seed"]
            && report["controller"] == crate::readiness_metadata::value(&"P1").expect("validated native metadata")
            && report["regions"] == crate::report!(vec!["CPU+clocks", "WRAM", "framebuffer"])
            && report["available_frames"] == inputs["frames"]
            && report["coverage_sha256"]
                == crate::readiness_metadata::value(&digest(profile_path)?).expect("validated native metadata"),
        "instruction profile lacks complete accepted-prefix replay"
    );
    for (key, value) in expected_value.as_table().unwrap() {
        ensure!(
            report["identities"][key] == crate::readiness_metadata::value(value).expect("validated native metadata"),
            "accepted profile input/binary identity differs"
        );
    }
    let dispatches = report["generated_steps"]
        .as_integer()
        .and_then(|n| u64::try_from(n).ok())
        .context("generated dispatch count missing")?
        .checked_add(
            report["interpreted_steps"]
                .as_integer()
                .and_then(|n| u64::try_from(n).ok())
                .context("interpreted dispatch count missing")?,
        )
        .context("dispatch overflow")?;
    let instructions = profile
        .generated_instructions
        .checked_add(profile.interpreted_instructions)
        .context("instruction count overflow")?;
    ensure!(
        Some(profile.generated_dispatches)
            == report["generated_steps"].as_integer().and_then(|n| u64::try_from(n).ok())
            && Some(profile.interpreted_dispatches)
                == report["interpreted_steps"].as_integer().and_then(|n| u64::try_from(n).ok())
            && Some(profile.noninstruction_dispatches) == dispatches.checked_sub(instructions),
        "profile dispatch totals differ from actual replay"
    );
    let target = run.path.join("generated/selection-profile.toml");
    pipeline_model::write(&target, &profile)?;
    run.record_artifact("selection-original-profile", profile_path, Some("generated/selection-original-profile.toml"))?;
    run.record_artifact("selection-profile", &target, None)?;
    run.guard(&target)?;
    let provenance = ProfileProvenance {
        schema_version: 1,
        baseline: baseline(owner, manifest)?,
        profile_sha256: digest(&target)?,
        original_profile_sha256: digest(profile_path)?,
        report_sha256: digest(report_path)?,
        expected_frames: 18000,
        identities: expected,
    };
    let provenance_path = run.path.join("checks/selection-profile-provenance.toml");
    pipeline_model::write(&provenance_path, &provenance)?;
    run.record_artifact("selection-profile-provenance", &provenance_path, None)?;
    run . record_check ("selection-profile" , crate :: report ! ({ "status" : "pass" , "complete" : true , "evidence" : "checks/selection-profile-provenance.toml" , "sha256" : digest (& provenance_path) ? , })) ? ;
    Ok(target)
}
pub(crate) fn select(run: &mut Run, owner: &Path, timeout: u64) -> Result<BoundSelection> {
    let verified = reuse::verify_accepted(owner, true)?;
    readiness::verify_origins(owner, &verified)?;
    let manifest = &verified["manifest"];
    run.manifest.accepted_parent = Some(family::field(manifest, "run_id")?.into());
    run . manifest . selection_baseline = Some (crate :: report ! ({ "run_id" : manifest ["run_id"] , "run_path" : fs :: canonicalize (owner) ? , "source_sha256" : manifest ["identities"] ["source"] ["sha256"] , "domain" : contract :: domain () , }) . as_table () . unwrap () . clone ()) ;
    run.save()?;
    run.record_check(
        "selection-baseline",
        crate :: report ! ({ "status" : "pass" , "complete" : true , "run_id" : manifest ["run_id"] , }),
    )?;
    let failures = preflight::verify(run)?;
    ensure!(failures.is_empty(), "readiness preflight failed: {}", failures.join("; "));
    for key in ["rom_sha256", "movie_sha256", "input_sha256", "seed", "profile", "region", "controller", "reset"] {
        ensure!(
            run.manifest.identities["inputs"][key] == manifest["identities"]["inputs"][key],
            "current selection input domain differs"
        );
    }
    for relative in ["config/functions.toml", "crates/recomp/src/step.rs"] {
        ensure!(
            digest(&run.root.join(relative))? == digest(&safe_artifact(&owner.join("source/files"), relative)?)?,
            "selection reviewed routines/timed support differ from accepted baseline"
        );
    }
    let shared = |files: &Value| -> Result<Value> {
        Ok(Value::Array(
            family::array(files, "files")?
                .iter()
                .filter(|r| {
                    r["path"].as_str().is_some_and(|p| {
                        ["crates/snes/src/", "crates/runtime/src/", "crates/diff/src/", "tools/patches/"]
                            .iter()
                            .any(|prefix| p.starts_with(prefix))
                    })
                })
                .cloned()
                .collect::<Vec<_>>(),
        ))
    };
    ensure!(
        shared(&origins::document(&run.path.join("source/inventory.toml"))?)? == shared(&verified["inventory"])?,
        "selection shared execution inputs differ"
    );
    let gate = &verified["verdict"]["checks"]["generated-parity"];
    let raw = safe_artifact(owner, family::field(gate, "evidence")?)?;
    let mut profile_path = raw.parent().unwrap().join("coverage.toml");
    let mut report_path = raw.clone();
    if !profile_path.exists() || origins::document(&raw)?.get("coverage_sha256").is_none() {
        let binary = origins::artifact(owner, "generated-binary", manifest)?;
        run.guard(&binary)?;
        let output = run.path.join("diagnostics/baseline-profile-refresh");
        let root = run.root.clone();
        let stage = run.stage(
            "baseline-profile-refresh",
            &[
                binary.to_string_lossy().into_owned(),
                root.join("movies/super-metroid-any-3653.lsmv").to_string_lossy().into_owned(),
                "--generated".into(),
                "--frames".into(),
                "18000".into(),
                "--rom".into(),
                root.join("rom/sm.sfc").to_string_lossy().into_owned(),
                "--out".into(),
                output.to_string_lossy().into_owned(),
            ],
            &root,
            &BTreeMap::new(),
            Duration::from_secs(timeout),
            &BTreeMap::new(),
        )?;
        ensure!(
            stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "accepted profile refresh failed"
        );
        profile_path = output.join("coverage.toml");
        report_path = output.join("parity.toml");
        run.manifest.selection_profile_origin = Some("fresh replay of preserved accepted executable".into());
    } else {
        run.manifest.selection_profile_origin = Some("retained accepted cold replay".into());
    }
    run.save()?;
    let profile = bind_profile(run, owner, manifest, &report_path, &profile_path)?;
    let inventory = origins::artifact(owner, "generated-key-inventory", manifest)?;
    let typed_inventory: KeyInventory = pipeline_model::read(&inventory)?;
    typed_inventory.validate()?;
    let native_inventory = run.path.join("generated/selection-baseline-inventory.toml");
    pipeline_model::write(&native_inventory, &typed_inventory)?;
    run.record_artifact(
        "selection-original-inventory",
        &inventory,
        Some("generated/selection-original-inventory.toml"),
    )?;
    run.record_artifact("selection-baseline-inventory", &native_inventory, None)?;
    let mut discoveries = raw.parent().unwrap().join("misses.csv");
    if !discoveries.exists() {
        discoveries = report_path.parent().unwrap().join("misses.csv");
    }
    ensure!(discoveries.is_file(), "baseline call-target discovery missing");
    run.record_artifact("selection-discovery", &discoveries, Some("generated/selection-discovery.csv"))?;
    let root = run.root.clone();
    let target = run.path.join("build/selection");
    let env = BTreeMap::from([
        ("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned()),
        ("CARGO_INCREMENTAL".into(), "0".into()),
    ]);
    let stage = run.stage(
        "selection-generator-build",
        &["cargo", "build", "--release", "-p", "recomp", "--locked", "--offline", "--jobs", "1"].map(str::to_owned),
        &root,
        &env,
        Duration::from_secs(timeout),
        &BTreeMap::new(),
    )?;
    ensure!(
        stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "selection generator build failed"
    );
    run.record_artifact("selection-generator", &target.join("release/recomp"), Some("generated/selection-generator"))?;
    let output = run.path.join("checks/selection.toml");
    let command = vec![
        run.path.join("generated/selection-generator").to_string_lossy().into_owned(),
        "--rom".into(),
        root.join("rom/sm.sfc").to_string_lossy().into_owned(),
        "--functions".into(),
        root.join("config/functions.toml").to_string_lossy().into_owned(),
        "--selection-profile".into(),
        profile.to_string_lossy().into_owned(),
        "--selection-inventory".into(),
        run.path.join("generated/selection-baseline-inventory.toml").to_string_lossy().into_owned(),
        "--selection-entries".into(),
        run.path.join("generated/selection-discovery.csv").to_string_lossy().into_owned(),
        "--selection-out".into(),
        output.to_string_lossy().into_owned(),
    ];
    let stage =
        run.stage("selection-ranking", &command, &root, &env, Duration::from_secs(timeout), &BTreeMap::new())?;
    ensure!(
        stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "measured selection ranking failed"
    );
    let ranking: SelectionReport = pipeline_model::read(&output)?;
    ranking.validate()?;
    let report = BoundSelection {
        schema_version: 1,
        baseline: baseline(owner, manifest)?,
        source_sha256: family::field(&run.manifest.identities["source"], "sha256")?.into(),
        profile_sha256: digest(&profile)?,
        baseline_inventory_sha256: digest(&native_inventory)?,
        original_inventory_sha256: digest(&inventory)?,
        discovery_sha256: digest(&discoveries)?,
        identities: toml_value_decode::<InputDomain>(run.manifest.identities["inputs"].clone())?,
        ranking,
    };
    let path = run.path.join("selection.toml");
    pipeline_model::write(&path, &report)?;
    run.record_artifact("selection-result", &path, None)?;
    run . record_check ("measured-selection" , crate :: report ! ({ "status" : "pass" , "complete" : true , "outcome" : report . ranking . outcome , "evidence" : "selection.toml" , "sha256" : digest (& path) ? , })) ? ;
    run.check_sources()?;
    Ok(report)
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mut run = Run::create(&root, "iteration-selection", contract::domain(), None)?;
    eprintln!("Run: {}", run.path.display());
    let result = (|| -> Result<BoundSelection> {
        let owner = if let Some(path) = request.get("predecessor").and_then(Value::as_str) {
            PathBuf::from(path)
        } else {
            let pointer = origins::document(&root.join("out/accepted.toml"))?;
            PathBuf::from(family::field(&pointer, "run_path")?)
        };
        select(
            &mut run,
            &owner,
            request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400),
        )
    })();
    match result {
        Ok(report) => run.finalize(
            "pass",
            Some("measured-selection"),
            Some(match report.ranking.outcome {
                pipeline_model::SelectionOutcome::SupportedRoutine => "supported-routine",
                pipeline_model::SelectionOutcome::NeedsTimedPattern => "needs-timed-pattern",
                pipeline_model::SelectionOutcome::NoCoveredCandidate => "no-covered-candidate",
            }),
            &["selection-baseline".into(), "selection-profile".into(), "measured-selection".into()],
        ),
        Err(error) => run.finalize(
            if run.was_interrupted() { "incomplete" } else { "error" },
            Some("measured-selection"),
            Some(&format!("{error:#}")),
            &[],
        ),
    }
}
fn baseline(owner: &Path, manifest: &Value) -> Result<AcceptedBaseline> {
    Ok(AcceptedBaseline {
        run_id: family::field(manifest, "run_id")?.into(),
        run_path: fs::canonicalize(owner)?,
        source_sha256: family::field(&manifest["identities"]["source"], "sha256")?.into(),
    })
}
