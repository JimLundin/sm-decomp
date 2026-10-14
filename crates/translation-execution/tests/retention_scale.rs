use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};
use translation_execution::{
    model::{ExecutionPhase, OperationContext, sha256},
    retention::{self, Reference, Retained, RetainedKind, RetentionResult},
};

const METADATA_LIMIT: u64 = 4 * 1024 * 1024;
const BYTE_LIMIT: u64 = 64 * 1024 * 1024;
const GROUP_REFERENCES: usize = 65_536;
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "translation-retention-scale-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        Self(root)
    }

    fn inventory(&self, count: usize) -> Vec<Retained> {
        let inputs = self.0.join("inputs");
        fs::create_dir(&inputs).unwrap();
        (0..count)
            .map(|index| {
                let path = inputs.join(format!("fixture-{index:06}.bin"));
                fs::write(&path, b"pair").unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
                Retained { kind: RetainedKind::Accepted, reference: retention::seal_file(&path, BYTE_LIMIT).unwrap() }
            })
            .collect()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn context() -> OperationContext {
    OperationContext {
        run_id: sha256("retention-scale-run"),
        attempt: 1,
        policy_sha256: sha256("retention-scale-policy"),
        operation_key: sha256("retention-scale-finalize"),
        phase: ExecutionPhase::Finalization,
    }
}

fn record(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn children(manifest: &Value, key: &str) -> Vec<RetentionResult> {
    serde_json::from_value(manifest[key].clone()).unwrap()
}

fn seal_record(path: &Path, bytes: &[u8]) -> Reference {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o400)).unwrap();
    retention::seal_file(path, BYTE_LIMIT).unwrap()
}

struct RestoreFile {
    path: PathBuf,
    bytes: Vec<u8>,
}

impl RestoreFile {
    fn new(path: &Path) -> Self {
        Self { path: path.to_path_buf(), bytes: fs::read(path).unwrap() }
    }

    fn replace(&self, bytes: &[u8]) -> Reference {
        fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600)).unwrap();
        seal_record(&self.path, bytes)
    }
}

impl Drop for RestoreFile {
    fn drop(&mut self) {
        self.replace(&self.bytes);
    }
}

fn with_changed_manifest(
    result: &RetentionResult,
    change: impl FnOnce(&mut Value),
    check: impl FnOnce(&RetentionResult),
) {
    let restore = RestoreFile::new(&result.manifest.path);
    let mut manifest = record(&result.manifest.path);
    change(&mut manifest);
    let mut changed = result.clone();
    changed.manifest = restore.replace(&serde_json::to_vec(&manifest).unwrap());
    check(&changed);
}

fn assert_metadata_rejected(result: &RetentionResult, reason: &str) {
    let error = retention::retained_entries(result).unwrap_err();
    assert!(error.to_string().contains(reason), "{error:#}");
    assert!(retention::authenticate_finalization(result, BYTE_LIMIT).is_err());
}

fn reject_parent_change(result: &RetentionResult, change: impl FnOnce(&mut Value), reason: &str) {
    with_changed_manifest(result, change, |changed| assert_metadata_rejected(changed, reason));
}

fn reject_group_change(result: &RetentionResult, index: usize, change: impl FnOnce(&mut Value), reason: &str) {
    let group = &children(&record(&result.manifest.path), "groups")[index];
    with_changed_manifest(group, change, |changed| {
        reject_parent_change(
            result,
            |manifest| manifest["groups"][index]["manifest"] = serde_json::to_value(&changed.manifest).unwrap(),
            reason,
        );
    });
}

type Stamp = (PathBuf, u64, u64, SystemTime);

fn stamps<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> Vec<Stamp> {
    paths
        .map(|path| {
            let metadata = path.metadata().unwrap();
            (path.clone(), metadata.ino(), metadata.len(), metadata.modified().unwrap())
        })
        .collect()
}

fn assert_stamps_unchanged(before: &[Stamp], recreated: &[&Path]) {
    for (path, inode, bytes, modified) in before {
        if recreated.contains(&path.as_path()) {
            continue;
        }
        let metadata = path.metadata().unwrap();
        assert_eq!(
            (metadata.ino(), metadata.len(), metadata.modified().unwrap()),
            (*inode, *bytes, *modified),
            "{}",
            path.display()
        );
    }
}

