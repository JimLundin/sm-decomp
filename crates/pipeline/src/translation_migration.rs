//! Read-only migration preparation and refusal of retired producer routes.
//! A retained proof validates its own closed policy; it is never an approval
//! token for another claim or for an unimplemented managed result publisher.
use crate::{
    assignment, batch, digest, emit, progress, translation, translation_execution as supervisor,
    translation_native as native,
};
use ::translation_execution::{
    ClaimLineage, Completion, FrozenPolicy,
    checkout::ControllerProjection,
    model::{digest_valid, sealed},
    operation,
    retention::{self, Reference},
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::OpenOptions,
    io::Read,
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt},
        io::AsRawFd,
        process::CommandExt,
    },
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const FENCE: &str = ".scratch/decomp-parallel/deterministic-execution.json";
const AUTHORITY: &str = ".scratch/decomp-parallel/authority.toml";
const MAX_BYTES: u64 = 4 * 1024 * 1024;
const MAX_REFERENCES: usize = 4096;
const MAX_RETAINED_BYTES: u64 = 1024 * 1024 * 1024;
const MANAGED_METADATA_RESERVATION: u64 = 24 * MAX_BYTES;
// Authentication reads the same complete proof at each trust transition before
// scanning the production catalog. Bound total IO independently of the unique
// retained reference extent and of every producer's installed output budget.
const PREPARATION_IO_BYTES: u64 = 8 * crate::inputs::bounded::DEFAULT_BYTES;
const PREPARATION_TIMEOUT_MS: u64 = 300_000;

