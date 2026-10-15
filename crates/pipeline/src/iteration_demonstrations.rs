use crate::digest;
use crate::iteration_diagnosis;
use crate::iteration_diagnosis::Request;
use crate::iteration_session;
use crate::readiness_origins as origins;
use crate::readiness_run::Run;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};
use toml::Value;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Report {
    schema_version: u32,
    scope: String,
    accepted_count: u64,
    source_sha256: String,
    policy_sha256: String,
    probe_sha256: String,
    history_root: PathBuf,
    candidates: Vec<PathBuf>,
    pointer_before: Option<String>,
    pointer_after: Option<String>,
    private_pointer_sha256: String,
}
fn pointer(root: &Path) -> Result<Option<String>> {
    let path = root.join("out/accepted.toml");
    if path.exists() { Ok(Some(digest(&path)?)) } else { Ok(None) }
}
fn probe(output: &Path, source: &str, binary: &Path) -> Result<Value> {
    let report = origins::document(&output.join("report.toml"))?;
    ensure!(
        report["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && report["complete"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && report["accepted_count"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
            && report["source_sha256"] == crate::readiness_metadata::value(&source).expect("validated native metadata")
            && report["binary_sha256"]
                == crate::readiness_metadata::value(&digest(binary)?).expect("validated native metadata")
            && report["rom_sha256"]
                == crate::readiness_metadata::value(&digest(&output.join("fixture.sfc"))?)
                    .expect("validated native metadata")
            && report["input_sha256"]
                == crate::readiness_metadata::value(&digest(&output.join("fixture.inputs"))?)
                    .expect("validated native metadata")
            && report["early"]["status"]
                == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && report["early"]["completed_instructions"]
                == crate::readiness_metadata::value(&65).expect("validated native metadata")
            && report["candidate"]["status"]
                == crate::readiness_metadata::value(&"divergence").expect("validated native metadata")
            && report["candidate"]["first_mismatch"]["instruction"]
                == crate::readiness_metadata::value(&86).expect("validated native metadata")
            && report["candidate"]["first_mismatch"]["subject"]["generated_key"] == crate::report!(vec![0x8004, 3]),
        "late occurrence probe scope or identities differ"
    );
    if report["schema_version"].as_integer() == Some(3) {
        let cache = &report["cache"];
        let validation = origins::document(&output.join("divergent-cache/validation.toml"))?;
        ensure!(
            cache["validation"] == validation
                && cache["status"].as_str() == Some("pass")
                && cache["complete"].as_bool() == Some(true)
                && cache["validation"]["complete"].as_bool() == Some(true)
                && cache["validation"]["observed_rollovers"].as_integer() == Some(2)
                && cache["validation"]["candidate_reference_comparison"].as_str() == Some("divergence"),
            "late occurrence cache did not record actual divergence"
        );
        let warm = origins::document(&output.join("warm/instruction.toml"))?;
        ensure!(
            report["warm"] == warm
                && warm["status"].as_str() == Some("divergence")
                && warm["complete"].as_bool() == Some(true)
                && warm["first_mismatch"]["instruction"].as_integer() == Some(86)
                && warm["first_mismatch"]["subject"]["generated_key"] == crate::report!(vec![0x8004, 3]),
            "restored cache hid late candidate divergence"
        );
    } else {
        ensure!(
            report["schema_version"].as_integer() == Some(2)
                && report["cache_refusal"].as_str().is_some_and(|s| !s.is_empty()),
            "historical late occurrence cache refusal missing"
        );
    }
    for name in ["predecessor", "disabled"] {
        ensure!(
            report[name]["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
                && report[name]["completed_instructions"]
                    == crate::readiness_metadata::value(&87).expect("validated native metadata")
                && report[name]["start_input_index"]
                    == crate::readiness_metadata::value(&0).expect("validated native metadata"),
            "late occurrence control differs"
        );
    }
    for name in ["reference-before", "subject-before", "reference-after", "subject-after"] {
        ensure!(
            output.join("candidate").join(format!("{name}.hardware.bin")).is_file(),
            "late mismatch snapshot missing"
        );
    }
    Ok(report)
}
pub(crate) fn operate(root: &Path, timeout: u64) -> Result<Value> {
    let before = pointer(root)?;
    let mut run = Run::create(
        root,
        "iteration-demonstrations",
        crate :: report ! ({ "scope" : "synthetic nonaccepting" , }),
        None,
    )?;
    eprintln!("Run: {}", run.path.display());
    let result = (|| -> Result<()> {
        run.capture_source()?;
        let source = crate::documents::field(&run.manifest.identities["source"], "sha256")?.to_owned();
        iteration_session::policy(&run.path.join("diagnostics/policy"))?;
        let private = run.path.join("diagnostics/lineage");
        fs::create_dir(&private)?;
        ensure!(
            Command::new("git").args(["init", "-q"]).arg(&private).status()?.success(),
            "private repository initialization failed"
        );
        fs::create_dir(private.join("crates"))?;
        fs::write(private.join("crates/control.rs"), "// simulated failure\n")?;
        let mut candidates = Vec::new();
        for _ in 0..3 {
            let mut candidate =
                Run::create(&private, "iteration", crate::readiness_contract::domain(), Some("symbolic-parent"))?;
            candidate.capture_source()?;
            candidate . record_check ("generated-parity" , crate :: report ! ({ "status" : "divergence" , "complete" : true , "first_divergence" : crate :: report ! ({ "frame" : 0 , }) , "scope" : "simulation" , }) ,) ? ;
            candidate.finalize("divergence", Some("generated-parity"), Some("simulated execution failure"), &[])?;
            iteration_diagnosis::operate(Request {
                checkout: private.clone(),
                increment_id: "synthetic-increment".into(),
                history: false,
                session_id: Some("synthetic-session".into()),
                candidate: Some(candidate.path.clone()),
                hypothesis: Some("later key changes effects".into()),
                discriminator: Some("cold late instruction differs".into()),
                result: Some("no new discriminating evidence".into()),
                evidence: vec![],
            })?;
            candidates.push(candidate.path);
        }
        fs::write(private.join("out/accepted.toml"), b"symbolic_predecessor = \"preserve\"\n")?;
        let private_hash = digest(&private.join("out/accepted.toml"))?;
        let driver = run.path.join("generated/demonstration-driver");
        run.record_artifact("demonstration-driver", &std::env::current_exe()?, Some("generated/demonstration-driver"))?;
        let command = vec![
            driver.to_string_lossy().into_owned(),
            "readiness-accept".into(),
            "--run".into(),
            candidates.last().unwrap().to_string_lossy().into_owned(),
        ];
        let refusal = run.stage(
            "failed-publication",
            &command,
            &private,
            &BTreeMap::new(),
            Duration::from_secs(timeout),
            &BTreeMap::from([(2, "pass".into())]),
        )?;
        ensure!(
            refusal["returncode"] == crate::readiness_metadata::value(&2).expect("validated native metadata")
                && digest(&private.join("out/accepted.toml"))? == private_hash,
            "failed publication changed private predecessor"
        );
        let target = run.path.join("build/iteration-probe");
        let env = BTreeMap::from([
            ("CARGO_TARGET_DIR".into(), target.to_string_lossy().into_owned()),
            ("CARGO_INCREMENTAL".into(), "0".into()),
        ]);
        let stage = run.stage(
            "iteration-probe-build",
            &[
                "cargo",
                "build",
                "--release",
                "-p",
                "snes",
                "--bin",
                "iteration-probe",
                "--locked",
                "--offline",
                "--jobs",
                "1",
            ]
            .map(str::to_owned),
            root,
            &env,
            Duration::from_secs(timeout),
            &BTreeMap::new(),
        )?;
        ensure!(
            stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "iteration probe build failed"
        );
        run.record_artifact(
            "iteration-probe",
            &target.join("release/iteration-probe"),
            Some("generated/iteration-probe"),
        )?;
        let output = run.path.join("diagnostics/late-occurrence");
        let binary = run.path.join("generated/iteration-probe");
        let stage = run.stage(
            "late-occurrence-probe",
            &[binary.to_string_lossy().into_owned(), output.to_string_lossy().into_owned(), source.clone()],
            root,
            &BTreeMap::new(),
            Duration::from_secs(timeout),
            &BTreeMap::new(),
        )?;
        ensure!(
            stage["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
            "late occurrence probe failed"
        );
        probe(&output, &source, &binary)?;
        let report = Report {
            schema_version: 1,
            scope: "synthetic nonaccepting".into(),
            accepted_count: 0,
            source_sha256: source,
            policy_sha256: digest(&run.path.join("diagnostics/policy/report.toml"))?,
            probe_sha256: digest(&output.join("report.toml"))?,
            history_root: private,
            candidates,
            pointer_before: before.clone(),
            pointer_after: pointer(root)?,
            private_pointer_sha256: private_hash,
        };
        ensure!(report.pointer_before == report.pointer_after, "workspace accepted pointer changed");
        let path = run.path.join("iteration-demonstrations.toml");
        pipeline_model::write(&path, &report)?;
        run.record_artifact("demonstration-report", &path, None)?;
        run . record_check ("iteration-demonstrations" , crate :: report ! ({ "status" : "pass" , "complete" : true , "accepted_count" : 0 , "evidence" : "iteration-demonstrations.toml" , "sha256" : digest (& path) ? , })) ? ;
        run.check_sources()
    })();
    match result {
        Ok(()) => run.finalize(
            "pass",
            Some("iteration-demonstrations"),
            Some("synthetic controls passed; production acceptance remains separate"),
            &["iteration-demonstrations".into()],
        ),
        Err(error) => run.finalize(
            "error",
            Some("iteration-demonstrations"),
            Some(&format!("{error:#}; workspace pointer unchanged={}", pointer(root)? == before)),
            &[],
        ),
    }
}
pub(crate) fn verify(owner: &Path) -> Result<Report> {
    let owner_path = fs::canonicalize(owner)?;
    let owner = owner_path.as_path();
    let (manifest, _) = origins::snapshot(owner)?;
    origins::owned(owner, &manifest)?;
    let verdict = origins::document(&owner.join("verdict.toml"))?;
    ensure!(
        manifest["type"]
            == crate::readiness_metadata::value(&"iteration-demonstrations").expect("validated native metadata")
            && manifest["state"] == crate::readiness_metadata::value(&"complete").expect("validated native metadata")
            && verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata")
            && verdict["acceptance_eligible"]
                == crate::readiness_metadata::value(&false).expect("validated native metadata"),
        "demonstrations did not complete"
    );
    let path = origins::artifact(owner, "demonstration-report", &manifest)?;
    let report: Report = pipeline_model::read(&path)?;
    ensure!(
        report.schema_version == 1
            && report.scope == "synthetic nonaccepting"
            && report.accepted_count == 0
            && crate::readiness_metadata::value(&report.source_sha256).expect("validated native metadata")
                == manifest["identities"]["source"]["sha256"]
            && report.pointer_before == report.pointer_after
            && crate::readiness_metadata::value(&digest(&path)?).expect("validated native metadata")
                == verdict["checks"]["iteration-demonstrations"]["sha256"],
        "demonstration scope or source differs"
    );
    let output = owner.join("diagnostics/policy");
    let policy = pipeline_model::read(&output.join("report.toml"))?;
    ensure!(digest(&output.join("report.toml"))? == report.policy_sha256, "policy checksum differs");
    iteration_session::verify_policy(&output, &policy)?;
    let binary = origins::artifact(owner, "iteration-probe", &manifest)?;
    let output = owner.join("diagnostics/late-occurrence");
    ensure!(digest(&output.join("report.toml"))? == report.probe_sha256, "probe checksum differs");
    probe(&output, &report.source_sha256, &binary)?;
    origins::process(owner, "iteration-probe-build", &manifest, None, &[0])?;
    origins::process(
        owner,
        "late-occurrence-probe",
        &manifest,
        Some(&Value::try_from(vec![
            binary.to_string_lossy().into_owned(),
            output.to_string_lossy().into_owned(),
            report.source_sha256.clone(),
        ])?),
        &[0],
    )?;
    let expected = owner.join("diagnostics/lineage");
    ensure!(report.history_root == expected && report.candidates.len() == 3, "private lineage scope differs");
    let history = iteration_diagnosis::verify_history(&expected, "synthetic-increment")?;
    ensure!(
        history.len() == 3
            && history.iter().enumerate().all(|(i, r)| r.consecutive_without_evidence == i as u64 + 1
                && r.decision_required == (i == 2)
                && r.candidate_path == report.candidates[i]),
        "three-experiment rule differs"
    );
    let driver = origins::artifact(owner, "demonstration-driver", &manifest)?;
    origins::process(
        owner,
        "failed-publication",
        &manifest,
        Some(&crate::report!(vec![
            driver,
            PathBuf::from("readiness-accept"),
            PathBuf::from("--run"),
            report.candidates.last().unwrap().clone()
        ])),
        &[2],
    )?;
    ensure!(
        digest(&expected.join("out/accepted.toml"))? == report.private_pointer_sha256,
        "private predecessor changed"
    );
    Ok(report)
}
pub(crate) fn run(args: &[String]) -> Result<()> {
    let command = args.first().context("usage: iteration-demonstrations run|verify|policy [OPTIONS]")?;
    let mut root = std::env::current_dir()?;
    let mut output = None;
    let mut timeout = 2400;
    for pair in args[1..].chunks(2) {
        ensure!(pair.len() == 2, "demonstration option value missing");
        match pair[0].as_str() {
            "--root" => root = pair[1].clone().into(),
            "--run" | "--out" => output = Some(PathBuf::from(&pair[1])),
            "--timeout" => timeout = pair[1].parse()?,
            other => anyhow::bail!("unknown demonstration option {other}"),
        }
    }
    match command.as_str() {
        "policy" => {
            println!("{}", toml::to_string_pretty(&iteration_session::policy(&output.context("--out missing")?)?)?)
        }
        "verify" => println!("{}", toml::to_string_pretty(&verify(&output.context("--run missing")?)?)?),
        "run" => {
            let verdict = operate(&root, timeout)?;
            crate::emit(&verdict)?;
            ensure!(
                verdict["status"] == crate::readiness_metadata::value(&"pass").expect("validated native metadata"),
                "demonstrations failed"
            );
        }
        other => anyhow::bail!("unknown demonstration command {other}"),
    }
    Ok(())
}