fn record_paths(result: &RetentionResult) -> impl Iterator<Item = &PathBuf> {
    std::iter::once(&result.manifest.path).chain(result.cleanup_records.iter().map(|reference| &reference.path))
}

#[test]
fn hierarchy_preserves_the_exact_inventory_and_unchanged_replay() {
    let fixture = Fixture::new();
    let retained = fixture.inventory(GROUP_REFERENCES + 1);
    let mut requested = retained.iter().rev().cloned().collect::<Vec<_>>();
    requested.push(retained[0].clone());
    let result = retention::finalize(&fixture.0, &context(), &requested, &[], BYTE_LIMIT).unwrap();
    let manifest = record(&result.manifest.path);
    assert_eq!(manifest["schema"], "translation-retention-groups-v1");
    assert_eq!(manifest["reference_count"], GROUP_REFERENCES + 1);
    let groups = children(&manifest, "groups");
    assert_eq!(groups.len(), 2);
    let full_group = record(&groups[0].manifest.path);
    assert_eq!(full_group["schema"], "translation-retention-pages-v1");
    assert_eq!(full_group["reference_count"], GROUP_REFERENCES);
    let pages = children(&full_group, "pages");
    assert_eq!(pages.len(), 16);
    assert_eq!(record(&groups[1].manifest.path)["retained"].as_array().unwrap().len(), 1);
    for node in std::iter::once(&result).chain(&groups).chain(&pages) {
        assert!(node.manifest.bytes <= METADATA_LIMIT);
        assert!(node.cleanup_records.iter().all(|reference| reference.bytes <= METADATA_LIMIT));
        assert_eq!(node.reclaimed_bytes, 0);
    }
    assert_eq!(retention::retained_entries(&result).unwrap(), retained);
    retention::authenticate_finalization(&result, BYTE_LIMIT).unwrap();
    let before = stamps(
        std::iter::once(&result)
            .chain(&groups)
            .chain(&pages)
            .flat_map(record_paths)
            .chain(retained.iter().map(|item| &item.reference.path)),
    );
    // Recover missing observations at different depths without rewriting any
    // surviving prefix or changing the original full closure.
    let recreated = [&result.cleanup_records[1].path, &pages[0].cleanup_records[0].path];
    for path in recreated {
        fs::remove_file(path).unwrap();
    }
    assert_eq!(retention::finalize(&fixture.0, &context(), &retained, &[], BYTE_LIMIT).unwrap(), result);
    assert_stamps_unchanged(&before, &recreated.map(PathBuf::as_path));
    assert!(recreated.iter().all(|path| path.is_file()));

    let mut conflict = requested.clone();
    conflict.last_mut().unwrap().kind = RetainedKind::Failed;
    assert!(retention::finalize(&fixture.0, &context(), &conflict, &[], BYTE_LIMIT).is_err());
    *conflict.last_mut().unwrap() = retained[0].clone();
    conflict.last_mut().unwrap().reference.bytes += 1;
    assert!(retention::finalize(&fixture.0, &context(), &conflict, &[], BYTE_LIMIT).is_err());
    assert!(retention::finalize(&fixture.0, &context(), &retained[..GROUP_REFERENCES], &[], BYTE_LIMIT).is_err());
    assert!(
        retention::finalize(&fixture.0, &context(), &retained, &[retained[0].reference.clone()], BYTE_LIMIT).is_err()
    );

    let mut changed_context = context();
    changed_context.run_id = sha256("other-run");
    assert!(retention::finalize(&fixture.0, &changed_context, &retained, &[], BYTE_LIMIT).is_err());
    assert!(retention::finalize(&fixture.0, &context(), &retained, &[], BYTE_LIMIT + 1).is_err());
    assert!(retention::authenticate_finalization(&result, BYTE_LIMIT - 1).is_err());

    let mut budget_context = context();
    budget_context.operation_key = sha256("insufficient-hierarchy-budget");
    let error =
        retention::finalize(&fixture.0, &budget_context, &retained, &[], result.retained_bytes - 1).unwrap_err();
    assert!(error.to_string().contains("aggregate grouped retention plus metadata exceeds budget"), "{error:#}");
    let refused_root = fixture.0.join(format!("retention-{}", budget_context.operation_key));
    assert!(!refused_root.join("manifest.json").exists());
    assert!(!refused_root.join("groups").exists(), "all nested metadata must be reserved before publishing a child");

    for (field, value) in [
        ("run_id", json!(sha256("changed-run"))),
        ("attempt", json!(2)),
        ("policy_sha256", json!(sha256("changed-policy"))),
        ("phase", json!("publication")),
    ] {
        reject_parent_change(
            &result,
            |manifest| manifest["context"][field] = value,
            "retention group page identity or bound changed",
        );
    }
    reject_parent_change(
        &result,
        |manifest| manifest["byte_limit"] = json!(BYTE_LIMIT + 1),
        "retention group page identity or bound changed",
    );
    reject_group_change(
        &result,
        1,
        |manifest| manifest["context"]["operation_key"] = json!(sha256("changed-child")),
        "retention group leaf identity changed",
    );
    reject_group_change(
        &result,
        1,
        |manifest| manifest["byte_limit"] = json!(BYTE_LIMIT + 1),
        "retention group leaf identity changed",
    );

    let relocated = fixture.0.join("relocated-manifest.json");
    let relocated = seal_record(&relocated, &fs::read(&groups[0].manifest.path).unwrap());
    reject_parent_change(
        &result,
        |manifest| manifest["groups"][0]["manifest"] = serde_json::to_value(&relocated).unwrap(),
        "retention group location differs from original operation",
    );
    let relocated_page = fixture.0.join("relocated-page.json");
    let relocated_page = seal_record(&relocated_page, &fs::read(&pages[0].manifest.path).unwrap());
    reject_group_change(
        &result,
        0,
        |manifest| manifest["pages"][0]["manifest"] = serde_json::to_value(&relocated_page).unwrap(),
        "retention page location differs from original operation",
    );

    reject_parent_change(
        &result,
        |manifest| manifest["reference_count"] = json!(GROUP_REFERENCES * 256 + 1),
        "grouped retention inventory exceeds hard bounds",
    );
    reject_parent_change(
        &result,
        |manifest| manifest["groups"] = json!(vec![groups[0].clone(); 257]),
        "grouped retention inventory exceeds hard bounds",
    );
    reject_parent_change(
        &result,
        |manifest| manifest["reference_count"] = json!(GROUP_REFERENCES + 2),
        "retention groups duplicated, reordered or omitted a reference",
    );
    reject_group_change(
        &result,
        0,
        |manifest| manifest["reference_count"] = json!(GROUP_REFERENCES + 1),
        "retention group page identity or bound changed",
    );
    reject_group_change(
        &result,
        0,
        |manifest| manifest["pages"].as_array_mut().unwrap().push(serde_json::to_value(&pages[0]).unwrap()),
        "retention group page identity or bound changed",
    );
    reject_group_change(
        &result,
        0,
        |manifest| {
            manifest["pages"].as_array_mut().unwrap().pop();
            manifest["reference_count"] = json!(GROUP_REFERENCES - 4096);
        },
        "retention group prefix is incomplete",
    );
    reject_group_change(
        &result,
        1,
        |manifest| manifest["retained"][0] = serde_json::to_value(&retained[0]).unwrap(),
        "retention groups duplicated, reordered or omitted a reference",
    );
    reject_group_change(
        &result,
        0,
        |manifest| {
            *manifest = json!({
                "schema": "translation-retention-groups-v1",
                "context": manifest["context"],
                "byte_limit": BYTE_LIMIT,
                "reference_count": GROUP_REFERENCES + 1,
                "groups": [result],
            });
        },
        "retention groups must contain only leaf or paged records",
    );

    let oversized = RestoreFile::new(&groups[1].manifest.path);
    let mut bytes = oversized.bytes.clone();
    bytes.resize(METADATA_LIMIT as usize + 1, b' ');
    let oversized_reference = oversized.replace(&bytes);
    reject_parent_change(
        &result,
        |manifest| manifest["groups"][1]["manifest"] = serde_json::to_value(&oversized_reference).unwrap(),
        "invalid retention reference",
    );
    drop(oversized);

    let relocated_cleanup =
        seal_record(&fixture.0.join("relocated-cleanup.json"), &fs::read(&groups[1].cleanup_records[0].path).unwrap());
    with_changed_manifest(
        &result,
        |manifest| manifest["groups"][1]["cleanup_records"][0] = serde_json::to_value(&relocated_cleanup).unwrap(),
        |changed| {
            assert_eq!(retention::retained_entries(changed).unwrap(), retained);
            let error = retention::authenticate_finalization(changed, BYTE_LIMIT).unwrap_err();
            assert!(error.to_string().contains("retention leaf cleanup location or extent changed"), "{error:#}");
        },
    );

    for path in
        [&groups[0].manifest.path, &groups[1].cleanup_records[1].path, &retained[GROUP_REFERENCES].reference.path]
    {
        let changed = RestoreFile::new(path);
        changed.replace(b"changed");
        assert!(retention::authenticate_finalization(&result, BYTE_LIMIT).is_err());
    }
    retention::authenticate_finalization(&result, BYTE_LIMIT).unwrap();
    assert_eq!(retention::retained_entries(&result).unwrap(), retained);
}

