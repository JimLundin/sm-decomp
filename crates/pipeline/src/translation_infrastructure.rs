//! Frozen standalone Rust infrastructure gates. Both rustc and the test binary
//! execute as bounded confined producers; the trusted supervisor owns receipts.
use crate::{acquire_lease_until, atomic_toml, builds, digest, family, family_model::Asset, inputs};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use translation_execution::{
    OperationContext,
    budget::ByteBudget,
    confinement::{self, Program, ReadOnlyBind, RunLimits},
    model::{digest_valid, sealed},
    operation,
    retention::{self, Reference},
};

const MAX_FILES: usize = 128;
const MAX_SOURCE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RECORD_BYTES: u64 = 256 * 1024;
const MAX_GATE_BYTES: u64 = 512 * 1024 * 1024;
const ACCEPTANCE_MODEL: &str = "reviewed-in-process-rust-tests";

fn compiler_arguments(test_root: &str) -> Vec<String> {
    vec![
        "--edition=2024".into(),
        "--test".into(),
        "--crate-name=translation_infrastructure_gate".into(),
        Path::new("/inputs/rust").join(test_root).display().to_string(),
        "--sysroot".into(),
        "/toolchain".into(),
        "-C".into(),
        "linker=/usr/bin/cc".into(),
        "-C".into(),
        "linker-features=-lld".into(),
        "-C".into(),
        "link-arg=-B/usr/lib/gcc/x86_64-linux-gnu/9/".into(),
        "-C".into(),
        "link-arg=-B/usr/bin/".into(),
        "--emit=link,dep-info=/work/compiler.d".into(),
        "-o".into(),
        "/work/tests".into(),
    ]
}

