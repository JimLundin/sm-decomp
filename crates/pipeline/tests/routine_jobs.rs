//! Invalid job requests and predecessors must fail before building any candidate.
use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn requested_jobs_and_missing_predecessor_cannot_start_builds() {
    let root = std::env::temp_dir().join(format!(
        "sm-routine-admission-{}-{}",
        std::process::id(),
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    for (i, flags) in [vec!["--job", "unknown"], vec!["--count", "2"], vec!["--count", "0"]].into_iter().enumerate() {
        let output = root.join(format!("job-{i}"));
        let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .current_dir(checkout)
            .arg("decomp-jobs")
            .args(flags)
            .arg("--output")
            .arg(&output)
            .output()
            .unwrap();
        assert!(!result.status.success());
        let report: Value = toml::from_str(&fs::read_to_string(output.join("progress.toml")).unwrap()).unwrap();
        assert_eq!(report["status"].as_str().unwrap(), "error");
        assert!(report["accepted_jobs"].as_array().unwrap().is_empty());
        assert_eq!(fs::read_dir(output).unwrap().count(), 1);
    }
    let output = root.join("live");
    let result = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .current_dir(checkout)
        .arg("routine-live")
        .arg("--accepted")
        .arg(root.join("missing.toml"))
        .arg("--output")
        .arg(&output)
        .output()
        .unwrap();
    assert!(!result.status.success());
    let run = fs::read_dir(output).unwrap().next().unwrap().unwrap().path();
    let report: Value = toml::from_str(&fs::read_to_string(run.join("verdict.toml")).unwrap()).unwrap();
    assert_eq!(report["status"].as_str().unwrap(), "error");
    assert!(report["stages"].as_array().unwrap().is_empty());
    assert!(!report["mixed_integration"].as_bool().unwrap());
    assert_eq!(fs::read_dir(run).unwrap().count(), 1);
    fs::remove_dir_all(root).unwrap();
}
