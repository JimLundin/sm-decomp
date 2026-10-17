use std::{
    fs,
    os::unix::{fs::PermissionsExt, io::AsRawFd},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use translation_execution::{checkout, model::*, rewrite};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Project {
    root: PathBuf,
    baseline: String,
    paths: Vec<String>,
}
impl Project {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "translation-revision-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let repository = root.join("repository");
        fs::create_dir_all(repository.join("src")).unwrap();
        git(&repository, &["init", "-q"]);
        fs::write(repository.join("src/first.rs"), "first baseline").unwrap();
        fs::write(repository.join("src/second.rs"), "second baseline").unwrap();
        fs::write(repository.join("frozen-test.rs"), "immutable test declaration").unwrap();
        git(&repository, &["add", "--all"]);
        git(&repository, &["commit", "-qm", "baseline"]);
        let baseline = git(&repository, &["rev-parse", "HEAD"]);
        Self { root, baseline, paths: vec!["src/first.rs".into(), "src/second.rs".into()] }
    }
    fn candidate(&self) -> PathBuf {
        let candidate = self.root.join("candidate");
        checkout::create(&self.root.join("repository"), &candidate, &self.baseline).unwrap();
        candidate
    }
    fn edit(&self, candidate: &Path, name: &str, text: &str) -> SourceSnapshot {
        let context = OperationContext {
            run_id: sha256("revision-run"),
            attempt: 1,
            policy_sha256: sha256("revision-policy"),
            operation_key: sha256(name),
            phase: ExecutionPhase::Candidate,
        };
        let source = checkout::snapshot(candidate, &self.paths).unwrap();
        let capsule = rewrite::capture(
            candidate,
            CapsuleBinding {
                run_id: context.run_id.clone(),
                attempt: 1,
                policy_sha256: context.policy_sha256.clone(),
                source_sha256: source.sha256,
            },
            &self.paths,
            "fixed retained declaration".into(),
            limits(),
        )
        .unwrap();
        let replacements = SourceReplacements {
            capsule_id: capsule.id.clone(),
            files: vec![Replacement {
                path: self.paths[0].clone(),
                base_sha256: capsule.files[0].sha256.clone(),
                content: text.into(),
            }],
        };
        checkout::apply_commit(candidate, &context, &capsule, &replacements, limits()).unwrap()
    }
    fn archive(&self, candidate: &Path) -> checkout::RevisionArchive {
        checkout::retain_revision(candidate, &self.root.join("archive"), &self.baseline, &self.paths, 1024 * 1024)
            .unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
fn git(path: &Path, args: &[&str]) -> String {
    let output = Command::new("/usr/bin/git")
        .args(["-c", "user.name=Fixture", "-c", "user.email=fixture@localhost"])
        .args(args)
        .current_dir(path)
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap().trim().into()
}
fn limits() -> RewriteLimits {
    RewriteLimits { max_context_bytes: 4096, max_response_bytes: 8192, max_file_bytes: 2048, max_total_bytes: 4096 }
}
fn no_git(root: &Path) {
    for entry in fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        assert_ne!(entry.file_name(), ".git");
        if entry.file_type().unwrap().is_dir() {
            no_git(&entry.path());
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis().try_into().unwrap()
}

fn publication_context() -> OperationContext {
    OperationContext {
        run_id: sha256("revision-run"),
        attempt: 1,
        policy_sha256: sha256("revision-policy"),
        operation_key: sha256("host-publication"),
        phase: ExecutionPhase::Publication,
    }
}

fn publication_archives(project: &Project) -> (checkout::RevisionArchive, checkout::RevisionArchive) {
    let candidate = project.candidate();
    let source = project.edit(&candidate, "candidate-publication", "source ready for host publication");
    let integration = project.root.join("integration");
    checkout::create(&project.root.join("repository"), &integration, &project.baseline).unwrap();
    let context = OperationContext {
        operation_key: sha256("integration-publication"),
        phase: ExecutionPhase::Integration,
        ..publication_context()
    };
    checkout::integrate(&candidate, &integration, &context, &source, &project.baseline, &project.paths).unwrap();
    let candidate_archive = project.archive(&candidate);
    let integration_archive = checkout::retain_revision(
        &integration,
        &project.root.join("integration-archive"),
        &project.baseline,
        &project.paths,
        1024 * 1024,
    )
    .unwrap();
    (candidate_archive, integration_archive)
}

fn publish(
    project: &Project,
    candidate: &checkout::RevisionArchive,
    integrated: &checkout::RevisionArchive,
) -> anyhow::Result<Vec<translation_execution::retention::Reference>> {
    checkout::publish_revisions(
        &project.root.join("repository"),
        candidate,
        integrated,
        &publication_context(),
        &project.paths,
        &project.root.join("host-publication"),
        now_ms() + 120_000,
    )
}

#[test]
fn inherited_expired_deadlines_refuse_before_clone_index_refs_or_source_mutation() {
    let project = Project::new();
    let repository = project.root.join("repository");
    let index = fs::read(repository.join(".git/index")).unwrap();
    let expired = now_ms().saturating_sub(1);
    assert!(checkout::checkout_bytes_until(&repository, &project.baseline, 4096, expired).is_err());
    let refused = project.root.join("refused");
    assert!(checkout::create_until(&repository, &refused, &project.baseline, expired).is_err());
    assert!(!refused.exists());
    let candidate = project.candidate();
    assert!(checkout::snapshot_until(&candidate, &project.paths, expired).is_err());
    let context = OperationContext {
        operation_key: sha256("expired-apply"),
        phase: ExecutionPhase::Candidate,
        ..publication_context()
    };
    let source = checkout::snapshot(&candidate, &project.paths).unwrap();
    let capsule = rewrite::capture(
        &candidate,
        CapsuleBinding {
            run_id: context.run_id.clone(),
            attempt: 1,
            policy_sha256: context.policy_sha256.clone(),
            source_sha256: source.sha256,
        },
        &project.paths,
        "deadline contract".into(),
        limits(),
    )
    .unwrap();
    let replacements = SourceReplacements {
        capsule_id: capsule.id.clone(),
        files: vec![Replacement {
            path: project.paths[0].clone(),
            base_sha256: capsule.files[0].sha256.clone(),
            content: "must not be written".into(),
        }],
    };
    assert!(checkout::apply_commit_until(&candidate, &context, &capsule, &replacements, limits(), expired).is_err());
    assert_eq!(git(&candidate, &["rev-parse", "HEAD"]), project.baseline);
    assert_eq!(fs::read_to_string(candidate.join(&project.paths[0])).unwrap(), "first baseline");
    let (candidate_archive, integrated_archive) = publication_archives(&project);
    let unlaunched = project.root.join("unlaunched-restore");
    assert!(
        checkout::restore_revision_until(&repository, &unlaunched, &candidate_archive, &project.paths, expired)
            .is_err()
    );
    assert!(!unlaunched.exists());
    assert!(
        checkout::publish_revisions(
            &repository,
            &candidate_archive,
            &integrated_archive,
            &publication_context(),
            &project.paths,
            &project.root.join("expired-publication"),
            expired
        )
        .is_err()
    );
    assert!(!project.root.join("expired-publication").exists());
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), project.baseline);
    assert_eq!(fs::read(repository.join(".git/index")).unwrap(), index);
    assert_eq!(git(&repository, &["for-each-ref", "--format=%(refname)", "refs/translation-execution"]), "");
}

#[test]
fn host_publication_cas_retains_candidate_and_integration_and_retries_exactly() {
    let project = Project::new();
    let (candidate, integrated) = publication_archives(&project);
    let repository = project.root.join("repository");
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), project.baseline);
    assert_eq!(fs::read_to_string(repository.join(&project.paths[0])).unwrap(), "first baseline");
    fs::remove_dir_all(project.root.join("candidate")).unwrap();
    fs::remove_dir_all(project.root.join("integration")).unwrap();
    let references = publish(&project, &candidate, &integrated).unwrap();
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), integrated.source.commit);
    assert_eq!(git(&repository, &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    assert!(!repository.join(".git/index.lock").exists());
    let prefix = format!("refs/translation-execution/{}/attempt-1", publication_context().run_id);
    assert_eq!(git(&repository, &["rev-parse", &format!("{prefix}/candidate")]), candidate.source.commit);
    assert_eq!(git(&repository, &["rev-parse", &format!("{prefix}/integrated")]), integrated.source.commit);
    assert!(git(&repository, &["symbolic-ref", "HEAD"]).starts_with("refs/heads/"));
    assert_eq!(fs::read_to_string(repository.join("frozen-test.rs")).unwrap(), "immutable test declaration");
    let index = fs::read(repository.join(".git/index")).unwrap();
    assert_eq!(publish(&project, &candidate, &integrated).unwrap(), references);
    assert_eq!(fs::read(repository.join(".git/index")).unwrap(), index);
    for reference in &references {
        translation_execution::retention::authenticate_reference(reference, 1024 * 1024).unwrap();
    }
    fs::remove_file(project.root.join("host-publication/result.json")).unwrap();
    assert_eq!(
        publish(&project, &candidate, &integrated).unwrap(),
        references,
        "lost response after branch CAS recovers the same sealed result"
    );
    assert_eq!(git(&repository, &["rev-list", "--count", "HEAD"]), "2");
}

#[test]
fn host_publication_recovers_partial_scoped_copy_and_its_exact_surviving_index_lock() {
    let project = Project::new();
    let (candidate, integrated) = publication_archives(&project);
    let expected = publish(&project, &candidate, &integrated).unwrap();
    let repository = project.root.join("repository");
    fs::remove_file(project.root.join("host-publication/result.json")).unwrap();
    git(&repository, &["reset", "--mixed", &project.baseline]);
    fs::write(repository.join(&project.paths[1]), "second baseline").unwrap();
    fs::hard_link(project.root.join("host-publication/index-lock.marker"), repository.join(".git/index.lock")).unwrap();
    assert_eq!(publish(&project, &candidate, &integrated).unwrap(), expected);
    assert_eq!(git(&repository, &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), integrated.source.commit);
    assert!(!repository.join(".git/index.lock").exists());
}

#[test]
fn host_publication_refuses_external_edits_foreign_head_and_index_writer() {
    let project = Project::new();
    let (candidate, integrated) = publication_archives(&project);
    let repository = project.root.join("repository");
    let index = fs::read(repository.join(".git/index")).unwrap();
    fs::write(repository.join("frozen-test.rs"), "external fixture edit").unwrap();
    assert!(publish(&project, &candidate, &integrated).is_err());
    assert_eq!(fs::read(repository.join(".git/index")).unwrap(), index);
    assert_eq!(fs::read_to_string(repository.join("frozen-test.rs")).unwrap(), "external fixture edit");
    assert!(!project.root.join("host-publication/intent.json").exists());
    fs::write(repository.join("frozen-test.rs"), "immutable test declaration").unwrap();
    fs::write(repository.join(&project.paths[0]), "external approved edit").unwrap();
    assert!(publish(&project, &candidate, &integrated).is_err());
    assert_eq!(fs::read_to_string(repository.join(&project.paths[0])).unwrap(), "external approved edit");
    fs::write(repository.join(&project.paths[0]), "first baseline").unwrap();
    fs::write(repository.join(".git/index.lock"), "foreign index writer").unwrap();
    let error = publish(&project, &candidate, &integrated).unwrap_err();
    assert!(error.downcast_ref::<translation_execution::ExecutionPending>().is_some());
    assert_eq!(fs::read_to_string(repository.join(".git/index.lock")).unwrap(), "foreign index writer");
    fs::remove_file(repository.join(".git/index.lock")).unwrap();
    fs::write(repository.join("frozen-test.rs"), "external committed edit").unwrap();
    git(&repository, &["add", "--", "frozen-test.rs"]);
    git(&repository, &["commit", "-qm", "foreign commit"]);
    let foreign = git(&repository, &["rev-parse", "HEAD"]);
    assert!(publish(&project, &candidate, &integrated).is_err());
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), foreign);
    assert_eq!(fs::read_to_string(repository.join("frozen-test.rs")).unwrap(), "external committed edit");
}

