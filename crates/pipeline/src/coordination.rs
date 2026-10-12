//! Coordinator adapter: derive jobs and validate outcomes before durable claims change.
use crate::{affected, assignment::{self, Artifact, Claim, Job, Kind, Update, WorkResult}, batch, cleanup, digest, family, family_model, translation};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::{Path, PathBuf}, process::Command};

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
}
impl Default for Request {
    fn default() -> Self {
        Self { operation: "status".into(), checkout: PathBuf::new(), effort: "affected-family-assignment".into(), coordinator: String::new(), capacity: 3, available_slots: 0, profile: "offline-all".into(), families: None, changed_since: None, translations: true, source_inventory: None, exercise: None, batch_result: None, scope: None, update: None }
    }
}
fn hash(value: impl AsRef<[u8]>) -> String { format!("{:x}", Sha256::digest(value.as_ref())) }
fn git(checkout: &Path, args: &[&str]) -> Result<String> {
    let result = Command::new("git").args(args).current_dir(checkout).output()?;
    ensure!(result.status.success(), "git {} failed: {}", args.join(" "), String::from_utf8_lossy(&result.stderr));
    Ok(String::from_utf8(result.stdout)?.trim().into())
}
fn main_checkout(checkout: &Path) -> Result<PathBuf> {
    let common = fs::canonicalize(checkout.join(git(checkout, &["rev-parse", "--git-common-dir"])?) )?;
    ensure!(common.file_name().is_some_and(|name| name == ".git"), "assignment requires a main checkout");
    Ok(common.parent().context("main checkout missing")?.into())
}
fn queue(request: &Request, operation: &str) -> assignment::Request {
    assignment::Request { operation: operation.into(), checkout: request.checkout.clone(), effort: request.effort.clone(), coordinator: request.coordinator.clone(), capacity: request.capacity, available_slots: request.available_slots, jobs: vec![], update: None }
}
fn checked(artifact: &Artifact) -> Result<()> {
    ensure!(artifact.path.is_file() && !artifact.path.is_symlink() && digest(&artifact.path)? == artifact.sha256, "result artifact changed: {}", artifact.path.display());
    Ok(())
}
fn artifact(path: PathBuf) -> Result<Artifact> { Ok(Artifact { sha256: digest(&path)?, path }) }
fn attach_base(checkout: &Path, job: &mut Job) -> Result<()> {
    if job.kind == Kind::Implement {
        job.inputs.insert("base-head".into(), git(checkout, &["rev-parse", "HEAD"])?);
        job.revision = hash(toml::to_string(&job.inputs)?);
    }
    Ok(())
}
fn verification_job(report: &affected::Report) -> Result<Option<Job>> {
    let families = report.families();
    if families.is_empty() { return Ok(None); }
    let mut inputs = report.planning_inputs.iter().map(|(path, hash)| (format!("file:{}", path.display()), hash.clone())).collect::<BTreeMap<_, _>>();
    inputs.insert("families".into(), families.join("\n"));
    inputs.insert("selection".into(), hash(toml::to_string(report)?));
    let revision = hash(toml::to_string(&inputs)?);
    let id = format!("verify-{}", &hash(families.join("\n"))[..16]);
    Ok(Some(Job { id, revision, kind: Kind::Verify, subject: format!("Verify {} affected Offline families", families.len()), writable_paths: vec![], blockers: vec![], inputs, instructions: format!("Run one `sm-pipeline batch verify --families {} --batch ASSIGNMENT_BATCH` using the shared SM_PIPELINE_ROOT/SM_PIPELINE_WORKERS. Keep batch jobs within shared worker capacity. Record the batch.toml artifact; current-source authentication is required before completion. Selection rationale is available through batch plan. Preserve failed verdicts for diagnosis and complete run cleanup.", families.join(" ")), ticket: None }))
}
fn diagnosis_jobs(checkout: &Path, path: &Path) -> Result<Vec<Job>> {
    let result: batch::Batch = pipeline_model::read(path)?;
    let mut jobs = vec![];
    for (name, record) in result.families {
        if record.exit_code == 0 { continue; }
        let mut inputs = BTreeMap::from([(format!("file:{}", fs::canonicalize(path)?.display()), digest(path)?)]);
        if let Some(verdict) = record.verdict { inputs.insert(format!("file:{}", verdict.display()), digest(&verdict)?); }
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
        let Ok(result) = pipeline_model::read::<batch::Batch>(&item.path) else { continue; };
        ensure!(result.schema_version == 1 && result.publication_error.is_none(), "batch publication is incomplete");
        let names = required.clone().unwrap_or_else(|| result.families.keys().map(String::as_str).collect());
        for name in names {
            let record = result.families.get(name).with_context(|| format!("batch does not verify {name}"))?;
            ensure!(record.exit_code == 0, "{name} has not passed actual-C verification");
            let manifest = match catalog.families.get(name).context("verified family missing from current catalog")? {
                batch::CatalogEntry::Kind(_) => PathBuf::from(format!("tools/port/{name}.toml")),
                batch::CatalogEntry::Details(entry) => entry.manifest.clone(),
            };
            ensure!(record.manifest_sha256.as_deref() == Some(digest(&checkout.join(manifest))?.as_str()), "verified family manifest changed");
            let verdict = record.verdict.as_ref().context("passing family verdict missing")?;
            let authenticated = family::authenticate(family_model::AuthenticateRequest { checkout: checkout.into(), verdict: verdict.clone() })?;
            ensure!(authenticated.status == "offline-verified", "family evidence did not authenticate");
            verified.insert(name.to_owned(), verdict.clone());
        }
        roots.push(item.path.parent().context("batch parent missing")?.to_path_buf());
    }
    ensure!(!verified.is_empty(), "completion requires an actual-C batch record with authenticated family evidence");
    if let Some(required) = required { ensure!(required.iter().all(|name| verified.contains_key(*name)), "affected families remain unverified"); }
    Ok(roots)
}
fn receipt_path(status: &assignment::Receipt, claim: &Claim, name: &str) -> Result<PathBuf> {
    let root = status.ledger.parent().context("ledger parent missing")?.join("receipts");
    fs::create_dir_all(&root)?;
    Ok(root.join(format!("{}-{}-{name}.toml", claim.id, claim.attempt)))
}
fn authentication(status: &assignment::Receipt, claim: &Claim, evidence: &[Artifact], head: &str) -> Result<Artifact> {
    let path = receipt_path(status, claim, "authentication")?;
    crate::atomic_toml(&path, &crate::report!({"schema":"assignment-authentication-v1","claim":claim,"head":head,"evidence":evidence,"contract":"existing current-source family authentication"}))?;
    artifact(path)
}
fn submission(checkout: &Path, job: &Job, commit: &str, integrated: bool) -> Result<()> {
    ensure!(commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit()), "submission needs an exact commit");
    let base = job.inputs.get("base-head").context("implementation baseline missing")?;
    git(checkout, &["merge-base", "--is-ancestor", base, commit])?;
    if integrated { git(checkout, &["merge-base", "--is-ancestor", commit, "HEAD"])?; }
    let result = Command::new("git").args(["diff", "--name-only", "--no-renames", "-z", base, commit, "--"]).current_dir(checkout).output()?;
    ensure!(result.status.success(), "cannot inspect implementation commit");
    let mut changed = 0;
    for bytes in result.stdout.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
        let path = Path::new(std::str::from_utf8(bytes)?);
        ensure!(job.writable_paths.iter().any(|writable| path == writable), "submission exceeds writable scope: {}", path.display());
        changed += 1;
    }
    ensure!(changed > 0, "submission contains no scoped implementation change");
    Ok(())
}
fn followups(checkout: &Path, parent: &Job, evidence: &[Artifact]) -> Result<Vec<Job>> {
    let mut jobs = vec![];
    for item in evidence {
        checked(item)?;
        let Ok(value) = pipeline_model::read::<toml::Value>(&item.path) else { continue; };
        if value.get("format").and_then(toml::Value::as_str) != Some("translation-scope-v1") { continue; }
        match translation::validate_scope(checkout, &item.path) {
            Ok(mut job) => { attach_base(checkout, &mut job)?; jobs.push(job); }
            Err(error) => jobs.push(Job { id: format!("research-{}", &hash(format!("{}:{}", parent.id, item.sha256))[..16]), revision: item.sha256.clone(), kind: Kind::Research, subject: format!("Complete scope: {}", parent.subject), writable_paths: vec![], blockers: vec![], inputs: BTreeMap::from([(format!("file:{}", item.path.display()), item.sha256.clone())]), instructions: format!("Scope {} remains research: {error:#}. Establish the missing facts in a new scope receipt. Initial experiments are bounded at 60 seconds.", item.path.display()), ticket: None }),
        }
    }
    Ok(jobs)
}
fn validate_update(request: &Request, status: &assignment::Receipt, update: &mut Update) -> Result<()> {
    let Update::Finished { claim, result } = update else { return Ok(()); };
    let entry = status.jobs.iter().find(|entry| entry.claim.as_ref() == Some(claim)).context("unknown current claim")?;
    let head = git(&request.checkout, &["rev-parse", "HEAD"])?;
    match result {
        WorkResult::Verified { evidence, authentication: auth } => {
            authenticate_batches(&request.checkout, evidence, entry.job.inputs.get("families").map(String::as_str))?;
            *auth = authentication(status, claim, evidence, &head)?;
        }
        WorkResult::Researched { evidence, followups: jobs } | WorkResult::Diagnosed { evidence, followups: jobs } => {
            *jobs = followups(&request.checkout, &entry.job, evidence)?;
        }
        WorkResult::Failed { evidence, followups: jobs, .. } => {
            *jobs = vec![];
            for item in evidence.iter() {
                checked(item)?;
                if pipeline_model::read::<batch::Batch>(&item.path).is_ok() { jobs.extend(diagnosis_jobs(&request.checkout, &item.path)?); }
            }
        }
        WorkResult::Submitted { commit, evidence } => {
            submission(&request.checkout, &entry.job, commit, false)?;
            for item in evidence { checked(item)?; }
        }
        WorkResult::Integrated { submissions, evidence, authentication: auth, cleanup: cleaned } => {
            for claim in submissions.iter() {
                let submitted = status.jobs.iter().find(|e| e.claim.as_ref() == Some(claim)).context("submission claim missing")?;
                let Some(WorkResult::Submitted { commit, .. }) = &submitted.result else { anyhow::bail!("implementation is not submitted"); };
                submission(&request.checkout, &submitted.job, commit, true)?;
                let name = format!("{}-{}-{}", request.effort, claim.id, claim.attempt);
                let worktree = std::env::temp_dir().join(format!("sm-{name}"));
                ensure!(!worktree.join(".git").exists(), "integrated implementer worktree still registered: {}", worktree.display());
            }
            let roots = authenticate_batches(&request.checkout, evidence, None)?;
            let mut receipts = vec![];
            for root in roots { for deduplicate in [false, true] { receipts.push(cleanup::cleanup(cleanup::CleanupRequest { root: root.clone(), apply: true, deduplicate })?); } }
            authenticate_batches(&request.checkout, evidence, None)?;
            let path = receipt_path(status, claim, "cleanup")?;
            crate::atomic_toml(&path, &crate::report!({"schema":"assignment-cleanup-v1","receipts":receipts,"submissions":submissions}))?;
            *cleaned = artifact(path)?;
            *auth = authentication(status, claim, evidence, &head)?;
        }
    }
    ensure!(git(&request.checkout, &["rev-parse", "HEAD"])? == head, "source HEAD changed during result validation");
    Ok(())
}
pub(crate) fn operate(mut request: Request) -> Result<toml::Value> {
    request.checkout = main_checkout(&request.checkout)?;
    ensure!(["plan", "next", "record", "status"].contains(&request.operation.as_str()), "unsupported assign operation");
    if request.operation == "status" { return Ok(toml::Value::try_from(assignment::operate(queue(&request, "status"))?)?); }
    if request.operation == "record" {
        let status = assignment::operate(queue(&request, "status"))?;
        let mut update = request.update.take().context("record requires a typed update")?;
        validate_update(&request, &status, &mut update)?;
        let mut queued = queue(&request, "record"); queued.update = Some(update);
        return Ok(toml::Value::try_from(assignment::operate(queued)?)?);
    }
    ensure!(request.update.is_none(), "updates require record");
    let catalog = batch::catalog(&request.checkout)?;
    let names = request.families.clone().or_else(|| catalog.profiles.get(&request.profile).cloned()).context("unknown profile")?;
    let selection = affected::select(&request.checkout, &names, request.changed_since.as_deref())?;
    let mut jobs = verification_job(&selection)?.into_iter().collect::<Vec<_>>();
    if request.translations { jobs.extend(translation::discover(&request.checkout, request.source_inventory.as_deref(), request.exercise.as_deref())?); }
    if let Some(path) = &request.scope { let mut job = translation::validate_scope(&request.checkout, path)?; attach_base(&request.checkout, &mut job)?; jobs.push(job); }
    if let Some(path) = &request.batch_result { jobs.extend(diagnosis_jobs(&request.checkout, path)?); }
    let mut queued = queue(&request, &request.operation); queued.jobs = jobs;
    let receipt = assignment::operate(queued)?;
    Ok(crate::report!({"selection":selection,"queue":receipt}))
}
