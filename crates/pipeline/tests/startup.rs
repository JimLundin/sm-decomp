//! Opt-in actual executable-read controls. Requires Linux strace for the probe,
//! not for the pipeline or its ordinary tests. Can be compiled independently
//! with rustc --test to inspect retained binaries without rebuilding the tools.
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    time::Instant,
};

fn binary() -> PathBuf {
    std::env::var_os("SM_STARTUP_PROBE_BINARY")
        .map(PathBuf::from)
        .or_else(|| option_env!("CARGO_BIN_EXE_sm-pipeline").map(PathBuf::from))
        .expect("SM_STARTUP_PROBE_BINARY or Cargo binary required")
}

fn trace(name: &str, operation: &str) -> (u64, u64) {
    let binary = fs::canonicalize(binary()).unwrap();
    let proof = std::env::var_os("SM_STARTUP_PROOF").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let root = proof.join(format!("{name}-{}", std::process::id()));
    fs::create_dir_all(root.join("empty")).unwrap();
    let log = root.join("reads.strace");
    let mut command = Command::new("timeout");
    command
        .args(["--signal=TERM", "--kill-after=2s", "30s", "strace", "-f", "-qq", "-s", "0"])
        .args(["-e", "trace=read,pread64", "-P"])
        .arg(&binary)
        .arg("-o")
        .arg(&log)
        .arg(&binary)
        .arg(operation)
        .stdin(Stdio::null());
    if operation == "cleanup" {
        command.args(["--root"]).arg(root.join("empty"));
    } else {
        let request = root.join("invalid-request.txt");
        fs::write(&request, "this is not = valid [TOML").unwrap();
        command.stdin(Stdio::from(fs::File::open(request).unwrap()));
    }
    let started = Instant::now();
    let output = command.output().expect("Linux strace and timeout required for explicit startup probe");
    let elapsed = started.elapsed().as_nanos();
    fs::write(root.join("stdout"), &output.stdout).unwrap();
    fs::write(root.join("stderr"), &output.stderr).unwrap();
    if operation == "cleanup" {
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    } else {
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("invalid TOML request"));
    }
    assert!(fs::metadata(&log).unwrap().len() <= 8 * 1024 * 1024, "startup syscall trace exceeds probe bound");
    let reads = fs::read_to_string(&log).unwrap();
    let bytes: u64 = reads
        .lines()
        .filter(|line| line.contains("read(") || line.contains("pread64("))
        .filter_map(|line| line.rsplit_once(" = ").and_then(|(_, result)| result.split_whitespace().next()))
        .filter_map(|count| count.parse::<u64>().ok())
        .sum();
    let size = fs::metadata(&binary).unwrap().len();
    fs::write(
        root.join("observation.toml"),
        format!(
            "binary = {:?}\noperation = {:?}\nbinary_bytes = {size}\nread_bytes = {bytes}\nelapsed_ns = {elapsed}\n",
            binary.to_str().unwrap(),
            operation
        ),
    )
    .unwrap();
    println!("{operation}: read {bytes} of {size} executable bytes");
    (bytes, size)
}

#[test]
#[ignore = "explicit Linux executable-read probe"]
fn utility_startup_does_not_read_controller_image() {
    let (bytes, _) = trace("utility", "cleanup");
    assert_eq!(bytes, 0, "utility command hashed an identity its contract does not consume");
}

#[test]
#[ignore = "explicit Linux executable-read probe"]
fn offline_identity_reads_the_complete_controller_image() {
    let (bytes, size) = trace("offline", "family-verify");
    assert_eq!(bytes, size, "Offline preparation must consume the complete exact producer image once");
}
