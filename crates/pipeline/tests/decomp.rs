//! Exercise the public job against independent whole-memory and store-order expectations.
#[path = "../src/source_stamps.rs"]
mod source_stamps;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
struct Run(PathBuf);
fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}
fn snapshot(checkout: &Path, root: &Path) -> PathBuf {
    let snapshot = root.join("checkout");
    fs::create_dir(&snapshot).unwrap();
    for relative in ["crates/decomp", "crates/pipeline/src", "tools/decomp"] {
        copy_tree(&checkout.join(relative), &snapshot.join(relative));
    }
    for (name, source) in source_stamps::all() {
        let path = snapshot.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    fs::copy(checkout.join("Cargo.lock"), snapshot.join("Cargo.lock")).unwrap();
    fs::copy(checkout.join("tools/dependencies.toml"), snapshot.join("tools/dependencies.toml")).unwrap();
    copy_tree(&checkout.join("deps/snesrev-sm"), &snapshot.join("deps/snesrev-sm"));
    fs::create_dir_all(snapshot.join("deps/downloads")).unwrap();
    fs::copy(checkout.join("deps/downloads/snesrev-sm.tar.gz"), snapshot.join("deps/downloads/snesrev-sm.tar.gz"))
        .unwrap();
    snapshot
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn invoke(request: Value, success: bool) -> Value {
    invoke_command("decomp-verify", request, success)
}
fn invoke_command(command: &str, request: Value, success: bool) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{}", toml::to_string(&request).unwrap()).unwrap();
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.success(), success, "{}", String::from_utf8_lossy(&result.stderr));
    if result.stdout.is_empty() {
        Value::Table(toml::Table::new())
    } else {
        toml::from_str(&String::from_utf8(result.stdout).unwrap()).unwrap()
    }
}
#[test]
fn synthetic_all_modes_and_transient_store_rejection_preserve_partial_scope() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    if !checkout.join("deps/snesrev-sm/src/sm_80.c").exists() {
        return;
    }
    let root = Run(std::env::temp_dir()
        .join(format!("sm-decomp-job-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
    fs::create_dir(&root.0).unwrap();
    // Acceptance rechecks current sources; hold a private snapshot throughout the
    // controls so development in the shared worktree cannot invalidate the test.
    let snapshot = snapshot(checkout, &root.0);
    let checkout = snapshot.as_path();
    let request = Value::Table(toml::Table::from_iter([
        ("checkout".to_owned(), Value::try_from(checkout).unwrap()),
        ("output".to_owned(), Value::try_from(root.0.join("out")).unwrap()),
        ("root".to_owned(), Value::try_from(root.0.join("workers")).unwrap()),
        ("workers".to_owned(), Value::Integer(4_i64)),
    ]));
    let verdict = invoke(request.clone(), true);
    assert_eq!(verdict["status"].as_str(), Some("partial"));
    assert_eq!(verdict["routine_accepted"].as_bool(), Some(false));
    assert_eq!(verdict["synthetic_verified"].as_bool(), Some(true));
    assert_eq!(verdict["missing_cases"], Value::Array(vec![]));
    let manifest: Value =
        toml::from_str(&fs::read_to_string(checkout.join("tools/decomp/decompress-job.toml")).unwrap()).unwrap();
    let mut required = manifest["required_cases"].as_array().unwrap().clone();
    required.sort_by_key(Value::to_string);
    assert_eq!(verdict["observed_cases"], Value::try_from(&(required)).unwrap());
    for fixture in verdict["fixtures"].as_array().unwrap() {
        assert_eq!(fixture["status"].as_str(), Some("matched"));
        assert_eq!(fixture["subject_hashes"]["c"], fixture["expected_sha256"]);
        assert_eq!(fixture["subject_hashes"]["rust"], fixture["expected_sha256"]);
        assert_eq!(fixture["c_ordered_writes_sha256"], fixture["rust_ordered_writes_sha256"]);
    }
    let mut fault = request.clone();
    fault.as_table_mut().unwrap().insert("fault".into(), "write-restore".into());
    let rejected = invoke(fault, false);
    assert!(rejected["error"].as_str().unwrap().contains("ordered WRAM stores differ"));
    assert_eq!(rejected["routine_accepted"].as_bool(), Some(false));
    assert_eq!(rejected["fixtures"][0]["subject_hashes"]["c"], rejected["fixtures"][0]["subject_hashes"]["rust"]);
    let mut invalid = manifest;
    invalid["fixtures"][0]["destination"] = 0x1ffff.into();
    let path = root.0.join("invalid.toml");
    fs::write(&path, toml::to_string(&invalid).unwrap()).unwrap();
    let mut invalid_request = request;
    invalid_request.as_table_mut().unwrap().insert("manifest".into(), Value::try_from(&path).unwrap());
    let verdict = invoke(invalid_request, false);
    assert_eq!(verdict["stages"], Value::Array(vec![]));
    assert!(verdict["error"].as_str().unwrap().contains("fixture destination"));
}

#[test]
fn complete_acceptance_rejects_sparse_artifacts_and_preserves_prior_pointer_on_failed_attempt() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let Some(captures) = std::env::var_os("SM_DECOMP_CAPTURE_MANIFEST").map(PathBuf::from) else {
        return;
    };
    let rom = checkout.join("rom/sm.sfc");
    if !captures.exists() || !rom.exists() {
        return;
    }
    let root = Run(std::env::temp_dir().join(format!(
        "sm-native-acceptance-test-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    )));
    fs::create_dir(&root.0).unwrap();
    let snapshot = snapshot(checkout, &root.0);
    let checkout = snapshot.as_path();
    let request = Value::Table(toml::Table::from_iter([
        ("checkout".to_owned(), Value::try_from(checkout).unwrap()),
        ("output".to_owned(), Value::try_from(root.0.join("out")).unwrap()),
        ("root".to_owned(), Value::try_from(root.0.join("workers")).unwrap()),
        ("workers".to_owned(), Value::Integer(4_i64)),
        ("captures".to_owned(), Value::try_from(&(captures)).unwrap()),
        ("original_rom".to_owned(), Value::try_from(&(rom)).unwrap()),
        ("accept".to_owned(), Value::Boolean(true)),
    ]));
    let verdict = invoke(request.clone(), true);
    assert_eq!(verdict["status"].as_str(), Some("accepted"));
    assert_eq!(verdict["fixtures"].as_array().unwrap().len(), 63);
    assert_eq!(verdict["negative_controls"].as_array().unwrap().len(), 11);
    assert!(
        verdict["negative_controls"]
            .as_array()
            .unwrap()
            .iter()
            .all(|control| control["rejected"].as_bool() == Some(true))
    );
    let pointer = root.0.join("out/decomp-acceptance.toml");
    let auth = Value::Table(toml::Table::from_iter([
        ("checkout".to_owned(), Value::try_from(checkout).unwrap()),
        ("pointer".to_owned(), Value::try_from(&(pointer)).unwrap()),
    ]));
    invoke_command("decomp-authenticate", auth.clone(), true);
    let run = Path::new(verdict["run"].as_str().unwrap());
    let original_id = verdict["original_rom_parity"]["fixtures"][0]["id"].as_str().unwrap();
    for omitted in [
        format!("{}/c.wram", verdict["fixtures"][0]["id"].as_str().unwrap()),
        format!("{original_id}/original.wram"),
        "original-oracle".into(),
        "original_cpu.c".into(),
    ] {
        let artifact = run.join(omitted);
        let moved = artifact.with_extension("temporarily-missing");
        fs::rename(&artifact, &moved).unwrap();
        invoke_command("decomp-authenticate", auth.clone(), false);
        fs::rename(moved, artifact).unwrap();
    }
    let saved = fs::read(&pointer).unwrap();
    let mut failure = request;
    failure.as_table_mut().unwrap().remove("captures");
    failure.as_table_mut().unwrap().remove("original_rom");
    failure["accept"] = false.into();
    failure.as_table_mut().unwrap().insert("fault".into(), "crash".into());
    assert_eq!(invoke(failure, false)["routine_accepted"].as_bool(), Some(false));
    assert_eq!(fs::read(&pointer).unwrap(), saved);
    invoke_command("decomp-authenticate", auth, true);
}
