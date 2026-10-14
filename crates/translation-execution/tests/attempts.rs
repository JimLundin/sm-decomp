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
    tampered_finalization: Option<String>,
    authenticated_finalizations: BTreeMap<String, usize>,
    projected_journal_bytes: u64,
    finalization_inputs: Vec<(OperationContext, FinalizationRequest)>,
    finalization_projection: Option<(OperationContext, FinalizationRequest, Finalization, u64)>,
    interrupt_finalization_once: bool,
    interrupt_publication_once: bool,
    publication_projection: Option<(OperationContext, Completion, Publication, u64)>,
    imported: Option<ImportedSubmission>,
    interrupt_import_once: bool,
    import_contexts: Vec<OperationContext>,
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
    fn import_submission(&mut self, ctx: &OperationContext, _: &FrozenPolicy) -> Result<Option<ImportedSubmission>> {
        let mut observed = self.0.lock().unwrap();
        if observed.imported.is_none() {
            return Ok(None);
        }
        observed.import_contexts.push(ctx.clone());
        if observed.interrupt_import_once {
            observed.interrupt_import_once = false;
            return Err(ExecutionPending { reason: "lost imported preparation response".into() }.into());
        }
        Ok(observed.imported.clone())
    }
    fn authenticate_submission(&self, _: &FrozenPolicy, imported: &ImportedSubmission) -> Result<()> {
        ensure!(self.0.lock().unwrap().imported.as_ref() == Some(imported), "imported admission or source changed");
        Ok(())
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
    fn authenticate_gate(
        &self,
        _: &FrozenPolicy,
        _: &SourceSnapshot,
        _: VerificationStage,
        outcome: &GateOutcome,
    ) -> Result<()> {
        let observed = self.0.lock().unwrap();
        ensure!(observed.produced.get(&outcome.seal.operation_key) == Some(outcome), "retained gate changed");
        Ok(())
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
        journal: &FinalizationJournal<'_>,
    ) -> Result<Finalization> {
        self.call("finalize");
        let mut observed = self.0.lock().unwrap();
        observed.finalization_inputs.push((ctx.clone(), request.clone()));
        observed.interrupted_finalization = request.interrupted_operation.is_some();
        ensure!(!observed.incomplete_cleanup, "selected cleanup pending");
        let source = request.source.as_ref().map(|source| source.sha256.as_str()).unwrap_or(&ctx.policy_sha256);
        let finalization = Finalization {
            seal: seal(ctx, source),
            cleanup_complete: true,
            retained_authenticatable: true,
            worker_retired: true,
            retained_bytes: 128,
            reclaimed_bytes: 1024,
        };
        ensure!(
            journal.additional_bytes(&finalization)? <= journal.max_additional_bytes()?,
            "actual finalization event exceeded its reserved maximum decimal widths"
        );
        observed.finalization_projection =
            Some((ctx.clone(), request.clone(), finalization.clone(), journal.additional_bytes(&finalization)?));
        if observed.interrupt_finalization_once {
            observed.interrupt_finalization_once = false;
            return Err(ExecutionPending { reason: "lost response after retained finalization".into() }.into());
        }
        Ok(finalization)
    }
    fn publish(
        &mut self,
        ctx: &OperationContext,
        _: &FrozenPolicy,
        completion: &Completion,
        journal: &PublicationJournal<'_>,
    ) -> Result<Publication> {
        self.call("publish");
        let publication = Publication {
            seal: seal(ctx, &completion.integrated.sha256),
            observation_ids: vec![format!("{}-integrated", ctx.operation_key)],
            retained_bytes: 64,
        };
        let mut observed = self.0.lock().unwrap();
        let bytes = journal.retained_bytes(&publication)?;
        observed.projected_journal_bytes = bytes;
        observed.publication_projection = Some((ctx.clone(), completion.clone(), publication.clone(), bytes));
        if observed.interrupt_publication_once {
            observed.interrupt_publication_once = false;
            return Err(ExecutionPending { reason: "lost response after sealed publication".into() }.into());
        }
        Ok(publication)
    }
    fn authenticate(&self, _: &FrozenPolicy, _: &FinalizationRequest, finalization: &Finalization) -> Result<()> {
        let mut observed = self.0.lock().unwrap();
        *observed.authenticated_finalizations.entry(finalization.seal.operation_key.clone()).or_default() += 1;
        ensure!(
            !observed.retained_tampered
                && observed.tampered_finalization.as_ref() != Some(&finalization.seal.operation_key),
            "retained evidence changed"
        );
        Ok(())
    }
    fn authenticate_publication(&self, _: &FrozenPolicy, _: &Completion, _: &Publication) -> Result<()> {
        ensure!(!self.0.lock().unwrap().retained_tampered, "retained publication changed");
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

fn imported_adapter() -> (Adapter, ImportedSubmission) {
    let adapter = Adapter::new();
    let mut submitted = source("already submitted");
    submitted.commit = "c".repeat(40);
    let imported = ImportedSubmission { source: submitted, admission_sha256: sha256("original submitted claim") };
    adapter.0.lock().unwrap().imported = Some(imported.clone());
    (adapter, imported)
}

#[test]
fn imported_submission_recovers_exact_intent_and_runs_every_gate_without_rewrite() {
    let project = Project::new();
    let (adapter, imported) = imported_adapter();
    adapter.0.lock().unwrap().interrupt_import_once = true;
    let mut frozen = policy();
    frozen.verification_stages = vec![VerificationStage::Focused, VerificationStage::Offline];
    let mut run = Supervisor::open(&project.0, frozen.clone(), adapter.clone()).unwrap();
    assert_eq!(run.run_with_rewrite(|_, _, _| panic!("import launched rewrite")).unwrap().state, State::Preparing);
    assert_eq!(adapter.count("verify"), 0);
    drop(run);
    let mut run = Supervisor::open(&project.0, frozen.clone(), adapter.clone()).unwrap();
    let complete = run.run_with_rewrite(|_, _, _| panic!("import launched rewrite")).unwrap();
    let State::Complete { completion, .. } = &complete.state else { panic!("imported submission did not complete") };
    assert_eq!(completion.candidate, imported.source);
    assert_eq!(completion.integrated.files_sha256, imported.source.files_sha256);
    assert_eq!(completion.gates.len(), 4);
    let contexts = adapter.0.lock().unwrap().import_contexts.clone();
    assert_eq!(contexts.len(), 2);
    assert_eq!(contexts[0], contexts[1], "lost import response must retain its operation key");
    drop(run);
    assert_eq!(adapter.count("prepare"), 0);
    assert_eq!(adapter.count("apply"), 0);
    assert_eq!(adapter.count("verify"), 4);
    let mut run = Supervisor::open(&project.0, frozen, adapter.clone()).unwrap();
    assert_eq!(run.run_with_rewrite(|_, _, _| panic!("completed import launched rewrite")).unwrap(), complete);
    assert_eq!(adapter.count("verify"), 4);
    assert_eq!(adapter.count("publish"), 1);
}

#[test]
fn imported_failure_retries_same_candidate_but_requires_new_admission_to_repair() {
    let project = Project::new();
    let (adapter, imported) = imported_adapter();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut run = supervisor(&project.0, adapter.clone());
    assert!(matches!(run.advance().unwrap().state, State::Stopped { disposition: Disposition::Failed, .. }));
    assert!(run.repair("change existing submission").is_err());
    adapter.0.lock().unwrap().fail_gate = false;
    let retried = run.retry("retry fixed verification prerequisite").unwrap();
    let State::Complete { completion, .. } = retried.state else { panic!("import retry did not complete") };
    assert_eq!(completion.attempt, 2);
    assert_eq!(completion.candidate, imported.source);
    assert_eq!(adapter.0.lock().unwrap().import_contexts.len(), 1);
    assert_eq!(adapter.count("prepare"), 0);
    assert_eq!(adapter.count("apply"), 0);
    assert_eq!(adapter.count("verify"), 3);
}

#[test]
fn imported_journal_tamper_is_authenticated_before_any_recovery_producer() {
    for field in ["admission", "source", "missing"] {
        let project = Project::new();
        let (adapter, _) = imported_adapter();
        adapter.0.lock().unwrap().interrupt_once = true;
        let mut run = supervisor(&project.0, adapter.clone());
        assert!(matches!(run.advance().unwrap().state, State::Verifying { .. }));
        let paths = events(&project.0);
        let imported_path = paths
            .iter()
            .find(|path| {
                let event: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
                event["state"]["imported_submission"].is_object() && event["state"]["intent"].is_null()
            })
            .unwrap();
        let mut event: serde_json::Value = serde_json::from_slice(&fs::read(imported_path).unwrap()).unwrap();
        match field {
            "admission" => event["state"]["imported_submission"]["admission_sha256"] = sha256("forged claim").into(),
            "source" => {
                event["state"]["imported_submission"]["source"]["commit"] = "d".repeat(40).into();
                event["state"]["candidate"]["commit"] = "d".repeat(40).into();
            }
            "missing" => {
                event["state"].as_object_mut().unwrap().remove("imported_submission");
            }
            _ => unreachable!(),
        }
        for path in paths.iter().filter(|path| *path > imported_path) {
            fs::remove_file(path).unwrap();
        }
        fs::write(imported_path, serde_json::to_vec(&event).unwrap()).unwrap();
        assert!(run.advance().is_err(), "forged {field} import authorized another producer");
        assert!(Supervisor::inspect_retained(&project.0, policy(), adapter.clone()).is_err());
        assert_eq!(adapter.count("verify"), 1);
        assert_eq!(adapter.count("prepare"), 0);
        assert_eq!(adapter.count("apply"), 0);
    }
}

#[test]
fn completed_import_reauthenticates_the_frozen_admission() {
    let project = Project::new();
    let (adapter, _) = imported_adapter();
    let mut run = supervisor(&project.0, adapter.clone());
    assert!(matches!(run.advance().unwrap().state, State::Complete { .. }));
    adapter.0.lock().unwrap().imported = None;
    assert!(run.inspect().is_err());
    assert!(run.advance().is_err());
    assert!(Supervisor::open(&project.0, policy(), adapter.clone()).is_err());
    assert_eq!(adapter.count("verify"), 2);
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
    let journal_bytes = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| entry.unwrap().metadata().unwrap())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum::<u64>();
    assert_eq!(
        adapter.0.lock().unwrap().projected_journal_bytes,
        journal_bytes,
        "publication must reserve the exact eventual terminal journal extent"
    );
    drop(run);
    let mut resumed = supervisor(&project.0, adapter.clone());
    assert!(matches!(resumed.submit(edits).unwrap().state, State::Complete { .. }));
    assert!(matches!(resumed.advance().unwrap().state, State::Complete { .. }));
    assert_eq!(adapter.count("verify"), 2);
    assert_eq!(adapter.count("publish"), 1);
}

#[test]
fn publication_extent_recovers_lost_terminal_and_rejects_completed_journal_tamper() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().interrupt_publication_once = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    assert_eq!(run.submit(replacement(&view, "new")).unwrap().state, State::Publishing);
    let (context, completion, publication, projected_bytes) =
        adapter.0.lock().unwrap().publication_projection.clone().unwrap();
    let before: BTreeMap<_, _> = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect();
    assert_eq!(
        authenticate_publication_extent(&project.0, &policy(), &context, &completion, &publication).unwrap(),
        projected_bytes
    );
    let after: BTreeMap<_, _> = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect();
    assert_eq!(after, before, "missing terminal authentication must allocate no replacement result or journal event");
    let mut wrong_completion = completion.clone();
    wrong_completion.integrated.commit = "c".repeat(40);
    assert!(authenticate_publication_extent(&project.0, &policy(), &context, &wrong_completion, &publication).is_err());
    drop(run);
    let mut run = supervisor(&project.0, adapter.clone());
    assert!(matches!(
        run.cancel("operator cancellation during publication recovery").unwrap().state,
        State::Complete { .. }
    ));
    assert_eq!(
        authenticate_publication_extent(&project.0, &policy(), &context, &completion, &publication).unwrap(),
        projected_bytes,
        "sealed publication must keep the same journal extent after missing terminal recovery"
    );
    let actual_bytes = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| entry.unwrap().metadata().unwrap())
        .filter(|metadata| metadata.is_file())
        .map(|metadata| metadata.len())
        .sum::<u64>();
    assert_eq!(actual_bytes, projected_bytes);
    let terminal_path = events(&project.0).pop().unwrap();
    let original = fs::read(&terminal_path).unwrap();
    let mut whitespace = original.clone();
    whitespace.push(b'\n');
    fs::write(&terminal_path, &whitespace).unwrap();
    assert!(
        authenticate_publication_extent(&project.0, &policy(), &context, &completion, &publication).is_err(),
        "byte-changed but semantically identical final event must reject"
    );
    fs::write(&terminal_path, &original).unwrap();
    let mut altered: serde_json::Value = serde_json::from_slice(&original).unwrap();
    altered["state"]["publication"]["retained_bytes"] = serde_json::json!(publication.retained_bytes + 1);
    fs::write(&terminal_path, serde_json::to_vec(&altered).unwrap()).unwrap();
    assert!(authenticate_publication_extent(&project.0, &policy(), &context, &completion, &publication).is_err());
    fs::write(&terminal_path, &original).unwrap();
    let extra_path = project.0.join("foreign-output.bin");
    fs::write(&extra_path, b"unsealed extra output").unwrap();
    assert_ne!(
        authenticate_publication_extent(&project.0, &policy(), &context, &completion, &publication).unwrap(),
        projected_bytes,
        "unexpected regular journal bytes must change the authenticated total"
    );
    fs::remove_file(extra_path).unwrap();
    assert_eq!(
        authenticate_publication_extent(&project.0, &policy(), &context, &completion, &publication).unwrap(),
        projected_bytes
    );
    assert_eq!(adapter.count("verify"), 2);
    assert_eq!(adapter.count("apply"), 1);
    assert_eq!(adapter.count("finalize"), 1);
    assert_eq!(adapter.count("publish"), 2);
}

