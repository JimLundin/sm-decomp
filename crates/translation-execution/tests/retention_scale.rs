use serde_json::Value;
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use translation_execution::{
    model::{ExecutionPhase, OperationContext, sha256},
    retention::{self, Retained, RetainedKind, RetentionResult},
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

#[test]
fn hierarchy_preserves_the_exact_inventory_and_unchanged_replay() {
    let fixture = Fixture::new();
    let retained = fixture.inventory(GROUP_REFERENCES + 1);
    let result = retention::finalize(&fixture.0, &context(), &retained, &[], BYTE_LIMIT).unwrap();
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
    assert_eq!(retention::finalize(&fixture.0, &context(), &retained, &[], BYTE_LIMIT).unwrap(), result);
}
