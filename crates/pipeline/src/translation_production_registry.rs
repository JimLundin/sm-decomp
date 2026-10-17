//! Immutable production ownership metadata shared by queue admission and the
//! production command. Reading this module never launches producers.
use crate::translation_migration_guard::Migration;
use ::translation_execution::{
    ClaimLineage,
    model::{digest_valid, sealed},
    retention::{self, Reference},
};
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::BTreeSet,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};
use tool_execution::bounded;

pub(crate) const REGISTRY: &str = ".scratch/translation-execution/production.json";
pub(crate) const AUTHORITY: &str = ".scratch/decomp-parallel/authority.toml";
pub(crate) const LEGACY_FENCE: &str = ".scratch/decomp-parallel/deterministic-execution.json";
pub(crate) const MAX_RECORD: u64 = 4 * 1024 * 1024;
pub(crate) const MAX_DEPLOYMENTS: usize = 32;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Deployment {
    pub prepared: Reference,
    pub active: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BlockedClaim {
    pub effort: String,
    pub claim: ClaimLineage,
    pub original_claim: Reference,
    pub reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Request {
    pub schema: String,
    pub checkout: PathBuf,
    pub authority: Reference,
    pub migrations: Vec<Deployment>,
    pub installations: Vec<Reference>,
    pub launch_plan: Reference,
    pub blocked: Vec<BlockedClaim>,
}

/// Presence is the single commit point. Managed admission checks an exact
/// migration member; legacy dispatch is fenced for every member's effort.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct Registry {
    pub schema: String,
    pub checkout: PathBuf,
    pub request: Reference,
    pub authority: Reference,
    pub migrations: Vec<Reference>,
    pub launch_plan: Reference,
    pub blocked: Vec<BlockedClaim>,
}

/// Historical binaries reject this schema before reaching an unrestricted
/// producer. Publishing it before the registry also fences interrupted switches.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct LegacyFence {
    schema: String,
    checkout: PathBuf,
    request: Reference,
    registry: Reference,
}

pub(crate) fn protected(path: &Path) -> Result<()> {
    bounded::safe(path)?;
    ensure!(path.components().collect::<PathBuf>().as_os_str() == path.as_os_str(), "production path alias");
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.uid() == unsafe { libc::geteuid() }
            && metadata.mode() & 0o022 == 0,
        "production record must be controller-owned, regular and unaliased"
    );
    Ok(())
}

pub(crate) fn object<T: DeserializeOwned>(path: &Path) -> Result<T> {
    protected(path)?;
    let bytes = bounded::read_limit(path, MAX_RECORD)?;
    if path.extension().is_some_and(|extension| extension == "toml") {
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    } else {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

pub(crate) fn authenticate(reference: &Reference) -> Result<()> {
    ensure!(digest_valid(&reference.sha256) && reference.bytes <= MAX_RECORD, "production reference extent invalid");
    protected(&reference.path)?;
    retention::authenticate_reference(reference, MAX_RECORD)
}

fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 160 && value.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
}

pub(crate) fn registry_for(request: &Request, reference: &Reference) -> Registry {
    Registry {
        schema: "translation-production-registry-v1".into(),
        checkout: request.checkout.clone(),
        request: reference.clone(),
        authority: request.authority.clone(),
        migrations: request.migrations.iter().map(|deployment| deployment.active.clone()).collect(),
        launch_plan: request.launch_plan.clone(),
        blocked: request.blocked.clone(),
    }
}

pub(crate) fn fence_for(registry: &Registry) -> Result<LegacyFence> {
    let bytes = serde_json::to_vec(registry)?;
    Ok(LegacyFence {
        schema: "translation-production-fence-v1".into(),
        checkout: registry.checkout.clone(),
        request: registry.request.clone(),
        registry: Reference {
            path: registry.checkout.join(REGISTRY),
            sha256: ::translation_execution::model::sha256(&bytes),
            bytes: bytes.len() as u64,
        },
    })
}

pub(crate) fn request_shape(request: &Request) -> Result<()> {
    bounded::safe(&request.checkout)?;
    ensure!(
        request.schema == "translation-production-request-v1"
            && !request.migrations.is_empty()
            && request.migrations.len() <= MAX_DEPLOYMENTS
            && !request.installations.is_empty()
            && request.installations.len() <= 32
            && request.blocked.len() <= 128 * MAX_DEPLOYMENTS
            && request.authority.path == request.checkout.join(AUTHORITY),
        "production request schema, authority or inventory invalid"
    );
    let mut blocked = BTreeSet::new();
    for claim in &request.blocked {
        ensure!(
            name(&claim.effort)
                && !claim.reason.trim().is_empty()
                && claim.reason.len() <= 4096
                && blocked.insert((claim.effort.clone(), sealed(&claim.claim)?)),
            "production blocked claim is duplicated or lacks a reason"
        );
    }
    Ok(())
}

pub(crate) fn migrations(request: &Request) -> Result<Vec<Migration>> {
    request_shape(request)?;
    let mut efforts = BTreeSet::new();
    let mut result = Vec::new();
    for deployment in &request.migrations {
        authenticate(&deployment.prepared)?;
        authenticate(&deployment.active)?;
        ensure!(deployment.prepared.path != deployment.active.path, "production replaced its prepared migration");
        let prepared = crate::translation_migration_guard::read(&deployment.prepared.path)?;
        let active = crate::translation_migration_guard::read(&deployment.active.path)?;
        ensure!(
            prepared.state == "prepared"
                && active.state == "active"
                && prepared.checkout == request.checkout
                && active.authority == request.authority
                && efforts.insert(active.effort.clone()),
            "production migration state, checkout, authority or effort differs"
        );
        let mut comparison = active.clone();
        comparison.state = "prepared".into();
        ensure!(sealed(&comparison)? == sealed(&prepared)?, "active migration changes its reviewed prepared package");
        result.push(active);
    }
    let mut matched = BTreeSet::new();
    for active in &result {
        for claim in &active.claims {
            if let Some(blocked) =
                request.blocked.iter().find(|blocked| blocked.effort == active.effort && blocked.claim == claim.claim)
            {
                ensure!(blocked.original_claim == claim.original_claim, "blocked claim changed original lineage");
                authenticate(&blocked.original_claim)?;
                matched.insert((blocked.effort.clone(), sealed(&blocked.claim)?));
            }
        }
    }
    ensure!(matched.len() == request.blocked.len(), "blocked claim is absent from imported migrations");
    Ok(result)
}

/// Cheap immutable metadata authentication for every dispatch boundary. Current
/// ledger bytes are deliberately not reread: managed publication advances them.
pub(crate) fn read_registry(root: &Path) -> Result<Option<Registry>> {
    decomp_source::context::limited(|| {
        let path = root.join(REGISTRY);
        match path.symlink_metadata() {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
            Ok(_) => {}
        }
        let registry: Registry = object(&path)?;
        ensure!(registry.checkout == root, "production registry names another checkout");
        let fence: LegacyFence = object(&root.join(LEGACY_FENCE))?;
        ensure!(fence == fence_for(&registry)?, "production legacy dispatch fence is missing or changed");
        authenticate(&fence.registry)?;
        authenticate(&registry.request)?;
        let request: Request = object(&registry.request.path)?;
        ensure!(
            registry == registry_for(&request, &registry.request),
            "production registry changed its checked request"
        );
        migrations(&request)?;
        authenticate(&registry.launch_plan)?;
        for installation in &request.installations {
            authenticate(installation)?;
        }
        Ok(Some(registry))
    })
}
