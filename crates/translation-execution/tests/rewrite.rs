use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use translation_execution::{CapsuleBinding, Replacement, RewriteLimits, SourceReplacements, model::sha256, rewrite};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-rewrite-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("src")).unwrap();
        fs::write(path.join("src/a.rs"), "fn original() {}\n").unwrap();
        fs::write(path.join("src/b.rs"), "fn dependency() {}\n").unwrap();
        Self(path)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn limits() -> RewriteLimits {
    RewriteLimits { max_context_bytes: 1024, max_response_bytes: 4096, max_file_bytes: 1024, max_total_bytes: 2048 }
}
fn capsule(root: &Directory) -> translation_execution::RewriteCapsule {
    rewrite::capture(
        &root.0,
        CapsuleBinding {
            run_id: "scope-test".into(),
            attempt: 1,
            policy_sha256: sha256("policy"),
            source_sha256: sha256("source"),
        },
        &["src/a.rs".into(), "src/b.rs".into()],
        "recovered C and fixed constraints".into(),
        limits(),
    )
    .unwrap()
}
fn response(capsule: &translation_execution::RewriteCapsule) -> SourceReplacements {
    SourceReplacements {
        capsule_id: capsule.id.clone(),
        files: vec![Replacement {
            path: "src/a.rs".into(),
            base_sha256: capsule.files[0].sha256.clone(),
            content: "fn translated() {}\n".into(),
        }],
    }
}

#[test]
fn structured_replacement_updates_only_the_approved_source() {
    let root = Directory::new();
    let capsule = capsule(&root);
    let bytes = serde_json::to_vec(&response(&capsule)).unwrap();
    let decoded = rewrite::decode(&capsule, &bytes, limits()).unwrap();
    let result = rewrite::apply(&root.0, &capsule, &decoded, limits()).unwrap();
    assert_eq!(fs::read_to_string(root.0.join("src/a.rs")).unwrap(), "fn translated() {}\n");
    assert_eq!(fs::read_to_string(root.0.join("src/b.rs")).unwrap(), "fn dependency() {}\n");
    assert_eq!(result.files_sha256["src/a.rs"], sha256("fn translated() {}\n"));
    assert_eq!(result.files_sha256.len(), 2);
}

#[test]
fn expired_source_deadline_and_aggregate_extent_refuse_before_mutation() {
    let root = Directory::new();
    let captured = capsule(&root);
    assert!(
        rewrite::capture_until(
            &root.0,
            CapsuleBinding {
                run_id: captured.run_id.clone(),
                attempt: captured.attempt,
                policy_sha256: captured.policy_sha256.clone(),
                source_sha256: captured.source_sha256.clone(),
            },
            &["src/a.rs".into()],
            "context".into(),
            limits(),
            0,
        )
        .is_err()
    );
    assert!(rewrite::apply_until(&root.0, &captured, &response(&captured), limits(), 0).is_err());
    assert_eq!(fs::read_to_string(root.0.join("src/a.rs")).unwrap(), "fn original() {}\n");
    fs::write(root.0.join("src/a.rs"), "a".repeat(1024)).unwrap();
    fs::write(root.0.join("src/b.rs"), "b".repeat(1024)).unwrap();
    fs::write(root.0.join("src/c.rs"), "c").unwrap();
    assert!(
        rewrite::capture(
            &root.0,
            CapsuleBinding {
                run_id: "extent".into(),
                attempt: 1,
                policy_sha256: sha256("policy"),
                source_sha256: sha256("source"),
            },
            &["src/a.rs".into(), "src/b.rs".into(), "src/c.rs".into()],
            "context".into(),
            limits(),
        )
        .is_err()
    );
}

#[test]
fn foreign_alias_stale_and_oversize_messages_fail_before_source_changes() {
    let root = Directory::new();
    let capsule = capsule(&root);
    for path in [
        "../outside.rs",
        "src/../src/a.rs",
        "src/./a.rs",
        "./src/a.rs",
        "src//a.rs",
        "src/a.rs/",
        "src\\a.rs",
        "src/a.rs\0",
        ".scratch/ledger.toml",
        "src/not-approved.rs",
    ] {
        let mut invalid = response(&capsule);
        invalid.files[0].path = path.into();
        assert!(
            rewrite::decode(&capsule, &serde_json::to_vec(&invalid).unwrap(), limits()).is_err(),
            "accepted {path}"
        );
    }
    let mut invalid = response(&capsule);
    invalid.capsule_id = sha256("another capsule");
    assert!(rewrite::apply(&root.0, &capsule, &invalid, limits()).is_err());
    let mut invalid = response(&capsule);
    invalid.files[0].content = "x".repeat(1025);
    assert!(rewrite::decode(&capsule, &serde_json::to_vec(&invalid).unwrap(), limits()).is_err());
    assert!(rewrite::decode(&capsule, &vec![b' '; 4097], limits()).is_err());
    assert_eq!(fs::read_to_string(root.0.join("src/a.rs")).unwrap(), "fn original() {}\n");
}

#[test]
fn altered_dependency_rejects_all_replacements_before_mutation() {
    let root = Directory::new();
    let capsule = capsule(&root);
    fs::write(root.0.join("src/b.rs"), "changed by another owner").unwrap();
    assert!(rewrite::apply(&root.0, &capsule, &response(&capsule), limits()).is_err());
    assert_eq!(fs::read_to_string(root.0.join("src/a.rs")).unwrap(), "fn original() {}\n");
}

