//! Original composite prerequisite failure through the installed supervisor's
//! public lifecycle. This intentionally incomplete fixture grants no native,
//! managed production, Live replay or original-ROM acceptance.
use super::*;
use crate::translation_execution::{self as supervisor, Contract, InstalledFile, tests::Project};
use offline_verification::fixtures::Asset;
use std::{
    process::Command,
    time::{Duration, Instant, SystemTime},
};
use translation_execution::model::{ClaimLineage, Disposition, State, View};

const OWNED: [&str; 3] = [
    "crates/port/src/pause_equipment_render.rs",
    "crates/port/tests/pause_equipment_render.rs",
    "tools/port/pause-equipment-render.smf",
];
const MISSING_AUXILIARY: &str =
    "pause equipment requires auxiliary direct animation; plain Native-family acceptance is insufficient";
const LIMITATION: &str = "Isolated Original prerequisite-refusal fixture. The primary declaration is retained, but the required auxiliary family is deliberately absent. No passing native or managed production evidence is supplied.";

fn git(root: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("/usr/bin/git")
        .args(["-c", "core.hooksPath=/dev/null", "-c", "core.fsmonitor=false"])
        .args(arguments)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .current_dir(root)
        .output()?;
    ensure!(output.status.success(), "Original fixture Git failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(String::from_utf8(output.stdout)?.trim().into())
}

fn put(root: &Path, name: &str, bytes: &[u8]) -> Result<String> {
    let path = root.join(name);
    fs::create_dir_all(path.parent().context("Original fixture parent missing")?)?;
    fs::write(path, bytes)?;
    Ok(sha256(bytes))
}

fn asset(reference: Reference) -> Asset {
    Asset { file: reference.path, sha256: reference.sha256 }
}

fn fixture() -> Result<Project> {
    let mut project = Project::new()?;
    let source = &project.installation.source_repository;
    let verification = project.root.join("verification-source");
    let contents: [(&str, &[u8]); 6] = [
        (OWNED[0], include_bytes!("../../port/src/pause_equipment_render.rs")),
        (OWNED[1], include_bytes!("../../port/tests/pause_equipment_render.rs")),
        (OWNED[2], include_bytes!("../../../tools/port/pause-equipment-render.smf")),
        ("Cargo.toml", include_bytes!("../../../Cargo.toml")),
        ("Cargo.lock", include_bytes!("../../../Cargo.lock")),
        ("crates/port/Cargo.toml", include_bytes!("../../port/Cargo.toml")),
    ];
    let mut files = BTreeMap::new();
    for (name, bytes) in contents {
        put(source, name, bytes)?;
        files.insert(name.to_owned(), put(&verification, name, bytes)?);
    }
    git(source, &["add", "Cargo.toml", "Cargo.lock", "crates", "tools"])?;
    git(source, &["commit", "-qm", "Frozen original pause prerequisite fixture"])?;
    let baseline = git(source, &["rev-parse", "HEAD"])?;
    let original_files: BTreeMap<String, String> =
        OWNED.iter().map(|name| ((*name).into(), files[*name].clone())).collect();
    let claim = assignment::Claim {
        id: "pause-equipment-render-offline".into(),
        revision: "original-prerequisite-fixture".into(),
        attempt: 1,
        coordinator: "decomp-run-20261013".into(),
    };
    let original = OriginalClaim {
        schema: "translation-original-claim-v1".into(),
        checkout: source.clone(),
        effort: "decomp-parallel".into(),
        job: assignment::Job {
            id: claim.id.clone(),
            revision: claim.revision.clone(),
            kind: assignment::Kind::Implement,
            subject: "Original pause composite prerequisite refusal".into(),
            writable_paths: OWNED.iter().map(PathBuf::from).collect(),
            blockers: vec![],
            inputs: BTreeMap::from([("admission-domain".into(), "offline-coverage-v1".into())]),
            instructions: LIMITATION.into(),
            ticket: None,
        },
        claim: claim.clone(),
        state: assignment::State::Submitted,
        agent: Some("original-pause-submitter".into()),
        result: Some(assignment::WorkResult::Submitted { commit: baseline.clone(), evidence: vec![] }),
        observations: vec![],
        source_checkout: source.clone(),
        source_commit: baseline.clone(),
        source_files: original_files.clone(),
    };
    let original_claim = write(&project.root.join("original-claim.json"), &original)?;
    let vendor = project.root.join("vendor");
    fs::create_dir(&vendor)?;
    let cargo = cargo::CargoContract {
        schema: "translation-cargo-contract-v1".into(),
        kind: cargo::CargoKind::PauseEquipment,
        source_files: files.clone(),
        approved_files: OWNED.iter().map(|name| (*name).into()).collect(),
        vendor: cargo::CargoClosure { root: vendor, files: BTreeMap::new() },
        dependency_cache: None,
        limits: cargo::CargoLimits {
            timeout_seconds: 60,
            writable_bytes: 256 * 1024 * 1024,
            stdout_bytes: 65536,
            stderr_bytes: 65536,
        },
    };
    let cargo = write(&project.root.join("cargo-contract.json"), &cargo)?;
    let requirement = write(&project.root.join("original-requirements.json"), &LIMITATION)?;
    let catalog = project.root.join("catalog.toml");
    fs::write(&catalog, "[families]\npause-equipment-render = \"family\"\n[profiles]\n")?;
    let manifest = offline_verification::fixtures::decode(contents[2].1.to_vec())?;
    let coverage: BTreeMap<String, Vec<String>> = manifest
        .routines
        .iter()
        .map(|routine| (routine.symbol.clone(), routine.cases.iter().map(|fixture| fixture.name.clone()).collect()))
        .collect();
    ensure!(coverage.values().map(Vec::len).sum::<usize>() == 247, "original primary Fixture declaration changed");
    // Completeness is checked before native provenance or C preparation. These
    // declarations preserve the real primary manifest/coverage and cannot be
    // mistaken for a prepared native proof: its auxiliary declaration is absent.
    let primary: native::NativeContract = serde_json::from_value(serde_json::json!({
        "family": manifest.family,
        "manifest": Asset { file: OWNED[2].into(), sha256: files[OWNED[2]].clone() },
        "source_inventory": Asset {
            file: "unavailable-before-composite-completeness.toml".into(),
            sha256: sha256(LIMITATION),
        },
        "coverage": coverage,
        "provenance": native::NativeProvenance::ReviewedCatalog {
            catalog: asset(retention::seal_file(&catalog, MAX_RECORD)?),
            requirements: asset(requirement),
            checkout: verification.clone(),
            rationale: LIMITATION.into(),
        },
        "frozen_inputs": files.clone(),
        "compiler_limits": native::NativeLimits {
            timeout_seconds: 60,
            writable_bytes: 128 * 1024 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 65536,
            stdin_bytes: 0,
        },
        "fixture_limits": native::NativeLimits {
            timeout_seconds: 5,
            writable_bytes: 512 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        },
    }))?;
    let primary = write(&project.root.join("primary-family.json"), &primary)?;
    let rustc = &project.installation.rustc.path;
    let cargo_executable = rustc.parent().context("fixture toolchain missing")?.join("cargo");
    let contract = OriginalContract {
        schema: "translation-original-contract-v1".into(),
        kind: OriginalKind::PauseEquipment,
        original_claim,
        legacy: None,
        verification_source: VerificationSource { root: verification, files },
        cargo,
        cargo_executable: retention::seal_file(&cargo_executable, MAX_SOURCE)?,
        cargo_rustc: retention::seal_file(rustc, MAX_SOURCE)?,
        families: BTreeMap::from([("pause-equipment-render".into(), primary)]),
        compact: None,
        catalog_sha256: None,
    };
    let policy = &mut project.installation.policy;
    policy.scope_id = "original-pause-prerequisite-failure".into();
    policy.claim =
        ClaimLineage { id: claim.id, revision: claim.revision, attempt: claim.attempt, coordinator: claim.coordinator };
    policy.baseline_commit = baseline.clone();
    policy.approved_files = original_files;
    policy.verification_stages = vec![VerificationStage::Focused, VerificationStage::Offline];
    policy.budgets.max_attempts = 2;
    policy.budgets.gate_timeout_seconds = 60;
    policy.budgets.max_evidence_bytes = 1024 * 1024 * 1024;
    policy.dependencies_sha256 = sealed(&contract)?;
    policy.verification_sha256 = sealed(&(LIMITATION, &contract))?;
    policy.pinned_c_sha256 = sha256("no original C acceptance before complete composite preparation");
    project.installation.id = "original-prerequisite-refusal".into();
    project.installation.integration_baseline = baseline;
    project.installation.contract = Contract::Original { contract };
    project.installation.admission = supervisor::Admission::IsolatedProof { limitations: vec![LIMITATION.into()] };
    // The real confinement-readiness process exits immediately. A rewrite is
    // never reached; no test-only execution hook or model response is installed.
    let probe = project.root.join("installed/original-prerequisite-probe");
    fs::copy("/usr/bin/false", &probe)?;
    project.installation.rewrite.executable = InstalledFile { sha256: crate::digest(&probe)?, path: probe };
    project.installation.rewrite.recorded = false;
    project.installation.policy.adapter_sha256 = supervisor::installation_identity(&project.installation)?;
    write(&project.root.join("installation.json"), &project.installation)?;
    Ok(project)
}

fn retained_files(root: &Path) -> Result<BTreeMap<PathBuf, (String, u64, SystemTime)>> {
    supervisor::tests::records(root)?
        .into_iter()
        .map(|(path, (digest, bytes))| {
            let modified = fs::metadata(&path)?.modified()?;
            Ok((path, (digest, bytes, modified)))
        })
        .collect()
}

fn unchanged_replay(installation: &supervisor::Installation, stopped: &View) -> Result<()> {
    let retained = retained_files(&installation.state_root)?;
    for action in ["inspect", "resume", "run", "inspect"] {
        ensure!(
            supervisor::execute(installation.clone(), action, None)? == *stopped,
            "{action} changed the stopped view"
        );
        ensure!(
            retained_files(&installation.state_root)? == retained,
            "{action} added or rewrote an attempt, gate, model, operation, capture or retained diagnosis"
        );
    }
    Ok(())
}

#[test]
fn original_missing_auxiliary_prerequisite_stops_once_and_resume_preserves_submission() -> Result<()> {
    let project = fixture()?;
    let installation = &project.installation;
    let policy = installation.policy.clone();
    let source_before = supervisor::tests::records(&installation.source_repository)?;
    let inputs_before = supervisor::tests::records(&project.root.join("verification-source"))?;
    let claim_before = fs::read(project.root.join("original-claim.json"))?;
    let installation_before = fs::read(project.root.join("installation.json"))?;
    let started = Instant::now();
    let stopped = supervisor::execute(installation.clone(), "run", None)?;
    ensure!(started.elapsed() < Duration::from_secs(60), "Original prerequisite stop exceeded its frozen bound");
    let State::Stopped { disposition: Disposition::Failed, reason, finalization: Some(finalization) } = &stopped.state
    else {
        anyhow::bail!("Original prerequisite did not produce a finalized failed stop: {stopped:?}");
    };
    ensure!(reason.contains(MISSING_AUXILIARY), "unrelated failure cannot satisfy the Original regression: {reason}");
    ensure!(stopped.attempt == 1 && stopped.policy_sha256 == sealed(&policy)?, "stopped original identity changed");
    ensure!(
        finalization.cleanup_complete && finalization.retained_authenticatable && finalization.worker_retired,
        "Original prerequisite failure omitted cleanup, retention or checkout retirement"
    );
    ensure!(
        finalization.retained_bytes > 0 && finalization.retained_bytes <= policy.budgets.max_evidence_bytes,
        "Original stopped evidence escaped its frozen byte bound"
    );
    unchanged_replay(installation, &stopped)?;
    ensure!(supervisor::tests::records(&installation.source_repository)? == source_before, "Submitted source changed");
    ensure!(
        supervisor::tests::records(&project.root.join("verification-source"))? == inputs_before,
        "frozen verification input changed"
    );
    ensure!(fs::read(project.root.join("original-claim.json"))? == claim_before, "original Submitted claim changed");
    ensure!(
        fs::read(project.root.join("installation.json"))? == installation_before,
        "frozen installed policy changed"
    );
    write(&project.root.join("stopped-view.json"), &stopped)?;
    println!("Original supervisor proof root: {}", project.root.display());
    Ok(())
}

#[test]
#[ignore = "read-only replay of a retained Original prerequisite fixture after cleanup"]
fn retained_original_prerequisite_failure_authenticates_without_execution() -> Result<()> {
    let root = PathBuf::from(std::env::var("SM_ORIGINAL_SUPERVISOR_PROOF_ROOT")?);
    let installation: supervisor::Installation = serde_json::from_slice(&fs::read(root.join("installation.json"))?)?;
    let stopped: View = serde_json::from_slice(&fs::read(root.join("stopped-view.json"))?)?;
    ensure!(
        matches!(&stopped.state, State::Stopped {
            disposition: Disposition::Failed, reason, finalization: Some(_)
        } if reason.contains(MISSING_AUXILIARY)),
        "retained fixture is not the expected Original prerequisite failure"
    );
    let before = retained_files(&root)?;
    unchanged_replay(&installation, &stopped)?;
    ensure!(retained_files(&root)? == before, "retained replay wrote source, inputs or evidence");
    println!("Original prerequisite failure authenticated without new work: {}", root.display());
    Ok(())
}
