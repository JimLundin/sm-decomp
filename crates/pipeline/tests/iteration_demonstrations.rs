//! Public native policy controls retain count, stop and failed publication evidence.
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
#[test]
fn retained_policy_rejects_overwrite_and_preserves_failed_increment() {
    let output = std::env::temp_dir()
        .join(format!("sm-native-policy-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    let call = || {
        Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["iteration-demonstrations", "policy", "--out"])
            .arg(&output)
            .output()
            .unwrap()
    };
    let result = call();
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report: toml::Value = toml::from_str(&fs::read_to_string(output.join("report.toml")).unwrap()).unwrap();
    assert_eq!(report["production_accepted_count"].as_integer(), Some(0));
    assert_eq!(report["cases"]["count2"]["accepted_count"].as_integer(), Some(2));
    assert_eq!(report["cases"]["failed-publication"]["accepted_count"].as_integer(), Some(0));
    assert_eq!(report["cases"]["user-stop"]["status"].as_str(), Some("stopped"));
    let before = fs::read(output.join("report.toml")).unwrap();
    assert!(!call().status.success());
    assert_eq!(fs::read(output.join("report.toml")).unwrap(), before);
    fs::remove_dir_all(output).unwrap();
}
