//! Authenticate complete offline decompressor evidence before live admission.
use crate::decomp_model::*;
use crate::{digest, family};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Component, Path, PathBuf},
};
fn document<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(toml::from_str(&fs::read_to_string(path)?)?)
}
fn artifact(root: &Path, name: &str) -> Result<PathBuf> {
    ensure!(
        !name.is_empty() && Path::new(name).components().all(|c| matches!(c, Component::Normal(_))),
        "invalid accepted artifact path"
    );
    let mut path = root.to_path_buf();
    for component in Path::new(name).components() {
        path.push(component);
        ensure!(!path.is_symlink(), "accepted artifact is a symlink");
    }
    ensure!(path.is_file(), "accepted artifact missing: {name}");
    Ok(path)
}
pub fn source(checkout: &Path, manifest: &Job) -> Result<std::collections::BTreeMap<String, String>> {
    let identity = family::verify_source(checkout, &checkout.join("deps/snesrev-sm"))?;
    ensure!(
        identity.revision == manifest.source_revision && identity.archive_sha256 == manifest.source_archive_sha256,
        "source archive identity differs from routine manifest"
    );
    let mut selected = std::collections::BTreeMap::new();
    for (name, hash) in &identity.files {
        if name == "LICENSE.txt" || name == "src/sm_80.c" || name == "src/snes/cpu.c" || name.ends_with(".h") {
            selected.insert(name.clone(), hash.clone());
        }
    }
    let bank = fs::read(checkout.join("deps/snesrev-sm/src/sm_80.c"))?;
    let start = bank
        .windows(b"static uint32 decompress_src;".len())
        .position(|w| w == b"static uint32 decompress_src;")
        .context("routine start missing")?;
    let end = start
        + bank[start..]
            .windows(b"static uint8 ReadPpuByte".len())
            .position(|w| w == b"static uint8 ReadPpuByte")
            .context("routine end missing")?;
    selected.insert("routine-and-helper".into(), format!("{:x}", Sha256::digest(&bank[start..end])));
    Ok(selected)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticateRequest {
    pub checkout: PathBuf,
    pub pointer: PathBuf,
}
pub fn authenticate(request: AuthenticateRequest) -> Result<Seal> {
    let checkout = fs::canonicalize(&request.checkout)?;
    let pointer: Pointer = document(&request.pointer)?;
    let run = &pointer.run;
    ensure!(pointer.status == "accepted" && pointer.routine_accepted, "offline pointer is not accepted");
    let verdict_path = artifact(run, "verdict.toml")?;
    ensure!(digest(&verdict_path)? == pointer.verdict_sha256, "accepted verdict identity differs");
    let verdict: Verdict = document(&verdict_path)?;
    let seal_path = artifact(run, "acceptance.toml")?;
    ensure!(
        verdict.status == "accepted"
            && verdict.routine_accepted
            && digest(&seal_path)? == verdict.acceptance_sha256.as_deref().context("acceptance digest missing")?,
        "accepted seal identity differs"
    );
    let seal: Seal = document(&seal_path)?;
    ensure!(
        seal.artifact_inventory_schema == "decomp-artifact-inventory-v2"
            && seal.contract.schema == "decomp-offline-acceptance-contract-v2"
            && seal.status == "accepted"
            && seal.routine_accepted,
        "unsupported accepted seal"
    );
    ensure!(
        seal.fixtures == verdict.fixtures
            && seal.observed_cases == verdict.observed_cases
            && seal.negative_controls == verdict.negative_controls
            && Some(&seal.original_rom_parity) == verdict.original_rom_parity.as_ref()
            && seal.implementation == verdict.implementation_identities
            && seal.source == verdict.source_identities
            && seal.binaries == verdict.binaries
            && seal.job_sha256 == verdict.manifest_sha256,
        "accepted verdict/seal evidence differs"
    );
    ensure!(
        Some(&seal.capture_manifest_sha256) == verdict.captures_manifest_sha256.as_ref()
            && digest(&artifact(run, "captures.toml")?)? == seal.capture_manifest_sha256
            && document::<Contract>(&artifact(run, "acceptance-contract.toml")?)? == seal.contract,
        "accepted capture/contract evidence differs"
    );
    let artifacts = &seal.artifacts;
    ensure!(!artifacts.is_empty(), "accepted artifact inventory is empty");
    for (name, hash) in artifacts {
        ensure!(digest(&artifact(run, name)?)? == *hash, "accepted artifact changed: {name}");
    }
    ensure!(
        digest(&checkout.join("tools/decomp/offline-acceptance-v1.toml"))? == seal.contract_sha256
            && digest(&checkout.join("tools/decomp/decompress-job.toml"))? == seal.job_sha256,
        "accepted contract/job changed"
    );
    for (name, hash) in &seal.implementation {
        ensure!(digest(&family::source_path(&checkout, name))? == *hash, "accepted implementation changed: {name}");
    }
    for (name, hash) in &seal.binaries {
        ensure!(digest(&artifact(run, name)?)? == *hash, "accepted binary changed: {name}");
    }
    ensure!(
        source(&checkout, &document(&artifact(run, "job.toml")?)?)? == seal.source,
        "accepted upstream source changed"
    );
    for fixture in &seal.fixtures {
        let id = &fixture.id;
        for (file, hash) in [
            ("fixture.toml", &fixture.fixture_sha256),
            ("rom.bin", &fixture.rom_sha256),
            ("before.wram", &fixture.initial_sha256),
            ("expected.wram", &fixture.expected_sha256),
            ("c_trace.wram.writes", &fixture.c_ordered_writes_sha256),
            ("rust.wram.writes", &fixture.rust_ordered_writes_sha256),
        ] {
            ensure!(
                digest(&artifact(run, &format!("{id}/{file}"))?)? == *hash,
                "accepted fixture changed: {id}/{file}"
            );
        }
    }
    for control in &seal.negative_controls {
        let relative = control.run.strip_prefix(&seal.run)?;
        ensure!(
            control.rejected
                && digest(&artifact(run, &relative.join("verdict.toml").to_string_lossy())?)? == control.verdict_sha256,
            "accepted negative evidence changed"
        );
    }
    Ok(seal)
}
