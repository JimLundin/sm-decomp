//! Missing native movie retains a nonaccepting terminal diagnostic without compiling C.
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn missing_movie_cannot_accept_or_start_prefix_build() {
    let output = std::env::temp_dir()
        .join(format!("sm-prefix-missing-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["decomp-prefix", "--movie"])
        .arg(output.join("missing.lsmv"))
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1), "{}", String::from_utf8_lossy(&result.stderr));
    let response: toml::Value = toml::from_str(&String::from_utf8(result.stdout).unwrap()).unwrap();
    let owner = std::path::Path::new(response["evidence"].as_str().unwrap());
    let verdict: toml::Value = toml::from_str(&fs::read_to_string(owner.join("verdict.toml")).unwrap()).unwrap();
    assert_eq!(verdict["status"].as_str(), Some("error"));
    assert_eq!(verdict["accepted"].as_bool(), Some(false));
    assert!(!owner.join("prefix-host").exists());
    fs::remove_dir_all(output).unwrap();
}