#[test]
fn host_publication_supports_the_original_linked_branch_with_ignored_controller_state() {
    let project = Project::new();
    let (candidate, integrated) = publication_archives(&project);
    let repository = project.root.join("repository");
    let linked = project.root.join("linked-owner");
    git(&repository, &["worktree", "add", "-b", "original-owner", linked.to_str().unwrap(), &project.baseline]);
    fs::write(repository.join(".git/info/exclude"), ".scratch/\n").unwrap();
    let root = linked.join(".scratch/publication-state");
    let references = checkout::publish_revisions(
        &linked,
        &candidate,
        &integrated,
        &publication_context(),
        &project.paths,
        &root,
        now_ms() + 120_000,
    )
    .unwrap();
    assert_eq!(git(&linked, &["rev-parse", "HEAD"]), integrated.source.commit);
    assert_eq!(git(&linked, &["symbolic-ref", "HEAD"]), "refs/heads/original-owner");
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), project.baseline);
    assert_eq!(fs::read_to_string(repository.join(&project.paths[0])).unwrap(), "first baseline");
    assert_eq!(git(&linked, &["status", "--porcelain=v1", "--untracked-files=all"]), "");
    assert_eq!(
        checkout::publish_revisions(
            &linked,
            &candidate,
            &integrated,
            &publication_context(),
            &project.paths,
            &root,
            now_ms() + 120_000
        )
        .unwrap(),
        references
    );
}

