//! Independent immutable run, process and artifact authentication.
use crate::{
    digest, family,
    readiness_run::{identity, safe_artifact},
};
use anyhow::{Context, Result, ensure};
use pipeline_model::readiness::{ArtifactIdentity, ProcessRecord, RUN_VERSION, RunManifest, SourceInventory};
use std::{
    collections::BTreeSet,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use toml::Value;
pub fn document(path: &Path) -> Result<Value> {
    pipeline_model::read(path)
}
pub fn snapshot_typed(owner: &Path) -> Result<(RunManifest, SourceInventory)> {
    let manifest: RunManifest = pipeline_model::read(&owner.join("manifest.toml"))?;
    let inventory: SourceInventory = pipeline_model::read(&owner.join("source/inventory.toml"))?;
    ensure!(
        manifest.schema_version == RUN_VERSION
            && inventory.schema_version == RUN_VERSION
            && identity(&inventory.files)? == inventory.sha256
            && Some(inventory.sha256.as_str())
                == manifest.identities.get("source").and_then(|v| v.get("sha256")).and_then(Value::as_str),
        "source inventory identity differs"
    );
    let mut names = BTreeSet::new();
    for entry in &inventory.files {
        ensure!(names.insert(&entry.path), "duplicate source snapshot path");
        let path = safe_artifact(&owner.join("source/files"), &entry.path)?;
        ensure!(
            digest(&path)? == entry.sha256 && path.metadata()?.permissions().mode() & 0o777 == entry.mode,
            "source snapshot unsafe/corrupt: {}",
            entry.path
        );
    }
    Ok((manifest, inventory))
}
pub fn snapshot(owner: &Path) -> Result<(Value, Value)> {
    let (manifest, inventory) = snapshot_typed(owner)?;
    Ok((Value::try_from(&manifest)?, Value::try_from(&inventory)?))
}
pub fn process(owner: &Path, name: &str, manifest: &Value, command: Option<&Value>, allowed: &[i64]) -> Result<Value> {
    let manifest: RunManifest = manifest.clone().try_into()?;
    let path = safe_artifact(owner, &format!("logs/{name}.toml"))?;
    let process: ProcessRecord = pipeline_model::read(&path)?;
    ensure!(
        process.complete
            && process.signal.is_none()
            && process.returncode.is_some_and(|n| allowed.contains(&i64::from(n)))
            && manifest.commands.contains(&process),
        "process failed/incomplete or lacks owner origin: {name}"
    );
    if let Some(expected) = command {
        ensure!(Value::try_from(&process.command)? == *expected, "process invocation differs: {name}");
    }
    let seal = manifest.process_evidence.get(name).context("process seal missing")?;
    ensure!(seal.report == digest(&path)?, "process report checksum differs: {name}");
    for (field, relative, hash) in
        [("stdout", &process.stdout, &seal.stdout), ("stderr", &process.stderr, &seal.stderr)]
    {
        let path = safe_artifact(owner, relative)?;
        ensure!(Some(digest(&path)?) == *hash, "process log checksum differs: {name}/{field}");
    }
    Ok(Value::try_from(process)?)
}
pub fn artifact(owner: &Path, name: &str, manifest: &Value) -> Result<PathBuf> {
    let manifest: RunManifest = manifest.clone().try_into()?;
    let entry: ArtifactIdentity =
        manifest.identities.get(name).context("artifact identity missing")?.clone().try_into()?;
    let path = safe_artifact(owner, entry.evidence.as_deref().context("artifact evidence missing")?)?;
    ensure!(digest(&path)? == entry.sha256, "owned artifact checksum differs: {name}");
    Ok(path)
}
pub fn owned(owner: &Path, manifest: &Value) -> Result<()> {
    let manifest: RunManifest = manifest.clone().try_into()?;
    for (name, entry) in &manifest.identities {
        if let Some(relative) = entry.get("evidence").and_then(Value::as_str) {
            let path = safe_artifact(owner, relative)?;
            ensure!(digest(&path)? == family::field(entry, "sha256")?, "owned artifact changed: {name}");
        }
    }
    for (name, entry) in &manifest.check_evidence {
        let path = safe_artifact(owner, &entry.path)?;
        ensure!(digest(&path)? == entry.sha256, "check evidence checksum differs: {name}");
        let report = document(&path)?;
        ensure!(report.is_table(), "check evidence must be a table: {name}");
    }
    for name in manifest.process_evidence.keys() {
        let path = safe_artifact(owner, &format!("logs/{name}.toml"))?;
        let process: ProcessRecord = pipeline_model::read(&path)?;
        let seal = &manifest.process_evidence[name];
        ensure!(
            manifest.commands.contains(&process) && digest(&path)? == seal.report,
            "process origin/checksum differs: {name}"
        );
        for (relative, hash) in [(&process.stdout, &seal.stdout), (&process.stderr, &seal.stderr)] {
            if let Some(hash) = hash {
                ensure!(digest(&safe_artifact(owner, relative)?)? == *hash, "process log checksum differs: {name}");
            }
        }
        let check = manifest.check_evidence.get(name).context("process check seal missing")?;
        ensure!(
            document(&safe_artifact(owner, &check.path)?)? == Value::try_from(&process)?,
            "process check differs: {name}"
        );
    }
    Ok(())
}
