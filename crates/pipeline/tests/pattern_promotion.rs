#![doc = " Invalid proof cannot mutate source or start candidate generation."]
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn missing_proof_preserves_config_and_declaration() {
    let root = std::env::temp_dir()
        .join(format!("sm-pattern-promotion-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir(&root).unwrap();
    assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
    fs::create_dir(root.join("config")).unwrap();
    fs::create_dir(root.join("proof")).unwrap();
    fs::create_dir(root.join("selection")).unwrap();
    fs::create_dir_all(root.join("crates/recomp/src")).unwrap();
    let config = root.join("config/functions.toml");
    let declaration = root.join("crates/recomp/src/timed_patterns.toml");
    fs::write(&config, "preserved config").unwrap();
    fs::write(&declaration, "preserved declaration").unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["iteration-pattern", "promote", "--root"])
        .arg(&root)
        .arg("--selection")
        .arg(root.join("selection"))
        .arg("--proof")
        .arg(root.join("proof"))
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert_eq!(fs::read_to_string(config).unwrap(), "preserved config");
    assert_eq!(fs::read_to_string(declaration).unwrap(), "preserved declaration");
    let owner = fs::read_dir(root.join("out/runs")).unwrap().next().unwrap().unwrap().path();
    let manifest: Value = toml::from_str(&fs::read_to_string(owner.join("manifest.toml")).unwrap()).unwrap();
    assert!(manifest["commands"].as_array().unwrap().is_empty());
    assert!(!root.join("out/pattern-application.toml").exists());
    fs::remove_dir_all(root).unwrap();
}
