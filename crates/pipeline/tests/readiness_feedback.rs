#![doc = " Actual failure subprocesses cannot publish acceptance, including a one-worker budget."]
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn failure_controls_preserve_pointer_and_reject_stale_owners() {
    let root = std::env::temp_dir().join(format!(
        "sm-native-feedback-controls-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    for case in ["preflight-build-failure", "interrupted-truncated", "incompatible-stale"] {
        let output = root.join(case);
        let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["readiness-feedback-case", "--case", case, "--out"])
            .arg(&output)
            .env("SM_PIPELINE_ROOT", root.join("workers"))
            .env("SM_PIPELINE_WORKERS", "1")
            .output()
            .unwrap();
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        let report: Value = toml::from_str(&fs::read_to_string(output.join("report.toml")).unwrap()).unwrap();
        for observation in report["observations"].as_table().unwrap().values() {
            assert_eq!(observation["accept_returncode"], toml::Value::try_from(2).unwrap());
            assert_eq!(observation["pointer_unchanged"], toml::Value::try_from(true).unwrap());
        }
        let pointer = fs::read_to_string(output.join("repository/out/accepted.toml")).unwrap();
        assert!(pointer.contains("must stay unchanged"));
    }
    fs::remove_dir_all(root).unwrap();
}