#[test]
fn bounded_read_only_planning_queries_preserve_nul_bytes_and_reject_mutating_argv() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let project = Project::new();
    let repository = project.root.join("repository");
    let unusual = OsString::from_vec(vec![b'u', b'n', 0xff]);
    fs::write(repository.join(&unusual), "untracked context").unwrap();
    let bytes = checkout::read_only_paths_until(
        &repository,
        &["ls-files", "--others", "--exclude-standard", "-z"],
        4096,
        now_ms() + 120_000,
    )
    .unwrap();
    assert!(bytes.windows(4).any(|window| window == [b'u', b'n', 0xff, 0]));
    assert!(checkout::read_only_paths_until(&repository, &["reset", "--hard"], 4096, now_ms() + 120_000).is_err());
    assert!(
        checkout::read_only_paths_until(
            &repository,
            &["ls-files", "--others", "--exclude-standard", "-z"],
            1,
            now_ms() + 120_000
        )
        .is_err()
    );
    assert!(
        checkout::read_only_paths_until(
            &repository,
            &["ls-files", "--others", "--exclude-standard", "-z"],
            4096,
            now_ms().saturating_sub(1)
        )
        .is_err()
    );
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), project.baseline);
}

#[test]
fn tracked_controller_projection_changes_only_declared_paths_and_preserves_committed_index() {
    let mut project = Project::new();
    let repository = project.root.join("repository");
    let ledger = repository.join(".scratch/managed/ledger.toml");
    let future = repository.join(".scratch/managed/integration-ticket.md");
    fs::create_dir_all(ledger.parent().unwrap()).unwrap();
    fs::write(&ledger, "original running claim\n").unwrap();
    git(&repository, &["add", "--", ".scratch/managed/ledger.toml"]);
    git(&repository, &["commit", "-qm", "original managed metadata"]);
    project.baseline = git(&repository, &["rev-parse", "HEAD"]);
    let (candidate, integrated) = publication_archives(&project);
    fs::write(&ledger, "managed claim intent\n").unwrap();
    let mut controller = checkout::ControllerProjection {
        paths: vec![ledger.clone(), future.clone()],
        references: vec![translation_execution::retention::seal_file(&ledger, 4096).unwrap()],
        ..Default::default()
    };
    let root = project.root.join("host-publication");
    let publish_current = |controller: &checkout::ControllerProjection| {
        checkout::publish_revisions_with_controller_until(
            &repository,
            &candidate,
            &integrated,
            &publication_context(),
            &project.paths,
            &root,
            controller,
            now_ms() + 120_000,
        )
    };
    let refs = publish_current(&controller).unwrap();
    assert_eq!(fs::read_to_string(&ledger).unwrap(), "managed claim intent\n");
    assert_eq!(git(&repository, &["show", "HEAD:.scratch/managed/ledger.toml"]), "original running claim");
    assert_eq!(git(&repository, &["diff", "--cached", "--name-only"]), "");
    checkout::authenticate_shared_revision_until(
        &repository,
        &integrated,
        &project.paths,
        &controller,
        now_ms() + 120_000,
    )
    .unwrap();
    let mut broadened = controller.clone();
    broadened.paths.push(repository.join(".scratch/foreign.md"));
    assert!(publish_current(&broadened).is_err(), "same publication cannot broaden controller inventory");
    fs::write(&ledger, "managed claim integrated\n").unwrap();
    assert!(publish_current(&controller).is_err(), "stale current controller hash refuses changed material");
    assert!(
        checkout::authenticate_shared_revision_until(
            &repository,
            &integrated,
            &project.paths,
            &controller,
            now_ms() + 120_000
        )
        .is_err()
    );
    fs::write(&future, "exact derived integration ticket\n").unwrap();
    controller.references = vec![
        translation_execution::retention::seal_file(&ledger, 4096).unwrap(),
        translation_execution::retention::seal_file(&future, 4096).unwrap(),
    ];
    assert_eq!(
        publish_current(&controller).unwrap(),
        refs,
        "current authenticated controller hashes may change only at predeclared paths"
    );
    checkout::authenticate_shared_revision_until(
        &repository,
        &integrated,
        &project.paths,
        &controller,
        now_ms() + 120_000,
    )
    .unwrap();
    fs::write(repository.join(".scratch/unowned.md"), "foreign metadata\n").unwrap();
    assert!(publish_current(&controller).is_err());
    assert!(
        checkout::authenticate_shared_revision_until(
            &repository,
            &integrated,
            &project.paths,
            &controller,
            now_ms() + 120_000
        )
        .is_err()
    );
    assert_eq!(fs::read_to_string(&ledger).unwrap(), "managed claim integrated\n");
    assert_eq!(git(&repository, &["diff", "--cached", "--name-only"]), "");
}