fn compiler_dependencies(prepared: &PreparedInfrastructure, bytes: &[u8]) -> Result<()> {
    let mut observed = BTreeSet::new();
    for path in inputs::dep_paths(std::str::from_utf8(bytes)?)? {
        let relative = Path::new(&path)
            .strip_prefix("/inputs/rust")
            .context("infrastructure compiler used an undeclared dependency")?;
        // rustc preserves a frozen #[path = "../src/module.rs"] spelling in
        // dep-info. Resolve it lexically inside the sealed input root; never
        // follow host paths or permit a traversal out of that root.
        let mut normalized = PathBuf::new();
        for component in relative.components() {
            match component {
                Component::Normal(name) => normalized.push(name),
                Component::CurDir => {}
                Component::ParentDir => ensure!(normalized.pop(), "infrastructure compiler dependency escapes inputs"),
                _ => anyhow::bail!("infrastructure compiler dependency is not relative"),
            }
        }
        let name = normalized.to_str().context("non-UTF8 infrastructure dependency")?;
        ensure!(prepared.contract.source_files.contains_key(name), "infrastructure compiler dependency is not frozen");
        observed.insert(name.to_owned());
    }
    ensure!(
        observed.contains(&prepared.contract.test_root),
        "infrastructure compiler omitted the frozen test root dependency"
    );
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureLimits {
    pub timeout_seconds: u64,
    pub writable_bytes: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdin_bytes: usize,
}

impl InfrastructureLimits {
    fn run_limits(&self) -> Result<RunLimits> {
        ensure!(
            (1..=3600).contains(&self.timeout_seconds)
                && (4096..=MAX_GATE_BYTES).contains(&self.writable_bytes)
                && self.writable_bytes.is_multiple_of(4096)
                && self.stdout_bytes > 0
                && self.stdout_bytes <= 1024 * 1024
                && self.stderr_bytes > 0
                && self.stderr_bytes <= 65536
                && self.stdin_bytes == 0,
            "infrastructure producer requires hard time/output budgets and no stdin"
        );
        ensure!(
            self.stdout_bytes.checked_add(self.stderr_bytes).is_some_and(|bytes| bytes as u64 <= self.writable_bytes),
            "infrastructure log extents exceed writable quota"
        );
        Ok(RunLimits {
            timeout: Duration::from_secs(self.timeout_seconds),
            writable_bytes: self.writable_bytes,
            stdout_bytes: self.stdout_bytes,
            stderr_bytes: self.stderr_bytes,
            stdin_bytes: self.stdin_bytes,
        })
    }
}

/// Sealed operator review of the complete standalone source/test closure.
/// A rewrite response cannot supply provenance or change this coverage.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureProvenance {
    pub schema: String,
    pub scope_id: String,
    pub source_files: BTreeMap<String, String>,
    pub approved_files: Vec<String>,
    pub test_root: String,
    pub test_names: Vec<String>,
    pub coverage: BTreeMap<String, Vec<String>>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureContract {
    pub schema: String,
    pub scope_id: String,
    pub source_files: BTreeMap<String, String>,
    pub approved_files: Vec<String>,
    pub test_root: String,
    pub test_names: Vec<String>,
    pub coverage: BTreeMap<String, Vec<String>>,
    pub provenance: Asset,
    pub compiler_limits: InfrastructureLimits,
    pub tests_limits: InfrastructureLimits,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct PreparedInfrastructure {
    pub contract: InfrastructureContract,
    pub provenance: InfrastructureProvenance,
    pub dependencies_sha256: String,
    pub verification_sha256: String,
}

pub(crate) struct StrictRuntime {
    pub helper: PathBuf,
    pub rustc: PathBuf,
    pub operation_root: PathBuf,
    pub pipeline_root: PathBuf,
    pub workers: usize,
    pub gate_timeout_seconds: u64,
    pub max_output_bytes: u64,
    pub deadline_unix_ms: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureReceipt {
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub runtime_sha256: String,
    pub max_output_bytes: u64,
    pub reserved_bytes: u64,
    pub verdict: Asset,
    pub passed: bool,
    pub diagnostic: String,
    pub evidence_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureVerdict {
    pub schema: String,
    pub evidence_kind: String,
    pub acceptance_model: String,
    pub scope_id: String,
    pub passed: bool,
    pub test_names: Vec<String>,
    pub tested_count: usize,
    pub coverage: BTreeMap<String, Vec<String>>,
    pub limitations: Vec<String>,
    pub actual_c: String,
    pub original_rom_parity: String,
    pub live_replay: String,
    pub prepared_sha256: String,
    pub candidate_sha256: String,
    pub context: OperationContext,
    pub runtime_sha256: String,
    pub max_output_bytes: u64,
    pub reserved_bytes: u64,
    pub diagnostic: String,
    pub source_snapshots: BTreeMap<String, Reference>,
    pub compiler_outputs: BTreeMap<String, Reference>,
    pub operations: Vec<InfrastructureInvocation>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureInvocation {
    pub label: String,
    pub context: OperationContext,
    pub program: Program,
    pub limits: RunLimits,
    pub operation_root: PathBuf,
    pub result_sha256: String,
    pub references: Vec<Reference>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InfrastructureIntent {
    context: OperationContext,
    prepared_sha256: String,
    candidate_sha256: String,
    deadline_unix_ms: u64,
    max_output_bytes: u64,
    runtime_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InvocationIntent {
    context: OperationContext,
    request_sha256: String,
    limits: RunLimits,
    reserved_bytes: u64,
}

fn relative(path: &str) -> bool {
    let value = Path::new(path);
    !path.is_empty()
        && !path.contains(['\\', '\0'])
        && path.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
        && value.components().all(|part| matches!(part, Component::Normal(_)))
        && value.extension().is_some_and(|extension| extension == "rs")
        && !value.components().any(|part| {
            part.as_os_str()
                .to_str()
                .is_some_and(|name| matches!(name, ".git" | ".scratch" | "out" | "target" | ".codex"))
        })
}

fn safe(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "infrastructure paths must be absolute");
    let normalized = path.components().collect::<PathBuf>();
    ensure!(normalized.as_os_str() == path.as_os_str(), "infrastructure path alias");
    let mut current = PathBuf::new();
    for part in path.components() {
        ensure!(matches!(part, Component::RootDir | Component::Normal(_)), "infrastructure path alias");
        current.push(part);
        match current.symlink_metadata() {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "infrastructure path crosses a symlink"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn regular(path: &Path) -> Result<PathBuf> {
    safe(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_file() && metadata.nlink() == 1 && metadata.len() <= MAX_SOURCE_BYTES,
        "infrastructure source must be regular, unaliased, and bounded"
    );
    Ok(path.into())
}

fn text(path: &Path) -> Result<String> {
    inputs::bounded::text(&regular(path)?).with_context(|| format!("read infrastructure Rust input {}", path.display()))
}

fn approved_source(path: &str, source: &str) -> Result<()> {
    ensure!(
        relative(path)
            && !Path::new(path).components().any(|part| part
                .as_os_str()
                .to_str()
                .is_some_and(|name| matches!(name, "tests" | "fixtures" | "manifests"))),
        "infrastructure writable source overlaps frozen tests or manifests"
    );
    let compact = source.chars().filter(|value| !value.is_whitespace()).collect::<String>();
    ensure!(
        !compact.contains("#[test]")
            && !compact.contains("cfg(test)")
            && !compact.contains("cfg_attr(test,")
            && !compact.contains("externcrate")
            && !compact.contains("include!("),
        "infrastructure writable file contains verification declarations or unsupported package/macro dependencies"
    );
    Ok(())
}

fn checked_provenance(checkout: &Path, contract: &InfrastructureContract) -> Result<InfrastructureProvenance> {
    let path = if contract.provenance.file.is_absolute() {
        contract.provenance.file.clone()
    } else {
        ensure!(
            !contract.provenance.file.as_os_str().is_empty()
                && contract.provenance.file.components().all(|part| matches!(part, Component::Normal(_))),
            "infrastructure provenance path alias"
        );
        checkout.join(&contract.provenance.file)
    };
    let path = regular(&path)?;
    ensure!(
        digest_valid(&contract.provenance.sha256) && inputs::bounded::digest(&path)? == contract.provenance.sha256,
        "reviewed infrastructure provenance changed"
    );
    let provenance: InfrastructureProvenance = serde_json::from_str(&text(&path)?)?;
    ensure!(
        provenance.schema == "translation-infrastructure-provenance-v1"
            && provenance.scope_id == contract.scope_id
            && provenance.source_files == contract.source_files
            && provenance.approved_files == contract.approved_files
            && provenance.test_root == contract.test_root
            && provenance.test_names == contract.test_names
            && provenance.coverage == contract.coverage,
        "infrastructure contract differs from reviewed provenance"
    );
    ensure!(
        !provenance.limitations.is_empty()
            && provenance.limitations.len() <= 16
            && provenance
                .limitations
                .iter()
                .all(|limitation| !limitation.trim().is_empty() && limitation.len() <= 4096),
        "infrastructure provenance requires bounded acceptance limitations"
    );
    Ok(provenance)
}

pub(crate) fn prepare(checkout: &Path, contract: &InfrastructureContract) -> Result<PreparedInfrastructure> {
    inputs::bounded::limited(|| prepare_inner(checkout, contract))
}

pub(crate) fn prepare_until(
    checkout: &Path,
    contract: &InfrastructureContract,
    deadline_unix_ms: u64,
    max_input_bytes: u64,
) -> Result<PreparedInfrastructure> {
    inputs::bounded::until(deadline_unix_ms, max_input_bytes, || prepare_inner(checkout, contract))
}

fn prepare_inner(checkout: &Path, contract: &InfrastructureContract) -> Result<PreparedInfrastructure> {
    inputs::bounded::check()?;
    safe(checkout)?;
    ensure!(
        contract.schema == "translation-infrastructure-contract-v1" && !contract.scope_id.trim().is_empty(),
        "unsupported infrastructure contract"
    );
    ensure!(
        !contract.source_files.is_empty() && contract.source_files.len() <= MAX_FILES,
        "infrastructure source closure missing or excessive"
    );
    ensure!(
        relative(&contract.test_root)
            && contract.source_files.contains_key(&contract.test_root)
            && !contract.approved_files.is_empty()
            && contract.approved_files.len() <= MAX_FILES,
        "infrastructure requires a frozen test root and approved source"
    );
    let tests = contract.test_names.iter().cloned().collect::<BTreeSet<_>>();
    ensure!(
        !tests.is_empty()
            && tests.len() == contract.test_names.len()
            && tests.len() <= 1024
            && contract.test_names.windows(2).all(|pair| pair[0] < pair[1])
            && tests.iter().all(|name| !name.is_empty()
                && name.len() <= 512
                && name.chars().all(|value| value.is_ascii_alphanumeric() || matches!(value, '_' | ':'))),
        "infrastructure requires a nonempty exact sorted test inventory"
    );
    ensure!(
        !contract.coverage.is_empty()
            && contract.coverage.len() <= MAX_FILES
            && contract.coverage.iter().all(|(domain, cases)| !domain.trim().is_empty()
                && domain.len() <= 512
                && !cases.is_empty()
                && cases.len() <= tests.len()
                && cases.iter().all(|case| tests.contains(case))
                && cases.iter().collect::<BTreeSet<_>>().len() == cases.len())
            && contract.coverage.values().flatten().cloned().collect::<BTreeSet<_>>() == tests,
        "infrastructure prescribed coverage missing or inconsistent"
    );
    contract.compiler_limits.run_limits()?;
    contract.tests_limits.run_limits()?;
    let mut total = 0u64;
    for (name, expected) in &contract.source_files {
        inputs::bounded::check()?;
        ensure!(relative(name) && digest_valid(expected), "infrastructure source path/hash invalid");
        let path = regular(&checkout.join(name))?;
        total = total.checked_add(path.metadata()?.len()).context("infrastructure source extent overflow")?;
        ensure!(
            total <= MAX_SOURCE_BYTES && inputs::bounded::digest(&path)? == *expected,
            "infrastructure source dependency missing or changed: {name}"
        );
        let compact = text(&path)?.chars().filter(|value| !value.is_whitespace()).collect::<String>();
        ensure!(
            !compact.contains("externcrate") && !compact.contains("include!("),
            "standalone infrastructure does not support external package or source macro dependencies"
        );
    }
    let mut approved = BTreeSet::new();
    for name in &contract.approved_files {
        ensure!(
            contract.source_files.contains_key(name) && name != &contract.test_root && approved.insert(name),
            "infrastructure approved ownership overlaps test root or is duplicated"
        );
        approved_source(name, &text(&checkout.join(name))?)?;
    }
    ensure!(
        text(&checkout.join(&contract.test_root))?
            .chars()
            .filter(|value| !value.is_whitespace())
            .collect::<String>()
            .contains("#[test]"),
        "infrastructure frozen test root declares no tests"
    );
    let provenance = checked_provenance(checkout, contract)?;
    let dependencies_sha256 = sealed(&("standalone-rust-infrastructure-dependencies-v1", &contract.source_files))?;
    let verification_sha256 = sealed(&(contract, &provenance))?;
    Ok(PreparedInfrastructure { contract: contract.clone(), provenance, dependencies_sha256, verification_sha256 })
}

fn check_candidate(prepared: &PreparedInfrastructure, checkout: &Path) -> Result<BTreeMap<String, String>> {
    inputs::bounded::check()?;
    safe(checkout)?;
    ensure!(
        prepared.dependencies_sha256
            == sealed(&("standalone-rust-infrastructure-dependencies-v1", &prepared.contract.source_files))?
            && prepared.verification_sha256 == sealed(&(&prepared.contract, &prepared.provenance))?,
        "infrastructure prepared identities changed"
    );
    let mut identities = BTreeMap::new();
    let mut total_bytes = 0u64;
    for (name, expected) in &prepared.contract.source_files {
        inputs::bounded::check()?;
        let path = regular(&checkout.join(name))?;
        total_bytes = total_bytes
            .checked_add(path.metadata()?.len())
            .context("infrastructure candidate source extent overflow")?;
        ensure!(total_bytes <= MAX_SOURCE_BYTES, "infrastructure candidate aggregate source extent exceeded");
        let hash = inputs::bounded::digest(&path)?;
        if prepared.contract.approved_files.contains(name) {
            approved_source(name, &text(&path)?)?;
        } else {
            ensure!(hash == *expected, "frozen infrastructure harness or dependency changed: {name}");
        }
        identities.insert(name.clone(), hash);
    }
    Ok(identities)
}

fn now_ms() -> Result<u64> {
    Ok(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?)
}

fn runtime_identity(runtime: &StrictRuntime) -> Result<String> {
    sealed(&(
        &runtime.helper,
        &runtime.rustc,
        &runtime.operation_root,
        &runtime.pipeline_root,
        runtime.workers,
        runtime.gate_timeout_seconds,
        runtime.max_output_bytes,
        runtime.deadline_unix_ms,
    ))
}

fn publish_bytes(path: &Path, bytes: &[u8], executable: bool) -> Result<()> {
    safe(path)?;
    if path.try_exists()? {
        let metadata = path.symlink_metadata()?;
        ensure!(
            metadata.is_file()
                && metadata.nlink() == 1
                && metadata.len() == bytes.len() as u64
                && inputs::bounded::read_limit(path, bytes.len() as u64)? == bytes,
            "infrastructure retained output changed"
        );
        return Ok(());
    }
    fs::create_dir_all(path.parent().context("infrastructure output parent missing")?)?;
    let pending = path.with_extension("pending");
    safe(&pending)?;
    if pending.try_exists()? {
        ensure!(pending.symlink_metadata()?.is_file(), "infrastructure orphaned output is not regular");
        fs::remove_file(&pending)?;
    }
    let mut file =
        OpenOptions::new().write(true).create_new(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    file.set_permissions(fs::Permissions::from_mode(if executable { 0o500 } else { 0o400 }))?;
    fs::rename(pending, path)?;
    File::open(path.parent().context("infrastructure output parent missing")?)?.sync_all()?;
    Ok(())
}

struct Gate<'a> {
    runtime: &'a StrictRuntime,
    output: &'a Path,
    context: &'a OperationContext,
    deadline_unix_ms: u64,
    budget: ByteBudget,
    operations: Vec<InfrastructureInvocation>,
}

impl Gate<'_> {
    fn invoke(&mut self, label: &str, program: Program, limits: RunLimits) -> Result<operation::OperationResult> {
        let mut context = self.context.clone();
        context.operation_key = sealed(&(&self.context.operation_key, self.output, label))?;
        let request_sha256 = sealed(&(&program, &limits))?;
        let path = self.output.join(format!("{label}-invocation.toml"));
        let existed = path.is_file();
        let _held = if existed {
            None
        } else {
            ensure!(now_ms()? < self.deadline_unix_ms, "infrastructure aggregate gate time budget exhausted");
            Some(acquire_lease_until(&self.runtime.pipeline_root, self.runtime.workers, self.deadline_unix_ms)?.0)
        };
        let declared_timeout = limits.timeout;
        let invocation = if existed {
            let saved: InvocationIntent = family::object(&path)?;
            ensure!(
                saved.context == context
                    && saved.request_sha256 == request_sha256
                    && !saved.limits.timeout.is_zero()
                    && saved.limits.timeout <= declared_timeout,
                "infrastructure invocation retried with changed inputs"
            );
            saved
        } else {
            let reserved_bytes = operation::reserve_bytes_until(&program, &limits, 0, self.deadline_unix_ms)?
                .checked_add(MAX_RECORD_BYTES)
                .context("infrastructure producer reservation overflow")?;
            self.budget.reserve(reserved_bytes)?;
            let remaining = self.deadline_unix_ms.saturating_sub(now_ms()?);
            ensure!(remaining > 0, "infrastructure aggregate gate time budget exhausted");
            let mut actual = limits;
            actual.timeout = actual.timeout.min(Duration::from_millis(remaining));
            // Reserve before publishing invocation intent or launching its writer.
            let saved = InvocationIntent { context: context.clone(), request_sha256, limits: actual, reserved_bytes };
            atomic_toml(&path, &saved)?;
            saved
        };
        if existed {
            self.budget.reserve(invocation.reserved_bytes)?;
            // Reconstructed controllers charge each original producer once.
            // The original invocation's sealed operation owns its imported
            // runtime identity; recovery never rediscovers it to dispatch.
        }
        if !existed {
            ensure!(
                now_ms()? < self.deadline_unix_ms,
                "infrastructure aggregate deadline exhausted before producer launch"
            );
        }
        let directory = operation::directory(&self.runtime.operation_root, &context)?;
        if existed {
            ensure!(
                directory.join("intent.json").is_file(),
                "infrastructure invocation interrupted before durable launch; explicit operator retry required"
            );
        }
        let result = if now_ms()? >= self.deadline_unix_ms {
            ensure!(directory.join("intent.json").is_file(), "infrastructure gate time budget exhausted before launch");
            if !directory.join("result.json").is_file() {
                operation::stop(&self.runtime.operation_root, &context)?;
            }
            operation::authenticate_request(&self.runtime.operation_root, &context, &program, &invocation.limits, &[])?
        } else {
            operation::run_until(
                &self.runtime.operation_root,
                &context,
                &program,
                &invocation.limits,
                &[],
                self.deadline_unix_ms,
            )?
        };
        let references = operation::references(&self.runtime.operation_root, &context)?;
        self.operations.push(InfrastructureInvocation {
            label: label.into(),
            context,
            program,
            limits: invocation.limits,
            operation_root: self.runtime.operation_root.clone(),
            result_sha256: result.result_sha256.clone(),
            references,
        });
        ensure!(
            result.passed && result.status_raw == Some(0),
            "confined infrastructure {label} failed: {}",
            result.diagnostic
        );
        Ok(result)
    }
}

fn listed_tests(bytes: &[u8], prescribed: &[String]) -> Result<()> {
    let text = std::str::from_utf8(bytes)?;
    ensure!(!text.is_empty(), "infrastructure test list is empty");
    let mut listed = Vec::new();
    let mut summary = false;
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if let Some(name) = line.strip_suffix(": test") {
            listed.push(name.to_owned());
        } else {
            ensure!(
                line == format!(
                    "{} test{}",
                    prescribed.len(),
                    if prescribed.len() == 1 { ", 0 benchmarks" } else { "s, 0 benchmarks" }
                ),
                "unexpected infrastructure test list record: {line}"
            );
            summary = true;
        }
    }
    listed.sort();
    ensure!(summary && listed == prescribed, "compiled infrastructure coverage differs from frozen test inventory");
    Ok(())
}

fn passed_tests(bytes: &[u8], prescribed: &[String]) -> Result<()> {
    let text = std::str::from_utf8(bytes)?;
    ensure!(
        !text.is_empty()
            && text.lines().filter(|line| line.starts_with("running ")).collect::<Vec<_>>()
                == vec![format!("running {} test{}", prescribed.len(), if prescribed.len() == 1 { "" } else { "s" })],
        "infrastructure test invocation ran no prescribed tests"
    );
    let mut passed = text
        .lines()
        .filter_map(|line| line.strip_prefix("test ").and_then(|line| line.strip_suffix(" ... ok")))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    passed.sort();
    ensure!(passed == prescribed, "infrastructure test output omits or adds prescribed cases");
    let summary =
        format!("test result: ok. {} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;", prescribed.len());
    ensure!(
        text.lines().filter(|line| line.starts_with("test result:")).count() == 1 && text.contains(&summary),
        "infrastructure nonzero full-test summary missing"
    );
    Ok(())
}

pub(crate) fn check_ready(
    prepared: &PreparedInfrastructure,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<InfrastructureReceipt> {
    let candidate = check_candidate(prepared, checkout)?;
    ensure!(candidate == prepared.contract.source_files, "infrastructure baseline readiness requires reviewed source");
    run(prepared, checkout, output, runtime, context)
}

pub(crate) fn run(
    prepared: &PreparedInfrastructure,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
) -> Result<InfrastructureReceipt> {
    let entered_unix_ms = now_ms()?;
    run_inner(prepared, checkout, output, runtime, context, entered_unix_ms)
}

fn run_inner(
    prepared: &PreparedInfrastructure,
    checkout: &Path,
    output: &Path,
    runtime: &StrictRuntime,
    context: &OperationContext,
    entered_unix_ms: u64,
) -> Result<InfrastructureReceipt> {
    let receipt_path = output.join("infrastructure-receipt.toml");
    let verdict_path = output.join("infrastructure-verdict.toml");
    if receipt_path.is_file() || verdict_path.is_file() {
        // Finalized producers and interrupted receipt publication get finite
        // authentication/publication IO and cannot launch another candidate.
        return inputs::bounded::reconcile(inputs::bounded::DEFAULT_BYTES, || {
            if receipt_path.is_file() {
                let receipt: InfrastructureReceipt = family::object(&receipt_path)?;
                ensure!(
                    receipt.context == *context && receipt.runtime_sha256 == runtime_identity(runtime)?,
                    "infrastructure cached receipt belongs to another operation or installed runtime"
                );
                authenticate(prepared, checkout, &receipt)?;
                return Ok(receipt);
            }
            let verdict: InfrastructureVerdict = family::object(&verdict_path)?;
            ensure!(
                verdict.context == *context && verdict.runtime_sha256 == runtime_identity(runtime)?,
                "infrastructure sealed verdict belongs to another operation or installed runtime"
            );
            finish_receipt(prepared, checkout, &verdict_path, &verdict)
        });
    }
    let intent_path = output.join("infrastructure-intent.toml");
    let saved: Option<InfrastructureIntent> = if intent_path.is_file() {
        Some(inputs::bounded::reconcile(MAX_RECORD_BYTES, || family::object(&intent_path))?)
    } else {
        None
    };
    let local_deadline = entered_unix_ms
        .checked_add(runtime.gate_timeout_seconds.checked_mul(1000).context("infrastructure deadline overflow")?)
        .context("infrastructure deadline overflow")?;
    let deadline = saved
        .as_ref()
        .map_or(runtime.deadline_unix_ms.map_or(local_deadline, |outer| outer.min(local_deadline)), |intent| {
            intent.deadline_unix_ms
        });
    let resumed = saved.is_some();
    if !resumed {
        ensure!(entered_unix_ms < deadline, "infrastructure whole-stage time budget exhausted before preparation");
    }
    let setup = || -> Result<_> {
        inputs::bounded::check()?;
        let candidates = check_candidate(prepared, checkout)?;
        safe(output)?;
        safe(&runtime.operation_root)?;
        safe(&runtime.pipeline_root)?;
        ensure!(
            (1..=3600).contains(&runtime.gate_timeout_seconds)
                && (1..=64).contains(&runtime.workers)
                && (MAX_RECORD_BYTES..=MAX_GATE_BYTES).contains(&runtime.max_output_bytes),
            "installed infrastructure gate time/byte budget missing"
        );
        fs::create_dir_all(output)?;
        let prepared_sha256 = sealed(prepared)?;
        let candidate_sha256 = sealed(&candidates)?;
        let runtime_sha256 = runtime_identity(runtime)?;
        let intent = if let Some(saved) = saved {
            ensure!(
                saved.context == *context
                    && saved.prepared_sha256 == prepared_sha256
                    && saved.candidate_sha256 == candidate_sha256
                    && saved.max_output_bytes == runtime.max_output_bytes
                    && saved.runtime_sha256 == runtime_sha256,
                "infrastructure gate retried with changed source or identity"
            );
            saved
        } else {
            InfrastructureIntent {
                context: context.clone(),
                prepared_sha256: prepared_sha256.clone(),
                candidate_sha256: candidate_sha256.clone(),
                deadline_unix_ms: deadline,
                max_output_bytes: runtime.max_output_bytes,
                runtime_sha256: runtime_sha256.clone(),
            }
        };
        let mut gate = Gate {
            runtime,
            output,
            context,
            deadline_unix_ms: intent.deadline_unix_ms,
            budget: ByteBudget::new(runtime.max_output_bytes),
            operations: vec![],
        };
        gate.budget.reserve(MAX_RECORD_BYTES.checked_mul(4).context("infrastructure record reservation overflow")?)?;
        let source_bytes = prepared.contract.source_files.keys().try_fold(0u64, |total, name| {
            total
                .checked_add(regular(&checkout.join(name))?.metadata()?.len())
                .context("infrastructure snapshot extent overflow")
        })?;
        gate.budget.reserve(source_bytes.checked_mul(2).context("infrastructure snapshot reservation overflow")?)?;
        atomic_toml(&intent_path, &intent)?;
        let snapshots = output.join("source");
        let mut source_snapshots = BTreeMap::new();
        for name in prepared.contract.source_files.keys() {
            inputs::bounded::check()?;
            let source = regular(&checkout.join(name))?;
            let destination = snapshots.join(name);
            let bytes = inputs::bounded::read_limit(&source, MAX_SOURCE_BYTES)?;
            publish_bytes(&destination, &bytes, false)?;
            source_snapshots.insert(name.clone(), retention::seal_file(&destination, MAX_SOURCE_BYTES)?);
        }
        let verdict = InfrastructureVerdict {
            schema: "translation-infrastructure-verdict-v1".into(),
            evidence_kind: "infrastructure".into(),
            acceptance_model: ACCEPTANCE_MODEL.into(),
            scope_id: prepared.contract.scope_id.clone(),
            passed: false,
            test_names: prepared.contract.test_names.clone(),
            tested_count: 0,
            coverage: prepared.contract.coverage.clone(),
            limitations: prepared.provenance.limitations.clone(),
            actual_c: "not-tested".into(),
            original_rom_parity: "not-tested".into(),
            live_replay: "not-tested".into(),
            prepared_sha256: prepared_sha256.clone(),
            candidate_sha256: candidate_sha256.clone(),
            context: context.clone(),
            runtime_sha256: runtime_sha256.clone(),
            max_output_bytes: runtime.max_output_bytes,
            reserved_bytes: 0,
            diagnostic: String::new(),
            source_snapshots,
            compiler_outputs: BTreeMap::new(),
            operations: vec![],
        };
        Ok((candidates, snapshots, gate, verdict, intent))
    };
    let (candidates, snapshots, mut gate, mut verdict, intent) = if resumed && entered_unix_ms >= deadline {
        inputs::bounded::reconcile(inputs::bounded::DEFAULT_BYTES, setup)?
    } else {
        inputs::bounded::until(deadline, inputs::bounded::DEFAULT_BYTES, setup)?
    };
    let produce = || -> Result<()> {
        let compiler = fs::canonicalize(&runtime.rustc)?;
        let helper = fs::canonicalize(&runtime.helper)?;
        let inputs = prepared
            .contract
            .source_files
            .keys()
            .map(|name| {
                Ok(ReadOnlyBind {
                    source: regular(&snapshots.join(name))?,
                    destination: Path::new("/inputs/rust").join(name),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let program = Program {
            executable: compiler.clone(),
            helper: helper.clone(),
            runtime: if now_ms()? >= deadline {
                builds::strict_compiler_runtime(&compiler)?
            } else {
                builds::strict_compiler_runtime_until(&compiler, deadline, inputs::bounded::DEFAULT_BYTES)?
            },
            inputs,
            arguments: compiler_arguments(&prepared.contract.test_root),
            outputs: vec!["tests".into(), "compiler.d".into()],
        };
        // The trusted projection duplicates one bounded compiler output extent.
        gate.budget.reserve(prepared.contract.compiler_limits.writable_bytes)?;
        let compiled = gate.invoke("compile", program, prepared.contract.compiler_limits.run_limits()?)?;
        let executable =
            compiled.outputs.get(Path::new("tests")).context("infrastructure compiler omitted test binary")?;
        let dependencies = compiled
            .outputs
            .get(Path::new("compiler.d"))
            .context("infrastructure compiler omitted dependency closure")?;
        ensure!(!executable.is_empty(), "infrastructure compiler returned an empty executable");
        compiler_dependencies(prepared, dependencies)?;
        let binary = output.join("tests");
        publish_bytes(&binary, executable, true)?;
        let dependency_record = output.join("compiler.d");
        publish_bytes(&dependency_record, dependencies, false)?;
        verdict.compiler_outputs.insert("tests".into(), retention::seal_file(&binary, MAX_GATE_BYTES)?);
        verdict.compiler_outputs.insert("compiler.d".into(), retention::seal_file(&dependency_record, MAX_GATE_BYTES)?);
        let base = Program {
            executable: binary,
            helper,
            runtime: confinement::helper_runtime()?,
            inputs: vec![],
            arguments: vec![],
            outputs: vec![],
        };
        let mut listing = base.clone();
        listing.arguments = vec!["--list".into(), "--format=pretty".into()];
        let listed = gate.invoke("list", listing, prepared.contract.tests_limits.run_limits()?)?;
        listed_tests(&listed.stdout, &prepared.contract.test_names)?;
        let mut tests = base;
        tests.arguments = vec!["--test-threads=1".into(), "--format=pretty".into(), "--color=never".into()];
        let output = gate.invoke("tests", tests, prepared.contract.tests_limits.run_limits()?)?;
        passed_tests(&output.stdout, &prepared.contract.test_names)?;
        ensure!(now_ms()? <= intent.deadline_unix_ms, "infrastructure aggregate gate deadline exhausted");
        ensure!(
            check_candidate(prepared, checkout)? == candidates,
            "infrastructure candidate changed during verification"
        );
        verdict.passed = true;
        verdict.tested_count = prepared.contract.test_names.len();
        Ok(())
    };
    let result = if resumed && now_ms()? >= deadline {
        inputs::bounded::reconcile(inputs::bounded::DEFAULT_BYTES, produce)
    } else {
        inputs::bounded::until(deadline, inputs::bounded::DEFAULT_BYTES, produce)
    };
    if let Err(error) = result {
        if error.downcast_ref::<translation_execution::ExecutionPending>().is_some() {
            return Err(error);
        }
        verdict.passed = false;
        verdict.tested_count = 0;
        let diagnostic = format!("{error:#}");
        let mut end = diagnostic.len().min(4096);
        while !diagnostic.is_char_boundary(end) {
            end -= 1;
        }
        verdict.diagnostic = diagnostic[..end].into();
    }
    verdict.operations = gate.operations;
    verdict.reserved_bytes = gate.budget.used();
    inputs::bounded::reconcile(inputs::bounded::DEFAULT_BYTES, || {
        if verdict.passed && now_ms()? >= intent.deadline_unix_ms {
            verdict.passed = false;
            verdict.tested_count = 0;
            verdict.diagnostic = "infrastructure whole-stage time budget exhausted before acceptance".into();
        }
        let path = output.join("infrastructure-verdict.toml");
        let bytes = toml::to_string_pretty(&verdict)?.into_bytes();
        ensure!(bytes.len() as u64 <= MAX_RECORD_BYTES, "infrastructure verdict metadata exceeds reserved extent");
        publish_bytes(&path, &bytes, false)?;
        finish_receipt(prepared, checkout, &path, &verdict)
    })
}

fn finish_receipt(
    prepared: &PreparedInfrastructure,
    checkout: &Path,
    path: &Path,
    verdict: &InfrastructureVerdict,
) -> Result<InfrastructureReceipt> {
    let receipt = InfrastructureReceipt {
        prepared_sha256: verdict.prepared_sha256.clone(),
        candidate_sha256: verdict.candidate_sha256.clone(),
        context: verdict.context.clone(),
        runtime_sha256: verdict.runtime_sha256.clone(),
        max_output_bytes: verdict.max_output_bytes,
        verdict: Asset { file: path.to_path_buf(), sha256: digest(path)? },
        passed: verdict.passed,
        diagnostic: verdict.diagnostic.clone(),
        evidence_bytes: exact_evidence_bytes(verdict, path)?,
        reserved_bytes: verdict.reserved_bytes,
    };
    ensure!(
        receipt.evidence_bytes <= verdict.max_output_bytes,
        "infrastructure accepted evidence exceeds pre-reserved gate bound"
    );
    authenticate(prepared, checkout, &receipt)?;
    atomic_toml(
        &path.parent().context("infrastructure verdict parent missing")?.join("infrastructure-receipt.toml"),
        &receipt,
    )?;
    Ok(receipt)
}

fn exact_evidence_bytes(verdict: &InfrastructureVerdict, verdict_path: &Path) -> Result<u64> {
    let mut references = verdict.source_snapshots.values().cloned().collect::<Vec<_>>();
    references.extend(verdict.compiler_outputs.values().cloned());
    references.extend(verdict.operations.iter().flat_map(|operation| operation.references.clone()));
    references.push(retention::seal_file(verdict_path, MAX_RECORD_BYTES)?);
    let mut paths = BTreeMap::new();
    let mut bytes = 0u64;
    for reference in references {
        if let Some(prior) = paths.insert(reference.path.clone(), reference.clone()) {
            ensure!(prior == reference, "infrastructure conflicting evidence identity");
        } else {
            bytes = bytes.checked_add(reference.bytes).context("infrastructure evidence extent overflow")?;
        }
    }
    Ok(bytes)
}

pub(crate) fn authenticate(
    prepared: &PreparedInfrastructure,
    checkout: &Path,
    receipt: &InfrastructureReceipt,
) -> Result<InfrastructureVerdict> {
    inputs::bounded::limited(|| authenticate_inner(prepared, checkout, receipt))
}

fn authenticate_inner(
    prepared: &PreparedInfrastructure,
    checkout: &Path,
    receipt: &InfrastructureReceipt,
) -> Result<InfrastructureVerdict> {
    let candidate = check_candidate(prepared, checkout)?;
    ensure!(
        receipt.prepared_sha256 == sealed(prepared)? && receipt.candidate_sha256 == sealed(&candidate)?,
        "infrastructure receipt source/preparation identity differs"
    );
    let path = regular(&receipt.verdict.file)?;
    ensure!(inputs::bounded::digest(&path)? == receipt.verdict.sha256, "retained infrastructure verdict changed");
    let verdict: InfrastructureVerdict = family::object(&path)?;
    ensure!(
        verdict.schema == "translation-infrastructure-verdict-v1"
            && verdict.evidence_kind == "infrastructure"
            && verdict.acceptance_model == ACCEPTANCE_MODEL
            && verdict.scope_id == prepared.contract.scope_id
            && verdict.passed == receipt.passed
            && verdict.diagnostic == receipt.diagnostic
            && verdict.prepared_sha256 == receipt.prepared_sha256
            && verdict.candidate_sha256 == receipt.candidate_sha256
            && verdict.context == receipt.context
            && verdict.runtime_sha256 == receipt.runtime_sha256
            && verdict.max_output_bytes == receipt.max_output_bytes
            && verdict.reserved_bytes == receipt.reserved_bytes
            && verdict.reserved_bytes <= verdict.max_output_bytes
            && receipt.evidence_bytes <= receipt.reserved_bytes
            && verdict.test_names == prepared.contract.test_names
            && verdict.coverage == prepared.contract.coverage
            && verdict.limitations == prepared.provenance.limitations
            && verdict.actual_c == "not-tested"
            && verdict.original_rom_parity == "not-tested"
            && verdict.live_replay == "not-tested",
        "infrastructure receipt acceptance or limitations changed"
    );
    ensure!(verdict.source_snapshots.len() == candidate.len(), "infrastructure source snapshot closure differs");
    for (name, hash) in &candidate {
        let snapshot = verdict.source_snapshots.get(name).context("infrastructure source snapshot missing")?;
        retention::authenticate_reference(snapshot, MAX_SOURCE_BYTES)?;
        ensure!(&snapshot.sha256 == hash, "infrastructure retained source differs from candidate");
    }
    for (name, reference) in &verdict.compiler_outputs {
        ensure!(
            matches!(name.as_str(), "tests" | "compiler.d")
                && reference.path == path.parent().context("infrastructure verdict parent missing")?.join(name),
            "infrastructure compiler output projection changed"
        );
        retention::authenticate_reference(reference, prepared.contract.compiler_limits.writable_bytes)?;
    }
    let mut observed = BTreeMap::new();
    for invocation in &verdict.operations {
        let mut expected_context = verdict.context.clone();
        expected_context.operation_key = sealed(&(
            &verdict.context.operation_key,
            path.parent().context("infrastructure verdict parent missing")?,
            &invocation.label,
        ))?;
        ensure!(invocation.context == expected_context, "infrastructure producer belongs to a foreign gate identity");
        ensure!(!invocation.limits.timeout.is_zero(), "infrastructure producer timeout missing");
        let snapshot_root = path.parent().context("infrastructure verdict parent missing")?.join("source");
        if invocation.label == "compile" {
            let expected_inputs = prepared
                .contract
                .source_files
                .keys()
                .map(|name| ReadOnlyBind {
                    source: snapshot_root.join(name),
                    destination: Path::new("/inputs/rust").join(name),
                })
                .collect::<Vec<_>>();
            let declared = prepared.contract.compiler_limits.run_limits()?;
            ensure!(
                invocation.program.inputs == expected_inputs
                    && invocation.program.outputs == vec![PathBuf::from("tests"), PathBuf::from("compiler.d")]
                    && invocation.program.arguments == compiler_arguments(&prepared.contract.test_root)
                    && invocation.limits.timeout <= declared.timeout
                    && invocation.limits.writable_bytes == declared.writable_bytes
                    && invocation.limits.stdout_bytes == declared.stdout_bytes
                    && invocation.limits.stderr_bytes == declared.stderr_bytes
                    && invocation.limits.stdin_bytes == 0,
                "infrastructure compile recipe changed"
            );
        } else {
            let expected_args = match invocation.label.as_str() {
                "list" => vec!["--list".to_owned(), "--format=pretty".to_owned()],
                "tests" => {
                    vec!["--test-threads=1".to_owned(), "--format=pretty".to_owned(), "--color=never".to_owned()]
                }
                _ => anyhow::bail!("unknown infrastructure producer label"),
            };
            let declared = prepared.contract.tests_limits.run_limits()?;
            ensure!(
                invocation.program.arguments == expected_args
                    && invocation.program.inputs.is_empty()
                    && invocation.program.outputs.is_empty()
                    && invocation.program.executable
                        == path.parent().context("infrastructure verdict parent missing")?.join("tests")
                    && invocation.limits.timeout <= declared.timeout
                    && invocation.limits.writable_bytes == declared.writable_bytes
                    && invocation.limits.stdout_bytes == declared.stdout_bytes
                    && invocation.limits.stderr_bytes == declared.stderr_bytes
                    && invocation.limits.stdin_bytes == 0,
                "infrastructure runtime recipe changed"
            );
        }
        ensure!(
            observed.insert(invocation.label.as_str(), invocation).is_none(),
            "duplicate infrastructure producer receipt"
        );
        let actual = operation::authenticate_request(
            &invocation.operation_root,
            &invocation.context,
            &invocation.program,
            &invocation.limits,
            &[],
        )?;
        ensure!(
            actual.result_sha256 == invocation.result_sha256
                && operation::references(&invocation.operation_root, &invocation.context)? == invocation.references,
            "infrastructure producer retained evidence differs"
        );
        if receipt.passed {
            ensure!(actual.passed && actual.status_raw == Some(0), "infrastructure accepted a failed producer");
        }
        if receipt.passed && invocation.label == "compile" {
            compiler_dependencies(
                prepared,
                actual
                    .outputs
                    .get(Path::new("compiler.d"))
                    .context("infrastructure retained dependency closure missing")?,
            )?;
        }
        if invocation.label == "compile" && !verdict.compiler_outputs.is_empty() {
            ensure!(actual.passed, "infrastructure compiler projections came from a failed operation");
            for (name, reference) in &verdict.compiler_outputs {
                let bytes = actual.outputs.get(Path::new(name)).context("infrastructure compiler output absent")?;
                ensure!(
                    reference.bytes == bytes.len() as u64
                        && reference.sha256 == translation_execution::model::sha256(bytes),
                    "infrastructure compiler output projection differs from sealed operation"
                );
            }
        }
        if receipt.passed && invocation.label == "list" {
            listed_tests(&actual.stdout, &prepared.contract.test_names)?;
        }
        if receipt.passed && invocation.label == "tests" {
            passed_tests(&actual.stdout, &prepared.contract.test_names)?;
        }
    }
    if receipt.passed {
        ensure!(
            observed.keys().copied().collect::<BTreeSet<_>>() == BTreeSet::from(["compile", "list", "tests"])
                && verdict.tested_count == prepared.contract.test_names.len()
                && verdict.compiler_outputs.keys().map(String::as_str).collect::<BTreeSet<_>>()
                    == BTreeSet::from(["tests", "compiler.d"]),
            "infrastructure complete nonzero gates missing"
        );
    } else {
        ensure!(
            verdict.tested_count == 0 && !verdict.diagnostic.is_empty(),
            "infrastructure failure lacks bounded diagnostic"
        );
    }
    ensure!(
        receipt.evidence_bytes == exact_evidence_bytes(&verdict, &path)?,
        "infrastructure retained byte count differs"
    );
    Ok(verdict)
}

pub(crate) fn references(receipt: &InfrastructureReceipt) -> Result<Vec<Reference>> {
    let path = regular(&receipt.verdict.file)?;
    ensure!(digest(&path)? == receipt.verdict.sha256, "infrastructure evidence pointer changed");
    let verdict: InfrastructureVerdict = family::object(&path)?;
    let mut references = verdict.source_snapshots.into_values().collect::<Vec<_>>();
    references.extend(verdict.compiler_outputs.into_values());
    references.extend(verdict.operations.into_iter().flat_map(|operation| operation.references));
    references.push(retention::seal_file(&path, MAX_RECORD_BYTES)?);
    Ok(references)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Command,
        sync::atomic::{AtomicU64, Ordering},
    };
    use translation_execution::model::{ExecutionPhase, sha256};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Fixture {
        root: PathBuf,
        contract: InfrastructureContract,
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "sm-infrastructure-gate-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("src")).unwrap();
            fs::create_dir_all(root.join("tests")).unwrap();
            fs::write(root.join("src/domain.rs"), "pub fn doubled(value: u32) -> u32 { value * 2 }\n").unwrap();
            fs::write(
                root.join("tests/frozen.rs"),
                r#"
#[path = "../src/domain.rs"] mod domain;
#[test] fn handles_zero() { assert_eq!(domain::doubled(0), 0); }
#[test] fn doubles_positive() { assert_eq!(domain::doubled(7), 14); }
"#,
            )
            .unwrap();
            let source_files = ["src/domain.rs", "tests/frozen.rs"]
                .into_iter()
                .map(|name| (name.to_owned(), digest(&root.join(name)).unwrap()))
                .collect();
            let tests = vec!["doubles_positive".into(), "handles_zero".into()];
            let producer = InfrastructureLimits {
                timeout_seconds: 120,
                writable_bytes: 32 * 1024 * 1024,
                stdout_bytes: 65536,
                stderr_bytes: 65536,
                stdin_bytes: 0,
            };
            let mut contract = InfrastructureContract {
                schema: "translation-infrastructure-contract-v1".into(),
                scope_id: "standalone-arithmetic-infrastructure".into(),
                source_files,
                approved_files: vec!["src/domain.rs".into()],
                test_root: "tests/frozen.rs".into(),
                test_names: tests.clone(),
                coverage: BTreeMap::from([("arithmetic domain".into(), tests)]),
                provenance: Asset::default(),
                compiler_limits: producer,
                tests_limits: InfrastructureLimits {
                    timeout_seconds: 5,
                    writable_bytes: 1024 * 1024,
                    stdout_bytes: 65536,
                    stderr_bytes: 65536,
                    stdin_bytes: 0,
                },
            };
            let provenance = InfrastructureProvenance {
                schema: "translation-infrastructure-provenance-v1".into(),
                scope_id: contract.scope_id.clone(),
                source_files: contract.source_files.clone(),
                approved_files: contract.approved_files.clone(),
                test_root: contract.test_root.clone(),
                test_names: contract.test_names.clone(),
                coverage: contract.coverage.clone(),
                limitations: vec![
                    "Standalone Rust infrastructure tests only; no actual-C, ROM parity or live exercise acceptance."
                        .into(),
                ],
            };
            let path = root.join("reviewed-provenance.json");
            fs::write(&path, serde_json::to_vec(&provenance).unwrap()).unwrap();
            contract.provenance = Asset { file: path.clone(), sha256: digest(&path).unwrap() };
            Self { root, contract }
        }

        fn context(&self, label: &str) -> OperationContext {
            OperationContext {
                run_id: sha256("standalone-infrastructure-tests"),
                attempt: 1,
                policy_sha256: sha256(&self.contract.scope_id),
                operation_key: sha256(label),
                phase: ExecutionPhase::Candidate,
            }
        }

        fn runtime(&self) -> StrictRuntime {
            let helper = std::env::var_os("SM_TRANSLATION_TEST_HELPER")
                .map(PathBuf::from)
                .or_else(|| {
                    std::env::var_os("CARGO_TARGET_DIR")
                        .map(|path| PathBuf::from(path).join("debug/sm-translation-sandbox"))
                })
                .unwrap_or_else(|| PathBuf::from("/tmp/sm-translation-execution-target/debug/sm-translation-sandbox"));
            assert!(
                helper.is_file(),
                "build the actual sandbox helper and set SM_TRANSLATION_TEST_HELPER before this gate"
            );
            let installed = self.root.join("installed-sandbox-helper");
            fs::copy(helper, &installed).unwrap();
            let rustc = Command::new("rustup").args(["which", "rustc"]).output().unwrap();
            assert!(rustc.status.success());
            let rustc = PathBuf::from(String::from_utf8(rustc.stdout).unwrap().trim());
            StrictRuntime {
                helper: installed,
                rustc,
                operation_root: self.root.join("operations"),
                pipeline_root: std::env::var_os("SM_PIPELINE_ROOT")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| self.root.join("pipeline")),
                workers: 4,
                gate_timeout_seconds: 360,
                max_output_bytes: 256 * 1024 * 1024,
                deadline_unix_ms: None,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn records(path: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        let mut out = Vec::new();
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                out.extend(records(&path));
            } else {
                out.push((path.clone(), fs::read(path).unwrap()));
            }
        }
        out.sort_by(|left, right| left.0.cmp(&right.0));
        out
    }

    #[test]
    fn compiler_dependency_spellings_stay_within_the_frozen_source_closure() {
        let fixture = Fixture::new();
        let prepared = prepare(&fixture.root, &fixture.contract).unwrap();
        assert!(
            compiler_dependencies(
                &prepared,
                b"out: /inputs/rust/tests/frozen.rs /inputs/rust/tests/../src/domain.rs\n"
            )
            .is_ok()
        );
        for dependency in [
            "/inputs/rust/../rust/src/domain.rs",
            "/inputs/rust/tests/../../private.rs",
            "/inputs/rust/src/undeclared.rs",
            "/home/controller/private.rs",
        ] {
            let bytes = format!("out: /inputs/rust/tests/frozen.rs {dependency}\n");
            assert!(compiler_dependencies(&prepared, bytes.as_bytes()).is_err(), "{dependency}");
        }
        assert!(compiler_dependencies(&prepared, b"out: /inputs/rust/src/domain.rs\n").is_err());
    }

    #[test]
    fn infrastructure_nonzero_confined_gates_authenticate_and_replay_exactly() {
        let fixture = Fixture::new();
        let prepared = prepare(&fixture.root, &fixture.contract).unwrap();
        let runtime = fixture.runtime();
        let output = fixture.root.join("gate");
        let context = fixture.context("baseline");
        let receipt = check_ready(&prepared, &fixture.root, &output, &runtime, &context).unwrap();
        assert!(receipt.passed, "{}", receipt.diagnostic);
        let verdict = authenticate(&prepared, &fixture.root, &receipt).unwrap();
        assert_eq!(verdict.tested_count, 2);
        assert_eq!(verdict.operations.len(), 3);
        assert_eq!(verdict.evidence_kind, "infrastructure");
        assert_eq!(verdict.acceptance_model, ACCEPTANCE_MODEL);
        assert_eq!(verdict.actual_c, "not-tested");
        assert_eq!(verdict.compiler_outputs.len(), 2);
        let retained = references(&receipt).unwrap();
        for reference in verdict.compiler_outputs.values() {
            assert!(retained.contains(reference));
        }
        let before = records(&fixture.root.join("operations"));
        assert_eq!(run(&prepared, &fixture.root, &output, &runtime, &context).unwrap(), receipt);
        assert_eq!(before, records(&fixture.root.join("operations")));
        let relocated = fixture.root.join("retained-source");
        for name in prepared.contract.source_files.keys() {
            let destination = relocated.join(name);
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            fs::copy(fixture.root.join(name), destination).unwrap();
        }
        assert!(authenticate(&prepared, &relocated, &receipt).is_ok());
        fs::write(fixture.root.join("src/domain.rs"), "pub fn doubled(value: u32) -> u32 { value + value }\n").unwrap();
        let candidate =
            run(&prepared, &fixture.root, &fixture.root.join("candidate"), &runtime, &fixture.context("candidate"))
                .unwrap();
        assert!(candidate.passed, "{}", candidate.diagnostic);
        assert_ne!(candidate.candidate_sha256, receipt.candidate_sha256);
        assert!(authenticate(&prepared, &fixture.root, &receipt).is_err());
        assert!(authenticate(&prepared, &fixture.root, &candidate).is_ok());
    }

    #[test]
    fn confined_standalone_linker_replays_with_readonly_source_inputs() {
        let fixture = Fixture::new();
        let runtime = fixture.runtime();
        let snapshots = fixture.root.join("compiler-input");
        for name in fixture.contract.source_files.keys() {
            publish_bytes(&snapshots.join(name), &fs::read(fixture.root.join(name)).unwrap(), false).unwrap();
        }
        let program = Program {
            executable: runtime.rustc.clone(),
            helper: runtime.helper.clone(),
            arguments: compiler_arguments(&fixture.contract.test_root),
            inputs: fixture
                .contract
                .source_files
                .keys()
                .map(|name| ReadOnlyBind {
                    source: snapshots.join(name),
                    destination: Path::new("/inputs/rust").join(name),
                })
                .collect(),
            runtime: builds::strict_compiler_runtime(&runtime.rustc).unwrap(),
            outputs: vec!["tests".into(), "compiler.d".into()],
        };
        let limits = fixture.contract.compiler_limits.run_limits().unwrap();
        for attempt in 0..6 {
            let output = confinement::run(&program, &limits, b"")
                .unwrap_or_else(|error| panic!("strict compiler/linker {attempt} failed: {error:#}"));
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            assert!(!output.outputs[Path::new("tests")].is_empty());
        }
    }

    #[test]
    fn missing_coverage_provenance_dependency_and_writable_test_rules_stop_preparation() {
        let fixture = Fixture::new();
        let mut contract = fixture.contract.clone();
        contract.coverage.clear();
        assert!(prepare(&fixture.root, &contract).is_err());
        let mut contract = fixture.contract.clone();
        contract.provenance.sha256 = sha256("changed review");
        assert!(prepare(&fixture.root, &contract).is_err());
        let mut contract = fixture.contract.clone();
        contract.source_files.insert("src/missing.rs".into(), sha256("missing dependency"));
        assert!(prepare(&fixture.root, &contract).is_err());
        let mut contract = fixture.contract.clone();
        contract.approved_files.push(contract.test_root.clone());
        assert!(prepare(&fixture.root, &contract).is_err());
        fs::write(fixture.root.join("src/domain.rs"), "#[test] fn worker_gate() {}\n").unwrap();
        assert!(prepare(&fixture.root, &fixture.contract).is_err());
    }

    #[test]
    fn failed_test_is_bounded_and_exact_retry_produces_no_new_capture() {
        let fixture = Fixture::new();
        let prepared = prepare(&fixture.root, &fixture.contract).unwrap();
        fs::write(fixture.root.join("src/domain.rs"), "pub fn doubled(value: u32) -> u32 { value }\n").unwrap();
        let runtime = fixture.runtime();
        let output = fixture.root.join("failed");
        let context = fixture.context("failed");
        let receipt = run(&prepared, &fixture.root, &output, &runtime, &context).unwrap();
        assert!(!receipt.passed);
        let verdict = authenticate(&prepared, &fixture.root, &receipt).unwrap();
        assert_eq!(verdict.operations.last().unwrap().label, "tests", "{}", receipt.diagnostic);
        assert!(!receipt.diagnostic.is_empty() && receipt.diagnostic.len() <= 4096);
        assert_eq!(verdict.tested_count, 0);
        let before = records(&fixture.root.join("operations"));
        assert_eq!(run(&prepared, &fixture.root, &output, &runtime, &context).unwrap(), receipt);
        assert_eq!(before, records(&fixture.root.join("operations")));
        let mut foreign = context.clone();
        foreign.operation_key = sha256("foreign failure retry");
        assert!(run(&prepared, &fixture.root, &output, &runtime, &foreign).is_err());
        fs::write(fixture.root.join("tests/frozen.rs"), "changed harness\n").unwrap();
        assert!(authenticate(&prepared, &fixture.root, &receipt).is_err());
    }

    #[test]
    fn outer_deadline_refuses_setup_then_exact_replay_after_expiry_reuses_receipt() {
        let fixture = Fixture::new();
        let prepared = prepare(&fixture.root, &fixture.contract).unwrap();
        let mut runtime = fixture.runtime();
        runtime.deadline_unix_ms = Some(1);
        let output = fixture.root.join("expired");
        let context = fixture.context("expired");
        assert!(
            run(&prepared, &fixture.root, &output, &runtime, &context).unwrap_err().to_string().contains("time budget")
        );
        assert!(!runtime.operation_root.exists() && !output.exists());
        runtime.deadline_unix_ms = None;
        let receipt = run(&prepared, &fixture.root, &output, &runtime, &context).unwrap();
        assert!(receipt.passed, "{}", receipt.diagnostic);
        let mut intent: InfrastructureIntent = family::object(&output.join("infrastructure-intent.toml")).unwrap();
        intent.deadline_unix_ms = 1;
        atomic_toml(&output.join("infrastructure-intent.toml"), &intent).unwrap();
        let before = records(&output);
        assert_eq!(run(&prepared, &fixture.root, &output, &runtime, &context).unwrap(), receipt);
        assert_eq!(before, records(&output));
        assert!(authenticate(&prepared, &fixture.root, &receipt).is_ok());
        let receipt_path = output.join("infrastructure-receipt.toml");
        let bytes = fs::read(&receipt_path).unwrap();
        fs::remove_file(&receipt_path).unwrap();
        assert_eq!(run(&prepared, &fixture.root, &output, &runtime, &context).unwrap(), receipt);
        assert_eq!(fs::read(&receipt_path).unwrap(), bytes);
        assert_eq!(before, records(&output));
    }

    #[test]
    fn infrastructure_aggregate_reservation_refuses_compiler_before_launch() {
        let fixture = Fixture::new();
        let prepared = prepare(&fixture.root, &fixture.contract).unwrap();
        let mut runtime = fixture.runtime();
        runtime.max_output_bytes = 2 * 1024 * 1024;
        let receipt =
            run(&prepared, &fixture.root, &fixture.root.join("too-small"), &runtime, &fixture.context("too-small"))
                .unwrap();
        assert!(!receipt.passed && receipt.diagnostic.contains("budget"), "{}", receipt.diagnostic);
        assert!(!runtime.operation_root.exists());
        assert_eq!(authenticate(&prepared, &fixture.root, &receipt).unwrap().operations.len(), 0);
    }

    #[test]
    fn compiled_infrastructure_candidate_cannot_read_controller_or_overflow_tmp() {
        let fixture = Fixture::new();
        let prepared = prepare(&fixture.root, &fixture.contract).unwrap();
        let secret = fixture.root.join("controller-policy.json");
        fs::write(&secret, b"protected").unwrap();
        fs::write(fixture.root.join("src/domain.rs"), format!(
            "pub fn doubled(value: u32) -> u32 {{ assert!(std::fs::read({:?}).is_err()); assert!(std::fs::write({:?}, b\"attack\").is_err()); value * 2 }}\n",
            secret.display().to_string(), secret.display().to_string()
        )).unwrap();
        let runtime = fixture.runtime();
        let denied =
            run(&prepared, &fixture.root, &fixture.root.join("denied"), &runtime, &fixture.context("denied")).unwrap();
        assert!(denied.passed, "{}", denied.diagnostic);
        assert_eq!(fs::read(&secret).unwrap(), b"protected");
        fs::write(
            fixture.root.join("src/domain.rs"),
            r#"
pub fn doubled(value: u32) -> u32 {
    use std::io::Write;
    let mut file = std::fs::File::create("/tmp/overflow").unwrap();
    while file.write_all(&[b'q'; 4096]).is_ok() {}
    drop(file);
    let _ = std::fs::remove_file("/tmp/overflow");
    value * 2
}
"#,
        )
        .unwrap();
        let overflow =
            run(&prepared, &fixture.root, &fixture.root.join("overflow"), &runtime, &fixture.context("overflow"))
                .unwrap();
        assert!(!overflow.passed && overflow.diagnostic.contains("quota"), "{}", overflow.diagnostic);
    }
}
