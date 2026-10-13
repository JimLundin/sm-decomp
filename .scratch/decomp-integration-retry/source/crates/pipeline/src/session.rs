//! Maintained infrastructure admission and durable, scoped worker checkpoints.
//! Agent launch and observed liveness remain with the session's caller.
use crate::assignment::{self, Artifact, Claim, Job, Kind, SessionAuthority, State, Update, WorkResult};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::process::CommandExt,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NextCommand {
    pub cwd: PathBuf,
    pub argv: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    pub timeout_seconds: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Gate {
    pub argv: Vec<String>,
    pub timeout_seconds: u64,
    #[serde(default)]
    pub integration: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InfrastructureScope {
    pub schema: String,
    pub baseline: String,
    pub job: Job,
    pub gates: Vec<Gate>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "domain", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Admission {
    /// Exact old request retained by the sealed legacy bridge; no new admission.
    Legacy { original_request: Artifact },
    /// Reviewed scope artifacts, sealed before queue publication.
    Infrastructure { scopes: Vec<Artifact> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessObservation {
    pub pid: u32,
    pub boot_id: String,
    pub start_ticks: u64,
    pub argv: Vec<String>,
    pub cwd: PathBuf,
    pub observed_at: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Cleanup {
    pub status: String,
    pub reclaimed_bytes: u64,
    pub receipts: Vec<Artifact>,
    pub retained_dependencies: Vec<Artifact>,
    pub remaining_action: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Handoff {
    pub schema: String,
    pub sequence: u64,
    pub session_id: String,
    pub worker_session_id: String,
    /// Display name; the queue stores worker_session_id as its actual owner.
    pub agent: String,
    pub authority_generation: u64,
    pub effort: String,
    pub claim: Claim,
    pub job: Job,
    pub baseline: String,
    pub branch: Option<String>,
    pub worktree: PathBuf,
    pub source_commit: String,
    pub submitted_commit: Option<String>,
    #[serde(default)]
    pub runs: Vec<PathBuf>,
    #[serde(default)]
    pub processes: Vec<ProcessObservation>,
    #[serde(default)]
    pub evidence: Vec<Artifact>,
    #[serde(default)]
    pub dependencies: Vec<Artifact>,
    #[serde(default)]
    pub stages: Vec<String>,
    #[serde(default)]
    pub limitations: Vec<String>,
    pub cleanup: Cleanup,
    pub next: NextCommand,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Request {
    InfrastructurePlan { queue: assignment::Request, admission: Admission },
    InfrastructureNext { queue: assignment::Request, admission: Admission },
    InfrastructureRecord { queue: assignment::Request, admission: Admission },
    Inspect { queue: assignment::Request, handoff: Option<PathBuf> },
    Start { queue: assignment::Request, handoff: Box<Handoff> },
    Checkpoint { queue: assignment::Request, handoff: Box<Handoff>, message: String },
    Resume { queue: assignment::Request, handoff: PathBuf },
    Adopt { queue: assignment::Request, handoff: PathBuf, transfer: Artifact },
}
impl Request {
    pub(crate) fn operation(&self) -> &'static str {
        match self {
            Self::InfrastructurePlan { .. } => "infrastructure-plan",
            Self::InfrastructureNext { .. } => "infrastructure-next",
            Self::InfrastructureRecord { .. } => "infrastructure-record",
            Self::Inspect { .. } => "inspect",
            Self::Start { .. } => "start",
            Self::Checkpoint { .. } => "checkpoint",
            Self::Resume { .. } => "resume",
            Self::Adopt { .. } => "adopt",
        }
    }
}

fn hash(bytes: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(bytes.as_ref()))
}
fn checked(artifact: &Artifact) -> Result<()> {
    no_symlinks(&artifact.path)?;
    ensure!(
        artifact.path.is_absolute() && artifact.path.is_file() && !artifact.path.is_symlink(),
        "artifact must be an absolute regular file"
    );
    ensure!(crate::digest(&artifact.path)? == artifact.sha256, "artifact changed: {}", artifact.path.display());
    Ok(())
}
fn artifact(path: PathBuf) -> Result<Artifact> {
    Ok(Artifact { sha256: crate::digest(&path)?, path })
}
fn exact_commit(commit: &str) -> bool {
    commit.len() == 40 && commit.bytes().all(|byte| byte.is_ascii_hexdigit())
}
fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}
fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|component| matches!(component, Component::Normal(_)))
}
fn no_symlinks(path: &Path) -> Result<()> {
    let mut prefix = PathBuf::new();
    for component in path.components() {
        ensure!(!matches!(component, Component::ParentDir), "unsafe path traversal");
        prefix.push(component.as_os_str());
        ensure!(!prefix.is_symlink(), "path crosses a symlink: {}", prefix.display());
    }
    Ok(())
}
fn git(checkout: &Path, argv: &[&str]) -> Result<String> {
    let result = Command::new("git").args(argv).current_dir(checkout).output()?;
    ensure!(result.status.success(), "git {} failed: {}", argv.join(" "), String::from_utf8_lossy(&result.stderr));
    Ok(String::from_utf8(result.stdout)?.trim().into())
}
fn paths(checkout: &Path, argv: &[&str]) -> Result<Vec<PathBuf>> {
    let result = Command::new("git").args(argv).current_dir(checkout).output()?;
    ensure!(result.status.success(), "cannot inspect scoped Git paths");
    result
        .stdout
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
        .map(|bytes| Ok(PathBuf::from(std::str::from_utf8(bytes)?)))
        .collect()
}
fn dirty(checkout: &Path) -> Result<Vec<PathBuf>> {
    let mut changed = paths(checkout, &["diff", "--name-only", "--no-renames", "-z", "--"])?;
    changed.extend(paths(checkout, &["diff", "--cached", "--name-only", "--no-renames", "-z", "--"])?);
    changed.extend(paths(checkout, &["ls-files", "--others", "--exclude-standard", "-z"])?);
    changed.sort();
    changed.dedup();
    Ok(changed)
}
fn source_dirty(checkout: &Path) -> Result<Vec<PathBuf>> {
    let mut changed = paths(checkout, &["diff", "--name-only", "--no-renames", "-z", "--"])?;
    changed.extend(paths(checkout, &["diff", "--cached", "--name-only", "--no-renames", "-z", "--"])?);
    changed.extend(paths(checkout, &["ls-files", "--others", "--exclude-standard", "-z"])?.into_iter().filter(
        |path| {
            ["crates", "docs", "tools", ".cargo"].iter().any(|root| path.starts_with(root))
                || ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "AGENTS.md", "GLOSSARY.md"]
                    .iter()
                    .any(|name| path == Path::new(name))
        },
    ));
    Ok(changed)
}
fn scoped(checkout: &Path, job: &Job, changed: &[PathBuf]) -> Result<()> {
    for path in changed {
        ensure!(
            relative(path) && job.writable_paths.contains(path),
            "change exceeds approved files: {}",
            path.display()
        );
        no_symlinks(&checkout.join(path))?;
    }
    Ok(())
}
fn cargo_gate(argv: &[String]) -> Result<()> {
    ensure!(
        argv.len() >= 3 && argv.len() <= 64 && argv[0] == "cargo" && argv[1] == "test",
        "infrastructure gates require reviewed cargo test argv"
    );
    ensure!(argv.iter().all(|arg| !arg.is_empty() && arg.len() <= 4096 && !arg.contains('\0')), "invalid gate argv");
    ensure!(
        argv.iter().any(|arg| arg == "--locked") && argv.iter().any(|arg| arg == "--offline"),
        "infrastructure tests must be locked and offline"
    );
    ensure!(
        !argv.iter().any(|arg| arg.starts_with("--manifest-path")
            || arg.starts_with("--config")
            || arg.starts_with("--target-dir")),
        "gate must use the assigned workspace and private target"
    );
    Ok(())
}
fn next_command(handoff: &Handoff) -> Result<()> {
    ensure!(
        handoff.next.cwd == handoff.worktree && handoff.next.cwd.is_absolute(),
        "next command must use the assigned worktree"
    );
    cargo_gate(&handoff.next.argv)?;
    ensure!((1..=3600).contains(&handoff.next.timeout_seconds), "next command timeout must be 1..3600 seconds");
    for (name, value) in &handoff.next.env {
        ensure!(
            ["SM_PIPELINE_ROOT", "SM_PIPELINE_WORKERS", "CARGO_TARGET_DIR", "RUST_BACKTRACE"].contains(&name.as_str()),
            "unreviewed next-command environment: {name}"
        );
        ensure!(!value.contains('\0'), "invalid environment value");
        if name == "SM_PIPELINE_WORKERS" {
            ensure!(value == "4", "shared process-worker budget must be 4");
        }
    }
    Ok(())
}

fn read_scope(seal: &Artifact) -> Result<InfrastructureScope> {
    checked(seal)?;
    let scope: InfrastructureScope = toml::from_str(&fs::read_to_string(&seal.path)?)?;
    checked(seal)?;
    ensure!(
        scope.schema == "decomp-infrastructure-scope-v1" && exact_commit(&scope.baseline),
        "unsupported infrastructure scope or inexact baseline"
    );
    ensure!(
        scope.job.kind == Kind::Implement && !scope.job.writable_paths.is_empty(),
        "infrastructure scope must declare exact implementation files"
    );
    ensure!(
        scope.job.writable_paths.iter().all(|path| relative(path)
            && (path.starts_with("crates/pipeline/src")
                || path.starts_with("crates/pipeline/tests")
                || path.starts_with("docs"))),
        "infrastructure ownership must use pipeline tools, tests or docs"
    );
    ensure!(scope.job.inputs.get("base-head") == Some(&scope.baseline), "scope baseline differs from job baseline");
    ensure!(
        !scope.job.inputs.contains_key("scope-file") && !scope.job.inputs.contains_key("scope-sha256"),
        "native gameplay scopes require assign admission"
    );
    ensure!(
        !scope.gates.is_empty()
            && scope.gates.iter().any(|gate| !gate.integration)
            && scope.gates.iter().any(|gate| gate.integration),
        "infrastructure needs submission and integration gates"
    );
    for gate in &scope.gates {
        cargo_gate(&gate.argv)?;
        ensure!(
            (1..=if gate.integration { 3600 } else { 60 }).contains(&gate.timeout_seconds),
            "gate timeout exceeds reviewed bound"
        );
    }
    Ok(scope)
}
fn scope_job(seal: &Artifact, scope: &InfrastructureScope) -> Job {
    let mut job = scope.job.clone();
    job.inputs.insert("admission-domain".into(), "infrastructure-v1".into());
    job.inputs.insert("infrastructure-scope-file".into(), seal.path.display().to_string());
    job.inputs.insert("infrastructure-scope-sha256".into(), seal.sha256.clone());
    job.revision = hash(toml::to_string(&job).expect("serializable infrastructure job"));
    job
}
fn status(queue: &assignment::Request) -> Result<assignment::Receipt> {
    let mut request = queue.clone();
    request.operation = "status".into();
    request.jobs.clear();
    request.update = None;
    assignment::operate(request)
}
fn current<'a>(receipt: &'a assignment::Receipt, claim: &Claim) -> Result<&'a assignment::Entry> {
    let entry = receipt.jobs.iter().find(|entry| entry.job.id == claim.id).context("handoff claim is absent")?;
    ensure!(
        entry.claim.as_ref() == Some(claim) && !entry.stale && entry.pending_job.is_none(),
        "stale claim, attempt or job revision"
    );
    Ok(entry)
}
fn credential(queue: &assignment::Request, integration: bool) -> Result<&SessionAuthority> {
    let credential = queue.authority.as_ref().context("real session authority is required")?;
    assignment::authority::validate(&queue.checkout, &queue.effort, &queue.coordinator, Some(credential), integration)?;
    Ok(credential)
}