#[test]
fn restricted_rewrite_lost_response_recovers_original_intent_and_terminal_runs_no_worker() {
    let project = Project::new();
    let adapter = Adapter::new();
    let mut run = supervisor(&project.0, adapter.clone());
    let mut original = None;
    let pending = run
        .run_with_rewrite(|_, context, capsule| {
            original = Some((context.clone(), capsule.clone()));
            Err(ExecutionPending { reason: "restricted transport response was lost".into() }.into())
        })
        .unwrap();
    assert!(matches!(pending.state, State::Rewrite { .. }));
    assert_eq!(adapter.count("apply"), 0);
    drop(run);
    let mut run = supervisor(&project.0, adapter.clone());
    let complete = run
        .run_with_rewrite(|_, context, capsule| {
            assert_eq!(original.as_ref(), Some(&(context.clone(), capsule.clone())));
            Ok(SourceReplacements {
                capsule_id: capsule.id.clone(),
                files: vec![Replacement {
                    path: capsule.files[0].path.clone(),
                    base_sha256: capsule.files[0].sha256.clone(),
                    content: "new".into(),
                }],
            })
        })
        .unwrap();
    assert!(matches!(complete.state, State::Complete { .. }));
    assert_eq!(run.run_with_rewrite(|_, _, _| panic!("completed run launched a worker")).unwrap(), complete);
    assert_eq!(adapter.count("apply"), 1);
    assert_eq!(adapter.count("verify"), 2);
}

