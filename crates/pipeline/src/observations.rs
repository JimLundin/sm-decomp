//! Authenticate retained observations using their sealed Rust reader executable.
use crate::{acquire_lease, digest};
use anyhow::{Context, Result, ensure};
use evidence::{Inventory, Pointer};
use serde::{Deserialize, Serialize};
#[derive(Deserialize, Serialize)]
pub struct AuthenticateRequest {
    pub verdict: std::path::PathBuf,
    pub root: std::path::PathBuf,
    pub workers: usize,
}
#[derive(Deserialize, Serialize)]
pub struct AuthenticationReceipt {
    pub status: String,
    pub artifacts_per_subject: usize,
    pub subjects: usize,
    pub binary: bool,
    pub storage: String,
    pub verdict_sha256: String,
    pub scope: String,
}
#[derive(Deserialize)]
struct SealedSubject {
    pointer_sha256: String,
    run: String,
    observations: String,
}
#[derive(Deserialize)]
struct EvidenceStorage {
    format: String,
    subjects: BTreeMap<String, SealedSubject>,
}
#[derive(Deserialize)]
struct Verdict {
    state_effects_match: bool,
    artifacts: BTreeMap<String, String>,
    binary_sha256: String,
    evidence_reader_sha256: String,
    evidence_storage: EvidenceStorage,
    #[serde(default)]
    projectile_native_installed: bool,
    projectile_abi: Option<String>,
    #[serde(default)]
    frames: u64,
    #[serde(default)]
    counters: BTreeMap<String, crate::live_model::Counters>,
}
use std::{
    collections::BTreeMap,
    fs::{self},
    path::Path,
    process::Command,
};
const SUBJECTS: [&str; 4] = ["c-0", "c-1", "mixed-0", "mixed-1"];

fn document<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    ensure!(!path.is_symlink(), "evidence document is a symlink");
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
pub fn subject(directory: &Path, root: &Path, workers: usize, expected_pointer: Option<&str>) -> Result<Pointer> {
    let context = crate::metrics::active();
    subject_with_metrics(directory, root, workers, expected_pointer, context.as_ref())
}
pub(crate) fn subject_with_metrics(
    directory: &Path,
    root: &Path,
    workers: usize,
    expected_pointer: Option<&str>,
    metrics: Option<&crate::metrics::Context>,
) -> Result<Pointer> {
    ensure!(!directory.is_symlink(), "subject directory is a symlink");
    let pointer_path = directory.join("evidence.toml");
    if let Some(expected) = expected_pointer {
        ensure!(digest(&pointer_path)? == expected, "subject pointer changed");
    }
    let pointer: Pointer = document(&pointer_path)?;
    ensure!(pointer.format == "sm-evidence-v2", "unknown evidence format");
    let store_path = directory.join(pointer.store.as_ref().context("pointer store missing")?);
    ensure!(!store_path.is_symlink(), "evidence store is a symlink");
    let store = fs::canonicalize(store_path)?;
    let reader = directory.parent().context("subject parent missing")?.join("sm-evidence");
    let expected = pointer.provenance.reader_sha256.as_deref().context("reader digest missing")?;
    ensure!(!reader.is_symlink() && digest(&reader)? == expected, "retained evidence reader changed");
    let (_held, _wait) = acquire_lease(root, workers)?;
    let mut command = Command::new(reader);
    command.arg("inventory").arg(store).arg(&pointer.run);
    let output = if let Some(metrics) = metrics {
        metrics.output(&mut command, "subject-authentication")?
    } else {
        command.output()?
    };
    ensure!(
        output.status.success(),
        "packed evidence authentication failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Inventory = toml::from_str(std::str::from_utf8(&output.stdout)?)?;
    ensure!(
        actual.observations == pointer.observations
            && actual.files == pointer.files
            && actual.bytes == pointer.bytes
            && actual.artifacts == pointer.artifacts
            && actual.provenance == pointer.provenance,
        "packed evidence inventory differs"
    );
    Ok(pointer)
}

pub fn authenticate(request: AuthenticateRequest) -> Result<AuthenticationReceipt> {
    let path = &request.verdict;
    let verdict: Verdict = document(path)?;
    let run = path.parent().context("verdict parent missing")?;
    ensure!(verdict.state_effects_match && !verdict.artifacts.is_empty(), "no finished matching observation inventory");
    let binary = run.join("live");
    ensure!(!binary.is_symlink() && digest(&binary)? == verdict.binary_sha256, "retained live binary changed");
    let packed = &verdict.evidence_storage;
    ensure!(packed.format == "sm-evidence-v2", "unknown packed format");
    ensure!(
        packed.subjects.len() == 4 && SUBJECTS.iter().all(|name| packed.subjects.contains_key(*name)),
        "packed subjects incomplete"
    );
    ensure!(digest(&run.join("sm-evidence"))? == verdict.evidence_reader_sha256, "retained evidence reader changed");
    for name in SUBJECTS {
        let sealed = &packed.subjects[name];
        let pointer = subject(&run.join(name), &request.root, request.workers, Some(&sealed.pointer_sha256))?;
        let expected = pointer
            .artifacts
            .iter()
            .filter(|(name, _)| {
                Path::new(name).file_name().is_none_or(|v| v != "calls.bin")
                    && Path::new(name).extension().is_none_or(|v| v != "rom")
            })
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect::<BTreeMap<_, _>>();
        ensure!(
            expected == verdict.artifacts && pointer.run == sealed.run && pointer.observations == sealed.observations,
            "packed observations differ from verdict"
        );
        ensure!(
            pointer.provenance.binary_sha256.as_deref() == Some(verdict.binary_sha256.as_str())
                && pointer.provenance.subject.as_deref() == Some(name),
            "packed producer provenance differs"
        );
    }
    if verdict.projectile_native_installed {
        ensure!(
            verdict.projectile_abi.as_deref() == Some("synchronous-eight-word-host-v2-with-gfx"),
            "unknown native projectile ABI"
        );
        for name in SUBJECTS {
            let recorded = verdict.counters.get(name).context("missing native projectile subject counters")?;
            let symbols = recorded.hits.keys().cloned().collect::<Vec<_>>();
            // port-hit rows retain generation order, while map keys are sorted.
            // Projectile admission compares its typed counters and progress.
            let actual = crate::live::counters(&run.join(name), &symbols, name.split('-').next().unwrap())?;
            ensure!(
                actual.projectile_hits == recorded.projectile_hits
                    && actual.projectile_id_hits == recorded.projectile_id_hits
                    && actual.projectile_allocation_hits == recorded.projectile_allocation_hits
                    && actual.progress == recorded.progress,
                "native projectile counters differ from retained stdout"
            );
        }
        crate::live::projectile_admission(&verdict.counters, &verdict.artifacts, verdict.frames)?;
    }
    Ok(AuthenticationReceipt {
        status: "authenticated".into(),
        artifacts_per_subject: verdict.artifacts.len(),
        subjects: 4,
        binary: true,
        storage: "sm-evidence-v2".into(),
        verdict_sha256: digest(path)?,
        scope: "retained frozen binary and observations; current-source admission and original-ROM parity separate"
            .into(),
    })
}
