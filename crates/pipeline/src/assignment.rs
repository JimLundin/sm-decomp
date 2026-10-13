//! Durable decomp work ownership. Agent launch and evidence admission stay with callers.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

#[path = "authority.rs"]
pub(crate) mod authority;
pub(crate) use authority::SessionAuthority;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Kind {
    Verify,
    Diagnose,
    Research,
    Implement,
    Integrate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Job {
    pub id: String,
    pub revision: String,
    pub kind: Kind,
    pub subject: String,
    #[serde(default)]
    pub writable_paths: Vec<PathBuf>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, String>,
    pub instructions: String,
    pub ticket: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Claim {
    pub id: String,
    pub revision: String,
    pub attempt: u64,
    pub coordinator: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Artifact {
    pub path: PathBuf,
    pub sha256: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "result", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum WorkResult {
    Verified { evidence: Vec<Artifact>, authentication: Artifact },
    Diagnosed { evidence: Vec<Artifact>, followups: Vec<Job> },
    Researched { evidence: Vec<Artifact>, followups: Vec<Job> },
    Submitted { commit: String, evidence: Vec<Artifact> },
    Integrated { submissions: Vec<Claim>, evidence: Vec<Artifact>, authentication: Artifact, cleanup: Artifact },
    Failed { error: String, evidence: Vec<Artifact>, followups: Vec<Job> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum Update {
    Started { claim: Claim, agent: String },
    LaunchFailed { claim: Claim, reason: String },
    Finished { claim: Claim, result: WorkResult },
    OwnerStopped { claim: Claim, observation: Artifact },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Request {
    pub operation: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub coordinator: String,
    pub authority: Option<SessionAuthority>,
    pub capacity: usize,
    pub available_slots: usize,
    pub jobs: Vec<Job>,
    pub update: Option<Update>,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            operation: "status".into(),
            checkout: PathBuf::new(),
            effort: "affected-family-assignment".into(),
            coordinator: String::new(),
            authority: None,
            capacity: 3,
            available_slots: 0,
            jobs: Vec::new(),
            update: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum State {
    Ready,
    NotRequired,
    Reserved,
    Running,
    Submitted,
    Complete,
    Failed,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Entry {
    pub job: Job,
    pub state: State,
    pub claim: Option<Claim>,
    pub agent: Option<String>,
    pub pending_job: Option<Job>,
    #[serde(default)]
    pub stale: bool,
    pub result: Option<WorkResult>,
    #[serde(default)]
    pub attempts: u64,
    #[serde(default)]
    pub observations: Vec<Artifact>,
    #[serde(default)]
    pub last_update: Option<Update>,
    #[serde(default)]
    pub history: Vec<AttemptRecord>,
    #[serde(default)]
    pub adopted_ticket: bool,
    #[serde(default)]
    projection_pending: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct AttemptRecord {
    pub job: Job,
    pub claim: Option<Claim>,
    pub agent: Option<String>,
    pub result: Option<WorkResult>,
    pub observations: Vec<Artifact>,
}
fn retain_attempt(entry: &mut Entry) {
    entry.history.push(AttemptRecord {
        job: entry.job.clone(),
        claim: entry.claim.clone(),
        agent: entry.agent.clone(),
        result: entry.result.clone(),
        observations: entry.observations.clone(),
    });
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Workspace {
    pub branch: Option<String>,
    pub worktree: Option<PathBuf>,
    pub batch: Option<PathBuf>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Assignment {
    pub job: Job,
    pub claim: Claim,
    pub ticket: PathBuf,
    pub workspace: Workspace,
    pub experiment_seconds: Option<u64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Receipt {
    pub status: String,
    pub ledger: PathBuf,
    pub jobs: Vec<Entry>,
    pub assignment: Option<Assignment>,
    pub recovery: Vec<Claim>,
    pub deferred: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Ledger {
    schema: String,
    capacity: usize,
    integration_owner: Option<String>,
    jobs: BTreeMap<String, Entry>,
}

fn safe_name(name: &str) -> bool {
    !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}
fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}
fn validate_job(job: &Job) -> Result<()> {
    ensure!(safe_name(&job.id) && !job.revision.is_empty(), "invalid stable job or revision identity");
    ensure!(!job.subject.is_empty() && !job.instructions.is_empty(), "job subject and instructions are required");
    ensure!(job.writable_paths.iter().all(|p| relative(p)), "writable ownership must use safe relative paths");
    if let Some(ticket) = &job.ticket {
        ensure!(relative(ticket) && ticket.starts_with(".scratch"), "ticket must be a relative .scratch path");
    }
    ensure!(
        !matches!(job.kind, Kind::Implement) || !job.writable_paths.is_empty(),
        "implementation requires explicit writable ownership"
    );
    ensure!(
        !matches!(job.kind, Kind::Research | Kind::Diagnose | Kind::Verify) || job.writable_paths.is_empty(),
        "research, diagnosis and verification cannot own source writes"
    );
    Ok(())
}

fn validate_resources(root: &Path, checkout: &Path, job: &Job) -> Result<()> {
    for resource in job.writable_paths.iter().chain(job.ticket.iter()) {
        for checkout in [root, checkout] {
            let mut path = checkout.to_path_buf();
            for component in resource.components() {
                path.push(component);
                ensure!(!path.is_symlink(), "writable/ticket resource crosses a symlink: {}", path.display());
            }
        }
    }
    Ok(())
}

pub(crate) fn main_checkout(checkout: &Path) -> Result<PathBuf> {
    let checkout = fs::canonicalize(checkout)?;
    let output = Command::new("git").args(["rev-parse", "--git-common-dir"]).current_dir(&checkout).output()?;
    ensure!(output.status.success(), "cannot discover main checkout: {}", String::from_utf8_lossy(&output.stderr));
    let common_text = std::str::from_utf8(&output.stdout)?.trim();
    let common_path = Path::new(common_text);
    let common = fs::canonicalize(if common_path.is_absolute() {
        common_path.to_path_buf()
    } else {
        checkout.join(common_path)
    })
    .with_context(|| format!("git common directory missing: {common_text}"))?;
    ensure!(common.file_name().is_some_and(|n| n == ".git"), "assignment ledger requires a main working checkout");
    Ok(common.parent().context("main checkout parent missing")?.to_path_buf())
}

static PUBLICATION: AtomicU64 = AtomicU64::new(0);
fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().context("publication parent missing")?;
    fs::create_dir_all(parent)?;
    let sequence = PUBLICATION.fetch_add(1, Ordering::Relaxed);
    let pending = parent.join(format!(".assignment-{}-{sequence}.pending", std::process::id()));
    let result = (|| -> Result<()> {
        let mut file = OpenOptions::new().create_new(true).write(true).open(&pending)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&pending, path)?;
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(pending);
    }
    result
}
fn save(path: &Path, ledger: &Ledger) -> Result<()> {
    atomic_write(path, toml::to_string_pretty(ledger)?.as_bytes())
}
fn read_ledger(path: &Path, capacity: usize) -> Result<Ledger> {
    ensure!(!path.is_symlink(), "assignment ledger cannot be a symlink");
    if !path.exists() {
        return Ok(Ledger {
            schema: "decomp-assignment-v1".into(),
            capacity,
            integration_owner: None,
            jobs: BTreeMap::new(),
        });
    }
    let ledger: Ledger = toml::from_str(&fs::read_to_string(path)?)?;
    ensure!(ledger.schema == "decomp-assignment-v1", "unsupported assignment ledger schema");
    ensure!(ledger.capacity == capacity, "assignment capacity differs from authoritative ledger");
    Ok(ledger)
}

pub(crate) fn mutation_lock(root: &Path) -> Result<File> {
    let scratch = root.join(".scratch");
    ensure!(!scratch.is_symlink(), "assignment root cannot be a symlink");
    fs::create_dir_all(&scratch)?;
    let path = scratch.join("assignment.lock");
    ensure!(!path.is_symlink(), "assignment lock cannot be a symlink");
    let held = OpenOptions::new().create(true).append(true).open(path)?;
    held.lock()?;
    Ok(held)
}

fn read_ledgers(root: &Path) -> Result<BTreeMap<String, Ledger>> {
    let scratch = root.join(".scratch");
    let mut ledgers = BTreeMap::new();
    if !scratch.exists() {
        return Ok(ledgers);
    }
    ensure!(!scratch.is_symlink(), "assignment root cannot be a symlink");
    for entry in fs::read_dir(scratch)? {
        let entry = entry?;
        let path = entry.path().join("ledger.toml");
        if !path.exists() && !path.is_symlink() {
            continue;
        }
        ensure!(!entry.path().is_symlink() && !path.is_symlink(), "assignment effort/ledger cannot be a symlink");
        let name = entry.file_name().into_string().map_err(|_| anyhow::anyhow!("assignment effort is not UTF-8"))?;
        ensure!(safe_name(&name), "invalid ledger effort identity");
        let text = fs::read_to_string(&path)?;
        let value: toml::Value =
            toml::from_str(&text).with_context(|| format!("cannot inspect ledger {}", path.display()))?;
        if value.get("schema").and_then(toml::Value::as_str) != Some("decomp-assignment-v1") {
            continue;
        }
        let ledger: Ledger = toml::from_str(&text)?;
        for (id, entry) in &ledger.jobs {
            ensure!(id == &entry.job.id, "ledger job key differs from identity");
            validate_job(&entry.job)?;
            validate_resources(root, root, &entry.job)?;
            if owns_paths(&entry.state) {
                ensure!(
                    entry.claim.as_ref().is_some_and(|claim| claim.id == *id && claim.revision == entry.job.revision),
                    "held ledger job lacks exact original claim"
                );
            }
        }
        ledgers.insert(name, ledger);
    }
    Ok(ledgers)
}
fn field<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    text.lines().find_map(|line| line.strip_prefix(name).map(str::trim))
}
fn ticket_text(root: &Path, path: &Path) -> Result<Option<String>> {
    ensure!(relative(path) && path.starts_with(".scratch"), "unsafe ticket reference");
    let path = root.join(path);
    ensure!(!path.is_symlink(), "ticket cannot be a symlink");
    if path.exists() { Ok(Some(fs::read_to_string(path)?)) } else { Ok(None) }
}
fn ticket_ready(root: &Path, job: &Job) -> Result<Option<String>> {
    let Some(path) = &job.ticket else { return Ok(None) };
    let Some(text) = ticket_text(root, path)? else { return Ok(Some("referenced ticket is missing".into())) };
    match field(&text, "Status:") {
        Some("open") => {}
        Some(state) => return Ok(Some(format!("ticket is {state}"))),
        None => return Ok(Some("ticket status is missing".into())),
    }
    if let Some(triage) = field(&text, "Triage:")
        && triage != "ready-for-agent"
    {
        return Ok(Some(format!("ticket triage is {triage}")));
    }
    let blockers = field(&text, "Blocked by:").unwrap_or("missing");
    if blockers == "None" {
        return Ok(None);
    }
    let directory = root.join(path.parent().context("ticket directory missing")?);
    for blocker in blockers.split(',').map(str::trim) {
        let number = blocker.parse::<u64>().ok();
        let mut found = false;
        for entry in fs::read_dir(&directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let n = name.to_string_lossy().split('-').next().and_then(|n| n.parse::<u64>().ok());
            if number.is_some() && n == number {
                ensure!(!entry.path().is_symlink(), "blocker ticket cannot be a symlink");
                found = matches!(field(&fs::read_to_string(entry.path())?, "Status:"), Some("resolved" | "done"));
                break;
            }
        }
        if !found {
            return Ok(Some(format!("ticket blocked by {blocker}")));
        }
    }
    Ok(None)
}

fn active(state: &State) -> bool {
    matches!(state, State::Reserved | State::Running)
}
fn owns_paths(state: &State) -> bool {
    active(state) || *state == State::Submitted
}
fn same_identity(a: &Job, b: &Job) -> bool {
    a == b
}
fn refresh(ledger: &mut Ledger, jobs: &[Job]) -> Result<()> {
    let mut ids = BTreeSet::new();
    for job in jobs {
        validate_job(job)?;
        ensure!(ids.insert(&job.id), "duplicate seed job: {}", job.id);
        match ledger.jobs.get_mut(&job.id) {
            Some(entry) if entry.job.revision == job.revision => {
                ensure!(same_identity(&entry.job, job), "job scope changed without a new input revision: {}", job.id);
            }
            Some(entry) if owns_paths(&entry.state) => {
                entry.pending_job = Some(job.clone());
            }
            Some(entry) => {
                retain_attempt(entry);
                entry.job = job.clone();
                entry.state = State::Ready;
                entry.claim = None;
                entry.agent = None;
                entry.pending_job = None;
                entry.stale = false;
                entry.result = None;
                // Changed verification inputs must not reopen an externally
                // resolved/claimed issue. Only lifecycle updates project it.
                entry.projection_pending = entry.job.ticket.is_none();
            }
            None => {
                ledger.jobs.insert(
                    job.id.clone(),
                    Entry {
                        job: job.clone(),
                        state: State::Ready,
                        claim: None,
                        agent: None,
                        pending_job: None,
                        stale: false,
                        result: None,
                        attempts: 0,
                        observations: Vec::new(),
                        last_update: None,
                        history: Vec::new(),
                        adopted_ticket: false,
                        projection_pending: true,
                    },
                );
            }
        }
    }
    Ok(())
}

fn inputs_current(checkout: &Path, job: &Job) -> Result<bool> {
    if let Some(expected) = job.inputs.get("source-fingerprint")
        && source_fingerprint(checkout)? != *expected
    {
        return Ok(false);
    }
    for (key, expected) in &job.inputs {
        // Providers mark file inputs explicitly; legacy plain paths are also accepted.
        let name = key.strip_prefix("file:").unwrap_or(key);
        let path = Path::new(name);
        if !(expected.len() == 64 && expected.bytes().all(|b| b.is_ascii_hexdigit())) {
            continue;
        }
        if !relative(path) && !path.is_absolute() {
            continue;
        }
        if !key.starts_with("file:") && !name.contains('/') {
            continue;
        }
        let path = if path.is_absolute() { path.to_path_buf() } else { checkout.join(path) };
        let owned = path
            .strip_prefix(checkout)
            .ok()
            .is_some_and(|relative| job.writable_paths.iter().any(|owned| relative.starts_with(owned)));
        // Source submissions are checked against their commit by the coordinating
        // adapter. Expected owned edits do not invalidate immutable planning inputs.
        if matches!(job.kind, Kind::Implement | Kind::Integrate) && owned {
            continue;
        }
        if !path.is_file() || path.is_symlink() || crate::digest(&path)? != *expected {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Namespace and source contents used by conservative verification planning.
/// Kept here so saved jobs recheck exactly the same facts before reservation.
pub(crate) fn source_fingerprint(checkout: &Path) -> Result<String> {
    fn collect(root: &Path, path: &Path, files: &mut BTreeMap<String, String>) -> Result<()> {
        ensure!(!path.is_symlink(), "source snapshot crosses a symlink: {}", path.display());
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                collect(root, &entry?.path(), files)?;
            }
        } else if path.is_file() {
            files.insert(path.strip_prefix(root)?.display().to_string(), crate::digest(path)?);
        }
        Ok(())
    }
    let mut files = BTreeMap::new();
    for path in [
        "crates/pipeline/src",
        "crates/pipeline-model/src",
        "crates/port/src",
        "crates/port/tests",
        "tools",
        ".cargo",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "crates/port/Cargo.toml",
    ] {
        let path = checkout.join(path);
        if path.exists() {
            collect(checkout, &path, &mut files)?;
        }
    }
    Ok(format!("{:x}", Sha256::digest(toml::to_string(&files)?.as_bytes())))
}
fn retire_empty_verification(entry: &mut Entry) {
    if entry.state == State::Ready
        && entry.job.kind == Kind::Verify
        && entry.job.inputs.get("families").is_some_and(|families| families.trim().is_empty())
    {
        entry.state = State::NotRequired;
    }
}
fn blocker_reason(root: &Path, ledger: &Ledger, entry: &Entry) -> Result<Option<String>> {
    if entry.stale || entry.pending_job.is_some() {
        return Ok(Some("input revision changed; refresh and reconcile existing claim".into()));
    }
    let parent_transfer = entry.adopted_ticket
        && entry.job.ticket.as_ref().is_some_and(|ticket| {
            ledger.jobs.values().any(|parent| {
                parent.job.ticket.as_ref() == Some(ticket)
                    && parent.job.kind == Kind::Research
                    && parent.state == State::Complete
                    && parent
                        .result
                        .as_ref()
                        .is_some_and(|result| follows(result).iter().any(|job| job.id == entry.job.id))
            })
        });
    let transfer_ready = parent_transfer
        && entry
            .job
            .ticket
            .as_ref()
            .map(|ticket| ticket_text(root, ticket))
            .transpose()?
            .flatten()
            .is_some_and(|text| field(&text, "Status:") == Some("claimed") && managed_ticket(&text));
    if !transfer_ready && let Some(reason) = ticket_ready(root, &entry.job)? {
        // A queue-owned claim is reconciled from the ledger, never ticket copies.
        return Ok(Some(reason));
    }
    for blocker in &entry.job.blockers {
        if let Some(ticket) = blocker.strip_prefix("ticket:") {
            let text = ticket_text(root, Path::new(ticket))?;
            if !text.as_ref().is_some_and(|text| matches!(field(text, "Status:"), Some("resolved" | "done"))) {
                return Ok(Some(format!("blocked by {blocker}")));
            }
        } else if !ledger
            .jobs
            .get(blocker)
            .is_some_and(|e| e.state == State::Complete && !e.stale && e.pending_job.is_none())
        {
            return Ok(Some(format!("blocked by {blocker}")));
        }
    }
    for other in ledger.jobs.values().filter(|other| other.job.id != entry.job.id && owns_paths(&other.state)) {
        if ticket_path("", &entry.job) == ticket_path("", &other.job) && entry.job.ticket.is_some() {
            return Ok(Some(format!("ticket ownership held by {}", other.job.id)));
        }
    }
    if matches!(entry.job.kind, Kind::Implement | Kind::Integrate) {
        for other in ledger.jobs.values() {
            if other.job.id != entry.job.id
                && owns_paths(&other.state)
                && !assigned_submission(&entry.job, other)
                && entry
                    .job
                    .writable_paths
                    .iter()
                    .any(|a| other.job.writable_paths.iter().any(|b| a.starts_with(b) || b.starts_with(a)))
            {
                return Ok(Some(format!("writable ownership held by {}", other.job.id)));
            }
        }
    }
    if entry.job.kind == Kind::Integrate {
        if ledger.jobs.values().any(|e| active(&e.state) && e.job.kind == Kind::Integrate) {
            return Ok(Some("integration owner is already running".into()));
        }
        if let Some(submission) = entry.job.inputs.get("submission-id") {
            let valid = ledger.jobs.get(submission).is_some_and(|e| {
                e.state == State::Submitted
                    && !e.stale
                    && e.pending_job.is_none()
                    && e.claim.as_ref().is_some_and(|claim| {
                        entry.job.inputs.get("submission-revision") == Some(&claim.revision)
                            && entry.job.inputs.get("submission-attempt") == Some(&claim.attempt.to_string())
                    })
            });
            if !valid {
                return Ok(Some("integration submission is stale or no longer submitted".into()));
            }
        }
    }
    Ok(None)
}

fn assigned_submission(job: &Job, held: &Entry) -> bool {
    job.kind == Kind::Integrate
        && held.state == State::Submitted
        && held.claim.as_ref().is_some_and(|claim| {
            job.inputs.get("submission-id") == Some(&claim.id)
                && job.inputs.get("submission-revision") == Some(&claim.revision)
                && job.inputs.get("submission-attempt") == Some(&claim.attempt.to_string())
        })
}

fn repository_blocker(effort: &str, job: &Job, others: &BTreeMap<String, Ledger>) -> Option<String> {
    let ticket = ticket_path(effort, job);
    for (other_effort, ledger) in others {
        if other_effort == effort {
            continue;
        }
        for other in ledger.jobs.values().filter(|other| owns_paths(&other.state)) {
            if ticket == ticket_path(other_effort, &other.job) {
                return Some(format!("ticket ownership held by {other_effort}/{}", other.job.id));
            }
            if job
                .writable_paths
                .iter()
                .any(|path| other.job.writable_paths.iter().any(|held| authority::overlaps(path, held)))
            {
                return Some(format!("writable ownership held by {other_effort}/{}", other.job.id));
            }
            if job.kind == Kind::Integrate && other.job.kind == Kind::Integrate && active(&other.state) {
                return Some(format!("integration already running in {other_effort}/{}", other.job.id));
            }
        }
    }
    None
}

fn ticket_path(effort: &str, job: &Job) -> PathBuf {
    job.ticket.clone().unwrap_or_else(|| PathBuf::from(format!(".scratch/{effort}/assignments/{}.md", job.id)))
}
fn marker(job: &Job) -> String {
    format!("<!-- decomp-assignment:{} -->", job.id)
}
fn managed_ticket(ticket: &str) -> bool {
    ticket.contains("<!-- decomp-assignment:")
}
fn replace_status(text: &str, status: &str) -> String {
    text.lines()
        .map(|line| if line.starts_with("Status:") { format!("Status: {status}") } else { line.to_owned() })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
fn published_status(entry: &Entry) -> &'static str {
    if entry.state == State::Complete
        && entry.job.kind == Kind::Research
        && entry.job.ticket.is_some()
        && entry.job.inputs.get("origin-ticket-type").is_some_and(|kind| kind == "task")
    {
        return if entry
            .result
            .as_ref()
            .is_some_and(|result| follows(result).iter().any(|job| job.kind == Kind::Implement))
        {
            "claimed"
        } else {
            "open"
        };
    }
    match entry.state {
        State::Ready | State::Failed | State::NotRequired => "open",
        State::Reserved | State::Running | State::Submitted => "claimed",
        State::Complete => "resolved",
    }
}
fn publication_details(entry: &Entry) -> Result<String> {
    let ownership = entry.job.writable_paths.iter().map(|path| format!("- `{}`\n", path.display())).collect::<String>();
    let state = match entry.state {
        State::Ready => "ready",
        State::NotRequired => "not-required",
        State::Reserved => "reserved",
        State::Running => "running",
        State::Submitted => "submitted",
        State::Complete => "complete",
        State::Failed => "failed",
    };
    let mut text = format!(
        "<!-- decomp-assignment-record:start -->\n\n## Current assignment\n\nStable work: `{}`\nInput revision: `{}`\nLifecycle: `{}`\n\n{}\n\nWritable ownership:\n\n{}\n",
        entry.job.id,
        entry.job.revision,
        state.trim(),
        entry.job.instructions,
        if ownership.is_empty() { "No maintained source writes.\n" } else { &ownership }
    );
    if let Some(claim) = &entry.claim {
        text.push_str(&format!("Claim owner: `{}`; attempt: `{}`.\n", claim.coordinator, claim.attempt));
    }
    if let Some(result) = &entry.result {
        text.push_str("\n## Completion evidence for this assignment\n\n");
        text.push_str(&format!("```toml\n{}\n```\n", toml::to_string_pretty(result)?));
    }
    text.push_str("\n<!-- decomp-assignment-record:end -->\n");
    Ok(text)
}
fn with_details(text: &str, details: &str) -> String {
    let start = "<!-- decomp-assignment-record:start -->";
    let end = "<!-- decomp-assignment-record:end -->";
    if let Some(begin) = text.find(start)
        && let Some(finish) = text[begin..].find(end)
    {
        let finish = begin + finish + end.len();
        return format!("{}{}{}", &text[..begin], details, &text[finish..]);
    }
    format!("{text}\n{details}")
}
fn publish_ticket(root: &Path, effort: &str, entry: &Entry) -> Result<()> {
    if !entry.projection_pending {
        return Ok(());
    }
    let relative = ticket_path(effort, &entry.job);
    let path = root.join(&relative);
    let mark = marker(&entry.job);
    let status = published_status(entry);
    let details = publication_details(entry)?;
    let existing = ticket_text(root, &relative)?;
    if let Some(existing) = existing {
        if !existing.contains(&mark) {
            if entry.claim.is_none() {
                return Ok(());
            }
            ensure!(
                field(&existing, "Status:") == Some("open")
                    || (entry.adopted_ticket
                        && field(&existing, "Status:") == Some("claimed")
                        && managed_ticket(&existing)),
                "cannot claim preexisting non-open ticket"
            );
            let updated = with_details(&format!("{}\n{mark}\n", replace_status(&existing, status)), &details);
            return atomic_write(&path, updated.as_bytes());
        }
        // Preserve all human prose; only the authoritative lifecycle is projected.
        return atomic_write(&path, with_details(&replace_status(&existing, status), &details).as_bytes());
    }
    ensure!(entry.job.ticket.is_none(), "referenced ticket disappeared");
    let text = format!(
        "# {}\n\nStatus: {status}\nBlocked by: None\nTriage: ready-for-agent\nType: task\n\n{mark}\n\nStable work: `{}`\nInput revision: `{}`\n\n## Assignment\n\n{}\n\n## Completion evidence\n\nLifecycle and sealed result pointers are authoritative in `../ledger.toml`.\n",
        entry.job.subject, entry.job.id, entry.job.revision, entry.job.instructions
    );
    atomic_write(&path, with_details(&text, &details).as_bytes())
}

fn check_artifact(artifact: &Artifact) -> Result<()> {
    ensure!(
        artifact.path.is_absolute() && !artifact.path.is_symlink(),
        "result artifact must be an absolute regular file"
    );
    ensure!(
        artifact.path.is_file() && crate::digest(&artifact.path)? == artifact.sha256,
        "result artifact identity differs: {}",
        artifact.path.display()
    );
    Ok(())
}
pub(crate) fn result_artifacts(result: &WorkResult) -> Vec<&Artifact> {
    match result {
        WorkResult::Verified { evidence, authentication } => evidence.iter().chain([authentication]).collect(),
        WorkResult::Integrated { evidence, authentication, cleanup, .. } => {
            evidence.iter().chain([authentication, cleanup]).collect()
        }
        WorkResult::Diagnosed { evidence, .. }
        | WorkResult::Researched { evidence, .. }
        | WorkResult::Submitted { evidence, .. }
        | WorkResult::Failed { evidence, .. } => evidence.iter().collect(),
    }
}
fn follows(result: &WorkResult) -> &[Job] {
    match result {
        WorkResult::Diagnosed { followups, .. }
        | WorkResult::Researched { followups, .. }
        | WorkResult::Failed { followups, .. } => followups,
        _ => &[],
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StoppedObservation {
    agent: Option<String>,
    coordinator: String,
    status: String,
    claim: Claim,
    #[serde(default)]
    session_id: Option<String>,
    #[serde(default)]
    observed_at: Option<String>,
    #[serde(default)]
    evidence: Vec<Artifact>,
    #[serde(default)]
    processes: Vec<authority::ProcessObservation>,
}
fn checked_stop(claim: &Claim, entry: &Entry, observation: &Artifact, managed: bool) -> Result<()> {
    check_artifact(observation)?;
    let stopped: StoppedObservation = toml::from_str(&fs::read_to_string(&observation.path)?)?;
    ensure!(
        stopped.claim == *claim && stopped.coordinator == claim.coordinator,
        "stopped observation names another claim owner"
    );
    ensure!(stopped.agent == entry.agent, "stopped observation names another agent");
    ensure!(
        ["completed", "failed", "interrupted", "not-launched"].contains(&stopped.status.as_str()),
        "observation does not establish stopped owner"
    );
    ensure!(
        entry.agent.is_none() || stopped.status != "not-launched",
        "started agent cannot be recovered as not launched"
    );
    if managed {
        ensure!(
            stopped.session_id.as_deref().is_some_and(authority::real_session_id)
                && stopped.observed_at.as_ref().is_some_and(|value| !value.is_empty())
                && !stopped.evidence.is_empty(),
            "managed stopped observation requires real owner inspection evidence"
        );
        if entry.agent.as_deref().is_some_and(authority::real_session_id) {
            ensure!(
                stopped.session_id.as_ref() == entry.agent.as_ref(),
                "stopped observation names another real child session"
            );
        }
        for artifact in &stopped.evidence {
            check_artifact(artifact)?;
        }
        for process in &stopped.processes {
            ensure!(
                process.pid > 0
                    && !process.argv.is_empty()
                    && !process.started_at.is_empty()
                    && ["exited", "stopped", "completed", "failed", "not-running"].contains(&process.status.as_str()),
                "stopped child still has a running or ambiguous process"
            );
            check_artifact(&process.observation)?;
        }
    }
    Ok(())
}

fn record(
    root: &Path,
    checkout: &Path,
    ledger: &mut Ledger,
    coordinator: &str,
    update: &Update,
    managed: bool,
) -> Result<()> {
    let claim = match update {
        Update::Started { claim, .. }
        | Update::LaunchFailed { claim, .. }
        | Update::Finished { claim, .. }
        | Update::OwnerStopped { claim, .. } => claim,
    };
    ensure!(claim.coordinator == coordinator, "coordinator cannot update another owner's claim");
    let entry = ledger.jobs.get(&claim.id).context("unknown assignment claim")?;
    // A response may be lost after durable publication. Only the exact recorded
    // event is idempotent, and never once inputs or a later attempt changed.
    if entry.last_update.as_ref() == Some(update)
        && entry.job.revision == claim.revision
        && entry.attempts == claim.attempt
        && entry.pending_job.is_none()
        && !entry.stale
    {
        if let Update::Finished { result, .. } = update {
            ensure!(inputs_current(checkout, &entry.job)?, "stale source revision cannot retry recorded completion");
            for artifact in result_artifacts(result) {
                check_artifact(artifact)?;
            }
        }
        return Ok(());
    }
    ensure!(entry.claim.as_ref() == Some(claim), "stale assignment attempt or revision");
    if let Update::Finished { result, .. } = update
        && entry.result.as_ref() == Some(result)
        && entry.pending_job.is_none()
        && !entry.stale
        && matches!(entry.state, State::Complete | State::Submitted | State::Failed)
    {
        ensure!(inputs_current(checkout, &entry.job)?, "stale source revision cannot retry recorded completion");
        for artifact in result_artifacts(result) {
            check_artifact(artifact)?;
        }
        return Ok(());
    }
    match update {
        Update::Started { agent, .. } => {
            ensure!(!agent.is_empty(), "agent identity missing");
            if entry.state == State::Running && entry.agent.as_ref() == Some(agent) {
                return Ok(());
            }
            ensure!(entry.state == State::Reserved, "only a reserved assignment can start");
            ensure!(
                !managed || authority::real_session_id(agent),
                "managed start must record the actual child thread id"
            );
            let entry = ledger.jobs.get_mut(&claim.id).unwrap();
            entry.agent = Some(agent.clone());
            entry.state = State::Running;
        }
        Update::LaunchFailed { reason, .. } => {
            ensure!(
                entry.state == State::Reserved && entry.agent.is_none(),
                "launch failure cannot release a started agent"
            );
            ensure!(!reason.is_empty(), "launch failure diagnostic missing");
            let entry = ledger.jobs.get_mut(&claim.id).unwrap();
            retain_attempt(entry);
            entry.state = State::Ready;
            entry.claim = None;
            entry.result = None;
            if let Some(job) = entry.pending_job.take() {
                entry.job = job;
                entry.stale = false;
            }
        }
        Update::OwnerStopped { observation, .. } => {
            ensure!(
                active(&entry.state),
                "submitted ownership remains held through integration; claim is not awaiting recovery"
            );
            checked_stop(claim, entry, observation, managed)?;
            let entry = ledger.jobs.get_mut(&claim.id).unwrap();
            entry.observations.push(observation.clone());
            retain_attempt(entry);
            if let Some(job) = entry.pending_job.take() {
                entry.job = job;
                entry.stale = false;
            }
            entry.state = State::Ready;
            entry.claim = None;
            entry.agent = None;
            entry.result = None;
        }
        Update::Finished { result, .. } => {
            ensure!(entry.state == State::Running, "only a started assignment can finish");
            ensure!(
                entry.pending_job.is_none() && !entry.stale && inputs_current(checkout, &entry.job)?,
                "stale source revision cannot complete current work"
            );
            for artifact in result_artifacts(result) {
                check_artifact(artifact)?;
            }
            ensure!(!result_artifacts(result).is_empty(), "finished work requires sealed evidence");
            ensure!(
                follows(result).iter().all(|job| job.id != claim.id),
                "followup cannot redefine its finishing assignment"
            );
            let state = match (&entry.job.kind, result) {
                (Kind::Verify, WorkResult::Verified { .. })
                | (Kind::Diagnose, WorkResult::Diagnosed { .. })
                | (Kind::Research, WorkResult::Researched { .. }) => State::Complete,
                (Kind::Implement, WorkResult::Submitted { commit, .. }) => {
                    ensure!(
                        commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()),
                        "submission requires exact commit identity"
                    );
                    State::Submitted
                }
                (Kind::Integrate, WorkResult::Integrated { submissions, .. }) => {
                    ensure!(
                        ledger.integration_owner.as_deref() == Some(coordinator),
                        "only the integration owner can integrate submissions"
                    );
                    ensure!(!submissions.is_empty(), "integration must name submitted claims");
                    for submission in submissions {
                        let submitted = ledger.jobs.get(&submission.id).context("unknown integration submission")?;
                        ensure!(
                            submitted.claim.as_ref() == Some(submission)
                                && submitted.state == State::Submitted
                                && submitted.pending_job.is_none()
                                && !submitted.stale,
                            "integration submission is stale or not submitted"
                        );
                    }
                    ensure!(
                        submissions.len() == 1
                            && entry.job.inputs.get("submission-id") == Some(&submissions[0].id)
                            && entry.job.inputs.get("submission-attempt") == Some(&submissions[0].attempt.to_string())
                            && entry.job.inputs.get("submission-revision") == Some(&submissions[0].revision),
                        "integration result does not name its assigned submission"
                    );
                    State::Complete
                }
                (_, WorkResult::Failed { error, .. }) => {
                    ensure!(!error.is_empty(), "failure diagnostic missing");
                    State::Failed
                }
                _ => bail!("result does not match assignment work kind"),
            };
            let mut result = result.clone();
            let origin = (entry.job.kind == Kind::Research
                && entry.job.inputs.get("origin-ticket-type").is_some_and(|kind| kind == "task"))
            .then(|| entry.job.ticket.clone())
            .flatten();
            if let Some(ticket) = origin
                && let WorkResult::Researched { followups, .. } = &mut result
            {
                let implementations =
                    followups.iter_mut().filter(|job| job.kind == Kind::Implement).collect::<Vec<_>>();
                ensure!(implementations.len() <= 1, "task research must establish one coherent implementation scope");
                for implementation in implementations {
                    ensure!(
                        implementation.ticket.as_ref().is_none_or(|existing| existing == &ticket),
                        "promoted implementation names a different originating task"
                    );
                    implementation.ticket = Some(ticket.clone());
                    if !implementation.blockers.contains(&claim.id) {
                        implementation.blockers.push(claim.id.clone());
                    }
                }
            }
            refresh(ledger, follows(&result))?;
            for followup in follows(&result) {
                if followup.kind == Kind::Implement
                    && followup.ticket.is_some()
                    && followup.ticket == ledger.jobs[&claim.id].job.ticket
                {
                    let implementation = ledger.jobs.get_mut(&followup.id).unwrap();
                    implementation.adopted_ticket = true;
                    // The research owner projects the shared task until the
                    // implementation actually reserves its transferred ownership.
                    implementation.projection_pending = false;
                }
            }
            if let WorkResult::Integrated { submissions, .. } = &result {
                for submission in submissions {
                    let submitted = ledger.jobs.get_mut(&submission.id).unwrap();
                    submitted.state = State::Complete;
                    submitted.projection_pending = true;
                }
            }
            let entry = ledger.jobs.get_mut(&claim.id).unwrap();
            entry.state = state;
            entry.result = Some(result.clone());
            if matches!(result, WorkResult::Submitted { .. }) {
                let submission = entry.job.clone();
                let integration = Job {
                    id: format!("integrate-{}", submission.id),
                    revision: format!("{}-{}", claim.revision, claim.attempt),
                    kind: Kind::Integrate,
                    subject: format!("Integrate {}", submission.subject),
                    writable_paths: submission.writable_paths,
                    blockers: Vec::new(),
                    inputs: BTreeMap::from([
                        (
                            "base-head".into(),
                            submission.inputs.get("base-head").cloned().unwrap_or_else(|| {
                                let WorkResult::Submitted { commit, .. } = &result else { unreachable!() };
                                commit.clone()
                            }),
                        ),
                        ("submission-id".into(), claim.id.clone()),
                        ("submission-revision".into(), claim.revision.clone()),
                        ("submission-attempt".into(), claim.attempt.to_string()),
                    ]),
                    instructions: format!(
                        "Integrate submitted Translation scope {} from its isolated worktree. Freeze merged source, run required verification, authenticate evidence and finish scoped cleanup before recording integrated completion.",
                        submission.subject
                    ),
                    ticket: None,
                };
                ledger.integration_owner.get_or_insert_with(|| coordinator.to_owned());
                refresh(ledger, &[integration])?;
            }
        }
    }
    if let Some(entry) = ledger.jobs.get_mut(&claim.id) {
        entry.projection_pending = true;
        entry.last_update = Some(update.clone());
    }
    let _ = root;
    Ok(())
}

fn workspace(root: &Path, effort: &str, claim: &Claim, kind: &Kind) -> Workspace {
    let name = format!("{}-{}-{}", effort, claim.id, claim.attempt);
    match kind {
        Kind::Implement => Workspace {
            branch: Some(name.clone()),
            worktree: Some(std::env::temp_dir().join(format!("sm-{name}"))),
            batch: Some(PathBuf::from(format!("out/port/batches/{name}/batch.toml"))),
        },
        Kind::Verify | Kind::Integrate => {
            Workspace { batch: Some(root.join(format!("out/port/batches/{name}/batch.toml"))), ..Default::default() }
        }
        _ => Workspace::default(),
    }
}

fn priority(entry: &Entry) -> (u8, u64, &str) {
    let local_number =
        entry.job.ticket.as_ref().and_then(|path| path.file_name()?.to_str()?.split('-').next()?.parse::<u64>().ok());
    if let Some(number) = local_number {
        return (0, number, &entry.job.id);
    }
    let kind = match entry.job.kind {
        Kind::Integrate => 1,
        Kind::Implement | Kind::Diagnose => 2,
        Kind::Verify => 3,
        Kind::Research => 4,
    };
    let rank = entry.job.inputs.get("candidate-rank").and_then(|rank| rank.parse().ok()).unwrap_or(u64::MAX);
    (kind, rank, &entry.job.id)
}

/// Caller validates evidence admission and researched scopes before recording results.
/// `plan` and `status` are read-only. `next` reserves before exposing launchable work.
pub(crate) fn operate(request: Request) -> Result<Receipt> {
    ensure!(["plan", "next", "record", "status"].contains(&request.operation.as_str()), "unknown assignment operation");
    ensure!(
        safe_name(&request.effort) && (1..=64).contains(&request.capacity),
        "invalid effort or assignment capacity"
    );
    let readonly = matches!(request.operation.as_str(), "plan" | "status");
    ensure!(readonly || !request.coordinator.is_empty(), "coordinator token is required for mutation");
    ensure!(request.operation == "record" || request.update.is_none(), "updates require record operation");
    let root = main_checkout(&request.checkout)?;
    let path = root.join(format!(".scratch/{}/ledger.toml", request.effort));
    let parent = path.parent().unwrap();
    ensure!(!parent.is_symlink(), "assignment effort cannot be a symlink");
    let _lock = if readonly { None } else { Some(mutation_lock(&root)?) };
    let policy = if readonly {
        authority::read(&root)?
    } else {
        authority::validate_locked(&root, &request.effort, &request.coordinator, request.authority.as_ref(), false)?
    };
    if let Some(policy) = &policy {
        ensure!(
            request.capacity == policy.preparation.capacity,
            "assignment capacity differs from shared session capacity"
        );
    }
    let bridge = policy.as_ref().map(|policy| authority::bridge(policy, &root, false)).transpose()?;
    let other_ledgers = read_ledgers(&root)?;
    let mut ledger = read_ledger(&path, request.capacity)?;
    if let Some(policy) = &policy {
        ledger.integration_owner = Some(authority::integrator(policy).to_owned());
    }
    if !readonly && let Some((policy, bridge)) = policy.as_ref().zip(bridge.as_ref()) {
        authority::validate_mutation(policy, bridge, &request.effort, &request, &ledger)?;
    }
    for job in &request.jobs {
        validate_resources(&root, &request.checkout, job)?;
    }
    if request.operation != "status" {
        let mut seeds = Vec::new();
        for job in &request.jobs {
            // A projected claimed/resolved ticket may describe a running job's
            // old revision. Existing ownership remains exclusively ledger-owned.
            // Providers cannot accidentally reopen it by discovering a new id.
            let owned = ledger.jobs.contains_key(&job.id);
            let projected = job
                .ticket
                .as_ref()
                .map(|ticket| ticket_text(&root, ticket))
                .transpose()?
                .flatten()
                .is_some_and(|text| managed_ticket(&text) && field(&text, "Status:") != Some("open"));
            if owned || !projected {
                seeds.push(job.clone());
            }
        }
        refresh(&mut ledger, &seeds)?;
    }
    for entry in ledger.jobs.values_mut() {
        if !inputs_current(&request.checkout, &entry.job)? {
            entry.stale = true;
        }
        retire_empty_verification(entry);
    }
    if request.operation == "record" {
        let update = request.update.as_ref().context("record update missing")?;
        let claim = match update {
            Update::Started { claim, .. }
            | Update::LaunchFailed { claim, .. }
            | Update::Finished { claim, .. }
            | Update::OwnerStopped { claim, .. } => claim,
        };
        let integration = ledger.jobs.get(&claim.id).is_some_and(|entry| entry.job.kind == Kind::Integrate);
        if integration {
            authority::validate_locked(&root, &request.effort, &request.coordinator, request.authority.as_ref(), true)?;
        }
        record(&root, &request.checkout, &mut ledger, &request.coordinator, update, policy.is_some())?;
        for entry in ledger.jobs.values_mut() {
            retire_empty_verification(entry);
        }
    }
    let mut deferred = BTreeMap::new();
    let mut recovery = Vec::new();
    for entry in ledger.jobs.values() {
        if owns_paths(&entry.state)
            && (entry.state == State::Reserved || entry.pending_job.is_some() || entry.stale)
            && let Some(claim) = &entry.claim
        {
            recovery.push(claim.clone());
        }
        if entry.state == State::Ready {
            let reason = repository_blocker(&request.effort, &entry.job, &other_ledgers).or_else(|| {
                policy.as_ref().zip(bridge.as_ref()).and_then(|(policy, bridge)| {
                    authority::held_reason(policy, bridge, &request.effort, &entry.job, request.authority.as_ref())
                })
            });
            if let Some(reason) = reason.or(blocker_reason(&root, &ledger, entry)?) {
                deferred.insert(entry.job.id.clone(), reason);
            }
        }
    }
    let mut assignment = None;
    if request.operation == "next" && request.available_slots > 0 {
        let used = ledger.jobs.values().filter(|e| active(&e.state)).count()
            + if policy.is_some() {
                other_ledgers
                    .iter()
                    .filter(|(effort, _)| *effort != &request.effort)
                    .map(|(_, ledger)| ledger.jobs.values().filter(|entry| active(&entry.state)).count())
                    .sum::<usize>()
            } else {
                0
            };
        if used < policy.as_ref().map(|policy| policy.preparation.capacity).unwrap_or(ledger.capacity) {
            let chosen = ledger
                .jobs
                .values()
                .filter(|entry| entry.state == State::Ready && !deferred.contains_key(&entry.job.id))
                .filter(|entry| {
                    entry.job.kind != Kind::Integrate
                        || ledger.integration_owner.as_ref().is_none_or(|owner| owner == &request.coordinator)
                })
                .min_by_key(|entry| priority(entry))
                .map(|entry| entry.job.id.clone());
            if let Some(id) = chosen {
                let entry = ledger.jobs.get_mut(&id).unwrap();
                entry.attempts += 1;
                let claim = Claim {
                    id,
                    revision: entry.job.revision.clone(),
                    attempt: entry.attempts,
                    coordinator: request.coordinator.clone(),
                };
                entry.claim = Some(claim.clone());
                entry.agent = None;
                entry.state = State::Reserved;
                entry.projection_pending = true;
                if entry.job.kind == Kind::Integrate {
                    ledger.integration_owner = Some(request.coordinator.clone());
                }
                assignment = Some(Assignment {
                    job: entry.job.clone(),
                    ticket: root.join(ticket_path(&request.effort, &entry.job)),
                    workspace: workspace(&root, &request.effort, &claim, &entry.job.kind),
                    experiment_seconds: (entry.job.kind == Kind::Research).then_some(60),
                    claim,
                });
            }
        }
    }
    if !readonly {
        // Commit ownership first. An interrupted ticket projection is repaired on resume.
        save(&path, &ledger)?;
        for entry in ledger.jobs.values_mut() {
            publish_ticket(&root, &request.effort, entry)?;
            entry.projection_pending = false;
        }
        save(&path, &ledger)?;
    }
    let status = if assignment.is_some() {
        "ready"
    } else if !recovery.is_empty() {
        "recovery"
    } else if !ledger.jobs.is_empty()
        && ledger.jobs.values().all(|entry| matches!(entry.state, State::Complete | State::NotRequired) && !entry.stale)
    {
        "complete"
    } else if readonly {
        request.operation.as_str()
    } else {
        "waiting"
    };
    Ok(Receipt {
        status: status.into(),
        ledger: path,
        jobs: ledger.jobs.into_values().collect(),
        assignment,
        recovery,
        deferred,
    })
}