#[test]
fn restricted_rewrite_refusal_finalizes_without_applying_or_launching_gates() {
    let project = Project::new();
    let adapter = Adapter::new();
    let mut run = supervisor(&project.0, adapter.clone());
    let stopped = run
        .run_with_rewrite(|_, _, capsule| {
            Ok(SourceReplacements {
                capsule_id: capsule.id.clone(),
                files: vec![Replacement {
                    path: "../foreign.rs".into(),
                    base_sha256: capsule.files[0].sha256.clone(),
                    content: "new".into(),
                }],
            })
        })
        .unwrap();
    assert!(matches!(stopped.state, State::Stopped { disposition: Disposition::Failed, .. }));
    assert_eq!(adapter.count("apply"), 0);
    assert_eq!(adapter.count("verify"), 0);
    assert!(adapter.0.lock().unwrap().interrupted_finalization);
    assert_eq!(run.run_with_rewrite(|_, _, _| panic!("stopped run launched a worker")).unwrap(), stopped);
    assert_eq!(adapter.count("finalize"), 1);
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
    let (context, request, result, projected_bytes) =
        adapter.0.lock().unwrap().finalization_projection.clone().unwrap();
    assert_eq!(request.disposition, Disposition::Cancelled);
    assert_eq!(
        authenticate_finalization_tail(&project.0, &policy(), &context, &request, &result).unwrap(),
        projected_bytes,
        "cancelled attempts authenticate the same prescribed exact terminal tail"
    );
    run.advance().unwrap();
    assert_eq!(adapter.count("verify"), 1);
}

