#![doc = " Missing setup cannot invent a prepared candidate or accept the scoped run."]
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn missing_setup_retains_terminal_scope_without_binary_identity() {
    let root = std::env::temp_dir()
        .join(format!("sm-parity-admission-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&root).unwrap();
    assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
    let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["readiness-parity", "--root"])
        .arg(&root)
        .args(["--mode", "fallback-only", "--frames", "18000"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let run = fs::read_dir(root.join("out/runs")).unwrap().next().unwrap().unwrap().path();
    let verdict: Value = toml::from_str(&fs::read_to_string(run.join("verdict.toml")).unwrap()).unwrap();
    let manifest: Value = toml::from_str(&fs::read_to_string(run.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!(verdict["status"], toml::Value::try_from("error").unwrap());
    assert_eq!(verdict["acceptance_eligible"], toml::Value::try_from(false).unwrap());
    assert!(manifest["identities"].get("fallback-only-binary").is_none());
    assert_eq!(manifest["scope"]["frames"], toml::Value::try_from(18000).unwrap());
    fs::remove_dir_all(root).unwrap();
}
