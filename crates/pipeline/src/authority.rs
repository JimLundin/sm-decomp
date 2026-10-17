//! Session fencing and retained ownership policy for the existing assignment queue.
use super::{Artifact, Claim, Job, Kind, atomic_write, check_artifact, main_checkout, relative, safe_name};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

const POLICY: &str = ".scratch/decomp-parallel/authority.toml";
const SCHEMA: &str = "decomp-session-authority-v1";

#[path = "drill_evidence.rs"]
pub(crate) mod drill_evidence;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionAuthority {
    pub session_id: String,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum RoleKind {
    Integrator,
    Dispatcher,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleRegistration {
    pub token: String,
    pub kind: RoleKind,
    pub session_id: String,
    pub observation: Artifact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RoleTransfer {
    pub token: String,
    pub session_id: String,
    pub observation: Artifact,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Preparation {
    pub canonical_effort: String,
    pub capacity: usize,
    pub admitted_efforts: Vec<String>,
    pub drill_efforts: Vec<String>,
    pub bridge: Artifact,
    pub integrator: RoleRegistration,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Role {
    pub token: String,
    pub kind: RoleKind,
    pub session_id: String,
    pub generation: u64,
    pub registration: Artifact,
    #[serde(default)]
    pub transfers: Vec<Artifact>,
    pub last_transfer: Option<RoleTransfer>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct Request {
    pub operation: String,
    pub checkout: PathBuf,
    pub coordinator: String,
    pub authority: Option<SessionAuthority>,
    pub preparation: Option<Preparation>,
    pub registration: Option<RoleRegistration>,
    pub transfer: Option<RoleTransfer>,
    pub drill: Option<Artifact>,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            operation: "status".into(),
            checkout: PathBuf::new(),
            coordinator: String::new(),
            authority: None,
            preparation: None,
            registration: None,
            transfer: None,
            drill: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Receipt {
    pub schema: String,
    pub status: String,
    pub policy: PathBuf,
    pub canonical_effort: Option<String>,
    pub capacity: Option<usize>,
    pub roles: Vec<Role>,
    pub bridge: Option<Artifact>,
    pub drill: Option<Artifact>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Policy {
    schema: String,
    pub(super) state: String,
    pub(super) preparation: Preparation,
    roles: BTreeMap<String, Role>,
    drill: Option<Artifact>,
    prepared_head: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SessionObservation {
    pub schema: String,
    pub session_id: String,
    pub status: String,
    pub observed_at: String,
    pub evidence: Vec<Artifact>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ChildObservation {
    pub effort: String,
    pub claim: Claim,
    pub agent: Option<String>,
    pub status: String,
    pub observation: Artifact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessObservation {
    pub pid: u32,
    pub argv: Vec<String>,
    pub started_at: String,
    pub status: String,
    pub agent: Option<String>,
    pub observation: Artifact,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TransferObservation {
    pub schema: String,
    pub token: String,
    pub session_id: String,
    pub generation: u64,
    pub status: String,
    pub observed_at: String,
    pub evidence: Vec<Artifact>,
    pub replacement: Artifact,
    #[serde(default)]
    pub children: Vec<ChildObservation>,
    #[serde(default)]
    pub processes: Vec<ProcessObservation>,
}

/// Additional lineage fields in a retained bridge are read without rewriting it.
#[derive(Clone, Debug, Deserialize)]
pub(super) struct Bridge {
    pub schema: String,
    pub ledger: Option<Artifact>,
    pub source_baseline: Option<String>,
    #[serde(default)]
    pub holdings: Vec<Holding>,
    #[serde(default)]
    liveness_inspections: Vec<Artifact>,
    maintenance_recommendation: Option<Artifact>,
    maintenance_approval: Option<Artifact>,
    #[serde(default)]
    maintenance_adoptions: Vec<MaintenanceAdoption>,
}

#[derive(Clone, Debug, Deserialize)]
pub(super) struct Holding {
    pub id: String,
    #[serde(default)]
    pub effort: String,
    #[serde(default)]
    pub kind: String,
    pub state: String,
    pub ticket: Option<PathBuf>,
    pub claim: Option<Claim>,
    pub job_id: Option<String>,
    pub revision: Option<String>,
    pub related_submission_hold: Option<String>,
    #[serde(default)]
    pub writable_paths: Vec<PathBuf>,
    pub retained: bool,
    #[serde(default)]
    pub evidence: Vec<Artifact>,
    #[serde(default)]
    pub source: Vec<Artifact>,
    pub source_snapshot: Option<String>,
    pub observation: Option<Artifact>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MaintenanceAdoption {
    id: String,
    holding_ids: Vec<String>,
    effort: String,
    session_id: String,
    writable_paths: Vec<PathBuf>,
    observation: Artifact,
}

pub(crate) fn real_session_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(
            |(i, b)| {
                if [8, 13, 18, 23].contains(&i) { b == b'-' } else { b.is_ascii_hexdigit() }
            },
        )
}

fn caller(id: &str) -> Result<()> {
    ensure!(real_session_id(id), "authority requires a real session/thread id");
    let actual = std::env::var("CODEX_THREAD_ID").context("CODEX_THREAD_ID is required for session mutation")?;
    ensure!(actual == id, "authority session differs from the calling real session");
    Ok(())
}

pub(super) fn stopped(status: &str) -> bool {
    ["completed", "failed", "interrupted"].contains(&status)
}
fn live(status: &str) -> bool {
    ["running", "active"].contains(&status)
}
fn known_status(status: &str) -> bool {
    stopped(status) || live(status) || status == "not-launched"
}

fn artifacts(artifacts: &[Artifact]) -> Result<()> {
    ensure!(!artifacts.is_empty(), "observation requires sealed inspection evidence");
    for artifact in artifacts {
        check_artifact(artifact)?;
    }
    Ok(())
}

fn session_observation(artifact: &Artifact, id: &str) -> Result<()> {
    check_artifact(artifact)?;
    let observation: SessionObservation = toml::from_str(&fs::read_to_string(&artifact.path)?)?;
    ensure!(observation.schema == "decomp-session-observation-v1", "unsupported session observation");
    ensure!(observation.session_id == id && real_session_id(id), "observation names another real session");
    ensure!(
        live(&observation.status) && !observation.observed_at.is_empty(),
        "registered session is not observed live"
    );
    artifacts(&observation.evidence)
}

pub(super) fn read(root: &Path) -> Result<Option<Policy>> {
    let path = root.join(POLICY);
    ensure!(!path.is_symlink(), "session authority cannot be a symlink");
    if !path.exists() {
        return Ok(None);
    }
    let policy: Policy = toml::from_str(&fs::read_to_string(&path)?)?;
    ensure!(
        policy.schema == SCHEMA && ["prepared", "active"].contains(&policy.state.as_str()),
        "unsupported session authority policy"
    );
    ensure!(policy.preparation.capacity > 0 && policy.preparation.capacity <= 64, "invalid shared assignment capacity");
    ensure!(
        policy.roles.values().filter(|role| role.kind == RoleKind::Integrator).count() == 1,
        "one explicit integrator is required"
    );
    check_artifact(&policy.preparation.bridge)?;
    for (token, role) in &policy.roles {
        ensure!(
            token == &role.token && safe_name(token) && real_session_id(&role.session_id) && role.generation > 0,
            "invalid session authority role"
        );
    }
    Ok(Some(policy))
}

fn role<'a>(policy: &'a Policy, token: &str, credential: Option<&SessionAuthority>) -> Result<&'a Role> {
    let credential = credential.context("managed mutation requires session authority")?;
    caller(&credential.session_id)?;
    let role = policy.roles.get(token).context("coordinator token is not a registered session role")?;
    ensure!(
        role.session_id == credential.session_id && role.generation == credential.generation,
        "stale session authority generation or owner"
    );
    Ok(role)
}

pub(super) fn integrator(policy: &Policy) -> &str {
    &policy.roles.values().find(|role| role.kind == RoleKind::Integrator).unwrap().token
}

pub(super) fn validate_locked(
    root: &Path,
    effort: &str,
    coordinator: &str,
    credential: Option<&SessionAuthority>,
    integration: bool,
) -> Result<Option<Policy>> {
    let Some(policy) = read(root)? else {
        ensure!(effort != "decomp-parallel", "canonical decomp authority has not been prepared");
        ensure!(credential.is_none(), "session authority policy is absent");
        return Ok(None);
    };
    let admitted = policy.preparation.admitted_efforts.iter().any(|name| name == effort)
        || policy.preparation.drill_efforts.iter().any(|name| name == effort)
        || (policy.state == "active" && policy.preparation.canonical_effort == effort);
    ensure!(admitted, "effort is not admitted by current session authority");
    ensure!(
        policy.state == "active" || effort != policy.preparation.canonical_effort,
        "canonical production authority is prepared and inactive"
    );
    let current = role(&policy, coordinator, credential)?;
    ensure!(
        !integration || current.kind == RoleKind::Integrator,
        "only the current integrator session can mutate integration"
    );
    Ok(Some(policy))
}

/// Read-only preflight; assignment repeats this check inside the global mutation lock.
pub(crate) fn validate(
    checkout: &Path,
    effort: &str,
    coordinator: &str,
    credential: Option<&SessionAuthority>,
    integration: bool,
) -> Result<()> {
    validate_locked(&main_checkout(checkout)?, effort, coordinator, credential, integration).map(|_| ())
}

fn git_commit(root: &Path, revision: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--verify", &format!("{revision}^{{commit}}")])
        .current_dir(root)
        .output()?;
    ensure!(output.status.success(), "retained source commit is missing: {revision}");
    Ok(std::str::from_utf8(&output.stdout)?.trim().to_owned())
}

pub(super) fn bridge(policy: &Policy, root: &Path, authenticate: bool) -> Result<Bridge> {
    check_artifact(&policy.preparation.bridge)?;
    let mut bridge: Bridge = toml::from_str(&fs::read_to_string(&policy.preparation.bridge.path)?)?;
    ensure!(bridge.schema == "decomp-legacy-owner-bridge-v1", "unsupported retained ownership bridge");
    let ledger_effort =
        bridge.ledger.as_ref().and_then(|artifact| artifact.path.parent()?.file_name()?.to_str()).map(str::to_owned);
    let mut ids = BTreeSet::new();
    for holding in &mut bridge.holdings {
        ensure!(
            safe_name(&holding.id) && ids.insert(holding.id.clone()) && holding.retained,
            "invalid or released bridge holding"
        );
        ensure!(
            holding.writable_paths.iter().all(|path| relative(path)),
            "bridge writable paths must be safe relative resources"
        );
        if let Some(ticket) = &holding.ticket {
            ensure!(relative(ticket) && ticket.starts_with(".scratch"), "bridge ticket is unsafe");
        }
        if holding.effort.is_empty() {
            holding.effort = if holding.claim.is_some() || holding.kind == "ready-unclaimed-job" {
                ledger_effort.clone().context("durable bridge holding lacks original effort")?
            } else {
                holding
                    .ticket
                    .as_ref()
                    .and_then(|ticket| ticket.components().nth(1))
                    .and_then(|part| part.as_os_str().to_str())
                    .context("Markdown holding lacks effort")?
                    .to_owned()
            };
        }
        ensure!(safe_name(&holding.effort), "invalid original bridge effort");
        if authenticate {
            for artifact in holding.evidence.iter().chain(&holding.source).chain(holding.observation.iter()) {
                check_artifact(artifact)?;
            }
            if let Some(commit) = &holding.source_snapshot {
                ensure!(git_commit(root, commit)? == *commit, "source snapshot must be a full retained commit");
            }
        }
    }
    if authenticate {
        if let Some(ledger) = &bridge.ledger {
            check_artifact(ledger)?;
        }
        if let Some(commit) = &bridge.source_baseline {
            ensure!(git_commit(root, commit)? == *commit, "bridge source baseline must be exact");
        }
        for artifact in bridge
            .liveness_inspections
            .iter()
            .chain(bridge.maintenance_recommendation.iter())
            .chain(bridge.maintenance_approval.iter())
        {
            check_artifact(artifact)?;
        }
    }
    bridge.maintenance_adoptions.extend(approved_maintenance(&bridge)?);
    validate_adoptions(&bridge)?;
    Ok(bridge)
}

#[derive(Deserialize)]
struct MaintenanceApproval {
    schema: String,
    status: String,
    legacy_claims_released: bool,
    gameplay_launch_authorized: bool,
    recommendation: Artifact,
    observation: Artifact,
    workers: Vec<ApprovedWorker>,
    #[serde(default)]
    previous_approval: Option<Artifact>,
}
#[derive(Deserialize)]
struct ApprovedWorker {
    adoption: String,
    session_id: String,
    writable_paths: Vec<PathBuf>,
}
#[derive(Deserialize)]
struct Recommendation {
    adoptions: Vec<RecommendedAdoption>,
}
#[derive(Deserialize)]
struct RecommendedAdoption {
    id: String,
    effort: String,
    original_hold_ids: Vec<String>,
    writable_paths: Vec<PathBuf>,
}

fn approval(bridge: &Bridge) -> Result<Option<(MaintenanceApproval, Recommendation)>> {
    let Some(artifact) = &bridge.maintenance_approval else {
        return Ok(None);
    };
    check_artifact(artifact)?;
    let approval: MaintenanceApproval = toml::from_str(&fs::read_to_string(&artifact.path)?)?;
    ensure!(
        approval.schema == "decomp-preparation-maintenance-approval-v1"
            && approval.status == "approved"
            && !approval.legacy_claims_released
            && !approval.gameplay_launch_authorized,
        "maintenance approval cannot release legacy work"
    );
    check_artifact(&approval.recommendation)?;
    check_artifact(&approval.observation)?;
    if let Some(previous) = &approval.previous_approval {
        check_artifact(previous)?;
    }
    let recommendation: Recommendation = toml::from_str(&fs::read_to_string(&approval.recommendation.path)?)?;
    Ok(Some((approval, recommendation)))
}

fn approved_maintenance(bridge: &Bridge) -> Result<Vec<MaintenanceAdoption>> {
    let Some((approval, recommendation)) = approval(bridge)? else {
        return Ok(Vec::new());
    };
    let mut result = Vec::new();
    for worker in approval.workers {
        if bridge.maintenance_adoptions.iter().any(|adoption| adoption.id == worker.adoption) {
            continue;
        }
        let recommended = recommendation
            .adoptions
            .iter()
            .find(|adoption| adoption.id == worker.adoption)
            .context("approval lacks original maintenance recommendation")?;
        result.push(MaintenanceAdoption {
            id: worker.adoption,
            holding_ids: recommended.original_hold_ids.clone(),
            effort: recommended.effort.clone(),
            session_id: worker.session_id,
            writable_paths: worker.writable_paths,
            observation: bridge.maintenance_approval.clone().unwrap(),
        });
    }
    Ok(result)
}

fn validate_adoptions(bridge: &Bridge) -> Result<()> {
    let checked = approval(bridge)?;
    let mut ids = BTreeSet::new();
    for adoption in &bridge.maintenance_adoptions {
        ensure!(
            safe_name(&adoption.id)
                && ids.insert(&adoption.id)
                && safe_name(&adoption.effort)
                && real_session_id(&adoption.session_id)
                && !adoption.writable_paths.is_empty()
                && adoption.writable_paths.iter().all(|path| relative(path)),
            "invalid exact maintenance adoption"
        );
        check_artifact(&adoption.observation)?;
        let (approval, recommendation) = checked.as_ref().context("maintenance adoption requires checked approval")?;
        ensure!(
            Some(&adoption.observation) == bridge.maintenance_approval.as_ref(),
            "maintenance adoption must name the checked approval seal"
        );
        let worker = approval
            .workers
            .iter()
            .find(|worker| worker.adoption == adoption.id)
            .context("maintenance worker not approved")?;
        let recommended = recommendation
            .adoptions
            .iter()
            .find(|recommended| recommended.id == adoption.id)
            .context("maintenance lineage is missing")?;
        ensure!(
            worker.session_id == adoption.session_id
                && worker.writable_paths == adoption.writable_paths
                && recommended.effort == adoption.effort
                && recommended.writable_paths == adoption.writable_paths,
            "maintenance adoption expands approved identity or paths"
        );
        ensure!(
            !adoption.holding_ids.is_empty()
                && adoption.holding_ids.iter().all(|id| bridge.holdings.iter().any(|holding| &holding.id == id)),
            "maintenance adoption names an unknown original holding"
        );
        for holding in &bridge.holdings {
            if adoption.writable_paths.iter().any(|path| holding.writable_paths.iter().any(|held| overlaps(path, held)))
            {
                ensure!(
                    adoption.holding_ids.contains(&holding.id),
                    "maintenance adoption omits overlapping original holding {}",
                    holding.id
                );
            }
        }
    }
    Ok(())
}

pub(super) fn overlaps(a: &Path, b: &Path) -> bool {
    a.starts_with(b) || b.starts_with(a)
}

pub(super) fn held_reason(
    policy: &Policy,
    bridge: &Bridge,
    effort: &str,
    job: &Job,
    session: Option<&SessionAuthority>,
) -> Option<String> {
    for holding in &bridge.holdings {
        // A continuation uses its original durable claim and retained submission.
        let same = holding.effort == effort
            && holding.writable_paths == job.writable_paths
            && (holding.claim.as_ref().is_some_and(|claim| claim.id == job.id && claim.revision == job.revision)
                || (holding.job_id.as_deref() == Some(job.id.as_str())
                    && holding.revision.as_deref() == Some(job.revision.as_str())));
        let submitted = job.kind == Kind::Integrate
            && holding.effort == effort
            && holding.claim.as_ref().is_some_and(|claim| {
                job.inputs.get("submission-id") == Some(&claim.id)
                    && job.inputs.get("submission-revision") == Some(&claim.revision)
                    && job.inputs.get("submission-attempt") == Some(&claim.attempt.to_string())
            });
        let related = job.kind == Kind::Integrate
            && holding.related_submission_hold.as_ref().is_some_and(|id| {
                bridge.holdings.iter().any(|parent| {
                    &parent.id == id
                        && parent.effort == effort
                        && parent.claim.as_ref().is_some_and(|claim| {
                            job.inputs.get("submission-id") == Some(&claim.id)
                                && job.inputs.get("submission-revision") == Some(&claim.revision)
                                && job.inputs.get("submission-attempt") == Some(&claim.attempt.to_string())
                        })
                })
            });
        if same || submitted || related {
            continue;
        }
        let ticket = holding.ticket.as_ref().is_some_and(|ticket| super::ticket_path(effort, job) == *ticket);
        let paths = job
            .writable_paths
            .iter()
            .filter(|path| holding.writable_paths.iter().any(|held| overlaps(path, held)))
            .collect::<Vec<_>>();
        if !ticket && paths.is_empty() {
            continue;
        }
        let maintenance = !ticket
            && session.is_some_and(|session| {
                bridge.maintenance_adoptions.iter().any(|adoption| {
                    adoption.effort == effort
                        && adoption.session_id == session.session_id
                        && adoption.holding_ids.contains(&holding.id)
                        && paths.iter().all(|path| adoption.writable_paths.contains(path))
                })
            });
        if !maintenance {
            return Some(format!(
                "{} held by retained legacy {} ({})",
                if ticket { "ticket ownership" } else { "writable ownership" },
                holding.id,
                holding.state
            ));
        }
    }
    let _ = policy;
    None
}

pub(super) fn validate_mutation(
    policy: &Policy,
    bridge: &Bridge,
    effort: &str,
    request: &super::Request,
    ledger: &super::Ledger,
) -> Result<()> {
    if let Some(update) = &request.update {
        let claim = match update {
            super::Update::Started { claim, .. }
            | super::Update::LaunchFailed { claim, .. }
            | super::Update::Finished { claim, .. }
            | super::Update::OwnerStopped { claim, .. } => claim,
        };
        if let Some(entry) = ledger.jobs.get(&claim.id) {
            ensure!(
                held_reason(policy, bridge, effort, &entry.job, request.authority.as_ref()).is_none(),
                "current mutation conflicts with retained legacy ownership"
            );
            if let super::Update::OwnerStopped { observation, .. } = update
                && let Some(holding) = bridge
                    .holdings
                    .iter()
                    .find(|holding| holding.effort == effort && holding.claim.as_ref() == Some(claim))
            {
                let proof: toml::Value = toml::from_str(&fs::read_to_string(&observation.path)?)?;
                // Stored display aliases in historical claims require the pinned
                // bridge's real owner thread, rather than a newly invented session.
                let bridge_value: toml::Value = toml::from_str(&fs::read_to_string(&policy.preparation.bridge.path)?)?;
                let original = bridge_value
                    .get("holdings")
                    .and_then(toml::Value::as_array)
                    .and_then(|holdings| {
                        holdings
                            .iter()
                            .find(|value| value.get("id").and_then(toml::Value::as_str) == Some(holding.id.as_str()))
                    })
                    .and_then(|value| value.get("owner_thread"))
                    .and_then(toml::Value::as_str)
                    .context("retained legacy stop lacks a real owner thread")?;
                ensure!(
                    proof.get("session_id").and_then(toml::Value::as_str) == Some(original),
                    "legacy stopped proof names another real owner thread"
                );
            }
        }
    }
    Ok(())
}

fn registration(registration: &RoleRegistration) -> Result<Role> {
    ensure!(safe_name(&registration.token), "invalid persistent session token");
    session_observation(&registration.observation, &registration.session_id)?;
    Ok(Role {
        token: registration.token.clone(),
        kind: registration.kind.clone(),
        session_id: registration.session_id.clone(),
        generation: 1,
        registration: registration.observation.clone(),
        transfers: Vec::new(),
        last_transfer: None,
    })
}

fn checked_transfer(root: &Path, policy: &Policy, transfer: &RoleTransfer, old: &Role) -> Result<()> {
    caller(&transfer.session_id)?;
    check_artifact(&transfer.observation)?;
    let observation: TransferObservation = toml::from_str(&fs::read_to_string(&transfer.observation.path)?)?;
    ensure!(
        observation.schema == "decomp-session-transfer-v1"
            && observation.token == old.token
            && observation.session_id == old.session_id
            && observation.generation == old.generation,
        "transfer observation names another authority generation or owner"
    );
    ensure!(
        stopped(&observation.status) && !observation.observed_at.is_empty(),
        "transfer does not establish stopped real owner"
    );
    artifacts(&observation.evidence)?;
    session_observation(&observation.replacement, &transfer.session_id)?;
    let ledgers = super::read_ledgers(root)?;
    let retained = bridge(policy, root, false)?;
    let mut matched = BTreeSet::new();
    for (effort, ledger) in &ledgers {
        for entry in ledger.jobs.values().filter(|entry| super::owns_paths(&entry.state)) {
            let Some(claim) = &entry.claim else {
                continue;
            };
            if claim.coordinator != old.token {
                continue;
            }
            let child = observation
                .children
                .iter()
                .find(|child| child.effort == *effort && child.claim == *claim)
                .context("transfer omits held child claim")?;
            ensure!(
                matched.insert((child.effort.clone(), child.claim.id.clone()))
                    && child.agent == entry.agent
                    && known_status(&child.status),
                "transfer child identity/status differs"
            );
            check_artifact(&child.observation)?;
            let proof: toml::Value = toml::from_str(&fs::read_to_string(&child.observation.path)?)?;
            ensure!(
                proof.get("schema").and_then(toml::Value::as_str) == Some("decomp-child-observation-v1")
                    && proof.get("effort").and_then(toml::Value::as_str) == Some(effort.as_str())
                    && proof.get("status").and_then(toml::Value::as_str) == Some(child.status.as_str())
                    && proof.get("claim").cloned().map(|value| value.try_into::<Claim>()).transpose()?.as_ref()
                        == Some(claim)
                    && proof.get("agent").and_then(toml::Value::as_str) == child.agent.as_deref()
                    && proof.get("session_id").and_then(toml::Value::as_str).is_some_and(real_session_id)
                    && proof.get("observed_at").and_then(toml::Value::as_str).is_some_and(|text| !text.is_empty()),
                "child stop/live proof names another owner"
            );
            let inspected: Vec<Artifact> =
                proof.get("evidence").cloned().context("child proof omits sealed liveness inspection")?.try_into()?;
            artifacts(&inspected)?;
            if let Some(agent) = child.agent.as_deref().filter(|agent| real_session_id(agent)) {
                ensure!(
                    proof.get("session_id").and_then(toml::Value::as_str) == Some(agent),
                    "child proof names another real child thread"
                );
            } else if let Some(holding) = retained
                .holdings
                .iter()
                .find(|holding| holding.effort == *effort && holding.claim.as_ref() == Some(claim))
            {
                let value: toml::Value = toml::from_str(&fs::read_to_string(&policy.preparation.bridge.path)?)?;
                let owner = value
                    .get("holdings")
                    .and_then(toml::Value::as_array)
                    .and_then(|holdings| {
                        holdings
                            .iter()
                            .find(|value| value.get("id").and_then(toml::Value::as_str) == Some(holding.id.as_str()))
                    })
                    .and_then(|value| value.get("owner_thread"))
                    .and_then(toml::Value::as_str)
                    .context("legacy child bridge lacks a real owner thread")?;
                ensure!(
                    proof.get("session_id").and_then(toml::Value::as_str) == Some(owner),
                    "legacy child proof names another real owner thread"
                );
            }
            if child.agent.is_some() {
                ensure!(child.status != "not-launched", "started child cannot be treated as not launched");
            }
        }
    }
    ensure!(matched.len() == observation.children.len(), "transfer includes an unknown or duplicate child claim");
    for process in &observation.processes {
        ensure!(
            process.pid > 0 && !process.argv.is_empty() && !process.started_at.is_empty(),
            "process inspection lacks exact start identity"
        );
        check_artifact(&process.observation)?;
        let proof: toml::Value = toml::from_str(&fs::read_to_string(&process.observation.path)?)?;
        ensure!(
            proof.get("schema").and_then(toml::Value::as_str) == Some("decomp-process-observation-v1")
                && proof.get("pid").and_then(toml::Value::as_integer) == Some(i64::from(process.pid))
                && proof.get("argv").cloned().map(|value| value.try_into::<Vec<String>>()).transpose()?.as_ref()
                    == Some(&process.argv)
                && proof.get("started_at").and_then(toml::Value::as_str) == Some(process.started_at.as_str())
                && proof.get("status").and_then(toml::Value::as_str) == Some(process.status.as_str())
                && proof.get("agent").and_then(toml::Value::as_str) == process.agent.as_deref(),
            "process stop/live proof names another process identity"
        );
        if live(&process.status) {
            ensure!(
                observation
                    .children
                    .iter()
                    .any(|child| live(&child.status) && child.agent == process.agent && process.agent.is_some()),
                "running process is not accounted to a surviving child"
            );
        } else {
            ensure!(
                ["exited", "stopped", "failed", "completed", "not-running"].contains(&process.status.as_str()),
                "process inspection is ambiguous"
            );
        }
    }
    let _ = policy;
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DrillObservation {
    pub schema: String,
    pub checkout: PathBuf,
    pub source_checkout: PathBuf,
    pub isolated_checkout: PathBuf,
    pub bridge: Artifact,
    pub head: String,
    pub source_fingerprint: String,
    pub controls: BTreeMap<String, bool>,
    pub evidence: Vec<Artifact>,
    #[serde(default)]
    pub execution: Option<Artifact>,
}

fn checked_drill(root: &Path, checkout: &Path, policy: &Policy, artifact: &Artifact) -> Result<()> {
    check_artifact(artifact)?;
    let drill: DrillObservation = toml::from_str(&fs::read_to_string(&artifact.path)?)?;
    ensure!(
        drill.schema == "decomp-concurrency-drill-v1"
            && fs::canonicalize(&drill.checkout)? == root
            && main_checkout(&drill.source_checkout)? == root
            && fs::canonicalize(&drill.source_checkout)? == fs::canonicalize(checkout)?,
        "drill names another authority/source checkout"
    );
    ensure!(drill.bridge == policy.preparation.bridge, "drill used another legacy bridge");
    ensure!(
        drill.head.len() == 40
            && git_commit(checkout, "HEAD")? == drill.head
            && super::source_fingerprint(checkout)? == drill.source_fingerprint,
        "drill source baseline changed"
    );
    for control in [
        "independent-sessions",
        "writable-overlap",
        "duplicate-ticket",
        "crash-restart",
        "surviving-child",
        "stopped-transfer",
        "stale-authority",
        "same-attempt-retry",
        "shared-capacity",
        "cleanup",
        "integration-success",
    ] {
        ensure!(drill.controls.get(control) == Some(&true), "drill control did not pass: {control}");
    }
    artifacts(&drill.evidence)?;
    drill_evidence::validate(root, checkout, &drill)
}

pub(crate) fn operate(request: Request) -> Result<Receipt> {
    ensure!(
        ["status", "prepare", "register", "transfer", "activate", "validate-activation"]
            .contains(&request.operation.as_str()),
        "unknown session authority operation"
    );
    let root = main_checkout(&request.checkout)?;
    let path = root.join(POLICY);
    let _lock = if request.operation == "status" { None } else { Some(super::mutation_lock(&root)?) };
    let mut policy = read(&root)?;
    match request.operation.as_str() {
        "status" => {}
        "prepare" => {
            let preparation = request.preparation.as_ref().context("preparation policy is missing")?;
            caller(&preparation.integrator.session_id)?;
            ensure!(
                preparation.integrator.kind == RoleKind::Integrator
                    && request.coordinator == preparation.integrator.token,
                "preparation must name its explicit integrator"
            );
            ensure!(
                safe_name(&preparation.canonical_effort) && (1..=64).contains(&preparation.capacity),
                "invalid canonical effort or shared capacity"
            );
            let mut efforts = BTreeSet::new();
            for effort in preparation.admitted_efforts.iter().chain(&preparation.drill_efforts) {
                ensure!(
                    safe_name(effort) && efforts.insert(effort) && effort != &preparation.canonical_effort,
                    "invalid or duplicate admitted effort"
                );
            }
            if let Some(existing) = &policy {
                role(existing, &request.coordinator, request.authority.as_ref())?;
                ensure!(existing.preparation == *preparation, "preparation policy differs from durable authority");
            } else {
                ensure!(
                    request
                        .authority
                        .as_ref()
                        .is_some_and(|credential| credential.session_id == preparation.integrator.session_id
                            && credential.generation == 1),
                    "initial preparation requires explicit generation one authority"
                );
                let integrator = registration(&preparation.integrator)?;
                let prepared = Policy {
                    schema: SCHEMA.into(),
                    state: "prepared".into(),
                    preparation: preparation.clone(),
                    roles: BTreeMap::from([(integrator.token.clone(), integrator)]),
                    drill: None,
                    prepared_head: git_commit(&request.checkout, "HEAD")?,
                };
                let retained = bridge(&prepared, &root, true)?;
                for (effort, ledger) in super::read_ledgers(&root)? {
                    ensure!(
                        preparation.admitted_efforts.contains(&effort),
                        "existing effort not admitted at preparation: {effort}"
                    );
                    ensure!(
                        ledger.capacity == preparation.capacity,
                        "existing effort capacity differs from shared capacity: {effort}"
                    );
                    if let Some(owner) = &ledger.integration_owner {
                        ensure!(
                            owner == &preparation.integrator.token,
                            "preparation must preserve existing persistent integrator token"
                        );
                    }
                    for entry in ledger.jobs.values().filter(|entry| super::owns_paths(&entry.state)) {
                        ensure!(
                            retained.holdings.iter().any(|holding| holding.effort == effort
                                && holding.claim.as_ref() == entry.claim.as_ref()
                                && holding.writable_paths == entry.job.writable_paths),
                            "legacy bridge omits exact held claim/path lineage {effort}/{}",
                            entry.job.id
                        );
                    }
                }
                if let Some(ledger) = &retained.ledger {
                    ensure!(
                        super::main_checkout(ledger.path.parent().context("legacy ledger has no parent")?)? == root,
                        "bridge ledger belongs to another repository"
                    );
                }
                for holding in retained.holdings.iter().filter(|holding| {
                    holding.claim.as_ref().is_some_and(|claim| claim.coordinator == preparation.integrator.token)
                        && ["reserved", "running", "submitted"].contains(&holding.state.as_str())
                }) {
                    // Initial preparation must not silently replace the first
                    // historical integrator using only possession of its token.
                    let value: toml::Value =
                        toml::from_str(&fs::read_to_string(&preparation.integrator.observation.path)?)?;
                    let evidence: Vec<Artifact> = value
                        .get("evidence")
                        .cloned()
                        .context("integrator observation lacks inspection")?
                        .try_into()?;
                    let owner_value: toml::Value = toml::from_str(&fs::read_to_string(&preparation.bridge.path)?)?;
                    let owner = owner_value
                        .get("holdings")
                        .and_then(toml::Value::as_array)
                        .and_then(|entries| {
                            entries.iter().find(|entry| {
                                entry.get("id").and_then(toml::Value::as_str) == Some(holding.id.as_str())
                            })
                        })
                        .and_then(|entry| entry.get("parent_thread").or_else(|| entry.get("owner_thread")))
                        .and_then(toml::Value::as_str)
                        .context("legacy integrator real owner is missing")?;
                    if owner != preparation.integrator.session_id {
                        let checked = evidence.iter().any(|artifact| {
                            fs::read_to_string(&artifact.path)
                                .ok()
                                .and_then(|text| toml::from_str::<TransferObservation>(&text).ok())
                                .is_some_and(|proof| {
                                    proof.token == preparation.integrator.token
                                        && proof.session_id == owner
                                        && proof.generation == 0
                                        && stopped(&proof.status)
                                        && proof.schema == "decomp-session-transfer-v1"
                                })
                        });
                        ensure!(
                            checked,
                            "initial legacy integrator registration requires checked stopped-owner transfer evidence"
                        );
                        let artifact = evidence
                            .iter()
                            .find(|artifact| {
                                fs::read_to_string(&artifact.path)
                                    .ok()
                                    .and_then(|text| toml::from_str::<TransferObservation>(&text).ok())
                                    .is_some_and(|proof| {
                                        proof.token == preparation.integrator.token
                                            && proof.session_id == owner
                                            && proof.generation == 0
                                    })
                            })
                            .unwrap();
                        let old = Role {
                            token: preparation.integrator.token.clone(),
                            kind: RoleKind::Integrator,
                            session_id: owner.to_owned(),
                            generation: 0,
                            registration: artifact.clone(),
                            transfers: Vec::new(),
                            last_transfer: None,
                        };
                        checked_transfer(
                            &root,
                            &prepared,
                            &RoleTransfer {
                                token: old.token.clone(),
                                session_id: preparation.integrator.session_id.clone(),
                                observation: artifact.clone(),
                            },
                            &old,
                        )?;
                    }
                }
                policy = Some(prepared);
            }
        }
        "register" => {
            let policy = policy.as_mut().context("session authority has not been prepared")?;
            ensure!(
                role(policy, &request.coordinator, request.authority.as_ref())?.kind == RoleKind::Integrator,
                "only the integrator can register sessions"
            );
            let requested = request.registration.as_ref().context("role registration is missing")?;
            ensure!(requested.kind == RoleKind::Dispatcher, "the integrator role already exists");
            let new = registration(requested)?;
            if let Some(old) = policy.roles.get(&requested.token) {
                ensure!(
                    old.generation == 1
                        && old.session_id == new.session_id
                        && old.kind == new.kind
                        && old.registration == new.registration,
                    "persistent token already belongs to another role/session"
                );
            } else {
                ensure!(
                    !policy
                        .roles
                        .values()
                        .any(|old| old.session_id == new.session_id && old.kind == RoleKind::Dispatcher),
                    "dispatcher session already has a persistent token"
                );
                policy.roles.insert(new.token.clone(), new);
            }
        }
        "transfer" => {
            let policy = policy.as_mut().context("session authority has not been prepared")?;
            let transfer = request.transfer.as_ref().context("role transfer is missing")?;
            let old = policy.roles.get(&transfer.token).context("unknown persistent transfer token")?.clone();
            ensure!(request.coordinator == transfer.token, "transfer must retain original persistent token");
            caller(&transfer.session_id)?;
            if old.last_transfer.as_ref() == Some(transfer) {
                role(policy, &request.coordinator, request.authority.as_ref())?;
                check_artifact(&transfer.observation)?;
            } else {
                ensure!(
                    request.authority.as_ref().is_some_and(|credential| credential.session_id == transfer.session_id
                        && credential.generation == old.generation + 1),
                    "replacement must request the next authority generation"
                );
                checked_transfer(&root, policy, transfer, &old)?;
                let current = policy.roles.get_mut(&transfer.token).unwrap();
                current.session_id = transfer.session_id.clone();
                current.generation += 1;
                current.transfers.push(transfer.observation.clone());
                current.last_transfer = Some(transfer.clone());
            }
        }
        "activate" | "validate-activation" => {
            let policy = policy.as_mut().context("session authority has not been prepared")?;
            ensure!(
                role(policy, &request.coordinator, request.authority.as_ref())?.kind == RoleKind::Integrator,
                "only the current integrator can activate authority"
            );
            let drill = request.drill.as_ref().context("activation requires a sealed recovery/concurrency drill")?;
            checked_drill(&root, &request.checkout, policy, drill)?;
            if policy.state == "active" {
                ensure!(policy.drill.as_ref() == Some(drill), "active cutover receipt differs");
            }
            if request.operation == "activate" {
                policy.state = "active".into();
                policy.drill = Some(drill.clone());
            }
        }
        _ => unreachable!(),
    }
    if !["status", "validate-activation"].contains(&request.operation.as_str()) {
        let policy = policy.as_ref().unwrap();
        atomic_write(&path, toml::to_string_pretty(policy)?.as_bytes())?;
    }
    Ok(Receipt {
        schema: SCHEMA.into(),
        status: policy.as_ref().map(|policy| policy.state.clone()).unwrap_or_else(|| "unprepared".into()),
        policy: path,
        canonical_effort: policy.as_ref().map(|policy| policy.preparation.canonical_effort.clone()),
        capacity: policy.as_ref().map(|policy| policy.preparation.capacity),
        roles: policy.as_ref().map(|policy| policy.roles.values().cloned().collect()).unwrap_or_default(),
        bridge: policy.as_ref().map(|policy| policy.preparation.bridge.clone()),
        drill: policy.and_then(|policy| policy.drill),
    })
}
