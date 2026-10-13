use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
use translation_execution::{
    model::*,
    retention::{self, Retained, RetainedKind},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "translation-retention-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn file(&self, name: &str, bytes: &[u8]) -> retention::Reference {
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        retention::seal_file(&path, 1024 * 1024).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn context() -> OperationContext {
    OperationContext {
        run_id: sha256("run"),
        attempt: 1,
        policy_sha256: sha256("policy"),
        operation_key: sha256("finalize"),
        phase: ExecutionPhase::Finalization,
    }
}

#[test]
fn exact_accepted_failed_and_source_closure_survives_retirement_and_retries() {
    let f = Fixture::new();
    let accepted = f.file("accepted.bin", b"immutable-verdict");
    let failed = f.file("failed.txt", b"bounded-failure");
    let source = f.file("source.rs", b"fn main() {}");
    let disposable = f.file("temporary.o", &[0u8; 8192]);
    let unknown = f.file("unknown.keep", b"unreferenced-historical");
    let retained = vec![
        Retained { kind: RetainedKind::Accepted, reference: accepted },
        Retained { kind: RetainedKind::Failed, reference: failed },
        Retained { kind: RetainedKind::Source, reference: source },
    ];
    let result =
        retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&disposable), 64 * 1024).unwrap();
    assert_eq!(result.reclaimed_bytes, 8192);
    assert!(!disposable.path.exists());
    assert!(unknown.path.exists());
    retention::authenticate_finalization(&result, 64 * 1024).unwrap();
    assert_eq!(
        retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&disposable), 64 * 1024).unwrap(),
        result
    );
    for r in &retained {
        retention::authenticate_reference(&r.reference, 64 * 1024).unwrap();
    }
    assert!(result.cleanup_records.iter().all(|r| r.path.is_file()));
}

#[test]
fn foreign_disposable_alias_and_changed_closure_are_rejected() {
    let f = Fixture::new();
    let accepted = f.file("accepted", b"accepted");
    let retained = vec![Retained { kind: RetainedKind::Accepted, reference: accepted.clone() }];
    assert!(retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&accepted), 64 * 1024).is_err());
    let outside = Fixture::new();
    let foreign = outside.file("foreign", b"outside");
    assert!(retention::finalize(&f.0, &context(), &retained, &[foreign], 64 * 1024).is_err());
    let alias = f.0.join("alias");
    fs::hard_link(&accepted.path, &alias).unwrap();
    let linked = retention::seal_file(&alias, 64 * 1024).unwrap();
    assert!(retention::finalize(&f.0, &context(), &retained, &[linked], 64 * 1024).is_err());
    let result = retention::finalize(&f.0, &context(), &retained, &[], 64 * 1024).unwrap();
    let mut changed = retained.clone();
    changed[0].kind = RetainedKind::Failed;
    assert!(retention::finalize(&f.0, &context(), &changed, &[], 64 * 1024).is_err());
    fs::write(accepted.path, b"modified").unwrap();
    assert!(retention::authenticate_finalization(&result, 64 * 1024).is_err());
}

#[test]
fn byte_limit_reserves_cleanup_metadata_before_removing_any_file() {
    let f = Fixture::new();
    let keep = f.file("keep", b"accepted");
    let temporary = f.file("temporary", b"temp");
    let retained = vec![Retained { kind: RetainedKind::Accepted, reference: keep }];
    assert!(retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&temporary), 8).is_err());
    assert!(temporary.path.exists(), "metadata budget failure must precede side effects");
}

#[test]
fn interrupted_cleanup_reconciles_exact_missing_file_and_keeps_unknown_data() {
    let f = Fixture::new();
    let keep = f.file("keep", b"accepted");
    let temporary = f.file("temporary", b"temporary-bytes");
    let retained = vec![Retained { kind: RetainedKind::Accepted, reference: keep }];
    let first = retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&temporary), 64 * 1024).unwrap();
    fs::remove_file(&first.cleanup_records[0].path).unwrap();
    fs::remove_file(&first.cleanup_records[1].path).unwrap();
    let unknown = f.file("unknown", b"keep");
    let recovered =
        retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&temporary), 64 * 1024).unwrap();
    assert_eq!(first, recovered);
    assert!(unknown.path.exists());
    retention::authenticate_finalization(&recovered, 64 * 1024).unwrap();
}

#[test]
fn cleanup_record_tampering_and_recreated_intermediate_rejects() {
    let f = Fixture::new();
    let keep = f.file("keep", b"accepted");
    let temporary = f.file("temporary", b"temp");
    let retained = vec![Retained { kind: RetainedKind::Accepted, reference: keep }];
    let result = retention::finalize(&f.0, &context(), &retained, std::slice::from_ref(&temporary), 64 * 1024).unwrap();
    fs::write(&temporary.path, b"recreated").unwrap();
    assert!(retention::authenticate_finalization(&result, 64 * 1024).is_err());
    fs::remove_file(&temporary.path).unwrap();
    fs::set_permissions(&result.cleanup_records[1].path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&result.cleanup_records[1].path, b"{}").unwrap();
    assert!(retention::authenticate_finalization(&result, 64 * 1024).is_err());
}