#[test]
fn source_publication_preserves_frozen_preexisting_metadata_and_rejects_changed_history() {
    let project = Project::new();
    let repository = project.root.join("repository");
    let (candidate, integrated) = publication_archives(&project);
    let history = repository.join(".scratch/retained-original.bin");
    fs::create_dir_all(history.parent().unwrap()).unwrap();
    fs::write(&history, "preexisting historical evidence").unwrap();
    let reference = translation_execution::retention::seal_file(&history, 4096).unwrap();
    let controller: checkout::ControllerProjection = serde_json::from_value(serde_json::json!({
        "paths": [], "references": [], "preserved": [reference]
    }))
    .unwrap();
    let root = project.root.join("publication-with-history");
    let publish = |controller: &checkout::ControllerProjection| {
        checkout::publish_revisions_with_controller_until(
            &repository,
            &candidate,
            &integrated,
            &publication_context(),
            &project.paths,
            &root,
            controller,
            now_ms() + 120_000,
        )
    };
    publish(&controller).unwrap();
    checkout::authenticate_shared_revision_until(
        &repository,
        &integrated,
        &project.paths,
        &controller,
        now_ms() + 120_000,
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&history).unwrap(), "preexisting historical evidence");
    fs::write(&history, "changed historical evidence").unwrap();
    assert!(publish(&controller).is_err());
    let changed = translation_execution::retention::seal_file(&history, 4096).unwrap();
    let changed: checkout::ControllerProjection = serde_json::from_value(serde_json::json!({
        "paths": [], "references": [], "preserved": [changed]
    }))
    .unwrap();
    assert!(publish(&changed).is_err(), "a retry cannot adopt newly sealed historical bytes");
}

