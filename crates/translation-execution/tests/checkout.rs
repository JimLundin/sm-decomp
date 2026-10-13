use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
use translation_execution::{checkout, model::*, rewrite};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Project {
    root: PathBuf,
    base: String,
    paths: Vec<String>,
}
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "translation-checkout-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("original/crates/port/src")).unwrap();
        let source = root.join("original");
        git(&source, &["init", "-q"]);
        fs::write(source.join("crates/port/src/first.rs"), "first original").unwrap();
        fs::write(source.join("crates/port/src/second.rs"), "second original").unwrap();
        fs::write(source.join("frozen-test.rs"), "frozen fixture declaration").unwrap();
        git(&source, &["add", "--all"]);
        git(&source, &["commit", "-qm", "baseline"]);
        let base = git(&source, &["rev-parse", "HEAD"]);
        Self { root, base, paths: vec!["crates/port/src/first.rs".into(), "crates/port/src/second.rs".into()] }
    }
    fn clone_at(&self, name: &str) -> PathBuf {
        let path = self.root.join(name);
        checkout::create(&self.root.join("original"), &path, &self.base).unwrap();
        path
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn git(path: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@localhost"])
        .args(args)
        .current_dir(path)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn context(name: &str, phase: ExecutionPhase) -> OperationContext {
    OperationContext {
        run_id: sha256("run"),
        attempt: 1,
        policy_sha256: sha256("policy"),
        operation_key: sha256(name),
        phase,
    }
}
fn limits() -> RewriteLimits {
    RewriteLimits { max_context_bytes: 4096, max_response_bytes: 8192, max_file_bytes: 1024, max_total_bytes: 4096 }
}
fn edits(project: &Project, checkout: &Path) -> (OperationContext, RewriteCapsule, SourceReplacements) {
    let ctx = context("apply", ExecutionPhase::Candidate);
    let source = checkout::snapshot(checkout, &project.paths).unwrap();
    let capsule = rewrite::capture(
        checkout,
        CapsuleBinding {
            run_id: ctx.run_id.clone(),
            attempt: ctx.attempt,
            policy_sha256: ctx.policy_sha256.clone(),
            source_sha256: source.sha256,
        },
        &project.paths,
        "fixed fixture contract".into(),
        limits(),
    )
    .unwrap();
    let replacements = SourceReplacements {
        capsule_id: capsule.id.clone(),
        files: capsule
            .files
            .iter()
            .map(|file| Replacement {
                path: file.path.clone(),
                base_sha256: file.sha256.clone(),
                content: format!("{} rewritten", file.path),
            })
            .collect(),
    };
    (ctx, capsule, replacements)
}

#[test]
fn real_scoped_commit_integrates_and_exact_retries_make_no_new_commit() {
    let p = Project::new();
    let candidate = p.clone_at("candidate");
    let integrated = p.clone_at("integration");
    let (ctx, capsule, replacements) = edits(&p, &candidate);
    let source = checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    assert_eq!(checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap(), source);
    assert_eq!(git(&candidate, &["rev-list", "--count", "HEAD"]), "2");
    let merge = context("integration", ExecutionPhase::Integration);
    let merged = checkout::integrate(&candidate, &integrated, &merge, &source, &p.base, &p.paths).unwrap();
    assert_ne!(merged.sha256, source.sha256);
    assert_eq!(merged.files_sha256, source.files_sha256);
    assert_eq!(checkout::integrate(&candidate, &integrated, &merge, &source, &p.base, &p.paths).unwrap(), merged);
    assert_eq!(git(&integrated, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(git(&p.root.join("original"), &["rev-parse", "HEAD"]), p.base);
    assert_eq!(fs::read_to_string(integrated.join("frozen-test.rs")).unwrap(), "frozen fixture declaration");
    let retained = p.root.join("retained");
    assert_eq!(checkout::retain_source(&integrated, &retained, &p.paths).unwrap(), merged.files_sha256);
}

#[test]
fn interrupted_partial_replacement_is_completed_from_the_durable_intent() {
    let p = Project::new();
    let candidate = p.clone_at("candidate");
    let (ctx, capsule, replacements) = edits(&p, &candidate);
    checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    // Reconstruct the observable state after the first replacement but before
    // the controller commit. This leaves the real protected operation intent.
    git(&candidate, &["reset", "--mixed", &p.base]);
    fs::write(candidate.join(&p.paths[1]), "second original").unwrap();
    let recovered = checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    assert_eq!(git(&candidate, &["rev-list", "--count", "HEAD"]), "2");
    assert_eq!(recovered.files_sha256[&p.paths[1]], sha256(&replacements.files[1].content));
}

#[test]
fn foreign_dirty_source_and_unexpected_integration_head_are_rejected() {
    let p = Project::new();
    let candidate = p.clone_at("candidate");
    let integration = p.clone_at("integration");
    let (ctx, capsule, replacements) = edits(&p, &candidate);
    fs::write(candidate.join("frozen-test.rs"), "modified fixture").unwrap();
    assert!(checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).is_err());
    fs::write(candidate.join("frozen-test.rs"), "frozen fixture declaration").unwrap();
    let submitted = checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    assert!(
        checkout::integrate(
            &candidate,
            &integration,
            &context("merge", ExecutionPhase::Integration),
            &submitted,
            &"f".repeat(40),
            &p.paths
        )
        .is_err()
    );
    fs::write(integration.join("unowned"), "foreign write").unwrap();
    assert!(
        checkout::integrate(
            &candidate,
            &integration,
            &context("merge", ExecutionPhase::Integration),
            &submitted,
            &p.base,
            &p.paths
        )
        .is_err()
    );
}

#[test]
fn changed_candidate_and_source_aliases_fail_snapshot_admission() {
    let p = Project::new();
    let candidate = p.clone_at("candidate");
    let integration = p.clone_at("integration");
    let (ctx, capsule, replacements) = edits(&p, &candidate);
    let submitted = checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    fs::write(candidate.join(&p.paths[0]), "different source").unwrap();
    assert!(
        checkout::integrate(
            &candidate,
            &integration,
            &context("merge", ExecutionPhase::Integration),
            &submitted,
            &p.base,
            &p.paths
        )
        .is_err()
    );
    fs::remove_file(candidate.join(&p.paths[0])).unwrap();
    std::os::unix::fs::symlink("../../../../outside", candidate.join(&p.paths[0])).unwrap();
    assert!(checkout::snapshot(&candidate, &p.paths).is_err());
}

#[test]
fn interrupted_integration_copy_recovers_the_same_scoped_commit() {
    let p = Project::new();
    let candidate = p.clone_at("candidate");
    let integrated = p.clone_at("integration");
    let (ctx, capsule, replacements) = edits(&p, &candidate);
    let submitted = checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    let integration = context("integration", ExecutionPhase::Integration);
    let first = checkout::integrate(&candidate, &integrated, &integration, &submitted, &p.base, &p.paths).unwrap();
    git(&integrated, &["reset", "--mixed", &p.base]);
    fs::write(integrated.join(&p.paths[1]), "second original").unwrap();
    assert_eq!(
        checkout::integrate(&candidate, &integrated, &integration, &submitted, &p.base, &p.paths).unwrap(),
        first
    );
}

#[test]
fn protected_publication_reuses_original_bytes_and_rejects_changed_acceptance() {
    let p = Project::new();
    let candidate = p.clone_at("candidate");
    let integrated = p.clone_at("integration");
    let (ctx, capsule, replacements) = edits(&p, &candidate);
    let submitted = checkout::apply_commit(&candidate, &ctx, &capsule, &replacements, limits()).unwrap();
    let merged = checkout::integrate(
        &candidate,
        &integrated,
        &context("integration", ExecutionPhase::Integration),
        &submitted,
        &p.base,
        &p.paths,
    )
    .unwrap();
    let publication = context("publication", ExecutionPhase::Publication);
    let completion = Completion {
        run_id: publication.run_id.clone(),
        attempt: 1,
        policy_sha256: publication.policy_sha256.clone(),
        candidate: submitted,
        integrated: merged.clone(),
        gates: vec![],
        finalization: Finalization {
            seal: OutcomeSeal {
                operation_key: sha256("finalize"),
                policy_sha256: publication.policy_sha256.clone(),
                source_sha256: merged.sha256,
                evidence_sha256: sha256("retained"),
            },
            cleanup_complete: true,
            retained_authenticatable: true,
            worker_retired: true,
            retained_bytes: 32,
            reclaimed_bytes: 128,
        },
    };
    let path = p.root.join("publication/completion.json");
    let result = checkout::publish_completion(&path, &publication, &completion).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(checkout::publish_completion(&path, &publication, &completion).unwrap(), result);
    assert_eq!(fs::read(&path).unwrap(), bytes);
    let mut changed = completion;
    changed.finalization.retained_bytes += 1;
    assert!(checkout::publish_completion(&path, &publication, &changed).is_err());
}
