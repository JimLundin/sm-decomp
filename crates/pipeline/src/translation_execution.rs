//! One installed supervisor owns preparation, source application, prescribed
//! gates, integration, source retention, cleanup and stable publication. The
//! worker interface carries only the immutable capsule and source replacements.
use crate::{
    affected, batch, cleanup, digest, emit, family, family_model::Asset, translation_infrastructure as infrastructure,
    translation_native as native,
};
use ::translation_execution::{
    self as execution, ExecutionAdapter, FinalizationJournal, PublicationJournal, Supervisor, checkout,
    confinement::{self, Program, RunLimits},
    model::*,
    model_rewrite, operation,
    retention::{self, Reference, Retained, RetainedKind, RetentionResult},
    rewrite_adapter::{InstalledRewriteAdapter, RecordedRewriteAdapter, RewriteAdapter},
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_SCOPE_FILES: usize = 128;
const JOURNAL_RESERVATION: u64 = 128 * 1024 * 1024;
const ATTEMPT_METADATA_RESERVATION: u64 = 48 * MAX_RECORD;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InstalledFile {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RewriteInstallation {
    pub executable: InstalledFile,
    pub limits: RunLimits,
    /// A recorded source transformation is used only in an isolated proof.
    /// The installed program still executes under the real restricted transport.
    pub recorded: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider: Option<model_rewrite::Provider>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Contract {
    Native { contracts: Vec<native::NativeContract>, catalog_sha256: String, focused_family: String },
    Infrastructure { contract: infrastructure::InfrastructureContract },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Admission {
    /// A closed, explicitly declared test repository. It never changes a host
    /// queue or publishes gameplay observations to the production inventory.
    IsolatedProof { limitations: Vec<String> },
    /// The original queue claim is inspected, never replaced by a new claim.
    /// Activation and publication must use the separately reviewed migration.
    Managed {
        checkout: PathBuf,
        effort: String,
        coordinator: String,
        session_id: String,
        generation: u64,
        migration: Asset,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Installation {
    pub schema: String,
    pub id: String,
    pub controller: InstalledFile,
    pub helper: InstalledFile,
    pub rustc: InstalledFile,
    pub c_compiler: InstalledFile,
    pub source_repository: PathBuf,
    pub integration_repository: PathBuf,
    pub integration_baseline: String,
    pub pinned_source: PathBuf,
    pub state_root: PathBuf,
    pub pipeline_root: PathBuf,
    pub workers: usize,
    pub max_gate_bytes: u64,
    pub max_checkout_bytes: u64,
    pub max_context_bytes: u64,
    pub rewrite: RewriteInstallation,
    pub policy: FrozenPolicy,
    pub contract: Contract,
    pub admission: Admission,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Prepared {
    Native { families: Vec<native::PreparedNative> },
    Infrastructure { prepared: infrastructure::PreparedInfrastructure },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum Receipt {
    Native {
        prepared: native::PreparedNative,
        source: PathBuf,
        receipt: native::NativeReceipt,
    },
    Infrastructure {
        prepared: infrastructure::PreparedInfrastructure,
        source: PathBuf,
        receipt: infrastructure::InfrastructureReceipt,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct GateRecord {
    context: OperationContext,
    source: SourceSnapshot,
    stage: VerificationStage,
    receipts: Vec<Receipt>,
    outcome: GateOutcome,
}

/// Fully authenticated native acceptance passed to the original-claim publisher.
/// The publisher receives frozen results, never a candidate-selected gate path.
pub(crate) struct AuthenticatedNativeGate {
    pub phase: ExecutionPhase,
    pub stage: VerificationStage,
    pub source: SourceSnapshot,
    pub prepared: native::PreparedNative,
    pub receipt: native::NativeReceipt,
    pub verdict: crate::family_model::Verdict,
}

pub(crate) struct AuthenticatedCompletion {
    pub policy: FrozenPolicy,
    pub candidate_revision: checkout::RevisionArchive,
    pub integrated_revision: checkout::RevisionArchive,
    pub native_gates: Vec<AuthenticatedNativeGate>,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalRecord {
    context: OperationContext,
    request: FinalizationRequest,
    retention: RetentionResult,
    result: Finalization,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    journal_delta_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    journal_reserved_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FinalIntent {
    context: OperationContext,
    request: FinalizationRequest,
    retained_gates: Vec<GateRecord>,
    retained_readiness: Vec<Receipt>,
    revisions: BTreeMap<String, checkout::RevisionArchive>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    prior_finalizations: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct PublicationRecord {
    context: OperationContext,
    completion: Completion,
    publication: Publication,
    projections: Vec<Reference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    managed: Option<crate::translation_migration::ManagedPublication>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    journal_delta_bytes: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct StageDeadline {
    context: OperationContext,
    deadline_unix_ms: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Allocation {
    key: String,
    kind: String,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Retirement {
    root: PathBuf,
    files: Vec<Reference>,
    directories: Vec<PathBuf>,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PartialSource {
    producer: PathBuf,
    diagnostic: String,
    files: BTreeMap<String, Reference>,
}

#[derive(Clone)]
struct Adapter {
    installed: Installation,
    identity: String,
    run_root: PathBuf,
}

fn safe(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "installed path must be absolute");
    ensure!(path.components().collect::<PathBuf>().as_os_str() == path.as_os_str(), "installed path is noncanonical");
    let mut current = PathBuf::new();
    for part in path.components() {
        ensure!(matches!(part, Component::RootDir | Component::Normal(_)), "installed path alias");
        current.push(part);
        match current.symlink_metadata() {
            Ok(meta) => ensure!(!meta.file_type().is_symlink(), "installed path crosses a symlink"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn protected(path: &Path) -> Result<()> {
    safe(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "installation must be owned by the controller and not writable by other users: {}",
        path.display()
    );
    Ok(())
}

fn record_bytes(path: &Path) -> Result<Vec<u8>> {
    safe(path)?;
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
    ensure!(file.metadata()?.is_file() && file.metadata()?.len() <= MAX_RECORD, "bounded record required");
    let before = file.metadata()?;
    let mut bytes = Vec::new();
    (&mut file).take(MAX_RECORD + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    ensure!(
        bytes.len() as u64 == before.len()
            && after.len() == before.len()
            && before.mtime() == after.mtime()
            && before.mtime_nsec() == after.mtime_nsec()
            && before.ctime() == after.ctime()
            && before.ctime_nsec() == after.ctime_nsec(),
        "record changed while reading"
    );
    Ok(bytes)
}

fn read<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = record_bytes(path)?;
    let text = std::str::from_utf8(&bytes)?;
    if path.extension().is_some_and(|extension| extension == "toml") {
        Ok(toml::from_str(text)?)
    } else {
        Ok(serde_json::from_str(text)?)
    }
}

fn write_once<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    write_bytes(path, &bytes)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    safe(path)?;
    ensure!(bytes.len() as u64 <= MAX_RECORD, "controller record budget exceeded");
    if path.exists() {
        ensure!(record_bytes(path)? == bytes, "recorded result changed on retry");
        return Ok(());
    }
    let parent = path.parent().context("record parent missing")?;
    fs::create_dir_all(parent)?;
    let pending = path.with_extension("pending");
    safe(&pending)?;
    if pending.exists() {
        ensure!(pending.is_file(), "orphaned pending record is not regular");
        fs::remove_file(&pending)?;
    }
    let mut file =
        OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    fs::rename(pending, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn installed_file(file: &InstalledFile) -> Result<()> {
    protected(&file.path)?;
    ensure!(
        file.path.is_file() && digest_valid(&file.sha256) && digest(&file.path)? == file.sha256,
        "installed controller/tool identity changed: {}",
        file.path.display()
    );
    Ok(())
}

pub(crate) fn installation_identity(installation: &Installation) -> Result<String> {
    let mut canonical = installation.clone();
    canonical.policy.adapter_sha256 = "0".repeat(64);
    if let Admission::Managed { checkout, migration, .. } = &mut canonical.admission {
        let path = if migration.file.is_absolute() { migration.file.clone() } else { checkout.join(&migration.file) };
        protected(&path)?;
        let domain: crate::translation_migration::Migration = read(&path)?;
        migration.sha256 = crate::translation_migration::domain_identity(&domain)?;
    }
    sealed(&("installed-translation-supervisor-v1", canonical))
}

impl Installation {
    pub(crate) fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == "translation-supervisor-installation-v1"
                && !self.id.is_empty()
                && self.id.len() <= 128
                && self.id.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b)),
            "unsupported installed supervisor"
        );
        self.policy.validate()?;
        if let Some(provider) = &self.rewrite.provider {
            provider.validate()?;
            ensure!(!self.rewrite.recorded, "model-backed rewrites cannot use recorded transformations");
            ensure!(
                provider.timeout_seconds <= self.policy.budgets.gate_timeout_seconds,
                "model deadline exceeds frozen whole-stage limit"
            );
        }
        ensure!(self.workers == 4 && self.policy.budgets.workers == 4, "installed shared worker capacity must be four");
        ensure!(
            self.policy.verification_stages.iter().all(|stage| *stage != VerificationStage::Live),
            "installed confined Live replay adapter is required before this scope can launch"
        );
        ensure!(
            self.max_gate_bytes > 0
                && self.max_checkout_bytes > 0
                && self.max_context_bytes > 0
                && self.max_context_bytes <= self.policy.rewrite_limits.max_context_bytes,
            "installed aggregate resource budgets missing"
        );
        ensure!(self.policy.approved_files.len() <= MAX_SCOPE_FILES, "too many approved source files");
        let metadata = u64::from(self.policy.budgets.max_attempts)
            .checked_mul(ATTEMPT_METADATA_RESERVATION)
            .and_then(|bytes| bytes.checked_add(JOURNAL_RESERVATION))
            .context("controller metadata reservation overflow")?;
        ensure!(
            metadata <= self.policy.budgets.max_evidence_bytes,
            "run budget cannot reserve journal and cleanup metadata for every admitted attempt"
        );
        let rewrite_extent = self
            .policy
            .rewrite_limits
            .max_total_bytes
            .checked_mul(u64::from(self.policy.budgets.max_attempts))
            .and_then(|bytes| bytes.checked_mul(8))
            .and_then(|bytes| bytes.checked_add(8 * MAX_RECORD))
            .context("rewrite checkout reservation overflow")?;
        ensure!(
            rewrite_extent <= self.max_checkout_bytes,
            "installed checkout budget cannot contain the frozen rewrite and revision limits"
        );
        for file in [&self.controller, &self.helper, &self.rustc, &self.c_compiler, &self.rewrite.executable] {
            installed_file(file)?;
        }
        ensure!(
            digest(&std::env::current_exe()?)? == self.controller.sha256,
            "running supervisor differs from installation"
        );
        for path in [
            &self.source_repository,
            &self.integration_repository,
            &self.state_root,
            &self.pipeline_root,
            &self.pinned_source,
        ] {
            safe(path)?;
        }
        let roots = [
            &self.source_repository,
            &self.integration_repository,
            &self.state_root,
            &self.pipeline_root,
            &self.pinned_source,
        ];
        for (index, root) in roots.iter().enumerate() {
            for (other_index, other) in roots.iter().enumerate().skip(index + 1) {
                if index == 0 && other_index == 1 && root == other {
                    continue;
                }
                ensure!(
                    !root.starts_with(other) && !other.starts_with(root),
                    "installed resource roots overlap: {} and {}",
                    root.display(),
                    other.display()
                );
            }
        }
        for file in [&self.controller, &self.helper, &self.rustc, &self.c_compiler, &self.rewrite.executable] {
            ensure!(
                !file.path.starts_with(&self.state_root) && !file.path.starts_with(&self.pipeline_root),
                "installed executable lies inside a writable resource root"
            );
        }
        ensure!(
            self.integration_baseline.len() == 40 && self.integration_baseline.bytes().all(|b| b.is_ascii_hexdigit()),
            "integration baseline must be exact"
        );
        ensure!(
            self.policy.adapter_sha256 == installation_identity(self)?,
            "installed configuration differs from frozen adapter"
        );
        match &self.admission {
            Admission::IsolatedProof { limitations } => {
                ensure!(
                    !limitations.is_empty()
                        && limitations.iter().all(|text| !text.trim().is_empty() && text.len() <= 4096),
                    "isolated proof requires explicit acceptance limitations"
                );
            }
            Admission::Managed { .. } => {
                ensure!(!self.rewrite.recorded, "production cannot install recorded candidate transformations");
            }
        }
        Ok(())
    }
}

impl Adapter {
    fn existing(installed: Installation) -> Result<Self> {
        installed.validate()?;
        let identity = installation_identity(&installed)?;
        let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
        let run_root = installed.state_root.join(run_id);
        protected(&installed.state_root)?;
        protected(&run_root)?;
        Ok(Self { installed, identity, run_root })
    }
    fn new(installed: Installation) -> Result<Self> {
        installed.validate()?;
        fs::create_dir_all(&installed.state_root)?;
        protected(&installed.state_root)?;
        let identity = installation_identity(&installed)?;
        let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
        let run_root = installed.state_root.join(run_id);
        fs::create_dir_all(&run_root)?;
        protected(&run_root)?;
        let adapter = Self { installed, identity, run_root };
        adapter.reserve(&sha256("controller-journal-reservation"), "controller-journal", JOURNAL_RESERVATION)?;
        // Cleanup extents exist before attempt 1 and every deliberate retry is
        // recorded. Finalization never competes with a producer for emergency
        // metadata after a stop or aggregate launch refusal.
        for number in 1..=adapter.installed.policy.budgets.max_attempts {
            adapter.reserve_attempt(number)?;
        }
        Ok(adapter)
    }
    fn paths(&self) -> Vec<String> {
        self.installed.policy.approved_files.keys().cloned().collect()
    }
    fn attempt(&self, number: u32) -> PathBuf {
        self.run_root.join(format!("attempt-{number:04}"))
    }
    fn candidate(&self, number: u32) -> PathBuf {
        self.attempt(number).join("candidate")
    }
    fn integration(&self, number: u32) -> PathBuf {
        self.attempt(number).join("integration")
    }
    fn operations(&self) -> PathBuf {
        self.run_root.join("operations")
    }
    fn model_operations(&self) -> PathBuf {
        self.run_root.join("model-operations")
    }
    fn reserve_attempt(&self, number: u32) -> Result<()> {
        self.reserve(
            &sha256(format!("attempt-{number}-controller-metadata")),
            "controller-finalization-publication-metadata",
            ATTEMPT_METADATA_RESERVATION,
        )
    }
    fn rewrite(
        &mut self,
        context: &OperationContext,
        capsule: &RewriteCapsule,
        recorded: Option<&SourceReplacements>,
    ) -> Result<SourceReplacements> {
        ensure!(
            context.attempt == capsule.attempt && context.run_id == capsule.run_id,
            "rewrite operation differs from capsule"
        );
        ensure!(
            recorded.is_none()
                || (self.installed.rewrite.recorded
                    && matches!(self.installed.admission, Admission::IsolatedProof { .. })),
            "recorded rewrite is limited to an isolated proof"
        );
        if let Some(provider) = &self.installed.rewrite.provider {
            ensure!(recorded.is_none(), "model transport refuses recorded source");
            let limits = self.installed.policy.rewrite_limits;
            self.reserve(
                &context.operation_key,
                "model-rewrite",
                model_rewrite::reserve_bytes(provider, capsule, limits)?,
            )?;
            let deadline = self.deadline(context)?;
            let adapter = model_rewrite::ModelRewriteAdapter::new(
                self.model_operations(),
                provider.clone(),
                self.installed.helper.path.clone(),
                self.installed.helper.sha256.clone(),
            )?;
            let result = adapter.run_until(context, capsule, limits, deadline)?;
            ensure!(result.passed, "model rewrite stopped: {}", result.diagnostic);
            return result.replacements.context("model completion lacks replacements");
        }
        let arguments = recorded.map(serde_json::to_string).transpose()?.into_iter().collect();
        let program = Program {
            executable: self.installed.rewrite.executable.path.clone(),
            helper: self.installed.helper.path.clone(),
            arguments,
            inputs: vec![],
            outputs: vec![],
            runtime: confinement::helper_runtime()?,
        };
        let input = serde_json::to_vec(capsule)?;
        self.reserve(
            &context.operation_key,
            "restricted-rewrite",
            operation::reserve_bytes(&program, &self.installed.rewrite.limits, input.len() as u64)?,
        )?;
        let identity = execution::rewrite_adapter::installed_identity(&program, &self.installed.rewrite.limits)?;
        match recorded {
            Some(replacements) => {
                let mut adapter = RecordedRewriteAdapter::new(
                    self.operations(),
                    program,
                    self.installed.rewrite.limits.clone(),
                    identity,
                    capsule,
                    replacements,
                    self.installed.policy.rewrite_limits,
                )?;
                adapter.rewrite(context, capsule, self.installed.policy.rewrite_limits)
            }
            None => {
                let mut adapter = InstalledRewriteAdapter::new(
                    self.operations(),
                    program,
                    self.installed.rewrite.limits.clone(),
                    identity,
                )?;
                adapter.rewrite(context, capsule, self.installed.policy.rewrite_limits)
            }
        }
    }
    fn deadline(&self, context: &OperationContext) -> Result<u64> {
        let path = self.run_root.join("deadlines").join(format!("{}.json", context.operation_key));
        if path.exists() {
            let saved: StageDeadline = read(&path)?;
            ensure!(saved.context == *context, "whole-stage deadline context changed");
            return Ok(saved.deadline_unix_ms);
        }
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        let deadline_unix_ms = now
            .checked_add(
                self.installed
                    .policy
                    .budgets
                    .gate_timeout_seconds
                    .checked_mul(1000)
                    .context("whole-stage time budget overflow")?,
            )
            .context("whole-stage time budget overflow")?;
        write_once(&path, &StageDeadline { context: context.clone(), deadline_unix_ms })?;
        Ok(deadline_unix_ms)
    }
    fn reserve(&self, key: &str, kind: &str, bytes: u64) -> Result<()> {
        let path = self.run_root.join("allocations").join(format!("{key}.json"));
        let allocation = Allocation { key: key.into(), kind: kind.into(), bytes };
        if path.exists() {
            let saved: Allocation = read(&path)?;
            ensure!(saved == allocation, "allocation retry changed its hard bound");
            return Ok(());
        }
        let root = self.run_root.join("allocations");
        fs::create_dir_all(&root)?;
        let mut total = bytes;
        for entry in fs::read_dir(&root)? {
            let saved: Allocation = read(&entry?.path())?;
            total = total.checked_add(saved.bytes).context("aggregate operation reservation overflow")?;
        }
        ensure!(
            total <= self.installed.policy.budgets.max_evidence_bytes,
            "run aggregate resource reservation exhausted before launch: {total}"
        );
        write_once(&path, &allocation)
    }
    fn bootstrap(&self, context: &OperationContext, deadline: u64) -> Result<()> {
        let mut probe = context.clone();
        probe.operation_key = sealed(&(&context.operation_key, "confinement-readiness"))?;
        let program = Program {
            executable: self.installed.rewrite.executable.path.clone(),
            helper: self.installed.helper.path.clone(),
            arguments: vec![],
            inputs: vec![],
            outputs: vec![],
            runtime: confinement::helper_runtime()?,
        };
        let limits = RunLimits {
            timeout: std::time::Duration::from_secs(10),
            writable_bytes: 128 * 1024,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            stdin_bytes: 0,
        };
        self.reserve(&probe.operation_key, "confinement-readiness", operation::reserve_bytes(&program, &limits, 0)?)?;
        let result = operation::run_until(&self.operations(), &probe, &program, &limits, &[], deadline)?;
        ensure!(
            result.status_raw.is_some(),
            "confinement unavailable before source/rewrite dispatch: {}",
            result.diagnostic
        );
        operation::authenticate_request(&self.operations(), &probe, &program, &limits, &[])?;
        Ok(())
    }
    fn reserve_checkout(&self, key: &str, repository: &Path, baseline: &str, deadline: u64) -> Result<()> {
        // The trusted Git tree inspection precedes clone. Hooks and config
        // injection are disabled; only exact regular tracked paths are admitted.
        let bytes = checkout::checkout_bytes_until(repository, baseline, self.installed.max_checkout_bytes, deadline)?;
        // Independent source writes, immutable source snapshot, raw-object
        // revision archive, Git index and bounded controller records all have
        // reserved extents before a source checkout is created.
        let rewritten = self
            .installed
            .policy
            .rewrite_limits
            .max_total_bytes
            .checked_mul(u64::from(self.installed.policy.budgets.max_attempts))
            .and_then(|bytes| bytes.checked_mul(8))
            .context("rewrite archive reservation overflow")?;
        let bound = bytes
            .checked_mul(4)
            .and_then(|bytes| bytes.checked_add(rewritten))
            .and_then(|bytes| bytes.checked_add(8 * MAX_RECORD))
            .context("checkout resource reservation overflow")?;
        ensure!(
            bound <= self.installed.max_checkout_bytes,
            "source tree exceeds installed checkout bound before clone"
        );
        self.reserve(key, "owned-source-checkout", self.installed.max_checkout_bytes)
    }
    fn reserve_gate(&self, context: &OperationContext, count: usize) -> Result<()> {
        let bytes =
            self.installed.max_gate_bytes.checked_mul(count as u64).context("prescribed gate extent overflow")?;
        self.reserve(&context.operation_key, "prescribed-verification", bytes)
    }
    fn prepared(&self) -> Result<Prepared> {
        read(&self.run_root.join("prepared.json"))
    }
    fn baseline_readiness(&self, prepared: &Prepared) -> Result<Vec<Receipt>> {
        let receipts: Vec<Receipt> = read(&self.attempt(1).join("retained-readiness.json"))?;
        ensure!(!receipts.is_empty(), "repair requires retained baseline harness readiness");
        for receipt in &receipts {
            let matches = match (prepared, receipt) {
                (Prepared::Native { families }, Receipt::Native { prepared: prior, receipt, .. }) => {
                    receipt.passed && families.iter().any(|family| sealed(family).ok() == sealed(prior).ok())
                }
                (Prepared::Infrastructure { prepared }, Receipt::Infrastructure { prepared: prior, receipt, .. }) => {
                    receipt.passed && sealed(prepared)? == sealed(prior)?
                }
                _ => false,
            };
            ensure!(matches, "repair baseline readiness differs from frozen preparation");
            self.authenticate_receipt(receipt)?;
        }
        let expected = match prepared {
            Prepared::Native { families } => families.len(),
            Prepared::Infrastructure { .. } => 1,
        };
        ensure!(receipts.len() == expected, "repair baseline readiness is incomplete");
        Ok(receipts)
    }
    fn check_admission(&self) -> Result<()> {
        if matches!(self.installed.admission, Admission::Managed { .. }) {
            crate::translation_migration::admit_installation(&self.installed)?;
        }
        Ok(())
    }
    fn source_path(&self, source: &SourceSnapshot, deadline: u64) -> Result<PathBuf> {
        for entry in fs::read_dir(&self.run_root)? {
            let path = entry?.path();
            if !path.file_name().and_then(|v| v.to_str()).is_some_and(|name| name.starts_with("attempt-")) {
                continue;
            }
            for name in ["candidate", "integration"] {
                let source_path = path.join(name);
                if source_path.join(".git").is_dir()
                    && checkout::snapshot_until(&source_path, &self.paths(), deadline)? == *source
                {
                    return Ok(source_path);
                }
            }
        }
        bail!("exact retained source checkout is missing")
    }
    fn source_archive(&self, source: &SourceSnapshot) -> Result<checkout::RevisionArchive> {
        for entry in fs::read_dir(&self.run_root)? {
            let path = entry?.path().join("finalization-intent.json");
            if !path.is_file() {
                continue;
            }
            let intent: FinalIntent = read(&path)?;
            if let Some(archive) = intent.revisions.values().find(|archive| archive.source == *source) {
                checkout::authenticate_revision(archive)?;
                return Ok(archive.clone());
            }
        }
        bail!("exact stopped source revision archive is missing")
    }
    fn restore_source(&self, source: &SourceSnapshot, context: &OperationContext, deadline: u64) -> Result<PathBuf> {
        let candidate = self.candidate(context.attempt);
        if candidate.join(".git").is_dir() {
            ensure!(
                checkout::snapshot_until(&candidate, &self.paths(), deadline)? == *source,
                "restored candidate changed"
            );
            return Ok(candidate);
        }
        let archive = self.source_archive(source)?;
        self.reserve_checkout(
            &sealed(&(&context.operation_key, "restored-candidate-checkout"))?,
            &self.installed.source_repository,
            &archive.baseline,
            deadline,
        )?;
        checkout::restore_revision_until(
            &self.installed.source_repository,
            &candidate,
            &archive,
            &self.paths(),
            deadline,
        )?;
        Ok(candidate)
    }
    fn native_runtime(&self) -> native::StrictRuntime {
        native::StrictRuntime {
            helper: self.installed.helper.path.clone(),
            rustc: self.installed.rustc.path.clone(),
            c_compiler: self.installed.c_compiler.path.clone(),
            operation_root: self.operations(),
            pipeline_root: self.installed.pipeline_root.clone(),
            workers: self.installed.workers,
            gate_timeout_seconds: self.installed.policy.budgets.gate_timeout_seconds,
            deadline_unix_ms: None,
            max_output_bytes: self.installed.max_gate_bytes,
        }
    }
    fn infrastructure_runtime(&self) -> infrastructure::StrictRuntime {
        infrastructure::StrictRuntime {
            helper: self.installed.helper.path.clone(),
            rustc: self.installed.rustc.path.clone(),
            operation_root: self.operations(),
            pipeline_root: self.installed.pipeline_root.clone(),
            workers: self.installed.workers,
            gate_timeout_seconds: self.installed.policy.budgets.gate_timeout_seconds,
            deadline_unix_ms: None,
            max_output_bytes: self.installed.max_gate_bytes,
        }
    }
    fn authenticate_receipt(&self, receipt: &Receipt) -> Result<()> {
        match receipt {
            Receipt::Native { prepared, source, receipt } => {
                native::authenticate(prepared, source, receipt)?;
            }
            Receipt::Infrastructure { prepared, source, receipt } => {
                infrastructure::authenticate(prepared, source, receipt)?;
            }
        }
        Ok(())
    }
    fn receipt_references(&self, receipt: &Receipt) -> Result<Vec<Reference>> {
        match receipt {
            Receipt::Native { receipt, .. } => native::references(receipt),
            Receipt::Infrastructure { receipt, .. } => infrastructure::references(receipt),
        }
    }
    fn reusable_receipt(
        &self,
        prepared: &Prepared,
        source: &Path,
        family_name: Option<&str>,
    ) -> Result<Option<Receipt>> {
        let root = self.run_root.join("gates");
        if !root.is_dir() {
            return Ok(None);
        }
        let mut records =
            fs::read_dir(&root)?.map(|entry| entry.map(|entry| entry.path())).collect::<std::io::Result<Vec<_>>>()?;
        records.sort();
        for path in records {
            let mut record: GateRecord = read(&path)?;
            record.receipts = self.authenticated_record_receipts(&record)?;
            for receipt in record.receipts {
                let reused = match (&prepared, receipt) {
                    (Prepared::Native { families }, Receipt::Native { prepared: prior, receipt, .. })
                        if receipt.passed && family_name == Some(prior.contract.family.as_str()) =>
                    {
                        let current = families
                            .iter()
                            .find(|family| Some(family.contract.family.as_str()) == family_name)
                            .context("requested reusable family missing")?;
                        if sealed(current)? != sealed(&prior)? {
                            continue;
                        }
                        if native::authenticate(current, source, &receipt).is_err() {
                            continue;
                        }
                        Receipt::Native { prepared: current.clone(), source: source.into(), receipt }
                    }
                    (
                        Prepared::Infrastructure { prepared: current },
                        Receipt::Infrastructure { prepared: prior, receipt, .. },
                    ) if receipt.passed && sealed(current)? == sealed(&prior)? => {
                        if infrastructure::authenticate(current, source, &receipt).is_err() {
                            continue;
                        }
                        Receipt::Infrastructure { prepared: current.clone(), source: source.into(), receipt }
                    }
                    _ => continue,
                };
                return Ok(Some(reused));
            }
        }
        Ok(None)
    }
    fn preserve_receipt(&self, receipt: &mut Receipt) -> Result<()> {
        let source = match receipt {
            Receipt::Native { source, .. } | Receipt::Infrastructure { source, .. } => source.clone(),
        };
        self.authenticate_receipt(receipt)?;
        if source.join(".git").is_dir() {
            let files = self.frozen_source_closure(&source)?;
            let paths = files.keys().cloned().collect::<Vec<_>>();
            let retained = self.run_root.join("source-snapshots").join(sealed(&files)?);
            checkout::retain_source(&source, &retained, &paths)?;
            match receipt {
                Receipt::Native { source, .. } | Receipt::Infrastructure { source, .. } => *source = retained,
            }
        }
        self.authenticate_receipt(receipt)
    }
    fn seal_outcome(&self, context: &OperationContext, source: &str, evidence: &impl Serialize) -> Result<OutcomeSeal> {
        Ok(OutcomeSeal {
            operation_key: context.operation_key.clone(),
            policy_sha256: context.policy_sha256.clone(),
            source_sha256: source.into(),
            evidence_sha256: sealed(evidence)?,
        })
    }
    fn gate_record(&self, key: &str) -> PathBuf {
        self.run_root.join("gates").join(format!("{key}.json"))
    }
    fn verify_record(&self, record: &GateRecord) -> Result<()> {
        self.verify_record_seal(record)?;
        for receipt in &record.receipts {
            self.authenticate_receipt(receipt)?;
        }
        Ok(())
    }
    fn authenticated_record_receipts(&self, record: &GateRecord) -> Result<Vec<Receipt>> {
        // The outcome binds original producer paths. Path relocation changes
        // only the authenticated source wrapper, never the original seal.
        self.verify_record_seal(record)?;
        let mut receipts = record.receipts.clone();
        for receipt in &mut receipts {
            let source = match receipt {
                Receipt::Native { source, .. } | Receipt::Infrastructure { source, .. } => source.clone(),
            };
            if !source.exists() {
                let intent: FinalIntent = read(&self.attempt(record.context.attempt).join("finalization-intent.json"))?;
                ensure!(
                    intent.context.attempt == record.context.attempt && intent.context.run_id == record.context.run_id,
                    "retired gate finalization lineage changed"
                );
                let retained = intent
                    .retained_gates
                    .iter()
                    .find(|gate| {
                        gate.context == record.context
                            && gate.source == record.source
                            && gate.stage == record.stage
                            && gate.outcome == record.outcome
                    })
                    .context("retired gate lost its equivalent retained result")?;
                ensure!(retained.receipts.len() == record.receipts.len(), "retired gate receipt count changed");
                let matched = retained
                    .receipts
                    .iter()
                    .find(|item| match (&*receipt, item) {
                        (
                            Receipt::Native { prepared: a, receipt: b, .. },
                            Receipt::Native { prepared: c, receipt: d, .. },
                        ) => sealed(&(a, b)).ok() == sealed(&(c, d)).ok(),
                        (
                            Receipt::Infrastructure { prepared: a, receipt: b, .. },
                            Receipt::Infrastructure { prepared: c, receipt: d, .. },
                        ) => sealed(&(a, b)).ok() == sealed(&(c, d)).ok(),
                        _ => false,
                    })
                    .context("retired gate receipt identity changed")?;
                *receipt = matched.clone();
            }
            self.authenticate_receipt(receipt)?;
        }
        Ok(receipts)
    }
    fn verify_record_seal(&self, record: &GateRecord) -> Result<()> {
        ensure!(
            record.outcome.seal
                == self.seal_outcome(
                    &record.context,
                    &record.source.sha256,
                    &(&record.context, &record.source, record.stage, &record.receipts)
                )?,
            "gate record identity changed"
        );
        Ok(())
    }
    fn frozen_source_closure(&self, root: &Path) -> Result<BTreeMap<String, String>> {
        let prepared = self.prepared()?;
        let names = match prepared {
            Prepared::Native { families } => {
                let mut paths = BTreeSet::new();
                for family in families {
                    paths.extend(family.contract.frozen_inputs.keys().cloned());
                    paths.insert(family.contract.manifest.file.display().to_string());
                }
                paths.into_iter().collect::<Vec<_>>()
            }
            Prepared::Infrastructure { prepared } => prepared.contract.source_files.keys().cloned().collect(),
        };
        names.into_iter().map(|name| Ok((name.clone(), digest(&root.join(name))?))).collect()
    }
    fn retirement(&self, root: &Path, context: &OperationContext) -> Result<u64> {
        let record_path =
            self.attempt(context.attempt).join(format!("retire-{}.json", root.file_name().unwrap().to_string_lossy()));
        if !root.exists() && !record_path.exists() {
            return Ok(0);
        }
        let _leases = if root.exists() { checkout::retirement_leases(root)? } else { vec![] };
        let plan: Retirement = if record_path.exists() {
            read(&record_path)?
        } else {
            let mut files = Vec::new();
            let mut directories = Vec::new();
            collect(root, &mut files, &mut directories, self.installed.max_checkout_bytes)?;
            let bytes =
                files.iter().try_fold(0u64, |n, r| n.checked_add(r.bytes).context("checkout extent overflow"))?;
            ensure!(bytes <= self.installed.max_checkout_bytes, "owned checkout budget exceeded");
            let plan = Retirement { root: root.into(), files, directories, bytes };
            write_once(&record_path, &plan)?;
            plan
        };
        ensure!(plan.root == root && root.starts_with(self.attempt(context.attempt)), "retirement ownership changed");
        for reference in &plan.files {
            ensure!(reference.path.starts_with(root), "retirement path escapes owned checkout");
            if reference.path.exists() {
                retention::authenticate_reference(reference, self.installed.max_checkout_bytes)?;
                fs::remove_file(&reference.path)?;
            }
        }
        let mut directories = plan.directories.clone();
        directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
        for directory in directories {
            if directory.exists() {
                safe(&directory)?;
                fs::remove_dir(directory)?;
            }
        }
        ensure!(!root.exists(), "owned implementer checkout was not retired");
        Ok(plan.bytes)
    }
    fn preserve_partial_source(&self, source: &Path, context: &OperationContext, error: &anyhow::Error) -> Result<()> {
        let name = source.file_name().context("partial source name missing")?;
        let root = self.attempt(context.attempt).join("partial-source").join(name);
        let mut files = BTreeMap::new();
        for name in self.paths() {
            let original = source.join(&name);
            if !original.is_file() {
                continue;
            }
            safe(&original)?;
            ensure!(
                original.metadata()?.len() <= self.installed.policy.rewrite_limits.max_file_bytes,
                "partial source exceeds approved per-file extent"
            );
            let destination = root.join(&name);
            write_bytes(&destination, &fs::read(&original)?)?;
            files
                .insert(name, retention::seal_file(&destination, self.installed.policy.rewrite_limits.max_file_bytes)?);
        }
        let diagnostic = format!("{error:#}").chars().take(2048).collect();
        write_once(&root.join("partial-source.json"), &PartialSource { producer: source.into(), diagnostic, files })
    }
    fn authentic_retirement(&self, number: u32) -> Result<()> {
        for name in ["candidate", "integration"] {
            let root = self.attempt(number).join(name);
            ensure!(!root.exists(), "terminal attempt still has an implementer checkout");
            let record_path = self.attempt(number).join(format!("retire-{name}.json"));
            if record_path.exists() {
                let plan: Retirement = read(&record_path)?;
                ensure!(
                    plan.root == root && plan.files.iter().all(|reference| !reference.path.exists()),
                    "retired checkout contains recreated outputs"
                );
            }
        }
        Ok(())
    }
    fn stages(
        &self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        completion: &Completion,
        publication: &Publication,
    ) -> Vec<serde_json::Value> {
        let meanings = match &self.installed.contract {
            Contract::Native { .. } => vec!["translated", "verified", "integrated"],
            Contract::Infrastructure { .. } => vec!["infrastructure-tested", "infrastructure-integrated"],
        };
        let limitations = match &self.installed.admission {
            Admission::IsolatedProof { limitations } => limitations.clone(),
            Admission::Managed { .. } => {
                vec!["Acceptance limited to the frozen prescribed verification domain.".into()]
            }
        };
        meanings
            .iter()
            .map(|stage| {
                serde_json::json!({
                    "id": sha256(format!("{}:{stage}", context.operation_key)), "stage":stage,
                    "scope": policy.scope_id, "claim":policy.claim, "candidate":completion.candidate,
                    "integrated":completion.integrated, "evidence":publication.seal.evidence_sha256,
                    "limitations":limitations
                })
            })
            .collect()
    }
    fn publication_record(&self, completion: &Completion, publication: &Publication) -> Result<PublicationRecord> {
        let path = self.attempt(completion.attempt).join("publication/accepted.json");
        let record: PublicationRecord = read(&path)?;
        ensure!(
            record.completion == *completion && record.publication == *publication,
            "accepted publication differs from journal"
        );
        ensure!(
            record.context.run_id == completion.run_id
                && record.context.attempt == completion.attempt
                && record.context.policy_sha256 == completion.policy_sha256
                && record.publication.seal
                    == self.seal_outcome(&record.context, &completion.integrated.sha256, completion)?,
            "publication accepted identity changed"
        );
        let stages = self.stages(&record.context, &self.installed.policy, completion, publication);
        let root = path.parent().context("publication parent missing")?;
        let expected = [
            (root.join("completion.json"), serde_json::to_vec(completion)?),
            (root.join("stages.json"), serde_json::to_vec(&stages)?),
        ];
        let projections = expected
            .iter()
            .map(|(path, bytes)| Reference { path: path.clone(), sha256: sha256(bytes), bytes: bytes.len() as u64 })
            .collect::<Vec<_>>();
        ensure!(record.projections == projections, "accepted projection references changed");
        let total = projections
            .iter()
            .try_fold(fs::metadata(&path)?.len(), |total, item| {
                total.checked_add(item.bytes).context("publication retained extent overflow")
            })?
            .checked_add(record.managed.as_ref().map_or(0, |value| value.retained_bytes))
            .and_then(|total| total.checked_add(record.journal_delta_bytes.unwrap_or(0)))
            .context("managed publication retained extent overflow")?;
        ensure!(
            record.managed.is_some() == matches!(self.installed.admission, Admission::Managed { .. }),
            "accepted managed publication receipt is missing or foreign"
        );
        ensure!(
            total == publication.retained_bytes && total <= ATTEMPT_METADATA_RESERVATION,
            "publication retained byte accounting changed"
        );
        let final_record: FinalRecord = read(&self.attempt(completion.attempt).join("finalization.json"))?;
        ensure!(
            completion
                .finalization
                .retained_bytes
                .checked_add(total)
                .is_some_and(|bytes| bytes <= final_record.request.remaining_evidence_bytes),
            "publication exceeds remaining attempt evidence budget"
        );
        self.authenticate_journal_extent(completion, &record)?;
        Ok(record)
    }
    fn finalized_journal_bytes(&self, final_record: &FinalRecord) -> Result<u64> {
        let manifest: serde_json::Value = read(&final_record.retention.manifest.path)?;
        let retained: Vec<Retained> =
            serde_json::from_value(manifest.get("retained").context("retention closure missing")?.clone())?;
        let root = self.run_root.join("journal");
        retained.iter().filter(|item| item.reference.path.starts_with(&root)).try_fold(0u64, |bytes, item| {
            bytes.checked_add(item.reference.bytes).context("journal retained extent overflow")
        })
    }
    fn authenticate_journal_extent(&self, completion: &Completion, record: &PublicationRecord) -> Result<()> {
        if let Some(delta) = record.journal_delta_bytes {
            let final_record: FinalRecord = read(&self.attempt(completion.attempt).join("finalization.json"))?;
            let bytes = execution::authenticate_publication_extent(
                &self.run_root.join("journal"),
                &self.installed.policy,
                &record.context,
                completion,
                &record.publication,
            )?;
            ensure!(
                self.finalized_journal_bytes(&final_record)?.checked_add(delta) == Some(bytes),
                "terminal journal retained byte accounting changed"
            );
        }
        Ok(())
    }
}

fn collect(root: &Path, files: &mut Vec<Reference>, directories: &mut Vec<PathBuf>, maximum: u64) -> Result<()> {
    safe(root)?;
    let meta = root.symlink_metadata()?;
    if meta.is_dir() {
        directories.push(root.into());
        for entry in fs::read_dir(root)? {
            collect(&entry?.path(), files, directories, maximum)?;
        }
    } else {
        ensure!(meta.is_file(), "controller output is not regular");
        files.push(retention::seal_file(root, maximum)?);
    }
    Ok(())
}

impl ExecutionAdapter for Adapter {
    fn identity(&self) -> String {
        self.identity.clone()
    }
    fn authenticate_gate(
        &self,
        policy: &FrozenPolicy,
        source: &SourceSnapshot,
        stage: VerificationStage,
        outcome: &GateOutcome,
    ) -> Result<()> {
        ensure!(*policy == self.installed.policy, "gate authentication frozen policy differs");
        let record: GateRecord = read(&self.gate_record(&outcome.seal.operation_key))?;
        ensure!(
            record.source == *source && record.stage == stage && record.outcome == *outcome,
            "journal gate differs from trusted immutable result"
        );
        self.authenticated_record_receipts(&record)?;
        Ok(())
    }
    fn prepare(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        previous: Option<&SourceSnapshot>,
    ) -> Result<Preparation> {
        self.check_admission()?;
        self.reserve_attempt(context.attempt)?;
        let stage_deadline = self.deadline(context)?;
        self.bootstrap(context, stage_deadline)?;
        let input_budget = self
            .installed
            .max_checkout_bytes
            .checked_mul(8)
            .context("preparation input budget overflow")?
            .min(8 * crate::inputs::bounded::DEFAULT_BYTES);
        let (candidate, source, prepared) = crate::inputs::bounded::until(stage_deadline, input_budget, || {
            let candidate = self.candidate(context.attempt);
            if !candidate.join(".git").is_dir() {
                if let Some(previous) = previous {
                    self.restore_source(previous, context, stage_deadline)?;
                } else {
                    self.reserve_checkout(
                        &sealed(&(&context.operation_key, "candidate-checkout"))?,
                        &self.installed.source_repository,
                        &policy.baseline_commit,
                        stage_deadline,
                    )?;
                    checkout::create_until(
                        &self.installed.source_repository,
                        &candidate,
                        &policy.baseline_commit,
                        stage_deadline,
                    )?;
                }
            }
            let source = checkout::snapshot_until(&candidate, &self.paths(), stage_deadline)?;
            if let Some(previous) = previous {
                ensure!(source == *previous, "repair source differs from stopped candidate");
            }
            let prepared_path = self.run_root.join("prepared.json");
            let prepared = if prepared_path.exists() {
                self.prepared()?
            } else {
                let prepared = match &self.installed.contract {
                    Contract::Native { contracts, catalog_sha256, focused_family } => {
                        ensure!(
                            crate::inputs::bounded::digest(&candidate.join("tools/port/batch.toml"))?
                                == *catalog_sha256,
                            "frozen full catalog changed"
                        );
                        let catalog = batch::catalog_until(&candidate, stage_deadline, input_budget)?;
                        let declared =
                            contracts.iter().map(|contract| contract.family.clone()).collect::<BTreeSet<_>>();
                        ensure!(
                            !declared.is_empty()
                                && declared.len() == contracts.len()
                                && declared.contains(focused_family),
                            "native frozen gate declarations missing"
                        );
                        // With no changed source yet, every potentially unknown
                        // dependency remains affected. A closed installation must
                        // support the complete catalog before dispatching a worker.
                        ensure!(
                            catalog.families.keys().cloned().collect::<BTreeSet<_>>() == declared,
                            "preparation lacks confined adapters/provenance for the complete affected catalog"
                        );
                        let families = contracts
                            .iter()
                            .map(|contract| {
                                native::prepare_until(
                                    &candidate,
                                    &self.installed.pinned_source,
                                    contract,
                                    stage_deadline,
                                    input_budget,
                                )
                            })
                            .collect::<Result<Vec<_>>>()?;
                        ensure!(
                            families.iter().flat_map(|family| [family.candidate_path.clone()]).collect::<BTreeSet<_>>()
                                == self.paths().into_iter().collect(),
                            "native writable ownership differs from complete family sources"
                        );
                        Prepared::Native { families }
                    }
                    Contract::Infrastructure { contract } => {
                        ensure!(
                            contract.approved_files.iter().cloned().collect::<BTreeSet<_>>()
                                == self.paths().into_iter().collect(),
                            "infrastructure ownership differs"
                        );
                        Prepared::Infrastructure {
                            prepared: infrastructure::prepare_until(
                                &candidate,
                                contract,
                                stage_deadline,
                                input_budget,
                            )?,
                        }
                    }
                };
                write_once(&prepared_path, &prepared)?;
                prepared
            };
            let (dependencies, verification, pinned) = match &prepared {
                Prepared::Native { families } => (
                    sealed(&families.iter().map(|family| &family.dependencies_sha256).collect::<Vec<_>>())?,
                    sealed(&families.iter().map(|family| &family.verification_sha256).collect::<Vec<_>>())?,
                    sealed(&families.iter().map(|family| &family.source).collect::<Vec<_>>())?,
                ),
                Prepared::Infrastructure { prepared } => (
                    prepared.dependencies_sha256.clone(),
                    prepared.verification_sha256.clone(),
                    sha256("infrastructure-no-actual-c-v1"),
                ),
            };
            ensure!(
                dependencies == policy.dependencies_sha256
                    && verification == policy.verification_sha256
                    && pinned == policy.pinned_c_sha256,
                "prepared dependency, verification or pinned-C identity differs from frozen policy"
            );
            Ok((candidate, source, prepared))
        })?;
        // Readiness executes the same confined compiler/runtime before a
        // capsule exists. Its durable child keys reconcile exact retries.
        let ready = self.attempt(context.attempt).join("readiness");
        let mut ready_receipts = if previous.is_some() { self.baseline_readiness(&prepared)? } else { Vec::new() };
        let mut failed = None;
        if previous.is_none() {
            match &prepared {
                Prepared::Native { families } => {
                    self.reserve_gate(context, families.len())?;
                    let mut runtime = self.native_runtime();
                    runtime.deadline_unix_ms = Some(stage_deadline);
                    for family in families {
                        let mut child = context.clone();
                        child.operation_key = sealed(&(&context.operation_key, &family.contract.family))?;
                        let receipt = native::check_ready(
                            family,
                            &candidate,
                            &ready.join(&family.contract.family),
                            &runtime,
                            &child,
                        )?;
                        if !receipt.passed {
                            failed = Some(format!("native harness readiness stopped: {}", receipt.diagnostic));
                        }
                        ready_receipts.push(Receipt::Native {
                            prepared: family.clone(),
                            source: candidate.clone(),
                            receipt,
                        });
                        if failed.is_some() {
                            break;
                        }
                    }
                }
                Prepared::Infrastructure { prepared } => {
                    self.reserve_gate(context, 1)?;
                    let mut runtime = self.infrastructure_runtime();
                    runtime.deadline_unix_ms = Some(stage_deadline);
                    let receipt = infrastructure::check_ready(prepared, &candidate, &ready, &runtime, context)?;
                    if !receipt.passed {
                        failed = Some(format!("infrastructure harness readiness stopped: {}", receipt.diagnostic));
                    }
                    ready_receipts.push(Receipt::Infrastructure {
                        prepared: prepared.clone(),
                        source: candidate.clone(),
                        receipt,
                    });
                }
            }
        }
        // Readiness refers to baseline source. Preserve it before a replacement
        // can change the implementer checkout used by that independent proof.
        let preservation_deadline = crate::inputs::bounded::now_unix_ms()?
            .checked_add(120_000)
            .context("readiness preservation deadline overflow")?;
        crate::inputs::bounded::until(preservation_deadline, input_budget, || {
            for receipt in &mut ready_receipts {
                self.preserve_receipt(receipt)?;
            }
            write_once(&self.attempt(context.attempt).join("readiness.json"), &ready_receipts)
        })?;
        if let Some(reason) = failed {
            bail!("{reason}");
        }
        crate::inputs::bounded::until(stage_deadline, input_budget, || {
            let mut context_text = format!(
                "Approved scope: {}. Frozen dependencies: {}. Prescribed coverage: {}.\n",
                policy.scope_id, policy.dependencies_sha256, policy.verification_sha256
            );
            if let Prepared::Native { families } = &prepared {
                for family in families {
                    crate::inputs::bounded::check()?;
                    let bank =
                        crate::inputs::bounded::text(family.pinned_source.join("src").join(&family.manifest.bank))?;
                    for routine in &family.manifest.routines {
                        crate::inputs::bounded::check()?;
                        let remaining = self.installed.max_context_bytes.saturating_sub(context_text.len() as u64);
                        let definition =
                            recovered_definition_bounded(&bank, &routine.symbol, remaining.saturating_sub(1))?;
                        context_text.push_str(&definition);
                        context_text.push('\n');
                    }
                }
            }
            ensure!(
                context_text.len() as u64 <= self.installed.max_context_bytes,
                "recovered source context exceeds approved bound"
            );
            let capsule = execution::rewrite::capture_until(
                &candidate,
                CapsuleBinding {
                    run_id: context.run_id.clone(),
                    attempt: context.attempt,
                    policy_sha256: context.policy_sha256.clone(),
                    source_sha256: source.sha256.clone(),
                },
                &self.paths(),
                context_text,
                policy.rewrite_limits,
                stage_deadline,
            )?;
            Ok(Preparation { source, capsule })
        })
    }
    fn apply(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        capsule: &RewriteCapsule,
        replacements: &SourceReplacements,
    ) -> Result<SourceSnapshot> {
        self.check_admission()?;
        for file in &replacements.files {
            let compact = file.content.chars().filter(|c| !c.is_whitespace()).collect::<String>();
            ensure!(
                !compact.contains("#[test]") && !compact.contains("cfg(test)") && !compact.contains("#[path="),
                "candidate cannot supply frozen verification declarations"
            );
        }
        checkout::apply_commit_until(
            &self.candidate(context.attempt),
            context,
            capsule,
            replacements,
            policy.rewrite_limits,
            self.deadline(context)?,
        )
    }
    fn verify(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        source: &SourceSnapshot,
        stage: VerificationStage,
    ) -> Result<GateOutcome> {
        self.check_admission()?;
        self.reserve_attempt(context.attempt)?;
        let record_path = self.gate_record(&context.operation_key);
        if record_path.exists() {
            let record: GateRecord = read(&record_path)?;
            ensure!(
                record.context == *context && record.source == *source && record.stage == stage,
                "gate retry identity differs"
            );
            self.verify_record(&record)?;
            return Ok(record.outcome);
        }
        let stage_deadline = self.deadline(context)?;
        let checkout = match self.source_path(source, stage_deadline) {
            Ok(path) => path,
            Err(_) => self.restore_source(source, context, stage_deadline)?,
        };
        let output = self.attempt(context.attempt).join(format!("gate-{}", context.operation_key));
        let mut receipts = Vec::new();
        let prepared_contract = self.prepared()?;
        match &prepared_contract {
            Prepared::Native { families } => {
                let names = batch::catalog_until(&checkout, stage_deadline, self.installed.max_checkout_bytes)?
                    .families
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>();
                let plan = affected::select_until(
                    &checkout,
                    &names,
                    Some(&policy.baseline_commit),
                    stage_deadline,
                    self.installed.max_checkout_bytes,
                )?;
                write_once(&output.join("affected.json"), &plan)?;
                let selected = match stage {
                    VerificationStage::Focused => {
                        let Contract::Native { focused_family, .. } = &self.installed.contract else { unreachable!() };
                        vec![focused_family.clone()]
                    }
                    VerificationStage::Offline => plan.families(),
                    VerificationStage::Live => bail!("required confined Live replay adapter missing"),
                };
                ensure!(!selected.is_empty(), "prescribed native gate selected no families");
                self.reserve_gate(context, selected.len())?;
                let mut runtime = self.native_runtime();
                runtime.deadline_unix_ms = Some(stage_deadline);
                for name in selected {
                    if let Some(receipt) = self.reusable_receipt(&prepared_contract, &checkout, Some(&name))? {
                        receipts.push(receipt);
                        continue;
                    }
                    let prepared = families
                        .iter()
                        .find(|family| family.contract.family == name)
                        .with_context(|| format!("affected family {name} lacks a confined adapter"))?
                        .clone();
                    let mut child = context.clone();
                    child.operation_key = sealed(&(&context.operation_key, &name))?;
                    let receipt = native::run(&prepared, &checkout, &output.join(&name), &runtime, &child)?;
                    let passed = receipt.passed;
                    receipts.push(Receipt::Native { prepared, source: checkout.clone(), receipt });
                    if !passed {
                        break;
                    }
                }
            }
            Prepared::Infrastructure { prepared } => {
                ensure!(stage == VerificationStage::Focused, "infrastructure cannot publish native or Live acceptance");
                self.reserve_gate(context, 1)?;
                let mut runtime = self.infrastructure_runtime();
                runtime.deadline_unix_ms = Some(stage_deadline);
                if let Some(receipt) = self.reusable_receipt(&prepared_contract, &checkout, None)? {
                    receipts.push(receipt);
                } else {
                    let receipt = infrastructure::run(prepared, &checkout, &output, &runtime, context)?;
                    receipts.push(Receipt::Infrastructure {
                        prepared: prepared.clone(),
                        source: checkout.clone(),
                        receipt,
                    });
                }
            }
        }
        let passed = receipts.iter().all(|receipt| match receipt {
            Receipt::Native { receipt, .. } => receipt.passed,
            Receipt::Infrastructure { receipt, .. } => receipt.passed,
        });
        let diagnostic = receipts
            .iter()
            .find_map(|receipt| match receipt {
                Receipt::Native { receipt, .. } if !receipt.passed => Some(receipt.diagnostic.clone()),
                Receipt::Infrastructure { receipt, .. } if !receipt.passed => Some(receipt.diagnostic.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let evidence_bytes = receipts.iter().try_fold(0u64, |n, receipt| {
            n.checked_add(match receipt {
                Receipt::Native { receipt, .. } => receipt.evidence_bytes,
                Receipt::Infrastructure { receipt, .. } => receipt.evidence_bytes,
            })
            .context("gate evidence overflow")
        })?;
        let outcome = GateOutcome {
            seal: self.seal_outcome(context, &source.sha256, &(context, source, stage, &receipts))?,
            passed,
            diagnostic,
            evidence_bytes,
        };
        let record = GateRecord { context: context.clone(), source: source.clone(), stage, receipts, outcome };
        self.verify_record(&record)?;
        write_once(&record_path, &record)?;
        Ok(record.outcome)
    }
    fn integrate(
        &mut self,
        context: &OperationContext,
        _policy: &FrozenPolicy,
        candidate: &SourceSnapshot,
    ) -> Result<SourceSnapshot> {
        self.check_admission()?;
        let stage_deadline = self.deadline(context)?;
        let integration = self.integration(context.attempt);
        if !integration.join(".git").is_dir() {
            self.reserve_checkout(
                &sealed(&(&context.operation_key, "integration-checkout"))?,
                &self.installed.integration_repository,
                &self.installed.integration_baseline,
                stage_deadline,
            )?;
            checkout::create_until(
                &self.installed.integration_repository,
                &integration,
                &self.installed.integration_baseline,
                stage_deadline,
            )?;
        }
        checkout::integrate_until(
            &self.source_path(candidate, stage_deadline)?,
            &integration,
            context,
            candidate,
            &self.installed.integration_baseline,
            &self.paths(),
            stage_deadline,
        )
    }
    fn finalize(
        &mut self,
        context: &OperationContext,
        _policy: &FrozenPolicy,
        request: &FinalizationRequest,
        journal: &FinalizationJournal<'_>,
    ) -> Result<Finalization> {
        self.reserve_attempt(context.attempt)?;
        let path = self.attempt(context.attempt).join("finalization.json");
        if path.exists() {
            let saved: FinalRecord = read(&path)?;
            ensure!(saved.context == *context && saved.request == *request, "finalization retry differs");
            self.authenticate(&_policy.clone(), request, &saved.result)?;
            return Ok(saved.result);
        }
        let complete = request.disposition == Disposition::Complete;
        let journal_reservation = if complete { 0 } else { journal.max_additional_bytes()? };
        let retention_limit = request
            .remaining_evidence_bytes
            .checked_sub(MAX_RECORD)
            .and_then(|bytes| bytes.checked_sub(journal_reservation))
            .context("no extent remains for final controller result and journal")?;
        if let Some(interrupted) = &request.interrupted_operation
            && operation::reference(&self.operations(), interrupted).is_ok()
        {
            operation::stop(&self.operations(), interrupted)?;
        }
        let attempt = self.attempt(context.attempt);
        fs::create_dir_all(&attempt)?;
        // Stop all exact active runner intents belonging to this attempt before
        // the utility or checkout retirement can change their source inputs.
        let operation_root = self.operations();
        if operation_root.is_dir() {
            for entry in fs::read_dir(&operation_root)? {
                let operation_path = entry?.path();
                let intent_path = operation_path.join("intent.json");
                if !intent_path.is_file() {
                    continue;
                }
                let intent: serde_json::Value = read(&intent_path)?;
                if let Some(value) = intent.get("context") {
                    let owned: OperationContext = serde_json::from_value(value.clone())?;
                    if owned.attempt == context.attempt && owned.run_id == context.run_id {
                        operation::stop(&operation_root, &owned)?;
                    }
                }
            }
        }
        let model_root = self.model_operations();
        if model_root.is_dir() {
            for entry in fs::read_dir(&model_root)? {
                let intent_path = entry?.path().join("intent.json");
                if !intent_path.is_file() {
                    continue;
                }
                let intent: serde_json::Value = read(&intent_path)?;
                let owned: OperationContext =
                    serde_json::from_value(intent.get("context").context("model intent context missing")?.clone())?;
                ensure!(owned.run_id == context.run_id, "foreign model intent in run");
                if owned.attempt == context.attempt {
                    model_rewrite::stop(&model_root, &owned)?;
                }
            }
        }
        let intent_path = attempt.join("finalization-intent.json");
        let intent: FinalIntent = if intent_path.exists() {
            let intent: FinalIntent = read(&intent_path)?;
            ensure!(intent.context == *context && intent.request == *request, "finalization intent changed on retry");
            intent
        } else {
            let mut records = Vec::new();
            for gate in &request.gates {
                let mut record: GateRecord = read(&self.gate_record(&gate.seal.operation_key))?;
                ensure!(record.outcome == *gate, "finalization gate evidence changed");
                self.verify_record(&record)?;
                for receipt in &mut record.receipts {
                    self.preserve_receipt(receipt)?;
                }
                records.push(record);
            }
            let readiness_path = attempt.join("readiness.json");
            let mut readiness: Vec<Receipt> = if readiness_path.exists() { read(&readiness_path)? } else { vec![] };
            for receipt in &mut readiness {
                self.preserve_receipt(receipt)?;
            }
            let mut revisions = BTreeMap::new();
            for (name, baseline) in
                [("candidate", &_policy.baseline_commit), ("integration", &self.installed.integration_baseline)]
            {
                let producer = attempt.join(name);
                if producer.join(".git").is_dir() {
                    let _leases = checkout::retirement_leases(&producer)?;
                    match checkout::snapshot(&producer, &self.paths()) {
                        Ok(_) => {
                            drop(_leases);
                            let archive = checkout::retain_revision(
                                &producer,
                                &attempt.join("revisions").join(name),
                                baseline,
                                &self.paths(),
                                self.installed.max_checkout_bytes,
                            )?;
                            revisions.insert(name.into(), archive);
                        }
                        Err(error) if request.disposition != Disposition::Complete => {
                            self.preserve_partial_source(&producer, context, &error)?;
                        }
                        Err(error) => return Err(error),
                    }
                } else if producer.exists() && request.disposition != Disposition::Complete {
                    let _leases = checkout::retirement_leases(&producer)?;
                    self.preserve_partial_source(
                        &producer,
                        context,
                        &anyhow::anyhow!("source checkout creation was incomplete"),
                    )?;
                }
            }
            let mut prior_finalizations = Vec::new();
            for number in 1..context.attempt {
                let path = self.attempt(number).join("finalization.json");
                let previous: FinalRecord = read(&path)?;
                ensure!(
                    previous.context.attempt == number
                        && previous.context.run_id == context.run_id
                        && previous.context.policy_sha256 == context.policy_sha256,
                    "historical failed attempt lineage changed"
                );
                self.authenticate(&_policy.clone(), &previous.request, &previous.result)?;
                prior_finalizations.push(retention::seal_file(&path, MAX_RECORD)?);
            }
            let intent = FinalIntent {
                context: context.clone(),
                request: request.clone(),
                retained_gates: records,
                retained_readiness: readiness,
                revisions,
                prior_finalizations,
            };
            write_once(&intent_path, &intent)?;
            intent
        };
        let records = &intent.retained_gates;
        for record in records {
            for receipt in &record.receipts {
                self.authenticate_receipt(receipt)?;
            }
        }
        for receipt in &intent.retained_readiness {
            self.authenticate_receipt(receipt)?;
        }
        for archive in intent.revisions.values() {
            checkout::authenticate_revision(archive)?;
        }
        for reference in &intent.prior_finalizations {
            retention::authenticate_reference(reference, MAX_RECORD)?;
        }
        write_once(&attempt.join("retained-gates.json"), records)?;
        write_once(&attempt.join("retained-readiness.json"), &intent.retained_readiness)?;
        let first_path = attempt.join("project-cleanup-1.json");
        if !first_path.exists() {
            let receipt =
                cleanup::cleanup(cleanup::CleanupRequest { root: attempt.clone(), apply: true, deduplicate: false })?;
            write_once(&first_path, &receipt)?;
        }
        let second_path = attempt.join("project-cleanup-2.json");
        if !second_path.exists() {
            let receipt =
                cleanup::cleanup(cleanup::CleanupRequest { root: attempt.clone(), apply: true, deduplicate: true })?;
            write_once(&second_path, &receipt)?;
        }
        for record in records {
            for receipt in &record.receipts {
                self.authenticate_receipt(receipt)?;
            }
        }
        let mut reclaimed = 0u64;
        for name in ["candidate", "integration"] {
            reclaimed = reclaimed
                .checked_add(self.retirement(&attempt.join(name), context)?)
                .context("checkout cleanup overflow")?;
        }
        let mut files = Vec::new();
        let mut directories = Vec::new();
        for entry in fs::read_dir(&attempt)? {
            let path = entry?.path();
            if path.file_name().is_some_and(|name| name == "candidate" || name == "integration") {
                continue;
            }
            collect(&path, &mut files, &mut directories, request.remaining_evidence_bytes)?;
        }
        for gate in &request.gates {
            files.push(retention::seal_file(&self.gate_record(&gate.seal.operation_key), MAX_RECORD)?);
        }
        for record in records {
            for receipt in &record.receipts {
                files.extend(self.receipt_references(receipt)?);
            }
        }
        for receipt in &intent.retained_readiness {
            files.extend(self.receipt_references(receipt)?);
        }
        for archive in intent.revisions.values() {
            files.extend(checkout::revision_references(archive)?);
        }
        files.extend(intent.prior_finalizations.iter().cloned());
        let journal_root = self.run_root.join("journal");
        if journal_root.is_dir() {
            collect(&journal_root, &mut files, &mut directories, request.remaining_evidence_bytes)?;
        }
        if self.run_root.join("prepared.json").is_file() {
            files.push(retention::seal_file(&self.run_root.join("prepared.json"), MAX_RECORD)?);
        }
        for root in [self.run_root.join("deadlines"), self.run_root.join("allocations")] {
            if root.is_dir() {
                collect(&root, &mut files, &mut directories, request.remaining_evidence_bytes)?;
            }
        }
        let snapshots = self.run_root.join("source-snapshots");
        if snapshots.is_dir() {
            collect(&snapshots, &mut files, &mut directories, request.remaining_evidence_bytes)?;
        }
        // Follow the actual immutable operation closure, including retained
        // manifests, start identities and failure teardown observations.
        if operation_root.is_dir() {
            for entry in fs::read_dir(&operation_root)? {
                let intent_path = entry?.path().join("intent.json");
                if !intent_path.is_file() {
                    continue;
                }
                let intent: serde_json::Value = read(&intent_path)?;
                if let Some(value) = intent.get("context") {
                    let owned: OperationContext = serde_json::from_value(value.clone())?;
                    if owned.attempt == context.attempt && owned.run_id == context.run_id {
                        files.extend(operation::references(&operation_root, &owned)?);
                    }
                }
            }
        }
        if model_root.is_dir() {
            for entry in fs::read_dir(&model_root)? {
                let intent_path = entry?.path().join("intent.json");
                if !intent_path.is_file() {
                    continue;
                }
                let intent: serde_json::Value = read(&intent_path)?;
                let owned: OperationContext =
                    serde_json::from_value(intent.get("context").context("model intent context missing")?.clone())?;
                if owned.attempt == context.attempt && owned.run_id == context.run_id {
                    files.extend(model_rewrite::references(&model_root, &owned)?);
                }
            }
        }
        let kind = if complete { RetainedKind::Accepted } else { RetainedKind::Failed };
        let mut refs = BTreeMap::new();
        for reference in files {
            if let Some(saved) = refs.insert(reference.path.clone(), reference.clone()) {
                ensure!(saved == reference, "conflicting finalization closure");
            }
        }
        let retained = refs.into_values().map(|reference| Retained { kind, reference }).collect::<Vec<_>>();
        // The final record and later publication are outside the immutable
        // closure because they contain its digest. Reserve their full bounded
        // extent before finalization writes or retires anything.
        let final_record_budget = MAX_RECORD;
        let retention = retention::finalize(&self.run_root, context, &retained, &[], retention_limit)?;
        for record in records {
            for receipt in &record.receipts {
                self.authenticate_receipt(receipt)?;
            }
        }
        for receipt in &intent.retained_readiness {
            self.authenticate_receipt(receipt)?;
        }
        let source = request.source.as_ref().map(|source| source.sha256.as_str()).unwrap_or(&context.policy_sha256);
        let mut record = FinalRecord {
            context: context.clone(),
            request: request.clone(),
            retention: retention.clone(),
            journal_delta_bytes: if complete { None } else { Some(0) },
            journal_reserved_bytes: if complete { None } else { Some(journal_reservation) },
            result: Finalization {
                seal: self.seal_outcome(context, source, &retention)?,
                cleanup_complete: true,
                retained_authenticatable: true,
                worker_retired: !self.candidate(context.attempt).exists()
                    && !self.integration(context.attempt).exists(),
                retained_bytes: retention.retained_bytes,
                reclaimed_bytes: reclaimed
                    .checked_add(retention.reclaimed_bytes)
                    .context("cleanup measured extent overflow")?,
            },
        };
        for _ in 0..32 {
            if !complete {
                record.journal_delta_bytes = Some(journal.additional_bytes(&record.result)?);
            }
            let bytes = serde_json::to_vec(&record)?.len() as u64;
            ensure!(bytes <= final_record_budget, "final result metadata budget exceeded");
            let total = retention
                .retained_bytes
                .checked_add(bytes)
                .and_then(|bytes| bytes.checked_add(record.journal_delta_bytes.unwrap_or(0)))
                .context("final retained byte overflow")?;
            ensure!(
                total <= request.remaining_evidence_bytes,
                "final result and terminal journal exceed remaining extent"
            );
            if total == record.result.retained_bytes {
                write_once(&path, &record)?;
                return Ok(record.result);
            }
            record.result.retained_bytes = total;
        }
        bail!("final result retained-byte accounting did not converge")
    }
    fn publish(
        &mut self,
        context: &OperationContext,
        policy: &FrozenPolicy,
        completion: &Completion,
        journal: &PublicationJournal<'_>,
    ) -> Result<Publication> {
        self.check_admission()?;
        self.reserve_attempt(context.attempt)?;
        let root = self.attempt(context.attempt).join("publication");
        let accepted = root.join("accepted.json");
        if accepted.exists() {
            let saved: PublicationRecord = read(&accepted)?;
            ensure!(saved.context == *context && saved.completion == *completion, "publication retry differs");
            self.resume_publication(policy, completion, &saved.publication)?;
            return Ok(saved.publication);
        }
        let mut publication = Publication {
            seal: self.seal_outcome(context, &completion.integrated.sha256, completion)?,
            observation_ids: vec![format!("{}-integrated", context.operation_key)],
            retained_bytes: 0,
        };
        let stages = self.stages(context, policy, completion, &publication);
        let completion_bytes = serde_json::to_vec(completion)?;
        let stages_bytes = serde_json::to_vec(&stages)?;
        let projections =
            [(root.join("completion.json"), &completion_bytes), (root.join("stages.json"), &stages_bytes)]
                .into_iter()
                .map(|(path, bytes)| Reference { path, sha256: sha256(bytes), bytes: bytes.len() as u64 })
                .collect::<Vec<_>>();
        let mut record = PublicationRecord {
            context: context.clone(),
            completion: completion.clone(),
            publication: publication.clone(),
            projections,
            managed: None,
            journal_delta_bytes: Some(0),
        };
        let final_record: FinalRecord = read(&self.attempt(context.attempt).join("finalization.json"))?;
        let publication_reservation = if matches!(self.installed.admission, Admission::Managed { .. }) {
            24 * MAX_RECORD + 3 * MAX_RECORD + JOURNAL_RESERVATION
        } else {
            3 * MAX_RECORD + JOURNAL_RESERVATION
        };
        ensure!(
            completion
                .finalization
                .retained_bytes
                .checked_add(publication_reservation)
                .is_some_and(|bytes| bytes <= final_record.request.remaining_evidence_bytes),
            "publication metadata and managed admission extent cannot be reserved before side effects"
        );
        write_once(&root.join("completion.json"), completion)?;
        write_once(&root.join("stages.json"), &stages)?;
        if matches!(self.installed.admission, Admission::Managed { .. }) {
            let managed =
                crate::translation_migration::publish(&self.installed, completion, &root.join("stages.json"))?;
            record.publication.observation_ids = managed.observation_ids.clone();
            record.managed = Some(managed);
        }
        for _ in 0..32 {
            record.journal_delta_bytes = Some(
                journal
                    .retained_bytes(&record.publication)?
                    .checked_sub(self.finalized_journal_bytes(&final_record)?)
                    .context("publication journal extent regressed")?,
            );
            let bytes = serde_json::to_vec(&record)?.len() as u64;
            ensure!(
                bytes <= MAX_RECORD
                    && completion_bytes.len() as u64 <= MAX_RECORD
                    && stages_bytes.len() as u64 <= MAX_RECORD,
                "publication metadata extent exceeded"
            );
            let total = bytes
                .checked_add(completion_bytes.len() as u64)
                .and_then(|n| n.checked_add(stages_bytes.len() as u64))
                .and_then(|bytes| bytes.checked_add(record.managed.as_ref().map_or(0, |value| value.retained_bytes)))
                .and_then(|bytes| bytes.checked_add(record.journal_delta_bytes.unwrap_or(0)))
                .context("publication retained byte overflow")?;
            if total == record.publication.retained_bytes {
                break;
            }
            record.publication.retained_bytes = total;
        }
        ensure!(
            completion
                .finalization
                .retained_bytes
                .checked_add(record.publication.retained_bytes)
                .is_some_and(|bytes| bytes <= final_record.request.remaining_evidence_bytes),
            "publication exceeds remaining evidence budget before writing"
        );
        write_once(&accepted, &record)?;
        publication = record.publication;
        self.authenticate_publication(policy, completion, &publication)?;
        Ok(publication)
    }
    fn authenticate(&self, _policy: &FrozenPolicy, request: &FinalizationRequest, result: &Finalization) -> Result<()> {
        let mut matching = None;
        for entry in fs::read_dir(&self.run_root)? {
            let path = entry?.path().join("finalization.json");
            if path.is_file() {
                let record: FinalRecord = read(&path)?;
                if record.result == *result {
                    ensure!(
                        record_bytes(&path)? == serde_json::to_vec(&record)?,
                        "final controller result is not the original canonical bytes"
                    );
                    matching = Some(record);
                    break;
                }
            }
        }
        let record = matching.context("retained finalization is missing")?;
        ensure!(record.request == *request, "retained finalization request changed");
        let retention_limit = request
            .remaining_evidence_bytes
            .checked_sub(MAX_RECORD)
            .and_then(|bytes| bytes.checked_sub(record.journal_reserved_bytes.unwrap_or(0)))
            .context("finalization reserved extent changed")?;
        retention::authenticate_finalization(&record.retention, retention_limit)?;
        let source =
            request.source.as_ref().map(|source| source.sha256.as_str()).unwrap_or(&record.context.policy_sha256);
        ensure!(
            result.seal == self.seal_outcome(&record.context, source, &record.retention)?,
            "retained finalization seal changed"
        );
        let records: Vec<GateRecord> = read(&self.attempt(record.context.attempt).join("retained-gates.json"))?;
        ensure!(records.len() == request.gates.len(), "retained gate count changed");
        for (saved, gate) in records.iter().zip(&request.gates) {
            ensure!(saved.outcome == *gate, "retained gate result changed");
            for receipt in &saved.receipts {
                self.authenticate_receipt(receipt)?;
            }
        }
        let readiness: Vec<Receipt> = read(&self.attempt(record.context.attempt).join("retained-readiness.json"))?;
        for receipt in &readiness {
            self.authenticate_receipt(receipt)?;
        }
        let intent: FinalIntent = read(&self.attempt(record.context.attempt).join("finalization-intent.json"))?;
        ensure!(
            intent.context == record.context && intent.request == record.request,
            "retained finalization intent differs from accepted result"
        );
        ensure!(
            intent.prior_finalizations.len() == record.context.attempt.saturating_sub(1) as usize,
            "retained historical failed attempt closure is incomplete"
        );
        for (index, reference) in intent.prior_finalizations.iter().enumerate() {
            retention::authenticate_reference(reference, MAX_RECORD)?;
            let previous: FinalRecord = read(&reference.path)?;
            ensure!(
                reference.path == self.attempt(index as u32 + 1).join("finalization.json")
                    && previous.context.attempt == index as u32 + 1
                    && previous.context.run_id == record.context.run_id
                    && previous.context.policy_sha256 == record.context.policy_sha256,
                "historical finalization identity changed"
            );
            self.authenticate(&_policy.clone(), &previous.request, &previous.result)?;
        }
        for archive in intent.revisions.values() {
            checkout::authenticate_revision(archive)?;
        }
        if let Some(delta) = record.journal_delta_bytes {
            let expected_reservation = delta
                .checked_add(20 - result.retained_bytes.to_string().len() as u64)
                .and_then(|bytes| bytes.checked_add(20 - result.reclaimed_bytes.to_string().len() as u64))
                .context("stopped journal reservation overflow")?;
            ensure!(
                record.request.disposition != Disposition::Complete
                    && record.journal_reserved_bytes == Some(expected_reservation)
                    && execution::authenticate_finalization_tail(
                        &self.run_root.join("journal"),
                        _policy,
                        &record.context,
                        &record.request,
                        &record.result
                    )? == delta,
                "stopped terminal journal byte accounting changed"
            );
        }
        ensure!(
            record
                .retention
                .retained_bytes
                .checked_add(serde_json::to_vec(&record)?.len() as u64)
                .and_then(|bytes| bytes.checked_add(record.journal_delta_bytes.unwrap_or(0)))
                == Some(result.retained_bytes),
            "final result retained byte accounting changed"
        );
        self.authentic_retirement(record.context.attempt)?;
        Ok(())
    }
    fn authenticate_publication(
        &self,
        _policy: &FrozenPolicy,
        completion: &Completion,
        publication: &Publication,
    ) -> Result<()> {
        let record = self.publication_record(completion, publication)?;
        for reference in &record.projections {
            retention::authenticate_reference(reference, MAX_RECORD)?;
        }
        if let Some(managed) = &record.managed {
            crate::translation_migration::authenticate_publication(&self.installed, completion, managed)?;
        }
        Ok(())
    }
    fn resume_publication(
        &mut self,
        policy: &FrozenPolicy,
        completion: &Completion,
        publication: &Publication,
    ) -> Result<()> {
        let record = self.publication_record(completion, publication)?;
        if let Some(managed) = &record.managed {
            crate::translation_migration::resume_publication(&self.installed, completion, managed)?;
        }
        let stages = self.stages(&record.context, policy, completion, publication);
        let bytes = [serde_json::to_vec(completion)?, serde_json::to_vec(&stages)?];
        for (reference, bytes) in record.projections.iter().zip(&bytes) {
            if reference.path.exists() {
                retention::authenticate_reference(reference, MAX_RECORD)?;
            } else {
                ensure!(
                    sha256(bytes) == reference.sha256 && bytes.len() as u64 == reference.bytes,
                    "missing projection differs from accepted record"
                );
                // Repair exact JSON bytes, not a JSON-encoded byte vector.
                write_bytes(&reference.path, bytes)?;
            }
        }
        self.authenticate_publication(policy, completion, publication)
    }
}

/// Read-only authentication of a retained complete supervisor execution. Its
/// installation describes that exact closed scope and never approves a new
/// managed claim, source, policy, or publication.
pub(crate) fn authenticate_retained_proof(installation_path: &Path, completion_path: &Path) -> Result<FrozenPolicy> {
    protected(installation_path)?;
    let installed: Installation = read(installation_path)?;
    ensure!(
        installed.schema == "translation-supervisor-installation-v1"
            && matches!(installed.admission, Admission::IsolatedProof { .. }),
        "proof must retain its isolated installation"
    );
    installed.policy.validate()?;
    ensure!(
        installed.policy.adapter_sha256 == installation_identity(&installed)?,
        "retained installation identity changed"
    );
    let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
    let run_root = installed.state_root.join(run_id);
    safe(&run_root)?;
    let adapter = Adapter { identity: installed.policy.adapter_sha256.clone(), installed: installed.clone(), run_root };
    let completion: Completion = read(completion_path)?;
    ensure!(completion.policy_sha256 == sealed(&installed.policy)?, "proof completion policy changed");
    ensure!(
        completion_path == adapter.attempt(completion.attempt).join("publication/completion.json"),
        "proof completion path differs from exact publication"
    );
    let final_record: FinalRecord = read(&adapter.attempt(completion.attempt).join("finalization.json"))?;
    ensure!(
        final_record.result == completion.finalization && final_record.request.disposition == Disposition::Complete,
        "proof completion is not an accepted finalization"
    );
    adapter.authenticate(&installed.policy, &final_record.request, &completion.finalization)?;
    let publication: PublicationRecord = read(&adapter.attempt(completion.attempt).join("publication/accepted.json"))?;
    adapter.authenticate_publication(&installed.policy, &completion, &publication.publication)?;
    adapter.authenticate_journal_extent(&completion, &publication)?;
    execution::authenticate_finalized_history(&adapter.run_root.join("journal"), &installed.policy, &adapter)?;
    execution::authenticate_completion_history(
        &adapter.run_root.join("journal"),
        &installed.policy,
        &completion,
        &publication.publication,
    )?;
    Ok(installed.policy)
}

pub(crate) fn authenticate_completion(
    installed: &Installation,
    completion: &Completion,
) -> Result<AuthenticatedCompletion> {
    installed.policy.validate()?;
    ensure!(
        installed.policy.adapter_sha256 == installation_identity(installed)?
            && completion.policy_sha256 == sealed(&installed.policy)?,
        "completion frozen installation differs"
    );
    let run_id = sha256(format!("translation-execution-run-v1:{}", sealed(&installed.policy)?));
    ensure!(
        completion.run_id == run_id
            && completion.attempt > 0
            && completion.attempt <= installed.policy.budgets.max_attempts,
        "completion evaluation identity differs"
    );
    let adapter = Adapter {
        installed: installed.clone(),
        identity: installed.policy.adapter_sha256.clone(),
        run_root: installed.state_root.join(run_id),
    };
    let final_path = adapter.attempt(completion.attempt).join("finalization.json");
    let final_record: FinalRecord = read(&final_path)?;
    ensure!(
        final_record.result == completion.finalization
            && final_record.request.disposition == Disposition::Complete
            && final_record.request.source.as_ref() == Some(&completion.integrated)
            && final_record.request.gates == completion.gates
            && completion.gates.iter().all(|gate| gate.passed)
            && completion.gates.len() == installed.policy.verification_stages.len() * 2,
        "completion differs from accepted prescribed finalization"
    );
    adapter.authenticate(&installed.policy, &final_record.request, &completion.finalization)?;
    let intent: FinalIntent = read(&adapter.attempt(completion.attempt).join("finalization-intent.json"))?;
    let candidate_revision = intent.revisions.get("candidate").context("candidate revision archive missing")?.clone();
    let integrated_revision =
        intent.revisions.get("integration").context("integrated revision archive missing")?.clone();
    ensure!(
        candidate_revision.source == completion.candidate
            && candidate_revision.baseline == installed.policy.baseline_commit
            && integrated_revision.source == completion.integrated
            && integrated_revision.baseline == installed.integration_baseline,
        "submitted and integrated source archive identities differ"
    );
    let mut native_gates = Vec::new();
    let mut references = BTreeMap::new();
    let manifest: serde_json::Value = read(&final_record.retention.manifest.path)?;
    let retained: Vec<Retained> =
        serde_json::from_value(manifest.get("retained").context("retention closure missing")?.clone())?;
    for reference in retained.into_iter().map(|item| item.reference).chain([
        final_record.retention.manifest.clone(),
        final_record.retention.cleanup_records[0].clone(),
        final_record.retention.cleanup_records[1].clone(),
        retention::seal_file(&final_path, MAX_RECORD)?,
    ]) {
        if let Some(prior) = references.insert(reference.path.clone(), reference.clone()) {
            ensure!(prior == reference, "completion reference closure conflicts");
        }
    }
    for (index, outcome) in completion.gates.iter().enumerate() {
        let phase = if index < installed.policy.verification_stages.len() {
            ExecutionPhase::Candidate
        } else {
            ExecutionPhase::Integration
        };
        let source = if phase == ExecutionPhase::Candidate { &completion.candidate } else { &completion.integrated };
        let stage = installed.policy.verification_stages[index % installed.policy.verification_stages.len()];
        let record: GateRecord = read(&adapter.gate_record(&outcome.seal.operation_key))?;
        ensure!(
            record.context.phase == phase
                && record.context.attempt == completion.attempt
                && record.context.run_id == completion.run_id
                && record.context.policy_sha256 == completion.policy_sha256
                && record.source == *source
                && record.stage == stage
                && record.outcome == *outcome,
            "completion gate source-stage identity changed"
        );
        for receipt in adapter.authenticated_record_receipts(&record)? {
            let Receipt::Native { prepared, source: path, receipt } = receipt else {
                continue;
            };
            let verdict = native::authenticate(&prepared, &path, &receipt)?;
            native_gates.push(AuthenticatedNativeGate {
                phase,
                stage,
                source: source.clone(),
                prepared,
                receipt,
                verdict,
            });
        }
    }
    Ok(AuthenticatedCompletion {
        policy: installed.policy.clone(),
        candidate_revision,
        integrated_revision,
        native_gates,
        references: references.into_values().collect(),
    })
}

pub(crate) fn authenticate_managed_completion(
    installed: &Installation,
    completion: &Completion,
) -> Result<AuthenticatedCompletion> {
    ensure!(
        matches!(installed.admission, Admission::Managed { .. }),
        "managed completion requires installed original lineage"
    );
    authenticate_completion(installed, completion)
}

/// Called only while the managed publisher holds original assignment ownership
/// and the current integration role. Git imports/CAS consume retained archives;
/// verification producers are never invoked by publication or recovery.
pub(crate) fn publish_managed_source(
    installed: &Installation,
    completion: &Completion,
    authenticated: &AuthenticatedCompletion,
    controller_projections: &checkout::ControllerProjection,
) -> Result<Vec<Reference>> {
    let Admission::Managed { checkout: owner, .. } = &installed.admission else {
        bail!("shared source publication requires managed integration ownership");
    };
    ensure!(
        authenticated.policy == installed.policy
            && authenticated.candidate_revision.source == completion.candidate
            && authenticated.integrated_revision.source == completion.integrated
            && installed.integration_repository == *owner,
        "shared publication differs from exact installed source identities"
    );
    let context = OperationContext {
        run_id: completion.run_id.clone(),
        attempt: completion.attempt,
        policy_sha256: completion.policy_sha256.clone(),
        phase: ExecutionPhase::Publication,
        operation_key: sha256(format!("{}:{}:managed-source-publication", completion.run_id, completion.attempt)),
    };
    let adapter = Adapter {
        installed: installed.clone(),
        identity: installed.policy.adapter_sha256.clone(),
        run_root: installed.state_root.join(&completion.run_id),
    };
    let root = adapter.attempt(completion.attempt).join("managed-source-publication");
    checkout::publish_revisions_with_controller_until(
        owner,
        &authenticated.candidate_revision,
        &authenticated.integrated_revision,
        &context,
        &adapter.paths(),
        &root,
        controller_projections,
        adapter.deadline(&context)?,
    )
}

pub(crate) fn authenticate_managed_source(
    installed: &Installation,
    completion: &Completion,
    authenticated: &AuthenticatedCompletion,
    controller_projections: &checkout::ControllerProjection,
) -> Result<()> {
    let Admission::Managed { checkout: owner, .. } = &installed.admission else {
        bail!("shared source authentication requires managed integration ownership");
    };
    ensure!(
        authenticated.policy == installed.policy
            && authenticated.integrated_revision.source == completion.integrated
            && installed.integration_repository == *owner,
        "shared source authentication differs from exact installed identities"
    );
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
    checkout::authenticate_shared_revision_until(
        owner,
        &authenticated.integrated_revision,
        &installed.policy.approved_files.keys().cloned().collect::<Vec<_>>(),
        controller_projections,
        now.checked_add(120_000).context("shared read-only authentication deadline overflow")?,
    )
}

fn recovered_definition_bounded(bank: &str, symbol: &str, max_bytes: u64) -> Result<String> {
    crate::inputs::bounded::check()?;
    ensure!(family::identifier(symbol), "recovered C symbol is invalid");
    let masked = crate::source_inventory::clean_bounded(bank)?;
    let token = format!("{symbol}(");
    let mut starts = Vec::new();
    for (offset, _) in String::from_utf8_lossy(&masked).match_indices(&token) {
        crate::inputs::bounded::check()?;
        let line_start = masked[..offset].iter().rposition(|byte| *byte == b'\n').map_or(0, |at| at + 1);
        let Some(brace) = masked[offset..].iter().position(|byte| b";{}".contains(byte)) else {
            continue;
        };
        let brace = offset + brace;
        if masked[brace] != b'{' {
            continue;
        }
        let prefix = String::from_utf8_lossy(&masked[line_start..offset]);
        if prefix.trim().is_empty() || prefix.contains('=') {
            continue;
        }
        let mut end = brace + 1;
        let mut depth = 1u32;
        while end < masked.len() && depth > 0 {
            if end.is_multiple_of(65536) {
                crate::inputs::bounded::check()?;
            }
            if masked[end] == b'{' {
                depth += 1;
            }
            if masked[end] == b'}' {
                depth -= 1;
            }
            end += 1;
        }
        ensure!(depth == 0, "recovered C routine body is unbalanced");
        ensure!(
            end.saturating_sub(line_start) as u64 <= max_bytes,
            "recovered C definition exceeds remaining context budget"
        );
        ensure!(starts.is_empty(), "recovered C definition is ambiguous: {symbol}");
        starts.push(bank[line_start..end].to_owned());
    }
    ensure!(starts.len() == 1, "recovered C definition is missing or ambiguous: {symbol}");
    Ok(starts.remove(0))
}

pub(crate) fn cli(args: &[String]) -> Result<()> {
    ensure!(
        args.len() >= 3 && args[0] == "--installation",
        "usage: translation-execute --installation INSTALLED.json inspect|resume|run|retry|repair|cancel [REASON]"
    );
    let path = Path::new(&args[1]);
    protected(path)?;
    let installed: Installation = read(path)?;
    let action = args[2].as_str();
    ensure!(
        matches!(action, "inspect" | "resume" | "run" | "retry" | "repair" | "cancel"),
        "unknown translation supervisor action"
    );
    let reason_required = matches!(action, "retry" | "repair" | "cancel");
    ensure!(args.len() == if reason_required { 4 } else { 3 }, "unexpected translation supervisor arguments");
    emit(&execute(installed, action, args.get(3).map(String::as_str))?)
}

/// Shared lifecycle used by the single-scope CLI and ordered Rust launcher.
pub(crate) fn execute(installed: Installation, action: &str, reason: Option<&str>) -> Result<View> {
    if action == "inspect" {
        let adapter = Adapter::existing(installed.clone())?;
        return Supervisor::inspect_retained(&adapter.run_root.join("journal"), installed.policy, adapter.clone());
    }
    let adapter = Adapter::new(installed.clone())?;
    let mut supervisor =
        Supervisor::open(&adapter.run_root.join("journal"), installed.policy.clone(), adapter.clone())?;
    let view = match action {
        "resume" => supervisor.advance()?,
        "run" => {
            ensure!(
                !installed.rewrite.recorded,
                "recorded proof invocations are created by the bounded Rust proof, not operator argv"
            );
            supervisor.run_with_rewrite(|adapter, context, capsule| adapter.rewrite(context, capsule, None))?
        }
        "retry" => supervisor.retry(reason.context("retry requires operator reason")?)?,
        "repair" => supervisor.repair(reason.context("repair requires operator reason")?)?,
        "cancel" => supervisor.cancel(reason.context("cancel requires operator reason")?)?,
        _ => bail!("unknown translation supervisor action"),
    };
    // Keep all reporting on accepted durable outcomes. Inspect/resume cannot
    // execute a producer after a stopped or completed evaluation.
    Ok(view)
}

#[cfg(test)]
#[path = "translation_execution_tests.rs"]
pub(crate) mod tests;

#[cfg(test)]
#[path = "translation_managed_tests.rs"]
mod managed_tests;
