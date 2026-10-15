use crate::atomic_toml;
use crate::coverage;
use crate::digest;
use crate::preflight;
use crate::readiness_contract as contract;
use crate::readiness_origins as origins;
use crate::readiness_run::Run;
use crate::readiness_run::safe_artifact;
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant},
};
use toml::Value;
const CASES: [(&str, &str); 9] = [
    ("correct-execution", "pass"),
    ("wrong-instruction", "divergence"),
    ("transient-effect", "divergence"),
    ("uncovered-key", "uncovered"),
    ("empty-compilation", "uncovered"),
    ("preflight-build-failure", "error"),
    ("interrupted-truncated", "incomplete"),
    ("incompatible-stale", "incomplete"),
    ("paired-cache", "pass"),
];
const LIFECYCLE: [&str; 3] = ["preflight-build-failure", "interrupted-truncated", "incompatible-stale"];
fn require(value: bool, message: &str) -> Result<()> {
    ensure!(value, "{message}");
    Ok(())
}
fn strings(values: &[&str]) -> Value {
    Value::Array(values.iter().map(|s| Value::String((*s).into())).collect())
}
fn capture_failure<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(action)).unwrap_or_else(|payload| {
        let reason = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied())
            .unwrap_or("unknown panic payload");
        Err(anyhow::anyhow!("feedback demonstrations panicked: {reason}"))
    })
}
fn assertions(case: &str, actual: &Value) -> Result<Value> {
    let expected = CASES.iter().find(|(name, _)| *name == case).context("unknown feedback case")?.1;
    require(
        actual["status"] == crate::readiness_metadata::value(&expected).expect("validated native metadata")
            && actual["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata"),
        "unexpected feedback outcome",
    )?;
    match case {
        "correct-execution" | "wrong-instruction" | "transient-effect" => {
            require(
                actual["frame_end_equal"]
                    == crate::readiness_metadata::value(&true).expect("validated native metadata"),
                "fixture frame-end parity missing",
            )?;
            if case == "correct-execution" {
                require(
                    actual["focus"]["completed_instructions"]
                        == crate::readiness_metadata::value(&8).expect("validated native metadata")
                        && !crate::documents::array(&actual["focus"], "executed_generated_keys")?.is_empty(),
                    "correct fixture did not execute generated instructions",
                )?;
                return Ok(strings(&["fresh generated execution", "complete instruction and frame parity"]));
            }
            let focus = &actual["focus"];
            let first = &focus["first_mismatch"];
            require(
                first["instruction"] == crate::readiness_metadata::value(&3).expect("validated native metadata")
                    && first["subject"]["generated_key"] == crate::report!(vec![0x8004, 3])
                    && actual["altered_key"] == crate::report!(vec![0x8004, 3])
                    && focus.get("preceding_matching_boundary").is_some()
                    && focus["first_bus_difference"]["reference"]["value"]
                        == crate::readiness_metadata::value(&0x12).expect("validated native metadata")
                    && focus["first_bus_difference"]["subject"]["value"]
                        == crate::readiness_metadata::value(&0x99).expect("validated native metadata"),
                "wrong store was not attributed at its earliest observable write",
            )?;
            Ok(strings(&[
                "first wrong write names altered generated key",
                "matching preceding boundary",
                "transient write detected despite matching frame-end CPU/clocks/WRAM/video",
            ]))
        }
        "uncovered-key" | "empty-compilation" => {
            let c = &actual["coverage"];
            require(
                c["status"] == crate::readiness_metadata::value(&"uncovered").expect("validated native metadata")
                    && actual["matched_frames"]
                        == crate::readiness_metadata::value(&2).expect("validated native metadata"),
                "actual coverage refusal missing",
            )?;
            if case == "uncovered-key" {
                require(
                    c["generated_instructions"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                        && c["uncovered_keys"]
                            == crate::report!(vec![crate :: report ! ({ "address" : 0x8100 , "mx" : 3 , })]),
                    "old execution masked new unexecuted entry",
                )?;
                Ok(strings(&["real old generated execution", "added unexecuted key refuses acceptance"]))
            } else {
                require(
                    c["generated_instructions"]
                        == crate::readiness_metadata::value(&0).expect("validated native metadata")
                        && actual["profile"]["compiled"] == crate::report!(vec![]),
                    "empty compilation did not refuse fallback execution",
                )?;
                Ok(strings(&["empty table refuses acceptance despite complete fallback parity"]))
            }
        }
        "paired-cache" => {
            require(
                actual["warm"]["status"]
                    == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                    && actual["cold"]["status"]
                        == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                    && actual["warm"]["history"] == actual["cold"]["history"],
                "warm/cold trace differs",
            )?;
            for (name, action) in
                [("created", "created"), ("reused", "reused"), ("corrupt", "rebuilt"), ("incompatible", "rebuilt")]
            {
                let c = &actual[name];
                require(
                    c["action"] == crate::readiness_metadata::value(&action).expect("validated native metadata")
                        && c["validation"]["completed_frames"]
                            == crate::readiness_metadata::value(&1).expect("validated native metadata")
                        && c["validation"]["observed_rollovers"]
                            == crate::readiness_metadata::value(&2).expect("validated native metadata"),
                    "incomplete cache control",
                )?;
                if action == "rebuilt" {
                    require(c["rejection"].as_str().is_some_and(|s| !s.is_empty()), "missing cache rejection reason")?;
                }
            }
            Ok(strings(&[
                "cold next complete frame validates both sides",
                "warm/cold traces and outcomes match",
                "corrupt and incompatible caches rejected and rebuilt",
            ]))
        }
        _ => {
            let expected: &[(&str, &str)] = match case {
                "preflight-build-failure" => &[("preflight", "error"), ("build", "error")],
                "interrupted-truncated" => &[
                    ("signal", "incomplete"),
                    ("timeout", "incomplete"),
                    ("truncated", "incomplete"),
                    ("unfinished", "incomplete"),
                ],
                _ => &[("stale", "incomplete"), ("identity", "error"), ("source", "error"), ("input", "error")],
            };
            let obs = actual["observations"].as_table().context("lifecycle observations missing")?;
            require(obs.len() == expected.len(), "lifecycle controls omitted")?;
            for (name, status) in expected {
                let o = &obs[*name];
                require(
                    o["status"] == crate::readiness_metadata::value(status).expect("validated native metadata")
                        && o["accept_returncode"]
                            == crate::readiness_metadata::value(&2).expect("validated native metadata")
                        && o["pointer_unchanged"]
                            == crate::readiness_metadata::value(&true).expect("validated native metadata"),
                    "failure control allowed publication",
                )?;
            }
            Ok(Value::try_from(
                expected
                    .iter()
                    .map(|(n, _)| format!("{n} refuses publication and preserves prior pointer"))
                    .collect::<Vec<_>>(),
            )?)
        }
    }
}
fn new(root: &Path) -> Result<Run> {
    let mut run = Run::create(root, "readiness", contract::domain(), None)?;
    run.capture_source()?;
    Ok(run)
}
fn terminal(owner: &Path) -> Result<Value> {
    if !owner.join("verdict.toml").exists() {
        return Ok(Value::String("incomplete".into()));
    }
    let manifest = origins::document(&owner.join("manifest.toml"))?;
    let verdict = origins::document(&owner.join("verdict.toml"))?;
    if verdict["run_id"] != manifest["run_id"] {
        return Ok(Value::String("incomplete".into()));
    }
    Ok(verdict["status"].clone())
}
fn reject(
    output: &Path,
    pointer: &Path,
    before: &[u8],
    name: &str,
    run: &Run,
    status: &str,
    observations: &mut Value,
) -> Result<()> {
    let binary = output.join("lifecycle-driver");
    let command = vec![
        binary.to_string_lossy().into_owned(),
        "readiness-accept".into(),
        "--run".into(),
        run.path.to_string_lossy().into_owned(),
    ];
    let result = Command::new(&binary).args(&command[1..]).output()?;
    atomic_toml(
        &output.join(format!("{name}-accept.toml")),
        &crate :: report ! ({ "command" : command , "returncode" : result . status . code () , "stdout" : String :: from_utf8_lossy (& result . stdout) , "stderr" : String :: from_utf8_lossy (& result . stderr) , }),
    )?;
    let actual = terminal(&run.path)?;
    ensure!(actual.as_str() == Some(status), "unexpected lifecycle terminal status: {name}");
    {
        let value: toml::Value = crate :: report ! ({ "status" : actual , "accept_returncode" : result . status . code () , "pointer_unchanged" : fs :: read (pointer) ? == before , "run" : run . path , });
        observations.as_table_mut().context("native metadata table missing")?.insert((name).into(), value);
    };
    Ok(())
}
pub fn lifecycle(case: &str, output: &Path) -> Result<Value> {
    ensure!(LIFECYCLE.contains(&case), "unknown lifecycle case");
    fs::create_dir(output)?;
    let running_image =
        if cfg!(target_os = "linux") { PathBuf::from("/proc/self/exe") } else { std::env::current_exe()? };
    fs::copy(&running_image, output.join("lifecycle-driver"))?;
    let root = output.join("repository");
    fs::create_dir(&root)?;
    ensure!(
        Command::new("git").args(["init", "-q"]).arg(&root).status()?.success(),
        "private Git initialization failed"
    );
    fs::create_dir(root.join("config"))?;
    fs::write(root.join("config/functions.toml"), "# isolated fixture\n")?;
    let pointer = root.join("out/accepted.toml");
    fs::create_dir(root.join("out"))?;
    fs::write(&pointer, b"fixture_predecessor = \"must stay unchanged\"\n")?;
    let before = fs::read(&pointer)?;
    let mut obs = crate::report!({});
    if case == "preflight-build-failure" {
        let mut run = Run::create(&root, "readiness", contract::domain(), None)?;
        let failures = preflight::verify(&mut run)?;
        ensure!(!failures.is_empty(), "missing setup passed preflight");
        run.finalize("error", Some("preflight"), None, &[])?;
        reject(output, &pointer, &before, "preflight", &run, "error", &mut obs)?;
        let mut run = new(&root)?;
        let process = run.stage(
            "build",
            &["sh".into(), "-c".into(), "echo deliberate build failure >&2; exit 7".into()],
            &root,
            &BTreeMap::new(),
            Duration::from_secs(5),
            &BTreeMap::new(),
        )?;
        ensure!(
            process["returncode"] == crate::readiness_metadata::value(&7).expect("validated native metadata")
                && !run.manifest.identities.contains_key("binary"),
            "build failure fabricated binary"
        );
        run.finalize("error", Some("build"), None, &[])?;
        reject(output, &pointer, &before, "build", &run, "error", &mut obs)?;
    } else if case == "interrupted-truncated" {
        for (name, script, timeout) in
            [("signal", "kill -TERM $$", Duration::from_secs(5)), ("timeout", "sleep 10", Duration::from_millis(50))]
        {
            let mut run = new(&root)?;
            let process = run.stage(
                name,
                &["sh".into(), "-c".into(), script.into()],
                &root,
                &BTreeMap::new(),
                timeout,
                &BTreeMap::new(),
            )?;
            ensure!(
                process["status"]
                    == crate::readiness_metadata::value(&"incomplete").expect("validated native metadata")
                    && process["complete"]
                        == crate::readiness_metadata::value(&false).expect("validated native metadata"),
                "interruption reported complete"
            );
            run.finalize("pass", None, None, &[name.into()])?;
            reject(output, &pointer, &before, name, &run, "incomplete", &mut obs)?;
        }
        let mut run = new(&root)?;
        let capture = run.path.join("generated/capture.csv");
        fs::write(&capture, "frame,event\n272,1\n")?;
        atomic_toml(
            &capture.with_extension("capture.toml"),
            &crate :: report ! ({ "schema_version" : 1 , "role" : "candidate" , "run_id" : run . manifest . run_id , "checkpoint_sha256" : digest (& capture) ? , "completed_frames" : 1 , }),
        )?;
        let failure = crate::readiness_sync::verify_capture(&capture, "candidate", &crate::report!({}), 18000)
            .err()
            .context("truncated capture accepted")?;
        ensure!(failure.to_string().contains("truncated"), "capture did not fail for truncated scope");
        run.record_check(
            "capture",
            crate :: report ! ({ "status" : "incomplete" , "complete" : false , "reason" : failure . to_string () , }),
        )?;
        run.finalize("pass", None, None, &["capture".into()])?;
        reject(output, &pointer, &before, "truncated", &run, "incomplete", &mut obs)?;
        let run = new(&root)?;
        reject(output, &pointer, &before, "unfinished", &run, "incomplete", &mut obs)?;
    } else {
        let mut old = new(&root)?;
        old.record_check("preflight", crate :: report ! ({ "status" : "pass" , "complete" : true , }))?;
        old.finalize("pass", None, None, &["preflight".into()])?;
        let run = new(&root)?;
        let stale = origins::document(&old.path.join("verdict.toml"))?;
        atomic_toml(&run.path.join("verdict.toml"), &stale)?;
        reject(output, &pointer, &before, "stale", &run, "incomplete", &mut obs)?;
        fs::create_dir(root.join("rom"))?;
        fs::write(root.join("rom/sm.sfc"), b"fixture ROM")?;
        fs::create_dir(root.join("movies"))?;
        let movie = root.join("movies/fixture.lsmv");
        let mut archive = zip::ZipWriter::new(fs::File::create(&movie)?);
        archive.start_file("rom.sha256", zip::write::SimpleFileOptions::default())?;
        use std::io::Write;
        archive.write_all("0".repeat(64).as_bytes())?;
        archive.finish()?;
        let mut run = new(&root)?;
        let failure = preflight :: movie_inputs (& mut run , & crate :: report ! ({ "file" : "fixture.lsmv" , "rom_sha256" : "0" . repeat (64) , "sha256" : digest (& movie) ? , "frames" : 1 , }) ,) . err () . context ("incompatible ROM accepted") ? ;
        run.finalize("error", Some("identity"), Some(&failure.to_string()), &[])?;
        reject(output, &pointer, &before, "identity", &run, "error", &mut obs)?;
        for name in ["source", "input"] {
            let mut run = new(&root)?;
            let path = if name == "source" { root.join("config/functions.toml") } else { root.join("rom/sm.sfc") };
            if name == "input" {
                run.guard(&path)?;
            }
            let mut bytes = fs::read(&path)?;
            bytes.extend_from_slice(b"changed");
            fs::write(&path, bytes)?;
            let failure = run.check_sources().err().context("contamination accepted")?;
            run.finalize("error", Some(name), Some(&failure.to_string()), &[])?;
            reject(output, &pointer, &before, name, &run, "error", &mut obs)?;
        }
    }
    ensure!(fs::read(pointer)? == before, "private predecessor replaced");
    let expected = CASES.iter().find(|(n, _)| *n == case).unwrap().1;
    let report =
        crate :: report ! ({ "schema_version" : 1 , "status" : expected , "complete" : true , "observations" : obs , });
    assertions(case, &report)?;
    atomic_toml(&output.join("report.toml"), &report)?;
    Ok(report)
}
fn fixture_inventory(root: &Path, inputs: &Value, source: &str) -> Result<(Run, PathBuf)> {
    let mut run = Run::create(root, "feedback-fixture", crate :: report ! ({ "frames" : 2 , }), None)?;
    run.capture_source()?;
    run.manifest.identities.insert("inputs".into(), inputs.clone());
    run.save()?;
    let steps = run.path.join("generated/steps.rs");
    fs::write(
        &steps,
        format!(
            "{source}\npub const ROM_SHA256: Option<&str> = Some(\"{}\");\n",
            crate::documents::field(inputs, "rom_sha256")?
        ),
    )?;
    let path = coverage::generated_inventory(&mut run, &steps)?;
    Ok((run, path))
}
fn coverage_control(case: &str, output: &Path) -> Result<()> {
    let mut actual = origins::document(&output.join("report.toml"))?;
    let root = output.join("coverage-repository");
    fs::create_dir(&root)?;
    ensure!(
        Command::new("git").args(["init", "-q"]).arg(&root).status()?.success(),
        "private Git initialization failed"
    );
    fs::create_dir(root.join("config"))?;
    fs::write(root.join("config/functions.toml"), "# synthetic emitter fixture\n")?;
    let inputs = actual["identities"].clone();
    let mut source = fs::read_to_string(output.join("steps.rs"))?;
    let previous = if case == "uncovered-key" {
        let previous =
            source.lines().filter(|line| !line.contains("(0x008100,")).map(|l| format!("{l}\n")).collect::<String>();
        let (mut run, _) = fixture_inventory(&root, &inputs, &previous)?;
        run.finalize("pass", None, None, &[])?;
        Some(run.path)
    } else {
        source = "pub static STEP_TABLE: &[runtime::StepEntry] = &[];\n".into();
        None
    };
    let (mut run, inventory) = fixture_inventory(&root, &inputs, &source)?;
    let mut profile = actual["profile"].clone();
    {
        let value: toml::Value = true.into();
        profile.as_table_mut().context("native metadata table missing")?.insert(("complete").into(), value);
    };
    {
        let value: toml::Value = 2.into();
        profile.as_table_mut().context("native metadata table missing")?.insert(("expected_frames").into(), value);
    };
    {
        let value: toml::Value = 2.into();
        profile.as_table_mut().context("native metadata table missing")?.insert(("completed_frames").into(), value);
    };
    {
        let value: toml::Value = digest(&inventory)?.into();
        profile
            .as_table_mut()
            .context("native metadata table missing")?
            .insert(("candidate_inventory_sha256").into(), value);
    };
    let path = run.path.join("checks/profile.toml");
    atomic_toml(&path, &profile)?;
    let coverage = coverage::evaluate(&mut run, &path, &inventory, previous.as_deref())?;
    actual.as_table_mut().context("feedback report must be a table")?.insert("coverage".into(), coverage);
    actual["status"] = actual["coverage"]["status"].clone();
    let status = crate::documents::field(&actual, "status")?;
    run.finalize(status, None, None, &[])?;
    assertions(case, &actual)?;
    atomic_toml(&output.join("coverage-control.toml"), &actual)
}
pub fn verify(owner: &Path, manifest: &Value, report: &Value) -> Result<()> {
    contract::validate("feedback-demonstrations", report)?;
    let source = crate::documents::field(&manifest["identities"]["source"], "sha256")?;
    let hardware = crate::documents::field(&manifest["identities"]["hardware"], "source_sha256")?;
    let binary = origins::artifact(owner, "feedback-binary", manifest)?;
    origins::process(
        owner,
        "feedback-build",
        manifest,
        Some(&crate::report!(vec![
            "cargo",
            "build",
            "--release",
            "-p",
            "snes",
            "--bin",
            "feedback-probe",
            "--locked",
            "--offline",
            "--jobs",
            "1"
        ])),
        &[0],
    )?;
    for (case, expected) in CASES {
        let detail = &report["cases"][case];
        let relative = crate::documents::field(&detail["evidence"], "path")?;
        let path = safe_artifact(owner, relative)?;
        let filename = if ["uncovered-key", "empty-compilation"].contains(&case) {
            "coverage-control.toml"
        } else {
            "report.toml"
        };
        ensure!(
            relative == format!("diagnostics/feedback/{case}/{filename}")
                && digest(&path)? == crate::documents::field(&detail["evidence"], "sha256")?,
            "feedback report identity differs"
        );
        let actual = origins::document(&path)?;
        ensure!(
            detail["assertions"]
                == crate::readiness_metadata::value(&assertions(case, &actual)?).expect("validated native metadata")
                && detail["expected_outcome"]
                    == crate::readiness_metadata::value(&expected).expect("validated native metadata")
                && detail["observed_outcome"] == actual["status"]
                && detail["provenance"]
                    == crate :: report ! ({ "source_sha256" : source , "hardware_sha256" : hardware , "synthetic" : true }),
            "feedback assertions/provenance differ"
        );
        let process = origins::process(
            owner,
            &format!("feedback-{case}"),
            manifest,
            None,
            if ["wrong-instruction", "transient-effect", "empty-compilation"].contains(&case) { &[1] } else { &[0] },
        )?;
        if LIFECYCLE.contains(&case) {
            let driver = origins::artifact(owner, "feedback-driver", manifest)?;
            ensure!(
                process["command"]
                    == crate::report!(vec![
                        driver.to_string_lossy().into_owned(),
                        "readiness-feedback-case".to_string(),
                        "--case".to_string(),
                        case.to_string(),
                        "--out".to_string(),
                        path.parent().unwrap().to_string_lossy().into_owned()
                    ]),
                "lifecycle process scope differs"
            );
            for observation in actual["observations"].as_table().context("lifecycle observations missing")?.values() {
                let candidate = Path::new(crate::documents::field(observation, "run")?);
                ensure!(candidate.starts_with(path.parent().unwrap()), "lifecycle origin escaped private repository");
                let observed = terminal(candidate)?;
                ensure!(
                    crate::readiness_metadata::value(&observed).expect("validated native metadata")
                        == observation["status"],
                    "lifecycle preserved verdict differs"
                );
            }
        } else {
            ensure!(
                process["command"]
                    == crate::report!(vec![
                        binary.to_string_lossy().into_owned(),
                        case.to_string(),
                        path.parent().unwrap().to_string_lossy().into_owned(),
                        source.to_string(),
                        hardware.to_string()
                    ])
                    && actual["binary_sha256"]
                        == crate::readiness_metadata::value(&digest(&binary)?).expect("validated native metadata"),
                "feedback executed binary/scope differs"
            );
            let ids = &actual["identities"];
            ensure!(
                ids["source_sha256"] == crate::readiness_metadata::value(&source).expect("validated native metadata")
                    && ids["hardware_sha256"]
                        == crate::readiness_metadata::value(&hardware).expect("validated native metadata")
                    && ids["rom_sha256"]
                        == crate::readiness_metadata::value(&digest(&path.parent().unwrap().join("fixture.sfc"))?)
                            .expect("validated native metadata")
                    && ids["input_sha256"]
                        == crate::readiness_metadata::value(&digest(&path.parent().unwrap().join("fixture.inputs"))?)
                            .expect("validated native metadata"),
                "feedback fixture identity differs"
            );
            if ["uncovered-key", "empty-compilation"].contains(&case) {
                let original = origins::document(&path.parent().unwrap().join("report.toml"))?;
                ensure!(
                    actual["profile"] == original["profile"] && actual["identities"] == original["identities"],
                    "coverage differs from actual probe profile"
                );
                let runs = path.parent().unwrap().join("coverage-repository/out/runs");
                let profiles = fs::read_dir(runs)?
                    .map(|e| Ok(e?.path().join("checks/profile.toml")))
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .filter(|p| p.exists())
                    .collect::<Vec<_>>();
                ensure!(
                    profiles.len() == 1
                        && digest(&profiles[0])? == crate::documents::field(&actual["coverage"], "profile_sha256")?,
                    "coverage actual profile differs"
                );
                let candidate = profiles[0].parent().and_then(Path::parent).context("coverage owner missing")?;
                let saved = origins::document(&candidate.join("manifest.toml"))?;
                let predecessor = if case == "uncovered-key" {
                    Some(
                        candidate
                            .parent()
                            .context("coverage run parent missing")?
                            .join(crate::documents::field(&actual["coverage"]["predecessor"], "run_id")?),
                    )
                } else {
                    None
                };
                ensure!(
                    crate::readiness_metadata::value(&coverage::derive(
                        candidate,
                        &saved,
                        &profiles[0],
                        &candidate.join("generated/key-inventory.toml"),
                        predecessor.as_deref()
                    )?)
                    .expect("validated native metadata")
                        == actual["coverage"],
                    "feedback coverage differs from independently derived execution"
                );
            }
        }
    }
    Ok(())
}
pub fn demonstrations(run: &mut Run, timeout: u64) -> Result<Value> {
    capture_failure(|| demonstrations_inner(run, timeout))
}
fn demonstrations_inner(run: &mut Run, timeout: u64) -> Result<Value> {
    let started = Instant::now();
    let output = run.path.join("diagnostics/feedback");
    fs::create_dir(&output)?;
    let target = run.path.join("build/feedback");
    let image = if cfg!(target_os = "linux") { PathBuf::from("/proc/self/exe") } else { std::env::current_exe()? };
    run.record_artifact("feedback-driver", &image, Some("generated/feedback-driver"))?;
    let driver = run.path.join("generated/feedback-driver");
    run.guard(&driver)?;
    let env = BTreeMap::from([
        ("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned()),
        ("CARGO_INCREMENTAL".into(), "0".into()),
    ]);
    let root = run.root.clone();
    let stage = run.stage(
        "feedback-build",
        &[
            "cargo",
            "build",
            "--release",
            "-p",
            "snes",
            "--bin",
            "feedback-probe",
            "--locked",
            "--offline",
            "--jobs",
            "1",
        ]
        .map(str::to_owned),
        &root,
        &env,
        Duration::from_secs(timeout),
        &BTreeMap::new(),
    )?;
    let binary = run.path.join("generated/feedback-probe");
    if stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata") {
        run.record_artifact(
            "feedback-binary",
            &target.join("release/feedback-probe"),
            Some("generated/feedback-probe"),
        )?;
        run.guard(&binary)?;
    }
    let source = crate::documents::field(&run.manifest.identities["source"], "sha256")?.to_owned();
    let hardware = crate::documents::field(&run.manifest.identities["hardware"], "source_sha256")?.to_owned();
    let mut cases = BTreeMap::new();
    for (case, expected) in CASES {
        let result = capture_failure(|| -> Result<Value> {
            let directory = output.join(case);
            let lifecycle = LIFECYCLE.contains(&case);
            let argv = if lifecycle {
                vec![
                    driver.to_string_lossy().into_owned(),
                    "readiness-feedback-case".into(),
                    "--case".into(),
                    case.into(),
                    "--out".into(),
                    directory.to_string_lossy().into_owned(),
                ]
            } else {
                ensure!(
                    stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
                    "feedback executable did not build"
                );
                vec![
                    binary.to_string_lossy().into_owned(),
                    case.into(),
                    directory.to_string_lossy().into_owned(),
                    source.clone(),
                    hardware.clone(),
                ]
            };
            let exit =
                if ["wrong-instruction", "transient-effect", "empty-compilation"].contains(&case) { 1 } else { 0 };
            let process = if lifecycle {
                run.stage_controller(
                    &format!("feedback-{case}"),
                    &argv,
                    &root,
                    &BTreeMap::new(),
                    Duration::from_secs(timeout),
                )?
            } else {
                run.stage(
                    &format!("feedback-{case}"),
                    &argv,
                    &root,
                    &BTreeMap::new(),
                    Duration::from_secs(timeout),
                    &BTreeMap::from([(1, "pass".into())]),
                )?
            };
            ensure!(
                process["returncode"] == crate::readiness_metadata::value(&exit).expect("validated native metadata")
                    && process["complete"]
                        == crate::readiness_metadata::value(&true).expect("validated native metadata")
                    && process["status"]
                        == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
                "feedback process did not complete expected outcome"
            );
            if ["uncovered-key", "empty-compilation"].contains(&case) {
                coverage_control(case, &directory)?;
            }
            let path = directory.join(if ["uncovered-key", "empty-compilation"].contains(&case) {
                "coverage-control.toml"
            } else {
                "report.toml"
            });
            let actual = origins::document(&path)?;
            Ok(
                crate :: report ! ({ "status" : "pass" , "complete" : true , "expected_outcome" : expected , "observed_outcome" : actual ["status"] , "assertions" : assertions (case , & actual) ? , "evidence" : crate :: report ! ({ "path" : path . strip_prefix (& run . path) ? , "sha256" : digest (& path) ? , }) , "provenance" : crate :: report ! ({ "source_sha256" : source , "hardware_sha256" : hardware , "synthetic" : true , }) , }),
            )
        });
        let detail = match result {
            Ok(report) => report,
            Err(error) => {
                crate :: report ! ({ "status" : if run . was_interrupted () { "incomplete" } else { "error" } , "complete" : false , "expected_outcome" : expected , "reason" : format ! ("{error:#}") , })
            }
        };
        cases.insert(case, detail);
        if run.was_interrupted() {
            break;
        }
    }
    let completed = cases
        .values()
        .filter(|r| r["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"))
        .count();
    let status = if completed == 9 {
        "pass"
    } else if run.was_interrupted() {
        "incomplete"
    } else {
        "error"
    };
    let report = crate :: report ! ({ "schema_version" : 1 , "status" : status , "complete" : status == "pass" , "expected_cases" : 9 , "completed_cases" : completed , "cases" : cases , "total_seconds" : started . elapsed () . as_secs_f64 () , "scope" : "synthetic fault controls; full domain acceptance requires every mandatory gate" , });
    if status == "pass" {
        verify(&run.path, &Value::try_from(&Value::try_from(&run.manifest)?)?, &report)?;
    }
    atomic_toml(&output.join("report.toml"), &report)?;
    run.check_sources()?;
    run.record_check("feedback-demonstrations", report.clone())?;
    Ok(report)
}
pub fn operate(request: Value) -> Result<Value> {
    let root = request.get("checkout").and_then(Value::as_str).map(PathBuf::from).unwrap_or(std::env::current_dir()?);
    let mut run = Run::create(&root, "feedback-demonstrations", crate :: report ! ({ "synthetic_cases" : 9 , }), None)?;
    eprintln!("Run: {}", run.path.display());
    let result = capture_failure(|| -> Result<()> {
        let failures = preflight::verify(&mut run)?;
        ensure!(failures.is_empty(), "preflight failed: {}", failures.join("; "));
        demonstrations(
            &mut run,
            request.get("timeout").and_then(Value::as_integer).and_then(|n| u64::try_from(n).ok()).unwrap_or(2400),
        )?;
        Ok(())
    });
    run.finalize(
        if result.is_ok() {
            "pass"
        } else if run.was_interrupted() {
            "incomplete"
        } else {
            "error"
        },
        Some("feedback-demonstrations"),
        result.err().as_ref().map(|e| e.to_string()).as_deref(),
        &["preflight".into(), "feedback-demonstrations".into()],
    )
}