#[test]
fn symbolic_and_hardlink_source_aliases_are_not_writable() {
    use std::os::unix::fs::symlink;
    let root = Directory::new();
    let capsule = capsule(&root);
    fs::remove_file(root.0.join("src/a.rs")).unwrap();
    symlink("b.rs", root.0.join("src/a.rs")).unwrap();
    assert!(rewrite::apply(&root.0, &capsule, &response(&capsule), limits()).is_err());
    assert!(
        rewrite::capture(
            &root.0,
            CapsuleBinding {
                run_id: "test".into(),
                attempt: 1,
                policy_sha256: sha256("policy"),
                source_sha256: sha256("source"),
            },
            &["src/a.rs".into()],
            "context".into(),
            limits()
        )
        .is_err()
    );
    fs::remove_file(root.0.join("src/a.rs")).unwrap();
    fs::hard_link(root.0.join("src/b.rs"), root.0.join("src/a.rs")).unwrap();
    assert!(
        rewrite::capture(
            &root.0,
            CapsuleBinding {
                run_id: "test".into(),
                attempt: 1,
                policy_sha256: sha256("policy"),
                source_sha256: sha256("source"),
            },
            &["src/a.rs".into()],
            "context".into(),
            limits()
        )
        .is_err()
    );
}

#[test]
fn response_has_no_execution_or_evidence_fields() {
    let root = Directory::new();
    let capsule = capsule(&root);
    for field in ["command", "stage", "authority", "evidence", "run_path", "worker_count", "policy"] {
        let mut unknown = serde_json::to_value(response(&capsule)).unwrap();
        unknown[field] = serde_json::json!(["sm-pipeline", "family-verify"]);
        assert!(
            rewrite::decode(&capsule, &serde_json::to_vec(&unknown).unwrap(), limits()).is_err(),
            "accepted {field}"
        );
        let mut nested = serde_json::to_value(response(&capsule)).unwrap();
        nested["files"][0][field] = serde_json::json!("forbidden");
        assert!(
            rewrite::decode(&capsule, &serde_json::to_vec(&nested).unwrap(), limits()).is_err(),
            "accepted replacement {field}"
        );
    }
}

#[test]
fn frozen_tests_manifests_and_source_root_aliases_refuse_capture() {
    use std::os::unix::fs::symlink;
    let root = Directory::new();
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "tests/frozen.rs",
        "src/tests/frozen.rs",
        "manifests/verification.rs",
        "fixtures/frozen.rs",
    ] {
        let target = root.0.join(path);
        fs::create_dir_all(target.parent().unwrap()).unwrap();
        fs::write(&target, "frozen").unwrap();
        assert!(
            rewrite::capture(
                &root.0,
                CapsuleBinding {
                    run_id: "test".into(),
                    attempt: 1,
                    policy_sha256: sha256("policy"),
                    source_sha256: sha256("source"),
                },
                &[path.into()],
                "context".into(),
                limits()
            )
            .is_err(),
            "captured frozen {path}"
        );
    }
    let alias = root.0.join("root-alias");
    symlink(&root.0, &alias).unwrap();
    for source_root in [alias.clone(), alias.join("src").join("..")] {
        assert!(
            rewrite::capture(
                &source_root,
                CapsuleBinding {
                    run_id: "test".into(),
                    attempt: 1,
                    policy_sha256: sha256("policy"),
                    source_sha256: sha256("source"),
                },
                &["src/a.rs".into()],
                "context".into(),
                limits()
            )
            .is_err()
        );
    }
}

#[test]
fn base_hash_duplicates_capsule_tampering_and_aggregate_overflow_reject() {
    let root = Directory::new();
    let capsule = capsule(&root);
    let mut invalid = response(&capsule);
    invalid.files[0].base_sha256 = sha256("foreign source");
    assert!(rewrite::decode(&capsule, &serde_json::to_vec(&invalid).unwrap(), limits()).is_err());
    let mut invalid = response(&capsule);
    invalid.files.push(invalid.files[0].clone());
    assert!(rewrite::decode(&capsule, &serde_json::to_vec(&invalid).unwrap(), limits()).is_err());
    let mut changed = capsule.clone();
    changed.context.push_str(" changed");
    assert!(rewrite::encode(&changed, limits()).is_err());
    let mut changed = serde_json::to_value(&capsule).unwrap();
    changed["authority"] = serde_json::json!({"generation":1});
    assert!(rewrite::decode_capsule(&serde_json::to_vec(&changed).unwrap(), limits()).is_err());
    let mut small = limits();
    small.max_total_bytes = 1024;
    let mut invalid = response(&capsule);
    invalid.files[0].content = "x".repeat(700);
    invalid.files.push(Replacement {
        path: "src/b.rs".into(),
        base_sha256: capsule.files[1].sha256.clone(),
        content: "y".repeat(700),
    });
    assert!(rewrite::apply(&root.0, &capsule, &invalid, small).is_err());
    assert_eq!(fs::read_to_string(root.0.join("src/a.rs")).unwrap(), "fn original() {}\n");
    assert_eq!(fs::read_to_string(root.0.join("src/b.rs")).unwrap(), "fn dependency() {}\n");
}
