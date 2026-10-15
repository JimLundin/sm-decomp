//! Test adapter to application-owned finished-run cleanup.
use anyhow::{Context, Result, ensure};
use std::{path::PathBuf, process::Command};

pub(crate) struct CleanupRequest {
    pub root: PathBuf,
    pub apply: bool,
    pub deduplicate: bool,
}

pub(crate) fn cleanup(request: CleanupRequest) -> Result<toml::Value> {
    let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let target = std::env::var_os("CARGO_TARGET_DIR").map(PathBuf::from).unwrap_or_else(|| workspace.join("target"));
    let executable = std::env::var_os("SM_PIPELINE_CLEANUP_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| target.join("debug/sm-pipeline"));
    let mut command = Command::new(&executable);
    command.args(["cleanup", "--root"]).arg(&request.root);
    if request.apply {
        command.arg("--apply");
    }
    if request.deduplicate {
        command.arg("--deduplicate");
    }
    let output = command.output().with_context(|| {
        format!("run test cleanup with {}; build sm-pipeline or set SM_PIPELINE_CLEANUP_BIN", executable.display())
    })?;
    ensure!(output.status.success(), "test run cleanup failed: {}", String::from_utf8_lossy(&output.stderr));
    Ok(toml::from_str(std::str::from_utf8(&output.stdout)?)?)
}
