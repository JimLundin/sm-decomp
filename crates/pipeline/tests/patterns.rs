//! Real emitted Rust steps against literal expectations and corrupted evidence controls.
use serde_json::{Value, json};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};
fn call(checkout: &Path, flags: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sm-pipeline")).current_dir(checkout).args(flags).output().unwrap()
}
fn external_document(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}
fn document(path: &Path) -> toml::Value {
    toml::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}
fn private_checkout(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    fn copy(source: &Path, destination: &Path) {
        if source.is_dir() {
            fs::create_dir_all(destination).unwrap();
            for entry in fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                if ["target", "generated", "__pycache__"].contains(&entry.file_name().to_str().unwrap()) {
                    continue;
                }
                copy(&entry.path(), &destination.join(entry.file_name()));
            }
        } else {
            fs::copy(source, destination).unwrap();
        }
    }
    for name in [
        "crates",
        "tools",
        "config",
        ".cargo",
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "rustfmt.toml",
        "justfile",
    ] {
        if source.join(name).exists() {
            copy(&source.join(name), &destination.join(name));
        }
    }
    std::os::unix::fs::symlink(source.join("deps"), destination.join("deps")).unwrap();
    assert!(Command::new("git").args(["init", "-q"]).arg(destination).status().unwrap().success());
}
#[test]
fn actual_step_ledger_rejects_wrong_expectation_scope_and_artifact_substitution() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let root = std::env::temp_dir().join(format!(
        "sm-pattern-controls-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    let corpus = root.join("corpus");
    fs::create_dir_all(&corpus).unwrap();
    let snapshot = root.join("checkout");
    private_checkout(checkout, &snapshot);
    let checkout = snapshot.as_path();
    let initial = json!({"pc":0xffff,"pbr":0x80,"s":0x1ff,"p":0x30,"a":0xab00,"x":0,"y":0,"dbr":0,"d":0,"e":0,"ram":[[0x80ffff,0xa9],[0x800000,0x80]]});
    let mut final_state = initial.clone();
    final_state["pc"] = 1.into();
    final_state["a"] = 0xab80.into();
    final_state["p"] = 0xb0.into();
    let mut case = json!({"name":"independent LDA wraps PC and preserves high A","initial":initial,"final":final_state,"cycles":[[0x80ffff,0xa9,"dp-r-mx-"],[0x800000,0x80,"-p-r-mx-"]]});
    fs::write(corpus.join("a9.n.json"), serde_json::to_vec(&json!([case])).unwrap()).unwrap();
    let good = root.join("good");
    let result = call(
        checkout,
        &["pattern-check", "--corpus", corpus.to_str().unwrap(), "--opcodes", "a9", "--out", good.to_str().unwrap()],
    );
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stderr),
        String::from_utf8_lossy(&result.stdout)
    );
    let report = document(&good.join("instruction.toml"));
    assert_eq!(
        (report["expected"].as_integer(), report["passed"].as_integer(), report["skipped"].as_integer()),
        (Some(3), Some(3), Some(0))
    );
    let original = fs::read(good.join("instruction.toml")).unwrap();
    let auth = || call(checkout, &["pattern-authenticate", "--out", good.to_str().unwrap(), "--development-scope"]);
    let result = auth();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    assert!(!call(checkout, &["pattern-authenticate", "--out", good.to_str().unwrap()]).status.success());
    let mut changed = report.clone();
    changed["declaration"]["patterns"][0]["mx"] = toml::Value::Array(vec![toml::Value::Integer(3)]);
    fs::write(good.join("instruction.toml"), toml::to_string(&changed).unwrap().into_bytes()).unwrap();
    assert!(!auth().status.success());
    fs::write(good.join("instruction.toml"), &original).unwrap();
    let binary = good.join("batch-0000/timed-run");
    let bytes = fs::read(&binary).unwrap();
    let mut corrupt = bytes.clone();
    corrupt.push(0);
    fs::write(&binary, &corrupt).unwrap();
    assert!(!auth().status.success());
    fs::write(&binary, &bytes).unwrap();
    let expected = good.join("expected-cases.bin");
    let bytes = fs::read(&expected).unwrap();
    fs::write(&expected, b"{}\n").unwrap();
    assert!(!auth().status.success());
    fs::write(&expected, bytes).unwrap();
    case["final"]["a"] = 0xab81.into();
    fs::write(corpus.join("a9.n.json"), serde_json::to_vec(&json!([case])).unwrap()).unwrap();
    let bad = root.join("bad");
    let result = call(
        checkout,
        &["pattern-check", "--corpus", corpus.to_str().unwrap(), "--opcodes", "a9", "--out", bad.to_str().unwrap()],
    );
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    assert_eq!(document(&bad.join("instruction.toml"))["failed"].as_integer(), Some(1));
    let declaration = root.join("reduced.toml");
    fs::write(&declaration, b"schema_version = 2\n").unwrap();
    let rejected = root.join("rejected");
    assert_eq!(
        call(
            checkout,
            &["pattern-check", "--declaration", declaration.to_str().unwrap(), "--out", rejected.to_str().unwrap()]
        )
        .status
        .code(),
        Some(2)
    );
    assert!(document(&rejected.join("instruction.toml"))["reason"].as_str().unwrap().contains("declaration"));
    for opcode in ["ea", "ca"] {
        let cases = external_document(&checkout.join(format!("deps/65816/v1/{opcode}.n.json")));
        fs::write(corpus.join(format!("{opcode}.n.json")), serde_json::to_vec(&json!([cases[0]])).unwrap()).unwrap();
    }
    let reordered = root.join("reordered");
    let result = call(
        checkout,
        &[
            "pattern-check",
            "--corpus",
            corpus.to_str().unwrap(),
            "--opcodes",
            "ea,ca",
            "--out",
            reordered.to_str().unwrap(),
        ],
    );
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let result = call(checkout, &["pattern-authenticate", "--out", reordered.to_str().unwrap(), "--development-scope"]);
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    fs::remove_dir_all(root).unwrap();
}
