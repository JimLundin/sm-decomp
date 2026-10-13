//! Durable worker recovery through typed requests in disposable repositories.
#[allow(dead_code)]
#[path = "../src/assignment.rs"]
mod assignment;
#[allow(dead_code)]
#[path = "../src/session.rs"]
mod session;
use assignment::{Artifact, Job, Kind, SessionAuthority, Update, WorkResult};
use session::{Cleanup, Handoff, NextCommand};
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
const WORKER: &str = "11111111-2222-4333-8444-555555555555";
fn caller() -> String {
    std::env::var("CODEX_THREAD_ID").expect("tests require the observed caller thread")
}
struct Fixture {
    root: PathBuf,
    effort: String,
    worktree: Option<PathBuf>,
}
impl Fixture {
    fn new() -> Self {
        let identity = format!("{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed));
        let root = std::env::temp_dir().join(format!("sm-session-test-{identity}"));
        fs::create_dir_all(root.join("crates/pipeline/src")).unwrap();
        fs::write(root.join("crates/pipeline/src/owned.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
        fs::write(root.join("other.txt"), "unowned\n").unwrap();
        fs::write(root.join(".gitignore"), ".scratch/\n*.toml\n").unwrap();
        fs::write(root.join("Cargo.toml"), "[package]\nname='session-fixture'\nversion='0.1.0'\nedition='2024'\n[lib]\npath='crates/pipeline/src/owned.rs'\n").unwrap();
        fs::write(root.join("Cargo.lock"), "version = 4\n[[package]]\nname = 'session-fixture'\nversion = '0.1.0'\n")
            .unwrap();
        let f = Self { root, effort: format!("session-drill-{identity}"), worktree: None };
        f.git(&["init", "-q"]);
        f.git(&["config", "user.name", "Session Test"]);
        f.git(&["config", "user.email", "session@example.invalid"]);
        f.git(&["add", "-f", "Cargo.toml", "Cargo.lock"]);
        f.git(&["add", "."]);
        f.git(&["commit", "-qm", "initial"]);
        f
    }
    fn git(&self, argv: &[&str]) -> String {
        git(&self.root, argv)
    }
    fn queue(&self, operation: &str) -> assignment::Request {
        assignment::Request {
            operation: operation.into(),
            checkout: self.root.clone(),
            effort: self.effort.clone(),
            coordinator: "dispatcher".into(),
            capacity: 3,
            available_slots: 1,
            authority: Some(SessionAuthority { session_id: caller(), generation: 1 }),
            ..Default::default()
        }
    }
    fn seal(&self, name: &str, contents: &str) -> Artifact {
        let path = self.root.join(name);
        fs::write(&path, contents).unwrap();
        Artifact { sha256: digest(&path).unwrap(), path }
    }
    fn job(&self) -> Job {
        Job {
            id: "owned".into(),
            revision: "revision-one".into(),
            kind: Kind::Implement,
            subject: "Disposable infrastructure".into(),
            writable_paths: vec!["crates/pipeline/src/owned.rs".into()],
            blockers: vec![],
            inputs: BTreeMap::from([("base-head".into(), self.git(&["rev-parse", "HEAD"]))]),
            instructions: "Change only owned.rs; run the reviewed test gate.".into(),
            ticket: None,
        }
    }
    fn reserve(&mut self) -> Handoff {
        // Registration helper is synchronized with the engine's typed contract.
        prepare_authority(self);
        let mut queue = self.queue("next");
        queue.jobs = vec![self.job()];
        let assigned = assignment::operate(queue).unwrap().assignment.unwrap();
        let worktree = assigned.workspace.worktree.unwrap();
        let branch = assigned.workspace.branch.unwrap();
        self.git(&[
            "worktree",
            "add",
            "-q",
            "-b",
            &branch,
            worktree.to_str().unwrap(),
            &assigned.job.inputs["base-head"],
        ]);
        self.worktree = Some(worktree.clone());
        Handoff {
            schema: "decomp-worker-handoff-v1".into(),
            sequence: 1,
            session_id: caller(),
            worker_session_id: WORKER.into(),
            agent: "/root/actual_worker".into(),
            authority_generation: 1,
            effort: self.effort.clone(),
            claim: assigned.claim,
            baseline: assigned.job.inputs["base-head"].clone(),
            source_commit: assigned.job.inputs["base-head"].clone(),
            job: assigned.job,
            branch: Some(branch),
            worktree: worktree.clone(),
            submitted_commit: None,
            runs: vec![],
            processes: vec![],
            evidence: vec![],
            dependencies: vec![],
            stages: vec![],
            limitations: vec!["Disposable tools test; no gameplay acceptance.".into()],
            cleanup: Cleanup {
                status: "pending".into(),
                reclaimed_bytes: 0,
                receipts: vec![],
                retained_dependencies: vec![],
                remaining_action: Some("finish gate and cleanup".into()),
            },
            next: NextCommand {
                cwd: worktree,
                argv: vec!["cargo".into(), "test".into(), "--locked".into(), "--offline".into()],
                env: BTreeMap::from([("SM_PIPELINE_WORKERS".into(), "4".into())]),
                timeout_seconds: 60,
            },
        }
    }
    fn start(&self, handoff: &Handoff) -> toml::Value {
        session::operate(session::Request::Start { queue: self.queue("record"), handoff: Box::new(handoff.clone()) })
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(worktree) = &self.worktree {
            let _ = Command::new("git")
                .args(["worktree", "remove", "--force"])
                .arg(worktree)
                .current_dir(&self.root)
                .status();
        }
        fs::remove_dir_all(&self.root).unwrap();
    }
}
fn git(root: &Path, argv: &[&str]) -> String {
    let output = Command::new("git").args(argv).current_dir(root).output().unwrap();
    assert!(output.status.success(), "git {argv:?}: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn prepare_authority(f: &Fixture) {
    let bridge = f.seal("bridge.toml", "schema='decomp-legacy-owner-bridge-v1'\nholdings=[]\n");
    prepare_with_bridge(f, bridge);
}
fn prepare_with_bridge(f: &Fixture, bridge: Artifact) {
    let inspection = f.seal("inspection.toml", "status='test-only-observed-running'\n");
    let observation = f.seal(
        "session-observation.toml",
        &toml::to_string(&assignment::authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: caller(),
            status: "running".into(),
            observed_at: "2026-10-13T09:00:00Z".into(),
            evidence: vec![inspection],
        })
        .unwrap(),
    );
    assignment::authority::operate(assignment::authority::Request {
        operation: "prepare".into(),
        checkout: f.root.clone(),
        coordinator: "dispatcher".into(),
        authority: f.queue("status").authority,
        preparation: Some(assignment::authority::Preparation {
            canonical_effort: "production-pending".into(),
            capacity: 3,
            admitted_efforts: vec![],
            drill_efforts: vec![f.effort.clone()],
            bridge,
            integrator: assignment::authority::RoleRegistration {
                token: "dispatcher".into(),
                kind: assignment::authority::RoleKind::Integrator,
                session_id: caller(),
                observation,
            },
        }),
        ..Default::default()
    })
    .unwrap();
}
fn path(value: &toml::Value) -> PathBuf {
    value["handoff"].as_str().unwrap().into()
}

#[test]
fn main_and_assigned_worktree_inspect_same_queue_and_start_retry_repairs_markdown() {
    let mut f = Fixture::new();
    let handoff = f.reserve();
    let pointer = f.start(&handoff);
    let mut queue = f.queue("status");
    let main = session::operate(session::Request::Inspect { queue: queue.clone(), handoff: None }).unwrap();
    queue.checkout = handoff.worktree.clone();
    let worker = session::operate(session::Request::Inspect { queue, handoff: None }).unwrap();
    assert_eq!(main["ledger"], worker["ledger"]);
    fs::remove_file(path(&pointer).with_extension("md")).unwrap();
    assert_eq!(f.start(&handoff), pointer);
    assert!(path(&pointer).with_extension("md").exists());
    let saved: Handoff = toml::from_str(&fs::read_to_string(path(&pointer)).unwrap()).unwrap();
    assert_eq!(saved, handoff);
    let mut conflict = handoff;
    conflict.limitations.push("conflicting same sequence".into());
    assert!(
        session::operate(session::Request::Start { queue: f.queue("record"), handoff: Box::new(conflict) })
            .unwrap_err()
            .to_string()
            .contains("conflicting handoff")
    );
}

#[test]
fn checkpoint_commits_only_owned_files_and_retry_reconstructs_exact_next_command() {
    let mut f = Fixture::new();
    let mut handoff = f.reserve();
    f.start(&handoff);
    handoff.sequence = 2;
    fs::write(handoff.worktree.join("crates/pipeline/src/owned.rs"), "pub fn value() -> u8 { 2 }\n").unwrap();
    let checkpoint = || {
        session::operate(session::Request::Checkpoint {
            queue: f.queue("record"),
            handoff: Box::new(handoff.clone()),
            message: "WIP owned change".into(),
        })
    };
    let first = checkpoint().unwrap();
    let commit = first["source_commit"].as_str().unwrap().to_owned();
    assert_eq!(commit.len(), 40);
    assert_ne!(commit, handoff.source_commit);
    assert_eq!(checkpoint().unwrap(), first);
    assert_eq!(git(&handoff.worktree, &["rev-list", "--count", "HEAD"]), "2");
    let resumed =
        session::operate(session::Request::Resume { queue: f.queue("status"), handoff: path(&first) }).unwrap();
    assert_eq!(resumed["next"], first["next"]);
    assert_eq!(resumed["source_commit"].as_str(), Some(commit.as_str()));
    assert!(path(&first).with_file_name("owned-1-sequence-1.toml").exists());
}

#[test]
fn unapproved_staged_file_is_rejected_before_index_or_commit_writes() {
    let mut f = Fixture::new();
    let mut handoff = f.reserve();
    f.start(&handoff);
    handoff.sequence = 2;
    fs::write(handoff.worktree.join("other.txt"), "unapproved changed\n").unwrap();
    git(&handoff.worktree, &["add", "other.txt"]);
    let index = git(&handoff.worktree, &["write-tree"]);
    let error = session::operate(session::Request::Checkpoint {
        queue: f.queue("record"),
        handoff: Box::new(handoff.clone()),
        message: "WIP".into(),
    })
    .unwrap_err();
    assert!(error.to_string().contains("exceeds approved files"));
    assert_eq!(git(&handoff.worktree, &["write-tree"]), index);
    assert_eq!(git(&handoff.worktree, &["rev-parse", "HEAD"]), handoff.source_commit);
}

#[test]
fn interrupted_commit_before_handoff_publication_is_recovered_without_another_commit() {
    let mut f = Fixture::new();
    let mut handoff = f.reserve();
    let original = f.start(&handoff);
    handoff.sequence = 2;
    let intent = toml::Value::try_from(BTreeMap::from([
        ("schema", toml::Value::String("decomp-checkpoint-intent-v1".into())),
        ("requested", toml::Value::try_from(&handoff).unwrap()),
        ("message", toml::Value::String("WIP recovered".into())),
    ]))
    .unwrap();
    fs::write(path(&original).with_file_name("owned-1-checkpoint-2.toml"), toml::to_string_pretty(&intent).unwrap())
        .unwrap();
    fs::write(handoff.worktree.join("crates/pipeline/src/owned.rs"), "pub fn value() -> u8 { 3 }\n").unwrap();
    git(&handoff.worktree, &["add", "crates/pipeline/src/owned.rs"]);
    git(&handoff.worktree, &["commit", "-qm", "WIP recovered"]);
    let completed = git(&handoff.worktree, &["rev-parse", "HEAD"]);
    let recovered = session::operate(session::Request::Checkpoint {
        queue: f.queue("record"),
        handoff: Box::new(handoff.clone()),
        message: "WIP recovered".into(),
    })
    .unwrap();
    assert_eq!(recovered["source_commit"].as_str(), Some(completed.as_str()));
    assert_eq!(git(&handoff.worktree, &["rev-list", "--count", "HEAD"]), "2");
}

#[test]
fn changed_evidence_or_attempt_rejects_resume_without_releasing_ownership() {
    let mut f = Fixture::new();
    let mut handoff = f.reserve();
    handoff.evidence = vec![f.seal("evidence.toml", "status='retained'\n")];
    let first = f.start(&handoff);
    fs::write(&handoff.evidence[0].path, "changed\n").unwrap();
    assert!(
        session::operate(session::Request::Resume { queue: f.queue("status"), handoff: path(&first) })
            .unwrap_err()
            .to_string()
            .contains("artifact changed")
    );
    let mut stopped = f.queue("record");
    let proof = f.seal("stopped-inspection.toml", "status='test-only-observed-stopped'\n");
    let observation = f.seal("stopped.toml", &format!("agent='{WORKER}'\ncoordinator='dispatcher'\nstatus='interrupted'\nsession_id='{WORKER}'\nobserved_at='2026-10-13T09:00:00Z'\n[[evidence]]\npath='{}'\nsha256='{}'\n[claim]\nid='{}'\nrevision='{}'\nattempt={}\ncoordinator='dispatcher'\n", proof.path.display(), proof.sha256, handoff.claim.id, handoff.claim.revision, handoff.claim.attempt));
    stopped.update = Some(Update::OwnerStopped { claim: handoff.claim.clone(), observation });
    assignment::operate(stopped).unwrap();
    let replacement = assignment::operate(f.queue("next")).unwrap().assignment.unwrap();
    assert_eq!(replacement.claim.attempt, 2);
    assert!(
        session::operate(session::Request::Resume { queue: f.queue("status"), handoff: path(&first) })
            .unwrap_err()
            .to_string()
            .contains("stale claim")
    );
}

#[test]
fn handoff_cannot_escape_shared_directory_or_resume_unreviewed_command() {
    let mut f = Fixture::new();
    let mut handoff = f.reserve();
    handoff.next.argv = vec!["sh".into(), "-c".into(), "touch outside".into()];
    assert!(
        session::operate(session::Request::Start { queue: f.queue("record"), handoff: Box::new(handoff) })
            .unwrap_err()
            .to_string()
            .contains("reviewed cargo test")
    );
    let external = f.seal("external.toml", "schema='not-a-handoff'\n");
    assert!(
        session::operate(session::Request::Inspect { queue: f.queue("status"), handoff: Some(external.path) })
            .unwrap_err()
            .to_string()
            .contains("shared session directory")
    );
}

#[test]
fn infrastructure_plan_is_read_only_and_rejects_native_scope_or_raw_job() {
    let f = Fixture::new();
    let scope = session::InfrastructureScope {
        schema: "decomp-infrastructure-scope-v1".into(),
        baseline: f.git(&["rev-parse", "HEAD"]),
        job: f.job(),
        gates: vec![
            session::Gate {
                argv: vec!["cargo".into(), "test".into(), "--locked".into(), "--offline".into()],
                timeout_seconds: 60,
                integration: false,
            },
            session::Gate {
                argv: vec!["cargo".into(), "test".into(), "--locked".into(), "--offline".into()],
                timeout_seconds: 60,
                integration: true,
            },
        ],
        limitations: vec!["No gameplay admission.".into()],
    };
    let seal = f.seal("scope.toml", &toml::to_string_pretty(&scope).unwrap());
    let planned = session::operate(session::Request::InfrastructurePlan {
        queue: f.queue("plan"),
        admission: session::Admission::Infrastructure { scopes: vec![seal.clone()] },
    })
    .unwrap();
    assert_eq!(planned["status"].as_str(), Some("plan"));
    assert!(!f.root.join(".scratch").exists());
    let mut raw = f.queue("plan");
    raw.jobs = vec![f.job()];
    assert!(
        session::operate(session::Request::InfrastructurePlan {
            queue: raw,
            admission: session::Admission::Infrastructure { scopes: vec![seal] }
        })
        .unwrap_err()
        .to_string()
        .contains("sealed reviewed scopes")
    );
    let mut native = scope;
    native.job.writable_paths = vec!["crates/port/src/gameplay.rs".into()];
    let seal = f.seal("native.toml", &toml::to_string_pretty(&native).unwrap());
    assert!(
        session::operate(session::Request::InfrastructurePlan {
            queue: f.queue("plan"),
            admission: session::Admission::Infrastructure { scopes: vec![seal] }
        })
        .unwrap_err()
        .to_string()
        .contains("pipeline tools")
    );
}

fn reviewed(f: &Fixture) -> (session::InfrastructureScope, Artifact) {
    let scope = session::InfrastructureScope {
        schema: "decomp-infrastructure-scope-v1".into(),
        baseline: f.git(&["rev-parse", "HEAD"]),
        job: f.job(),
        gates: [false, true]
            .into_iter()
            .map(|integration| session::Gate {
                argv: vec!["cargo".into(), "test".into(), "--locked".into(), "--offline".into()],
                timeout_seconds: 60,
                integration,
            })
            .collect(),
        limitations: vec!["Disposable test; no gameplay acceptance.".into()],
    };
    let seal = f.seal("scope.toml", &toml::to_string_pretty(&scope).unwrap());
    (scope, seal)
}
fn infrastructure_record(f: &Fixture, seal: &Artifact, update: Update) -> anyhow::Result<toml::Value> {
    let mut queue = f.queue("record");
    queue.update = Some(update);
    session::operate(session::Request::InfrastructureRecord {
        queue,
        admission: session::Admission::Infrastructure { scopes: vec![seal.clone()] },
    })
}
fn reserve_reviewed(f: &mut Fixture, seal: &Artifact) -> assignment::Assignment {
    prepare_authority(f);
    session::operate(session::Request::InfrastructureNext {
        queue: f.queue("next"),
        admission: session::Admission::Infrastructure { scopes: vec![seal.clone()] },
    })
    .unwrap();
    let status = assignment::operate(f.queue("status")).unwrap();
    let entry = status.jobs.iter().find(|entry| entry.job.id == "owned").unwrap();
    let claim = entry.claim.clone().unwrap();
    let branch = format!("{}-{}-{}", f.effort, claim.id, claim.attempt);
    let worktree = std::env::temp_dir().join(format!("sm-{branch}"));
    f.git(&["worktree", "add", "-q", "-b", &branch, worktree.to_str().unwrap(), &entry.job.inputs["base-head"]]);
    f.worktree = Some(worktree.clone());
    assignment::Assignment {
        claim,
        job: entry.job.clone(),
        ticket: PathBuf::new(),
        experiment_seconds: None,
        workspace: assignment::Workspace { worktree: Some(worktree), branch: Some(branch), batch: None },
    }
}

#[test]
fn reviewed_submission_runs_nonzero_gate_and_recovers_result_publication_retry() {
    let mut f = Fixture::new();
    let (_, seal) = reviewed(&f);
    let assigned = reserve_reviewed(&mut f, &seal);
    let claim = assigned.claim;
    infrastructure_record(&f, &seal, Update::Started { claim: claim.clone(), agent: WORKER.into() }).unwrap();
    let worktree = assigned.workspace.worktree.unwrap();
    fs::write(
        worktree.join("crates/pipeline/src/owned.rs"),
        "pub fn value() -> u8 { 2 }\n#[test] fn owned_gate() { assert_eq!(value(), 2); }\n",
    )
    .unwrap();
    git(&worktree, &["add", "crates/pipeline/src/owned.rs"]);
    git(&worktree, &["commit", "-qm", "scoped implementation"]);
    let commit = git(&worktree, &["rev-parse", "HEAD"]);
    let caller_evidence = f.seal("submission.toml", "status='diagnostic-source-submission'\n");
    let update = Update::Finished {
        claim: claim.clone(),
        result: WorkResult::Submitted { commit, evidence: vec![caller_evidence] },
    };
    infrastructure_record(&f, &seal, update.clone()).unwrap();
    let publication = f.root.join(format!(".scratch/{}/receipts/owned-1-infrastructure-result.toml", f.effort));
    let saved: toml::Value = toml::from_str(&fs::read_to_string(&publication).unwrap()).unwrap();
    let admitted: Update = saved["admitted"].clone().try_into().unwrap();
    let ledger_path = f.root.join(format!(".scratch/{}/ledger.toml", f.effort));
    let ledger: toml::Value = toml::from_str(&fs::read_to_string(&ledger_path).unwrap()).unwrap();
    let before = fs::read(&publication).unwrap();
    infrastructure_record(&f, &seal, update.clone()).unwrap();
    assert_eq!(fs::read(&publication).unwrap(), before);
    assert_eq!(toml::from_str::<toml::Value>(&fs::read_to_string(&ledger_path).unwrap()).unwrap(), ledger);
    let Update::Finished { result: WorkResult::Submitted { evidence, .. }, .. } = admitted else {
        panic!("wrong admitted result")
    };
    assert_eq!(evidence.len(), 4);
    let gate: toml::Value = toml::from_str(&fs::read_to_string(&evidence[1].path).unwrap()).unwrap();
    assert_eq!(gate["exit_code"].as_integer(), Some(0));
    assert!(!evidence[1].path.parent().unwrap().join("target").exists());
    fs::write(&evidence[2].path, "tampered gate stdout\n").unwrap();
    assert!(infrastructure_record(&f, &seal, update).unwrap_err().to_string().contains("artifact changed"));
}

#[test]
fn failed_or_started_native_job_cannot_use_infrastructure_record() {
    let mut f = Fixture::new();
    let handoff = f.reserve();
    let (_, seal) = reviewed(&f);
    let result = Update::Finished {
        claim: handoff.claim.clone(),
        result: WorkResult::Failed { error: "test refusal".into(), evidence: vec![], followups: vec![] },
    };
    assert!(
        infrastructure_record(&f, &seal, result)
            .unwrap_err()
            .to_string()
            .contains("exact reviewed infrastructure scope")
    );
    assert!(
        infrastructure_record(&f, &seal, Update::Started { claim: handoff.claim, agent: WORKER.into() })
            .unwrap_err()
            .to_string()
            .contains("exact reviewed infrastructure scope")
    );
}

#[test]
fn legacy_adapter_whitelists_exact_request_rows_not_receipts_or_nested_artifacts() {
    let f = Fixture::new();
    let mut original = f.queue("plan");
    original.authority = None;
    original.jobs = vec![f.job()];
    let request = f.seal("original-request.toml", &toml::to_string_pretty(&original).unwrap());
    let unrelated = f.seal("unrelated-request.toml", &toml::to_string_pretty(&original).unwrap());
    let receipt = f.seal("receipt.toml", &toml::to_string_pretty(&original).unwrap());
    let bridge = f.seal("bridge.toml", &format!("schema='decomp-legacy-owner-bridge-v1'\nholdings=[]\n[nested]\npath='{}'\nsha256='{}'\n[[legacy_request_receipt_artifacts]]\npath='{}'\nsha256='{}'\ndomain='legacy-infrastructure-plan-request'\naccepted_as_native_gameplay=false\n[[legacy_request_receipt_artifacts]]\npath='{}'\nsha256='{}'\ndomain='legacy-record-receipt'\naccepted_as_native_gameplay=false\n", unrelated.path.display(), unrelated.sha256, request.path.display(), request.sha256, receipt.path.display(), receipt.sha256));
    prepare_with_bridge(&f, bridge);
    let mut queue = original;
    queue.authority = f.queue("plan").authority;
    let legacy = |seal: &Artifact, queue: assignment::Request| {
        session::operate(session::Request::InfrastructurePlan {
            queue,
            admission: session::Admission::Legacy { original_request: seal.clone() },
        })
    };
    legacy(&request, queue.clone()).unwrap();
    assert!(legacy(&unrelated, queue.clone()).unwrap_err().to_string().contains("not listed as a request"));
    assert!(legacy(&receipt, queue.clone()).unwrap_err().to_string().contains("not listed as a request"));
    queue.jobs[0].instructions.push_str(" changed");
    assert!(legacy(&request, queue).unwrap_err().to_string().contains("differs from its original seal"));
}

#[test]
fn start_recovers_running_owner_when_publication_was_interrupted() {
    let mut f = Fixture::new();
    let handoff = f.reserve();
    let mut queue = f.queue("record");
    queue.update = Some(Update::Started { claim: handoff.claim.clone(), agent: WORKER.into() });
    assignment::operate(queue).unwrap();
    let path = f.root.join(format!(".scratch/decomp-parallel/sessions/{}/owned-1.toml", caller()));
    assert!(!path.exists());
    f.start(&handoff);
    assert!(path.exists());
    assert_eq!(assignment::operate(f.queue("status")).unwrap().jobs[0].agent.as_deref(), Some(WORKER));
}

#[test]
fn stale_handoff_generation_requires_checked_adoption() {
    let mut f = Fixture::new();
    let handoff = f.reserve();
    let pointer = f.start(&handoff);
    let path = path(&pointer);
    let mut stale = handoff;
    stale.authority_generation = 99;
    fs::write(&path, toml::to_string_pretty(&stale).unwrap()).unwrap();
    assert!(
        session::operate(session::Request::Resume { queue: f.queue("status"), handoff: path })
            .unwrap_err()
            .to_string()
            .contains("checked adopt")
    );
}

#[test]
fn infrastructure_cycle_integrates_frozen_source_after_retiring_worker() {
    let mut f = Fixture::new();
    let (_, seal) = reviewed(&f);
    let assigned = reserve_reviewed(&mut f, &seal);
    let submission = assigned.claim;
    infrastructure_record(&f, &seal, Update::Started { claim: submission.clone(), agent: WORKER.into() }).unwrap();
    let worktree = assigned.workspace.worktree.unwrap();
    fs::write(
        worktree.join("crates/pipeline/src/owned.rs"),
        "pub fn value() -> u8 { 2 }\n#[test] fn owned_gate() { assert_eq!(value(), 2); }\n",
    )
    .unwrap();
    git(&worktree, &["add", "crates/pipeline/src/owned.rs"]);
    git(&worktree, &["commit", "-qm", "scoped integration candidate"]);
    let commit = git(&worktree, &["rev-parse", "HEAD"]);
    infrastructure_record(
        &f,
        &seal,
        Update::Finished {
            claim: submission.clone(),
            result: WorkResult::Submitted {
                commit: commit.clone(),
                evidence: vec![f.seal("submission.toml", "status='source-only'\n")],
            },
        },
    )
    .unwrap();
    // Generated integration jobs must support the same durable worker route
    // before merging source, rather than requiring a raw started update.
    let assignment = assignment::operate(f.queue("next")).unwrap().assignment.unwrap();
    let integration = assignment.claim.clone();
    let baseline = assignment
        .job
        .inputs
        .get("base-head")
        .expect("generated integration retains the exact source baseline")
        .clone();
    let handoff = Handoff {
        schema: "decomp-worker-handoff-v1".into(),
        sequence: 1,
        session_id: caller(),
        worker_session_id: WORKER.into(),
        agent: "/root/integrator".into(),
        authority_generation: 1,
        effort: f.effort.clone(),
        claim: integration.clone(),
        job: assignment.job,
        baseline,
        branch: None,
        worktree: f.root.clone(),
        source_commit: f.git(&["rev-parse", "HEAD"]),
        submitted_commit: Some(commit.clone()),
        runs: vec![],
        processes: vec![],
        evidence: vec![],
        dependencies: vec![seal.clone()],
        stages: vec![],
        limitations: vec!["Disposable integration tools control; no gameplay.".into()],
        cleanup: Cleanup {
            status: "pending".into(),
            reclaimed_bytes: 0,
            receipts: vec![],
            retained_dependencies: vec![],
            remaining_action: Some("retire worker and authenticate integration".into()),
        },
        next: NextCommand {
            cwd: f.root.clone(),
            argv: vec!["cargo".into(), "test".into(), "--locked".into(), "--offline".into()],
            env: BTreeMap::from([("SM_PIPELINE_WORKERS".into(), "4".into())]),
            timeout_seconds: 60,
        },
    };
    let started =
        session::operate(session::Request::Start { queue: f.queue("record"), handoff: Box::new(handoff.clone()) })
            .unwrap();
    assert_eq!(
        session::operate(session::Request::Start { queue: f.queue("record"), handoff: Box::new(handoff) }).unwrap(),
        started
    );
    let resumed =
        session::operate(session::Request::Resume { queue: f.queue("status"), handoff: path(&started) }).unwrap();
    assert_eq!(resumed["claim"], started["claim"]);
    assert_eq!(resumed["next"], started["next"]);
    assert_eq!(resumed["source_commit"], started["source_commit"]);
    f.git(&["merge", "--ff-only", &commit]);
    // Runtime authority and receipts are deliberately untracked in this repository.
    // They must not invalidate frozen integration source; untracked source does.
    fs::write(f.root.join(".gitignore"), "*.toml\n").unwrap();
    f.git(&["add", ".gitignore"]);
    f.git(&["commit", "-qm", "allow visible untracked runtime receipts"]);
    f.git(&["worktree", "remove", worktree.to_str().unwrap()]);
    f.worktree = None;
    let cleanup = f.seal(
        "cleanup.toml",
        &toml::to_string_pretty(&Cleanup {
            status: "complete".into(),
            reclaimed_bytes: 0,
            receipts: vec![],
            retained_dependencies: vec![],
            remaining_action: None,
        })
        .unwrap(),
    );
    let update = Update::Finished {
        claim: integration,
        result: WorkResult::Integrated {
            submissions: vec![submission],
            evidence: vec![f.seal("merged.toml", "status='frozen'\n")],
            authentication: f.seal("caller-auth.toml", "status='pending-native-admission'\n"),
            cleanup,
        },
    };
    fs::write(f.root.join("crates/pipeline/src/untracked.rs"), "// untracked source\n").unwrap();
    assert!(infrastructure_record(&f, &seal, update.clone()).unwrap_err().to_string().contains("frozen and clean"));
    fs::remove_file(f.root.join("crates/pipeline/src/untracked.rs")).unwrap();
    infrastructure_record(&f, &seal, update.clone()).unwrap();
    infrastructure_record(&f, &seal, update).unwrap();
    let status = assignment::operate(f.queue("status")).unwrap();
    assert!(status.jobs.iter().all(|entry| entry.state == assignment::State::Complete));
    let original = f.root.join(format!(".scratch/{}/receipts/owned-1-infrastructure-result.toml", f.effort));
    let publication: toml::Value = toml::from_str(&fs::read_to_string(original).unwrap()).unwrap();
    let submitted: Update = publication["requested"].clone().try_into().unwrap();
    infrastructure_record(&f, &seal, submitted).unwrap();
}

#[test]
fn adopted_handoff_retains_exact_surviving_child_and_original_authority_history() {
    let mut f = Fixture::new();
    let mut handoff = f.reserve();
    let old_session = "aaaaaaaa-bbbb-4ccc-8ddd-eeeeeeeeeeee";
    handoff.session_id = old_session.into();
    let old_path = f.root.join(format!(".scratch/decomp-parallel/sessions/{old_session}/owned-1.toml"));
    fs::create_dir_all(old_path.parent().unwrap()).unwrap();
    fs::write(&old_path, toml::to_string_pretty(&handoff).unwrap()).unwrap();
    let mut started = f.queue("record");
    started.update = Some(Update::Started { claim: handoff.claim.clone(), agent: WORKER.into() });
    assignment::operate(started).unwrap();
    let evidence = f.seal("old-stop-inspection.toml", "status='test-only-parent-stopped-child-running'\n");
    let child = f.seal("child-live.toml", &format!("schema='decomp-child-observation-v1'\neffort='{}'\nagent='{WORKER}'\nsession_id='{WORKER}'\nstatus='running'\nobserved_at='2026-10-13T09:00:00Z'\n[[evidence]]\npath='{}'\nsha256='{}'\n[claim]\nid='{}'\nrevision='{}'\nattempt={}\ncoordinator='dispatcher'\n", f.effort, evidence.path.display(), evidence.sha256, handoff.claim.id, handoff.claim.revision, handoff.claim.attempt));
    let replacement = f.seal(
        "replacement-live.toml",
        &toml::to_string(&assignment::authority::SessionObservation {
            schema: "decomp-session-observation-v1".into(),
            session_id: caller(),
            status: "running".into(),
            observed_at: "2026-10-13T09:00:00Z".into(),
            evidence: vec![evidence.clone()],
        })
        .unwrap(),
    );
    let transfer = f.seal(
        "transfer.toml",
        &toml::to_string(&assignment::authority::TransferObservation {
            schema: "decomp-session-transfer-v1".into(),
            token: "dispatcher".into(),
            session_id: old_session.into(),
            generation: 1,
            status: "interrupted".into(),
            observed_at: "2026-10-13T09:00:00Z".into(),
            evidence: vec![evidence],
            replacement,
            children: vec![assignment::authority::ChildObservation {
                effort: f.effort.clone(),
                claim: handoff.claim.clone(),
                agent: Some(WORKER.into()),
                status: "running".into(),
                observation: child,
            }],
            processes: vec![],
        })
        .unwrap(),
    );
    // Model an observed original role so the real engine transfer performs its
    // complete held-child validation in this disposable repository.
    let policy_path = f.root.join(".scratch/decomp-parallel/authority.toml");
    let mut policy: toml::Value = toml::from_str(&fs::read_to_string(&policy_path).unwrap()).unwrap();
    policy["roles"]["dispatcher"]["session_id"] = toml::Value::String(old_session.into());
    fs::write(&policy_path, toml::to_string_pretty(&policy).unwrap()).unwrap();
    let mut queue = f.queue("status");
    queue.authority.as_mut().unwrap().generation = 2;
    assignment::authority::operate(assignment::authority::Request {
        operation: "transfer".into(),
        checkout: f.root.clone(),
        coordinator: "dispatcher".into(),
        authority: queue.authority.clone(),
        transfer: Some(assignment::authority::RoleTransfer {
            token: "dispatcher".into(),
            session_id: caller(),
            observation: transfer.clone(),
        }),
        ..Default::default()
    })
    .unwrap();
    let before = fs::read(&old_path).unwrap();
    assert!(
        session::operate(session::Request::Resume { queue: queue.clone(), handoff: old_path.clone() })
            .unwrap_err()
            .to_string()
            .contains("checked adopt")
    );
    let adopted = session::operate(session::Request::Adopt {
        queue: queue.clone(),
        handoff: old_path.clone(),
        transfer: transfer.clone(),
    })
    .unwrap();
    assert_eq!(
        session::operate(session::Request::Adopt { queue: queue.clone(), handoff: old_path.clone(), transfer })
            .unwrap(),
        adopted
    );
    assert_eq!(fs::read(&old_path).unwrap(), before);
    let saved: Handoff = toml::from_str(&fs::read_to_string(path(&adopted)).unwrap()).unwrap();
    assert_eq!(saved.claim, handoff.claim);
    assert_eq!(saved.worker_session_id, WORKER);
    assert_eq!(saved.authority_generation, 2);
    assert_eq!(saved.sequence, 2);
    session::operate(session::Request::Resume { queue, handoff: path(&adopted) }).unwrap();
}

#[test]
fn cli_inspection_is_read_only_and_rejects_mismatched_operation() {
    let f = Fixture::new();
    let request = f.seal(
        "inspect.toml",
        &toml::to_string_pretty(&session::Request::Inspect { queue: f.queue("status"), handoff: None }).unwrap(),
    );
    let invoke = |operation: &str| {
        Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["decomp-session", operation, "--request"])
            .arg(&request.path)
            .current_dir(&f.root)
            .output()
            .unwrap()
    };
    let result = invoke("inspect");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let receipt: toml::Value = toml::from_str(&String::from_utf8(result.stdout).unwrap()).unwrap();
    assert_eq!(
        receipt["ledger"].as_str(),
        Some(f.root.join(format!(".scratch/{}/ledger.toml", f.effort)).to_str().unwrap())
    );
    assert!(!f.root.join(".scratch").exists());
    let result = invoke("start");
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("operation differs"));
}
