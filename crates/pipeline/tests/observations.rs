#![doc = " Packed authentication rejects changed pointers, readers and escaping store paths."]
use evidence::{Pointer, Provenance, SourceIdentity};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
#[derive(Serialize, Deserialize)]
struct Sealed {
    pointer_sha256: String,
    run: String,
    observations: String,
}
#[derive(Serialize, Deserialize)]
struct Storage {
    format: String,
    subjects: BTreeMap<String, Sealed>,
}
#[derive(Serialize, Deserialize)]
struct Verdict {
    state_effects_match: bool,
    binary_sha256: String,
    evidence_reader_sha256: String,
    artifacts: BTreeMap<String, String>,
    evidence_storage: Storage,
}
#[derive(Serialize)]
struct Request {
    verdict: PathBuf,
    root: PathBuf,
    workers: usize,
}
struct Run(PathBuf);
impl Drop for Run {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn hash(data: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(data))
}
fn save<T: Serialize>(path: &Path, data: &T) {
    fs::write(path, toml::to_string(data).unwrap()).unwrap();
}
impl Run {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-observation-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
        let reader = std::env::var_os("SM_EVIDENCE_TEST_BINARY")
            .map(PathBuf::from)
            .unwrap_or_else(|| checkout.join("target/debug/sm-evidence"));
        assert!(reader.is_file(), "build current native evidence reader before observation controls");
        fs::copy(reader, path.join("sm-evidence")).unwrap();
        fs::write(path.join("live"), b"frozen binary").unwrap();
        let mut subjects = BTreeMap::new();
        for subject in ["c-0", "c-1", "mixed-0", "mixed-1"] {
            let directory = path.join(subject);
            fs::create_dir(&directory).unwrap();
            let input = directory.join("input");
            fs::create_dir(&input).unwrap();
            fs::write(input.join("terminal.bin"), b"identical observed state").unwrap();
            let provenance = Provenance {
                subject: Some(subject.into()),
                binary_sha256: Some(hash(b"frozen binary")),
                reader_sha256: Some(hash(fs::read(path.join("sm-evidence")).unwrap())),
                source: Some(SourceIdentity {
                    revision: "pin".into(),
                    archive_sha256: "a".repeat(64),
                    files: BTreeMap::new(),
                    freshness: "verified".into(),
                }),
                ..Default::default()
            };
            save(&directory.join("provenance.toml"), &provenance);
            let mut store = evidence::Store::create(&directory.join("store")).unwrap();
            let tree = evidence::ingest_tree(&mut store, &input, &mut |_| {}).unwrap();
            let run = evidence::put_run(&mut store, tree, &provenance).unwrap();
            store.commit().unwrap();
            let store = evidence::Store::open(&directory.join("store")).unwrap();
            let (run_info, inventory) = evidence::verified_inventory(&store, &run).unwrap();
            let pointer = Pointer {
                format: "sm-evidence-v2".into(),
                run: evidence::hex(&run),
                observations: evidence::hex(&tree),
                files: run_info.files,
                bytes: run_info.bytes,
                artifacts: inventory.iter().map(|(k, v)| (k.clone(), evidence::hex(&v.sha256))).collect(),
                sizes: inventory.iter().map(|(k, v)| (k.clone(), v.bytes)).collect(),
                provenance,
                store: Some(PathBuf::from("store")),
                pipeline_seconds: None,
                producer_seconds: None,
                seal_seconds: None,
                stored_added_bytes: None,
                worker_wait_seconds: None,
                store_wait_seconds: None,
                publication_seconds: None,
            };
            save(&directory.join("evidence.toml"), &pointer);
            subjects.insert(
                subject.into(),
                Sealed {
                    pointer_sha256: hash(fs::read(directory.join("evidence.toml")).unwrap()),
                    run: pointer.run,
                    observations: pointer.observations,
                },
            );
        }
        save(
            &path.join("verdict.toml"),
            &Verdict {
                state_effects_match: true,
                binary_sha256: hash(b"frozen binary"),
                evidence_reader_sha256: hash(fs::read(path.join("sm-evidence")).unwrap()),
                artifacts: BTreeMap::from([("terminal.bin".into(), hash(b"identical observed state"))]),
                evidence_storage: Storage { format: "sm-evidence-v2".into(), subjects },
            },
        );
        Self(path)
    }
    fn verify(&self, success: bool) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .arg("live-authenticate")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(
                toml::to_string(&Request {
                    verdict: self.0.join("verdict.toml"),
                    root: self.0.join("workers"),
                    workers: 2,
                })
                .unwrap()
                .as_bytes(),
            )
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.success(), success, "{}", String::from_utf8_lossy(&output.stderr));
    }
}
#[test]
fn retained_packed_observations_authenticate_and_changed_pointer_is_rejected() {
    let run = Run::new();
    run.verify(true);
    fs::write(run.0.join("mixed-1/evidence.toml"), b"changed").unwrap();
    run.verify(false);
}
#[test]
fn changed_binary_and_incomplete_inventory_are_rejected() {
    let run = Run::new();
    fs::write(run.0.join("live"), b"changed").unwrap();
    run.verify(false);
    fs::write(run.0.join("live"), b"frozen binary").unwrap();
    let path = run.0.join("verdict.toml");
    let mut verdict: Verdict = toml::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    verdict.artifacts.clear();
    save(&path, &verdict);
    run.verify(false);
}
#[test]
fn reader_identity_is_checked_before_execution() {
    let run = Run::new();
    fs::write(run.0.join("sm-evidence"), b"changed reader").unwrap();
    run.verify(false);
}
#[cfg(unix)]
#[test]
fn symlinked_subject_pointer_is_rejected() {
    let run = Run::new();
    let pointer = run.0.join("mixed-0/evidence.toml");
    fs::remove_file(&pointer).unwrap();
    std::os::unix::fs::symlink(run.0.join("c-0/evidence.toml"), pointer).unwrap();
    run.verify(false);
}