#[test]
fn cancellation_preserves_partial_finalization_identity_and_original_failure() {
    for failed in [false, true] {
        let project = Project::new();
        let adapter = Adapter::new();
        {
            let mut observed = adapter.0.lock().unwrap();
            observed.fail_gate = failed;
            observed.incomplete_cleanup = true;
        }
        let mut run = supervisor(&project.0, adapter.clone());
        let view = run.advance().unwrap();
        let original = run.submit(replacement(&view, "new")).unwrap();
        let disposition = if failed { Disposition::Failed } else { Disposition::Complete };
        assert_eq!(original.state, State::Finalizing { disposition });
        let before: BTreeMap<_, _> = fs::read_dir(&project.0)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), fs::read(entry.path()).unwrap())
            })
            .collect();
        let producer_count = adapter.count("verify");
        drop(run);
        let mut run = supervisor(&project.0, adapter.clone());
        assert_eq!(run.cancel("operator cancellation after partial cleanup").unwrap(), original);
        let after: BTreeMap<_, _> = fs::read_dir(&project.0)
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), fs::read(entry.path()).unwrap())
            })
            .collect();
        assert_eq!(after, before, "partial finalization reconciliation must allocate no duplicate metadata");
        adapter.0.lock().unwrap().incomplete_cleanup = false;
        let finished = run.cancel("operator cancellation while finalization resumes").unwrap();
        let finalization = match &finished.state {
            State::Stopped { disposition: Disposition::Failed, reason, finalization: Some(finalization) } => {
                assert!(failed);
                assert_eq!(reason, "first mismatch", "cleanup failure must preserve the original gate diagnosis");
                finalization
            }
            State::Complete { completion, .. } => {
                assert!(!failed);
                &completion.finalization
            }
            _ => panic!("partial finalization changed its original disposition"),
        };
        let observed = adapter.0.lock().unwrap();
        let first = observed.finalization_inputs.first().unwrap();
        assert_eq!(finalization.seal.operation_key, first.0.operation_key);
        assert_eq!(first.1.disposition, disposition);
        assert!(observed.finalization_inputs.iter().all(|input| input == first));
        drop(observed);
        assert_eq!(adapter.count("verify"), producer_count);
        assert_eq!(run.cancel("operator cancellation after completion").unwrap(), finished);
    }
}

