//! Coordinator adapter: derive jobs and validate outcomes before durable claims change.
use crate::{
    affected,
    assignment::{self, Artifact, Claim, Job, Kind, Update, WorkResult, source_fingerprint},
    batch, cleanup, digest, family, family_model, translation,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Request {
    pub operation: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub coordinator: String,
    pub capacity: usize,
    pub available_slots: usize,
    pub profile: String,
    pub families: Option<Vec<String>>,
    pub changed_since: Option<String>,
    pub translations: bool,
    pub source_inventory: Option<PathBuf>,
    pub exercise: Option<PathBuf>,
    pub batch_result: Option<PathBuf>,
    pub scope: Option<PathBuf>,
    pub update: Option<Update>,
    pub pipeline_root: Option<PathBuf>,
    pub workers: usize,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            operation: "status".into(),
            checkout: PathBuf::new(),
            effort: "affected-family-assignment".into(),
            coordinator: String::new(),
            capacity: 3,
            available_slots: 0,
            profile: "offline-all".into(),
            families: None,
            changed_since: None,
            translations: true,
            source_inventory: None,
            exercise: None,
            batch_result: None,
            scope: None,
            update: None,
            pipeline_root: None,
            workers: 4,
        }
    }
}
fn hash(value: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(value.as_ref()))
}
fn git(checkout: &Path, args: &[&str]) -> Result<String> {
    let result = Command::new("git").args(args).current_dir(checkout).output()?;
    ensure!(result.status.success(), "git {} failed: {}", args.join(" "), String::from_utf8_lossy(&result.stderr));
    Ok(String::from_utf8(result.stdout)?.trim().into())
}
fn main_checkout(checkout: &Path) -> Result<PathBuf> {
    let common = fs::canonicalize(checkout.join(git(checkout, &["rev-parse", "--git-common-dir"])?))?;
    ensure!(common.file_name().is_some_and(|name| name == ".git"), "assignment requires a main checkout");
    Ok(common.parent().context("main checkout missing")?.into())
}
fn queue(request: &Request, operation: &str) -> assignment::Request {
    assignment::Request {
        operation: operation.into(),
        checkout: request.checkout.clone(),
        effort: request.effort.clone(),
        coordinator: request.coordinator.clone(),
        capacity: request.capacity,
        available_slots: request.available_slots,
        jobs: vec![],
        update: None,
    }
}
fn checked(artifact: &Artifact) -> Result<()> {
    ensure!(
        artifact.path.is_absolute()
            && artifact.path.is_file()
            && !artifact.path.is_symlink()
            && digest(&artifact.path)? == artifact.sha256,
        "result artifact changed: {}",
        artifact.path.display()
    );
    Ok(())
}
fn artifact(path: PathBuf) -> Result<Artifact> {
    Ok(Artifact { sha256: digest(&path)?, path })
}
fn evidence_mut(result: &mut WorkResult) -> &mut Vec<Artifact> {
    match result {
        WorkResult::Verified { evidence, .. }
        | WorkResult::Diagnosed { evidence, .. }
        | WorkResult::Researched { evidence, .. }
        | WorkResult::Submitted { evidence, .. }
        | WorkResult::Integrated { evidence, .. }
        | WorkResult::Failed { evidence, .. } => evidence,
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RecordedRequest {
    schema: String,
    update: Update,
}
fn attach_base(checkout: &Path, job: &mut Job) -> Result<()> {
    if job.kind == Kind::Implement {
        job.inputs.insert("base-head".into(), git(checkout, &["rev-parse", "HEAD"])?);
        job.revision = hash(toml::to_string(&job.inputs)?);
    }
    Ok(())
}
fn verification_job(report: &affected::Report) -> Result<Job> {
    let families = report.families();
    let mut inputs = report
        .planning_inputs
        .iter()
        .map(|(path, hash)| (format!("file:{}", path.display()), hash.clone()))
        .collect::<BTreeMap<_, _>>();
    inputs.insert("families".into(), families.join("\n"));
    inputs.insert("selection".into(), hash(toml::to_string(report)?));
    inputs.insert("source-fingerprint".into(), source_fingerprint(&report.checkout)?);
    let revision = hash(toml::to_string(&inputs)?);
    let mut frontier =
        report.selected.iter().chain(&report.excluded).map(|decision| decision.family.clone()).collect::<Vec<_>>();
    frontier.sort();
    let id = format!("verify-{}", &hash(frontier.join("\n"))[..16]);
    Ok(Job {
        id,
        revision,
        kind: Kind::Verify,
        subject: format!("Verify {} affected Offline families", families.len()),
        writable_paths: vec![],
        blockers: vec![],
        inputs,
        instructions: format!(
            "Run one `sm-pipeline batch verify --families {} --batch ASSIGNMENT_BATCH` using the shared SM_PIPELINE_ROOT/SM_PIPELINE_WORKERS. Keep batch jobs within shared worker capacity. Record the batch.toml artifact; current-source authentication is required before completion. Selection rationale is available through batch plan. Preserve failed verdicts for diagnosis and complete run cleanup.",
            families.join(" ")
        ),
        ticket: None,
    })
}
fn diagnosis_jobs(checkout: &Path, path: &Path) -> Result<Vec<Job>> {
    let result: batch::Batch = pipeline_model::read(path)?;
    ensure!(result.schema_version == 2, "unsupported diagnosis batch version");
    let mut jobs = vec![];
    for (name, record) in result.families {
        if record.exit_code == 0 {
            continue;
        }
        let mut inputs = BTreeMap::from([(format!("file:{}", fs::canonicalize(path)?.display()), digest(path)?)]);
        if let Some(verdict) = record.verdict {
            inputs.insert(format!("file:{}", verdict.display()), digest(&verdict)?);
        }
        inputs.insert("base-head".into(), git(checkout, &["rev-parse", "HEAD"])?);
        jobs.push(Job { id: format!("diagnose-{}", &hash(&name)[..16]), revision: hash(toml::to_string(&inputs)?), kind: Kind::Diagnose, subject: format!("Diagnose {name}: {}", record.status), writable_paths: vec![], blockers: vec![], inputs, instructions: format!("Read the retained {name} failure in {}. Establish the first failing fixture and a bounded reproduction using existing Rust commands; cap initial experiments at 60 seconds. Report evidence and a local follow-up scope. Preserve diagnostic artifacts and complete cleanup.", path.display()), ticket: None });
    }
    Ok(jobs)
}

// A batch record alone is mutable metadata. Authenticate every requested verdict
// and its current catalog manifest before creating a coordinator receipt.
fn authenticate_batches(checkout: &Path, evidence: &[Artifact], required: Option<&str>) -> Result<Vec<PathBuf>> {
    let catalog = batch::catalog(checkout)?;
    let required = required.map(|names| names.lines().collect::<Vec<_>>());
    let mut verified = BTreeMap::new();
    let mut roots = vec![];
    for item in evidence {
        checked(item)?;
        let Ok(result) = pipeline_model::read::<batch::Batch>(&item.path) else {
            continue;
        };
        ensure!(result.schema_version == 2 && result.publication_error.is_none(), "batch publication is incomplete");
        let names = required.clone().unwrap_or_else(|| result.families.keys().map(String::as_str).collect());
        for name in names {
            let Some(record) = result.families.get(name) else { continue };
            ensure!(record.exit_code == 0, "{name} has not passed actual-C verification");
            let entry = batch::entry(&catalog, name)?;
            let manifest = fs::canonicalize(checkout.join(&entry.manifest))?;
            ensure!(
                record.manifest_sha256.as_deref() == Some(digest(&manifest)?.as_str())
                    && record.kind.as_deref() == Some(entry.kind.as_str()),
                "verified family manifest changed"
            );
            let verdict = record.verdict.as_ref().context("passing family verdict missing")?;
            let authenticated = family::authenticate(family_model::AuthenticateRequest {
                checkout: checkout.into(),
                verdict: verdict.clone(),
            })?;
            ensure!(authenticated.status == "offline-verified", "family evidence did not authenticate");
            let declared: family_model::Manifest = pipeline_model::read(&manifest)?;
            ensure!(authenticated.family == declared.family, "batch verdict verifies a different Offline family");
            if entry.kind != "plm-instructions" {
                let identity = authenticated
                    .implementation
                    .as_ref()
                    .and_then(|implementation| implementation.manifest.as_ref())
                    .context("authenticated family manifest missing")?;
                ensure!(
                    fs::canonicalize(&identity.file)? == manifest
                        && Some(identity.sha256.as_str()) == record.manifest_sha256.as_deref(),
                    "batch verdict belongs to a different family manifest"
                );
            }
            verified.insert(name.to_owned(), verdict.clone());
        }
        roots.push(item.path.parent().context("batch parent missing")?.to_path_buf());
    }
    ensure!(!verified.is_empty(), "completion requires an actual-C batch record with authenticated family evidence");
    if let Some(required) = required {
        ensure!(required.iter().all(|name| verified.contains_key(*name)), "affected families remain unverified");
    }
    roots.sort();
    roots.dedup();
    Ok(roots)
}
fn receipt_path(status: &assignment::Receipt, claim: &Claim, name: &str) -> Result<PathBuf> {
    let root = status.ledger.parent().context("ledger parent missing")?.join("receipts");
    fs::create_dir_all(&root)?;
    Ok(root.join(format!("{}-{}-{name}.toml", claim.id, claim.attempt)))
}
fn authentication(status: &assignment::Receipt, claim: &Claim, evidence: &[Artifact], head: &str) -> Result<Artifact> {
    let path = receipt_path(status, claim, "authentication")?;
    crate::atomic_toml(
        &path,
        &crate::report!({"schema":"assignment-authentication-v1","claim":claim,"head":head,"evidence":evidence,"contract":"existing current-source family authentication"}),
    )?;
    artifact(path)
}
fn finish_cleanup(status: &assignment::Receipt, claim: &Claim, roots: &[PathBuf]) -> Result<Artifact> {
    let mut receipts = Vec::new();
    for root in roots {
        for deduplicate in [false, true] {
            receipts.push(cleanup::cleanup(cleanup::CleanupRequest { root: root.clone(), apply: true, deduplicate })?);
        }
    }
    let path = receipt_path(status, claim, "cleanup")?;
    crate::atomic_toml(&path, &crate::report!({"schema":"assignment-cleanup-v1","receipts":receipts}))?;
    artifact(path)
}
fn submission(checkout: &Path, job: &Job, commit: &str, integrated: bool) -> Result<()> {
    ensure!(commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()), "submission needs an exact commit");
    let base = job.inputs.get("base-head").context("implementation baseline missing")?;
    git(checkout, &["merge-base", "--is-ancestor", base, commit])?;
    if integrated {
        git(checkout, &["merge-base", "--is-ancestor", commit, "HEAD"])?;
    }
    let result = Command::new("git")
        .args(["diff", "--name-only", "--no-renames", "-z", base, commit, "--"])
        .current_dir(checkout)
        .output()?;
    ensure!(result.status.success(), "cannot inspect implementation commit");
    let mut changed = 0;
    for bytes in result.stdout.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
        let path = Path::new(std::str::from_utf8(bytes)?);
        ensure!(
            job.writable_paths.iter().any(|writable| path == writable),
            "submission exceeds writable scope: {}",
            path.display()
        );
        changed += 1;
    }
    ensure!(changed > 0, "submission contains no scoped implementation change");
    Ok(())
}
fn approved_scope(job: &Job) -> Result<translation::Scope> {
    let path = Path::new(job.inputs.get("scope-file").context("approved scope path missing")?);
    ensure!(
        digest(path)? == *job.inputs.get("scope-sha256").context("approved scope seal missing")?,
        "approved scope changed"
    );
    let scope: translation::Scope = pipeline_model::read(path)?;
    ensure!(
        digest(path)? == *job.inputs.get("scope-sha256").context("approved scope seal missing")?,
        "approved scope changed while reading"
    );
    ensure!(scope.writable_paths == job.writable_paths, "approved writable scope differs");
    Ok(scope)
}
fn assigned_worktree(request: &Request, claim: &Claim) -> PathBuf {
    std::env::temp_dir().join(format!("sm-{}-{}-{}", request.effort, claim.id, claim.attempt))
}
// Execute required argv under the existing process-group owner. Native verification
// acquires its own shared leases; an outer lease would deadlock nested workers.
fn gates(
    request: &Request,
    status: &assignment::Receipt,
    claim: &Claim,
    job: &Job,
    checkout: &Path,
    integration: bool,
) -> Result<Vec<Artifact>> {
    let scope = approved_scope(job)?;
    let before = source_fingerprint(checkout)?;
    let run = status.ledger.parent().context("ledger parent missing")?.join("receipts").join(format!(
        "{}-{}-gates-{}",
        claim.id,
        claim.attempt,
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_nanos()
    ));
    fs::create_dir_all(run.parent().context("gate receipt parent missing")?)?;
    fs::create_dir(&run)?;
    let root = request.pipeline_root.clone().unwrap_or_else(|| request.checkout.join("out/pipeline"));
    let mut artifacts = vec![];
    for (index, gate) in scope.verification.iter().enumerate().filter(|(_, gate)| gate.integration == integration) {
        let mut argv = gate.argv.clone();
        if argv[0] == "sm-pipeline" {
            argv[0] = std::env::current_exe()?.display().to_string();
        }
        let stdout = run.join(format!("{index}.stdout"));
        let stderr = run.join(format!("{index}.stderr"));
        let mut command = Command::new(&argv[0]);
        command
            .args(&argv[1..])
            .current_dir(checkout)
            .env("SM_PIPELINE_ROOT", &root)
            .env("SM_PIPELINE_WORKERS", request.workers.to_string())
            .stdin(std::process::Stdio::null())
            .stdout(fs::File::create(&stdout)?)
            .stderr(fs::File::create(&stderr)?);
        let mut metrics = crate::metrics::Run::start(run.join(format!("{index}.metrics.toml")), "assignment-gate")?;
        let context = metrics.context.clone();
        let mut child = crate::metrics::OwnedChild::spawn(&mut command, &context, "assignment-scope-gate")?;
        let started = std::time::Instant::now();
        let mut timed_out = false;
        let result = loop {
            if let Some(result) = child.poll()? {
                break result;
            }
            if started.elapsed().as_secs() >= gate.timeout_seconds {
                timed_out = true;
                break child.terminate_with_timeout(std::time::Duration::from_secs(2))?;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        };
        let receipt = run.join(format!("{index}.toml"));
        crate::atomic_toml(
            &receipt,
            &crate::report!({"schema":"assignment-gate-v1","argv":gate.argv,"actual_argv":argv,"checkout":checkout,"source_fingerprint":before,"integration":integration,"seconds":started.elapsed().as_secs_f64(),"exit_code":result.code().unwrap_or(-1),"timed_out":timed_out,"stdout_sha256":digest(&stdout)?,"stderr_sha256":digest(&stderr)?}),
        )?;
        metrics.finish(if result.success() && !timed_out { "success" } else { "failed" }, None)?;
        ensure!(
            !timed_out,
            "scope gate {index} exceeded {} seconds; diagnostics retained in {}",
            gate.timeout_seconds,
            run.display()
        );
        ensure!(result.success(), "scope gate {index} failed; diagnostics retained in {}", run.display());
        if gate.argv.starts_with(&["cargo".into(), "test".into()]) {
            let output = fs::read_to_string(&stdout)?;
            ensure!(
                output.lines().any(|line| line
                    .strip_prefix("running ")
                    .and_then(|text| text.split_whitespace().next())
                    .and_then(|number| number.parse::<usize>().ok())
                    .is_some_and(|count| count > 0)),
                "required cargo gate executed zero reported tests"
            );
        } else {
            let output: toml::Value = toml::from_str(&fs::read_to_string(&stdout)?)
                .context("native verification gate did not produce a receipt")?;
            if let Some(path) = output.get("batch").and_then(toml::Value::as_str) {
                artifacts.push(artifact(fs::canonicalize(path)?)?);
            } else if let Some(path) = output.get("run").and_then(toml::Value::as_str) {
                artifacts.push(artifact(fs::canonicalize(Path::new(path).join("verdict.toml"))?)?);
            } else {
                anyhow::bail!("required native gate produced no verification evidence");
            }
        }
        artifacts.extend([artifact(receipt)?, artifact(stdout)?, artifact(stderr)?]);
    }
    ensure!(!artifacts.is_empty(), "scope is missing required verification gates");
    ensure!(source_fingerprint(checkout)? == before, "source changed during required scope gates");
    Ok(artifacts)
}
fn followups(checkout: &Path, parent: &Job, evidence: &[Artifact]) -> Result<Vec<Job>> {
    let mut jobs = vec![];
    for item in evidence {
        checked(item)?;
        let Ok(value) = pipeline_model::read::<toml::Value>(&item.path) else {
            continue;
        };
        if value.get("format").and_then(toml::Value::as_str) != Some("translation-scope-v1") {
            continue;
        }
        match translation::validate_scope(checkout, &item.path) {
            Ok(mut job) => { attach_base(checkout, &mut job)?; jobs.push(job); }
            Err(error) => jobs.push(Job { id: format!("research-{}", &hash(format!("{}:{}", parent.id, item.sha256))[..16]), revision: item.sha256.clone(), kind: Kind::Research, subject: format!("Complete scope: {}", parent.subject), writable_paths: vec![], blockers: vec![], inputs: BTreeMap::from([(format!("file:{}", item.path.display()), item.sha256.clone())]), instructions: format!("Scope {} remains research: {error:#}. Establish the missing facts in a new scope receipt. Initial experiments are bounded at 60 seconds.", item.path.display()), ticket: None }),
        }
    }
    Ok(jobs)
}
fn validate_update(request: &Request, status: &assignment::Receipt, update: &mut Update) -> Result<()> {
    let requested = update.clone();
    let Update::Finished { claim, result } = update else {
        return Ok(());
    };
    let entry =
        status.jobs.iter().find(|entry| entry.claim.as_ref() == Some(claim)).context("unknown current claim")?;
    ensure!(request.coordinator == claim.coordinator, "coordinator cannot validate another owner's result");
    ensure!(
        matches!(
            (&entry.job.kind, &*result),
            (Kind::Verify, WorkResult::Verified { .. })
                | (Kind::Diagnose, WorkResult::Diagnosed { .. })
                | (Kind::Research, WorkResult::Researched { .. })
                | (Kind::Implement, WorkResult::Submitted { .. })
                | (Kind::Integrate, WorkResult::Integrated { .. })
                | (_, WorkResult::Failed { .. })
        ),
        "result does not match assignment work kind"
    );
    let request_path = receipt_path(status, claim, &format!("request-{}", hash(toml::to_string(&requested)?)))?;
    if let Some(Update::Finished { result: previous, .. }) = &entry.last_update {
        let recorded_request =
            assignment::result_artifacts(previous).into_iter().find(|item| item.path == request_path);
        if entry.last_update.as_ref() == Some(&requested) || recorded_request.is_some() {
            for item in assignment::result_artifacts(previous) {
                checked(item)?;
            }
            if let Some(item) = recorded_request {
                let recorded: RecordedRequest = pipeline_model::read(&item.path)?;
                ensure!(
                    recorded.schema == "assignment-result-request-v1" && recorded.update == requested,
                    "recorded result request differs"
                );
            }
            *result = previous.clone();
            return Ok(());
        }
    }
    ensure!(
        entry.state == assignment::State::Running && !entry.stale && entry.pending_job.is_none(),
        "only current running work can validate a new result"
    );
    let head = git(&request.checkout, &["rev-parse", "HEAD"])?;
    let source = source_fingerprint(&request.checkout)?;
    if let Some(expected) = entry.job.inputs.get("source-fingerprint") {
        ensure!(*expected == source, "reserved verification source changed; refresh and reconcile claim");
    }
    match result {
        WorkResult::Verified { evidence, authentication: auth } => {
            let required = entry.job.inputs.get("families").map(String::as_str);
            let roots = authenticate_batches(&request.checkout, evidence, required)?;
            evidence.push(finish_cleanup(status, claim, &roots)?);
            authenticate_batches(&request.checkout, evidence, required)?;
            *auth = authentication(status, claim, evidence, &head)?;
        }
        WorkResult::Researched { evidence, followups: jobs } | WorkResult::Diagnosed { evidence, followups: jobs } => {
            *jobs = followups(&request.checkout, &entry.job, evidence)?;
        }
        WorkResult::Failed { evidence, followups: jobs, .. } => {
            *jobs = vec![];
            let mut roots = Vec::new();
            for item in evidence.iter() {
                checked(item)?;
                if pipeline_model::read::<batch::Batch>(&item.path).is_ok() {
                    jobs.extend(diagnosis_jobs(&request.checkout, &item.path)?);
                    roots.push(item.path.parent().context("failed batch parent missing")?.to_path_buf());
                }
            }
            if !roots.is_empty() {
                roots.sort();
                roots.dedup();
                evidence.push(finish_cleanup(status, claim, &roots)?);
            }
        }
        WorkResult::Submitted { commit, evidence } => {
            submission(&request.checkout, &entry.job, commit, false)?;
            for item in evidence.iter() {
                checked(item)?;
            }
            let worktree = assigned_worktree(request, claim);
            ensure!(
                worktree.join(".git").is_file() && git(&worktree, &["rev-parse", "HEAD"])? == *commit,
                "submission must use its assigned isolated worktree at the submitted commit"
            );
            ensure!(
                git(&worktree, &["status", "--porcelain"])?.is_empty(),
                "submission worktree has uncommitted edits"
            );
            evidence.extend(gates(request, status, claim, &entry.job, &worktree, false)?);
        }
        WorkResult::Integrated { submissions, evidence, authentication: auth, cleanup: cleaned } => {
            ensure!(
                submissions.len() == 1
                    && entry.job.inputs.get("submission-id") == Some(&submissions[0].id)
                    && entry.job.inputs.get("submission-attempt") == Some(&submissions[0].attempt.to_string())
                    && entry.job.inputs.get("submission-revision") == Some(&submissions[0].revision),
                "integration result does not name its assigned submission"
            );
            let mut required = std::collections::BTreeSet::new();
            for claim in submissions.iter() {
                let submitted =
                    status.jobs.iter().find(|e| e.claim.as_ref() == Some(claim)).context("submission claim missing")?;
                ensure!(
                    submitted.state == assignment::State::Submitted
                        && !submitted.stale
                        && submitted.pending_job.is_none(),
                    "integration submission is stale or not submitted"
                );
                let Some(WorkResult::Submitted { commit, .. }) = &submitted.result else {
                    anyhow::bail!("implementation is not submitted");
                };
                submission(&request.checkout, &submitted.job, commit, true)?;
                let config = batch::catalog(&request.checkout)?;
                let names = config.families.keys().cloned().collect::<Vec<_>>();
                let baseline = submitted.job.inputs.get("base-head").context("submission baseline missing")?;
                required.extend(affected::select(&request.checkout, &names, Some(baseline))?.families());
                evidence.extend(gates(request, status, claim, &submitted.job, &request.checkout, true)?);
                let name = format!("{}-{}-{}", request.effort, claim.id, claim.attempt);
                let worktree = std::env::temp_dir().join(format!("sm-{name}"));
                ensure!(
                    !worktree.join(".git").exists(),
                    "integrated implementer worktree still registered: {}",
                    worktree.display()
                );
            }
            let required = required.into_iter().collect::<Vec<_>>().join("\n");
            ensure!(
                !required.is_empty(),
                "integration changed no declared verification inputs; add an Offline family for the new scope"
            );
            let roots = authenticate_batches(&request.checkout, evidence, Some(&required))?;
            *cleaned = finish_cleanup(status, claim, &roots)?;
            authenticate_batches(&request.checkout, evidence, Some(&required))?;
            *auth = authentication(status, claim, evidence, &head)?;
        }
    }
    ensure!(git(&request.checkout, &["rev-parse", "HEAD"])? == head, "source HEAD changed during result validation");
    ensure!(source_fingerprint(&request.checkout)? == source, "source changed during result validation");
    crate::atomic_toml(
        &request_path,
        &RecordedRequest { schema: "assignment-result-request-v1".into(), update: requested },
    )?;
    evidence_mut(result).push(artifact(request_path)?);
    Ok(())
}
pub(crate) fn operate(mut request: Request) -> Result<toml::Value> {
    request.checkout = main_checkout(&request.checkout)?;
    if let Some(root) = &request.pipeline_root
        && !root.is_absolute()
    {
        request.pipeline_root = Some(request.checkout.join(root));
    }
    ensure!((1..=64).contains(&request.workers), "assignment worker capacity must be 1..64");
    ensure!(["plan", "next", "record", "status"].contains(&request.operation.as_str()), "unsupported assign operation");
    if request.operation == "status" {
        return Ok(toml::Value::try_from(assignment::operate(queue(&request, "status"))?)?);
    }
    if request.operation == "record" {
        let status = assignment::operate(queue(&request, "status"))?;
        let mut update = request.update.take().context("record requires a typed update")?;
        validate_update(&request, &status, &mut update)?;
        let mut queued = queue(&request, "record");
        queued.update = Some(update);
        return Ok(toml::Value::try_from(assignment::operate(queued)?)?);
    }
    ensure!(request.update.is_none(), "updates require record");
    let before = source_fingerprint(&request.checkout)?;
    let catalog = batch::catalog(&request.checkout)?;
    let names = request
        .families
        .clone()
        .or_else(|| catalog.profiles.get(&request.profile).cloned())
        .context("unknown profile")?;
    let selection = affected::select(&request.checkout, &names, request.changed_since.as_deref())?;
    ensure!(
        toml::to_string(&catalog)? == toml::to_string(&batch::catalog(&request.checkout)?)?,
        "catalog changed during assignment selection; retry after source freeze"
    );
    let mut jobs = vec![verification_job(&selection)?];
    if request.translations {
        jobs.extend(translation::discover(
            &request.checkout,
            request.source_inventory.as_deref(),
            request.exercise.as_deref(),
        )?);
    }
    if let Some(path) = &request.scope {
        let mut job = translation::validate_scope(&request.checkout, path)?;
        attach_base(&request.checkout, &mut job)?;
        jobs.push(job);
    }
    if let Some(path) = &request.batch_result {
        jobs.extend(diagnosis_jobs(&request.checkout, path)?);
    }
    let mut queued = queue(&request, &request.operation);
    ensure!(source_fingerprint(&request.checkout)? == before, "source changed while deriving assignments; retry");
    queued.jobs = jobs;
    let receipt = assignment::operate(queued)?;
    Ok(crate::report!({"selection":selection,"queue":receipt}))
}
