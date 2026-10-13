use anyhow::{Result, ensure};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use translation_execution::*;

static NEXT: AtomicU64 = AtomicU64::new(0);

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "translation-execution-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn source(content: &str) -> SourceSnapshot {
    let files_sha256 = BTreeMap::from([("crates/port/src/owned.rs".into(), sha256(content))]);
    SourceSnapshot { commit: "a".repeat(40), sha256: sealed(&files_sha256).unwrap(), files_sha256, clean: true }
}
fn policy() -> FrozenPolicy {
    FrozenPolicy {
        schema: "translation-execution-policy-v1".into(),
        version: 1,
        scope_id: "coherent-test-scope".into(),
        claim: ClaimLineage {
            id: "scope".into(),
            revision: "revision".into(),
            attempt: 1,
            coordinator: "integrator".into(),
        },
        baseline_commit: "a".repeat(40),
        pinned_c_sha256: sha256("C"),
        dependencies_sha256: sha256("dependencies"),
        verification_sha256: sha256("fixed-gates"),
        adapter_sha256: sha256("recorded-adapter"),
        approved_files: source("old").files_sha256,
        verification_stages: vec![VerificationStage::Offline],
        budgets: Budgets { workers: 4, max_attempts: 3, max_evidence_bytes: 1024, gate_timeout_seconds: 5 },
        rewrite_limits: RewriteLimits {
            max_context_bytes: 4096,
            max_response_bytes: 8192,
            max_file_bytes: 1024,
            max_total_bytes: 4096,
        },
    }
}
fn seal(context: &OperationContext, source: &str) -> OutcomeSeal {
    OutcomeSeal {
        operation_key: context.operation_key.clone(),
        policy_sha256: context.policy_sha256.clone(),
        source_sha256: source.into(),
        evidence_sha256: sha256("authenticated-local-evidence"),
    }
}