#[test]
fn stopped_finalization_tail_recovers_lost_response_and_authenticates_historical_extent() {
    let project = Project::new();
    let adapter = Adapter::new();
    {
        let mut observed = adapter.0.lock().unwrap();
        observed.fail_gate = true;
        observed.interrupt_finalization_once = true;
    }
    let mut run = supervisor(&project.0, adapter.clone());
    let view = run.advance().unwrap();
    assert_eq!(
        run.submit(replacement(&view, "new")).unwrap().state,
        State::Finalizing { disposition: Disposition::Failed }
    );
    let (context, request, result, projected_bytes) =
        adapter.0.lock().unwrap().finalization_projection.clone().unwrap();
    let before: BTreeMap<_, _> = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect();
    assert_eq!(
        authenticate_finalization_tail(&project.0, &policy(), &context, &request, &result).unwrap(),
        projected_bytes
    );
    let after: BTreeMap<_, _> = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            (entry.file_name(), fs::read(entry.path()).unwrap())
        })
        .collect();
    assert_eq!(after, before, "lost-response tail authentication must create no journal state");
    drop(run);
    let mut run = supervisor(&project.0, adapter.clone());
    assert!(matches!(run.advance().unwrap().state, State::Stopped { disposition: Disposition::Failed, .. }));
    let actual_bytes = fs::read_dir(&project.0)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with("event-"))
        .max()
        .unwrap()
        .metadata()
        .unwrap()
        .len();
    assert_eq!(projected_bytes, actual_bytes, "finalization reservation must exactly match its terminal result event");
    assert_eq!(
        authenticate_finalization_tail(&project.0, &policy(), &context, &request, &result).unwrap(),
        actual_bytes
    );
    adapter.0.lock().unwrap().fail_gate = false;
    assert!(matches!(
        run.retry("reevaluate after external prerequisite recovery").unwrap().state,
        State::Complete { .. }
    ));
    assert_eq!(
        authenticate_finalization_tail(&project.0, &policy(), &context, &request, &result).unwrap(),
        actual_bytes,
        "later journal growth must preserve the first stopped attempt's exact extent"
    );
    let mut foreign = context.clone();
    foreign.attempt = 2;
    assert!(authenticate_finalization_tail(&project.0, &policy(), &foreign, &request, &result).is_err());
    let mut altered = result.clone();
    altered.retained_bytes += 1;
    assert!(authenticate_finalization_tail(&project.0, &policy(), &context, &request, &altered).is_err());
    assert_eq!(adapter.count("verify"), 3);
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

fn events(path: &Path) -> Vec<PathBuf> {
    let mut events = fs::read_dir(path)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name().unwrap().to_string_lossy().starts_with("event-")
                && path.extension().is_some_and(|extension| extension == "json")
        })
        .collect::<Vec<_>>();
    events.sort();
    events
}