#[test]
fn tracked_publication_metadata_recovers_interrupted_intent_and_result_writes() {
    let project = Project::new();
    let (candidate, integrated) = publication_archives(&project);
    let repository = project.root.join("repository");
    let root = repository.join(".scratch/exact-publication");
    let run = || {
        checkout::publish_revisions(
            &repository,
            &candidate,
            &integrated,
            &publication_context(),
            &project.paths,
            &root,
            now_ms() + 120_000,
        )
    };
    let refs = run().unwrap();
    let intent_bytes = fs::read(root.join("intent.json")).unwrap();
    let result_bytes = fs::read(root.join("result.json")).unwrap();
    fs::remove_file(root.join("result.json")).unwrap();
    fs::remove_file(root.join("intent.json")).unwrap();
    git(&repository, &["reset", "--hard", &project.baseline]);
    fs::write(root.join("intent.pending"), &intent_bytes[..intent_bytes.len() / 2]).unwrap();
    assert_eq!(run().unwrap(), refs);
    fs::remove_file(root.join("result.json")).unwrap();
    fs::write(root.join("result.pending"), &result_bytes[..result_bytes.len() / 2]).unwrap();
    assert_eq!(run().unwrap(), refs);
    assert_eq!(git(&repository, &["diff", "--cached", "--name-only"]), "");
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), integrated.source.commit);
    fs::write(root.join("foreign.md"), "unowned publisher directory file").unwrap();
    assert!(run().is_err(), "publisher owns exact metadata files, not its whole directory");
}

