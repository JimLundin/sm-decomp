//! Atomic production ownership switch. Migration packages, original claims and
//! ledgers stay intact; one immutable registry retires every imported effort.
use crate::{
    assignment, emit,
    inputs::bounded,
    translation_execution::{Admission, Installation},
    translation_launch::Plan,
    translation_migration::{self as migration, Migration, MigrationExecution},
};
use ::translation_execution::{
    ClaimLineage,
    model::{digest_valid, sealed},
    retention::{self, Reference},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

pub(crate) const REGISTRY: &str = ".scratch/translation-execution/production.json";
const AUTHORITY: &str = ".scratch/decomp-parallel/authority.toml";
const LEGACY_FENCE: &str = ".scratch/decomp-parallel/deterministic-execution.json";
const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_DEPLOYMENTS: usize = 32;

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: String,
    registry: Registry,
    original_snapshots: Vec<Reference>,
    preparation_reports: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Activation {
    schema: String,
    registry: Reference,
    legacy_fence: Reference,
    intent: Reference,
}

/// Historical binaries reject this schema before reaching an unrestricted
/// producer. Publishing it before the registry also fences interrupted switches.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct LegacyFence {
    schema: String,
    checkout: PathBuf,
    request: Reference,
    registry: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Report {
    pub schema: String,
    pub state: String,
    pub request: Reference,
    pub registry: Option<Reference>,
    pub activation: Option<Reference>,
    pub launch_plan: Reference,
    pub runnable_scopes: Vec<String>,
    pub retired_efforts: Vec<String>,
    pub held_integration_claims: Vec<ClaimLineage>,
    pub blocked: Vec<BlockedClaim>,
    pub note: String,
}

#[derive(Deserialize)]
struct Authority {
    schema: String,
    state: String,
    preparation: assignment::authority::Preparation,
}

#[derive(Deserialize)]
struct Ledger {
    schema: String,
    jobs: BTreeMap<String, assignment::Entry>,
}

fn protected(path: &Path) -> Result<()> {
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

fn object<T: DeserializeOwned>(path: &Path) -> Result<T> {
    protected(path)?;
    let bytes = bounded::read_limit(path, MAX_RECORD)?;
    if path.extension().is_some_and(|extension| extension == "toml") {
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    } else {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn authenticate(reference: &Reference) -> Result<()> {
    ensure!(digest_valid(&reference.sha256) && reference.bytes <= MAX_RECORD, "production reference extent invalid");
    protected(&reference.path)?;
    retention::authenticate_reference(reference, MAX_RECORD)
}

fn name(value: &str) -> bool {
    !value.is_empty() && value.len() <= 160 && value.bytes().all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
}

fn registry_for(request: &Request, reference: &Reference) -> Registry {
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

fn fence_for(registry: &Registry) -> Result<LegacyFence> {
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

fn request_shape(request: &Request) -> Result<()> {
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

fn migrations(request: &Request) -> Result<Vec<Migration>> {
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
    bounded::limited(|| {
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

fn blocked(request: &Request, effort: &str, claim: &ClaimLineage) -> bool {
    request.blocked.iter().any(|blocked| blocked.effort == effort && &blocked.claim == claim)
}

fn plan_installations(request: &Request, active: &[Migration]) -> Result<Plan> {
    authenticate(&request.launch_plan)?;
    let plan: Plan = object(&request.launch_plan.path)?;
    let installations = plan.validate(&request.launch_plan.path)?;
    let metadata_root = request.checkout.join(".scratch/translation-execution");
    ensure!(
        !metadata_root.starts_with(&plan.state_root) && !plan.state_root.starts_with(&metadata_root),
        "production ownership metadata overlaps writable launch state"
    );
    let planned = plan.scopes.iter().map(|scope| &scope.installation).collect::<Vec<_>>();
    ensure!(
        planned == request.installations.iter().collect::<Vec<_>>(),
        "production installation inventory differs from exact launch order"
    );
    let mut expected = BTreeMap::new();
    for migration in active {
        for claim in &migration.claims {
            if matches!(claim.execution, Some(MigrationExecution::HeldIntegration { .. }))
                || blocked(request, &migration.effort, &claim.claim)
            {
                continue;
            }
            let path = claim.installation.as_ref().context("unblocked original claim lacks an installation")?;
            ensure!(expected.insert(path.clone(), (migration, claim)).is_none(), "two claims share an installation");
        }
    }
    ensure!(expected.len() == installations.len(), "launch omits an unblocked original executor");
    for (scope, installed) in plan.scopes.iter().zip(installations) {
        for writable in [&installed.state_root, &installed.pipeline_root] {
            ensure!(
                !metadata_root.starts_with(writable) && !writable.starts_with(&metadata_root),
                "production ownership metadata overlaps writable supervisor state"
            );
            for input in request
                .migrations
                .iter()
                .flat_map(|deployment| [&deployment.prepared, &deployment.active])
                .chain(request.installations.iter())
                .chain(std::iter::once(&request.launch_plan))
            {
                ensure!(!input.path.starts_with(writable), "production input lies in writable supervisor state");
            }
        }
        let (migration, claim) =
            expected.remove(&scope.installation.path).context("launch installs an undeclared or blocked claim")?;
        let Admission::Managed { checkout, effort, coordinator, session_id, generation, migration: asset } =
            &installed.admission
        else {
            anyhow::bail!("production launch contains an isolated proof installation")
        };
        let deployment = request
            .migrations
            .iter()
            .find(|deployment| deployment.active.path == asset.file)
            .context("installation lacks exact active migration member")?;
        ensure!(
            checkout == &request.checkout
                && effort == &migration.effort
                && coordinator == &migration.coordinator
                && session_id == &migration.session_id
                && *generation == migration.generation
                && asset.sha256 == deployment.active.sha256
                && installed.policy.claim == claim.claim
                && sealed(&installed.policy)? == claim.policy_sha256
                && installed.policy.scope_id == claim.scope_id,
            "production installation changed original claim, policy or authority"
        );
    }
    Ok(plan)
}

fn preserved_inputs(request: &Request, active: &[Migration]) -> Vec<Reference> {
    std::iter::once(request.authority.clone()).chain(active.iter().map(|migration| migration.ledger.clone())).collect()
}

fn validate_current_ownership(request: &Request, active: &[Migration]) -> Result<()> {
    authenticate(&request.authority)?;
    let authority: Authority = object(&request.authority.path)?;
    ensure!(
        authority.schema == "decomp-session-authority-v1" && authority.state == "active",
        "production authority is not active"
    );
    ensure!(
        assignment::main_checkout(&request.checkout)? == request.checkout,
        "production needs the canonical main checkout"
    );
    let production = std::iter::once(authority.preparation.canonical_effort)
        .chain(authority.preparation.admitted_efforts)
        .collect::<BTreeSet<_>>();
    let deployed = active.iter().map(|migration| migration.effort.clone()).collect::<BTreeSet<_>>();
    ensure!(deployed.is_subset(&production), "production includes an effort outside current authority");
    for effort in production {
        ensure!(!authority.preparation.drill_efforts.contains(&effort), "production includes a drill effort");
        let path = request.checkout.join(format!(".scratch/{effort}/ledger.toml"));
        if !path.try_exists()? {
            continue;
        }
        let ledger: Ledger = object(&path)?;
        ensure!(ledger.schema == "decomp-assignment-v1", "production ledger schema invalid");
        let mut held = BTreeMap::new();
        for (id, entry) in ledger.jobs {
            ensure!(id == entry.job.id, "production ledger job key differs");
            if matches!(
                entry.state,
                assignment::State::Reserved | assignment::State::Running | assignment::State::Submitted
            ) {
                let claim = entry.claim.context("held production job lacks its original claim")?;
                held.insert(
                    id,
                    ClaimLineage {
                        id: claim.id,
                        revision: claim.revision,
                        attempt: claim.attempt,
                        coordinator: claim.coordinator,
                    },
                );
            }
        }
        if held.is_empty() {
            continue;
        }
        let migration = active
            .iter()
            .find(|migration| migration.effort == effort)
            .context("production switch omits an effort with held claims")?;
        ensure!(migration.ledger.path == path, "production migration names another effort ledger");
        authenticate(&migration.ledger)?;
        ensure!(
            held.len() == migration.claims.len()
                && migration.claims.iter().all(|claim| held.get(&claim.claim.id) == Some(&claim.claim)),
            "production switch omits or replaces original held ownership"
        );
        assignment::authority::validate(
            &request.checkout,
            &migration.effort,
            &migration.coordinator,
            Some(&assignment::SessionAuthority {
                session_id: migration.session_id.clone(),
                generation: migration.generation,
            }),
            true,
        )?;
    }
    for reference in preserved_inputs(request, active) {
        authenticate(&reference)?;
    }
    Ok(())
}

fn validate_preparation(request: &Request, active: &[Migration]) -> Result<Vec<migration::PreparationReport>> {
    validate_current_ownership(request, active)?;
    plan_installations(request, active)?;
    let mut reports = Vec::new();
    for deployment in &request.migrations {
        let report = migration::validate_preparation(&deployment.prepared.path)?;
        for claim in &report.claims {
            ensure!(
                claim.exact_original_lineage
                    && claim.stopped_owner_observed
                    && (claim.production_policy_prepared || blocked(request, &report.effort, &claim.claim)),
                "unblocked production claim lacks checked preparation: {}: {}",
                claim.claim.id,
                claim.blockers.join("; ")
            );
        }
        reports.push(report);
    }
    // The representative proof's unrelated catalog gaps are retained in these
    // reports. Each executor must independently pass its own complete contract
    // preparation; a named blocked claim is fenced without asserting readiness.
    validate_current_ownership(request, active)?;
    Ok(reports)
}

fn directory(path: &Path) -> Result<()> {
    bounded::safe(path)?;
    fs::DirBuilder::new().recursive(true).mode(0o700).create(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "production output directory is foreign or writable by others"
    );
    Ok(())
}

/// A surviving prefix belongs to this exact record. Foreign partial writes are
/// rejected, and a complete identical retry never changes final file metadata.
fn write_once(path: &Path, bytes: &[u8]) -> Result<()> {
    ensure!(bytes.len() as u64 <= MAX_RECORD, "production metadata extent exceeded");
    bounded::safe(path)?;
    let parent = path.parent().context("production record parent missing")?;
    directory(parent)?;
    let pending = path.with_extension("pending");
    if path.try_exists()? {
        protected(path)?;
        ensure!(bounded::read_limit(path, MAX_RECORD)? == bytes, "production record changed on retry");
        ensure!(!pending.try_exists()?, "production final record has a foreign pending sibling");
        return Ok(());
    }
    let offset = if pending.try_exists()? {
        protected(&pending)?;
        let prefix = bounded::read_limit(&pending, MAX_RECORD)?;
        ensure!(bytes.starts_with(&prefix), "production pending record is not this exact intent");
        prefix.len()
    } else {
        0
    };
    let mut options = OpenOptions::new();
    options.write(true).append(true).mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    if pending.try_exists()? {
        options.create(false);
    } else {
        options.create_new(true);
    }
    let mut file = options.open(&pending)?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file() && metadata.nlink() == 1 && metadata.len() == offset as u64,
        "production pending record changed while opening"
    );
    file.write_all(&bytes[offset..])?;
    file.sync_all()?;
    fs::rename(&pending, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn write<T: Serialize>(path: &Path, value: &T) -> Result<Reference> {
    write_once(path, &serde_json::to_vec(value)?)?;
    retention::seal_file(path, MAX_RECORD)
}

fn transaction(root: &Path, request: &Reference) -> PathBuf {
    root.join(".scratch/translation-execution/activations").join(&request.sha256)
}

fn report(request: &Request, reference: &Reference, active: &[Migration], state: &str) -> Result<Report> {
    let plan: Plan = object(&request.launch_plan.path)?;
    let registry_path = request.checkout.join(REGISTRY);
    let receipt_path = transaction(&request.checkout, reference).join("activation.json");
    Ok(Report {
        schema: "translation-production-report-v1".into(),
        state: state.into(),
        request: reference.clone(),
        registry: registry_path.try_exists()?.then(|| retention::seal_file(&registry_path, MAX_RECORD)).transpose()?,
        activation: receipt_path.try_exists()?.then(|| retention::seal_file(&receipt_path, MAX_RECORD)).transpose()?,
        launch_plan: request.launch_plan.clone(),
        runnable_scopes: plan.scopes.into_iter().map(|scope| scope.id).collect(),
        retired_efforts: active.iter().map(|migration| migration.effort.clone()).collect(),
        held_integration_claims: active.iter().flat_map(|migration| &migration.claims).filter(|claim| matches!(claim.execution, Some(MigrationExecution::HeldIntegration { .. }))).map(|claim| claim.claim.clone()).collect(),
        blocked: request.blocked.clone(),
        note: "Ownership activation preserves original claims and acceptance requirements. A launched supervisor may stop at an unmet frozen prerequisite; activation asserts no translation, actual-C verification, Live replay or ROM parity.".into(),
    })
}

fn finish_activation(root: &Path, reference: &Reference, expected: &Registry) -> Result<()> {
    let output = transaction(root, reference);
    let intent_path = output.join("intent.json");
    let intent: Intent = object(&intent_path)?;
    ensure!(
        intent.schema == "translation-production-intent-v1" && intent.registry == *expected,
        "production activation intent changed"
    );
    for artifact in intent.original_snapshots.iter().chain(&intent.preparation_reports) {
        authenticate(artifact)?;
    }
    let legacy_fence = write(&root.join(LEGACY_FENCE), &fence_for(expected)?)?;
    write(&root.join(REGISTRY), expected)?;
    let receipt = Activation {
        schema: "translation-production-activation-v1".into(),
        registry: retention::seal_file(&root.join(REGISTRY), MAX_RECORD)?,
        legacy_fence,
        intent: retention::seal_file(&intent_path, MAX_RECORD)?,
    };
    write(&output.join("activation.json"), &receipt)?;
    Ok(())
}

pub(crate) fn operate(path: &Path, action: &str) -> Result<Report> {
    let deadline = bounded::now_unix_ms()?.checked_add(600_000).context("production activation deadline overflow")?;
    bounded::until(deadline, 8 * bounded::DEFAULT_BYTES, || operate_bounded(path, action))
}

fn operate_bounded(path: &Path, action: &str) -> Result<Report> {
    ensure!(matches!(action, "status" | "activate" | "recover"), "unknown production activation action");
    let reference = retention::seal_file(path, MAX_RECORD)?;
    authenticate(&reference)?;
    let request: Request = object(path)?;
    let active = migrations(&request)?;
    let expected = registry_for(&request, &reference);
    if let Some(registry) = read_registry(&request.checkout)? {
        ensure!(registry == expected, "production already activated a different checked request");
        if action != "status" {
            let _lock = assignment::mutation_lock(&request.checkout)?;
            ensure!(
                read_registry(&request.checkout)? == Some(expected.clone()),
                "production registry changed while recovering"
            );
            finish_activation(&request.checkout, &reference, &expected)?;
        }
        return report(&request, &reference, &active, "active");
    }
    if action == "status" {
        validate_preparation(&request, &active)?;
        return report(&request, &reference, &active, "ready");
    }
    // Expensive proof reads precede the shared lock; all authority/ledger seals
    // and request inputs are rechecked after acquiring it and before commit.
    let reports = validate_preparation(&request, &active)?;
    let _lock = assignment::mutation_lock(&request.checkout)?;
    if let Some(registry) = read_registry(&request.checkout)? {
        ensure!(registry == expected, "another production switch won the mutation lock");
        finish_activation(&request.checkout, &reference, &expected)?;
        return report(&request, &reference, &active, "active");
    }
    authenticate(&reference)?;
    migrations(&request)?;
    validate_current_ownership(&request, &active)?;
    plan_installations(&request, &active)?;
    let old_fence = request.checkout.join(LEGACY_FENCE);
    if old_fence.try_exists()? {
        let previous: LegacyFence =
            object(&old_fence).context("production will not replace an existing single-effort migration fence")?;
        ensure!(previous == fence_for(&expected)?, "production fence belongs to a different activation request");
    }
    let output = transaction(&request.checkout, &reference);
    let mut original_snapshots = Vec::new();
    for (index, input) in preserved_inputs(&request, &active).iter().enumerate() {
        authenticate(input)?;
        let snapshot = output.join(format!("original-{index:02}.toml"));
        write_once(&snapshot, &bounded::read_limit(&input.path, MAX_RECORD)?)?;
        let artifact = retention::seal_file(&snapshot, MAX_RECORD)?;
        ensure!(
            artifact.sha256 == input.sha256 && artifact.bytes == input.bytes,
            "production original snapshot differs"
        );
        original_snapshots.push(artifact);
    }
    let preparation_reports = reports
        .iter()
        .enumerate()
        .map(|(index, value)| write(&output.join(format!("preparation-{index:02}.json")), value))
        .collect::<Result<Vec<_>>>()?;
    write(
        &output.join("intent.json"),
        &Intent {
            schema: "translation-production-intent-v1".into(),
            registry: expected.clone(),
            original_snapshots,
            preparation_reports,
        },
    )?;
    validate_current_ownership(&request, &active)?;
    finish_activation(&request.checkout, &reference, &expected)?;
    report(&request, &reference, &active, "active")
}

pub(crate) fn cli(args: &[String]) -> Result<()> {
    ensure!(
        args.len() == 3 && matches!(args[0].as_str(), "--request" | "--checkout"),
        "usage: translation-production --request REQUEST.json status|activate|recover|run; translation-production --checkout CHECKOUT status|recover|run"
    );
    let request = if args[0] == "--checkout" {
        ensure!(matches!(args[2].as_str(), "status" | "recover" | "run"), "activation requires an exact --request");
        let root = Path::new(&args[1]);
        bounded::safe(root)?;
        if crate::translation_continuation::launch(root, &args[2])? {
            return Ok(());
        }
        read_registry(root)?
            .context("production registry is absent; use --request to activate or recover")?
            .request
            .path
    } else {
        PathBuf::from(&args[1])
    };
    if args[2] == "run" {
        return run(&request);
    }
    emit(&operate(&request, &args[2])?)
}

/// Human launch needs only the provider credential. The installed controller
/// receives the exact checked role identity in its child environment; inherited
/// interactive agent state is neither required nor modified.
fn run(path: &Path) -> Result<()> {
    let reference = retention::seal_file(path, MAX_RECORD)?;
    let request: Request = object(path)?;
    let registry = read_registry(&request.checkout)?.context("production is not activated; run activate first")?;
    ensure!(registry == registry_for(&request, &reference), "production launch request differs from active registry");
    authenticate(&request.authority)?;
    let active = migrations(&request)?;
    let sessions = active.iter().map(|migration| migration.session_id.as_str()).collect::<BTreeSet<_>>();
    ensure!(sessions.len() == 1, "production launch requires one checked controller session");
    plan_installations(&request, &active)?;
    let installed: Installation = object(&request.installations[0].path)?;
    installed.validate()?;
    let status = std::process::Command::new(&installed.controller.path)
        .args(["translation-launch", "--plan"])
        .arg(&request.launch_plan.path)
        .arg("run")
        .env("CODEX_THREAD_ID", sessions.iter().next().unwrap())
        .current_dir(&request.checkout)
        .status()?;
    ensure!(status.success(), "installed production launch exited with {status}");
    Ok(())
}

#[cfg(test)]
#[path = "translation_production_tests.rs"]
mod tests;
