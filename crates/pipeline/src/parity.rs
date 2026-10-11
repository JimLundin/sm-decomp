use crate::readiness_metadata::toml_from_slice;
use crate::{coverage, digest, family, preflight, readiness_run::Run};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use toml::Value;
pub(crate) struct Candidate {
    pub directory: PathBuf,
    pub target: PathBuf,
    pub game: PathBuf,
    pub steps: PathBuf,
    pub inventory: Option<PathBuf>,
}
impl Candidate {
    pub fn env(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("CARGO_TARGET_DIR".into(), self.target.to_string_lossy().into_owned()),
            ("SM_DECOMP_GENERATED_GAME".into(), self.game.to_string_lossy().into_owned()),
            ("SM_DECOMP_GENERATED_STEPS".into(), self.steps.to_string_lossy().into_owned()),
        ])
    }
}
pub(crate) fn prepare(run: &mut Run, mode: &str, timeout: Duration) -> Result<Candidate> {
    ensure!(["generated", "fallback-only"].contains(&mode), "unknown candidate mode");
    let directory = run.path.join("generated").join(mode);
    let target = run.path.join("build").join(mode);
    let game = directory.join("game.rs");
    let steps = directory.join("steps.rs");
    let inventory = if mode == "generated" { Some(run.path.join("generated/key-inventory.toml")) } else { None };
    let candidate = Candidate {
        directory: directory.clone(),
        target: target.clone(),
        game: game.clone(),
        steps: steps.clone(),
        inventory,
    };
    if directory.exists() {
        run.check_sources()?;
        for (name, path) in
            [("generated-game", &game), ("generated-steps", &steps), ("binary", &directory.join("diff"))]
        {
            ensure!(
                digest(path)? == family::field(&run.manifest.identities[&format!("{mode}-{name}")], "sha256")?,
                "prepared candidate artifact lacks matching provenance"
            );
        }
        return Ok(candidate);
    }
    fs::create_dir(&directory)?;
    let root = run.root.clone();
    let env = candidate.env();
    if mode == "generated" {
        let process = run.stage(
            "generated-generator-build",
            &["cargo", "build", "--release", "-p", "recomp", "--locked", "--offline", "--jobs", "1"].map(str::to_owned),
            &root,
            &BTreeMap::from([("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned())]),
            timeout,
            &BTreeMap::new(),
        )?;
        ensure!(
            process["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "candidate generator build failed"
        );
        let generator = directory.join("recomp");
        run.record_artifact(
            "generated-generator-binary",
            &target.join("release/recomp"),
            Some("generated/generated/recomp"),
        )?;
        run.guard(&generator)?;
        let command = vec![
            generator.to_string_lossy().into_owned(),
            "--rom".into(),
            root.join("rom/sm.sfc").to_string_lossy().into_owned(),
            "--functions".into(),
            root.join("config/functions.toml").to_string_lossy().into_owned(),
            "--out".into(),
            game.to_string_lossy().into_owned(),
            "--steps-out".into(),
            steps.to_string_lossy().into_owned(),
        ];
        let process = run.stage("generated-generation", &command, &root, &env, timeout, &BTreeMap::new())?;
        ensure!(
            process["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "candidate generation failed"
        );
    } else {
        fs::write(&game, "pub static TABLE: &[runtime::TableEntry] = &[];\n")?;
        fs::write(
            &steps,
            "pub static STEP_TABLE: &[runtime::StepEntry] = &[];\npub const ROM_SHA256: Option<&str> = None;\n",
        )?;
    }
    for (name, path) in [("game", &game), ("steps", &steps)] {
        let identity = format!("{mode}-generated-{name}");
        run.record_artifact(&identity, path, None)?;
        run.manifest.identities.get_mut(&identity).context("identity missing")?["evidence"] =
            path.strip_prefix(&run.path)?.to_string_lossy().into_owned().into();
        run.guard(path)?;
    }
    run . manifest . parity_build_inputs . insert (mode . into () , crate :: report ! ({ "game" : game . strip_prefix (& run . path) ? . to_str () , "steps" : steps . strip_prefix (& run . path) ? . to_str () , "target_directory" : target , "environment" : env , }) . as_table () . unwrap () . clone ()) ;
    run.save()?;
    if mode == "generated" {
        coverage::generated_inventory(run, &steps)?;
    }
    let process = run.stage(
        &format!("{mode}-build"),
        &["cargo", "build", "--release", "-p", "diff", "--bin", "diff", "--locked", "--offline", "--jobs", "1"]
            .map(str::to_owned),
        &root,
        &env,
        timeout,
        &BTreeMap::new(),
    )?;
    ensure!(
        process["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
        "candidate parity build failed"
    );
    run.record_artifact(
        &format!("{mode}-binary"),
        &target.join("release/diff"),
        Some(&format!("generated/{mode}/diff")),
    )?;
    run.guard(&directory.join("diff"))?;
    Ok(candidate)
}
pub(crate) fn replay(
    run: &mut Run,
    mode: &str,
    frames: u64,
    timeout: Duration,
    previous: Option<&Path>,
) -> Result<Value> {
    ensure!(
        frames > 0 && ["generated", "fallback-only"].contains(&mode),
        "parity requires valid mode and positive frame count"
    );
    let start = Instant::now();
    let inputs = run.manifest.identities["inputs"].clone();
    ensure!(
        inputs["frames"].as_integer().and_then(|n| u64::try_from(n).ok()).context("verified input frames missing")?
            >= frames,
        "verified movie shorter than requested scope"
    );
    let candidate = prepare(run, mode, timeout)?;
    let root = run.root.clone();
    let binary = candidate.directory.join("diff");
    let output = run.path.join("diagnostics").join(format!("{mode}-frame-parity"));
    let command = vec![
        binary.to_string_lossy().into_owned(),
        root.join("movies/super-metroid-any-3653.lsmv").to_string_lossy().into_owned(),
        format!("--{mode}"),
        "--frames".into(),
        frames.to_string(),
        "--rom".into(),
        root.join("rom/sm.sfc").to_string_lossy().into_owned(),
        "--out".into(),
        output.to_string_lossy().into_owned(),
    ];
    let process = run.stage(
        &format!("{mode}-replay"),
        &command,
        &root,
        &candidate.env(),
        timeout,
        &BTreeMap::from([(1, "divergence".into()), (2, "error".into())]),
    )?;
    let path = output.join("parity.toml");
    let mut report = if path.is_file() {
        run.record_artifact(&format!("{mode}-parity-report"), &path, None)?;
        toml_from_slice::<Value>(&fs::read(&path)?)?
    } else {
        crate :: report ! ({ "status" : if process ["status"] == crate :: readiness_metadata :: value (& "pass") ? { "incomplete" } else { process ["status"] . as_str () . unwrap_or ("error") } , "complete" : false , "expected_frames" : frames , "completed_frames" : 0 , "reason" : "replay produced no structured completion evidence" , })
    };
    if path.is_file() {
        let canonical = fs::read_to_string(run.path.join(family::field(&inputs, "input_evidence")?))?;
        let selected = canonical.lines().take(frames as usize).map(|line| format!("{line}\n")).collect::<String>();
        let mut expected = crate :: report ! ({ "rom_sha256" : inputs ["rom_sha256"] , "movie_sha256" : inputs ["movie_sha256"] , "input_sha256" : inputs ["input_sha256"] , "binary_sha256" : digest (& binary) ? , "selected_input_sha256" : format ! ("{:x}" , Sha256 :: digest (selected . as_bytes ())) , });
        if mode == "generated" {
            {
                let value: toml::Value = inputs["rom_sha256"].clone();
                expected
                    .as_table_mut()
                    .context("native metadata table missing")?
                    .insert(("compiled_rom_sha256").into(), value);
            };
        }
        let valid = report["schema_version"]
            == crate::readiness_metadata::value(&2).expect("validated native metadata")
            && report["mode"] == crate::readiness_metadata::value(&mode).expect("validated native metadata")
            && report["expected_frames"]
                == crate::readiness_metadata::value(&frames).expect("validated native metadata")
            && report["available_frames"] == inputs["frames"]
            && report["seed"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
            && report["controller"] == crate::readiness_metadata::value(&"P1").expect("validated native metadata")
            && report["profile"]
                == crate::readiness_metadata::value(&"lsnes-profile").expect("validated native metadata")
            && report["regions"] == crate::report!(vec!["CPU+clocks", "WRAM", "framebuffer"])
            && expected.as_table().unwrap().iter().all(|(name, value)| {
                report["identities"][name]
                    == crate::readiness_metadata::value(value).expect("validated native metadata")
            });
        if !valid {
            {
                let value: toml::Value = "error".into();
                report.as_table_mut().context("native metadata table missing")?.insert(("status").into(), value);
            };
            {
                let value: toml::Value = false.into();
                report.as_table_mut().context("native metadata table missing")?.insert(("complete").into(), value);
            };
            {
                let value: toml::Value = "replay evidence identity/scope mismatch".into();
                report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
            };
        } else if !process["complete"].as_bool().unwrap_or(false) {
            {
                let value: toml::Value = process["status"].clone();
                report.as_table_mut().context("native metadata table missing")?.insert(("status").into(), value);
            };
            {
                let value: toml::Value = false.into();
                report.as_table_mut().context("native metadata table missing")?.insert(("complete").into(), value);
            };
            {
                let value: toml::Value = process["reason"].clone();
                report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
            };
        } else if report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata") {
            let complete = process["returncode"]
                == crate::readiness_metadata::value(&0).expect("validated native metadata")
                && report["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
                && report["completed_frames"]
                    == crate::readiness_metadata::value(&frames).expect("validated native metadata")
                && report["matched_frames"]
                    == crate::readiness_metadata::value(&frames).expect("validated native metadata")
                && if mode == "generated" {
                    report["compiled_table_entries"]
                        .as_integer()
                        .and_then(|n| u64::try_from(n).ok())
                        .is_some_and(|n| n > 0)
                        && report["generated_steps"]
                            .as_integer()
                            .and_then(|n| u64::try_from(n).ok())
                            .is_some_and(|n| n > 0)
                } else {
                    report["generated_steps"]
                        == crate::readiness_metadata::value(&0).expect("validated native metadata")
                };
            if !complete {
                {
                    let value: toml::Value = "incomplete".into();
                    report.as_table_mut().context("native metadata table missing")?.insert(("status").into(), value);
                };
                {
                    let value: toml::Value = false.into();
                    report.as_table_mut().context("native metadata table missing")?.insert(("complete").into(), value);
                };
                {
                    let value: toml::Value = "successful replay did not complete mandatory scope".into();
                    report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
                };
            }
        } else {
            ensure!(
                [Some("divergence"), Some("uncovered"), Some("error"), Some("incomplete")]
                    .contains(&report["status"].as_str()),
                "unknown replay terminal status"
            );
        }
        report
            .as_table_mut()
            .context("report table missing")?
            .insert("evidence".into(), Value::String(path.strip_prefix(&run.path)?.to_string_lossy().into_owned()));
    }
    {
        let value: toml::Value = Value::Float(start.elapsed().as_secs_f64());
        report.as_table_mut().context("native metadata table missing")?.insert(("total_seconds").into(), value);
    };
    {
        let value: toml::Value = 1200.into();
        report
            .as_table_mut()
            .context("native metadata table missing")?
            .insert(("feedback_target_seconds").into(), value);
    };
    {
        let value: toml::Value = Value::Boolean(process["elapsed_seconds"].as_float().unwrap_or(0.0) > 1200.0);
        report
            .as_table_mut()
            .context("native metadata table missing")?
            .insert(("feedback_target_overrun").into(), value);
    };
    run.check_sources()?;
    if mode == "generated"
        && report["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
        && [Some("pass"), Some("divergence"), Some("uncovered")].contains(&report["status"].as_str())
    {
        let profile = output.join("coverage.toml");
        ensure!(
            digest(&profile)? == family::field(&report, "coverage_sha256")?,
            "instruction profile digest differs from replay report"
        );
        run.record_artifact("generated-instruction-profile", &profile, None)?;
        run.manifest.identities.get_mut("generated-instruction-profile").context("identity missing")?["evidence"] =
            profile.strip_prefix(&run.path)?.to_string_lossy().into_owned().into();
        run.save()?;
        let mut profile = crate::readiness_origins::document(&profile)?;
        for (field, value) in [
            ("complete", Value::Boolean(true)),
            ("expected_frames", Value::Integer(i64::try_from(frames)?)),
            ("completed_frames", report["completed_frames"].clone()),
            ("candidate_inventory_sha256", run.manifest.identities["generated-key-inventory"]["sha256"].clone()),
        ] {
            {
                let value: toml::Value = value;
                profile.as_table_mut().context("native metadata table missing")?.insert((field).into(), value);
            };
        }
        let bound = run.path.join("checks/generated-profile-bound.toml");
        pipeline_model::write(&bound, &profile)?;
        let gate = coverage::evaluate(run, &bound, candidate.inventory.as_ref().unwrap(), previous)?;
        if report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && gate["status"] != crate::readiness_metadata::value(&"pass").expect("validated native metadata")
        {
            {
                let value: toml::Value = gate["status"].clone();
                report.as_table_mut().context("native metadata table missing")?.insert(("status").into(), value);
            };
            {
                let value: toml::Value = gate["reason"].clone();
                report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
            };
        }
    } else if mode == "generated" {
        let gate = crate :: report ! ({ "status" : "incomplete" , "complete" : false , "reason" : "replay incomplete; instruction coverage is partial" , });
        run.record_check("changed-entry-coverage", gate)?;
    }
    run.record_check(&format!("{mode}-parity"), report.clone())?;
    Ok(report)
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mode = request.get("mode").and_then(Value::as_str).unwrap_or("generated");
    let frames = request.get("frames").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(18000);
    ensure!(frames > 0, "frames must be positive");
    let previous = request.get("predecessor").and_then(Value::as_str).map(Path::new);
    let parent = previous.and_then(|p| p.file_name()).and_then(|s| s.to_str());
    let mut run = Run::create(
        &root,
        "parity",
        crate :: report ! ({ "domain" : "Japan/USA any%" , "frames" : frames , "mode" : mode , "seed" : 0 , "controller" : "P1" , }),
        parent,
    )?;
    eprintln!("Run: {}", run.path.display());
    let outcome = (|| -> Result<Value> {
        let failures = preflight::verify(&mut run)?;
        ensure!(failures.is_empty(), "preflight failed: {}", failures.join("; "));
        replay(
            &mut run,
            mode,
            frames,
            Duration::from_secs(
                request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400),
            ),
            previous,
        )
    })();
    match outcome {
        Ok(report) => run.finalize(
            family::field(&report, "status")?,
            Some(&format!("{mode}-parity")),
            report
                .get("reason")
                .and_then(Value::as_str)
                .or(Some("cold parity scope complete; other readiness gates remain pending")),
            &["preflight".into(), format!("{mode}-parity")],
        ),
        Err(error) => run.finalize(
            if run.was_interrupted() { "incomplete" } else { "error" },
            Some("parity"),
            Some(&format!("{error:#}")),
            &[],
        ),
    }
}
