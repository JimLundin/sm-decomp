//! Shared migration ownership validation for every legacy producer entrypoint.
//! This guard reads sealed controller metadata and never launches verification.
use crate::{assignment, inputs::bounded};
use ::translation_execution::{
    ClaimLineage,
    model::digest_valid,
    retention::{self, Reference},
};
use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    os::unix::fs::MetadataExt,
    path::{Component, Path, PathBuf},
};
const FENCE: &str = ".scratch/decomp-parallel/deterministic-execution.json";
const MAX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_REFERENCES: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MigratedClaim {
    pub claim: ClaimLineage,
    pub scope_id: String,
    pub policy_sha256: String,
    pub source_commit: String,
    pub writable_files: Vec<PathBuf>,
    /// Typed immutable OriginalClaim, not a caller's nominal receipt hash.
    pub original_claim: Reference,
    /// Prepared installed configuration for this exact original claim. A path
    /// avoids a Migration -> Installation -> Migration digest cycle; validation
    /// seals and rechecks its actual bytes in the preparation reference closure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Migration {
    pub schema: String,
    pub state: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub coordinator: String,
    pub session_id: String,
    pub generation: u64,
    /// Exact current canonical authority and original effort ledger.
    pub authority: Reference,
    pub ledger: Reference,
    pub claims: Vec<MigratedClaim>,
    pub historical: Vec<Reference>,
    /// Typed OriginalOwnerStop observations. These preserve original claims;
    /// they do not perform an owner-stopped queue update or create a retry.
    pub stopped_owners: Vec<Reference>,
    /// Typed RetainedProof whose actual evidence is authenticated read-only.
    pub proof: Reference,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalClaim {
    pub schema: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub job: assignment::Job,
    pub claim: assignment::Claim,
    pub state: assignment::State,
    pub agent: Option<String>,
    pub result: Option<assignment::WorkResult>,
    pub observations: Vec<assignment::Artifact>,
    pub source_checkout: PathBuf,
    pub source_commit: String,
    pub source_files: BTreeMap<String, String>,
}

fn safe(path: &Path) -> Result<()> {
    bounded::check()?;
    ensure!(path.is_absolute(), "migration paths must be absolute");
    ensure!(path.components().collect::<PathBuf>().as_os_str() == path.as_os_str(), "migration path is not canonical");
    let mut prefix = PathBuf::new();
    for component in path.components() {
        bounded::check()?;
        ensure!(matches!(component, Component::RootDir | Component::Normal(_)), "migration path alias");
        prefix.push(component);
        ensure!(!prefix.symlink_metadata()?.file_type().is_symlink(), "migration path crosses a symlink");
    }
    Ok(())
}

fn object<T: DeserializeOwned>(path: &Path) -> Result<T> {
    safe(path)?;
    let bytes = bounded::read_limit(path, MAX_BYTES)?;
    if path.extension().is_some_and(|extension| extension == "toml") {
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    } else {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn relative(path: &Path) -> bool {
    let spelling = path.as_os_str().as_encoded_bytes();
    !spelling.is_empty()
        && !spelling.contains(&0)
        && !spelling.contains(&b'\\')
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|value| value.is_ascii_alphanumeric() || matches!(value, b'-' | b'_'))
}

fn exact_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|value| value.is_ascii_hexdigit())
}

fn same_claim(left: &ClaimLineage, right: &assignment::Claim) -> bool {
    left.id == right.id
        && left.revision == right.revision
        && left.attempt == right.attempt
        && left.coordinator == right.coordinator
}

pub(crate) fn read(path: &Path) -> Result<Migration> {
    bounded::limited(|| read_inner(path))
}

