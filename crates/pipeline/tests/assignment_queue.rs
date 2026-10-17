//! Exercise shared ownership through the queue interface, including interrupted publication.
#[allow(dead_code)]
#[path = "../src/assignment.rs"]
mod assignment;
#[allow(dead_code)]
#[path = "../src/translation_migration_guard.rs"]
mod translation_migration_guard;
#[allow(dead_code)]
#[path = "../src/translation_production_registry.rs"]
mod translation_production_registry;
use assignment::*;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

fn digest(path: &Path) -> anyhow::Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "sm-assignment-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
        Self(root)
    }
    fn request(&self, operation: &str) -> Request {
        Request {
            checkout: self.0.clone(),
            operation: operation.into(),
            coordinator: "owner-a".into(),
            capacity: 2,
            available_slots: 2,
            ..Default::default()
        }
    }
    fn artifact(&self, name: &str, contents: &str) -> Artifact {
        let path = self.0.join(name);
        fs::write(&path, contents).unwrap();
        Artifact { sha256: digest(&path).unwrap(), path }
    }
    fn start(&self, job: Job) -> Claim {
        let mut request = self.request("next");
        request.jobs = vec![job];
        let claim = operate(request).unwrap().assignment.unwrap().claim;
        let mut request = self.request("record");
        request.update = Some(Update::Started { claim: claim.clone(), agent: "agent-1".into() });
        operate(request).unwrap();
        claim
    }
    fn record(&self, claim: &Claim, result: WorkResult) -> anyhow::Result<Receipt> {
        let mut request = self.request("record");
        request.update = Some(Update::Finished { claim: claim.clone(), result });
        operate(request)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn job(id: &str, kind: Kind) -> Job {
    let writable_paths = if kind == Kind::Implement { vec![PathBuf::from("crates/port/src/enemy.rs")] } else { vec![] };
    Job {
        id: id.into(),
        revision: "revision-1".into(),
        kind,
        subject: format!("Work {id}"),
        writable_paths,
        blockers: vec![],
        inputs: BTreeMap::new(),
        instructions: "Use declared evidence and scope.".into(),
        ticket: None,
    }
}

#[test]
fn plan_is_read_only_and_reservation_survives_restart() {
    let f = Fixture::new();
    let mut request = f.request("plan");
    request.jobs = vec![job("verify", Kind::Verify)];
    let preview = operate(request.clone()).unwrap();
    assert_eq!(preview.jobs.len(), 1);
    assert!(!f.0.join(".scratch").exists());
    request.operation = "next".into();
    let ready = operate(request).unwrap().assignment.unwrap();
    assert!(ready.ticket.exists());
    let resumed = operate(f.request("status")).unwrap();
    assert_eq!(resumed.recovery, vec![ready.claim]);
    assert_eq!(resumed.jobs[0].state, State::Reserved);
}

#[test]
fn two_coordinators_and_worktree_share_exclusive_reservation() {
    let f = Fixture::new();
    assert!(
        Command::new("git")
            .args([
                "-C",
                f.0.to_str().unwrap(),
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "initial"
            ])
            .status()
            .unwrap()
            .success()
    );
    let worktree = f.0.join("other");
    assert!(
        Command::new("git")
            .args(["-C", f.0.to_str().unwrap(), "worktree", "add", "-q", "--detach"])
            .arg(&worktree)
            .status()
            .unwrap()
            .success()
    );
    let mut a = f.request("next");
    a.capacity = 1;
    a.jobs = vec![job("verify", Kind::Verify)];
    let mut b = a.clone();
    b.checkout = worktree;
    b.coordinator = "owner-b".into();
    let (a, b) = std::thread::scope(|s| {
        let a = s.spawn(|| operate(a).unwrap());
        let b = s.spawn(|| operate(b).unwrap());
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_eq!(usize::from(a.assignment.is_some()) + usize::from(b.assignment.is_some()), 1);
    assert_eq!(a.ledger, b.ledger);
    let ready = a.assignment.or(b.assignment).unwrap();
    let mut steal = f.request("record");
    steal.capacity = 1;
    steal.coordinator = if ready.claim.coordinator == "owner-a" { "owner-b" } else { "owner-a" }.into();
    steal.update = Some(Update::Started { claim: ready.claim, agent: "thief".into() });
    assert!(operate(steal).unwrap_err().to_string().contains("another owner's"));
}

#[test]
fn existing_tickets_and_blockers_are_authoritative() {
    let f = Fixture::new();
    let issues = f.0.join(".scratch/work/issues");
    fs::create_dir_all(&issues).unwrap();
    fs::write(issues.join("01-first.md"), "# First\nStatus: claimed\nBlocked by: None\nTriage: ready-for-agent\n")
        .unwrap();
    fs::write(issues.join("02-second.md"), "# Second\nStatus: open\nBlocked by: 01\nTriage: ready-for-agent\n")
        .unwrap();
    let mut first = job("first", Kind::Research);
    first.ticket = Some(".scratch/work/issues/01-first.md".into());
    let mut second = job("second", Kind::Research);
    second.ticket = Some(".scratch/work/issues/02-second.md".into());
    let mut request = f.request("next");
    request.jobs = vec![first, second];
    let waiting = operate(request.clone()).unwrap();
    assert!(waiting.assignment.is_none());
    assert_eq!(waiting.deferred["first"], "ticket is claimed");
    assert!(waiting.deferred["second"].contains("blocked by 01"));
    assert!(!fs::read_to_string(issues.join("01-first.md")).unwrap().contains("decomp-assignment"));
    fs::write(issues.join("01-first.md"), "# First\nStatus: done\nBlocked by: None\nTriage: ready-for-agent\n")
        .unwrap();
    let ready = operate(request).unwrap().assignment.unwrap();
    assert_eq!(ready.job.id, "second");
    assert!(fs::read_to_string(issues.join("02-second.md")).unwrap().contains("Status: claimed"));
}

#[test]
fn stale_completion_cannot_expand_ownership_or_complete_revised_work() {
    let f = Fixture::new();
    let original = job("implement", Kind::Implement);
    let claim = f.start(original.clone());
    let mut changed = original;
    changed.revision = "revision-2".into();
    changed.writable_paths.push("crates/port/src/audio.rs".into());
    let mut request = f.request("next");
    request.jobs = vec![changed];
    let receipt = operate(request).unwrap();
    assert_eq!(receipt.jobs[0].job.writable_paths.len(), 1);
    assert_eq!(receipt.jobs[0].pending_job.as_ref().unwrap().writable_paths.len(), 2);
    let evidence = f.artifact("submission.toml", "status = 'submitted'\n");
    assert!(
        f.record(&claim, WorkResult::Submitted { commit: "a".repeat(40), evidence: vec![evidence] })
            .unwrap_err()
            .to_string()
            .contains("stale source")
    );
    let state = operate(f.request("status")).unwrap();
    assert_eq!(state.jobs[0].state, State::Running);
}

#[test]
fn recovery_requires_matching_stopped_observation_and_new_attempt() {
    let f = Fixture::new();
    let claim = f.start(job("diagnose", Kind::Diagnose));
    let mut request = f.request("record");
    let wrong = f.artifact("wrong.toml", &format!("agent = 'agent-1'\ncoordinator = 'owner-a'\nstatus = 'running'\n[claim]\nid = '{}'\nrevision = '{}'\nattempt = {}\ncoordinator = '{}'\n", claim.id, claim.revision, claim.attempt, claim.coordinator));
    request.update = Some(Update::OwnerStopped { claim: claim.clone(), observation: wrong });
    assert!(operate(request.clone()).unwrap_err().to_string().contains("stopped owner"));
    let valid = f.artifact("stopped.toml", &format!("agent = 'agent-1'\ncoordinator = 'owner-a'\nstatus = 'interrupted'\n[claim]\nid = '{}'\nrevision = '{}'\nattempt = {}\ncoordinator = '{}'\n", claim.id, claim.revision, claim.attempt, claim.coordinator));
    request.update = Some(Update::OwnerStopped { claim: claim.clone(), observation: valid });
    operate(request).unwrap();
    let next = operate(f.request("next")).unwrap().assignment.unwrap();
    assert_eq!(next.claim.attempt, claim.attempt + 1);
    let mut stale = f.request("record");
    stale.update = Some(Update::Started { claim, agent: "old-agent".into() });
    assert!(operate(stale).unwrap_err().to_string().contains("stale assignment"));
}

#[test]
fn promotion_path_conflicts_and_integration_preserve_submission_lifecycle() {
    let f = Fixture::new();
    let research = f.start(job("research", Kind::Research));
    let evidence = f.artifact("research.toml", "status = 'established'\n");
    let mut implementation = job("implement", Kind::Implement);
    implementation.inputs.insert("base-head".into(), "a".repeat(40));
    let result = WorkResult::Researched { evidence: vec![evidence.clone()], followups: vec![implementation.clone()] };
    f.record(&research, result.clone()).unwrap();
    f.record(&research, result).unwrap();
    let ready = operate(f.request("next")).unwrap().assignment.unwrap();
    assert_eq!(ready.job.id, "implement");
    assert!(ready.workspace.worktree.is_some());
    let mut request = f.request("record");
    request.update = Some(Update::Started { claim: ready.claim.clone(), agent: "agent-1".into() });
    operate(request).unwrap();
    let mut conflicting = job("other", Kind::Implement);
    conflicting.blockers.clear();
    let mut request = f.request("next");
    request.jobs = vec![conflicting];
    let waiting = operate(request).unwrap();
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["other"].contains("writable ownership"));
    f.record(&ready.claim, WorkResult::Submitted { commit: "b".repeat(40), evidence: vec![evidence.clone()] }).unwrap();
    let status = operate(f.request("status")).unwrap();
    assert_eq!(status.jobs.iter().find(|e| e.job.id == "implement").unwrap().state, State::Submitted);
    let integration = operate(f.request("next")).unwrap().assignment.unwrap();
    assert_eq!(integration.job.kind, Kind::Integrate);
    assert_eq!(integration.job.inputs.get("base-head"), implementation.inputs.get("base-head"));
    let mut request = f.request("record");
    request.update = Some(Update::Started { claim: integration.claim.clone(), agent: "agent-1".into() });
    operate(request).unwrap();
    let authentication = f.artifact("auth.toml", "status = 'authenticated'\n");
    let cleanup = f.artifact("cleanup.toml", "status = 'complete'\n");
    f.record(
        &integration.claim,
        WorkResult::Integrated { submissions: vec![ready.claim], evidence: vec![evidence], authentication, cleanup },
    )
    .unwrap();
    assert_eq!(
        operate(f.request("status")).unwrap().jobs.iter().find(|e| e.job.id == "implement").unwrap().state,
        State::Complete
    );
}

#[test]
fn interrupted_ticket_publication_keeps_claim_for_reconciliation() {
    let f = Fixture::new();
    let parent = f.0.join(".scratch/affected-family-assignment");
    fs::create_dir_all(&parent).unwrap();
    fs::write(parent.join("assignments"), "cannot create ticket directory").unwrap();
    let mut request = f.request("next");
    request.jobs = vec![job("verify", Kind::Verify)];
    assert!(operate(request).is_err());
    let status = operate(f.request("status")).unwrap();
    assert_eq!(status.jobs[0].state, State::Reserved);
    assert_eq!(status.recovery.len(), 1);
    fs::remove_file(parent.join("assignments")).unwrap();
    let repaired = operate(f.request("next")).unwrap();
    assert!(repaired.assignment.is_none());
    assert!(parent.join("assignments/verify.md").exists());
}

#[test]
fn input_changes_and_missing_file_inputs_remain_uncertainty() {
    let f = Fixture::new();
    let input = f.artifact("input.rs", "first");
    let mut candidate = job("verify", Kind::Verify);
    candidate.inputs.insert(format!("file:{}", input.path.display()), input.sha256);
    fs::write(&input.path, "changed").unwrap();
    let mut request = f.request("next");
    request.jobs = vec![candidate];
    let receipt = operate(request).unwrap();
    assert!(receipt.assignment.is_none());
    assert!(receipt.jobs[0].stale);
}

#[test]
fn no_slots_do_not_claim_and_exact_launch_failure_is_idempotent() {
    let f = Fixture::new();
    let mut request = f.request("next");
    request.jobs = vec![job("verify", Kind::Verify)];
    request.available_slots = 0;
    assert!(operate(request).unwrap().assignment.is_none());
    let ready = operate(f.request("next")).unwrap().assignment.unwrap();
    let mut failure = f.request("record");
    failure.update =
        Some(Update::LaunchFailed { claim: ready.claim.clone(), reason: "spawn tool rejected launch".into() });
    operate(failure.clone()).unwrap();
    operate(failure.clone()).unwrap();
    let next = operate(f.request("next")).unwrap().assignment.unwrap();
    assert_eq!(next.claim.attempt, ready.claim.attempt + 1);
    assert!(operate(failure).is_err());
}

#[test]
fn revised_completed_work_retains_old_evidence_and_rejects_replayed_completion() {
    let f = Fixture::new();
    let original = job("research", Kind::Research);
    let claim = f.start(original.clone());
    let evidence = f.artifact("research.toml", "status = 'complete'\n");
    let result = WorkResult::Researched { evidence: vec![evidence], followups: vec![] };
    f.record(&claim, result.clone()).unwrap();
    let mut revised = original;
    revised.revision = "new-revision".into();
    let mut request = f.request("next");
    request.jobs = vec![revised];
    let receipt = operate(request).unwrap();
    assert_eq!(receipt.jobs[0].history.len(), 1);
    assert!(receipt.jobs[0].history[0].result.is_some());
    assert!(f.record(&claim, result).is_err());
}

#[test]
fn researching_existing_task_transfers_claim_and_resolves_only_after_integration() {
    let f = Fixture::new();
    let relative = PathBuf::from(".scratch/gameplay/issues/01-translation.md");
    let ticket = f.0.join(&relative);
    fs::create_dir_all(ticket.parent().unwrap()).unwrap();
    fs::write(&ticket, "# Translate gameplay\nStatus: open\nBlocked by: None\nTriage: ready-for-agent\nType: task\n")
        .unwrap();
    let mut research = job("research-task", Kind::Research);
    research.ticket = Some(relative.clone());
    research.inputs.insert("origin-ticket-type".into(), "task".into());
    let claim = f.start(research);
    let evidence = f.artifact("scope.toml", "status = 'established'\n");
    let implementation = job("implement-task", Kind::Implement);
    let result = WorkResult::Researched { evidence: vec![evidence.clone()], followups: vec![implementation] };
    f.record(&claim, result.clone()).unwrap();
    f.record(&claim, result).unwrap();
    assert!(fs::read_to_string(&ticket).unwrap().contains("Status: claimed"));
    let ready = operate(f.request("next")).unwrap().assignment.unwrap();
    assert_eq!(ready.job.kind, Kind::Implement);
    assert_eq!(ready.job.ticket, Some(relative));
    let mut start = f.request("record");
    start.update = Some(Update::Started { claim: ready.claim.clone(), agent: "agent-1".into() });
    operate(start).unwrap();
    f.record(&ready.claim, WorkResult::Submitted { commit: "c".repeat(40), evidence: vec![evidence.clone()] }).unwrap();
    assert!(fs::read_to_string(&ticket).unwrap().contains("Status: claimed"));
    let integration = operate(f.request("next")).unwrap().assignment.unwrap();
    assert_eq!(integration.job.inputs.get("base-head"), Some(&"c".repeat(40)));
    let mut start = f.request("record");
    start.update = Some(Update::Started { claim: integration.claim.clone(), agent: "agent-1".into() });
    operate(start).unwrap();
    let auth = f.artifact("auth.toml", "status = 'authenticated'\n");
    let cleanup = f.artifact("cleanup.toml", "status = 'complete'\n");
    f.record(
        &integration.claim,
        WorkResult::Integrated {
            submissions: vec![ready.claim],
            evidence: vec![evidence],
            authentication: auth,
            cleanup,
        },
    )
    .unwrap();
    assert!(fs::read_to_string(&ticket).unwrap().contains("Status: resolved"));
}

#[test]
fn incomplete_scope_research_never_resolves_originating_task() {
    let f = Fixture::new();
    let relative = PathBuf::from(".scratch/gameplay/issues/01-translation.md");
    let ticket = f.0.join(&relative);
    fs::create_dir_all(ticket.parent().unwrap()).unwrap();
    fs::write(&ticket, "# Translate gameplay\nStatus: open\nBlocked by: None\nTriage: ready-for-agent\nType: task\n")
        .unwrap();
    let mut research = job("research-task", Kind::Research);
    research.ticket = Some(relative);
    research.inputs.insert("origin-ticket-type".into(), "task".into());
    let claim = f.start(research);
    let evidence = f.artifact("research.toml", "status = 'missing-exercise'\n");
    f.record(&claim, WorkResult::Researched { evidence: vec![evidence], followups: vec![] }).unwrap();
    assert!(fs::read_to_string(&ticket).unwrap().contains("Status: open"));
}

#[test]
fn owned_source_edits_do_not_invalidate_submission_but_immutable_inputs_do() {
    let f = Fixture::new();
    let owned = f.0.join("crates/port/src/enemy.rs");
    fs::create_dir_all(owned.parent().unwrap()).unwrap();
    fs::write(&owned, "before").unwrap();
    let immutable = f.artifact("oracle.c", "fixed");
    let mut implementation = job("implement", Kind::Implement);
    implementation.inputs.insert("file:crates/port/src/enemy.rs".into(), digest(&owned).unwrap());
    implementation.inputs.insert(format!("file:{}", immutable.path.display()), immutable.sha256.clone());
    let claim = f.start(implementation);
    fs::write(&owned, "after").unwrap();
    let evidence = f.artifact("submission.toml", "status = 'submitted'\n");
    f.record(&claim, WorkResult::Submitted { commit: "d".repeat(40), evidence: vec![evidence] }).unwrap();
    let status = operate(f.request("status")).unwrap();
    assert!(!status.jobs.iter().find(|entry| entry.job.id == "implement").unwrap().stale);
    fs::write(&immutable.path, "changed").unwrap();
    let status = operate(f.request("status")).unwrap();
    assert!(status.jobs.iter().find(|entry| entry.job.id == "implement").unwrap().stale);
    assert!(operate(f.request("next")).unwrap().assignment.is_none());
}

#[test]
fn empty_affected_frontier_retires_unclaimed_work_without_granting_verification() {
    let f = Fixture::new();
    let mut pending = job("verify-frontier", Kind::Verify);
    pending.inputs.insert("families".into(), "enemy\naudio".into());
    let mut request = f.request("next");
    request.available_slots = 0;
    request.jobs = vec![pending.clone()];
    operate(request).unwrap();
    pending.revision = "no-affected-families".into();
    pending.inputs.insert("families".into(), String::new());
    let mut request = f.request("plan");
    request.jobs = vec![pending.clone()];
    let preview = operate(request).unwrap();
    assert_eq!(preview.status, "complete");
    assert_eq!(preview.jobs[0].state, State::NotRequired);
    assert!(preview.jobs[0].result.is_none());
    assert_eq!(operate(f.request("status")).unwrap().jobs[0].state, State::Ready);
    let mut request = f.request("next");
    request.jobs = vec![pending];
    let retired = operate(request).unwrap();
    assert_eq!(retired.status, "complete");
    assert!(retired.assignment.is_none());
    assert!(retired.jobs[0].claim.is_none());
    assert_eq!(retired.jobs[0].history.len(), 1);
}

#[test]
fn empty_frontier_preserves_active_old_claim_until_checked_recovery() {
    let f = Fixture::new();
    let mut pending = job("verify-frontier", Kind::Verify);
    pending.inputs.insert("families".into(), "enemy".into());
    let claim = f.start(pending.clone());
    pending.revision = "empty-frontier".into();
    pending.inputs.insert("families".into(), String::new());
    let mut request = f.request("next");
    request.jobs = vec![pending];
    let state = operate(request).unwrap();
    assert_eq!(state.jobs[0].state, State::Running);
    assert_eq!(state.recovery, vec![claim.clone()]);
    let stopped = f.artifact("stopped.toml", &format!("agent = 'agent-1'\ncoordinator = 'owner-a'\nstatus = 'interrupted'\n[claim]\nid = '{}'\nrevision = '{}'\nattempt = {}\ncoordinator = '{}'\n", claim.id, claim.revision, claim.attempt, claim.coordinator));
    let mut request = f.request("record");
    request.update = Some(Update::OwnerStopped { claim, observation: stopped });
    let recovered = operate(request).unwrap();
    assert_eq!(recovered.status, "complete");
    assert_eq!(recovered.jobs[0].state, State::NotRequired);
    assert!(recovered.jobs[0].result.is_none());
}

#[test]
fn saved_verification_checks_discovery_namespace_even_without_new_seeds() {
    let f = Fixture::new();
    let source = f.0.join("crates/port/src");
    fs::create_dir_all(&source).unwrap();
    fs::write(source.join("enemy.rs"), "before").unwrap();
    let mut candidate = job("verify-frontier", Kind::Verify);
    candidate.inputs.insert("source-fingerprint".into(), source_fingerprint(&f.0).unwrap());
    let mut request = f.request("next");
    request.available_slots = 0;
    request.jobs = vec![candidate];
    operate(request).unwrap();
    fs::write(source.join("new_module.rs"), "pub fn leaf() {}\n").unwrap();
    let saved = operate(f.request("next")).unwrap();
    assert!(saved.assignment.is_none());
    assert!(saved.jobs[0].stale);
}

#[test]
fn no_work_frontier_does_not_satisfy_a_verification_blocker() {
    let f = Fixture::new();
    let mut no_work = job("verify-frontier", Kind::Verify);
    no_work.inputs.insert("families".into(), String::new());
    let mut dependent = job("dependent", Kind::Research);
    dependent.blockers.push(no_work.id.clone());
    let mut request = f.request("next");
    request.jobs = vec![no_work, dependent];
    let waiting = operate(request).unwrap();
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["dependent"].contains("blocked by verify-frontier"));
}

#[test]
fn pull_order_honors_local_ticket_numbers_and_observed_candidate_ranking() {
    let f = Fixture::new();
    let mut first = job("z-earlier-ticket", Kind::Research);
    let mut later = job("a-later-ticket", Kind::Research);
    for (candidate, number) in [(&mut first, 1), (&mut later, 2)] {
        let path = PathBuf::from(format!(".scratch/work/issues/{number:02}-task.md"));
        fs::create_dir_all(f.0.join(path.parent().unwrap())).unwrap();
        fs::write(
            f.0.join(&path),
            "# Local work\nStatus: open\nBlocked by: None\nTriage: ready-for-agent\nType: research\n",
        )
        .unwrap();
        candidate.ticket = Some(path);
    }
    let mut observed = job("0-new-candidate", Kind::Research);
    observed.inputs.insert("candidate-rank".into(), "0".into());
    let mut request = f.request("next");
    request.jobs = vec![later, observed, first];
    assert_eq!(operate(request).unwrap().assignment.unwrap().job.id, "z-earlier-ticket");
    assert_eq!(operate(f.request("next")).unwrap().assignment.unwrap().job.id, "a-later-ticket");

    let other = Fixture::new();
    let mut preferred = job("z-preferred-observed", Kind::Research);
    preferred.inputs.insert("candidate-rank".into(), "0".into());
    let mut deferred = job("a-later-observed", Kind::Research);
    deferred.inputs.insert("candidate-rank".into(), "1".into());
    let mut request = other.request("next");
    request.jobs = vec![deferred, preferred];
    assert_eq!(operate(request).unwrap().assignment.unwrap().job.id, "z-preferred-observed");
}

#[test]
fn concurrent_efforts_in_main_and_worktree_cannot_overlap_ancestor_paths() {
    let f = Fixture::new();
    assert!(
        Command::new("git")
            .args([
                "-C",
                f.0.to_str().unwrap(),
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.invalid",
                "commit",
                "--allow-empty",
                "-qm",
                "initial"
            ])
            .status()
            .unwrap()
            .success()
    );
    let worktree = f.0.join("other");
    assert!(
        Command::new("git")
            .args(["-C", f.0.to_str().unwrap(), "worktree", "add", "-q", "--detach"])
            .arg(&worktree)
            .status()
            .unwrap()
            .success()
    );
    let mut a = f.request("next");
    a.effort = "effort-a".into();
    let mut parent = job("parent", Kind::Implement);
    parent.writable_paths = vec!["crates/port/src".into()];
    a.jobs = vec![parent];
    let mut b = f.request("next");
    b.checkout = worktree;
    b.effort = "effort-b".into();
    b.coordinator = "owner-b".into();
    b.jobs = vec![job("child", Kind::Implement)];
    let (a, b) = std::thread::scope(|s| {
        let a = s.spawn(|| operate(a).unwrap());
        let b = s.spawn(|| operate(b).unwrap());
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_eq!(usize::from(a.assignment.is_some()) + usize::from(b.assignment.is_some()), 1);
    let deferred = if a.assignment.is_none() { a } else { b };
    assert!(deferred.deferred.values().any(|reason| reason.contains("writable ownership held by effort-")));
}

#[test]
fn separate_efforts_keep_submitted_paths_and_duplicate_ticket_identity_held() {
    let f = Fixture::new();
    let mut a = f.request("next");
    a.effort = "effort-a".into();
    a.jobs = vec![job("implement", Kind::Implement)];
    let claim = operate(a).unwrap().assignment.unwrap().claim;
    let mut record = f.request("record");
    record.effort = "effort-a".into();
    record.update = Some(Update::Started { claim: claim.clone(), agent: "agent-1".into() });
    operate(record.clone()).unwrap();
    let evidence = f.artifact("evidence.toml", "verified = true\n");
    record.update = Some(Update::Finished {
        claim: claim.clone(),
        result: WorkResult::Submitted { commit: "e".repeat(40), evidence: vec![evidence] },
    });
    operate(record.clone()).unwrap();
    let mut b = f.request("next");
    b.effort = "effort-b".into();
    b.jobs = vec![job("overlap", Kind::Implement)];
    let waiting = operate(b.clone()).unwrap();
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["overlap"].contains("effort-a/implement"));
    let stopped = f.artifact("stopped-submitted.toml", &format!("agent = 'agent-1'\ncoordinator = 'owner-a'\nstatus = 'completed'\n[claim]\nid = '{}'\nrevision = '{}'\nattempt = {}\ncoordinator = '{}'\n", claim.id, claim.revision, claim.attempt, claim.coordinator));
    record.update = Some(Update::OwnerStopped { claim, observation: stopped });
    assert!(operate(record).unwrap_err().to_string().contains("submitted ownership remains held"));

    let ticket = PathBuf::from(".scratch/effort-a/assignments/implement.md");
    // Simulate a stale Markdown copy; durable identity still blocks it.
    fs::write(f.0.join(&ticket), "# Copy\nStatus: open\nBlocked by: None\nTriage: ready-for-agent\n").unwrap();
    let mut duplicate = job("duplicate-ticket", Kind::Research);
    duplicate.ticket = Some(ticket);
    b.jobs = vec![duplicate];
    let waiting = operate(b).unwrap();
    assert!(waiting.assignment.is_none());
    assert!(waiting.deferred["duplicate-ticket"].contains("ticket ownership held by effort-a/implement"));
}

#[test]
fn canonical_mutation_cannot_start_without_prepared_authority() {
    let f = Fixture::new();
    let mut request = f.request("next");
    request.effort = "decomp-parallel".into();
    request.jobs = vec![job("implementation", Kind::Implement)];
    assert!(operate(request).unwrap_err().to_string().contains("has not been prepared"));
    assert!(!f.0.join(".scratch/decomp-parallel/ledger.toml").exists());
}

#[test]
fn completion_retry_rechecks_the_same_attempt_sealed_evidence() {
    let f = Fixture::new();
    let claim = f.start(job("research", Kind::Research));
    let evidence = f.artifact("result.toml", "sealed = true\n");
    let result = WorkResult::Researched { evidence: vec![evidence.clone()], followups: Vec::new() };
    f.record(&claim, result.clone()).unwrap();
    f.record(&claim, result.clone()).unwrap();
    fs::write(evidence.path, "sealed = false\n").unwrap();
    assert!(f.record(&claim, result).unwrap_err().to_string().contains("artifact identity differs"));
}

#[test]
fn symlink_alias_cannot_bypass_repository_writable_ownership() {
    let f = Fixture::new();
    fs::create_dir_all(f.0.join("crates/port/src")).unwrap();
    std::os::unix::fs::symlink("crates/port/src", f.0.join("alias")).unwrap();
    let mut request = f.request("next");
    let mut candidate = job("alias", Kind::Implement);
    candidate.writable_paths = vec!["alias/enemy.rs".into()];
    request.jobs = vec![candidate];
    assert!(operate(request).unwrap_err().to_string().contains("crosses a symlink"));
}
