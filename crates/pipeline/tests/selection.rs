#![doc = " Selection admission authenticates an accepted owner before compiling any candidate."]
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn missing_or_forged_acceptance_is_rejected_before_build() {
    let root = std::env::temp_dir().join(format!(
        "sm-native-selection-admission-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
    fs::create_dir(root.join("out")).unwrap();
    fs::write(root.join("out/accepted.toml"), "run_path = \"/missing/accepted-owner\"\n").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["iteration-selection", "--root"])
        .arg(&root)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let owner = fs::read_dir(root.join("out/runs")).unwrap().next().unwrap().unwrap().path();
    let manifest: Value = toml::from_str(&fs::read_to_string(owner.join("manifest.toml")).unwrap()).unwrap();
    let verdict: Value = toml::from_str(&fs::read_to_string(owner.join("verdict.toml")).unwrap()).unwrap();
    assert_eq!(verdict["status"], toml::Value::try_from("error").unwrap());
    assert_eq!(verdict["acceptance_eligible"], toml::Value::try_from(false).unwrap());
    assert!(manifest["commands"].as_array().unwrap().is_empty());
    assert!(manifest["identities"].get("selection-generator").is_none());
    fs::remove_dir_all(root).unwrap();
}
