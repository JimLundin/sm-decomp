use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};
fn output(program: &str, args: &[&str]) -> String {
    let out = Command::new(program).args(args).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn hash(path: &Path) -> String {
    output("sha256sum", &[path.to_str().unwrap()]).split_whitespace().next().unwrap().into()
}
fn reference(path: &Path) -> Value {
    json!({"path":path,"sha256":hash(path),"bytes":fs::metadata(path).unwrap().len()})
}
fn write(path: &Path, value: &Value) {
    let bytes = serde_json::to_vec_pretty(value).unwrap();
    if path.exists() { assert_eq!(fs::read(path).unwrap(), bytes) } else { fs::write(path, bytes).unwrap() }
}
fn copy(source: &Path, target: &Path, relative: &Path, files: &mut BTreeMap<PathBuf, String>, bytes: &mut u64) {
    let mut entries = fs::read_dir(source.join(relative)).unwrap().map(|e| e.unwrap().path()).collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        let part = path.strip_prefix(source).unwrap();
        let meta = fs::symlink_metadata(&path).unwrap();
        assert!(!meta.file_type().is_symlink());
        if meta.is_dir() {
            fs::create_dir_all(target.join(part)).unwrap();
            copy(source, target, part, files, bytes)
        } else {
            assert!(meta.is_file());
            *bytes += meta.len();
            assert!(*bytes <= 64 * 1024 * 1024 && files.len() < 4096);
            let data = fs::read(&path).unwrap();
            let dest = target.join(part);
            if dest.exists() {
                assert_eq!(fs::read(&dest).unwrap(), data)
            } else {
                fs::write(&dest, data).unwrap()
            }
            let digest = hash(&path);
            assert_eq!(digest, hash(&dest));
            files.insert(part.into(), digest);
        }
    }
}
fn main() {
    let main = Path::new("/home/jim/sm-decomp");
    let architecture = Path::new("/home/jim/sm-decomp-translation-execution");
    let effort = architecture.join(".scratch/translation-scope-execution");
    let root = effort.join("production-migration");
    let pinned = Path::new("/home/jim/sm-decomp-production-inputs/snesrev-sm");
    fs::create_dir_all(pinned).unwrap();
    let mut files = BTreeMap::new();
    let mut bytes = 0;
    copy(&main.join("deps/snesrev-sm"), pinned, Path::new(""), &mut files, &mut bytes);
    write(
        &root.join("pinned-inputs.json"),
        &json!({"schema":"production-pinned-input-copy-v1","source":main.join("deps/snesrev-sm"),"destination":pinned,"bytes":bytes,"files":files}),
    );
    let template = effort.join("model-launch/native-02/execution/scopes/enemy-jump-instructions/installation.json");
    let config = json!({"schema":"translation-production-prepare-v1","output_root":"/home/jim/sm-decomp-production-execution","packages":[reference(&effort.join("production-preparation/ceres-production-01/package/migration.json")),reference(&effort.join("production-preparation/canonical-production-01/package/migration.json"))],"helper":reference(Path::new("/home/jim/sm-decomp/out/translation-production/sm-translation-sandbox")),"template":reference(&template),"pinned_source":pinned,"integration_baseline":output("git",&["-C",main.to_str().unwrap(),"rev-parse","HEAD"]),"executable":[{"effort":"ceres-continuation","claim_id":"shared-game-host","kind":"shared-game-host","submission":reference(&main.join(".scratch/ceres-continuation/host-implementation.toml")),"acceptance_inputs":[reference(&main.join(".scratch/decomp-parallel/run-20261013/host/diagnosis/findings.md")),reference(&main.join(".scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml"))]},{"effort":"ceres-continuation","claim_id":"compact-controller","kind":"compact-controller","submission":reference(&main.join(".scratch/ceres-continuation/controller-submission-evidence.toml")),"acceptance_inputs":[reference(&main.join(".scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml"))]}],"blocked":[{"effort":"decomp-parallel","claim_id":"pause-equipment-render-offline","reason":"Original submitted source and 607 actual-C fixture pairs plus 10 Rust tests are preserved. A confined composite adapter for the full original acceptance, including auxiliary direct animation, is required before deterministic execution. Existing held Ceres integration remains a dependency."}]});
    write(&root.join("production-config.json"), &config);
    println!("Prepared production configuration and authenticated {bytes} bytes of pinned C inputs.");
}