fn read_inner(path: &Path) -> Result<Migration> {
    safe(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.uid() == unsafe { libc::geteuid() }
            && metadata.mode() & 0o022 == 0,
        "migration must be controller-owned, regular and unaliased"
    );
    let migration: Migration = object(path)?;
    ensure!(
        migration.schema == "translation-execution-migration-v1"
            && matches!(migration.state.as_str(), "prepared" | "active")
            && name(&migration.effort)
            && name(&migration.coordinator)
            && migration.generation > 0
            && assignment::authority::real_session_id(&migration.session_id)
            && !migration.claims.is_empty()
            && migration.claims.len() <= 128
            && !migration.limitations.is_empty()
            && migration.limitations.len() <= 32
            && migration.limitations.iter().all(|text| !text.trim().is_empty() && text.len() <= 4096),
        "migration identity, state or limitations are incomplete"
    );
    safe(&migration.checkout)?;
    let mut claims = BTreeSet::new();
    for claim in &migration.claims {
        bounded::check()?;
        ensure!(
            name(&claim.claim.id)
                && !claim.claim.revision.is_empty()
                && claim.claim.revision.len() <= 4096
                && claim.claim.attempt > 0
                && name(&claim.claim.coordinator)
                && !claim.scope_id.trim().is_empty()
                && claim.scope_id.len() <= 512
                && digest_valid(&claim.policy_sha256)
                && exact_commit(&claim.source_commit)
                && !claim.writable_files.is_empty()
                && claim.writable_files.len() <= 128
                && claim.writable_files.iter().all(|path| relative(path))
                && claim.writable_files.iter().collect::<BTreeSet<_>>().len() == claim.writable_files.len()
                && claims.insert((
                    &claim.claim.id,
                    &claim.claim.revision,
                    claim.claim.attempt,
                    &claim.claim.coordinator
                )),
            "migration claim is duplicated or incomplete"
        );
        if let Some(installation) = &claim.installation {
            safe(installation)?;
        }
    }
    for left in &migration.claims {
        for right in &migration.claims {
            bounded::check()?;
            if left.claim == right.claim {
                continue;
            }
            let disjoint = left
                .writable_files
                .iter()
                .all(|a| right.writable_files.iter().all(|b| !a.starts_with(b) && !b.starts_with(a)));
            ensure!(
                disjoint || (migration.state == "prepared" && prepared_linked_pair(&migration, left, right)?),
                "migration claims overlap writable ownership outside an exact prepared submission/integration pair"
            );
        }
    }
    ensure!(
        migration.historical.len() + migration.stopped_owners.len() + migration.claims.len() + 3 <= MAX_REFERENCES,
        "migration retained reference count excessive"
    );
    Ok(migration)
}

/// Preparation records the existing standard queue lineage without granting
/// either held claim new writable execution. The historical legacy derivation
/// omitted base-head; accept only that exact older standard representation.
fn prepared_linked_pair(migration: &Migration, left: &MigratedClaim, right: &MigratedClaim) -> Result<bool> {
    retention::authenticate_reference(&left.original_claim, MAX_BYTES)?;
    retention::authenticate_reference(&right.original_claim, MAX_BYTES)?;
    let left_original: OriginalClaim = object(&left.original_claim.path)?;
    let right_original: OriginalClaim = object(&right.original_claim.path)?;
    let (submitted, integration, submitted_migrated, integrated_migrated) =
        if left_original.job.kind == assignment::Kind::Implement {
            (&left_original, &right_original, left, right)
        } else {
            (&right_original, &left_original, right, left)
        };
    let Some(assignment::WorkResult::Submitted { commit, .. }) = &submitted.result else {
        return Ok(false);
    };
    if submitted.state != assignment::State::Submitted
        || integration.job.kind != assignment::Kind::Integrate
        || !matches!(integration.state, assignment::State::Reserved | assignment::State::Running)
        || integration.result.is_some()
        || submitted.checkout != migration.checkout
        || integration.checkout != migration.checkout
        || submitted.effort != migration.effort
        || integration.effort != migration.effort
        || !same_claim(&submitted_migrated.claim, &submitted.claim)
        || !same_claim(&integrated_migrated.claim, &integration.claim)
        || submitted.source_commit != *commit
        || submitted_migrated.source_commit != submitted.source_commit
        || integrated_migrated.source_commit != integration.source_commit
        || submitted.claim.coordinator != integration.claim.coordinator
        || submitted_migrated.writable_files != submitted.job.writable_paths
        || integrated_migrated.writable_files != integration.job.writable_paths
    {
        return Ok(false);
    }
    let expected = assignment::admitted_integration_job(&submitted.job, &submitted.claim, commit)?;
    let mut historical = expected.clone();
    historical.inputs.remove("base-head");
    Ok((integration.job == expected || integration.job == historical)
        && integration.claim.id == expected.id
        && integration.claim.revision == expected.revision)
}

/// Held under assignment's shared mutation lock before any maintained queue
/// mutation. Plan/status remain available for historical inspection.
pub(crate) fn fence_legacy_locked(root: &Path, request: &assignment::Request) -> Result<()> {
    if matches!(request.operation.as_str(), "status" | "plan") {
        return Ok(());
    }
    fence_effort(root, &request.effort)
}

/// Session/coordination producer entrypoints check under the same shared lock,
/// before source commits, result gates, resumable commands or publication.
pub(crate) fn fence_session_locked(root: &Path, request: &assignment::Request) -> Result<()> {
    fence_effort(root, &request.effort)
}

fn fence_effort(root: &Path, effort: &str) -> Result<()> {
    bounded::limited(|| fence_effort_inner(root, effort))
}

fn fence_effort_inner(root: &Path, effort: &str) -> Result<()> {
    let path = root.join(FENCE);
    match path.symlink_metadata() {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
        Ok(_) => {}
    }
    let migration = read(&path)?;
    ensure!(migration.checkout == root, "installed migration fence names another checkout");
    if migration.state == "active" && migration.effort == effort {
        bail!(
            "unrestricted dispatch and evidence producers are retired for the migrated effort; use the installed translation supervisor"
        )
    }
    Ok(())
}