#[test]
fn raw_revision_survives_producer_retirement_and_restores_exact_repair_source() {
    let project = Project::new();
    let candidate = project.candidate();
    project.edit(&candidate, "initial", "first translated source");
    let source = project.edit(&candidate, "repair", "repaired translated source");
    let archive = project.archive(&candidate);
    assert_eq!(archive.source, source);
    assert!(archive.objects.iter().any(|object| object.kind == checkout::RevisionObjectKind::Commit));
    assert!(archive.objects.iter().any(|object| object.kind == checkout::RevisionObjectKind::Tree));
    assert!(archive.objects.iter().any(|object| object.kind == checkout::RevisionObjectKind::Blob));
    let refs = checkout::revision_references(&archive).unwrap();
    assert_eq!(refs.iter().map(|reference| reference.bytes).sum::<u64>(), archive.retained_bytes);
    no_git(&project.root.join("archive"));
    fs::remove_dir_all(&candidate).unwrap();
    checkout::authenticate_revision(&archive).unwrap();
    assert_eq!(
        project.archive(&candidate),
        archive,
        "lost-response retention returns exact original after checkout retirement"
    );
    let restored = project.root.join("restored");
    assert_eq!(
        checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).unwrap(),
        source
    );
    assert_eq!(
        checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).unwrap(),
        source
    );
    assert_eq!(git(&restored, &["rev-list", "--count", "HEAD"]), "3");
    assert_eq!(git(&project.root.join("repository"), &["rev-parse", "HEAD"]), project.baseline);
    assert_eq!(fs::read_to_string(restored.join("frozen-test.rs")).unwrap(), "immutable test declaration");
    let repaired = project.edit(&restored, "next-repair", "another exact repair");
    assert_ne!(repaired.commit, source.commit);
    assert!(checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).is_err());
}

#[test]
fn archive_resume_authenticates_existing_objects_and_partial_restore_reconciles() {
    let project = Project::new();
    let candidate = project.candidate();
    let source = project.edit(&candidate, "revision", "bounded exact revision");
    let first = project.archive(&candidate);
    let object = &first.objects[0].reference;
    let bytes = fs::read(&object.path).unwrap();
    fs::remove_file(&first.manifest.path).unwrap();
    fs::rename(&object.path, object.path.with_extension("pending")).unwrap();
    let archive = project.archive(&candidate);
    assert_eq!(archive, first);
    assert_eq!(fs::read(&object.path).unwrap(), bytes);
    let restored = project.root.join("restored");
    checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).unwrap();
    // Reconstruct a controller death during checkout after approved files were
    // copied but before HEAD advanced. The original restore intent remains.
    git(&restored, &["reset", "--mixed", &project.baseline]);
    assert_eq!(fs::read_to_string(restored.join(&project.paths[0])).unwrap(), "bounded exact revision");
    assert_eq!(
        checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).unwrap(),
        source
    );
}

#[test]
fn tampering_reference_aliases_and_foreign_checkout_paths_fail_closed() {
    let project = Project::new();
    let candidate = project.candidate();
    project.edit(&candidate, "revision", "exact accepted archive");
    let archive = project.archive(&candidate);
    let object = &archive.objects[0].reference;
    let original = fs::read(&object.path).unwrap();
    fs::set_permissions(&object.path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&object.path, b"forged raw object").unwrap();
    assert!(checkout::authenticate_revision(&archive).is_err());
    let destination = project.root.join("unlaunched");
    assert!(
        checkout::restore_revision(&project.root.join("repository"), &destination, &archive, &project.paths).is_err()
    );
    assert!(!destination.exists(), "tampered archive must fail before a clone");
    fs::write(&object.path, original).unwrap();
    checkout::authenticate_revision(&archive).unwrap();
    let mut redirected = archive.clone();
    redirected.objects[0].reference.path = project.root.join("foreign-object");
    assert!(checkout::authenticate_revision(&redirected).is_err());
    std::os::unix::fs::symlink(&object.path, project.root.join("archive/objects/foreign.bin")).unwrap();
    assert!(checkout::authenticate_revision(&archive).is_err());
    fs::remove_file(project.root.join("archive/objects/foreign.bin")).unwrap();
    let restored = project.root.join("restored");
    checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).unwrap();
    fs::write(restored.join("frozen-test.rs"), "foreign edit").unwrap();
    assert!(checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).is_err());
}