#[derive(Deserialize)]
struct LegacyRequestSeal {
    path: PathBuf,
    sha256: String,
    domain: String,
    accepted_as_native_gameplay: bool,
}
#[derive(Deserialize)]
struct LegacyBridge {
    schema: String,
    #[serde(default)]
    legacy_request_receipt_artifacts: Vec<LegacyRequestSeal>,
}
fn legacy(queue: &assignment::Request, seal: &Artifact) -> Result<()> {
    checked(seal)?;
    let mut original: assignment::Request = toml::from_str(&fs::read_to_string(&seal.path)?)?;
    checked(seal)?;
    // New optional authority is transport metadata; the legacy contract is exact.
    original.authority = queue.authority.clone();
    ensure!(
        toml::Value::try_from(&original)? == toml::Value::try_from(queue)?,
        "legacy request differs from its original seal"
    );
    let root = assignment::main_checkout(&queue.checkout)?;
    let policy =
        assignment::authority::operate(assignment::authority::Request { checkout: root, ..Default::default() })?;
    let bridge_seal = policy.bridge.context("pinned legacy bridge is required")?;
    checked(&bridge_seal)?;
    let bridge: LegacyBridge = toml::from_str(&fs::read_to_string(&bridge_seal.path)?)?;
    checked(&bridge_seal)?;
    ensure!(bridge.schema == "decomp-legacy-owner-bridge-v1", "unsupported legacy bridge");
    ensure!(
        bridge.legacy_request_receipt_artifacts.iter().any(|entry| {
            entry.path == seal.path
                && entry.sha256 == seal.sha256
                && !entry.accepted_as_native_gameplay
                && ["legacy-infrastructure-plan-request", "legacy-start-request", "legacy-submit-request"]
                    .contains(&entry.domain.as_str())
        }),
        "original legacy request is not listed as a request by the pinned reviewed bridge"
    );
    Ok(())
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResultPublication {
    schema: String,
    requested: Update,
    admitted: Update,
    scope: Artifact,
    checkout: Option<PathBuf>,
    source_commit: Option<String>,
    source_fingerprint: Option<String>,
}
fn publication(root: &Path, queue: &assignment::Request, claim: &Claim) -> Result<PathBuf> {
    ensure!(safe_name(&claim.id) && safe_name(&queue.effort), "unsafe result identity");
    let path = root
        .join(format!(".scratch/{}/receipts/{}-{}-infrastructure-result.toml", queue.effort, claim.id, claim.attempt));
    no_symlinks(&path)?;
    Ok(path)
}
fn run_gates(
    root: &Path,
    queue: &assignment::Request,
    claim: &Claim,
    scope: &InfrastructureScope,
    checkout: &Path,
    integration: bool,
) -> Result<Vec<Artifact>> {
    let before = assignment::source_fingerprint(checkout)?;
    let before_commit = git(checkout, &["rev-parse", "HEAD"])?;
    let run = root.join(format!(
        ".scratch/{}/receipts/{}-{}-infrastructure-gates-{}-{}",
        queue.effort,
        claim.id,
        claim.attempt,
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    no_symlinks(&run)?;
    fs::create_dir_all(run.parent().unwrap())?;
    fs::create_dir(&run)?;
    let mut evidence = Vec::new();
    for (index, gate) in scope.gates.iter().enumerate().filter(|(_, gate)| gate.integration == integration) {
        let stdout = run.join(format!("{index}.stdout"));
        let stderr = run.join(format!("{index}.stderr"));
        let target = run.join("target");
        let mut actual_argv = gate.argv.clone();
        let manifest: toml::Value = toml::from_str(&fs::read_to_string(checkout.join("Cargo.toml"))?)?;
        // Isolated Git worktrees omit this repository's retained dependency.
        // Resolve only its existing declared patch, never caller-supplied config.
        if manifest
            .get("patch")
            .and_then(|patch| patch.get("https://github.com/doublegate/RustySNES"))
            .and_then(|patch| patch.get("rustysnes-core"))
            .and_then(|patch| patch.get("path"))
            .and_then(toml::Value::as_str)
            == Some("deps/rustysnes/crates/rustysnes-core")
            && !checkout.join("deps/rustysnes/crates/rustysnes-core").exists()
        {
            let retained = root.join("deps/rustysnes/crates/rustysnes-core");
            no_symlinks(&retained)?;
            ensure!(retained.join("Cargo.toml").is_file(), "retained RustySNES dependency is missing");
            let config = format!(
                "patch.\"https://github.com/doublegate/RustySNES\".rustysnes-core.path={}",
                toml::Value::String(retained.display().to_string())
            );
            actual_argv.splice(2..2, ["--config".into(), config]);
        }
        let mut command = Command::new(&actual_argv[0]);
        command
            .args(&actual_argv[1..])
            .current_dir(checkout)
            .process_group(0)
            .env("SM_PIPELINE_ROOT", root.join("out/pipeline"))
            .env("SM_PIPELINE_WORKERS", "4")
            .env("CARGO_TARGET_DIR", &target)
            .stdin(Stdio::null())
            .stdout(File::create(&stdout)?)
            .stderr(File::create(&stderr)?);
        let mut child = command.spawn()?;
        let started = Instant::now();
        let (exit, timed_out) = loop {
            if let Some(exit) = child.try_wait()? {
                break (exit, false);
            }
            if started.elapsed() >= Duration::from_secs(gate.timeout_seconds) {
                // SAFETY: this command owns the child's process group until reaped.
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                break (child.wait()?, true);
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        let path = run.join(format!("{index}.toml"));
        atomic(
            &path,
            toml::to_string_pretty(&toml::Value::try_from(BTreeMap::from([
                ("schema", toml::Value::String("decomp-infrastructure-gate-v1".into())),
                ("argv", toml::Value::try_from(&actual_argv)?),
                ("checkout", toml::Value::String(checkout.display().to_string())),
                ("source_commit", toml::Value::String(before_commit.clone())),
                ("source_fingerprint", toml::Value::String(before.clone())),
                ("exit_code", toml::Value::Integer(exit.code().unwrap_or(-1) as i64)),
                ("timed_out", toml::Value::Boolean(timed_out)),
                ("stdout_sha256", toml::Value::String(crate::digest(&stdout)?)),
                ("stderr_sha256", toml::Value::String(crate::digest(&stderr)?)),
            ]))?)?
            .as_bytes(),
        )?;
        evidence.extend([artifact(path)?, artifact(stdout.clone())?, artifact(stderr)?]);
        if target.exists() {
            fs::remove_dir_all(&target)?;
        }
        ensure!(exit.success() && !timed_out, "infrastructure gate failed; retained evidence: {}", run.display());
        let output = fs::read_to_string(stdout)?;
        ensure!(
            output.lines().any(|line| line
                .strip_prefix("running ")
                .and_then(|text| text.split_whitespace().next())
                .and_then(|count| count.parse::<usize>().ok())
                .is_some_and(|count| count > 0)),
            "infrastructure gate executed zero tests; retained evidence: {}",
            run.display()
        );
    }
    ensure!(
        assignment::source_fingerprint(checkout)? == before
            && git(checkout, &["rev-parse", "HEAD"])? == before_commit
            && source_dirty(checkout)?.is_empty(),
        "source changed during infrastructure gates"
    );
    credential(queue, integration)?;
    Ok(evidence)
}
fn implementation(
    root: &Path,
    queue: &assignment::Request,
    entry: &assignment::Entry,
    scope: &InfrastructureScope,
    commit: &str,
    integration: bool,
) -> Result<PathBuf> {
    ensure!(exact_commit(commit), "exact submitted commit is required");
    let claim = entry.claim.as_ref().context("submission claim missing")?;
    let worktree = std::env::temp_dir().join(format!("sm-{}-{}-{}", queue.effort, claim.id, claim.attempt));
    let checkout = if integration { root } else { worktree.as_path() };
    no_symlinks(checkout)?;
    ensure!(assignment::main_checkout(checkout)? == root, "submission must use assigned repository");
    if !integration {
        ensure!(
            checkout != root
                && checkout.join(".git").is_file()
                && git(checkout, &["branch", "--show-current"])?
                    == format!("{}-{}-{}", queue.effort, claim.id, claim.attempt),
            "submission must use assigned branch/worktree"
        );
        ensure!(
            git(checkout, &["rev-parse", "HEAD"])? == commit && dirty(checkout)?.is_empty(),
            "submitted worktree must be clean at exact commit"
        );
    }
    git(checkout, &["merge-base", "--is-ancestor", &scope.baseline, commit])?;
    let changed = paths(checkout, &["diff", "--name-only", "--no-renames", "-z", &scope.baseline, commit, "--"])?;
    scoped(checkout, &scope.job, &changed)?;
    ensure!(!changed.is_empty(), "submission contains no scoped change");
    if integration {
        git(checkout, &["merge-base", "--is-ancestor", commit, "HEAD"])?;
        ensure!(source_dirty(checkout)?.is_empty(), "integration source must be frozen and clean");
    }
    Ok(checkout.to_path_buf())
}
fn completed_integration_retry(
    root: &Path,
    entry: &assignment::Entry,
    submitted: &assignment::Entry,
    saved: &ResultPublication,
) -> Result<()> {
    let Update::Finished {
        claim,
        result: result @ WorkResult::Integrated { submissions, evidence, authentication, cleanup },
    } = &saved.admitted
    else {
        bail!("cached result is not integrated")
    };
    ensure!(
        entry.state == State::Complete
            && entry.claim.as_ref() == Some(claim)
            && entry.result.as_ref() == Some(result)
            && submitted.state == State::Complete
            && submissions.as_slice()
                == [submitted.claim.as_ref().context("completed submission claim missing")?.clone()],
        "cached integration lacks exact completed lineage"
    );
    let checkout = saved.checkout.as_ref().context("cached integration checkout missing")?;
    no_symlinks(checkout)?;
    ensure!(fs::canonicalize(checkout)? == root, "cached integration names another checkout");
    ensure!(source_dirty(root)?.is_empty(), "cached integration source must be frozen and clean");
    let source = saved.source_commit.as_deref().context("cached integration source missing")?;
    let fingerprint = saved.source_fingerprint.as_deref().context("cached integration fingerprint missing")?;
    ensure!(exact_commit(source), "cached integration source is not an exact commit");
    git(root, &["merge-base", "--is-ancestor", source, "HEAD"])?;
    let Some(WorkResult::Submitted { commit, .. }) = &submitted.result else {
        bail!("completed submission source missing")
    };
    git(root, &["merge-base", "--is-ancestor", commit, source])?;

    // This replays the sealed historical acceptance only. Later clean merged
    // source gains no new acceptance, gate output or source identity from it.
    let auth: toml::Value = toml::from_str(&fs::read_to_string(&authentication.path)?)?;
    ensure!(
        auth.get("schema").and_then(toml::Value::as_str) == Some("decomp-infrastructure-authentication-v1")
            && auth.get("admission_domain").and_then(toml::Value::as_str) == Some("infrastructure-v1")
            && auth.get("source_commit").and_then(toml::Value::as_str) == Some(source)
            && auth.get("claim").cloned().context("cached authentication claim missing")?.try_into::<Claim>()?
                == *claim
            && auth
                .get("evidence")
                .cloned()
                .context("cached authentication evidence missing")?
                .try_into::<Vec<Artifact>>()?
                == *evidence,
        "cached integration authentication differs from historical source"
    );
    let mut gates = 0;
    for seal in evidence {
        let Some(gate) = fs::read_to_string(&seal.path).ok().and_then(|text| toml::from_str::<toml::Value>(&text).ok())
        else {
            continue;
        };
        if gate.get("schema").and_then(toml::Value::as_str) != Some("decomp-infrastructure-gate-v1") {
            continue;
        }
        ensure!(
            gate.get("source_commit").and_then(toml::Value::as_str) == Some(source)
                && gate.get("source_fingerprint").and_then(toml::Value::as_str) == Some(fingerprint)
                && gate.get("exit_code").and_then(toml::Value::as_integer) == Some(0)
                && gate.get("timed_out").and_then(toml::Value::as_bool) == Some(false),
            "cached integration gate differs from historical source"
        );
        for key in ["stdout_sha256", "stderr_sha256"] {
            ensure!(
                evidence.iter().any(|seal| Some(seal.sha256.as_str()) == gate.get(key).and_then(toml::Value::as_str)),
                "cached integration gate output is unsealed"
            );
        }
        gates += 1;
    }
    ensure!(gates > 0, "cached integration lacks historical gate evidence");
    let clean: Cleanup = toml::from_str(&fs::read_to_string(&cleanup.path)?)?;
    ensure!(clean.status == "complete" && clean.remaining_action.is_none(), "cached integration cleanup incomplete");
    for seal in clean.receipts.iter().chain(&clean.retained_dependencies) {
        checked(seal)?;
    }
    Ok(())
}
fn admit_result(
    root: &Path,
    queue: &assignment::Request,
    status: &assignment::Receipt,
    seals: &[Artifact],
    update: &Update,
) -> Result<Update> {
    let claim = match update {
        Update::Finished { claim, .. }
        | Update::Started { claim, .. }
        | Update::LaunchFailed { claim, .. }
        | Update::OwnerStopped { claim, .. } => claim,
    };
    let entry = current(status, claim)?;
    let submitted = if entry.job.kind == Kind::Integrate {
        let id = entry.job.inputs.get("submission-id").context("integration submission missing")?;
        let submission = status.jobs.iter().find(|entry| &entry.job.id == id).context("submitted claim absent")?;
        let exact = submission.claim.as_ref().context("submitted claim missing")?;
        ensure!(
            entry.job.inputs.get("submission-revision") == Some(&exact.revision)
                && entry.job.inputs.get("submission-attempt") == Some(&exact.attempt.to_string()),
            "integration must name its exact submitted claim"
        );
        current(status, exact)?
    } else {
        entry
    };
    let mut matched = None;
    for seal in seals {
        let scope = read_scope(seal)?;
        if scope_job(seal, &scope) == submitted.job {
            matched = Some((seal.clone(), scope));
            break;
        }
    }
    let (scope_seal, scope) = matched.context("update lacks its exact reviewed infrastructure scope")?;
    credential(queue, entry.job.kind == Kind::Integrate)?;
    let Update::Finished { result, .. } = update else { return Ok(update.clone()) };
    for seal in assignment::result_artifacts(result) {
        checked(seal)?;
    }
    ensure!(
        matches!(result, WorkResult::Submitted { .. } | WorkResult::Integrated { .. } | WorkResult::Failed { .. }),
        "infrastructure admits submitted, integrated or failed results only"
    );
    if let WorkResult::Integrated { submissions, .. } = result {
        ensure!(
            entry.job.kind == Kind::Integrate && submissions.as_slice() == [submitted.claim.as_ref().unwrap().clone()],
            "integration names another submitted claim"
        );
    }
    if let WorkResult::Failed { followups, .. } = result {
        ensure!(followups.is_empty(), "infrastructure failure cannot promote raw jobs");
    }
    let path = publication(root, queue, claim)?;
    if path.exists() {
        let saved: ResultPublication = toml::from_str(&fs::read_to_string(&path)?)?;
        ensure!(
            saved.schema == "decomp-infrastructure-result-v1"
                && saved.requested == *update
                && saved.scope == scope_seal,
            "conflicting infrastructure result retry"
        );
        if let Update::Finished { result, .. } = &saved.admitted {
            for seal in assignment::result_artifacts(result) {
                checked(seal)?;
            }
        }
        if entry.state == State::Complete
            && matches!(saved.admitted, Update::Finished { result: WorkResult::Integrated { .. }, .. })
        {
            completed_integration_retry(root, entry, submitted, &saved)?;
            return Ok(saved.admitted);
        }
        if let Some(checkout) = &saved.checkout {
            if checkout.join(".git").exists() {
                ensure!(
                    source_dirty(checkout)?.is_empty()
                        && saved.source_commit.as_deref() == Some(git(checkout, &["rev-parse", "HEAD"])?.as_str())
                        && saved.source_fingerprint.as_deref()
                            == Some(assignment::source_fingerprint(checkout)?.as_str()),
                    "cached infrastructure source changed"
                );
            } else {
                // An authenticated integration retires its implementer. An
                // exact submission retry keeps the original gate seals and
                // follows that completed integrated claim instead of its Git.
                let Update::Finished { result: WorkResult::Submitted { commit, .. }, .. } = &saved.admitted else {
                    bail!("cached infrastructure checkout is missing")
                };
                ensure!(
                    entry.state == State::Complete
                        && entry.result.as_ref()
                            == Some(match &saved.admitted {
                                Update::Finished { result, .. } => result,
                                _ => unreachable!(),
                            }),
                    "retired submission lacks completed integration"
                );
                let integrated = status
                    .jobs
                    .iter()
                    .find(|integration| {
                        integration.state == State::Complete
                            && integration.job.inputs.get("submission-id") == Some(&claim.id)
                            && integration.job.inputs.get("submission-revision") == Some(&claim.revision)
                            && integration.job.inputs.get("submission-attempt") == Some(&claim.attempt.to_string())
                    })
                    .context("retired submission has no exact integrated lineage")?;
                let Some(result @ WorkResult::Integrated { submissions, .. }) = &integrated.result else {
                    bail!("completed integration evidence missing")
                };
                ensure!(submissions.as_slice() == [claim.clone()], "completed integration names another submission");
                for seal in assignment::result_artifacts(result) {
                    checked(seal)?;
                }
                git(root, &["merge-base", "--is-ancestor", commit, "HEAD"])?;
            }
        }
        return Ok(saved.admitted);
    }
    if matches!(result, WorkResult::Integrated { .. }) {
        ensure!(submitted.state == State::Submitted, "infrastructure submission is not held");
    }
    if matches!(result, WorkResult::Failed { .. }) {
        atomic(
            &path,
            toml::to_string_pretty(&ResultPublication {
                schema: "decomp-infrastructure-result-v1".into(),
                requested: update.clone(),
                admitted: update.clone(),
                scope: scope_seal,
                checkout: None,
                source_commit: None,
                source_fingerprint: None,
            })?
            .as_bytes(),
        )?;
        return Ok(update.clone());
    }
    let mut admitted = update.clone();
    let Update::Finished { result: admitted_result, .. } = &mut admitted else { unreachable!() };
    let (commit, integration) = match result {
        WorkResult::Submitted { commit, .. } => (commit, false),
        WorkResult::Integrated { .. } => {
            let Some(WorkResult::Submitted { commit, .. }) = &submitted.result else {
                bail!("submitted source missing")
            };
            (commit, true)
        }
        _ => unreachable!(),
    };
    let checkout = implementation(root, queue, submitted, &scope, commit, integration)?;
    let gates = run_gates(root, queue, claim, &scope, &checkout, integration)?;
    match admitted_result {
        WorkResult::Submitted { evidence, .. } => evidence.extend(gates),
        WorkResult::Integrated { evidence, cleanup, authentication, .. } => {
            checked(cleanup)?;
            let clean: Cleanup = toml::from_str(&fs::read_to_string(&cleanup.path)?)?;
            ensure!(
                clean.status == "complete" && clean.remaining_action.is_none(),
                "integration cleanup is incomplete"
            );
            for seal in clean.receipts.iter().chain(&clean.retained_dependencies) {
                checked(seal)?;
            }
            let worktree = std::env::temp_dir().join(format!(
                "sm-{}-{}-{}",
                queue.effort,
                submitted.job.id,
                submitted.claim.as_ref().unwrap().attempt
            ));
            ensure!(!worktree.join(".git").exists(), "integrated implementer worktree remains registered");
            evidence.extend(gates);
            let auth =
                path.with_file_name(format!("{}-{}-infrastructure-authentication.toml", claim.id, claim.attempt));
            atomic(
                &auth,
                toml::to_string_pretty(&toml::Value::try_from(BTreeMap::from([
                    ("schema", toml::Value::String("decomp-infrastructure-authentication-v1".into())),
                    ("claim", toml::Value::try_from(claim)?),
                    ("source_commit", toml::Value::String(git(root, &["rev-parse", "HEAD"])?)),
                    ("evidence", toml::Value::try_from(&evidence)?),
                    ("admission_domain", toml::Value::String("infrastructure-v1".into())),
                ]))?)?
                .as_bytes(),
            )?;
            *authentication = artifact(auth)?;
        }
        _ => unreachable!(),
    }
    ensure!(source_dirty(&checkout)?.is_empty(), "infrastructure source changed before publication");
    atomic(
        &path,
        toml::to_string_pretty(&ResultPublication {
            schema: "decomp-infrastructure-result-v1".into(),
            requested: update.clone(),
            admitted: admitted.clone(),
            scope: scope_seal,
            source_commit: Some(git(&checkout, &["rev-parse", "HEAD"])?),
            source_fingerprint: Some(assignment::source_fingerprint(&checkout)?),
            checkout: Some(checkout),
        })?
        .as_bytes(),
    )?;
    Ok(admitted)
}

static NEXT: AtomicU64 = AtomicU64::new(0);
fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    no_symlinks(path)?;
    let parent = path.parent().context("publication parent missing")?;
    fs::create_dir_all(parent)?;
    let temporary =
        parent.join(format!(".session-{}-{}.pending", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().create_new(true).write(true).open(&temporary)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&temporary, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
fn handoff_path(root: &Path, handoff: &Handoff) -> Result<PathBuf> {
    ensure!(
        safe_name(&handoff.session_id) && safe_name(&handoff.worker_session_id) && safe_name(&handoff.claim.id),
        "handoff needs safe real identities"
    );
    let path = root.join(format!(
        ".scratch/decomp-parallel/sessions/{}/{}-{}.toml",
        handoff.session_id, handoff.claim.id, handoff.claim.attempt
    ));
    no_symlinks(&path)?;
    Ok(path)
}
fn read_handoff(root: &Path, path: &Path) -> Result<Handoff> {
    ensure!(
        path.is_absolute() && path.starts_with(root.join(".scratch/decomp-parallel/sessions")),
        "handoff must be in the shared session directory"
    );
    no_symlinks(path)?;
    let handoff: Handoff = toml::from_str(&fs::read_to_string(path)?)?;
    ensure!(handoff_path(root, &handoff)? == path, "handoff path differs from exact identity");
    Ok(handoff)
}
fn validate_handoff(
    root: &Path,
    queue: &assignment::Request,
    handoff: &Handoff,
    starting: bool,
    check_source: bool,
) -> Result<()> {
    ensure!(
        handoff.schema == "decomp-worker-handoff-v1" && handoff.sequence > 0,
        "unsupported handoff schema/sequence"
    );
    ensure!(
        handoff.effort == queue.effort && handoff.claim.coordinator == queue.coordinator,
        "handoff belongs to another effort/coordinator"
    );
    ensure!(
        !handoff.agent.is_empty()
            && assignment::authority::real_session_id(&handoff.worker_session_id)
            && assignment::authority::real_session_id(&handoff.session_id),
        "handoff requires actual session and worker identities"
    );
    let entry_status = status(queue)?;
    let entry = current(&entry_status, &handoff.claim)?;
    ensure!(entry.job == handoff.job, "approved job/files differ from current claim");
    ensure!(
        if starting {
            matches!(entry.state, State::Reserved | State::Running)
        } else {
            matches!(entry.state, State::Running | State::Submitted)
        },
        "worker state is not startable/resumable"
    );
    if entry.state != State::Reserved {
        ensure!(entry.agent.as_ref() == Some(&handoff.worker_session_id), "handoff names another worker");
    }
    ensure!(
        exact_commit(&handoff.baseline)
            && exact_commit(&handoff.source_commit)
            && handoff.job.inputs.get("base-head") == Some(&handoff.baseline),
        "handoff needs exact baseline/source commits"
    );
    ensure!(handoff.submitted_commit.as_ref().is_none_or(|commit| exact_commit(commit)), "inexact submitted commit");
    no_symlinks(&handoff.worktree)?;
    ensure!(handoff.worktree.is_absolute(), "worktree must be absolute");
    ensure!(assignment::main_checkout(&handoff.worktree)? == root, "handoff worktree uses another repository");
    if handoff.job.kind == Kind::Implement {
        let expected =
            std::env::temp_dir().join(format!("sm-{}-{}-{}", queue.effort, handoff.claim.id, handoff.claim.attempt));
        ensure!(
            handoff.worktree == expected && handoff.worktree != root && handoff.worktree.join(".git").is_file(),
            "handoff must use assigned isolated worktree"
        );
        let branch = format!("{}-{}-{}", queue.effort, handoff.claim.id, handoff.claim.attempt);
        ensure!(
            handoff.branch.as_deref() == Some(&branch)
                && git(&handoff.worktree, &["branch", "--show-current"])? == branch,
            "assigned branch differs"
        );
    }
    git(&handoff.worktree, &["merge-base", "--is-ancestor", &handoff.baseline, &handoff.source_commit])?;
    if handoff.job.kind == Kind::Integrate {
        ensure!(handoff.worktree == root, "integration handoff must use shared source");
        let id = handoff.job.inputs.get("submission-id").context("integration submission missing")?;
        let submitted = entry_status.jobs.iter().find(|entry| &entry.job.id == id).context("submitted claim absent")?;
        let claim = submitted.claim.as_ref().context("submitted claim missing")?;
        let submitted = current(&entry_status, claim)?;
        ensure!(
            handoff.job.inputs.get("submission-revision") == Some(&claim.revision)
                && handoff.job.inputs.get("submission-attempt") == Some(&claim.attempt.to_string())
                && submitted.state == State::Submitted
                && submitted.job.inputs.get("base-head") == Some(&handoff.baseline)
                && submitted.job.writable_paths == handoff.job.writable_paths,
            "integration must name its exact submitted claim"
        );
        let Some(WorkResult::Submitted { commit, .. }) = &submitted.result else {
            bail!("integration submitted source missing")
        };
        ensure!(
            handoff.submitted_commit.as_ref() == Some(commit),
            "handoff submitted commit differs from exact submission"
        );
        git(root, &["merge-base", "--is-ancestor", &handoff.baseline, commit])?;
        // Shared source may already contain other accepted scopes. Ownership
        // belongs to this exact submission, while the integration source stays
        // frozen at its separately checked shared commit.
        scoped(
            root,
            &submitted.job,
            &paths(root, &["diff", "--name-only", "--no-renames", "-z", &handoff.baseline, commit, "--"])?,
        )?;
        ensure!(source_dirty(root)?.is_empty(), "integration source must be frozen and clean");
    } else {
        scoped(
            &handoff.worktree,
            &handoff.job,
            &paths(&handoff.worktree, &["diff", "--name-only", "--no-renames", "-z", &handoff.baseline, "HEAD", "--"])?,
        )?;
    }
    if check_source {
        ensure!(
            git(&handoff.worktree, &["rev-parse", "HEAD"])? == handoff.source_commit,
            "handoff source commit is stale"
        );
    }
    for seal in handoff
        .evidence
        .iter()
        .chain(&handoff.dependencies)
        .chain(&handoff.cleanup.receipts)
        .chain(&handoff.cleanup.retained_dependencies)
    {
        checked(seal)?;
    }
    ensure!(["pending", "complete", "blocked"].contains(&handoff.cleanup.status.as_str()), "invalid cleanup state");
    ensure!(
        handoff.cleanup.status != "complete" || handoff.cleanup.remaining_action.is_none(),
        "completed cleanup has a remaining action"
    );
    for process in &handoff.processes {
        ensure!(
            process.pid > 0
                && process.start_ticks > 0
                && !process.boot_id.is_empty()
                && !process.argv.is_empty()
                && process.cwd.is_absolute()
                && !process.observed_at.is_empty(),
            "process observation lacks command/start identity"
        );
        ensure!(
            ["running", "completed", "failed", "interrupted", "ambiguous"].contains(&process.status.as_str()),
            "invalid process observation"
        );
    }
    next_command(handoff)?;
    Ok(())
}
fn render(handoff: &Handoff, path: &Path) -> Result<()> {
    atomic(&path.with_extension("md"), format!("# Worker handoff: {} attempt {}\n\nTyped authority: `{}`; sequence: `{}`.\nSession: `{}`; worker thread: `{}`; agent: `{}`.\nClaim: `({}, {}, {}, {})`.\nSource commit: `{}`; baseline: `{}`.\nWorktree: `{}`.\nCleanup: `{}`.\n\n```toml\n{}\n```\n", handoff.claim.id, handoff.claim.attempt, path.display(), handoff.sequence, handoff.session_id, handoff.worker_session_id, handoff.agent, handoff.claim.id, handoff.claim.revision, handoff.claim.attempt, handoff.claim.coordinator, handoff.source_commit, handoff.baseline, handoff.worktree.display(), handoff.cleanup.status, toml::to_string_pretty(handoff)?).as_bytes())
}
fn publish(root: &Path, handoff: &Handoff) -> Result<PathBuf> {
    let path = handoff_path(root, handoff)?;
    if path.exists() {
        let saved = read_handoff(root, &path)?;
        if saved.sequence == handoff.sequence {
            ensure!(saved == *handoff, "conflicting handoff sequence retry");
            render(handoff, &path)?;
            return Ok(path);
        }
        ensure!(
            handoff.sequence == saved.sequence + 1
                && saved.claim == handoff.claim
                && saved.job == handoff.job
                && saved.session_id == handoff.session_id
                && saved.worker_session_id == handoff.worker_session_id
                && saved.agent == handoff.agent,
            "handoff sequence or identity conflict"
        );
        let history = path
            .with_file_name(format!("{}-{}-sequence-{}.toml", handoff.claim.id, handoff.claim.attempt, saved.sequence));
        atomic(&history, toml::to_string_pretty(&saved)?.as_bytes())?;
    } else {
        ensure!(handoff.sequence == 1, "initial handoff sequence must be one");
    }
    atomic(&path, toml::to_string_pretty(handoff)?.as_bytes())?;
    render(handoff, &path)?;
    Ok(path)
}
fn worker_lock(root: &Path) -> Result<File> {
    // Same lock as authority transfer, held through scoped Git and publication.
    let path = root.join(".scratch/assignment.lock");
    no_symlinks(&path)?;
    fs::create_dir_all(path.parent().unwrap())?;
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    file.lock()?;
    Ok(file)
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointIntent {
    schema: String,
    requested: Handoff,
    message: String,
}
fn checkpoint(root: &Path, queue: &assignment::Request, mut handoff: Handoff, message: String) -> Result<PathBuf> {
    ensure!(
        !message.is_empty() && message.len() <= 4096 && !message.contains('\0'),
        "checkpoint needs a bounded commit message"
    );
    let _lock = worker_lock(root)?;
    let current_authority = credential(queue, false)?;
    ensure!(
        handoff.session_id == current_authority.session_id
            && handoff.authority_generation == current_authority.generation,
        "handoff authority differs from current session"
    );
    ensure!(
        handoff.job.kind == Kind::Implement && handoff.worktree != root,
        "scoped checkpoints require an assigned implementation worktree"
    );
    validate_handoff(root, queue, &handoff, false, false)?;
    let path = handoff_path(root, &handoff)?;
    let intent_path = path
        .with_file_name(format!("{}-{}-checkpoint-{}.toml", handoff.claim.id, handoff.claim.attempt, handoff.sequence));
    let intent = CheckpointIntent {
        schema: "decomp-checkpoint-intent-v1".into(),
        requested: handoff.clone(),
        message: message.clone(),
    };
    let head = git(&handoff.worktree, &["rev-parse", "HEAD"])?;
    if intent_path.exists() {
        let saved: CheckpointIntent = toml::from_str(&fs::read_to_string(&intent_path)?)?;
        ensure!(
            saved.schema == intent.schema && saved.requested == handoff && saved.message == message,
            "conflicting interrupted checkpoint retry"
        );
        if head != handoff.source_commit {
            ensure!(
                dirty(&handoff.worktree)?.is_empty()
                    && git(&handoff.worktree, &["rev-parse", "HEAD^"])? == handoff.source_commit
                    && git(&handoff.worktree, &["show", "-s", "--format=%B", "HEAD"])? == message,
                "interrupted checkpoint commit differs"
            );
            handoff.source_commit = head;
            credential(queue, false)?;
            return publish(root, &handoff);
        }
    } else {
        ensure!(head == handoff.source_commit, "checkpoint source commit is stale");
        if path.exists() {
            let saved = read_handoff(root, &path)?;
            ensure!(
                handoff.sequence == saved.sequence + 1
                    && saved.claim == handoff.claim
                    && saved.job == handoff.job
                    && saved.session_id == handoff.session_id
                    && saved.worker_session_id == handoff.worker_session_id
                    && saved.agent == handoff.agent
                    && saved.source_commit == handoff.source_commit,
                "checkpoint sequence/identity must follow latest handoff"
            );
        }
    }
    let changed = dirty(&handoff.worktree)?;
    scoped(&handoff.worktree, &handoff.job, &changed)?;
    atomic(&intent_path, toml::to_string_pretty(&intent)?.as_bytes())?;
    if !changed.is_empty() {
        // Validate every staged/unstaged/untracked path before touching the index.
        let mut command = Command::new("git");
        command.args(["add", "--"]).args(&changed).current_dir(&handoff.worktree);
        ensure!(command.status()?.success(), "scoped staging failed");
        scoped(
            &handoff.worktree,
            &handoff.job,
            &paths(&handoff.worktree, &["diff", "--cached", "--name-only", "--no-renames", "-z", "--"])?,
        )?;
        credential(queue, false)?;
        git(&handoff.worktree, &["commit", "-m", &message])?;
    }
    handoff.source_commit = git(&handoff.worktree, &["rev-parse", "HEAD"])?;
    ensure!(dirty(&handoff.worktree)?.is_empty(), "checkpoint left dirty worktree");
    credential(queue, false)?;
    publish(root, &handoff)
}
fn compact(queue: &assignment::Receipt) -> Result<toml::Value> {
    #[derive(Serialize)]
    struct Pointer<'a> {
        claim: &'a Option<Claim>,
        state: &'a State,
        agent: &'a Option<String>,
        stale: bool,
        pending_revision: Option<&'a str>,
    }
    #[derive(Serialize)]
    struct Receipt<'a> {
        status: &'a str,
        ledger: &'a Path,
        assignment: &'a Option<assignment::Assignment>,
        jobs: Vec<Pointer<'a>>,
        deferred: &'a BTreeMap<String, String>,
    }
    Ok(toml::Value::try_from(Receipt {
        status: &queue.status,
        ledger: &queue.ledger,
        assignment: &queue.assignment,
        jobs: queue
            .jobs
            .iter()
            .map(|entry| Pointer {
                claim: &entry.claim,
                state: &entry.state,
                agent: &entry.agent,
                stale: entry.stale,
                pending_revision: entry.pending_job.as_ref().map(|job| job.revision.as_str()),
            })
            .collect(),
        deferred: &queue.deferred,
    })?)
}
fn pointer(path: PathBuf, handoff: &Handoff, resumable: bool) -> Result<toml::Value> {
    #[derive(Serialize)]
    struct Pointer<'a> {
        status: &'a str,
        handoff: PathBuf,
        markdown: PathBuf,
        sequence: u64,
        claim: &'a Claim,
        source_commit: &'a str,
        next: &'a NextCommand,
        liveness_prerequisites: Vec<&'a str>,
    }
    Ok(toml::Value::try_from(Pointer {
        status: if resumable { "reviewed-resume" } else { "checkpointed" },
        markdown: path.with_extension("md"),
        handoff: path,
        sequence: handoff.sequence,
        claim: &handoff.claim,
        source_commit: &handoff.source_commit,
        next: &handoff.next,
        liveness_prerequisites: vec![
            "Inspect the stored real worker session and surviving children before launching.",
            "Recheck every stored process by boot id, start ticks, argv and cwd; ambiguous or surviving writers retain ownership.",
            "Resume returns argv only; agent launch and execution belong to the caller.",
        ],
    })?)
}

fn current_handoff_authority(queue: &assignment::Request, handoff: &Handoff) -> Result<()> {
    let current = credential(queue, false)?;
    ensure!(
        handoff.session_id == current.session_id && handoff.authority_generation == current.generation,
        "handoff authority changed; checked adopt is required before resume"
    );
    Ok(())
}

fn adopt(root: &Path, queue: &assignment::Request, path: &Path, transfer: &Artifact) -> Result<toml::Value> {
    let _lock = worker_lock(root)?;
    let current = credential(queue, false)?;
    let saved = read_handoff(root, path)?;
    validate_handoff(root, queue, &saved, false, true)?;
    ensure!(dirty(&saved.worktree)?.is_empty(), "adopt requires checkpointed source");
    checked(transfer)?;
    let authority = assignment::authority::operate(assignment::authority::Request {
        checkout: root.to_path_buf(),
        ..Default::default()
    })?;
    let role = authority.roles.iter().find(|role| role.token == queue.coordinator).context("current role missing")?;
    ensure!(
        role.session_id == current.session_id
            && role.generation == current.generation
            && role.last_transfer.as_ref().is_some_and(|record| record.observation == *transfer),
        "adopt requires the current role's checked transfer"
    );
    let proof: assignment::authority::TransferObservation = toml::from_str(&fs::read_to_string(&transfer.path)?)?;
    for evidence in &proof.evidence {
        checked(evidence)?;
    }
    checked(&proof.replacement)?;
    let replacement: assignment::authority::SessionObservation =
        toml::from_str(&fs::read_to_string(&proof.replacement.path)?)?;
    ensure!(replacement.session_id == current.session_id, "transfer replacement differs from current session");
    for evidence in &replacement.evidence {
        checked(evidence)?;
    }
    ensure!(
        proof.token == queue.coordinator
            && proof.session_id == saved.session_id
            && proof.generation == saved.authority_generation
            && current.generation == proof.generation + 1,
        "transfer does not bridge this exact handoff authority"
    );
    let child = proof
        .children
        .iter()
        .find(|child| {
            child.effort == queue.effort
                && child.claim == saved.claim
                && child.agent.as_deref() == Some(saved.worker_session_id.as_str())
        })
        .context("transfer omits this exact surviving child")?;
    ensure!(["running", "active"].contains(&child.status.as_str()), "adopt requires an observed surviving child");
    checked(&child.observation)?;
    let observation: toml::Value = toml::from_str(&fs::read_to_string(&child.observation.path)?)?;
    let inspected: Vec<Artifact> =
        observation.get("evidence").cloned().context("child inspection evidence missing")?.try_into()?;
    for evidence in inspected {
        checked(&evidence)?;
    }
    let mut adopted = saved.clone();
    adopted.sequence += 1;
    adopted.session_id = current.session_id.clone();
    adopted.authority_generation = current.generation;
    adopted.dependencies.extend([artifact(path.to_path_buf())?, transfer.clone(), child.observation.clone()]);
    let next = handoff_path(root, &adopted)?;
    ensure!(next != path, "adopt requires a replacement real session");
    if next.exists() {
        ensure!(read_handoff(root, &next)? == adopted, "conflicting adopted handoff retry");
    } else {
        atomic(&next, toml::to_string_pretty(&adopted)?.as_bytes())?;
    }
    render(&adopted, &next)?;
    pointer(next, &adopted, true)
}

pub(crate) fn operate(request: Request) -> Result<toml::Value> {
    let operation = request.operation();
    match request {
        Request::InfrastructurePlan { mut queue, admission }
        | Request::InfrastructureNext { mut queue, admission }
        | Request::InfrastructureRecord { mut queue, admission } => {
            ensure!(
                queue.operation == operation.strip_prefix("infrastructure-").unwrap(),
                "queue operation must match infrastructure route"
            );
            let root = assignment::main_checkout(&queue.checkout)?;
            let lock = if queue.operation == "plan" { None } else { Some(worker_lock(&root)?) };
            match admission {
                Admission::Legacy { original_request } => legacy(&queue, &original_request)?,
                Admission::Infrastructure { scopes } => {
                    ensure!(
                        !scopes.is_empty() && queue.jobs.is_empty(),
                        "infrastructure jobs derive only from sealed reviewed scopes"
                    );
                    let mut jobs = Vec::new();
                    for seal in &scopes {
                        let scope = read_scope(seal)?;
                        git(&root, &["cat-file", "-e", &format!("{}^{{commit}}", scope.baseline)])?;
                        jobs.push(scope_job(seal, &scope));
                    }
                    if queue.operation == "record" {
                        let receipt = status(&queue)?;
                        let update = queue.update.as_ref().context("record requires an exact typed update")?;
                        queue.update = Some(admit_result(&root, &queue, &receipt, &scopes, update)?);
                    } else {
                        queue.jobs = jobs;
                    }
                }
            }
            drop(lock); // Assignment revalidates the generation under this same lock.
            compact(&assignment::operate(queue)?)
        }
        Request::Inspect { queue, handoff } => {
            let root = assignment::main_checkout(&queue.checkout)?;
            if let Some(path) = handoff {
                let saved = read_handoff(&root, &path)?;
                validate_handoff(&root, &queue, &saved, true, true)?;
                pointer(path, &saved, false)
            } else {
                compact(&status(&queue)?)
            }
        }
        Request::Start { mut queue, handoff } => {
            let root = assignment::main_checkout(&queue.checkout)?;
            let credential = credential(&queue, false)?;
            ensure!(
                handoff.session_id == credential.session_id && handoff.authority_generation == credential.generation,
                "handoff names another session authority"
            );
            validate_handoff(&root, &queue, &handoff, true, true)?;
            // Persist the launched child first. If publication stops, the exact
            // typed caller request retries against this durable running owner.
            queue.operation = "record".into();
            queue.jobs.clear();
            queue.update =
                Some(Update::Started { claim: handoff.claim.clone(), agent: handoff.worker_session_id.clone() });
            assignment::operate(queue.clone())?;
            let _lock = worker_lock(&root)?;
            current_handoff_authority(&queue, &handoff)?;
            validate_handoff(&root, &queue, &handoff, true, true)?;
            let path = publish(&root, &handoff)?;
            pointer(path, &handoff, false)
        }
        Request::Checkpoint { queue, handoff, message } => {
            let root = assignment::main_checkout(&queue.checkout)?;
            let path = checkpoint(&root, &queue, *handoff, message)?;
            pointer(path.clone(), &read_handoff(&root, &path)?, false)
        }
        Request::Resume { queue, handoff } => {
            let root = assignment::main_checkout(&queue.checkout)?;
            let _lock = worker_lock(&root)?;
            let saved = read_handoff(&root, &handoff)?;
            current_handoff_authority(&queue, &saved)?;
            validate_handoff(&root, &queue, &saved, false, true)?;
            ensure!(dirty(&saved.worktree)?.is_empty(), "resume requires checkpointed source");
            pointer(handoff, &saved, true)
        }
        Request::Adopt { queue, handoff, transfer } => {
            let root = assignment::main_checkout(&queue.checkout)?;
            adopt(&root, &queue, &handoff, &transfer)
        }
    }
}
