use crate::readiness_metadata::toml_from_slice;
use crate::{atomic_toml, digest, family, parity, preflight, readiness_run::Run};
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use toml::Value;
pub(crate) fn focused(run: &mut Run, mode: &str, namespace: &str, timeout: Duration) -> Result<PathBuf> {
    ensure!(namespace.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-'), "unsafe diagnostic namespace");
    let preparation = parity::prepare(run, mode, timeout)?;
    let binary = preparation.target.join("release/focused-replay");
    let root = run.root.clone();
    if !run.manifest.identities.contains_key("focused-prepared-binary") {
        let process = run.stage(
            &format!("{namespace}focused-build"),
            &[
                "cargo",
                "build",
                "--release",
                "-p",
                "diff",
                "--bin",
                "focused-replay",
                "--locked",
                "--offline",
                "--jobs",
                "1",
            ]
            .map(str::to_owned),
            &root,
            &preparation.env(),
            timeout,
            &Default::default(),
        )?;
        ensure!(
            process["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "focused binary build failed"
        );
        run.record_artifact("focused-prepared-binary", &binary, Some("generated/focused-prepared"))?;
    } else {
        ensure!(
            digest(&binary)? == family::field(&run.manifest.identities["focused-prepared-binary"], "sha256")?,
            "focused binary changed after preparation"
        );
    }
    let relative = format!("generated/{namespace}focused-replay");
    let candidate = run.path.join(&relative);
    run.record_artifact(&format!("{namespace}focused-binary"), &binary, Some(&relative))?;
    run.guard(&candidate)?;
    Ok(candidate)
}
pub(crate) fn diagnose(run: &mut Run, request: &Value, namespace: &str) -> Result<Value> {
    let began = Instant::now();
    let mode = request.get("mode").and_then(Value::as_str).unwrap_or("generated");
    let timeout = Duration::from_secs(
        request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400),
    );
    let start = request.get("start_input").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(0);
    let skip =
        request.get("start_instruction").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(0);
    let count =
        request.get("instructions").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(100);
    ensure!(count > 0, "focused instruction count must be positive");
    let frame_end = request.get("until_frame_end").and_then(toml::Value::as_bool) == Some(true);
    let cache = request.get("cache").and_then(Value::as_str).map(Path::new);
    ensure!(
        !frame_end || (cache.is_none() && skip == 0),
        "failed-frame localization requires cold boundary without skipped instructions"
    );
    let mut disabled = request.get("disabled_keys").and_then(Value::as_array).cloned().unwrap_or_default();
    for key in &disabled {
        ensure!(
            key["address"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n <= 0xffffff)
                && key["mx"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n < 4),
            "invalid disabled instruction key"
        );
    }
    disabled.sort_by_key(|key| {
        (
            key["address"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap(),
            key["mx"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap(),
        )
    });
    disabled.dedup();
    let candidate = focused(run, mode, namespace, timeout)?;
    let root = run.root.clone();
    let output = run.path.join("diagnostics").join(format!("{namespace}instructions"));
    let mut command = vec![
        candidate.to_string_lossy().into_owned(),
        root.join("movies/super-metroid-any-3653.lsmv").to_string_lossy().into_owned(),
        "--out".into(),
        output.to_string_lossy().into_owned(),
        "--start-input".into(),
        start.to_string(),
        "--start-instruction".into(),
        skip.to_string(),
        "--instructions".into(),
        count.to_string(),
        "--rom".into(),
        root.join("rom/sm.sfc").to_string_lossy().into_owned(),
    ];
    if frame_end {
        command.push("--until-frame-end".into());
    }
    if mode == "fallback-only" {
        command.push("--fallback-only".into());
    }
    if !disabled.is_empty() {
        let path = run.path.join("generated").join(format!("{namespace}disabled-keys.toml"));
        atomic_toml(&path, &crate::report!({ "keys": disabled }))?;
        run.guard(&path)?;
        command.extend(["--disabled-keys".into(), path.to_string_lossy().into_owned()]);
    }
    if let Some(cache) = cache {
        let inputs = &run.manifest.identities["inputs"];
        let ids = &run.manifest.identities;
        let identity = pipeline_model::CheckpointIdentities {
            rom_sha256: family::field(inputs, "rom_sha256")?.into(),
            movie_sha256: family::field(inputs, "movie_sha256")?.into(),
            input_sha256: family::field(inputs, "input_sha256")?.into(),
            source_sha256: family::field(&ids["source"], "sha256")?.into(),
            hardware_sha256: family::field(&ids["hardware"], "source_sha256")?.into(),
            profile: "lsnes-profile".into(),
            serializer: "paired-v2".into(),
            compiler: family::field(&ids["compiler"], "version")?.into(),
            features: crate::pattern_contract::strings(&ids["hardware"]["features"])?,
            settings: pipeline_model::CheckpointSettings {
                seed: 0,
                mode: Some(mode.into()),
                candidate_sha256: Some(digest(&candidate)?),
                disabled_keys: disabled
                    .iter()
                    .map(|row| {
                        (
                            row["address"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap() as u32,
                            row["mx"].as_integer().and_then(|n| u64::try_from(n).ok()).unwrap() as u8,
                        )
                    })
                    .collect(),
            },
            dispatch_sha256: None,
        };
        let path = run.path.join("generated").join(format!("{namespace}checkpoint-identities.toml"));
        pipeline_model::write(&path, &identity)?;
        run.guard(&path)?;
        command.extend([
            "--cache".into(),
            cache.to_string_lossy().into_owned(),
            "--identities".into(),
            path.to_string_lossy().into_owned(),
        ]);
    }
    let preparation = parity::prepare(run, mode, timeout)?;
    let process = run.stage(
        &format!("{namespace}focused-replay"),
        &command,
        &root,
        &preparation.env(),
        timeout,
        &std::collections::BTreeMap::from([(1, "divergence".into()), (2, "error".into())]),
    )?;
    let path = output.join("instruction.toml");
    let mut report = if path.is_file() {
        toml_from_slice::<Value>(&fs::read(&path)?)?
    } else {
        crate :: report ! ({ "status" : if process ["status"] == crate :: readiness_metadata :: value (& "pass") ? { "incomplete" } else { process ["status"] . as_str () . unwrap_or ("error") } , "complete" : false , "reason" : "no focused completion report" , })
    };
    if path.is_file() {
        let inputs = &run.manifest.identities["inputs"];
        let valid = report["schema_version"]
            == crate::readiness_metadata::value(&2).expect("validated native metadata")
            && if frame_end {
                report["scope"]
                    == crate::readiness_metadata::value(&"until-frame-end").expect("validated native metadata")
                    && report.get("expected_instructions").is_none()
                    && report["instruction_budget"]
                        == crate::readiness_metadata::value(&2000000).expect("validated native metadata")
            } else {
                report["expected_instructions"]
                    == crate::readiness_metadata::value(&count).expect("validated native metadata")
            };
        let valid = valid
            && report["start_input_index"]
                == crate::readiness_metadata::value(&start).expect("validated native metadata")
            && report["start_instruction"]
                == crate::readiness_metadata::value(&skip).expect("validated native metadata")
            && report["replay"]
                == crate::readiness_metadata::value(&if cache.is_some() { "warm" } else { "cold" })
                    .expect("validated native metadata")
            && report["binary_sha256"]
                == crate::readiness_metadata::value(&digest(&candidate)?).expect("validated native metadata")
            && report["disabled_keys"]
                == crate::readiness_metadata::value(&disabled).expect("validated native metadata")
            && ["rom_sha256", "movie_sha256", "input_sha256"]
                .iter()
                .all(|key| report["identities"][key] == inputs[key]);
        let valid = valid
            && (cache.is_none()
                || (report["cache"]["status"]
                    == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                    && report["cache"]["complete"]
                        == crate::readiness_metadata::value(&true).expect("validated native metadata")
                    && report["cache"]["validation"]["completed_frames"]
                        == crate::readiness_metadata::value(&1).expect("validated native metadata")
                    && report["cache"]["validation"]["observed_rollovers"]
                        == crate::readiness_metadata::value(&2).expect("validated native metadata")
                    && report["cache"]["validation"]["complete"]
                        == crate::readiness_metadata::value(&true).expect("validated native metadata")));
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
                let value: toml::Value = "focused identity/scope mismatch".into();
                report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
            };
        } else if ![Some("pass"), Some("divergence")].contains(&process["status"].as_str()) {
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
        } else if report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && (process["returncode"] != crate::readiness_metadata::value(&0).expect("validated native metadata")
                || report["complete"] != crate::readiness_metadata::value(&true).expect("validated native metadata")
                || if frame_end {
                    report["observed_rollovers"]
                        != crate::readiness_metadata::value(&1).expect("validated native metadata")
                } else {
                    report["completed_instructions"]
                        != crate::readiness_metadata::value(&count).expect("validated native metadata")
                })
        {
            {
                let value: toml::Value = "incomplete".into();
                report.as_table_mut().context("native metadata table missing")?.insert(("status").into(), value);
            };
            {
                let value: toml::Value = false.into();
                report.as_table_mut().context("native metadata table missing")?.insert(("complete").into(), value);
            };
            {
                let value: toml::Value = "focused scope did not complete".into();
                report.as_table_mut().context("native metadata table missing")?.insert(("reason").into(), value);
            };
        }
        report
            .as_table_mut()
            .context("report table missing")?
            .insert("evidence".into(), Value::String(path.strip_prefix(&run.path)?.to_string_lossy().into_owned()));
        run.record_artifact(&format!("{namespace}focused-report"), &path, None)?;
    }
    if let Some(cache) = cache
        && report.get("cache").and_then(|cache| cache.get("status")).and_then(Value::as_str) == Some("pass")
    {
        for name in ["pair.bin", "validation.toml", "cache.toml"] {
            run.record_artifact(
                &format!("{namespace}checkpoint-{name}"),
                &cache.join(name),
                Some(&format!("diagnostics/{namespace}cache/{name}")),
            )?;
        }
    }
    {
        let value: toml::Value = Value::Array(disabled);
        report.as_table_mut().context("native metadata table missing")?.insert(("disabled_keys").into(), value);
    };
    {
        let value: toml::Value = Value::Float(began.elapsed().as_secs_f64());
        report.as_table_mut().context("native metadata table missing")?.insert(("total_seconds").into(), value);
    };
    {
        let value: toml::Value = 60.into();
        report
            .as_table_mut()
            .context("native metadata table missing")?
            .insert(("feedback_target_seconds").into(), value);
    };
    {
        let value: toml::Value = Value::Boolean(began.elapsed() > Duration::from_secs(60));
        report
            .as_table_mut()
            .context("native metadata table missing")?
            .insert(("feedback_target_overrun").into(), value);
    };
    {
        let value: toml::Value =
            Value::Boolean(report.get("warm_replay_seconds").and_then(Value::as_float).unwrap_or(0.0) > 60.0);
        report.as_table_mut().context("native metadata table missing")?.insert(("warm_target_overrun").into(), value);
    };
    run.check_sources()?;
    run.record_check(&format!("{namespace}instruction-diagnosis"), report.clone())?;
    Ok(report)
}
pub(crate) fn discover(run: &mut Run, keys: &Value, frames: u64, timeout: Duration) -> Result<Value> {
    let candidate = focused(run, "generated", "discovery-", timeout)?;
    let preparation = parity::prepare(run, "generated", timeout)?;
    let keys_path = run.path.join("generated/discovery-keys.toml");
    atomic_toml(&keys_path, &crate::report!({ "keys": keys }))?;
    run.guard(&keys_path)?;
    let root = run.root.clone();
    let output = run.path.join("diagnostics/discovery");
    let command = vec![
        candidate.to_string_lossy().into_owned(),
        root.join("movies/super-metroid-any-3653.lsmv").to_string_lossy().into_owned(),
        "--discover-keys".into(),
        keys_path.to_string_lossy().into_owned(),
        "--frames".into(),
        frames.to_string(),
        "--rom".into(),
        root.join("rom/sm.sfc").to_string_lossy().into_owned(),
        "--out".into(),
        output.to_string_lossy().into_owned(),
    ];
    let process = run.stage(
        "instruction-discovery-process",
        &command,
        &root,
        &preparation.env(),
        timeout,
        &std::collections::BTreeMap::from([(1, "uncovered".into()), (2, "error".into())]),
    )?;
    let path = output.join("discovery.toml");
    ensure!(path.is_file(), "discovery completion report missing");
    let mut report: Value = toml_from_slice(&fs::read(&path)?)?;
    run.record_artifact("instruction-discovery-report", &path, None)?;
    ensure!(
        report["schema_version"] == crate::readiness_metadata::value(&2).expect("validated native metadata")
            && report["expected_frames"]
                == crate::readiness_metadata::value(&frames).expect("validated native metadata")
            && report["keys"] == crate::readiness_metadata::value(keys).expect("validated native metadata")
            && report["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && report["binary_sha256"]
                == crate::readiness_metadata::value(&digest(&candidate)?).expect("validated native metadata")
            && ["rom_sha256", "movie_sha256", "input_sha256"]
                .iter()
                .all(|key| report["identities"][key] == run.manifest.identities["inputs"][key]),
        "discovery identity/scope mismatch"
    );
    if ![Some("pass"), Some("uncovered")].contains(&process["status"].as_str()) {
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
    }
    run.record_check("instruction-discovery", report.clone())?;
    Ok(report)
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mut run = Run::create(
        &root,
        "diagnostic",
        crate :: report ! ({ "frames" : request . get ("frames") . and_then (Value :: as_integer) . and_then (| n | u64 :: try_from (n) . ok ()) . unwrap_or (18000) , "start_input" : request . get ("start_input") . and_then (Value :: as_str) , "instructions" : request . get ("instructions") . and_then (Value :: as_str) , }),
        None,
    )?;
    eprintln!("Run: {}", run.path.display());
    let outcome = (|| -> Result<Value> {
        let failures = preflight::verify(&mut run)?;
        ensure!(failures.is_empty(), "preflight failed: {}", failures.join("; "));
        if request.get("keys").is_some() {
            discover(
                &mut run,
                &request["keys"],
                request.get("frames").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(18000),
                Duration::from_secs(
                    request
                        .get("timeout")
                        .and_then(Value::as_integer)
                        .and_then(|n| u64::try_from(n).ok())
                        .unwrap_or(2400),
                ),
            )
        } else {
            diagnose(&mut run, &request, "")
        }
    })();
    match outcome {
        Ok(report) => run.finalize(
            family::field(&report, "status")?,
            Some("instruction-diagnosis"),
            report
                .get("reason")
                .and_then(Value::as_str)
                .or(Some("focused scope complete; readiness requires other gates")),
            &[
                "preflight".into(),
                if request.get("keys").is_none() { "instruction-diagnosis" } else { "instruction-discovery" }.into(),
            ],
        ),
        Err(error) => run.finalize(
            if run.was_interrupted() { "incomplete" } else { "error" },
            Some("instruction-diagnosis"),
            Some(&format!("{error:#}")),
            &[],
        ),
    }
}