#[test]
fn archive_reserves_aggregate_bytes_before_publication_and_rejects_foreign_revisions() {
    let project = Project::new();
    let candidate = project.candidate();
    project.edit(&candidate, "revision", "bounded source");
    let low = project.root.join("low-budget");
    assert!(checkout::retain_revision(&candidate, &low, &project.baseline, &project.paths, 1).is_err());
    assert!(!low.join("intent.json").exists());
    assert!(!low.join("objects").exists());
    let archive = project.archive(&candidate);
    let exact = checkout::retain_revision(
        &candidate,
        &project.root.join("exact"),
        &project.baseline,
        &project.paths,
        archive.retained_bytes + 4096,
    )
    .unwrap();
    assert!(exact.retained_bytes <= exact.max_bytes);
    fs::write(candidate.join("frozen-test.rs"), "changed test declaration").unwrap();
    git(&candidate, &["add", "--", "frozen-test.rs"]);
    git(&candidate, &["commit", "-qm", "foreign change"]);
    assert!(
        checkout::retain_revision(
            &candidate,
            &project.root.join("foreign"),
            &project.baseline,
            &project.paths,
            1024 * 1024
        )
        .is_err()
    );
    let wrong = vec!["src/first.rs".into()];
    assert!(
        checkout::restore_revision(
            &project.root.join("repository"),
            &project.root.join("wrong-owner"),
            &archive,
            &wrong
        )
        .is_err()
    );
}

#[test]
fn baseline_archive_and_duplicate_controller_lease_are_exact() {
    let project = Project::new();
    let candidate = project.candidate();
    let archive = project.archive(&candidate);
    assert!(archive.objects.is_empty());
    let lock = fs::OpenOptions::new().read(true).write(true).open(project.root.join("archive/lock")).unwrap();
    assert_eq!(unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) }, 0);
    let error = checkout::retain_revision(
        &candidate,
        &project.root.join("archive"),
        &project.baseline,
        &project.paths,
        archive.max_bytes,
    )
    .unwrap_err();
    assert!(error.downcast_ref::<translation_execution::ExecutionPending>().is_some());
    drop(lock);
    let restored = project.root.join("restored");
    assert_eq!(
        checkout::restore_revision(&project.root.join("repository"), &restored, &archive, &project.paths).unwrap(),
        archive.source
    );
    let alias = project.root.join("archive-alias");
    std::os::unix::fs::symlink(project.root.join("archive"), &alias).unwrap();
    assert!(
        checkout::retain_revision(&candidate, &alias, &project.baseline, &project.paths, archive.max_bytes).is_err()
    );
}

#[test]
fn trusted_tree_measurement_refuses_symlinks_and_aggregate_bytes_before_clone() {
    let project = Project::new();
    let repository = project.root.join("repository");
    let bytes = checkout::checkout_bytes(&repository, &project.baseline, 4096).unwrap();
    assert_eq!(
        bytes,
        b"first baseline".len() as u64 + b"second baseline".len() as u64 + b"immutable test declaration".len() as u64
    );
    assert!(checkout::checkout_bytes(&repository, &project.baseline, bytes - 1).is_err());
    assert_eq!(git(&repository, &["rev-parse", "HEAD"]), project.baseline);
    std::os::unix::fs::symlink("/etc/passwd", repository.join("external")).unwrap();
    git(&repository, &["add", "--", "external"]);
    git(&repository, &["commit", "-qm", "unsafe mode"]);
    let unsafe_revision = git(&repository, &["rev-parse", "HEAD"]);
    assert!(checkout::checkout_bytes(&repository, &unsafe_revision, 4096).is_err());
}