#[cfg(test)]
use crate::translation_migration_guard::fence_legacy_locked;
use crate::translation_migration_guard::read;
pub(crate) use crate::translation_migration_guard::{MigratedClaim, Migration, MigrationExecution, OriginalClaim};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OwnerProcess {
    pub identity: ::translation_execution::operation::ProcessIdentity,
    pub status: String,
    pub observation: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OriginalOwnerStop {
    pub schema: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub claim: assignment::Claim,
    pub agent: Option<String>,
    pub observed_at: String,
    pub status: String,
    /// Exact observed real session for a started original worker. Recorded
    /// thread evidence remains an inspection, never an activation credential.
    pub session: Option<Reference>,
    pub evidence: Vec<Reference>,
    pub processes: Vec<OwnerProcess>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RetainedProof {
    pub schema: String,
    pub installation: Reference,
    pub completion: Reference,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ClaimPreparation {
    pub claim: ClaimLineage,
    pub scope_id: String,
    pub source_commit: String,
    pub exact_original_lineage: bool,
    pub stopped_owner_observed: bool,
    pub historical_reference_count: usize,
    pub production_policy_prepared: bool,
    pub blockers: Vec<String>,
}

/// Installation identity binds the semantic migration domain. Actual deployed
/// Migration bytes are checked separately at every managed trust transition.
/// State and derived policy/deployment seals cannot participate in their own
/// identity; every original owner, claim, source and evidence identity does.
pub(crate) fn domain_identity(migration: &Migration) -> Result<String> {
    let claims = migration
        .claims
        .iter()
        .map(|claim| {
            (&claim.claim, &claim.scope_id, &claim.source_commit, &claim.writable_files, &claim.original_claim)
        })
        .collect::<Vec<_>>();
    let historical = sealed(&(
        "translation-managed-domain-v1",
        &migration.schema,
        &migration.checkout,
        &migration.effort,
        &migration.coordinator,
        &migration.session_id,
        migration.generation,
        &migration.authority,
        &migration.ledger,
        claims,
        &migration.historical,
        &migration.stopped_owners,
        &migration.proof,
        &migration.limitations,
    ))?;
    if migration.claims.iter().all(|claim| claim.execution.is_none()) {
        Ok(historical)
    } else {
        sealed(&(historical, migration.claims.iter().map(|claim| (&claim.claim, &claim.execution)).collect::<Vec<_>>()))
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct CatalogCoverage {
    pub catalog_sha256: Option<String>,
    pub families: Vec<String>,
    pub representative_proof_families: Vec<String>,
    pub families_without_matching_prepared_proof: Vec<String>,
    pub structural_confined_recipe_gaps: BTreeMap<String, String>,
    pub note: String,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct PreparationReport {
    pub schema: String,
    pub status: String,
    pub activated: bool,
    pub activation_ready: bool,
    pub managed_launch_available: bool,
    pub managed_publication_available: bool,
    pub migration_sha256: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub current_integrator: String,
    pub current_session_id: String,
    pub current_generation: u64,
    pub claims: Vec<ClaimPreparation>,
    pub representative_proof_policy_sha256: String,
    pub representative_proof_scope: String,
    pub catalog: CatalogCoverage,
    /// The migration's bounded preparation-reference closure, including proof
    /// pointers. The authenticated proof owns its separate retained graph.
    pub retained_references: usize,
    pub retained_bytes: u64,
    pub authenticated_proof_finalization_bytes: u64,
    pub blockers: Vec<String>,
    pub limitations: Vec<String>,
}

#[derive(Deserialize)]
struct AuthorityView {
    schema: String,
    state: String,
    preparation: assignment::authority::Preparation,
    roles: BTreeMap<String, assignment::authority::Role>,
    drill: Option<assignment::Artifact>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LedgerView {
    schema: String,
    capacity: usize,
    #[serde(default)]
    integration_owner: Option<String>,
    jobs: BTreeMap<String, assignment::Entry>,
}

fn safe(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "migration paths must be absolute");
    ensure!(path.components().collect::<PathBuf>().as_os_str() == path.as_os_str(), "migration path is not canonical");
    let mut prefix = PathBuf::new();
    for component in path.components() {
        ensure!(matches!(component, Component::RootDir | Component::Normal(_)), "migration path alias");
        prefix.push(component);
        ensure!(!prefix.symlink_metadata()?.file_type().is_symlink(), "migration path crosses a symlink");
    }
    Ok(())
}

fn bytes(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    safe(path)?;
    crate::inputs::bounded::read_limit(path, maximum)
}

fn object<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = bytes(path, MAX_BYTES)?;
    if path.extension().is_some_and(|extension| extension == "toml") {
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    } else {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn json_record<T: DeserializeOwned>(path: &Path) -> Result<T> {
    Ok(serde_json::from_slice(&bytes(path, MAX_BYTES)?)?)
}

fn relative(path: &Path) -> bool {
    let spelling = path.as_os_str().as_encoded_bytes();
    !spelling.is_empty()
        && !spelling.contains(&0)
        && !spelling.contains(&b'\\')
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
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

struct References {
    values: BTreeMap<PathBuf, Reference>,
    bytes: u64,
}

impl References {
    fn new() -> Self {
        Self { values: BTreeMap::new(), bytes: 0 }
    }
    fn add(&mut self, reference: &Reference, maximum: u64) -> Result<()> {
        if let Some(prior) = self.values.get(&reference.path) {
            ensure!(prior == reference, "migration references conflict");
            return Ok(());
        }
        ensure!(self.values.len() < MAX_REFERENCES, "migration reference closure excessive");
        self.bytes = self.bytes.checked_add(reference.bytes).context("migration retained extent overflow")?;
        ensure!(self.bytes <= MAX_RETAINED_BYTES, "migration retained evidence bound exceeded");
        retention::authenticate_reference(reference, maximum)?;
        self.values.insert(reference.path.clone(), reference.clone());
        Ok(())
    }
    fn artifact(&mut self, artifact: &assignment::Artifact) -> Result<()> {
        ensure!(digest_valid(&artifact.sha256), "migration historical artifact seal invalid");
        let reference = retention::seal_file(&artifact.path, MAX_RETAINED_BYTES)?;
        ensure!(reference.sha256 == artifact.sha256, "migration historical artifact changed");
        self.add(&reference, MAX_RETAINED_BYTES)
    }
    fn all(&self) -> Result<()> {
        for reference in self.values.values() {
            retention::authenticate_reference(reference, MAX_RETAINED_BYTES)?;
        }
        Ok(())
    }
}

pub(crate) fn git_bytes(checkout: &Path, arguments: &[&str]) -> Result<Vec<u8>> {
    safe(checkout)?;
    let mut command = Command::new("/usr/bin/git");
    command
        .arg("--no-pager")
        .args(["-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null"])
        .args(["-c", "gc.auto=0", "-c", "maintenance.auto=false"])
        .args(arguments)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(checkout);
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut child = command.spawn()?;
    let mut reaped = false;
    let result = (|| -> Result<Vec<u8>> {
        let mut stdout = child.stdout.take().context("migration Git stdout missing")?;
        let mut stderr = child.stderr.take().context("migration Git stderr missing")?;
        for fd in [stdout.as_raw_fd(), stderr.as_raw_fd()] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            ensure!(
                flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0,
                "migration Git pipe setup failed"
            );
        }
        let mut output = Vec::new();
        let mut diagnostic = Vec::new();
        let mut stdout_done = false;
        let mut stderr_done = false;
        loop {
            drain_git(&mut stdout, &mut output, &mut stdout_done, MAX_BYTES as usize)?;
            drain_git(&mut stderr, &mut diagnostic, &mut stderr_done, 4096)?;
            // Keep the child unreaped while a descendant might own either pipe.
            // Its original process-group id therefore cannot be recycled.
            if stdout_done
                && stderr_done
                && let Some(status) = child.try_wait()?
            {
                reaped = true;
                ensure!(
                    status.success(),
                    "migration source inspection failed: {}",
                    String::from_utf8_lossy(&diagnostic)
                );
                return Ok(output);
            }
            ensure!(Instant::now() < deadline, "migration source inspection deadline exhausted");
            std::thread::sleep(Duration::from_millis(5));
        }
    })();
    if !reaped {
        unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn drain_git(reader: &mut impl Read, output: &mut Vec<u8>, finished: &mut bool, maximum: usize) -> Result<()> {
    if *finished {
        return Ok(());
    }
    let mut buffer = [0u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => {
                *finished = true;
                return Ok(());
            }
            Ok(count) => {
                ensure!(count <= maximum.saturating_sub(output.len()), "migration Git output budget exceeded");
                output.extend_from_slice(&buffer[..count]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
}

pub(crate) fn git(checkout: &Path, arguments: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_bytes(checkout, arguments)?)?.trim().into())
}

/// Read a prior controller's journal using that exact retained executable and
/// its original logical role. This has bounded pipes and no producer action.
pub(crate) fn controller_inspect(
    executable: &Path,
    path: &Path,
    installed: &supervisor::Installation,
) -> Result<::translation_execution::model::View> {
    let mut command = Command::new(executable);
    command
        .args(["translation-execute", "--installation"])
        .arg(path)
        .arg("inspect")
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("SM_PIPELINE_ROOT", &installed.pipeline_root)
        .env("SM_PIPELINE_WORKERS", "4")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&installed.source_repository);
    if let supervisor::Admission::Managed { session_id, .. } = &installed.admission {
        command.env("CODEX_THREAD_ID", session_id);
    }
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn()?;
    let mut reaped = false;
    let result = (|| {
        let mut stdout = child.stdout.take().context("prior controller stdout missing")?;
        let mut stderr = child.stderr.take().context("prior controller stderr missing")?;
        for fd in [stdout.as_raw_fd(), stderr.as_raw_fd()] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            ensure!(
                flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0,
                "prior controller pipe setup failed"
            );
        }
        let deadline = Instant::now() + Duration::from_secs(120);
        let (mut output, mut diagnostic) = (Vec::new(), Vec::new());
        let (mut out_done, mut err_done) = (false, false);
        loop {
            drain_git(&mut stdout, &mut output, &mut out_done, MAX_BYTES as usize)?;
            drain_git(&mut stderr, &mut diagnostic, &mut err_done, 64 * 1024)?;
            if out_done
                && err_done
                && let Some(status) = child.try_wait()?
            {
                reaped = true;
                ensure!(
                    status.success(),
                    "prior controller inspection failed: {}",
                    String::from_utf8_lossy(&diagnostic)
                );
                return toml::from_str(&String::from_utf8(output)?).context("invalid prior controller view");
            }
            ensure!(Instant::now() < deadline, "prior controller inspection deadline exhausted");
            std::thread::sleep(Duration::from_millis(5));
        }
    })();
    if !reaped {
        unsafe { libc::kill(-(child.id() as i32), libc::SIGKILL) };
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn current_authority(migration: &Migration, references: &mut References) -> Result<AuthorityView> {
    let path = migration.checkout.join(AUTHORITY);
    ensure!(migration.authority.path == path, "migration authority is not the current canonical policy");
    references.add(&migration.authority, MAX_BYTES)?;
    let maintained = assignment::authority::operate(assignment::authority::Request {
        checkout: migration.checkout.clone(),
        ..Default::default()
    })?;
    let authority: AuthorityView = object(&path)?;
    ensure!(
        authority.schema == "decomp-session-authority-v1"
            && authority.state == "active"
            && (authority.preparation.canonical_effort == migration.effort
                || authority.preparation.admitted_efforts.contains(&migration.effort))
            && !authority.preparation.drill_efforts.contains(&migration.effort),
        "migration needs current active production authority"
    );
    ensure!(maintained.status == authority.state, "migration current authority status differs");
    let integrators = authority
        .roles
        .values()
        .filter(|role| role.kind == assignment::authority::RoleKind::Integrator)
        .collect::<Vec<_>>();
    ensure!(integrators.len() == 1, "migration needs one current integrator");
    let role = integrators[0];
    ensure!(
        role.token == migration.coordinator
            && role.session_id == migration.session_id
            && role.generation == migration.generation,
        "migration has a stale integrator role or generation"
    );
    references.artifact(&authority.preparation.bridge)?;
    references.artifact(&role.registration)?;
    for transfer in &role.transfers {
        references.artifact(transfer)?;
    }
    if let Some(transfer) = &role.last_transfer {
        references.artifact(&transfer.observation)?;
    }
    references.artifact(authority.drill.as_ref().context("production authority lacks a retained activation drill")?)?;
    Ok(authority)
}

fn original_lineage(
    migration: &Migration,
    migrated: &MigratedClaim,
    entry: &assignment::Entry,
    references: &mut References,
) -> Result<OriginalClaim> {
    references.add(&migrated.original_claim, MAX_BYTES)?;
    let original: OriginalClaim = object(&migrated.original_claim.path)?;
    validate_original_source(migration, migrated, &original, references)?;
    ensure!(
        original.schema == "translation-original-claim-v1"
            && original.checkout == migration.checkout
            && original.effort == migration.effort
            && same_claim(&migrated.claim, &original.claim)
            && entry.claim.as_ref() == Some(&original.claim)
            && original.job == entry.job
            && original.state == entry.state
            && original.agent == entry.agent
            && original.result == entry.result
            && original.observations == entry.observations
            && !entry.stale
            && entry.pending_job.is_none()
            && entry.attempts == migrated.claim.attempt
            && matches!(
                entry.state,
                assignment::State::Reserved | assignment::State::Running | assignment::State::Submitted
            )
            && matches!(entry.job.kind, assignment::Kind::Implement | assignment::Kind::Integrate)
            && entry.job.writable_paths.iter().collect::<BTreeSet<_>>() == migrated.writable_files.iter().collect()
            && original.source_commit == migrated.source_commit
            && exact_commit(&original.source_commit)
            && !original.source_files.is_empty()
            && original.source_files.len() <= 128
            && migrated
                .writable_files
                .iter()
                .all(|path| path.to_str().is_some_and(|name| original.source_files.contains_key(name))),
        "migration original claim, submitted result, source or writable lineage differs from the current ledger"
    );
    if let Some(result) = &entry.result {
        for artifact in assignment::result_artifacts(result) {
            references.artifact(artifact)?;
        }
    }
    for observation in &entry.observations {
        references.artifact(observation)?;
    }
    for prior in &entry.history {
        if let Some(result) = &prior.result {
            for artifact in assignment::result_artifacts(result) {
                references.artifact(artifact)?;
            }
        }
        for observation in &prior.observations {
            references.artifact(observation)?;
        }
    }
    Ok(original)
}

fn validate_original_source(
    migration: &Migration,
    migrated: &MigratedClaim,
    original: &OriginalClaim,
    references: &mut References,
) -> Result<()> {
    ensure!(
        original.schema == "translation-original-claim-v1"
            && original.checkout == migration.checkout
            && original.effort == migration.effort
            && same_claim(&migrated.claim, &original.claim)
            && original.source_commit == migrated.source_commit
            && exact_commit(&original.source_commit)
            && !original.source_files.is_empty()
            && original.source_files.len() <= 128,
        "retained original source identity differs from migration"
    );
    safe(&original.source_checkout)?;
    for (name, hash) in &original.source_files {
        ensure!(relative(Path::new(name)) && digest_valid(hash), "migration source snapshot path/hash invalid");
        let reference = retention::seal_file(&original.source_checkout.join(name), MAX_BYTES)?;
        ensure!(reference.sha256 == *hash, "migration retained original source changed");
        references.add(&reference, MAX_BYTES)?;
    }
    if original.source_checkout.join(".git").exists() {
        ensure!(
            git(&original.source_checkout, &["rev-parse", "HEAD"])? == original.source_commit,
            "migration original worktree source commit changed"
        );
        ensure!(
            git(&original.source_checkout, &["status", "--porcelain", "--untracked-files=all"])?.is_empty(),
            "migration original worktree source is not checkpointed"
        );
        let baseline = original.job.inputs.get("base-head").context("original scope baseline is absent")?;
        ensure!(
            (7..=40).contains(&baseline.len()) && baseline.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "original scope baseline is not an exact or legacy abbreviated Git identity"
        );
        let resolved = git(&original.source_checkout, &["rev-parse", "--verify", &format!("{baseline}^{{commit}}")])?;
        ensure!(
            exact_commit(&resolved) && resolved.starts_with(baseline),
            "legacy original baseline is ambiguous or stale"
        );
        git(&original.source_checkout, &["merge-base", "--is-ancestor", &resolved, &original.source_commit])?;
        for (name, expected) in &original.source_files {
            let source =
                git_bytes(&original.source_checkout, &["show", &format!("{}:{name}", original.source_commit)])?;
            ensure!(
                ::translation_execution::model::sha256(&source) == *expected,
                "migration original source bytes do not match its checkpoint"
            );
        }
    } else {
        // A retired source checkout retains exact bytes, and its original commit
        // must remain reachable in the source authority checkout.
        ensure!(
            git(&migration.checkout, &["rev-parse", "--verify", &format!("{}^{{commit}}", original.source_commit)])?
                == original.source_commit,
            "migration retained source commit is unavailable"
        );
        for (name, expected) in &original.source_files {
            let source = git_bytes(&migration.checkout, &["show", &format!("{}:{name}", original.source_commit)])?;
            ensure!(
                ::translation_execution::model::sha256(&source) == *expected,
                "migration retained source bytes do not match the original commit"
            );
        }
    }
    if let Some(assignment::WorkResult::Submitted { commit, .. }) = &original.result {
        ensure!(commit == &original.source_commit, "migration changed the retained submitted source commit");
    }
    Ok(())
}

fn stopped_owner(
    migration: &Migration,
    original: &OriginalClaim,
    stops: &[OriginalOwnerStop],
    references: &mut References,
) -> Result<()> {
    let matching = stops.iter().filter(|stop| stop.claim == original.claim).collect::<Vec<_>>();
    ensure!(matching.len() == 1, "migration requires one exact observed stopped unrestricted owner per claim");
    let stop = matching[0];
    ensure!(
        stop.schema == "translation-original-owner-stop-v1"
            && stop.checkout == migration.checkout
            && stop.effort == migration.effort
            && stop.agent == original.agent
            && !stop.observed_at.trim().is_empty()
            && stop.observed_at.len() <= 128
            && matches!(stop.status.as_str(), "completed" | "failed" | "interrupted" | "not-launched")
            && (!stop.evidence.is_empty())
            && (original.agent.is_none() || stop.status != "not-launched"),
        "migration stopped observation names another or ambiguous original owner"
    );
    for evidence in &stop.evidence {
        references.add(evidence, MAX_BYTES)?;
    }
    if let Some(agent) = &original.agent {
        let session = stop.session.as_ref().context("started original owner lacks a real session observation")?;
        references.add(session, MAX_BYTES)?;
        let observed: assignment::authority::SessionObservation = object(&session.path)?;
        ensure!(
            observed.schema == "decomp-session-observation-v1"
                && assignment::authority::real_session_id(&observed.session_id)
                && (!assignment::authority::real_session_id(agent) || observed.session_id == *agent)
                && observed.status == stop.status
                && observed.observed_at == stop.observed_at
                && !observed.evidence.is_empty(),
            "migration stopped observation does not establish its exact real original session"
        );
        for evidence in &observed.evidence {
            references.artifact(evidence)?;
        }
    } else {
        ensure!(stop.session.is_none(), "unstarted original claim names another real session");
    }
    for process in &stop.processes {
        ensure!(
            process.identity.pid > 0
                && process.identity.start_ticks > 0
                && !process.identity.boot_id.is_empty()
                && matches!(process.status.as_str(), "exited" | "stopped" | "completed" | "failed" | "not-running"),
            "migration original producer process is still running or ambiguous"
        );
        references.add(&process.observation, MAX_BYTES)?;
        ensure!(
            operation::process_identity(process.identity.pid)?.as_ref() != Some(&process.identity),
            "migration original producer process still owns a writer"
        );
    }
    Ok(())
}

fn coverage(checkout: &Path, proof: &supervisor::Installation) -> Result<CatalogCoverage> {
    let catalog_path = checkout.join("tools/port/batch.toml");
    let mut result = CatalogCoverage {
        catalog_sha256: None,
        families: Vec::new(),
        representative_proof_families: Vec::new(),
        families_without_matching_prepared_proof: Vec::new(),
        structural_confined_recipe_gaps: BTreeMap::new(),
        note:
            "A representative closed proof does not prepare the production catalog or authorize another original claim."
                .into(),
    };
    if !catalog_path.exists() {
        result.note = "This checkout declares no native Offline catalog; standalone Rust tests require the reviewed-in-process-rust-tests acceptance model.".into();
        return Ok(result);
    }
    result.catalog_sha256 = Some(digest(&catalog_path)?);
    let catalog = batch::catalog(checkout)?;
    result.families = catalog.families.keys().cloned().collect();
    if let supervisor::Contract::Native { contracts, .. } = &proof.contract {
        result.representative_proof_families = contracts.iter().map(|contract| contract.family.clone()).collect();
        result.representative_proof_families.sort();
    }
    for name in &result.families {
        let entry = batch::entry(&catalog, name)?;
        let reason = if entry.kind != "family" || entry.prepare.is_some() {
            Some("The confined plain-family recipe does not support this registered dispatcher/preparation contract.")
        } else {
            let manifest: crate::family_model::Manifest = crate::family::object(&checkout.join(&entry.manifest))?;
            if crate::family::dispatcher(&manifest)
                || manifest.projectile_host
                || manifest.projectile_rendering
                || manifest.callback_adapter.is_some()
                || manifest.infrastructure_probe
                || !manifest.cases.is_empty()
            {
                Some("The registered harness requires a specialized confined execution adapter.")
            } else {
                None
            }
        };
        if let Some(reason) = reason {
            result.structural_confined_recipe_gaps.insert(name.clone(), reason.into());
        }
        let matched = if let supervisor::Contract::Native { contracts, catalog_sha256, .. } = &proof.contract {
            proof.source_repository == checkout
                && result.catalog_sha256.as_ref() == Some(catalog_sha256)
                && contracts.iter().any(|contract| contract.family == *name)
        } else {
            false
        };
        if !matched {
            result.families_without_matching_prepared_proof.push(name.clone());
        }
    }
    ensure!(
        Some(digest(&catalog_path)?) == result.catalog_sha256,
        "migration production catalog changed during inspection"
    );
    Ok(result)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct NativeAdmission {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    scope: Reference,
    inventory: Option<Reference>,
    source_revision: Option<String>,
    routines: Vec<String>,
    families: Vec<String>,
    limitations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    additional_references: Vec<Reference>,
}

impl NativeAdmission {
    fn inventory(&self) -> Result<&Reference> {
        self.inventory.as_ref().context("this acceptance domain has no native inventory")
    }
    fn source_revision(&self) -> Result<&str> {
        self.source_revision.as_deref().context("this acceptance domain has no native source revision")
    }
}

fn installed_binding(
    migration: &Migration,
    migrated: &MigratedClaim,
    installed: &supervisor::Installation,
) -> Result<()> {
    let supervisor::Admission::Managed { checkout, effort, coordinator, session_id, generation, migration: asset } =
        &installed.admission
    else {
        bail!("production claim needs a managed installation")
    };
    let path = if asset.file.is_absolute() { asset.file.clone() } else { checkout.join(&asset.file) };
    ensure!(
        checkout == &migration.checkout
            && effort == &migration.effort
            && coordinator == &migration.coordinator
            && session_id == &migration.session_id
            && *generation == migration.generation
            && digest(&path)? == asset.sha256
            && domain_identity(&read(&path)?)? == domain_identity(migration)?,
        "installed actual migration seal or semantic domain differs"
    );
    ensure!(
        installed.policy.claim == migrated.claim
            && installed.policy.scope_id == migrated.scope_id
            && sealed(&installed.policy)? == migrated.policy_sha256
            && installed.policy.adapter_sha256 == supervisor::installation_identity(installed)?
            && installed.policy.baseline_commit == migrated.source_commit
            && installed.policy.approved_files.keys().map(PathBuf::from).collect::<BTreeSet<_>>()
                == migrated.writable_files.iter().cloned().collect()
            && installed.workers == 4
            && installed.policy.budgets.workers == 4
            && !installed.rewrite.recorded,
        "installed policy, original source or ownership differs"
    );
    installed.policy.validate()?;
    Ok(())
}

/// Preserve the original declaration's requirements. Only complete plain native
/// family-verify gates have an equivalent confined adapter today. Cargo, batch,
/// Live replay and specialized recipes are explicit migration blockers.
fn declared_family(gate: &translation::Verification, contracts: &[native::NativeContract]) -> Result<String> {
    ensure!(
        gate.timeout_seconds > 0 && gate.timeout_seconds <= if gate.integration { 3600 } else { 60 },
        "original scope timeout is not bounded"
    );
    let args = gate.argv.iter().map(String::as_str).collect::<Vec<_>>();
    let operation = if args.first() == Some(&"sm-pipeline") {
        &args[1..]
    } else if args.starts_with(&["cargo", "run"]) {
        let separator = args.iter().position(|arg| *arg == "--").context("original Cargo wrapper is malformed")?;
        let prefix = &args[..separator];
        ensure!(
            prefix == ["cargo", "run", "-p", "pipeline", "--locked", "--offline"]
                || prefix == ["cargo", "run", "-p", "pipeline", "--locked", "--offline", "--bin", "sm-pipeline"],
            "original scope Cargo wrapper has no exact confined recipe"
        );
        &args[separator + 1..]
    } else {
        bail!("original scope requires an unsupported Cargo, Live or specialized gate")
    };
    ensure!(
        operation.len() == 3 && operation[0] == "family-verify" && operation[1] == "--manifest",
        "original scope gate has no equivalent declared plain-native recipe"
    );
    let contract = contracts
        .iter()
        .find(|contract| contract.manifest.file == Path::new(operation[2]))
        .context("original scope family gate is absent from the frozen complete native catalog")?;
    Ok(contract.family.clone())
}

fn native_admission(
    migration: &Migration,
    migrated: &MigratedClaim,
    original: &OriginalClaim,
    installed: &supervisor::Installation,
    validate_scope: bool,
) -> Result<NativeAdmission> {
    ensure!(
        ((original.state == assignment::State::Running && original.result.is_none() && migrated.execution.is_none())
            || (original.state == assignment::State::Submitted
                && migrated.execution == Some(MigrationExecution::ImportedSubmission)
                && matches!(&original.result, Some(assignment::WorkResult::Submitted { commit, .. })
                    if commit == &original.source_commit)))
            && original.job.kind == assignment::Kind::Implement
            && original.agent.is_some()
            && (original.claim.coordinator == migration.coordinator
                || (matches!(&installed.contract, supervisor::Contract::Original { contract }
                    if contract.original_claim == migrated.original_claim)
                    && migrated.execution == Some(MigrationExecution::ImportedSubmission)
                    && same_claim(&migrated.claim, &original.claim))),
        "managed execution requires an original running implementation or exact imported submission"
    );
    installed_binding(migration, migrated, installed)?;
    ensure!(installed.integration_repository == migration.checkout, "managed integration source authority differs");
    ensure!(
        original.source_files == installed.policy.approved_files,
        "installed rewrite does not begin at the exact retained original checkpoint"
    );
    if let supervisor::Contract::Legacy { contract } = &installed.contract {
        // Legacy readiness authenticates the preserved original lineage but
        // cannot publish source or pass completion. Its missing-adapter report
        // therefore does not require an otherwise clean host projection.
        let admitted = crate::translation_legacy::validate_original(contract, original)?;
        return Ok(NativeAdmission {
            domain: Some("legacy-infrastructure-v1".into()),
            scope: admitted.scope,
            inventory: None,
            source_revision: None,
            routines: vec![],
            families: vec![],
            limitations: admitted.limitations,
            additional_references: admitted.references,
        });
    }
    if let supervisor::Contract::Original { contract } = &installed.contract {
        if validate_scope {
            validate_projection_records_before_intent(migration, installed)?;
            validate_imported_linkage(migration, original)?;
        }
        let prepared = crate::translation_original::prepare(contract, &installed.pinned_source)?;
        ensure!(
            prepared.original == *original
                && prepared.dependencies_sha256 == installed.policy.dependencies_sha256
                && prepared.verification_sha256 == installed.policy.verification_sha256
                && prepared.pinned_c_sha256 == installed.policy.pinned_c_sha256
                && contract.kind.stages() == installed.policy.verification_stages,
            "original acceptance differs from its exact frozen policy"
        );
        let gameplay = contract.kind == crate::translation_original::OriginalKind::PauseEquipment;
        let (inventory, source_revision, routines) = if gameplay {
            let declared: native::NativeContract =
                crate::translation_original::read(&contract.families["pause-equipment-render"])?;
            let inventory_path = if declared.source_inventory.file.is_absolute() {
                declared.source_inventory.file.clone()
            } else {
                contract.verification_source.root.join(&declared.source_inventory.file)
            };
            let inventory = retention::seal_file(&inventory_path, MAX_BYTES)?;
            ensure!(inventory.sha256 == declared.source_inventory.sha256, "original pause inventory changed");
            let value: toml::Value = object(&inventory_path)?;
            let revision = value["source_identity"]["revision"]
                .as_str()
                .context("original pause source revision missing")?
                .to_owned();
            let routines = contract
                .families
                .values()
                .map(|reference| {
                    Ok(crate::translation_original::read::<native::NativeContract>(reference)?
                        .coverage
                        .into_keys()
                        .collect::<Vec<_>>())
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            (Some(inventory), Some(revision), routines)
        } else {
            (None, None, vec![])
        };
        return Ok(NativeAdmission {
            domain: Some(if gameplay { "original-pause-equipment-v1" } else { "legacy-infrastructure-v1" }.into()),
            scope: contract.original_claim.clone(),
            inventory,
            source_revision,
            routines,
            families: contract.families.keys().cloned().collect(),
            limitations: contract.kind.limitations(),
            additional_references: contract
                .families
                .values()
                .cloned()
                .chain(std::iter::once(contract.cargo.clone()))
                .chain(contract.compact.iter().cloned())
                .collect(),
        });
    }
    if validate_scope {
        validate_projection_records_before_intent(migration, installed)?;
        if migrated.execution == Some(MigrationExecution::ImportedSubmission) {
            validate_imported_linkage(migration, original)?;
        }
        for (key, path) in assignment::admitted_input_files(&migration.checkout, &original.job)? {
            ensure!(digest(&path)? == original.job.inputs[&key], "original native planned input changed: {key}");
        }
    }
    let supervisor::Contract::Native { contracts, catalog_sha256, focused_family } = &installed.contract else {
        bail!("managed original requires its distinct declared acceptance contract")
    };
    if original.job.inputs.get("admission-domain").map(String::as_str) == Some("offline-coverage-v1") {
        return require_offline_composite_adapter(&original.job.id);
    }
    let scope_path =
        PathBuf::from(original.job.inputs.get("scope-file").context("original job lacks native Scope declaration")?);
    let scope_path = if scope_path.is_absolute() { scope_path } else { migration.checkout.join(scope_path) };
    let scope_reference = retention::seal_file(&scope_path, MAX_BYTES)?;
    ensure!(
        Some(&scope_reference.sha256) == original.job.inputs.get("scope-sha256"),
        "original scope declaration seal changed"
    );
    let scope: translation::Scope = object(&scope_path)?;
    ensure!(
        scope.format == "translation-scope-v1"
            && scope.subject == original.job.subject
            && scope.origin_ticket == original.job.ticket
            && !scope.routines.is_empty()
            && scope.routines.len() <= 32
            && scope.writable_paths.iter().collect::<BTreeSet<_>>() == original.job.writable_paths.iter().collect(),
        "original native scope, ticket or writable ownership changed"
    );
    if validate_scope {
        let planned = translation::validate_scope(&original.source_checkout, &scope_path)?;
        ensure!(
            planned.id == original.job.id
                && planned.revision == original.job.revision
                && planned.writable_paths == original.job.writable_paths
                && planned.ticket == original.job.ticket
                && planned.inputs.iter().all(|(key, value)| original.job.inputs.get(key) == Some(value)),
            "original job no longer matches maintained native Scope admission"
        );
    }
    ensure!(
        installed.policy.verification_stages
            == [
                ::translation_execution::VerificationStage::Focused,
                ::translation_execution::VerificationStage::Offline
            ],
        "managed native scope requires complete focused and Offline stages"
    );
    let declared = contracts.iter().map(|contract| contract.family.clone()).collect::<BTreeSet<_>>();
    ensure!(
        !declared.is_empty() && declared.len() == contracts.len() && declared.contains(focused_family),
        "managed native installation lacks the complete conservative affected catalog"
    );
    if validate_scope {
        ensure!(
            digest(&installed.source_repository.join("tools/port/batch.toml"))? == *catalog_sha256
                && digest(&migration.checkout.join("tools/port/batch.toml"))? == *catalog_sha256
                && affected_families(installed)? == declared,
            "managed installation changed the catalog or omitted an affected Offline family"
        );
    }
    let inventory_path = if scope.source_inventory.is_absolute() {
        scope.source_inventory.clone()
    } else {
        migration.checkout.join(&scope.source_inventory)
    };
    let inventory = retention::seal_file(&inventory_path, MAX_BYTES)?;
    ensure!(inventory.sha256 == scope.source_inventory_sha256, "original scope inventory changed");
    let inventory_value: toml::Value = object(&inventory.path)?;
    let revision = inventory_value
        .get("source_identity")
        .and_then(|source| source.get("revision"))
        .and_then(toml::Value::as_str)
        .context("original inventory source revision missing")?
        .to_string();
    ensure!(exact_commit(&revision), "original inventory source revision is inexact");
    let mut all_routines = BTreeSet::new();
    for contract in contracts {
        ensure!(
            contract.source_inventory.sha256 == inventory.sha256,
            "confined native family uses another original scope inventory"
        );
        match &contract.provenance {
            native::NativeProvenance::NativeScope { scope: declared, checkout: owner } => {
                let path = if declared.file.is_absolute() { declared.file.clone() } else { owner.join(&declared.file) };
                ensure!(
                    declared.sha256 == scope_reference.sha256
                        && path == scope_reference.path
                        && *owner == original.source_checkout,
                    "managed native family provenance names another original Scope owner or declaration"
                );
            }
            native::NativeProvenance::ReviewedOfflineFamily { .. } => ensure!(
                contract.family != *focused_family,
                "reviewed historical Offline proof cannot replace the original focused native Scope admission"
            ),
            native::NativeProvenance::ReviewedCatalog { .. } => {
                bail!("reviewed catalog recipes require a distinct original composite contract")
            }
        }
        ensure!(!contract.coverage.is_empty(), "managed native provenance omits an original family routine");
        if validate_scope {
            let prepared = native::prepare(&installed.source_repository, &installed.pinned_source, contract)?;
            ensure!(prepared.source.revision == revision, "prepared native original source identity differs");
        }
        all_routines.extend(contract.coverage.keys().cloned());
    }
    if validate_scope {
        ensure!(
            contracts
                .iter()
                .flat_map(|contract| contract.frozen_inputs.iter())
                .all(|(name, expected)| original.source_files.contains_key(name)
                    || digest(&migration.checkout.join(name)).is_ok_and(|actual| actual == *expected)),
            "managed original host verification dependencies differ from installed source closure"
        );
    }
    ensure!(
        scope.routines.iter().all(|routine| all_routines.contains(routine)),
        "original native routine requirements lack complete fixture coverage"
    );
    let mut candidate = BTreeSet::new();
    let mut integration = BTreeSet::new();
    for gate in &scope.verification {
        let family = declared_family(gate, contracts)?;
        ensure!(
            installed.policy.budgets.gate_timeout_seconds <= gate.timeout_seconds,
            "confined whole-stage timeout weakens an original Scope gate deadline"
        );
        if gate.integration {
            integration.insert(family);
        } else {
            candidate.insert(family);
        }
    }
    ensure!(!candidate.is_empty() && !integration.is_empty(), "original native scope needs both source stages");
    for (name, expected) in &scope.inputs {
        ensure!(relative(Path::new(name)) && digest_valid(expected), "original native scope input identity malformed");
        if validate_scope {
            ensure!(
                digest(&migration.checkout.join(name))? == *expected,
                "original native Scope input changed: {name}"
            );
        } else if let Some(original) = original.source_files.get(name) {
            ensure!(original == expected, "original Scope changed its frozen writable input: {name}");
        }
    }
    Ok(NativeAdmission {
        domain: None,
        scope: scope_reference,
        inventory: Some(inventory),
        source_revision: Some(revision),
        routines: scope.routines,
        families: declared.into_iter().collect(),
        limitations: vec![
            "Actual-C acceptance covers the original frozen fixture and dependency domain.".into(),
            "Original-ROM parity and Live replay were not tested.".into(),
        ],
        additional_references: vec![],
    })
}

fn validate_imported_linkage(migration: &Migration, original: &OriginalClaim) -> Result<()> {
    let ledger = current_ledger(migration)?;
    let expected = assignment::admitted_integration_job(&original.job, &original.claim, &original.source_commit)?;
    let integration = ledger.jobs.get(&expected.id).context("imported submission has no linked integration")?;
    let mut historical = expected.clone();
    historical.inputs.remove("base-head");
    ensure!(
        (integration.job == expected || integration.job == historical)
            && !integration.stale
            && integration.pending_job.is_none()
            && integration.result.is_none(),
        "imported submission integration no longer has its exact original requirements"
    );
    if integration.state == assignment::State::Ready {
        ensure!(integration.claim.is_none() && integration.agent.is_none(), "ready imported integration is owned");
    } else {
        ensure!(
            matches!(integration.state, assignment::State::Reserved | assignment::State::Running),
            "imported linked integration cannot execute"
        );
        let claim = integration.claim.as_ref().context("held imported integration has no claim")?;
        let held = migration
            .claims
            .iter()
            .find(|migrated| same_claim(&migrated.claim, claim))
            .context("existing integration is missing from preserved migrated lineage")?;
        ensure!(
            matches!(&held.execution, Some(MigrationExecution::HeldIntegration { submission })
            if same_claim(submission, &original.claim)),
            "existing integration is not held by its imported submission"
        );
        let mut references = References::new();
        let held_original = original_lineage(migration, held, integration, &mut references)?;
        let stops = migration
            .stopped_owners
            .iter()
            .map(|reference| {
                references.add(reference, MAX_BYTES)?;
                object::<OriginalOwnerStop>(&reference.path)
            })
            .collect::<Result<Vec<_>>>()?;
        stopped_owner(migration, &held_original, &stops, &mut references)?;
        references.all()?;
    }
    Ok(())
}

fn affected_families(installed: &supervisor::Installation) -> Result<BTreeSet<String>> {
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
    let paths = installed.policy.approved_files.keys().cloned().collect::<Vec<_>>();
    let report = crate::affected::select_paths_until(
        &installed.source_repository,
        &paths,
        now.checked_add(PREPARATION_TIMEOUT_MS).context("affected admission deadline overflow")?,
        PREPARATION_IO_BYTES,
    )?;
    Ok(report.families().into_iter().collect())
}

/// The original coverage driver admitted composite family and Rust-test
/// evidence. A plain Native installation cannot express every one of those
/// gates, even when its family coverage is complete. Fail before producers until
/// a distinct frozen composite contract has an implemented confined adapter.
fn require_offline_composite_adapter(original_job_id: &str) -> Result<NativeAdmission> {
    if original_job_id == "pause-equipment-render-offline" {
        bail!(
            "original pause equipment coverage requires a confined composite contract for all 607 actual-C fixture pairs, including auxiliary direct-animation, and all 10 Rust tests; a plain Native contract cannot preserve that acceptance"
        )
    }
    bail!(
        "original offline coverage requires a distinct confined composite contract preserving every original family and Rust-test gate"
    )
}

/// Validates immutable lineage, current recorded authority and actual retained
/// proof. It activates nothing and reports unfinished production contracts.
pub(crate) fn validate_preparation(path: &Path) -> Result<PreparationReport> {
    let deadline = crate::inputs::bounded::now_unix_ms()?
        .checked_add(PREPARATION_TIMEOUT_MS)
        .context("migration preparation deadline overflow")?;
    crate::inputs::bounded::until(deadline, PREPARATION_IO_BYTES, || validate_preparation_inner(path))
}

fn validate_preparation_inner(path: &Path) -> Result<PreparationReport> {
    let migration_reference = retention::seal_file(path, MAX_BYTES)?;
    let migration = read(path)?;
    retention::authenticate_reference(&migration_reference, MAX_BYTES)?;
    ensure!(migration.state == "prepared", "switch package must remain prepared until an explicit decision");
    ensure!(
        assignment::main_checkout(&migration.checkout)? == migration.checkout,
        "migration checkout is not canonical main"
    );
    // Read-only validation never creates a shared lock. It authenticates exact
    // current authority/ledger before and after the whole inspection instead.
    let migration_sha256 = migration_reference.sha256;
    let mut references = References::new();
    current_authority(&migration, &mut references)?;
    ensure!(
        migration.ledger.path == migration.checkout.join(format!(".scratch/{}/ledger.toml", migration.effort)),
        "migration ledger is not the original effort ledger"
    );
    references.add(&migration.ledger, MAX_BYTES)?;
    let ledger: LedgerView = object(&migration.ledger.path)?;
    ensure!(
        ledger.schema == "decomp-assignment-v1" && ledger.capacity > 0 && ledger.capacity <= 64,
        "migration original ledger invalid"
    );
    for (id, entry) in &ledger.jobs {
        ensure!(id == &entry.job.id, "migration original ledger job key differs");
    }
    for reference in &migration.historical {
        references.add(reference, MAX_RETAINED_BYTES)?;
    }
    let mut stops = Vec::new();
    for reference in &migration.stopped_owners {
        references.add(reference, MAX_BYTES)?;
        stops.push(object::<OriginalOwnerStop>(&reference.path)?);
    }
    ensure!(stops.len() == migration.claims.len(), "migration has missing or foreign stopped-owner observations");
    let mut claims = Vec::new();
    let mut scope_blockers = Vec::new();
    for (index, left) in migration.claims.iter().enumerate() {
        for right in migration.claims.iter().skip(index + 1) {
            if left
                .writable_files
                .iter()
                .any(|a| right.writable_files.iter().any(|b| a.starts_with(b) || b.starts_with(a)))
                && !crate::translation_migration_guard::executable_linked_pair(left, right)
            {
                scope_blockers.push(format!(
                    "Original linked submission/integration claims {} and {} retain overlapping held ownership; managed execution for this sequencing remains unsupported.",
                    left.claim.id, right.claim.id
                ));
            }
        }
    }
    for migrated in &migration.claims {
        let entry = ledger.jobs.get(&migrated.claim.id).context("migration original claim is missing")?;
        let before = references.values.len();
        let original = original_lineage(&migration, migrated, entry, &mut references)?;
        stopped_owner(&migration, &original, &stops, &mut references)?;
        let mut blockers = Vec::new();
        let prepared = if matches!(migrated.execution, Some(MigrationExecution::HeldIntegration { .. })) {
            true
        } else if let Some(path) = &migrated.installation {
            let reference = retention::seal_file(path, MAX_BYTES)?;
            references.add(&reference, MAX_BYTES)?;
            let installed: supervisor::Installation = object(path)?;
            match native_admission(&migration, migrated, &original, &installed, true) {
                Ok(contract) => {
                    references.add(&contract.scope, MAX_BYTES)?;
                    for reference in contract.inventory.iter().chain(&contract.additional_references) {
                        references.add(reference, MAX_BYTES)?;
                    }
                    true
                }
                Err(error) => {
                    blockers.push(format!(
                        "Original claim {} is not prepared for managed execution: {error:#}",
                        migrated.claim.id
                    ));
                    false
                }
            }
        } else {
            blockers.push(format!(
                "Original claim {} lacks an exact supported native installation; its held lineage is preserved.",
                migrated.claim.id
            ));
            false
        };
        scope_blockers.extend(blockers.clone());
        claims.push(ClaimPreparation {
            claim: migrated.claim.clone(),
            scope_id: migrated.scope_id.clone(),
            source_commit: migrated.source_commit.clone(),
            exact_original_lineage: true,
            stopped_owner_observed: true,
            historical_reference_count: references.values.len() - before,
            production_policy_prepared: prepared,
            blockers,
        });
    }
    references.add(&migration.proof, MAX_BYTES)?;
    let proof: RetainedProof = object(&migration.proof.path)?;
    ensure!(proof.schema == "translation-supervisor-proof-v1", "migration proof is not a retained supervisor proof");
    references.add(&proof.installation, MAX_BYTES)?;
    references.add(&proof.completion, MAX_BYTES)?;
    let proof_policy = supervisor::authenticate_retained_proof(&proof.installation.path, &proof.completion.path)
        .context("authenticate representative retained supervisor proof")?;
    let installation: supervisor::Installation = object(&proof.installation.path)?;
    let completion: Completion = object(&proof.completion.path)?;
    ensure!(
        matches!(installation.admission, supervisor::Admission::IsolatedProof { .. }),
        "representative migration proof must declare its isolated acceptance domain"
    );
    let catalog = coverage(&migration.checkout, &installation).context("inspect production Offline catalog")?;
    references.all()?;
    ensure!(digest(path)? == migration_sha256, "migration package changed during validation");
    if !catalog.structural_confined_recipe_gaps.is_empty() {
        scope_blockers.push(
            "Production catalog contains specialized native families without a confined execution recipe.".into(),
        );
    }
    if !catalog.families_without_matching_prepared_proof.is_empty() {
        scope_blockers.push("The representative proof does not prepare every production Offline family.".into());
    }
    let activation_ready = scope_blockers.is_empty()
        && claims.iter().all(|claim| claim.production_policy_prepared)
        && catalog.structural_confined_recipe_gaps.is_empty()
        && catalog.families_without_matching_prepared_proof.is_empty();
    scope_blockers.push("Production activation requires the user's separate reviewed switch decision and current real owner/process reconciliation.".into());
    Ok(PreparationReport {
        schema: "translation-migration-preparation-v1".into(),
        status: "validated-with-blockers".into(),
        activated: false,
        activation_ready,
        managed_launch_available: false,
        managed_publication_available: false,
        migration_sha256,
        checkout: migration.checkout,
        effort: migration.effort,
        current_integrator: migration.coordinator,
        current_session_id: migration.session_id,
        current_generation: migration.generation,
        claims,
        representative_proof_policy_sha256: sealed(&proof_policy)?,
        representative_proof_scope: proof_policy.scope_id,
        catalog,
        retained_references: references.values.len(),
        retained_bytes: references.bytes,
        authenticated_proof_finalization_bytes: completion.finalization.retained_bytes,
        blockers: scope_blockers,
        limitations: migration.limitations,
    })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct ManagedPublication {
    pub accepted: Reference,
    pub references: Vec<Reference>,
    pub observation_ids: Vec<String>,
    pub retained_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AdmissionIntent {
    schema: String,
    domain_sha256: String,
    policy_sha256: String,
    completion: Completion,
    original: OriginalClaim,
    original_ledger: Reference,
    original_sources: BTreeMap<String, Reference>,
    original_inputs: assignment::FrozenJobInputs,
    original_ticket: Option<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original_integration_ticket: Option<Reference>,
    original_progress: Reference,
    contract: NativeAdmission,
    references: Vec<Reference>,
    existing_references: Vec<Reference>,
    submitted: assignment::Update,
    progress: Option<progress::AdmittedProgress>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AdmissionDraft {
    schema: String,
    domain_sha256: String,
    policy_sha256: String,
    completion: Completion,
    original: OriginalClaim,
    original_ledger: Reference,
    original_sources: BTreeMap<String, Reference>,
    original_inputs: assignment::FrozenJobInputs,
    original_ticket: Option<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    original_integration_ticket: Option<Reference>,
    original_progress: Reference,
    contract: NativeAdmission,
    existing_references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct AcceptedAdmission {
    schema: String,
    domain_sha256: String,
    policy_sha256: String,
    completion: Completion,
    intent: Reference,
    submission: assignment::Update,
    integration: assignment::Update,
    references: Vec<Reference>,
    existing_references: Vec<Reference>,
    observation_ids: Vec<String>,
    /// Original continuation binds completed mutable projections here before
    /// the terminal journal seals this accepted publication. Later scopes may
    /// change their live paths; the continuation retains exact content copies.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    completed_projection: Option<ControllerProjection>,
}

fn publication_root(migration: &Migration, policy: &FrozenPolicy, attempt: u32) -> Result<PathBuf> {
    ensure!(attempt > 0, "managed attempt number missing");
    Ok(migration.checkout.join(format!(
        ".scratch/{}/receipts/deterministic/{}/attempt-{attempt:04}",
        migration.effort,
        sealed(policy)?
    )))
}

fn managed_migration(installed: &supervisor::Installation) -> Result<(PathBuf, Migration, MigratedClaim)> {
    let supervisor::Admission::Managed { checkout, migration: asset, .. } = &installed.admission else {
        bail!("managed publication requires a managed installation")
    };
    let path = if asset.file.is_absolute() { asset.file.clone() } else { checkout.join(&asset.file) };
    let migration = read(&path)?;
    let registry = crate::translation_production::read_registry(checkout)?;
    let registered = registry.as_ref().is_some_and(|registry| {
        registry.migrations.iter().any(|reference| reference.path == path && reference.sha256 == asset.sha256)
    });
    let successor = crate::translation_continuation::registered(installed, &path, &asset.sha256)?;
    ensure!(
        migration.state == "active"
            && migration.checkout == *checkout
            && digest(&path)? == asset.sha256
            && (registered || successor || digest(&checkout.join(FENCE)).is_ok_and(|digest| digest == asset.sha256)),
        "reviewed deterministic migration is inactive, stale or not installed"
    );
    let migrated = migration
        .claims
        .iter()
        .find(|claim| claim.claim == installed.policy.claim)
        .context("policy is absent from imported original lineage")?
        .clone();
    ensure!(
        successor
            || !registry.as_ref().is_some_and(|registry| registry
                .blocked
                .iter()
                .any(|claim| claim.effort == migration.effort && claim.claim == migrated.claim)),
        "this migrated claim is explicitly stopped at its preserved acceptance prerequisite"
    );
    installed_binding(&migration, &migrated, installed)?;
    ensure!(
        !matches!(migrated.execution, Some(MigrationExecution::HeldIntegration { .. })),
        "held integration is executed only through its exact imported submission"
    );
    Ok((path, migration, migrated))
}

/// Imported source is read from the immutable original claim and never inferred
/// from a mutable current queue state. Every adapter import binds this seal.
pub(crate) fn imported_original(installed: &supervisor::Installation) -> Result<Option<(OriginalClaim, String)>> {
    let (_, migration, migrated) = managed_migration(installed)?;
    if migrated.execution != Some(MigrationExecution::ImportedSubmission) {
        return Ok(None);
    }
    retention::authenticate_reference(&migrated.original_claim, MAX_BYTES)?;
    let original: OriginalClaim = object(&migrated.original_claim.path)?;
    ensure!(
        original.state == assignment::State::Submitted
            && original.job.kind == assignment::Kind::Implement
            && same_claim(&migrated.claim, &original.claim)
            && original.source_commit == installed.policy.baseline_commit
            && original.source_files == installed.policy.approved_files
            && matches!(&original.result, Some(assignment::WorkResult::Submitted { commit, .. })
                if commit == &original.source_commit),
        "imported submission differs from original immutable source"
    );
    Ok(Some((original, sealed(&(domain_identity(&migration)?, migrated.original_claim))?)))
}

fn request(migration: &Migration, capacity: usize) -> assignment::Request {
    assignment::Request {
        operation: "record".into(),
        checkout: migration.checkout.clone(),
        effort: migration.effort.clone(),
        coordinator: migration.coordinator.clone(),
        authority: Some(assignment::SessionAuthority {
            session_id: migration.session_id.clone(),
            generation: migration.generation,
        }),
        capacity,
        ..Default::default()
    }
}

fn current_role(migration: &Migration) -> Result<()> {
    assignment::authority::validate(
        &migration.checkout,
        &migration.effort,
        &migration.coordinator,
        Some(&assignment::SessionAuthority {
            session_id: migration.session_id.clone(),
            generation: migration.generation,
        }),
        true,
    )
}

fn current_ledger(migration: &Migration) -> Result<LedgerView> {
    let path = migration.checkout.join(format!(".scratch/{}/ledger.toml", migration.effort));
    safe(&path)?;
    let ledger: LedgerView = object(&path)?;
    ensure!(
        ledger.schema == "decomp-assignment-v1" && (1..=64).contains(&ledger.capacity),
        "current managed ledger is invalid"
    );
    Ok(ledger)
}

/// Check original claim ownership and requirements before any supervisor
/// producer. A completed original claim may only enter exact accepted-result
/// resume; the supervisor cannot start a new rewrite under that lineage.
pub(crate) fn admit_installation(installed: &supervisor::Installation) -> Result<()> {
    crate::inputs::bounded::limited(|| admit_installation_inner(installed))
}

fn admit_installation_inner(installed: &supervisor::Installation) -> Result<()> {
    let (_path, migration, migrated) = managed_migration(installed)?;
    if crate::translation_continuation::historical(installed)?.is_some() {
        return Ok(());
    }
    let _lock = assignment::mutation_lock(&migration.checkout)?;
    current_role(&migration)?;
    let ledger = current_ledger(&migration)?;
    let entry = ledger.jobs.get(&migrated.claim.id).context("original managed claim missing")?;
    if entry.state == assignment::State::Running {
        let roots = migration.checkout.join(format!(
            ".scratch/{}/receipts/deterministic/{}",
            migration.effort,
            sealed(&installed.policy)?
        ));
        if roots.exists() {
            safe(&roots)?;
            let mut count = 0;
            for directory in std::fs::read_dir(&roots)? {
                crate::inputs::bounded::check()?;
                count += 1;
                ensure!(
                    count <= installed.policy.budgets.max_attempts,
                    "managed recovery attempts exceed frozen bound"
                );
                let path = directory?.path().join("intent.json");
                let pending = path.with_extension("pending");
                if !path.exists() && !pending.exists() {
                    continue;
                }
                let intent: AdmissionIntent = if path.exists() { object(&path)? } else { json_record(&pending)? };
                let authenticated = supervisor::authenticate_managed_completion(installed, &intent.completion)?;
                authenticate_intent(installed, &migration, &intent, &authenticated)?;
                check_intent_queue(installed, &migration, &intent, &ledger)?;
                // A completed finalization intent permits only publication
                // recovery. Source CAS may already have advanced the host while
                // the original queue still names its Running checkpoint.
                return current_role(&migration);
            }
        }
    }
    if entry.state == assignment::State::Complete {
        let roots = migration.checkout.join(format!(
            ".scratch/{}/receipts/deterministic/{}",
            migration.effort,
            sealed(&installed.policy)?
        ));
        safe(&roots)?;
        let mut accepted = false;
        let mut count = 0;
        for file in std::fs::read_dir(&roots)? {
            crate::inputs::bounded::check()?;
            count += 1;
            ensure!(count <= installed.policy.budgets.max_attempts, "managed recovery attempts exceed frozen bound");
            let directory = file?.path();
            let path = directory.join("accepted.json");
            let pending = path.with_extension("pending");
            if !path.exists() && !pending.exists() {
                let intent_path = directory.join("intent.json");
                if intent_path.exists() {
                    let intent: AdmissionIntent = object(&intent_path)?;
                    authenticate_interrupted_completion(installed, &migration, entry, &intent)?;
                    accepted = true;
                }
                continue;
            }
            let record: AcceptedAdmission = if path.exists() { object(&path)? } else { json_record(&pending)? };
            authenticate_accepted(installed, &migration, &record, false)?;
            accepted = true;
        }
        ensure!(accepted, "completed managed claim lacks its exact authenticated supervisor acceptance");
        return Ok(());
    }
    if entry.state == assignment::State::Submitted {
        // An interrupted publisher owns the original exact intent; it grants
        // publication recovery only after retained successful finalization.
        let roots = migration.checkout.join(format!(
            ".scratch/{}/receipts/deterministic/{}",
            migration.effort,
            sealed(&installed.policy)?
        ));
        let mut retained = false;
        let mut count = 0;
        let directories = if roots.exists() {
            safe(&roots)?;
            Some(std::fs::read_dir(&roots)?)
        } else {
            None
        };
        for directory in directories.into_iter().flatten() {
            crate::inputs::bounded::check()?;
            count += 1;
            ensure!(count <= installed.policy.budgets.max_attempts, "managed recovery attempts exceed frozen bound");
            let path = directory?.path().join("intent.json");
            if !path.exists() {
                continue;
            }
            let intent: AdmissionIntent = object(&path)?;
            let authenticated = supervisor::authenticate_managed_completion(installed, &intent.completion)?;
            authenticate_intent(installed, &migration, &intent, &authenticated)?;
            check_intent_queue(installed, &migration, &intent, &ledger)?;
            retained = true;
        }
        if retained {
            return Ok(());
        }
        ensure!(
            migrated.execution == Some(MigrationExecution::ImportedSubmission),
            "submitted managed claim cannot be freshly rewritten under its original identity"
        );
    }
    let mut references = References::new();
    current_authority(&migration, &mut references)?;
    references.add(&migration.ledger, MAX_BYTES)?;
    let original = original_lineage(&migration, &migrated, entry, &mut references)?;
    let stops = migration
        .stopped_owners
        .iter()
        .map(|reference| {
            references.add(reference, MAX_BYTES)?;
            object::<OriginalOwnerStop>(&reference.path)
        })
        .collect::<Result<Vec<_>>>()?;
    stopped_owner(&migration, &original, &stops, &mut references)?;
    native_admission(&migration, &migrated, &original, installed, true)?;
    references.all()?;
    current_role(&migration)
}

fn authenticate_interrupted_completion(
    installed: &supervisor::Installation,
    migration: &Migration,
    entry: &assignment::Entry,
    intent: &AdmissionIntent,
) -> Result<()> {
    ensure!(
        intent.schema == "translation-managed-publication-intent-v1"
            && intent.domain_sha256 == domain_identity(migration)?
            && intent.policy_sha256 == sealed(&installed.policy)?
            && entry.claim.as_ref() == Some(&intent.original.claim)
            && entry.state == assignment::State::Complete
            && entry.job == intent.original.job
            && entry.agent == intent.original.agent
            && entry.last_update.as_ref() == Some(&intent.submitted)
            && !entry.stale
            && entry.pending_job.is_none(),
        "interrupted managed completion lost original submitted identity"
    );
    let authenticated = supervisor::authenticate_managed_completion(installed, &intent.completion)?;
    authenticate_intent(installed, migration, intent, &authenticated)?;
    let ledger = current_ledger(migration)?;
    check_intent_queue(installed, migration, intent, &ledger)?;
    let integration = ledger
        .jobs
        .get(&format!("integrate-{}", intent.original.claim.id))
        .context("interrupted completion lacks standard derived integration")?;
    let Some(assignment::WorkResult::Integrated { submissions, evidence, authentication, cleanup }) =
        &integration.result
    else {
        bail!("interrupted completion lacks admitted integration evidence")
    };
    ensure!(
        integration.state == assignment::State::Complete
            && integration.claim.as_ref().is_some_and(|claim| claim.coordinator == migration.coordinator
                && claim.revision == integration.job.revision
                && claim.attempt == integration.attempts)
            && submissions == std::slice::from_ref(&intent.original.claim)
            && !integration.stale
            && integration.pending_job.is_none(),
        "interrupted completion derived integration identity changed"
    );
    for artifact in evidence.iter().chain([authentication, cleanup]) {
        ensure!(digest(&artifact.path)? == artifact.sha256, "interrupted completion queue evidence changed");
    }
    Ok(())
}

fn directory(path: &Path) -> Result<()> {
    let mut current = PathBuf::new();
    for part in path.components() {
        ensure!(matches!(part, Component::RootDir | Component::Normal(_)), "managed publication path alias");
        current.push(part);
        match current.symlink_metadata() {
            Ok(metadata) => ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "managed publication parent is aliased"
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => std::fs::create_dir(&current)?,
            Err(error) => return Err(error.into()),
        }
    }
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "managed publication root is not controller-owned"
    );
    Ok(())
}

fn write_bytes(path: &Path, value: &[u8]) -> Result<Reference> {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    ensure!(value.len() as u64 <= MAX_BYTES, "managed record metadata budget exceeded");
    directory(path.parent().context("managed record parent missing")?)?;
    let pending = path.with_extension("pending");
    if pending.exists() {
        let metadata = pending.symlink_metadata()?;
        ensure!(
            metadata.is_file()
                && !metadata.file_type().is_symlink()
                && metadata.uid() == unsafe { libc::geteuid() }
                && metadata.nlink() == 1
                && metadata.len() <= MAX_BYTES
                && bytes(&pending, MAX_BYTES)? == value,
            "managed pending record is partial, changed or unrecognized; preserve it for explicit recovery"
        );
        if path.exists() {
            ensure!(bytes(path, MAX_BYTES)? == value, "managed immutable record retry differs");
            std::fs::remove_file(&pending)?;
        } else {
            std::fs::set_permissions(&pending, std::fs::Permissions::from_mode(0o400))?;
            std::fs::rename(&pending, path)?;
            std::fs::File::open(path.parent().unwrap())?.sync_all()?;
            return retention::seal_file(path, MAX_BYTES);
        }
    }
    if path.exists() {
        ensure!(bytes(path, MAX_BYTES)? == value, "managed immutable record retry differs");
    } else {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&pending)?;
        file.write_all(value)?;
        file.sync_all()?;
        file.set_permissions(std::fs::Permissions::from_mode(0o400))?;
        std::fs::rename(&pending, path)?;
        std::fs::File::open(path.parent().unwrap())?.sync_all()?;
    }
    retention::seal_file(path, MAX_BYTES)
}

fn write<T: Serialize>(path: &Path, value: &T) -> Result<Reference> {
    write_bytes(path, &serde_json::to_vec(value)?)
}

fn completion_gates(
    installed: &supervisor::Installation,
    completion: &Completion,
    authenticated: &supervisor::AuthenticatedCompletion,
    contract: &NativeAdmission,
) -> Result<()> {
    if let supervisor::Contract::Legacy { contract } = &installed.contract {
        return crate::translation_legacy::authenticate_completion(contract);
    }
    use ::translation_execution::{ExecutionPhase, VerificationStage};
    ensure!(
        authenticated.policy == installed.policy
            && completion.policy_sha256 == sealed(&installed.policy)?
            && authenticated.candidate_revision.source == completion.candidate
            && authenticated.integrated_revision.source == completion.integrated
            && completion.finalization.cleanup_complete
            && completion.finalization.worker_retired
            && completion.finalization.retained_authenticatable
            && completion.gates.iter().all(|gate| gate.passed),
        "managed completion is not the exact authenticated successful finalization"
    );
    if let supervisor::Contract::Original { contract: original } = &installed.contract {
        ensure!(
            authenticated.original_gates.len() == original.kind.stages().len() * 2,
            "original composite completion omitted a source stage"
        );
        for phase in [ExecutionPhase::Candidate, ExecutionPhase::Integration] {
            let source =
                if phase == ExecutionPhase::Candidate { &completion.candidate } else { &completion.integrated };
            for stage in original.kind.stages() {
                let gates = authenticated
                    .original_gates
                    .iter()
                    .filter(|gate| gate.phase == phase && gate.stage == stage)
                    .collect::<Vec<_>>();
                ensure!(gates.len() == 1, "original completion duplicated or omitted a required source gate");
                let gate = gates[0];
                ensure!(
                    gate.source == *source
                        && gate.receipt.source == *source
                        && gate.receipt.passed
                        && gate.verdict.passed
                        && sealed(&gate.prepared.contract)? == sealed(original)?
                        && gate.prepared.dependencies_sha256 == installed.policy.dependencies_sha256
                        && gate.prepared.verification_sha256 == installed.policy.verification_sha256,
                    "original composite evidence changed its requirements or source identity"
                );
                crate::translation_original::authenticate(&gate.prepared, &gate.receipt)?;
            }
        }
        return Ok(());
    }
    let supervisor::Contract::Native { contracts, focused_family, .. } = &installed.contract else {
        bail!("managed completion requires native gameplay evidence")
    };
    for phase in [ExecutionPhase::Candidate, ExecutionPhase::Integration] {
        let source = if phase == ExecutionPhase::Candidate { &completion.candidate } else { &completion.integrated };
        for stage in [VerificationStage::Focused, VerificationStage::Offline] {
            let expected = if stage == VerificationStage::Focused {
                BTreeSet::from([focused_family.clone()])
            } else {
                contract.families.iter().cloned().collect::<BTreeSet<_>>()
            };
            let mut observed = BTreeSet::new();
            for gate in authenticated.native_gates.iter().filter(|gate| gate.phase == phase && gate.stage == stage) {
                ensure!(
                    gate.source == *source
                        && gate.receipt.passed
                        && gate.verdict.status == "offline-verified"
                        && gate.verdict.verified_cases > 0
                        && gate.verdict.verified_routines > 0
                        && expected.contains(&gate.verdict.family)
                        && observed.insert(gate.verdict.family.clone())
                        && contracts.iter().any(|declared| declared.family == gate.verdict.family
                            && sealed(declared).ok() == sealed(&gate.prepared.contract).ok()),
                    "managed actual-C gate source, coverage or prescribed family changed"
                );
                ensure!(
                    gate.verdict.source.as_ref() == Some(&gate.prepared.source)
                        && gate.prepared.source.revision == contract.source_revision()?,
                    "managed actual-C evidence changed original pinned source identity"
                );
            }
            ensure!(observed == expected, "managed actual-C completion lacks prescribed source-stage evidence");
        }
    }
    Ok(())
}

fn authentication_value(
    original: &OriginalClaim,
    completion: &Completion,
    contract: &NativeAdmission,
    reference: &Reference,
) -> serde_json::Value {
    if contract.domain.as_deref() == Some("legacy-infrastructure-v1") {
        return serde_json::json!({
            "schema":"translation-managed-infrastructure-authentication-v1", "claim":original.claim,
            "policy_sha256":completion.policy_sha256, "candidate":completion.candidate,
            "integrated":completion.integrated, "completion":reference, "original_scope":contract.scope,
            "admission_domain":"legacy-infrastructure-v1", "limitations":contract.limitations
        });
    }
    let mut value = serde_json::json!({
        "schema":"translation-managed-native-authentication-v1", "claim":original.claim,
        "policy_sha256":completion.policy_sha256, "candidate":completion.candidate,
        "integrated":completion.integrated, "completion":reference,
        "original_scope":contract.scope, "source_revision":contract.source_revision,
        "families":contract.families, "actual_c_verified":true,
        "original_rom_parity":"not-tested", "live_replay":"not-tested"
    });
    if let Some(domain) = &contract.domain {
        value["admission_domain"] = domain.clone().into();
    }
    value
}

fn submission_update(
    original: &OriginalClaim,
    completion: &Completion,
    completion_reference: &Reference,
    authentication: &Reference,
) -> Result<assignment::Update> {
    let result = if original.state == assignment::State::Submitted {
        ensure!(
            matches!(&original.result, Some(assignment::WorkResult::Submitted { commit, .. })
                if commit == &original.source_commit && commit == &completion.candidate.commit),
            "imported candidate changed its original submitted commit"
        );
        original.result.clone().unwrap()
    } else {
        assignment::WorkResult::Submitted {
            commit: completion.candidate.commit.clone(),
            evidence: vec![artifact(completion_reference), artifact(authentication)],
        }
    };
    Ok(assignment::Update::Finished { claim: original.claim.clone(), result })
}

/// Old standard integration jobs omitted base-head. Preserve those exact bytes
/// while still rejecting arbitrary jobs with superficially matching links.
fn derived_job(intent: &AdmissionIntent) -> Result<assignment::Job> {
    let expected = assignment::admitted_integration_job(
        &intent.original.job,
        &intent.original.claim,
        &intent.completion.candidate.commit,
    )?;
    let ledger: LedgerView = object(&intent.original_ledger.path)?;
    if let Some(entry) = ledger.jobs.get(&expected.id) {
        let mut historical = expected.clone();
        historical.inputs.remove("base-head");
        ensure!(entry.job == expected || entry.job == historical, "original linked integration job changed");
        Ok(entry.job.clone())
    } else {
        ensure!(intent.original.state != assignment::State::Submitted, "original submission lacks linked integration");
        Ok(expected)
    }
}

fn original_integration_ticket(migration: &Migration, original: &OriginalClaim) -> Result<Option<PathBuf>> {
    if original.state != assignment::State::Submitted {
        return Ok(None);
    }
    let job = assignment::admitted_integration_job(&original.job, &original.claim, &original.source_commit)?;
    let path = assignment::admitted_ticket_path(&migration.checkout, &migration.effort, &job);
    Ok(path.exists().then_some(path))
}

fn cleanup_value(
    original: &OriginalClaim,
    completion: &Completion,
    authenticated: &supervisor::AuthenticatedCompletion,
    reference: &Reference,
) -> serde_json::Value {
    serde_json::json!({
        "schema":"translation-managed-cleanup-v1", "claim":original.claim,
        "completion":reference, "finalization":completion.finalization,
        "candidate_revision":authenticated.candidate_revision.manifest,
        "integrated_revision":authenticated.integrated_revision.manifest
    })
}

fn check_original_intent(migration: &Migration, migrated: &MigratedClaim, intent: &AdmissionIntent) -> Result<()> {
    retention::authenticate_reference(&migrated.original_claim, MAX_BYTES)?;
    let original: OriginalClaim = object(&migrated.original_claim.path)?;
    retention::authenticate_reference(&intent.original_ledger, MAX_BYTES)?;
    let original_ledger: LedgerView = object(&intent.original_ledger.path)?;
    let entry =
        original_ledger.jobs.get(&original.claim.id).context("managed original ledger snapshot claim missing")?;
    ensure!(
        original == intent.original
            && original.schema == "translation-original-claim-v1"
            && original.checkout == migration.checkout
            && original.effort == migration.effort
            && same_claim(&migrated.claim, &original.claim)
            && matches!(original.state, assignment::State::Running | assignment::State::Submitted)
            && original.job.kind == assignment::Kind::Implement
            && ((original.state == assignment::State::Running
                && original.result.is_none()
                && migrated.execution.is_none())
                || (original.state == assignment::State::Submitted
                    && migrated.execution == Some(MigrationExecution::ImportedSubmission)
                    && matches!(&original.result, Some(assignment::WorkResult::Submitted { commit, .. })
                        if commit == &intent.completion.candidate.commit && commit == &original.source_commit)))
            && original.agent.is_some()
            && original.source_commit == migrated.source_commit
            && original.job.writable_paths.iter().collect::<BTreeSet<_>>() == migrated.writable_files.iter().collect()
            && entry.job == original.job
            && entry.claim.as_ref() == Some(&original.claim)
            && entry.state == original.state
            && entry.agent == original.agent
            && entry.result == original.result
            && entry.observations == original.observations
            && entry.attempts == original.claim.attempt
            && !entry.stale
            && entry.pending_job.is_none()
            && intent.original_ledger.sha256 == migration.ledger.sha256
            && intent.original_ledger.bytes == migration.ledger.bytes,
        "managed intent changed original immutable claim, job, owner or ledger"
    );
    ensure!(
        intent.original_sources.keys().collect::<BTreeSet<_>>() == original.source_files.keys().collect(),
        "managed intent original source closure changed"
    );
    for (name, reference) in &intent.original_sources {
        ensure!(
            relative(Path::new(name)) && digest_valid(&reference.sha256),
            "managed original source path/hash invalid"
        );
        retention::authenticate_reference(reference, MAX_BYTES)?;
        ensure!(
            original.source_files.get(name) == Some(&reference.sha256)
                && ::translation_execution::model::sha256(&git_bytes(
                    &migration.checkout,
                    &["show", &format!("{}:{name}", original.source_commit)]
                )?) == reference.sha256,
            "managed intent changed original frozen source or checkpoint"
        );
    }
    assignment::authenticate_frozen_job_inputs(&migration.checkout, &intent.original.job, &intent.original_inputs)?;
    Ok(())
}

fn projected_job_inputs(checkout: &Path, job: &assignment::Job, root: &Path) -> Result<assignment::FrozenJobInputs> {
    let mut total = 0u64;
    let mut files = BTreeMap::new();
    for (key, path) in assignment::admitted_input_files(checkout, job)? {
        let value = bytes(&path, MAX_BYTES)?;
        total = total.checked_add(value.len() as u64).context("original input extent overflow")?;
        ensure!(total <= MANAGED_METADATA_RESERVATION, "original frozen inputs exceed managed publication reservation");
        let sha256 = ::translation_execution::model::sha256(&value);
        ensure!(sha256 == job.inputs[&key], "original nonowned input changed before publication: {key}");
        files.insert(
            key.clone(),
            Reference {
                path: root.join("original-inputs").join(format!("{}.bin", sealed(&key)?)),
                sha256,
                bytes: value.len() as u64,
            },
        );
    }
    Ok(assignment::FrozenJobInputs {
        schema: "translation-original-job-inputs-v1".into(),
        checkout: checkout.into(),
        job: job.clone(),
        files,
    })
}

fn validate_projected_job_inputs(
    checkout: &Path,
    job: &assignment::Job,
    root: &Path,
    inputs: &assignment::FrozenJobInputs,
) -> Result<()> {
    ensure!(
        inputs.schema == "translation-original-job-inputs-v1" && inputs.checkout == checkout && inputs.job == *job,
        "managed original input projection identity changed"
    );
    let original = assignment::admitted_input_files(checkout, job)?;
    ensure!(original.keys().eq(inputs.files.keys()), "managed original input projection membership changed");
    for (key, reference) in &inputs.files {
        ensure!(
            reference.path == root.join("original-inputs").join(format!("{}.bin", sealed(key)?))
                && job.inputs.get(key) == Some(&reference.sha256)
                && reference.bytes <= MAX_BYTES,
            "managed original input projection location or seal changed"
        );
    }
    Ok(())
}

fn persist_job_inputs(
    checkout: &Path,
    job: &assignment::Job,
    root: &Path,
    inputs: &assignment::FrozenJobInputs,
) -> Result<()> {
    validate_projected_job_inputs(checkout, job, root, inputs)?;
    for (key, path) in assignment::admitted_input_files(checkout, job)? {
        let reference = write_bytes(&inputs.files[&key].path, &bytes(&path, MAX_BYTES)?)?;
        ensure!(reference == inputs.files[&key], "original planned input changed while retaining immutable bytes");
    }
    assignment::authenticate_frozen_job_inputs(checkout, job, inputs)
}

fn validate_projection_records_before_intent(
    migration: &Migration,
    installed: &supervisor::Installation,
) -> Result<()> {
    let preserved = crate::translation_continuation::preserved(installed)?;
    let mut allowed = preserved.iter().map(|reference| reference.path.clone()).collect::<BTreeSet<_>>();
    for reference in &preserved {
        retention::authenticate_reference(reference, 1024 * 1024 * 1024)?;
    }
    ensure!(
        installed.integration_baseline == git(&migration.checkout, &["rev-parse", "HEAD"])?
            && git(&migration.checkout, &["diff", "--cached", "--name-only"])?.is_empty(),
        "managed fresh publication requires unchanged tracked host source and index"
    );
    for name in git_bytes(&migration.checkout, &["diff", "--name-only", "--no-renames", "-z", "HEAD", "--"])?
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        ensure!(
            allowed.contains(&migration.checkout.join(std::str::from_utf8(name)?)),
            "managed fresh publication found an unreviewed tracked modification"
        );
    }
    let roots = migration.checkout.join(format!(
        ".scratch/{}/receipts/deterministic/{}",
        migration.effort,
        sealed(&installed.policy)?
    ));
    for attempt in 1..=installed.policy.budgets.max_attempts {
        let root = roots.join(format!("attempt-{attempt:04}"));
        let mut draft_path = root.join("draft.json");
        if !draft_path.exists() {
            draft_path = draft_path.with_extension("pending");
            if !draft_path.exists() {
                continue;
            }
        }
        let draft: AdmissionDraft = json_record(&draft_path)?;
        ensure!(
            draft.schema == "translation-managed-publication-draft-v1"
                && draft.domain_sha256 == domain_identity(migration)?
                && draft.policy_sha256 == sealed(&installed.policy)?
                && draft.completion.attempt == attempt
                && draft.completion.policy_sha256 == draft.policy_sha256,
            "managed draft recovery identity changed"
        );
        let authenticated = supervisor::authenticate_managed_completion(installed, &draft.completion)?;
        completion_gates(installed, &draft.completion, &authenticated, &draft.contract)?;
        ensure!(draft.existing_references == authenticated.references, "managed draft evidence closure changed");
        let migrated = migration
            .claims
            .iter()
            .find(|claim| claim.claim == installed.policy.claim)
            .context("managed draft original claim absent")?;
        retention::authenticate_reference(&migrated.original_claim, MAX_BYTES)?;
        ensure!(
            object::<OriginalClaim>(&migrated.original_claim.path)? == draft.original,
            "managed draft original lineage changed"
        );
        ensure!(
            native_admission(migration, migrated, &draft.original, installed, false)? == draft.contract,
            "managed draft original verification contract changed"
        );
        validate_projected_job_inputs(&migration.checkout, &draft.original.job, &root, &draft.original_inputs)?;
        ensure!(
            draft.original_inputs == projected_job_inputs(&migration.checkout, &draft.original.job, &root)?,
            "managed draft original input bindings changed"
        );
        let projected_reference = |path: PathBuf, source: &Path| -> Result<Reference> {
            let value = bytes(source, MAX_BYTES)?;
            Ok(Reference { path, sha256: ::translation_execution::model::sha256(&value), bytes: value.len() as u64 })
        };
        ensure!(
            draft.original_progress
                == projected_reference(
                    root.join("original-progress.json"),
                    &migration.checkout.join(".scratch/decomp-progress/registry.json")
                )?,
            "managed draft original registry changed"
        );
        let ticket = assignment::admitted_ticket_path(&migration.checkout, &migration.effort, &draft.original.job);
        let ticket_reference =
            ticket.exists().then(|| projected_reference(root.join("original-ticket.md"), &ticket)).transpose()?;
        ensure!(draft.original_ticket == ticket_reference, "managed draft original ticket changed");
        let integration_ticket = original_integration_ticket(migration, &draft.original)?
            .map(|path| projected_reference(root.join("original-integration-ticket.md"), &path))
            .transpose()?;
        ensure!(draft.original_integration_ticket == integration_ticket, "original integration ticket changed");
        ensure!(
            draft.original_ledger == Reference { path: root.join("original-ledger.toml"), ..migration.ledger.clone() }
                && draft.original_sources.keys().collect::<BTreeSet<_>>()
                    == draft.original.source_files.keys().collect()
                && draft
                    .original_sources
                    .iter()
                    .all(|(name, reference)| reference.path == root.join("original-source").join(name)
                        && draft.original.source_files.get(name) == Some(&reference.sha256))
                && draft.original_progress.path == root.join("original-progress.json")
                && draft
                    .original_ticket
                    .as_ref()
                    .is_none_or(|reference| reference.path == root.join("original-ticket.md")),
            "managed draft immutable projection locations changed"
        );
        for reference in [&draft.original_ledger, &draft.original_progress]
            .into_iter()
            .chain(draft.original_sources.values())
            .chain(draft.original_ticket.iter())
            .chain(draft.original_integration_ticket.iter())
            .chain(draft.original_inputs.files.values())
        {
            if reference.path.exists() {
                retention::authenticate_reference(reference, MAX_BYTES)?;
            }
            let pending = reference.path.with_extension("pending");
            if pending.exists() {
                retention::authenticate_reference(
                    &Reference { path: pending.clone(), ..reference.clone() },
                    MAX_BYTES,
                )?;
                allowed.insert(pending);
            }
        }
        let completion_path = root.join("completion.json");
        ensure!(
            completion_path.exists()
                || (!root.join("authentication.json").exists() && !root.join("cleanup.json").exists()),
            "partial managed result metadata lacks its exact completion dependency"
        );
        if completion_path.exists() {
            ensure!(object::<Completion>(&completion_path)? == draft.completion, "managed draft completion changed");
            let completion_reference = retention::seal_file(&completion_path, MAX_BYTES)?;
            for (path, expected) in [
                (
                    root.join("authentication.json"),
                    authentication_value(&draft.original, &draft.completion, &draft.contract, &completion_reference),
                ),
                (
                    root.join("cleanup.json"),
                    cleanup_value(&draft.original, &draft.completion, &authenticated, &completion_reference),
                ),
            ] {
                if path.exists() {
                    ensure!(object::<serde_json::Value>(&path)? == expected, "managed draft result metadata changed");
                }
                let pending = path.with_extension("pending");
                if pending.exists() {
                    ensure!(
                        bytes(&pending, MAX_BYTES)? == serde_json::to_vec(&expected)?,
                        "managed pending result metadata is partial or changed"
                    );
                    allowed.insert(pending);
                }
            }
        }
        for (path, expected) in [
            (root.join("draft.json"), serde_json::to_vec(&draft)?),
            (root.join("completion.json"), serde_json::to_vec(&draft.completion)?),
        ] {
            let pending = path.with_extension("pending");
            if pending.exists() {
                ensure!(
                    bytes(&pending, MAX_BYTES)? == expected,
                    "managed pending draft/completion is partial or changed"
                );
                allowed.insert(pending);
            }
        }
        let pending_intent = root.join("intent.pending");
        if pending_intent.exists() {
            let intent: AdmissionIntent = json_record(&pending_intent)?;
            ensure!(intent.completion == draft.completion, "managed pending intent completion changed");
            authenticate_intent(installed, migration, &intent, &authenticated)?;
            ensure!(
                bytes(&pending_intent, MAX_BYTES)? == serde_json::to_vec(&intent)?,
                "managed pending intent bytes differ"
            );
            allowed.insert(pending_intent);
        }
        allowed.extend([
            root.join("draft.json"),
            root.join("original-ledger.toml"),
            root.join("original-progress.json"),
            root.join("original-ticket.md"),
            root.join("original-integration-ticket.md"),
            root.join("completion.json"),
            root.join("authentication.json"),
            root.join("cleanup.json"),
        ]);
        allowed.extend(installed.policy.approved_files.keys().map(|name| root.join("original-source").join(name)));
        allowed.extend(draft.original_inputs.files.values().map(|reference| reference.path.clone()));
    }
    for name in git_bytes(&migration.checkout, &["ls-files", "--others", "--exclude-standard", "-z"])?
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
    {
        let path = migration.checkout.join(std::str::from_utf8(name)?);
        ensure!(allowed.contains(&path), "unrelated dirty host file before managed publication intent");
        if !preserved.iter().any(|reference| reference.path == path) {
            bytes(&path, MAX_BYTES)?;
        }
    }
    Ok(())
}

fn authenticate_intent(
    installed: &supervisor::Installation,
    migration: &Migration,
    intent: &AdmissionIntent,
    authenticated: &supervisor::AuthenticatedCompletion,
) -> Result<()> {
    let migrated = migration
        .claims
        .iter()
        .find(|claim| claim.claim == installed.policy.claim)
        .context("managed intent original migration claim missing")?;
    ensure!(
        intent.schema == "translation-managed-publication-intent-v1"
            && intent.domain_sha256 == domain_identity(migration)?
            && intent.policy_sha256 == sealed(&installed.policy)?
            && intent.completion.policy_sha256 == intent.policy_sha256,
        "managed immutable intent identity changed"
    );
    check_original_intent(migration, migrated, intent)?;
    ensure!(
        intent.original.source_files == installed.policy.approved_files,
        "managed intent changed frozen rewrite files"
    );
    let root = publication_root(migration, &installed.policy, intent.completion.attempt)?;
    let draft_path = root.join("draft.json");
    let draft_reference = retention::seal_file(&draft_path, MAX_BYTES)?;
    let draft: AdmissionDraft = object(&draft_path)?;
    ensure!(
        draft.schema == "translation-managed-publication-draft-v1"
            && draft.domain_sha256 == intent.domain_sha256
            && draft.policy_sha256 == intent.policy_sha256
            && draft.completion == intent.completion
            && draft.original == intent.original
            && draft.original_ledger == intent.original_ledger
            && draft.original_sources == intent.original_sources
            && draft.original_inputs == intent.original_inputs
            && draft.original_ticket == intent.original_ticket
            && draft.original_integration_ticket == intent.original_integration_ticket
            && draft.original_progress == intent.original_progress
            && draft.contract == intent.contract
            && draft.existing_references == intent.existing_references,
        "managed intent differs from its original prepublication draft"
    );
    validate_projected_job_inputs(&migration.checkout, &intent.original.job, &root, &intent.original_inputs)?;
    ensure!(
        intent.original_ledger.path == root.join("original-ledger.toml")
            && intent.original_progress.path == root.join("original-progress.json")
            && intent
                .original_ticket
                .as_ref()
                .is_none_or(|reference| reference.path == root.join("original-ticket.md"))
            && intent
                .original_integration_ticket
                .as_ref()
                .is_none_or(|reference| reference.path == root.join("original-integration-ticket.md"))
            && intent
                .original_sources
                .iter()
                .all(|(name, reference)| reference.path == root.join("original-source").join(name)),
        "managed immutable snapshots belong to another publication"
    );
    let contract = native_admission(migration, migrated, &intent.original, installed, false)?;
    ensure!(contract == intent.contract, "managed original verification contract differs from retained intent");
    completion_gates(installed, &intent.completion, authenticated, &contract)?;
    let completion_reference = retention::seal_file(&root.join("completion.json"), MAX_BYTES)?;
    ensure!(
        object::<Completion>(&completion_reference.path)? == intent.completion,
        "managed completion pointer changed"
    );
    let authentication = retention::seal_file(&root.join("authentication.json"), MAX_BYTES)?;
    ensure!(
        object::<serde_json::Value>(&authentication.path)?
            == authentication_value(&intent.original, &intent.completion, &contract, &completion_reference),
        "managed authentication metadata changed"
    );
    let cleanup = retention::seal_file(&root.join("cleanup.json"), MAX_BYTES)?;
    ensure!(
        object::<serde_json::Value>(&cleanup.path)?
            == cleanup_value(&intent.original, &intent.completion, authenticated, &completion_reference),
        "managed cleanup metadata changed"
    );
    ensure!(
        intent.submitted
            == submission_update(&intent.original, &intent.completion, &completion_reference, &authentication)?,
        "managed intent changed exact original submission"
    );
    let evidence = [
        (completion_reference.path.clone(), completion_reference.sha256.clone()),
        (authentication.path.clone(), authentication.sha256.clone()),
        (cleanup.path.clone(), cleanup.sha256.clone()),
    ];
    if let Some(plan) = &intent.progress {
        progress::validate_admitted_plan(
            plan,
            &sealed(&(intent.completion.run_id.clone(), intent.completion.attempt))?,
            &migrated.scope_id,
            intent.original.agent.as_deref().context("managed original worker missing")?,
            &contract.routines,
            contract.source_revision()?,
            &contract.inventory()?.sha256,
            &intent.completion.candidate.commit,
            &intent.completion.integrated.commit,
            &evidence,
            &contract.limitations,
        )?;
    } else {
        ensure!(
            contract.domain.as_deref() == Some("legacy-infrastructure-v1") && contract.routines.is_empty(),
            "native managed acceptance omitted gameplay progress"
        );
    }
    let mut references = References::new();
    for reference in &intent.references {
        references.add(reference, MAX_RETAINED_BYTES)?;
    }
    ensure!(
        references.values.len() == intent.references.len() && intent.existing_references == authenticated.references,
        "managed intent changed or duplicated retained reference closure"
    );
    for reference in [
        &migrated.original_claim,
        &draft_reference,
        &intent.original_ledger,
        &intent.original_progress,
        &contract.scope,
        &completion_reference,
        &authentication,
        &cleanup,
    ]
    .into_iter()
    .chain(intent.original_sources.values())
    .chain(contract.inventory.iter())
    .chain(&contract.additional_references)
    .chain(intent.original_ticket.iter())
    .chain(intent.original_integration_ticket.iter())
    .chain(intent.original_inputs.files.values())
    .chain(&intent.existing_references)
    {
        ensure!(
            references.values.get(&reference.path) == Some(reference),
            "managed intent omitted a required immutable reference"
        );
    }
    retention::authenticate_reference(&intent.original_progress, MAX_BYTES)?;
    if let Some(reference) = &intent.original_ticket {
        retention::authenticate_reference(reference, MAX_BYTES)?;
    }
    if let Some(reference) = &intent.original_integration_ticket {
        retention::authenticate_reference(reference, MAX_BYTES)?;
    }
    let incremental = references
        .values
        .values()
        .filter(|reference| !intent.existing_references.contains(reference))
        .try_fold(0u64, |sum, reference| sum.checked_add(reference.bytes).context("managed intent extent overflow"))?;
    ensure!(
        incremental.checked_add(8 * MAX_BYTES).is_some_and(|bytes| bytes <= MANAGED_METADATA_RESERVATION),
        "managed retained intent exceeds reserved metadata before publication"
    );
    Ok(())
}

fn exact_integration_update(
    migration: &Migration,
    policy: &FrozenPolicy,
    intent: &AdmissionIntent,
    claim: assignment::Claim,
) -> Result<assignment::Update> {
    let root = publication_root(migration, policy, intent.completion.attempt)?;
    let completion = retention::seal_file(&root.join("completion.json"), MAX_BYTES)?;
    let authentication = retention::seal_file(&root.join("authentication.json"), MAX_BYTES)?;
    let cleanup = retention::seal_file(&root.join("cleanup.json"), MAX_BYTES)?;
    Ok(assignment::Update::Finished {
        claim,
        result: assignment::WorkResult::Integrated {
            submissions: vec![intent.original.claim.clone()],
            evidence: vec![artifact(&completion)],
            authentication: artifact(&authentication),
            cleanup: artifact(&cleanup),
        },
    })
}

/// No source publication precedes exact reconciliation of the original and
/// standard derived queue states. Unrelated ledger entries retain their sealed
/// original bytes; this publisher cannot silently adopt another result.
fn check_intent_queue(
    installed: &supervisor::Installation,
    migration: &Migration,
    intent: &AdmissionIntent,
    ledger: &LedgerView,
) -> Result<()> {
    let original_ledger: LedgerView = object(&intent.original_ledger.path)?;
    ensure!(ledger.capacity == original_ledger.capacity, "managed queue capacity changed");
    ensure!(ledger.schema == original_ledger.schema, "managed queue schema changed");
    let original = ledger.jobs.get(&intent.original.claim.id).context("managed original current claim missing")?;
    // A legacy Running implementation can precede the first normal Submitted
    // transition that installs the integrator. Keep that absence exact until
    // submission; an existing or subsequently installed owner must be current.
    let owner_matches = match (original_ledger.integration_owner.as_deref(), ledger.integration_owner.as_deref()) {
        (Some(frozen), Some(current)) => frozen == migration.coordinator && current == frozen,
        (None, None) => original.state == assignment::State::Running,
        (None, Some(current)) => {
            current == migration.coordinator
                && matches!(original.state, assignment::State::Submitted | assignment::State::Complete)
        }
        _ => false,
    };
    ensure!(owner_matches, "managed queue integration owner changed outside the original submission transition");
    let frozen =
        original_ledger.jobs.get(&intent.original.claim.id).context("managed original snapshot entry missing")?;
    ensure!(
        original.job == intent.original.job
            && original.claim.as_ref() == Some(&intent.original.claim)
            && original.agent == intent.original.agent
            && original.attempts == intent.original.claim.attempt
            && original.pending_job.is_none()
            && !original.stale
            && original.observations == intent.original.observations
            && original.adopted_ticket == frozen.adopted_ticket
            && sealed(&original.history)? == sealed(&frozen.history)?,
        "managed publication changed original current queue identity"
    );
    let derived = derived_job(intent)?;
    ensure!(
        ledger.jobs.keys().all(|id| original_ledger.jobs.contains_key(id) || id == &derived.id)
            && original_ledger.jobs.keys().all(|id| ledger.jobs.contains_key(id)),
        "managed publication changed unrelated queue membership"
    );
    for (id, entry) in &original_ledger.jobs {
        if id != &original.job.id && id != &derived.id {
            ensure!(sealed(entry)? == sealed(&ledger.jobs[id])?, "managed publication changed unrelated queue state");
        }
    }
    if original.state == assignment::State::Running {
        ensure!(
            sealed(original)? == sealed(frozen)?
                && ledger.jobs.get(&derived.id).map(sealed).transpose()?
                    == original_ledger.jobs.get(&derived.id).map(sealed).transpose()?,
            "managed running claim already contains a foreign publication"
        );
        return Ok(());
    }
    ensure!(
        matches!(original.state, assignment::State::Submitted | assignment::State::Complete)
            && original.last_update.as_ref() == Some(&intent.submitted)
            && matches!(&intent.submitted, assignment::Update::Finished { result, .. }
                if original.result.as_ref() == Some(result)),
        "managed interrupted submission differs from immutable intent"
    );
    let integration =
        ledger.jobs.get(&derived.id).context("managed submission lacks its standard derived integration")?;
    if intent.original.state == assignment::State::Submitted {
        ensure!(
            original.result == frozen.result && original.last_update == frozen.last_update,
            "imported publication changed the existing submitted result or update"
        );
        let frozen_integration =
            original_ledger.jobs.get(&derived.id).context("imported submission lacks original linked integration")?;
        let expected = if integration.state == assignment::State::Complete {
            Some(exact_integration_update(
                migration,
                &installed.policy,
                intent,
                integration.claim.clone().context("completed imported integration claim missing")?,
            )?)
        } else {
            None
        };
        return check_imported_integration(migration, original, frozen_integration, integration, expected.as_ref());
    }
    ensure!(
        integration.job == derived
            && integration.pending_job.is_none()
            && !integration.stale
            && integration.observations.is_empty()
            && integration.history.is_empty()
            && !integration.adopted_ticket,
        "managed interrupted derived integration changed"
    );
    match integration.state {
        assignment::State::Ready => ensure!(
            original.state == assignment::State::Submitted
                && integration.claim.is_none()
                && integration.agent.is_none()
                && integration.result.is_none()
                && integration.last_update.is_none()
                && integration.attempts == 0,
            "managed ready integration has an unexpected owner or result"
        ),
        assignment::State::Reserved | assignment::State::Running | assignment::State::Complete => {
            let claim = integration.claim.as_ref().context("managed held integration claim missing")?;
            ensure!(
                claim.coordinator == migration.coordinator
                    && claim.revision == derived.revision
                    && claim.attempt == integration.attempts
                    && claim.attempt == 1,
                "managed derived integration has another owner or attempt"
            );
            if integration.state == assignment::State::Reserved {
                ensure!(
                    original.state == assignment::State::Submitted
                        && integration.agent.is_none()
                        && integration.result.is_none()
                        && integration.last_update.is_none(),
                    "managed reserved integration acquired unexpected state"
                );
            } else {
                ensure!(
                    integration.agent.as_deref() == Some(&migration.session_id),
                    "managed integration real owner changed"
                );
                let expected = if integration.state == assignment::State::Running {
                    ensure!(
                        original.state == assignment::State::Submitted && integration.result.is_none(),
                        "running integration source changed"
                    );
                    assignment::Update::Started { claim: claim.clone(), agent: migration.session_id.clone() }
                } else {
                    ensure!(
                        original.state == assignment::State::Complete,
                        "managed completed integration did not complete original submission"
                    );
                    let update = exact_integration_update(migration, &installed.policy, intent, claim.clone())?;
                    let assignment::Update::Finished { result, .. } = &update else { unreachable!() };
                    ensure!(
                        integration.result.as_ref() == Some(result),
                        "managed completed integration result changed"
                    );
                    update
                };
                ensure!(
                    integration.last_update.as_ref() == Some(&expected),
                    "managed derived integration last event changed"
                );
            }
        }
        _ => bail!("managed derived integration is not publication-recoverable"),
    }
    Ok(())
}

fn check_imported_integration(
    migration: &Migration,
    submission: &assignment::Entry,
    frozen: &assignment::Entry,
    current: &assignment::Entry,
    completed: Option<&assignment::Update>,
) -> Result<()> {
    ensure!(
        current.job == frozen.job
            && current.pending_job.is_none()
            && !current.stale
            && current.observations == frozen.observations
            && sealed(&current.history)? == sealed(&frozen.history)?
            && current.adopted_ticket == frozen.adopted_ticket
            && frozen.result.is_none()
            && matches!(
                frozen.state,
                assignment::State::Ready | assignment::State::Reserved | assignment::State::Running
            ),
        "imported linked integration changed original job, history or observations"
    );
    if current.state == frozen.state {
        ensure!(
            submission.state == assignment::State::Submitted
                && current.claim == frozen.claim
                && current.agent == frozen.agent
                && current.result == frozen.result
                && current.last_update == frozen.last_update
                && current.attempts == frozen.attempts,
            "imported held integration was replaced or restarted"
        );
        return Ok(());
    }
    ensure!(
        matches!(current.state, assignment::State::Reserved | assignment::State::Running | assignment::State::Complete)
            && (frozen.state != assignment::State::Running || current.state == assignment::State::Complete)
            && (frozen.state != assignment::State::Reserved || current.state != assignment::State::Reserved),
        "imported integration made a non-forward transition"
    );
    let claim = current.claim.as_ref().context("imported integration claim missing")?;
    ensure!(
        claim.id == frozen.job.id
            && claim.revision == frozen.job.revision
            && claim.coordinator == migration.coordinator
            && claim.attempt == current.attempts
            && if let Some(original) = &frozen.claim {
                claim == original
            } else {
                claim.attempt == frozen.attempts + 1
            },
        "imported integration replaced its original attempt"
    );
    if current.state == assignment::State::Reserved {
        ensure!(
            frozen.state == assignment::State::Ready
                && current.agent.is_none()
                && current.result.is_none()
                && current.last_update.is_none()
                && submission.state == assignment::State::Submitted,
            "newly reserved imported integration acquired unexpected state"
        );
    } else {
        let agent = frozen.agent.as_deref().unwrap_or(&migration.session_id);
        ensure!(current.agent.as_deref() == Some(agent), "imported integration changed its original worker");
        let update = if current.state == assignment::State::Running {
            ensure!(
                current.result.is_none() && submission.state == assignment::State::Submitted,
                "imported running integration has a foreign result"
            );
            assignment::Update::Started { claim: claim.clone(), agent: agent.into() }
        } else {
            ensure!(
                submission.state == assignment::State::Complete,
                "imported integration did not complete submission"
            );
            let update = completed.context("imported integration lacks authenticated completion")?.clone();
            ensure!(
                matches!(&update, assignment::Update::Finished { claim: expected, .. } if expected == claim),
                "imported completion belongs to another integration claim"
            );
            let assignment::Update::Finished { result, .. } = &update else { unreachable!() };
            ensure!(current.result.as_ref() == Some(result), "imported integration result differs");
            update
        };
        ensure!(current.last_update.as_ref() == Some(&update), "imported integration last update differs");
    }
    Ok(())
}

fn controller_projection(
    installed: &supervisor::Installation,
    migration: &Migration,
    intent: &AdmissionIntent,
    accepted: Option<&AcceptedAdmission>,
) -> Result<ControllerProjection> {
    let ledger = current_ledger(migration)?;
    check_intent_queue(installed, migration, intent, &ledger)?;
    if let Some(plan) = &intent.progress {
        progress::validate_admitted_projection(&migration.checkout, plan, &intent.original_progress.path)?;
    }
    let root = publication_root(migration, &installed.policy, intent.completion.attempt)?;
    let derived = derived_job(intent)?;
    let original_ticket = intent
        .original_ticket
        .as_ref()
        .map(|reference| {
            retention::authenticate_reference(reference, MAX_BYTES)?;
            Ok::<_, anyhow::Error>(String::from_utf8(bytes(&reference.path, MAX_BYTES)?)?)
        })
        .transpose()?;
    let original_entry = &ledger.jobs[&intent.original.claim.id];
    let ticket = assignment::admitted_ticket_path(&migration.checkout, &migration.effort, &intent.original.job);
    let original_ledger: LedgerView = object(&intent.original_ledger.path)?;
    let mut original_lifecycle = vec![original_ledger.jobs[&intent.original.claim.id].clone()];
    if matches!(original_entry.state, assignment::State::Submitted | assignment::State::Complete) {
        let mut submitted = original_entry.clone();
        submitted.state = assignment::State::Submitted;
        original_lifecycle.push(submitted);
    }
    original_lifecycle.push(original_entry.clone());
    assignment::validate_admitted_ticket_lifecycle(
        &migration.checkout,
        &migration.effort,
        &original_lifecycle,
        original_ticket.as_deref(),
    )?;
    let derived_ticket = assignment::admitted_ticket_path(&migration.checkout, &migration.effort, &derived);
    if derived_ticket.exists() {
        let entry = ledger.jobs.get(&derived.id).context("unexpected derived ticket before original submission")?;
        let mut ready = entry.clone();
        ready.state = assignment::State::Ready;
        ready.claim = None;
        ready.agent = None;
        ready.result = None;
        ready.attempts = 0;
        ready.last_update = None;
        let mut lifecycle = vec![ready];
        if entry.claim.is_some() {
            let mut reserved = entry.clone();
            reserved.state = assignment::State::Reserved;
            reserved.agent = None;
            reserved.result = None;
            reserved.last_update = None;
            lifecycle.push(reserved);
        }
        if entry.agent.is_some() {
            let mut running = entry.clone();
            running.state = assignment::State::Running;
            running.result = None;
            lifecycle.push(running);
        }
        lifecycle.push(entry.clone());
        let original_integration_text = intent
            .original_integration_ticket
            .as_ref()
            .map(|reference| {
                retention::authenticate_reference(reference, MAX_BYTES)?;
                Ok::<_, anyhow::Error>(String::from_utf8(bytes(&reference.path, MAX_BYTES)?)?)
            })
            .transpose()?;
        assignment::validate_admitted_ticket_lifecycle(
            &migration.checkout,
            &migration.effort,
            &lifecycle,
            original_integration_text.as_deref(),
        )?;
    }
    let mut paths = BTreeSet::from([
        migration.checkout.join(format!(".scratch/{}/ledger.toml", migration.effort)),
        ticket,
        derived_ticket,
        migration.checkout.join(".scratch/decomp-progress/registry.json"),
        migration.checkout.join(".scratch/decomp-progress/index.html"),
        root.join("original-ledger.toml"),
        root.join("original-progress.json"),
        root.join("original-ticket.md"),
        root.join("original-integration-ticket.md"),
        root.join("completion.json"),
        root.join("authentication.json"),
        root.join("cleanup.json"),
        root.join("intent.json"),
        root.join("draft.json"),
        root.join("accepted.json"),
        root.join("accepted.pending"),
    ]);
    paths.extend(intent.original_sources.values().map(|reference| reference.path.clone()));
    paths.extend(intent.original_inputs.files.values().map(|reference| reference.path.clone()));
    let immutable = intent
        .references
        .iter()
        .filter(|reference| reference.path.starts_with(&root))
        .map(|reference| (&reference.path, reference))
        .collect::<BTreeMap<_, _>>();
    let mut references = Vec::new();
    for path in &paths {
        if path.exists() {
            let reference = retention::seal_file(path, MAX_BYTES)?;
            if let Some(expected) = immutable.get(path) {
                ensure!(reference == **expected, "managed controller immutable projection changed");
            } else if path == &root.join("intent.json") {
                ensure!(object::<AdmissionIntent>(path)? == *intent, "managed intent projection changed");
            } else if path == &root.join("accepted.json") || path == &root.join("accepted.pending") {
                let expected = accepted.context("unexpected accepted metadata before authenticated publication")?;
                ensure!(
                    json_record::<AcceptedAdmission>(path)? == *expected,
                    "managed accepted controller projection changed"
                );
                if path.extension().is_some_and(|extension| extension == "pending") {
                    ensure!(
                        bytes(path, MAX_BYTES)? == serde_json::to_vec(expected)?,
                        "managed accepted pending is partial or changed"
                    );
                }
            } else if path == &root.join("original-ticket.md") || path == &root.join("original-integration-ticket.md") {
                bail!("unexpected original ticket snapshot");
            }
            references.push(reference);
        }
    }
    // The selected immutable root contains only enumerated files. Its parent
    // assignment lock and unrelated effort files earn no output capability.
    fn inspect(path: &Path, allowed: &BTreeSet<PathBuf>, count: &mut usize) -> Result<()> {
        for item in std::fs::read_dir(path)? {
            *count += 1;
            ensure!(*count <= MAX_REFERENCES, "managed controller directory exceeds bounded inventory");
            let path = item?.path();
            let metadata = path.symlink_metadata()?;
            ensure!(!metadata.file_type().is_symlink(), "managed controller directory crosses a symlink");
            if metadata.is_dir() {
                ensure!(allowed.iter().any(|file| file.starts_with(&path)), "unexpected managed controller directory");
                inspect(&path, allowed, count)?;
            } else {
                ensure!(metadata.is_file() && allowed.contains(&path), "undeclared managed controller output");
            }
        }
        Ok(())
    }
    inspect(&root, &paths, &mut 0)?;
    let preserved = crate::translation_continuation::preserved(installed)?
        .into_iter()
        .filter(|reference| !paths.contains(&reference.path))
        .collect();
    Ok(ControllerProjection { paths: paths.into_iter().collect(), references, preserved })
}

pub(crate) fn publish(
    installed: &supervisor::Installation,
    completion: &Completion,
    _stages: &Path,
) -> Result<ManagedPublication> {
    crate::inputs::bounded::limited(|| publish_inner(installed, completion, _stages))
}

fn publish_inner(
    installed: &supervisor::Installation,
    completion: &Completion,
    _stages: &Path,
) -> Result<ManagedPublication> {
    let authenticated = supervisor::authenticate_managed_completion(installed, completion)?;
    let (migration_path, migration, migrated) = managed_migration(installed)?;
    let _lock = assignment::mutation_lock(&migration.checkout)?;
    current_role(&migration)?;
    let root = publication_root(&migration, &installed.policy, completion.attempt)?;
    ensure!(
        completion
            .finalization
            .retained_bytes
            .checked_add(MANAGED_METADATA_RESERVATION)
            .is_some_and(|bytes| bytes <= installed.policy.budgets.max_evidence_bytes),
        "managed publication metadata reservation exhausted before any side effect"
    );
    directory(&root)?;
    let accepted_path = root.join("accepted.json");
    let pending_accepted = accepted_path.with_extension("pending");
    if accepted_path.exists() || pending_accepted.exists() {
        let accepted: AcceptedAdmission =
            if accepted_path.exists() { object(&accepted_path)? } else { json_record(&pending_accepted)? };
        ensure!(accepted.completion == *completion, "managed result retry differs from accepted completion");
        authenticate_accepted(installed, &migration, &accepted, false)?;
        if pending_accepted.exists() {
            write(&accepted_path, &accepted)?;
        }
        repair_accepted(installed, &migration, &accepted)?;
        authenticate_accepted(installed, &migration, &accepted, true)?;
        return publication(&accepted_path, &accepted, &installed.policy);
    }
    let intent_path = root.join("intent.json");
    let pending_intent = intent_path.with_extension("pending");
    let intent = if intent_path.exists() || pending_intent.exists() {
        let intent: AdmissionIntent =
            if intent_path.exists() { object(&intent_path)? } else { json_record(&pending_intent)? };
        ensure!(
            intent.completion == *completion
                && intent.policy_sha256 == sealed(&installed.policy)?
                && intent.domain_sha256 == domain_identity(&migration)?,
            "managed publication intent identity changed"
        );
        for reference in &intent.references {
            retention::authenticate_reference(reference, MAX_RETAINED_BYTES)?;
        }
        authenticate_intent(installed, &migration, &intent, &authenticated)?;
        if pending_intent.exists() {
            write(&intent_path, &intent)?;
        }
        intent
    } else {
        let mut references = References::new();
        current_authority(&migration, &mut references)?;
        let ledger = current_ledger(&migration)?;
        let entry = ledger.jobs.get(&migrated.claim.id).context("original managed claim missing")?;
        references.add(&migration.ledger, MAX_BYTES)?;
        let original = original_lineage(&migration, &migrated, entry, &mut references)?;
        let stops = migration
            .stopped_owners
            .iter()
            .map(|reference| {
                references.add(reference, MAX_BYTES)?;
                object::<OriginalOwnerStop>(&reference.path)
            })
            .collect::<Result<Vec<_>>>()?;
        stopped_owner(&migration, &original, &stops, &mut references)?;
        let contract = native_admission(&migration, &migrated, &original, installed, true)?;
        completion_gates(installed, completion, &authenticated, &contract)?;
        let original_ticket_path =
            assignment::admitted_ticket_path(&migration.checkout, &migration.effort, &original.job);
        let projected_reference = |path: PathBuf, source: &Path| -> Result<Reference> {
            let value = bytes(source, MAX_BYTES)?;
            Ok(Reference { path, sha256: ::translation_execution::model::sha256(&value), bytes: value.len() as u64 })
        };
        let draft = AdmissionDraft {
            schema: "translation-managed-publication-draft-v1".into(),
            domain_sha256: domain_identity(&migration)?,
            policy_sha256: completion.policy_sha256.clone(),
            completion: completion.clone(),
            original: original.clone(),
            original_ledger: Reference { path: root.join("original-ledger.toml"), ..migration.ledger.clone() },
            original_sources: original
                .source_files
                .keys()
                .map(|name| {
                    Ok((
                        name.clone(),
                        projected_reference(
                            root.join("original-source").join(name),
                            &original.source_checkout.join(name),
                        )?,
                    ))
                })
                .collect::<Result<_>>()?,
            original_inputs: projected_job_inputs(&migration.checkout, &original.job, &root)?,
            original_ticket: original_ticket_path
                .exists()
                .then(|| projected_reference(root.join("original-ticket.md"), &original_ticket_path))
                .transpose()?,
            original_integration_ticket: original_integration_ticket(&migration, &original)?
                .map(|path| projected_reference(root.join("original-integration-ticket.md"), &path))
                .transpose()?,
            original_progress: projected_reference(
                root.join("original-progress.json"),
                &migration.checkout.join(".scratch/decomp-progress/registry.json"),
            )?,
            contract: contract.clone(),
            existing_references: authenticated.references.clone(),
        };
        let draft_path = root.join("draft.json");
        if draft_path.with_extension("pending").exists() {
            ensure!(
                json_record::<AdmissionDraft>(&draft_path.with_extension("pending"))? == draft,
                "managed pending draft changed before replay"
            );
        }
        let draft_reference = write(&draft_path, &draft)?;
        references.add(&draft_reference, MAX_BYTES)?;
        persist_job_inputs(&migration.checkout, &original.job, &root, &draft.original_inputs)?;
        for reference in draft.original_inputs.files.values() {
            references.add(reference, MAX_BYTES)?;
        }
        let original_ledger =
            write_bytes(&root.join("original-ledger.toml"), &bytes(&migration.ledger.path, MAX_BYTES)?)?;
        ensure!(original_ledger.sha256 == migration.ledger.sha256, "original ledger snapshot changed");
        references.values.remove(&migration.ledger.path);
        references.bytes -= migration.ledger.bytes;
        references.add(&original_ledger, MAX_BYTES)?;
        let mut original_sources = BTreeMap::new();
        for (name, expected) in &original.source_files {
            let original_path = original.source_checkout.join(name);
            let snapshot = write_bytes(&root.join("original-source").join(name), &bytes(&original_path, MAX_BYTES)?)?;
            ensure!(snapshot.sha256 == *expected, "original source changed before managed publication");
            if let Some(reference) = references.values.remove(&original_path) {
                references.bytes -= reference.bytes;
            }
            references.add(&snapshot, MAX_BYTES)?;
            original_sources.insert(name.clone(), snapshot);
        }
        let original_ticket_path =
            assignment::admitted_ticket_path(&migration.checkout, &migration.effort, &original.job);
        let original_ticket = if original_ticket_path.exists() {
            let snapshot = write_bytes(&root.join("original-ticket.md"), &bytes(&original_ticket_path, MAX_BYTES)?)?;
            references.add(&snapshot, MAX_BYTES)?;
            Some(snapshot)
        } else {
            None
        };
        let original_integration_ticket = original_integration_ticket(&migration, &original)?
            .map(|path| {
                let reference = write_bytes(&root.join("original-integration-ticket.md"), &bytes(&path, MAX_BYTES)?)?;
                references.add(&reference, MAX_BYTES)?;
                Ok::<_, anyhow::Error>(reference)
            })
            .transpose()?;
        let original_progress = write_bytes(
            &root.join("original-progress.json"),
            &bytes(&migration.checkout.join(".scratch/decomp-progress/registry.json"), MAX_BYTES)?,
        )?;
        references.add(&original_progress, MAX_BYTES)?;
        references.add(&retention::seal_file(&migration_path, MAX_BYTES)?, MAX_BYTES)?;
        references.add(&contract.scope, MAX_BYTES)?;
        for reference in contract.inventory.iter().chain(&contract.additional_references) {
            references.add(reference, MAX_BYTES)?;
        }
        for reference in &authenticated.references {
            references.add(reference, MAX_RETAINED_BYTES)?;
        }
        let completion_reference = write(&root.join("completion.json"), completion)?;
        references.add(&completion_reference, MAX_BYTES)?;
        let authentication = write(
            &root.join("authentication.json"),
            &authentication_value(&original, completion, &contract, &completion_reference),
        )?;
        references.add(&authentication, MAX_BYTES)?;
        let cleanup = write(
            &root.join("cleanup.json"),
            &cleanup_value(&original, completion, &authenticated, &completion_reference),
        )?;
        references.add(&cleanup, MAX_BYTES)?;
        let submitted = submission_update(&original, completion, &completion_reference, &authentication)?;
        let evidence = [
            (completion_reference.path.clone(), completion_reference.sha256.clone()),
            (authentication.path.clone(), authentication.sha256.clone()),
            (cleanup.path.clone(), cleanup.sha256.clone()),
        ];
        let progress = if contract.domain.as_deref() == Some("legacy-infrastructure-v1") {
            None
        } else {
            Some(progress::prepare_admitted(
                &migration.checkout,
                &sealed(&(completion.run_id.clone(), completion.attempt))?,
                &migrated.scope_id,
                &original.agent.clone().unwrap(),
                &contract.routines,
                contract.source_revision()?,
                &contract.inventory()?.sha256,
                &completion.candidate.commit,
                &completion.integrated.commit,
                &evidence,
                &contract.limitations,
            )?)
        };
        references.all()?;
        let intent = AdmissionIntent {
            schema: "translation-managed-publication-intent-v1".into(),
            domain_sha256: domain_identity(&migration)?,
            policy_sha256: completion.policy_sha256.clone(),
            completion: completion.clone(),
            original,
            original_ledger,
            original_sources,
            original_inputs: draft.original_inputs,
            original_ticket,
            original_integration_ticket,
            original_progress,
            contract,
            references: references.values.into_values().collect(),
            existing_references: authenticated.references.clone(),
            submitted,
            progress,
        };
        let incremental = intent
            .references
            .iter()
            .filter(|reference| !intent.existing_references.contains(reference))
            .try_fold(0u64, |sum, reference| {
                sum.checked_add(reference.bytes).context("managed metadata extent overflow")
            })?;
        ensure!(
            incremental.checked_add(8 * MAX_BYTES).is_some_and(|bytes| bytes <= MANAGED_METADATA_RESERVATION),
            "managed frozen original metadata exceeds publication reservation before queue/source mutation"
        );
        write(&intent_path, &intent)?;
        intent
    };
    authenticate_intent(installed, &migration, &intent, &authenticated)?;
    let original_ledger: LedgerView = object(&intent.original_ledger.path)?;
    let mut request = request(&migration, original_ledger.capacity);
    current_role(&migration)?;
    let projections = controller_projection(installed, &migration, &intent, None)?;
    let source_publication = supervisor::publish_managed_source(installed, completion, &authenticated, &projections)?;
    current_role(&migration)?;
    if intent.original.state != assignment::State::Submitted {
        request.update = Some(intent.submitted.clone());
        assignment::record_admitted_locked(&request, &intent.original.claim, Some(&intent.original_inputs))?;
    }
    let integration_claim = if matches!(&installed.contract, supervisor::Contract::Original { contract }
        if contract.original_claim == migrated.original_claim)
        && migrated.execution == Some(MigrationExecution::ImportedSubmission)
        && same_claim(&migrated.claim, &intent.original.claim)
    {
        assignment::reserve_imported_integration_locked(
            &request,
            &intent.original.claim,
            &intent.original.job,
            intent.original.result.as_ref().context("imported original Submitted result missing")?,
        )?
    } else {
        assignment::reserve_admitted_integration_locked(&request, &intent.original.claim)?
    };
    let ledger = current_ledger(&migration)?;
    let integration_entry = ledger.jobs.get(&integration_claim.id).context("derived integration missing")?;
    if integration_entry.state == assignment::State::Reserved {
        request.update =
            Some(assignment::Update::Started { claim: integration_claim.clone(), agent: migration.session_id.clone() });
        assignment::record_admitted_locked(&request, &integration_claim, None)?;
    }
    let authentication = retention::seal_file(&root.join("authentication.json"), MAX_BYTES)?;
    let cleanup = retention::seal_file(&root.join("cleanup.json"), MAX_BYTES)?;
    let completion_reference = retention::seal_file(&root.join("completion.json"), MAX_BYTES)?;
    let integration = assignment::Update::Finished {
        claim: integration_claim.clone(),
        result: assignment::WorkResult::Integrated {
            submissions: vec![intent.original.claim.clone()],
            evidence: vec![artifact(&completion_reference)],
            authentication: artifact(&authentication),
            cleanup: artifact(&cleanup),
        },
    };
    request.update = Some(integration.clone());
    assignment::record_admitted_locked(&request, &integration_claim, None)?;
    let observation_ids = intent
        .progress
        .as_ref()
        .map(|plan| progress::publish_admitted(&migration.checkout, plan))
        .transpose()?
        .unwrap_or_default();
    let completed_projection = matches!(&installed.contract, supervisor::Contract::Original { .. })
        .then(|| controller_projection(installed, &migration, &intent, None))
        .transpose()?;
    let intent_reference = retention::seal_file(&intent_path, MAX_BYTES)?;
    let mut references = intent.references.clone();
    references.push(intent_reference.clone());
    references.extend(source_publication);
    let accepted = AcceptedAdmission {
        schema: "translation-managed-publication-v1".into(),
        domain_sha256: intent.domain_sha256,
        policy_sha256: intent.policy_sha256,
        completion: completion.clone(),
        intent: intent_reference,
        submission: intent.submitted,
        integration,
        references,
        existing_references: intent.existing_references,
        observation_ids,
        completed_projection,
    };
    let proposed = publication_value(&root.join("accepted.json"), &accepted, &installed.policy)?;
    ensure!(
        completion
            .finalization
            .retained_bytes
            .checked_add(proposed.retained_bytes)
            .is_some_and(|total| total <= installed.policy.budgets.max_evidence_bytes),
        "managed publication aggregate evidence budget exceeded"
    );
    write(&accepted_path, &accepted)?;
    authenticate_accepted(installed, &migration, &accepted, true)?;
    publication(&accepted_path, &accepted, &installed.policy)
}

fn artifact(reference: &Reference) -> assignment::Artifact {
    assignment::Artifact { path: reference.path.clone(), sha256: reference.sha256.clone() }
}

fn publication_value(path: &Path, accepted: &AcceptedAdmission, policy: &FrozenPolicy) -> Result<ManagedPublication> {
    let value = serde_json::to_vec(accepted)?;
    let reference = Reference {
        path: path.into(),
        sha256: ::translation_execution::model::sha256(&value),
        bytes: value.len() as u64,
    };
    let mut refs = BTreeMap::new();
    refs.insert(reference.path.clone(), reference.clone());
    for item in &accepted.references {
        if let Some(previous) = refs.insert(item.path.clone(), item.clone()) {
            ensure!(previous == *item, "managed reference collision");
        }
    }
    let existing =
        accepted.existing_references.iter().map(|reference| (&reference.path, reference)).collect::<BTreeMap<_, _>>();
    for (path, reference) in &existing {
        ensure!(refs.get(*path) == Some(*reference), "managed existing finalization reference differs");
    }
    let retained_bytes = refs
        .values()
        .filter(|reference| !existing.contains_key(&reference.path))
        .try_fold(0u64, |sum, reference| {
            sum.checked_add(reference.bytes).context("managed publication extent overflow")
        })?;
    ensure!(
        retained_bytes <= policy.budgets.max_evidence_bytes && refs.len() <= MAX_REFERENCES,
        "managed publication retained closure budget exceeded"
    );
    Ok(ManagedPublication {
        accepted: reference,
        references: refs.into_values().collect(),
        observation_ids: accepted.observation_ids.clone(),
        retained_bytes,
    })
}

fn publication(path: &Path, accepted: &AcceptedAdmission, policy: &FrozenPolicy) -> Result<ManagedPublication> {
    let publication = publication_value(path, accepted, policy)?;
    for reference in &publication.references {
        retention::authenticate_reference(reference, MAX_RETAINED_BYTES)?;
    }
    Ok(publication)
}

fn authenticate_accepted(
    installed: &supervisor::Installation,
    migration: &Migration,
    accepted: &AcceptedAdmission,
    projections: bool,
) -> Result<()> {
    authenticate_accepted_with_history(installed, migration, accepted, projections, None)
}

fn authenticate_accepted_with_history(
    installed: &supervisor::Installation,
    migration: &Migration,
    accepted: &AcceptedAdmission,
    projections: bool,
    history: Option<(&Reference, &str)>,
) -> Result<()> {
    ensure!(
        accepted.schema == "translation-managed-publication-v1"
            && accepted.domain_sha256 == domain_identity(migration)?
            && accepted.policy_sha256 == sealed(&installed.policy)?
            && accepted.completion.policy_sha256 == accepted.policy_sha256,
        "managed accepted publication identity changed"
    );
    let authenticated = supervisor::authenticate_managed_completion(installed, &accepted.completion)?;
    let intent: AdmissionIntent = object(&accepted.intent.path)?;
    retention::authenticate_reference(&accepted.intent, MAX_BYTES)?;
    ensure!(
        accepted.intent.path
            == publication_root(migration, &installed.policy, accepted.completion.attempt)?.join("intent.json"),
        "managed accepted intent belongs to another publication root"
    );
    ensure!(
        intent.completion == accepted.completion
            && intent.submitted == accepted.submission
            && intent.domain_sha256 == accepted.domain_sha256
            && intent.policy_sha256 == accepted.policy_sha256,
        "managed accepted publication differs from original intent"
    );
    authenticate_intent(installed, migration, &intent, &authenticated)?;
    let graph = accepted.references.iter().map(|reference| (&reference.path, reference)).collect::<BTreeMap<_, _>>();
    ensure!(graph.len() == accepted.references.len(), "managed accepted reference closure is duplicated");
    for reference in intent.references.iter().chain(std::iter::once(&accepted.intent)) {
        ensure!(
            graph.get(&reference.path) == Some(&reference),
            "managed accepted closure omitted its immutable intent graph"
        );
    }
    let source_root = installed
        .state_root
        .join(&accepted.completion.run_id)
        .join(format!("attempt-{:04}", accepted.completion.attempt))
        .join("managed-source-publication");
    for name in ["intent.json", "index-lock.marker", "result.json"] {
        let path = source_root.join(name);
        let reference = retention::seal_file(&path, MAX_BYTES)?;
        ensure!(graph.get(&path) == Some(&&reference), "managed accepted closure omitted source publication evidence");
    }
    let source_intent: serde_json::Value = object(&source_root.join("intent.json"))?;
    let source_result: serde_json::Value = object(&source_root.join("result.json"))?;
    let source_intent_reference = retention::seal_file(&source_root.join("intent.json"), MAX_BYTES)?;
    let context = ::translation_execution::OperationContext {
        run_id: accepted.completion.run_id.clone(),
        attempt: accepted.completion.attempt,
        policy_sha256: accepted.completion.policy_sha256.clone(),
        phase: ::translation_execution::ExecutionPhase::Publication,
        operation_key: ::translation_execution::model::sha256(format!(
            "{}:{}:managed-source-publication",
            accepted.completion.run_id, accepted.completion.attempt
        )),
    };
    ensure!(
        source_intent.get("schema").and_then(serde_json::Value::as_str)
            == Some("translation-source-publication-intent-v1")
            && source_result.get("schema").and_then(serde_json::Value::as_str)
                == Some("translation-source-publication-result-v1")
            && source_intent.get("repository") == Some(&serde_json::to_value(&migration.checkout)?)
            && source_intent.get("context") == Some(&serde_json::to_value(&context)?)
            && source_result.get("context") == Some(&serde_json::to_value(&context)?)
            && source_intent.get("candidate")
                == Some(&serde_json::to_value(&authenticated.candidate_revision.manifest)?)
            && source_intent.get("integrated")
                == Some(&serde_json::to_value(&authenticated.integrated_revision.manifest)?)
            && source_intent.get("baseline").and_then(serde_json::Value::as_str)
                == Some(installed.integration_baseline.as_str())
            && source_intent.get("source") == Some(&serde_json::to_value(&accepted.completion.integrated)?)
            && source_result.get("intent") == Some(&serde_json::to_value(source_intent_reference)?)
            && source_result.get("candidate") == Some(&serde_json::to_value(&accepted.completion.candidate)?)
            && source_result.get("integrated") == Some(&serde_json::to_value(&accepted.completion.integrated)?)
            && accepted.observation_ids
                == intent.progress.as_ref().map(progress::admitted_observation_ids).unwrap_or_default(),
        "managed accepted source publication or observation graph changed"
    );
    if matches!(&installed.contract, supervisor::Contract::Original { .. }) {
        let projection = accepted
            .completed_projection
            .as_ref()
            .context("original completion lacks sealed final controller projections")?;
        let mut paths = projection.paths.clone();
        paths.sort();
        ensure!(
            source_intent.get("controller_paths") == Some(&serde_json::to_value(&paths)?)
                && source_intent.get("preserved") == Some(&serde_json::to_value(&projection.preserved)?)
                && projection.references.len() <= MAX_REFERENCES,
            "original completed projection changed source publication ownership"
        );
        let mut observed = BTreeSet::new();
        for reference in &projection.references {
            ensure!(
                paths.binary_search(&reference.path).is_ok()
                    && observed.insert(&reference.path)
                    && reference.path != accepted.intent.path.parent().unwrap().join("accepted.json")
                    && reference.path != accepted.intent.path.parent().unwrap().join("accepted.pending"),
                "original completed projection contains a foreign, repeated or circular reference"
            );
        }
    } else {
        ensure!(accepted.completed_projection.is_none(), "foreign original continuation projection");
    }
    let migrated = migration
        .claims
        .iter()
        .find(|claim| claim.claim == installed.policy.claim)
        .context("accepted original migration claim missing")?;
    retention::authenticate_reference(&migrated.original_claim, MAX_BYTES)?;
    let original: OriginalClaim = object(&migrated.original_claim.path)?;
    let ledger_original: LedgerView = object(&intent.original_ledger.path)?;
    let original_entry =
        ledger_original.jobs.get(&original.claim.id).context("accepted original ledger snapshot claim missing")?;
    ensure!(
        original == intent.original
            && original_entry.job == original.job
            && original_entry.claim.as_ref() == Some(&original.claim)
            && original_entry.agent == original.agent
            && original_entry.state == original.state
            && original_entry.result == original.result
            && original_entry.observations == original.observations
            && !original_entry.stale
            && original_entry.pending_job.is_none()
            && intent.original_ledger.sha256 == migration.ledger.sha256
            && intent.original_ledger.bytes == migration.ledger.bytes,
        "accepted admission changed original immutable job/claim/source lineage"
    );
    ensure!(
        intent.original_sources.keys().collect::<BTreeSet<_>>() == intent.original.source_files.keys().collect(),
        "accepted original source snapshot closure changed"
    );
    for (name, reference) in &intent.original_sources {
        retention::authenticate_reference(reference, MAX_BYTES)?;
        ensure!(
            intent.original.source_files.get(name) == Some(&reference.sha256)
                && ::translation_execution::model::sha256(&git_bytes(
                    &migration.checkout,
                    &["show", &format!("{}:{name}", intent.original.source_commit)]
                )?) == reference.sha256,
            "accepted original source bytes or original commit identity changed"
        );
    }
    for reference in &accepted.references {
        retention::authenticate_reference(reference, MAX_RETAINED_BYTES)?;
    }
    ensure!(
        accepted.existing_references == authenticated.references
            && intent.existing_references == accepted.existing_references,
        "managed existing finalization closure identity changed"
    );
    let ledger = if let Some((reference, head)) = history {
        retention::authenticate_reference(reference, MAX_BYTES)?;
        ensure!(
            head == accepted.completion.integrated.commit,
            "historical publication source differs from its accepted integration"
        );
        object(&reference.path)?
    } else {
        current_ledger(migration)?
    };
    check_intent_queue(installed, migration, &intent, &ledger)?;
    for update in [&accepted.submission, &accepted.integration] {
        let assignment::Update::Finished { claim, result } = update else {
            bail!("accepted queue update is not a result")
        };
        let entry = ledger.jobs.get(&claim.id).context("accepted original queue claim missing")?;
        ensure!(
            entry.claim.as_ref() == Some(claim)
                && entry.job.revision == claim.revision
                && entry.attempts == claim.attempt
                && entry.pending_job.is_none()
                && !entry.stale
                && entry.state == assignment::State::Complete
                && entry.result.as_ref() == Some(result)
                && entry.last_update.as_ref() == Some(update),
            "accepted queue original claim/result identity changed"
        );
        for artifact in assignment::result_artifacts(result) {
            ensure!(digest(&artifact.path)? == artifact.sha256, "accepted queue evidence changed");
        }
        if claim == &intent.original.claim {
            ensure!(
                entry.agent == intent.original.agent && entry.job == intent.original.job,
                "accepted queue original owner or job changed"
            );
        }
        if projections {
            assignment::authenticate_admitted_ticket(&migration.checkout, &migration.effort, entry)?;
        }
    }
    let accepted_path =
        publication_root(migration, &installed.policy, accepted.completion.attempt)?.join("accepted.json");
    let accepted_path = if accepted_path.exists() { accepted_path } else { accepted_path.with_extension("pending") };
    ensure!(
        json_record::<AcceptedAdmission>(&accepted_path)? == *accepted,
        "managed accepted controller projection changed"
    );
    if history.is_none() {
        let controller = controller_projection(installed, migration, &intent, Some(accepted))?;
        if let Some(expected) = &accepted.completed_projection {
            let mut completed = controller.clone();
            completed.references.retain(|reference| {
                reference.path != accepted_path && reference.path != accepted_path.with_extension("pending")
            });
            ensure!(completed == *expected, "original completed controller projection changed");
        }
        supervisor::authenticate_managed_source(installed, &accepted.completion, &authenticated, &controller)?;
    }
    if projections {
        if let Some(plan) = &intent.progress {
            progress::authenticate_admitted(&migration.checkout, plan)?;
        }
    }
    if history.is_none() {
        current_role(migration)?;
    }
    Ok(())
}

pub(crate) fn capture_publication(
    installed: &supervisor::Installation,
    completion: &Completion,
) -> Result<(Reference, ControllerProjection)> {
    let (_, migration, _) = managed_migration(installed)?;
    let _lock = assignment::mutation_lock(&migration.checkout)?;
    let path = publication_root(&migration, &installed.policy, completion.attempt)?.join("accepted.json");
    let reference = retention::seal_file(&path, MAX_BYTES)?;
    let accepted: AcceptedAdmission = object(&path)?;
    ensure!(accepted.completion == *completion, "terminal capture changed accepted completion");
    authenticate_accepted(installed, &migration, &accepted, true)?;
    let intent: AdmissionIntent = object(&accepted.intent.path)?;
    Ok((reference, controller_projection(installed, &migration, &intent, Some(&accepted))?))
}

/// This authenticates a sealed past terminal queue, not the current queue. It
/// cannot admit source publication or repair mutable projections after a later
/// original job has advanced them.
pub(crate) fn authenticate_frozen_publication(
    installed: &supervisor::Installation,
    accepted: &Reference,
    ledger: &Reference,
    head: &str,
) -> Result<ControllerProjection> {
    let supervisor::Admission::Managed { checkout, migration: asset, .. } = &installed.admission else {
        bail!("historical original publication requires managed lineage");
    };
    let path = if asset.file.is_absolute() { asset.file.clone() } else { checkout.join(&asset.file) };
    ensure!(digest(&path)? == asset.sha256, "historical migration changed");
    let migration: Migration = read(&path)?;
    retention::authenticate_reference(accepted, MAX_BYTES)?;
    let value: AcceptedAdmission = object(&accepted.path)?;
    ensure!(
        accepted.path
            == publication_root(&migration, &installed.policy, value.completion.attempt)?.join("accepted.json"),
        "historical accepted publication location changed"
    );
    authenticate_accepted_with_history(installed, &migration, &value, false, Some((ledger, head)))?;
    let mut projection = value
        .completed_projection
        .context("original terminal lacks its authenticated completed controller projection")?;
    projection.references.push(accepted.clone());
    projection.references.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(projection)
}

fn repair_accepted(
    installed: &supervisor::Installation,
    migration: &Migration,
    accepted: &AcceptedAdmission,
) -> Result<()> {
    authenticate_accepted(installed, migration, accepted, false)?;
    let intent: AdmissionIntent = object(&accepted.intent.path)?;
    let ledger = current_ledger(migration)?;
    let request = request(migration, ledger.capacity);
    let claims = [&accepted.submission, &accepted.integration]
        .into_iter()
        .map(|update| {
            let assignment::Update::Finished { claim, .. } = update else { unreachable!() };
            claim.clone()
        })
        .collect::<Vec<_>>();
    assignment::repair_admitted_tickets_locked(&request, &claims)?;
    if let Some(plan) = &intent.progress
        && progress::authenticate_admitted(&migration.checkout, plan).is_err()
    {
        ensure!(
            progress::publish_admitted(&migration.checkout, plan)? == accepted.observation_ids,
            "managed accepted progress identity changed during repair"
        );
    }
    Ok(())
}

pub(crate) fn authenticate_publication(
    installed: &supervisor::Installation,
    completion: &Completion,
    publication: &ManagedPublication,
) -> Result<()> {
    crate::inputs::bounded::limited(|| authenticate_publication_inner(installed, completion, publication))
}

fn authenticate_publication_inner(
    installed: &supervisor::Installation,
    completion: &Completion,
    publication: &ManagedPublication,
) -> Result<()> {
    if let Some(terminal) = crate::translation_continuation::historical(installed)? {
        ensure!(
            terminal.accepted == publication.accepted
                && matches!(&terminal.view.state, ::translation_execution::model::State::Complete { completion: saved, .. } if **saved == *completion),
            "historical publication changed its exact accepted completion"
        );
        let accepted: AcceptedAdmission = object(&terminal.accepted.path)?;
        ensure!(
            publication_value(&terminal.accepted.path, &accepted, &installed.policy)? == *publication,
            "historical retained publication identity changed"
        );
        return Ok(());
    }
    let (_, migration, _) = managed_migration(installed)?;
    let _lock = assignment::mutation_lock(&migration.checkout)?;
    current_role(&migration)?;
    ensure!(
        publication.accepted.path
            == publication_root(&migration, &installed.policy, completion.attempt)?.join("accepted.json"),
        "managed retained acceptance belongs to another publication root"
    );
    retention::authenticate_reference(&publication.accepted, MAX_BYTES)?;
    let accepted: AcceptedAdmission = object(&publication.accepted.path)?;
    ensure!(
        accepted.completion == *completion
            && publication_value(&publication.accepted.path, &accepted, &installed.policy)? == *publication,
        "managed retained publication identity changed"
    );
    authenticate_accepted(installed, &migration, &accepted, true)
}

pub(crate) fn resume_publication(
    installed: &supervisor::Installation,
    completion: &Completion,
    publication: &ManagedPublication,
) -> Result<()> {
    crate::inputs::bounded::limited(|| resume_publication_inner(installed, completion, publication))
}

fn resume_publication_inner(
    installed: &supervisor::Installation,
    completion: &Completion,
    publication: &ManagedPublication,
) -> Result<()> {
    if crate::translation_continuation::historical(installed)?.is_some() {
        return authenticate_publication_inner(installed, completion, publication);
    }
    let (_, migration, _) = managed_migration(installed)?;
    let _lock = assignment::mutation_lock(&migration.checkout)?;
    current_role(&migration)?;
    ensure!(
        publication.accepted.path
            == publication_root(&migration, &installed.policy, completion.attempt)?.join("accepted.json"),
        "managed retained acceptance belongs to another publication root"
    );
    // A lost rename can leave the original accepted bytes at the pending name.
    // Bind those actual bytes to the retained publication before interpreting
    // them; neither a changed record nor an absent acceptance authorizes repair.
    let reference = match publication.accepted.path.symlink_metadata() {
        Ok(_) => publication.accepted.clone(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Reference { path: publication.accepted.path.with_extension("pending"), ..publication.accepted.clone() }
        }
        Err(error) => return Err(error.into()),
    };
    retention::authenticate_reference(&reference, MAX_BYTES)?;
    let accepted: AcceptedAdmission = json_record(&reference.path)?;
    ensure!(
        accepted.completion == *completion
            && publication_value(&publication.accepted.path, &accepted, &installed.policy)? == *publication,
        "managed retained publication retry differs"
    );
    authenticate_accepted(installed, &migration, &accepted, false)?;
    ensure!(
        write(&publication.accepted.path, &accepted)? == publication.accepted,
        "managed accepted pending promotion changed retained identity"
    );
    repair_accepted(installed, &migration, &accepted)?;
    authenticate_accepted(installed, &migration, &accepted, true)
}

pub(crate) fn cli(arguments: &[String]) -> Result<()> {
    ensure!(
        arguments.len() == 3 && arguments[0] == "validate-preparation" && arguments[1] == "--migration",
        "usage: sm-pipeline translation-migration validate-preparation --migration PACKAGE.json"
    );
    emit(&validate_preparation(Path::new(&arguments[2]))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ::translation_execution::model::sha256;
    use std::{
        fs,
        os::unix::fs::{PermissionsExt, symlink},
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    #[ignore = "read-only diagnosis of a retained managed publication intent; no source or queue mutation"]
    fn retained_managed_intent_and_controller_projection_authenticate_without_mutation() -> Result<()> {
        let root = PathBuf::from(std::env::var("SM_MANAGED_NATIVE_PROOF_ROOT")?);
        let installed: supervisor::Installation = object(&root.join("installation.json"))?;
        let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
        let completion: Completion =
            object(&installed.state_root.join(run_id).join("attempt-0001/publication/completion.json"))?;
        let before = supervisor::tests::records(&installed.integration_repository)?;
        let result = crate::inputs::bounded::limited(|| {
            let authenticated = supervisor::authenticate_managed_completion(&installed, &completion)
                .context("retained completion authentication")?;
            let (_, migration, _) = managed_migration(&installed)?;
            let root = publication_root(&migration, &installed.policy, completion.attempt)?;
            let intent: AdmissionIntent = object(&root.join("intent.json"))?;
            authenticate_intent(&installed, &migration, &intent, &authenticated)
                .context("retained original publication intent")?;
            let accepted_path = root.join("accepted.json");
            if accepted_path.exists() {
                let accepted: AcceptedAdmission = object(&accepted_path)?;
                authenticate_accepted(&installed, &migration, &accepted, true)
                    .context("retained accepted managed source and queue publication")?;
            } else {
                controller_projection(&installed, &migration, &intent, None)
                    .context("retained controller projection before source mutation")?;
            }
            let mut foreign = current_ledger(&migration)?;
            foreign.integration_owner = Some("foreign-integrator".into());
            ensure!(
                check_intent_queue(&installed, &migration, &intent, &foreign).is_err(),
                "managed projection admitted a foreign integration owner"
            );
            Ok(())
        });
        let after = supervisor::tests::records(&installed.integration_repository)?;
        let changed = before
            .keys()
            .chain(after.keys())
            .filter(|path| before.get(*path) != after.get(*path))
            .collect::<BTreeSet<_>>();
        ensure!(changed.is_empty(), "read-only managed intent diagnosis changed {changed:?}; outcome: {result:?}");
        result
    }

    struct Fixture {
        root: PathBuf,
        migration: Migration,
        original: OriginalClaim,
        entry: assignment::Entry,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "sm-migration-tests-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("src")).unwrap();
            fs::write(root.join("src/domain.rs"), "pub fn value() -> u32 { 7 }\n").unwrap();
            git(&root, &["init", "-q"]).unwrap();
            git(&root, &["config", "user.name", "migration proof"]).unwrap();
            git(&root, &["config", "user.email", "proof@localhost"]).unwrap();
            git(&root, &["add", "src/domain.rs"]).unwrap();
            git(&root, &["commit", "-q", "-m", "baseline"]).unwrap();
            let commit = git(&root, &["rev-parse", "HEAD"]).unwrap();
            fs::write(root.join(".git/info/exclude"), "/.scratch/\n").unwrap();
            let claim = assignment::Claim {
                id: "owned-subsystem".into(),
                revision: sha256("original revision"),
                attempt: 1,
                coordinator: "original-owner".into(),
            };
            let job = assignment::Job {
                id: claim.id.clone(),
                revision: claim.revision.clone(),
                kind: assignment::Kind::Implement,
                subject: "Implement subsystem".into(),
                writable_paths: vec!["src/domain.rs".into()],
                blockers: vec![],
                inputs: BTreeMap::from([("base-head".into(), commit.clone())]),
                instructions: "frozen implementation".into(),
                ticket: None,
            };
            let entry: assignment::Entry = serde_json::from_value(serde_json::json!({
                "job":job, "state":"running", "claim":claim, "agent":"01234567-89ab-cdef-0123-456789abcdef",
                "pending_job":null, "stale":false, "result":null, "attempts":1, "observations":[],
                "last_update":null, "history":[], "adopted_ticket":false, "projection_pending":false
            }))
            .unwrap();
            let original = OriginalClaim {
                schema: "translation-original-claim-v1".into(),
                checkout: root.clone(),
                effort: "owned-effort".into(),
                job: entry.job.clone(),
                claim: entry.claim.clone().unwrap(),
                state: entry.state.clone(),
                agent: entry.agent.clone(),
                result: None,
                observations: vec![],
                source_checkout: root.clone(),
                source_commit: commit.clone(),
                source_files: BTreeMap::from([("src/domain.rs".into(), digest(&root.join("src/domain.rs")).unwrap())]),
            };
            fs::create_dir_all(root.join(".scratch/decomp-parallel")).unwrap();
            let original_path = root.join(".scratch/original-claim.json");
            fs::write(&original_path, serde_json::to_vec(&original).unwrap()).unwrap();
            let placeholder = root.join(".scratch/retained.json");
            fs::write(&placeholder, b"{}").unwrap();
            let reference = retention::seal_file(&placeholder, MAX_BYTES).unwrap();
            let migration = Migration {
                schema: "translation-execution-migration-v1".into(),
                state: "prepared".into(),
                checkout: root.clone(),
                effort: original.effort.clone(),
                coordinator: "current-integrator".into(),
                session_id: "abcdef01-2345-6789-abcd-ef0123456789".into(),
                generation: 2,
                authority: reference.clone(),
                ledger: reference.clone(),
                claims: vec![MigratedClaim {
                    claim: ClaimLineage {
                        id: claim.id.clone(),
                        revision: claim.revision.clone(),
                        attempt: claim.attempt,
                        coordinator: claim.coordinator.clone(),
                    },
                    scope_id: "whole-subsystem".into(),
                    policy_sha256: sha256("proposed policy"),
                    source_commit: commit,
                    writable_files: job.writable_paths.clone(),
                    original_claim: retention::seal_file(&original_path, MAX_BYTES).unwrap(),
                    installation: None,
                    execution: None,
                }],
                historical: vec![],
                stopped_owners: vec![],
                proof: reference,
                limitations: vec!["Preparation only; production stays inactive.".into()],
            };
            Self { root, migration, original, entry }
        }

        fn fence(&self, state: &str) -> PathBuf {
            let mut migration = self.migration.clone();
            migration.state = state.into();
            let path = self.root.join(FENCE);
            fs::write(&path, serde_json::to_vec(&migration).unwrap()).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
            path
        }

        fn request(&self, operation: &str) -> assignment::Request {
            assignment::Request {
                operation: operation.into(),
                checkout: self.root.clone(),
                effort: self.migration.effort.clone(),
                coordinator: "current-integrator".into(),
                ..Default::default()
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn migration_lineage_rejects_changed_revision_writable_paths_and_checkpoint_bytes() {
        let fixture = Fixture::new();
        assert!(
            original_lineage(&fixture.migration, &fixture.migration.claims[0], &fixture.entry, &mut References::new())
                .is_ok()
        );
        let mut entry = fixture.entry.clone();
        entry.claim.as_mut().unwrap().revision = sha256("new revision");
        assert!(
            original_lineage(&fixture.migration, &fixture.migration.claims[0], &entry, &mut References::new()).is_err()
        );
        let mut entry = fixture.entry.clone();
        entry.job.writable_paths.push("src/other.rs".into());
        assert!(
            original_lineage(&fixture.migration, &fixture.migration.claims[0], &entry, &mut References::new()).is_err()
        );
        fs::write(fixture.root.join("src/domain.rs"), "pub fn value() -> u32 { 9 }\n").unwrap();
        assert!(
            original_lineage(&fixture.migration, &fixture.migration.claims[0], &fixture.entry, &mut References::new())
                .is_err()
        );
    }

    #[test]
    fn stopped_observation_must_name_exact_owner_and_no_current_kernel_writer() {
        let fixture = Fixture::new();
        let session_path = fixture.root.join(".scratch/session-observation.json");
        let observed = assignment::authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: fixture.original.agent.clone().unwrap(),
            observed_at: "2026-10-13 19:00 UTC".into(),
            status: "interrupted".into(),
            evidence: vec![assignment::Artifact {
                path: fixture.migration.proof.path.clone(),
                sha256: fixture.migration.proof.sha256.clone(),
            }],
        };
        fs::write(&session_path, serde_json::to_vec(&observed).unwrap()).unwrap();
        let stop = OriginalOwnerStop {
            schema: "translation-original-owner-stop-v1".into(),
            checkout: fixture.root.clone(),
            effort: fixture.migration.effort.clone(),
            claim: fixture.original.claim.clone(),
            agent: fixture.original.agent.clone(),
            observed_at: "2026-10-13 19:00 UTC".into(),
            status: "interrupted".into(),
            session: Some(retention::seal_file(&session_path, MAX_BYTES).unwrap()),
            evidence: vec![fixture.migration.proof.clone()],
            processes: vec![],
        };
        assert!(
            stopped_owner(&fixture.migration, &fixture.original, std::slice::from_ref(&stop), &mut References::new())
                .is_ok()
        );
        let mut foreign = stop.clone();
        foreign.claim.attempt += 1;
        assert!(stopped_owner(&fixture.migration, &fixture.original, &[foreign], &mut References::new()).is_err());
        let mut missing_session = stop.clone();
        missing_session.session = None;
        assert!(
            stopped_owner(&fixture.migration, &fixture.original, &[missing_session], &mut References::new()).is_err()
        );
        let mut wrong_session = observed;
        wrong_session.session_id = "abcdef01-2345-6789-abcd-ef0123456789".into();
        let wrong_path = fixture.root.join(".scratch/wrong-session.json");
        fs::write(&wrong_path, serde_json::to_vec(&wrong_session).unwrap()).unwrap();
        let mut foreign_session = stop.clone();
        foreign_session.session = Some(retention::seal_file(&wrong_path, MAX_BYTES).unwrap());
        assert!(
            stopped_owner(&fixture.migration, &fixture.original, &[foreign_session], &mut References::new()).is_err()
        );
        let mut live = stop;
        live.processes.push(OwnerProcess {
            identity: operation::process_identity(std::process::id()).unwrap().unwrap(),
            status: "stopped".into(),
            observation: fixture.migration.proof.clone(),
        });
        assert!(stopped_owner(&fixture.migration, &fixture.original, &[live], &mut References::new()).is_err());
    }

    #[test]
    fn active_fence_preserves_status_but_denies_assign_and_session_producers_before_writes() {
        let fixture = Fixture::new();
        let fence = fixture.fence("active");
        let before = fs::read(&fence).unwrap();
        assert!(assignment::operate(fixture.request("status")).is_ok());
        assert!(assignment::operate(fixture.request("next")).unwrap_err().to_string().contains("retired"));
        let session = crate::session::Request::Resume {
            queue: fixture.request("status"),
            handoff: fixture.root.join("missing-handoff.toml"),
        };
        assert!(crate::session::operate(session).unwrap_err().to_string().contains("retired"));
        let coordination = crate::coordination::Request {
            operation: "record".into(),
            checkout: fixture.root.clone(),
            effort: fixture.migration.effort.clone(),
            coordinator: "current-integrator".into(),
            ..Default::default()
        };
        assert!(crate::coordination::operate(coordination).unwrap_err().to_string().contains("retired"));
        assert_eq!(fs::read(fence).unwrap(), before);
        assert!(!fixture.root.join(format!(".scratch/{}/ledger.toml", fixture.migration.effort)).exists());
        assert!(!fixture.root.join(".scratch/decomp-parallel/deterministic-results").exists());
    }

    #[test]
    fn fence_symlinks_and_directory_aliases_fail_closed_but_preparation_is_not_activation() {
        let fixture = Fixture::new();
        let path = fixture.fence("prepared");
        assert!(fence_legacy_locked(&fixture.root, &fixture.request("next")).is_ok());
        let alias = fixture.root.join(".scratch/alias.json");
        symlink(&path, &alias).unwrap();
        assert!(read(&alias).is_err());
        fs::remove_file(&path).unwrap();
        symlink(&alias, &path).unwrap();
        assert!(fence_legacy_locked(&fixture.root, &fixture.request("next")).is_err());
        assert!(relative(Path::new("src/domain.rs")));
        assert!(!relative(Path::new("src//domain.rs")));
        assert!(!relative(Path::new("src/../domain.rs")));
    }

    #[test]
    fn nominal_proof_hash_is_never_accepted_as_completed_supervisor_evidence() {
        let fixture = Fixture::new();
        let path = fixture.root.join(".scratch/nominal-proof.json");
        fs::write(&path, br#"{"schema":"translation-supervisor-proof-v1","policy_sha256":"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"}"#).unwrap();
        assert!(object::<RetainedProof>(&path).is_err());
        assert!(supervisor::authenticate_retained_proof(&path, &path).is_err());
    }

    #[test]
    #[ignore = "read-only actual retained supervisor proof and original claim package; no producers"]
    fn retained_original_package_validates_read_only_after_complete_proof() -> Result<()> {
        let path = PathBuf::from(std::env::var("SM_TRANSLATION_MIGRATION_PROOF")?);
        let migration = read(&path)?;
        let registry = migration.checkout.join(".scratch/decomp-progress/registry.json");
        let saved = [&path, &migration.authority.path, &migration.ledger.path, &registry]
            .into_iter()
            .map(|path| Ok((path.clone(), crate::digest(path)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let retained_decision = path.parent().context("package parent missing")?.join("validation.json");
        let retained_decision_bytes = fs::read(&retained_decision)?;
        let proof: RetainedProof = object(&migration.proof.path)?;
        let installed: supervisor::Installation = object(&proof.installation.path)?;
        let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
        let before = supervisor::tests::records(&installed.state_root.join(&run_id))?;
        let report = validate_preparation(&path)?;
        ensure!(
            report.status == "validated-with-blockers"
                && !report.activated
                && !report.activation_ready
                && report.claims.len() == migration.claims.len()
                && report.claims.iter().all(|claim| claim.exact_original_lineage && claim.stopped_owner_observed),
            "read-only package validation changed original admission or activated production"
        );
        for (path, expected) in saved {
            ensure!(crate::digest(&path)? == expected, "validation changed an original input: {}", path.display());
        }
        ensure!(fs::read(retained_decision)? == retained_decision_bytes, "validation replaced a historical decision");
        ensure!(
            supervisor::tests::records(&installed.state_root.join(&run_id))? == before,
            "validation changed retained supervisor evidence or dispatched a producer"
        );
        Ok(())
    }

    #[test]
    fn managed_domain_binds_original_identity_without_derived_seal_cycles() {
        let fixture = Fixture::new();
        let domain = domain_identity(&fixture.migration).unwrap();
        let mut derived = fixture.migration.clone();
        derived.state = "active".into();
        derived.claims[0].policy_sha256 = sha256("derived installed policy");
        derived.claims[0].installation = Some(fixture.root.join(".scratch/installation.json"));
        assert_eq!(domain_identity(&derived).unwrap(), domain);
        derived.claims[0].claim.attempt += 1;
        assert_ne!(domain_identity(&derived).unwrap(), domain);
        let mut changed = fixture.migration.clone();
        changed.claims[0].source_commit = "d".repeat(40);
        assert_ne!(domain_identity(&changed).unwrap(), domain);
        let mut changed = fixture.migration.clone();
        changed.authority.sha256 = sha256("other authority generation");
        assert_ne!(domain_identity(&changed).unwrap(), domain);
        let mut changed = fixture.migration.clone();
        changed.claims[0].original_claim.sha256 = sha256("another original claim");
        assert_ne!(domain_identity(&changed).unwrap(), domain);
    }

    #[test]
    fn preparation_preserves_only_exact_legacy_linked_ownership_and_abbreviated_baseline() {
        let mut fixture = Fixture::new();
        let abbreviated = fixture.original.source_commit[..7].to_owned();
        fixture.original.job.inputs.insert("base-head".into(), abbreviated.clone());
        fixture.original.state = assignment::State::Submitted;
        fixture.original.result = Some(assignment::WorkResult::Submitted {
            commit: fixture.original.source_commit.clone(),
            evidence: vec![artifact(&fixture.migration.proof)],
        });
        let path = fixture.migration.claims[0].original_claim.path.clone();
        fs::write(&path, serde_json::to_vec(&fixture.original).unwrap()).unwrap();
        fixture.migration.claims[0].original_claim = retention::seal_file(&path, MAX_BYTES).unwrap();
        let mut job = assignment::admitted_integration_job(
            &fixture.original.job,
            &fixture.original.claim,
            &fixture.original.source_commit,
        )
        .unwrap();
        job.inputs.remove("base-head");
        let integration = OriginalClaim {
            schema: "translation-original-claim-v1".into(),
            checkout: fixture.root.clone(),
            effort: fixture.migration.effort.clone(),
            claim: assignment::Claim {
                id: job.id.clone(),
                revision: job.revision.clone(),
                attempt: 1,
                coordinator: fixture.original.claim.coordinator.clone(),
            },
            job,
            state: assignment::State::Running,
            agent: Some("abcdef01-2345-6789-abcd-ef0123456789".into()),
            result: None,
            observations: vec![],
            source_checkout: fixture.root.clone(),
            source_commit: fixture.original.source_commit.clone(),
            source_files: fixture.original.source_files.clone(),
        };
        let integrated_path = fixture.root.join(".scratch/integration-original.json");
        fs::write(&integrated_path, serde_json::to_vec(&integration).unwrap()).unwrap();
        fixture.migration.claims.push(MigratedClaim {
            claim: ClaimLineage {
                id: integration.claim.id.clone(),
                revision: integration.claim.revision.clone(),
                attempt: integration.claim.attempt,
                coordinator: integration.claim.coordinator.clone(),
            },
            scope_id: "original-derived-integration".into(),
            policy_sha256: sha256("blocked derived policy"),
            source_commit: integration.source_commit.clone(),
            writable_files: integration.job.writable_paths.clone(),
            original_claim: retention::seal_file(&integrated_path, MAX_BYTES).unwrap(),
            installation: None,
            execution: None,
        });
        let prepared = fixture.fence("prepared");
        let imported = read(&prepared).unwrap();
        assert_eq!(imported.claims.len(), 2);
        validate_original_source(
            &fixture.migration,
            &fixture.migration.claims[0],
            &fixture.original,
            &mut References::new(),
        )
        .unwrap();
        assert_eq!(fixture.original.job.inputs["base-head"], abbreviated);
        assert!(read(&fixture.fence("active")).is_err());
        let historical_domain = domain_identity(&fixture.migration).unwrap();
        fixture.migration.claims[0].execution = Some(MigrationExecution::ImportedSubmission);
        fixture.migration.claims[1].execution =
            Some(MigrationExecution::HeldIntegration { submission: fixture.migration.claims[0].claim.clone() });
        assert_ne!(domain_identity(&fixture.migration).unwrap(), historical_domain);
        assert_eq!(read(&fixture.fence("active")).unwrap().claims.len(), 2);
        fixture.migration.claims[1].execution = Some(MigrationExecution::ImportedSubmission);
        assert!(read(&fixture.fence("active")).is_err());
        fixture.migration.claims[1].execution =
            Some(MigrationExecution::HeldIntegration { submission: fixture.migration.claims[0].claim.clone() });
        let mut foreign = integration;
        foreign.job.inputs.insert("submission-attempt".into(), "2".into());
        fs::write(&integrated_path, serde_json::to_vec(&foreign).unwrap()).unwrap();
        fixture.migration.claims[1].original_claim = retention::seal_file(&integrated_path, MAX_BYTES).unwrap();
        assert!(read(&fixture.fence("prepared")).is_err());
    }

    fn imported_integration_fixture(fixture: &Fixture, running: bool) -> (assignment::Entry, assignment::Entry) {
        let mut submission = fixture.entry.clone();
        submission.state = assignment::State::Submitted;
        let result = assignment::WorkResult::Submitted {
            commit: fixture.original.source_commit.clone(),
            evidence: vec![artifact(&fixture.migration.proof)],
        };
        submission.result = Some(result.clone());
        submission.last_update = Some(assignment::Update::Finished { claim: fixture.original.claim.clone(), result });
        let mut integration = fixture.entry.clone();
        integration.job = assignment::admitted_integration_job(
            &submission.job,
            submission.claim.as_ref().unwrap(),
            &fixture.original.source_commit,
        )
        .unwrap();
        integration.job.inputs.remove("base-head");
        integration.result = None;
        integration.state = if running { assignment::State::Running } else { assignment::State::Ready };
        integration.agent = running.then(|| "11111111-2222-3333-4444-555555555555".into());
        integration.attempts = u64::from(running);
        integration.claim = running.then(|| assignment::Claim {
            id: integration.job.id.clone(),
            revision: integration.job.revision.clone(),
            attempt: 1,
            coordinator: fixture.migration.coordinator.clone(),
        });
        integration.last_update = integration.claim.as_ref().map(|claim| assignment::Update::Started {
            claim: claim.clone(),
            agent: integration.agent.clone().unwrap(),
        });
        (submission, integration)
    }

    #[test]
    fn imported_ready_integration_reserves_one_attempt_and_recovers_exact_completion_without_writes() {
        let fixture = Fixture::new();
        let (mut submission, frozen) = imported_integration_fixture(&fixture, false);
        let original_submission = submission.clone();
        let before = supervisor::tests::records(&fixture.root).unwrap();
        check_imported_integration(&fixture.migration, &submission, &frozen, &frozen, None).unwrap();
        let mut current = frozen.clone();
        current.state = assignment::State::Reserved;
        current.attempts = 1;
        let claim = assignment::Claim {
            id: frozen.job.id.clone(),
            revision: frozen.job.revision.clone(),
            attempt: 1,
            coordinator: fixture.migration.coordinator.clone(),
        };
        current.claim = Some(claim.clone());
        check_imported_integration(&fixture.migration, &submission, &frozen, &current, None).unwrap();
        current.state = assignment::State::Running;
        current.agent = Some(fixture.migration.session_id.clone());
        current.last_update =
            Some(assignment::Update::Started { claim: claim.clone(), agent: fixture.migration.session_id.clone() });
        check_imported_integration(&fixture.migration, &submission, &frozen, &current, None).unwrap();
        current.state = assignment::State::Complete;
        submission.state = assignment::State::Complete;
        let result = assignment::WorkResult::Integrated {
            submissions: vec![fixture.original.claim.clone()],
            evidence: vec![artifact(&fixture.migration.proof)],
            authentication: artifact(&fixture.migration.proof),
            cleanup: artifact(&fixture.migration.proof),
        };
        let update = assignment::Update::Finished { claim, result: result.clone() };
        current.result = Some(result);
        current.last_update = Some(update.clone());
        for _ in 0..2 {
            check_imported_integration(&fixture.migration, &submission, &frozen, &current, Some(&update)).unwrap();
        }
        assert_eq!(submission.result, original_submission.result);
        assert_eq!(submission.last_update, original_submission.last_update);
        assert_eq!(supervisor::tests::records(&fixture.root).unwrap(), before);
        let mut foreign = current;
        foreign.attempts = 2;
        assert!(check_imported_integration(&fixture.migration, &submission, &frozen, &foreign, Some(&update)).is_err());
    }

    #[test]
    fn imported_running_integration_preserves_paused_original_worker_and_refuses_restart() {
        let fixture = Fixture::new();
        let (submission, frozen) = imported_integration_fixture(&fixture, true);
        let before = supervisor::tests::records(&fixture.root).unwrap();
        for _ in 0..2 {
            check_imported_integration(&fixture.migration, &submission, &frozen, &frozen, None).unwrap();
        }
        let mut changed = frozen.clone();
        changed.agent = Some(fixture.migration.session_id.clone());
        assert!(check_imported_integration(&fixture.migration, &submission, &frozen, &changed, None).is_err());
        changed = frozen.clone();
        changed.state = assignment::State::Reserved;
        assert!(check_imported_integration(&fixture.migration, &submission, &frozen, &changed, None).is_err());
        changed = frozen.clone();
        changed.job.inputs.insert("base-head".into(), fixture.original.source_commit.clone());
        assert!(check_imported_integration(&fixture.migration, &submission, &frozen, &changed, None).is_err());
        assert_eq!(supervisor::tests::records(&fixture.root).unwrap(), before);
    }

    #[test]
    fn original_pause_coverage_refuses_reduced_native_acceptance_before_producers() {
        let error = require_offline_composite_adapter("pause-equipment-render-offline").unwrap_err().to_string();
        assert!(error.contains("607 actual-C fixture pairs"));
        assert!(error.contains("auxiliary direct-animation"));
        assert!(error.contains("10 Rust tests"));
        assert!(require_offline_composite_adapter("another-original-coverage-scope").is_err());
    }

    #[test]
    fn frozen_original_inputs_fail_before_retention_on_stale_hash_fingerprint_and_alias() {
        let mut fixture = Fixture::new();
        let dependency = fixture.root.join("src/dependency.rs");
        fs::write(&dependency, "original dependency\n").unwrap();
        fixture.entry.job.inputs.insert("file:src/dependency.rs".into(), digest(&dependency).unwrap());
        let root = fixture.root.join(".scratch/prepared-inputs");
        let original = projected_job_inputs(&fixture.root, &fixture.entry.job, &root).unwrap();
        assert!(!root.exists());
        fs::write(&dependency, "foreign dependency\n").unwrap();
        assert!(projected_job_inputs(&fixture.root, &fixture.entry.job, &root).is_err());
        assert!(!root.exists());
        fs::write(&dependency, "original dependency\n").unwrap();
        persist_job_inputs(&fixture.root, &fixture.entry.job, &root, &original).unwrap();
        assignment::authenticate_frozen_job_inputs(&fixture.root, &fixture.entry.job, &original).unwrap();
        fixture.entry.job.inputs.insert("source-fingerprint".into(), sha256("unsupported legacy tree"));
        assert!(assignment::admitted_input_files(&fixture.root, &fixture.entry.job).is_err());
        fixture.entry.job.inputs.remove("source-fingerprint");
        let alias = fixture.root.join("src/alias.rs");
        symlink(&dependency, &alias).unwrap();
        fixture.entry.job.inputs.insert("file:src/alias.rs".into(), digest(&dependency).unwrap());
        assert!(assignment::admitted_input_files(&fixture.root, &fixture.entry.job).is_err());
    }

    #[test]
    fn managed_immutable_write_recovers_exact_pending_and_preserves_partial_failure() {
        let fixture = Fixture::new();
        let path = fixture.root.join(".scratch/immutable-result.json");
        let pending = path.with_extension("pending");
        let expected = br#"{"exact":"original result"}"#;
        fs::write(&pending, expected).unwrap();
        let reference = write_bytes(&path, expected).unwrap();
        assert_eq!(reference.sha256, sha256(expected));
        assert!(!pending.exists());
        assert_eq!(write_bytes(&path, expected).unwrap(), reference);
        fs::write(&pending, br#"{"exact":"#).unwrap();
        assert!(write_bytes(&path, expected).is_err());
        assert_eq!(fs::read(&pending).unwrap(), br#"{"exact":"#);
        assert_eq!(fs::read(&path).unwrap(), expected);
        fs::remove_file(&pending).unwrap();
        symlink(&path, &pending).unwrap();
        assert!(write_bytes(&path, expected).is_err());
        assert_eq!(fs::read(&path).unwrap(), expected);
    }

    #[test]
    fn managed_original_commands_require_exact_confined_equivalence() {
        let family: native::NativeContract = serde_json::from_value(serde_json::json!({
            "family":"native-jump", "manifest":{"file":"tools/port/native-jump.toml", "sha256":sha256("manifest")},
            "source_inventory":{"file":"tools/port/source-inventory.toml","sha256":sha256("inventory")},
            "coverage":{"InstructionJump":["complete-fixture-domain"]},
            "provenance":{"kind":"reviewed-offline-family","verdict":{"file":"/retained/verdict.toml","sha256":sha256("verdict")},"checkout":"/retained/source"},
            "frozen_inputs":{}, "compiler_limits":{"timeout_seconds":60,"writable_bytes":1048576,"stdout_bytes":4096,"stderr_bytes":4096,"stdin_bytes":0},
            "fixture_limits":{"timeout_seconds":5,"writable_bytes":262144,"stdout_bytes":4096,"stderr_bytes":4096,"stdin_bytes":0}
        })).unwrap();
        let gate = |argv: &[&str]| translation::Verification {
            argv: argv.iter().map(|v| (*v).into()).collect(),
            timeout_seconds: 60,
            integration: false,
        };
        let native = gate(&["sm-pipeline", "family-verify", "--manifest", "tools/port/native-jump.toml"]);
        assert_eq!(declared_family(&native, std::slice::from_ref(&family)).unwrap(), "native-jump");
        let cargo = gate(&[
            "cargo",
            "run",
            "-p",
            "pipeline",
            "--locked",
            "--offline",
            "--",
            "family-verify",
            "--manifest",
            "tools/port/native-jump.toml",
        ]);
        assert_eq!(declared_family(&cargo, std::slice::from_ref(&family)).unwrap(), "native-jump");
        for gate in [
            gate(&["cargo", "test", "-p", "port", "--locked", "--offline"]),
            gate(&["sm-pipeline", "live-verify", "--family", "native-jump"]),
            gate(&["sm-pipeline", "batch", "verify", "--profile", "offline-all"]),
            gate(&["sm-pipeline", "family-verify", "--manifest", "tools/port/foreign.toml"]),
            gate(&["sm-pipeline", "family-verify", "--manifest", "tools/port/native-jump.toml", "--root", "/foreign"]),
        ] {
            assert!(declared_family(&gate, std::slice::from_ref(&family)).is_err());
        }
    }

    #[test]
    fn admitted_queue_keeps_original_attempt_and_standard_submission_integration_contract() {
        let mut fixture = Fixture::new();
        let session = std::env::var("CODEX_THREAD_ID").unwrap();
        fixture.migration.session_id = session.clone();
        fixture.migration.generation = 1;
        fixture.original.claim.coordinator = fixture.migration.coordinator.clone();
        fixture.entry.claim = Some(fixture.original.claim.clone());
        let evidence_path = fixture.root.join(".scratch/evidence.json");
        fs::write(&evidence_path, b"{\"trusted_fixture_evidence\":true}").unwrap();
        let evidence = assignment::Artifact { path: evidence_path.clone(), sha256: digest(&evidence_path).unwrap() };
        let ticket_path = fixture.root.join(".scratch/owned-effort/issues/original.md");
        fs::create_dir_all(ticket_path.parent().unwrap()).unwrap();
        let original_ticket = "# Original subsystem\n\nStatus: claimed\nBlocked by: None\n\nHuman scope notes remain exact.\n<!-- decomp-assignment:owned-subsystem -->\n";
        fs::write(&ticket_path, original_ticket).unwrap();
        let dependency = fixture.root.join("src/dependency.rs");
        fs::write(&dependency, "original dependency\n").unwrap();
        fixture.entry.job.ticket = Some(ticket_path.strip_prefix(&fixture.root).unwrap().into());
        fixture
            .entry
            .job
            .inputs
            .insert("file:.scratch/owned-effort/issues/original.md".into(), digest(&ticket_path).unwrap());
        fixture.entry.job.inputs.insert("file:src/dependency.rs".into(), digest(&dependency).unwrap());
        fixture
            .entry
            .job
            .inputs
            .insert("file:src/domain.rs".into(), digest(&fixture.root.join("src/domain.rs")).unwrap());
        fixture.original.job = fixture.entry.job.clone();
        let frozen_root = fixture.root.join(".scratch/frozen-original-inputs");
        let frozen_inputs = projected_job_inputs(&fixture.root, &fixture.entry.job, &frozen_root).unwrap();
        persist_job_inputs(&fixture.root, &fixture.entry.job, &frozen_root, &frozen_inputs).unwrap();
        assert_eq!(frozen_inputs.files.len(), 2);
        fs::write(fixture.root.join("src/domain.rs"), "pub fn value() -> u32 { 8 }\n").unwrap();
        let bridge_path = fixture.root.join(".scratch/bridge.toml");
        fs::write(&bridge_path, "schema = 'decomp-legacy-owner-bridge-v1'\nholdings = []\n").unwrap();
        let bridge = assignment::Artifact { path: bridge_path.clone(), sha256: digest(&bridge_path).unwrap() };
        let observed = assignment::authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: session.clone(),
            observed_at: "2026-10-13 21:00 UTC".into(),
            status: "running".into(),
            evidence: vec![evidence.clone()],
        };
        let observed_path = fixture.root.join(".scratch/session.toml");
        fs::write(&observed_path, toml::to_string(&observed).unwrap()).unwrap();
        let registration =
            assignment::Artifact { path: observed_path.clone(), sha256: digest(&observed_path).unwrap() };
        let preparation = assignment::authority::Preparation {
            canonical_effort: fixture.migration.effort.clone(),
            capacity: 3,
            admitted_efforts: vec![],
            drill_efforts: vec![],
            bridge,
            integrator: assignment::authority::RoleRegistration {
                token: fixture.migration.coordinator.clone(),
                kind: assignment::authority::RoleKind::Integrator,
                session_id: session.clone(),
                observation: registration.clone(),
            },
        };
        let role = assignment::authority::Role {
            token: fixture.migration.coordinator.clone(),
            kind: assignment::authority::RoleKind::Integrator,
            session_id: session.clone(),
            generation: 1,
            registration,
            transfers: vec![],
            last_transfer: None,
        };
        let mut policy = serde_json::json!({ "schema":"decomp-session-authority-v1", "state":"active", "prepared_head":fixture.original.source_commit,
            "preparation":preparation, "roles":{fixture.migration.coordinator.clone():role}, "drill":evidence });
        fn remove_nulls(value: &mut serde_json::Value) {
            match value {
                serde_json::Value::Object(values) => {
                    values.retain(|_, value| !value.is_null());
                    for value in values.values_mut() {
                        remove_nulls(value);
                    }
                }
                serde_json::Value::Array(values) => {
                    for value in values {
                        remove_nulls(value);
                    }
                }
                _ => {}
            }
        }
        remove_nulls(&mut policy);
        fs::write(fixture.root.join(AUTHORITY), toml::to_string(&policy).unwrap()).unwrap();
        let ledger_path = fixture.root.join(format!(".scratch/{}/ledger.toml", fixture.migration.effort));
        fs::create_dir_all(ledger_path.parent().unwrap()).unwrap();
        let mut ledger_value = serde_json::json!({ "schema":"decomp-assignment-v1", "capacity":3,
            "integration_owner":fixture.migration.coordinator, "jobs":{fixture.entry.job.id.clone():fixture.entry} });
        remove_nulls(&mut ledger_value);
        fs::write(&ledger_path, toml::to_string(&ledger_value).unwrap()).unwrap();
        let mut queued = request(&fixture.migration, 3);
        let _lock = assignment::mutation_lock(&fixture.root).unwrap();
        let claim = fixture.original.claim.clone();
        queued.update = Some(assignment::Update::Finished {
            claim: claim.clone(),
            result: assignment::WorkResult::Submitted {
                commit: fixture.original.source_commit.clone(),
                evidence: vec![evidence.clone()],
            },
        });
        assignment::record_admitted_locked(&queued, &claim, Some(&frozen_inputs)).unwrap();
        let submitted_bytes = fs::read(&ledger_path).unwrap();
        let submitted_ledger = current_ledger(&fixture.migration).unwrap();
        assignment::validate_admitted_ticket_projection(
            &fixture.root,
            &fixture.migration.effort,
            &submitted_ledger.jobs[&claim.id],
            Some(original_ticket),
        )
        .unwrap();
        assert!(assignment::record_admitted_locked(&queued, &claim, None).is_err());
        assert_eq!(fs::read(&ledger_path).unwrap(), submitted_bytes);
        assignment::record_admitted_locked(&queued, &claim, Some(&frozen_inputs)).unwrap();
        assert_eq!(fs::read(&ledger_path).unwrap(), submitted_bytes);
        let submitted_ticket = fs::read(&ticket_path).unwrap();
        fs::write(&dependency, "later host dependency\n").unwrap();
        assignment::record_admitted_locked(&queued, &claim, Some(&frozen_inputs)).unwrap();
        let mut foreign = frozen_inputs.clone();
        foreign.files.remove("file:src/dependency.rs");
        assert!(assignment::record_admitted_locked(&queued, &claim, Some(&foreign)).is_err());
        let reference = &frozen_inputs.files["file:src/dependency.rs"];
        fs::set_permissions(&reference.path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&reference.path, "tampered frozen dependency\n").unwrap();
        assert!(assignment::record_admitted_locked(&queued, &claim, Some(&frozen_inputs)).is_err());
        fs::write(&reference.path, "original dependency\n").unwrap();
        assert_eq!(fs::read(&ledger_path).unwrap(), submitted_bytes);
        let integration = assignment::reserve_admitted_integration_locked(&queued, &claim).unwrap();
        assert_eq!(integration.id, format!("integrate-{}", claim.id));
        assert_eq!(integration.revision, format!("{}-{}", claim.revision, claim.attempt));
        assert_eq!(assignment::reserve_admitted_integration_locked(&queued, &claim).unwrap(), integration);
        queued.update = Some(assignment::Update::Started { claim: integration.clone(), agent: session });
        assignment::record_admitted_locked(&queued, &integration, None).unwrap();
        let result = assignment::WorkResult::Integrated {
            submissions: vec![claim.clone()],
            evidence: vec![evidence.clone()],
            authentication: evidence.clone(),
            cleanup: evidence,
        };
        let exact = assignment::Update::Finished { claim: integration.clone(), result };
        queued.update = Some(exact.clone());
        assignment::record_admitted_locked(&queued, &integration, None).unwrap();
        let finished = fs::read(&ledger_path).unwrap();
        assignment::record_admitted_locked(&queued, &integration, None).unwrap();
        assert_eq!(fs::read(&ledger_path).unwrap(), finished);
        let ledger = current_ledger(&fixture.migration).unwrap();
        assert_eq!(ledger.jobs[&claim.id].claim.as_ref(), Some(&claim));
        assert_eq!(ledger.jobs[&claim.id].agent, fixture.original.agent);
        assert_eq!(ledger.jobs[&claim.id].state, assignment::State::Complete);
        assert_eq!(ledger.jobs[&integration.id].state, assignment::State::Complete);
        let ticket =
            assignment::admitted_ticket_path(&fixture.root, &fixture.migration.effort, &ledger.jobs[&claim.id].job);
        fs::write(&ticket, &submitted_ticket).unwrap();
        assignment::validate_admitted_ticket_lifecycle(
            &fixture.root,
            &fixture.migration.effort,
            &[submitted_ledger.jobs[&claim.id].clone(), ledger.jobs[&claim.id].clone()],
            Some(original_ticket),
        )
        .unwrap();
        assert!(
            assignment::authenticate_admitted_ticket(&fixture.root, &fixture.migration.effort, &ledger.jobs[&claim.id])
                .is_err()
        );
        assignment::repair_admitted_tickets_locked(&queued, &[claim.clone(), integration.clone()]).unwrap();
        assignment::authenticate_admitted_ticket(&fixture.root, &fixture.migration.effort, &ledger.jobs[&claim.id])
            .unwrap();
        let before = fs::read(&ledger_path).unwrap();
        queued.authority.as_mut().unwrap().generation = 2;
        assert!(assignment::record_admitted_locked(&queued, &integration, None).is_err());
        assert_eq!(fs::read(&ledger_path).unwrap(), before);
        queued.authority.as_mut().unwrap().generation = 1;
        if let assignment::Update::Finished { result: assignment::WorkResult::Integrated { submissions, .. }, .. } =
            queued.update.as_mut().unwrap()
        {
            submissions[0].attempt += 1;
        }
        assert!(assignment::record_admitted_locked(&queued, &integration, None).is_err());
        assert_eq!(fs::read(&ledger_path).unwrap(), before);
        assert!(!fixture.root.join("out").exists());
    }
}