#[derive(Default)]
struct Observed {
    calls: BTreeMap<String, usize>,
    produced: BTreeMap<String, GateOutcome>,
    fail_gate: bool,
    interrupt_once: bool,
    dirty_integration: bool,
    tamper_seal: bool,
    incomplete_cleanup: bool,
    interrupted_finalization: bool,
    retained_tampered: bool,
}
#[derive(Clone)]
struct Adapter(Arc<Mutex<Observed>>);
impl Adapter {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(Observed::default())))
    }
    fn call(&self, name: &str) {
        *self.0.lock().unwrap().calls.entry(name.into()).or_default() += 1;
    }
    fn count(&self, name: &str) -> usize {
        *self.0.lock().unwrap().calls.get(name).unwrap_or(&0)
    }
}
impl ExecutionAdapter for Adapter {
    fn identity(&self) -> String {
        sha256("recorded-adapter")
    }
    fn prepare(
        &mut self,
        ctx: &OperationContext,
        _: &FrozenPolicy,
        previous: Option<&SourceSnapshot>,
    ) -> Result<Preparation> {
        self.call("prepare");
        let source = previous.cloned().unwrap_or_else(|| source("old"));
        let content = if previous.is_some() { "new" } else { "old" };
        let project = Project::new();
        let path = "crates/port/src/owned.rs";
        fs::create_dir_all(project.0.join("crates/port/src"))?;
        fs::write(project.0.join(path), content)?;
        let capsule = translation_execution::rewrite::capture(
            &project.0,
            CapsuleBinding {
                run_id: ctx.run_id.clone(),
                attempt: ctx.attempt,
                policy_sha256: ctx.policy_sha256.clone(),
                source_sha256: source.sha256.clone(),
            },
            &[path.into()],
            "Pinned C and dependency contract".into(),
            policy().rewrite_limits,
        )?;
        Ok(Preparation { source, capsule })
    }
    fn apply(
        &mut self,
        _: &OperationContext,
        _: &FrozenPolicy,
        _: &RewriteCapsule,
        files: &SourceReplacements,
    ) -> Result<SourceSnapshot> {
        self.call("apply");
        Ok(source(&files.files[0].content))
    }
    fn verify(
        &mut self,
        ctx: &OperationContext,
        _: &FrozenPolicy,
        source: &SourceSnapshot,
        _: VerificationStage,
    ) -> Result<GateOutcome> {
        self.call("verify");
        let mut observed = self.0.lock().unwrap();
        if let Some(result) = observed.produced.get(&ctx.operation_key) {
            return Ok(result.clone());
        }
        let mut outcome = GateOutcome {
            seal: seal(ctx, &source.sha256),
            passed: !observed.fail_gate,
            diagnostic: if observed.fail_gate { "first mismatch".into() } else { String::new() },
            evidence_bytes: 64,
        };
        if observed.tamper_seal {
            outcome.seal.source_sha256 = sha256("other-source")
        }
        observed.produced.insert(ctx.operation_key.clone(), outcome.clone());
        if observed.interrupt_once {
            observed.interrupt_once = false;
            return Err(ExecutionPending { reason: "lost response after authenticated production".into() }.into());
        }
        Ok(outcome)
    }
    fn integrate(
        &mut self,
        _: &OperationContext,
        _: &FrozenPolicy,
        candidate: &SourceSnapshot,
    ) -> Result<SourceSnapshot> {
        self.call("integrate");
        let mut source = candidate.clone();
        source.commit = "b".repeat(40);
        source.clean = !self.0.lock().unwrap().dirty_integration;
        Ok(source)
    }
    fn finalize(
        &mut self,
        ctx: &OperationContext,
        _: &FrozenPolicy,
        request: &FinalizationRequest,
    ) -> Result<Finalization> {
        self.call("finalize");
        let mut observed = self.0.lock().unwrap();
        observed.interrupted_finalization = request.interrupted_operation.is_some();
        ensure!(!observed.incomplete_cleanup, "selected cleanup pending");
        let source = request.source.as_ref().map(|source| source.sha256.as_str()).unwrap_or(&ctx.policy_sha256);
        Ok(Finalization {
            seal: seal(ctx, source),
            cleanup_complete: true,
            retained_authenticatable: true,
            worker_retired: true,
            retained_bytes: 128,
            reclaimed_bytes: 1024,
        })
    }
    fn publish(&mut self, ctx: &OperationContext, _: &FrozenPolicy, completion: &Completion) -> Result<Publication> {
        self.call("publish");
        Ok(Publication {
            seal: seal(ctx, &completion.integrated.sha256),
            observation_ids: vec![format!("{}-integrated", ctx.operation_key)],
        })
    }
    fn authenticate(&self, _: &FrozenPolicy, _: &FinalizationRequest, _: &Finalization) -> Result<()> {
        ensure!(!self.0.lock().unwrap().retained_tampered, "retained evidence changed");
        Ok(())
    }
}
fn supervisor(path: &Path, adapter: Adapter) -> Supervisor<Adapter> {
    Supervisor::open(path, policy(), adapter).unwrap()
}
fn replacement(view: &View, content: &str) -> SourceReplacements {
    let State::Rewrite { capsule } = &view.state else { panic!("expected rewrite capsule") };
    SourceReplacements {
        capsule_id: capsule.id.clone(),
        files: vec![Replacement {
            path: capsule.files[0].path.clone(),
            base_sha256: capsule.files[0].sha256.clone(),
            content: content.into(),
        }],
    }
}

#[test]
fn successful_submission_and_lost_response_are_one_evaluation() {
    let project = Project::new();
    let adapter = Adapter::new();
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    let edits = replacement(&view, "new");
    assert!(matches!(run.submit(edits.clone()).unwrap().state, State::Complete { .. }));
    assert_eq!(adapter.count("verify"), 2);
    assert_eq!(adapter.count("apply"), 1);
    drop(run);
    let mut resumed = supervisor(&project.0, adapter.clone());
    assert!(matches!(resumed.submit(edits).unwrap().state, State::Complete { .. }));
    assert!(matches!(resumed.advance().unwrap().state, State::Complete { .. }));
    assert_eq!(adapter.count("verify"), 2);
    assert_eq!(adapter.count("publish"), 1);
}

#[test]
fn failed_gate_resume_does_no_work_and_explicit_retry_has_new_identity() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    let edits = replacement(&view, "new");
    assert!(matches!(
        run.submit(edits.clone()).unwrap().state,
        State::Stopped { disposition: Disposition::Failed, .. }
    ));
    for _ in 0..3 {
        run.advance().unwrap();
        run.submit(edits.clone()).unwrap();
    }
    assert_eq!(adapter.count("verify"), 1);
    assert_eq!(adapter.count("finalize"), 1);
    adapter.0.lock().unwrap().fail_gate = false;
    let done = run.retry("operator verified harness transient").unwrap();
    assert_eq!(done.attempt, 2);
    assert!(matches!(done.state, State::Complete { .. }));
    assert_eq!(adapter.count("verify"), 3);
    assert_eq!(adapter.count("apply"), 1);
}