#[test]
fn failed_terminal_tail_cannot_authorize_an_unexecuted_gate() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut frozen = policy();
    frozen.verification_stages = vec![VerificationStage::Focused, VerificationStage::Offline];
    let mut run = Supervisor::open(&project.0, frozen.clone(), adapter.clone()).unwrap();
    let rewrite = run.advance().unwrap();
    let stopped = run.submit(replacement(&rewrite, "new")).unwrap();
    assert!(matches!(stopped.state, State::Stopped { .. }));
    let captures = adapter.0.lock().unwrap().produced.len();
    let tail = events(&project.0).pop().unwrap();
    let original = fs::read(&tail).unwrap();
    let mut event: serde_json::Value = serde_json::from_slice(&original).unwrap();
    event["state"]["cursor"] = serde_json::json!({"CandidateGate": 1});
    event["state"]["intent"] = serde_json::Value::Null;
    // Preserve the predecessor hash: this was previously a valid hash chain.
    fs::write(&tail, serde_json::to_vec(&event).unwrap()).unwrap();
    assert!(run.advance().is_err());
    assert!(Supervisor::inspect_retained(&project.0, frozen.clone(), adapter.clone()).is_err());
    assert!(Supervisor::open(&project.0, frozen.clone(), adapter.clone()).is_err());
    assert_eq!(adapter.0.lock().unwrap().produced.len(), captures);
    assert_eq!(adapter.count("verify"), 1);
    fs::write(tail, original).unwrap();
    assert_eq!(run.advance().unwrap(), stopped);
    assert_eq!(adapter.count("verify"), 1);
}

#[test]
fn forged_successful_gate_tail_rejects_before_the_next_producer() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut frozen = policy();
    frozen.verification_stages = vec![VerificationStage::Focused, VerificationStage::Offline];
    let mut run = Supervisor::open(&project.0, frozen.clone(), adapter.clone()).unwrap();
    let rewrite = run.advance().unwrap();
    assert!(matches!(run.submit(replacement(&rewrite, "new")).unwrap().state, State::Stopped { .. }));
    let paths = events(&project.0);
    let failed = paths
        .iter()
        .find(|path| {
            let event: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
            event["state"]["cursor"] == "Finalize"
                && event["state"]["gates"].as_array().is_some_and(|gates| !gates.is_empty())
                && event["state"]["intent"].is_null()
        })
        .unwrap();
    let mut event: serde_json::Value = serde_json::from_slice(&fs::read(failed).unwrap()).unwrap();
    event["state"]["cursor"] = serde_json::json!({"CandidateGate": 1});
    event["state"]["disposition"] = "complete".into();
    event["state"]["reason"] = "".into();
    event["state"]["gates"][0]["passed"] = true.into();
    event["state"]["gates"][0]["diagnostic"] = "".into();
    // Make the accepted gate result the tail, as if its response was lost.
    for path in paths.iter().filter(|path| *path > failed) {
        fs::remove_file(path).unwrap();
    }
    fs::write(failed, serde_json::to_vec(&event).unwrap()).unwrap();
    assert!(run.advance().is_err());
    assert_eq!(adapter.count("verify"), 1);
    assert_eq!(adapter.0.lock().unwrap().produced.len(), 1);
}

#[test]
fn later_complete_attempt_authenticates_the_prior_failed_evidence() {
    let project = Project::new();
    let adapter = Adapter::new();
    adapter.0.lock().unwrap().fail_gate = true;
    let mut run = supervisor(&project.0, adapter.clone());
    let rewrite = run.advance().unwrap();
    let stopped = run.submit(replacement(&rewrite, "new")).unwrap();
    let State::Stopped { finalization: Some(first), .. } = stopped.state else { panic!("failed result missing") };
    adapter.0.lock().unwrap().fail_gate = false;
    assert!(matches!(run.retry("operator reevaluates exact candidate").unwrap().state, State::Complete { .. }));
    let captures = adapter.0.lock().unwrap().produced.len();
    adapter.0.lock().unwrap().tampered_finalization = Some(first.seal.operation_key.clone());
    assert!(run.inspect().is_err());
    assert!(run.advance().is_err());
    assert!(authenticate_finalized_history(&project.0, &policy(), &adapter).is_err());
    assert_eq!(adapter.0.lock().unwrap().produced.len(), captures);
    adapter.0.lock().unwrap().tampered_finalization = None;
    assert!(matches!(run.advance().unwrap().state, State::Complete { .. }));
}