// These wire fixtures pin the historical field order, independently of today's
// choice of format. Public finalize builds their original leaf records.
#[derive(Serialize)]
struct HistoricalPages {
    schema: &'static str,
    context: OperationContext,
    byte_limit: u64,
    reference_count: usize,
    pages: Vec<RetentionResult>,
}

#[derive(Serialize)]
struct HistoricalCleanup {
    schema: &'static str,
    manifest_sha256: String,
    pass: u8,
    retained_closure_bytes: u64,
    reclaimed_bytes: u64,
    allocated_bytes_reclaimed: u64,
    removed: Vec<Reference>,
    retained_authenticatable: bool,
}

#[test]
fn historical_paged_record_above_the_group_threshold_replays_without_upgrade() {
    let fixture = Fixture::new();
    let retained = fixture.inventory(GROUP_REFERENCES + 1);
    let context = context();
    let root = fixture.0.join(format!("retention-{}", context.operation_key));
    let mut pages = Vec::new();
    for (index, entries) in retained.chunks(4096).enumerate() {
        let mut child = context.clone();
        child.operation_key = sha256(format!("retention-page-v1:{}:{index}", context.operation_key));
        let child_root = root.join("pages").join(format!("{index:04}"));
        pages.push(retention::finalize(&child_root, &child, entries, &[], BYTE_LIMIT).unwrap());
    }
    assert_eq!(pages.len(), 17, "historical pages may exceed a new group's 16-page bound");
    let child_bytes = pages.iter().map(|page| page.retained_bytes).sum::<u64>();
    let manifest = HistoricalPages {
        schema: "translation-retention-pages-v1",
        context: context.clone(),
        byte_limit: BYTE_LIMIT,
        reference_count: retained.len(),
        pages,
    };
    let manifest_reference = seal_record(&root.join("manifest.json"), &serde_json::to_vec(&manifest).unwrap());
    let cleanup_records = [1, 2].map(|pass| {
        seal_record(
            &root.join(format!("cleanup-{pass}.json")),
            &serde_json::to_vec(&HistoricalCleanup {
                schema: "translation-cleanup-v1",
                manifest_sha256: manifest_reference.sha256.clone(),
                pass,
                retained_closure_bytes: child_bytes,
                reclaimed_bytes: 0,
                allocated_bytes_reclaimed: 0,
                removed: vec![],
                retained_authenticatable: true,
            })
            .unwrap(),
        )
    });
    let result = RetentionResult {
        retained_bytes: child_bytes + manifest_reference.bytes + cleanup_records.iter().map(|r| r.bytes).sum::<u64>(),
        manifest: manifest_reference,
        cleanup_records,
        reclaimed_bytes: 0,
    };
    retention::authenticate_finalization(&result, BYTE_LIMIT).unwrap();
    assert_eq!(retention::retained_entries(&result).unwrap(), retained);
    let before = stamps(std::iter::once(&result).chain(&manifest.pages).flat_map(record_paths));
    assert_eq!(retention::finalize(&fixture.0, &context, &retained, &[], BYTE_LIMIT).unwrap(), result);
    assert_stamps_unchanged(&before, &[]);
    assert_eq!(record(&result.manifest.path)["schema"], "translation-retention-pages-v1");
    assert!(!root.join("groups").exists());
}