#[test]
fn ambiguous_finished_gate_recovers_its_existing_effect() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().interrupt_once = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    let paused = run.submit(replacement(&view, "new")).unwrap();
    assert!(matches!(paused.state, State::Verifying { .. }));
    assert_eq!(adapter.0.lock().unwrap().produced.len(), 1);
    drop(run);
    let mut resumed = supervisor(&project.0, adapter.clone());
    assert!(matches!(resumed.advance().unwrap().state, State::Complete { .. }));
    assert_eq!(adapter.0.lock().unwrap().produced.len(), 2); // candidate + merged verification, no recapture
}

#[test]
fn foreign_or_unchanged_rewrite_is_rejected_before_source_effect() {
    let project = Project::new();
    let adapter = Adapter::new();
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    let mut edits = replacement(&view, "new");
    edits.files[0].path = "tools/port/fixture.toml".into();
    assert!(run.submit(edits).is_err());
    assert!(run.submit(replacement(&view, "old")).is_err());
    assert_eq!(adapter.count("apply"), 0);
}

#[test]
fn dirty_integration_or_tampered_gate_never_publishes_acceptance() {
    for tamper in [false, true] {
        let project = Project::new();
        let adapter = Adapter::new();
        {
            let mut state = adapter.0.lock().unwrap();
            state.dirty_integration = !tamper;
            state.tamper_seal = tamper;
        }
        let mut run = supervisor(&project.0, adapter.clone());
        let view = run.advance().unwrap();
        assert!(matches!(run.submit(replacement(&view, "new")).unwrap().state, State::Stopped { .. }));
        assert_eq!(adapter.count("publish"), 0);
    }
}

#[test]
fn incomplete_cleanup_resumes_only_finalization_and_then_publishes() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().incomplete_cleanup = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    assert!(matches!(run.submit(replacement(&view, "new")).unwrap().state, State::Finalizing { .. }));
    assert_eq!(adapter.count("verify"), 2);
    assert_eq!(adapter.count("publish"), 0);
    let retained = fs::read_dir(&project.0).unwrap().count();
    for _ in 0..3 {
        assert!(matches!(run.advance().unwrap().state, State::Finalizing { .. }));
    }
    assert_eq!(
        fs::read_dir(&project.0).unwrap().count(),
        retained,
        "identical failed cleanup resumes allocate no metadata"
    );
    adapter.0.lock().unwrap().incomplete_cleanup = false;
    assert!(matches!(run.advance().unwrap().state, State::Complete { .. }));
    assert_eq!(adapter.count("verify"), 2);
}

#[test]
fn cancellation_reconciles_an_ambiguous_writer_and_never_evaluates_again() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().interrupt_once = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    run.submit(replacement(&view, "new")).unwrap();
    assert!(matches!(
        run.cancel("operator stop").unwrap().state,
        State::Stopped { disposition: Disposition::Cancelled, .. }
    ));
    assert!(adapter.0.lock().unwrap().interrupted_finalization);
    run.advance().unwrap();
    assert_eq!(adapter.count("verify"), 1);
}

#[test]
fn repair_opens_new_capsule_and_policy_changes_cannot_resume() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    run.submit(replacement(&view, "new")).unwrap();
    let next = run.repair("repair first divergence").unwrap();
    assert_eq!(next.attempt, 2);
    assert!(matches!(next.state, State::Rewrite { .. }));
    drop(run);
    let mut changed = policy();
    changed.budgets.workers = 5;
    assert!(Supervisor::open(&project.0, changed, adapter).is_err());
}

#[test]
fn repeated_failed_attempts_consume_one_shared_retained_budget() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut limited = policy();
    limited.budgets.max_evidence_bytes = 192;
    let mut run = Supervisor::open(&project.0, limited, adapter.clone()).unwrap();
    let view = run.advance().unwrap();
    run.submit(replacement(&view, "new")).unwrap();
    assert!(
        matches!(run.retry("explicit retry").unwrap().state, State::Finalizing { .. }),
        "second retained failure cannot exceed run budget"
    );
    assert_eq!(adapter.count("verify"), 2);
    assert_eq!(adapter.count("publish"), 0);
}

#[test]
fn terminal_resume_reauthenticates_retained_evidence_without_recapture() {
    let project = Project::new();
    let adapter = Adapter::new();
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    run.submit(replacement(&view, "new")).unwrap();
    adapter.0.lock().unwrap().retained_tampered = true;
    assert!(run.advance().is_err());
    assert!(run.inspect().is_err());
    assert_eq!(adapter.count("verify"), 2);
}
