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

#[test]
#[ignore = "explicit untraced retained-tool startup comparison"]
fn alternating_utility_startup_comparison() {
    let baseline = fs::canonicalize(std::env::var_os("SM_STARTUP_BASELINE").expect("baseline tool")).unwrap();
    let candidate = fs::canonicalize(std::env::var_os("SM_STARTUP_CANDIDATE").expect("candidate tool")).unwrap();
    let root = PathBuf::from(std::env::var_os("SM_STARTUP_PROOF").expect("fresh proof root"));
    fs::create_dir(&root).unwrap();
    fs::create_dir(root.join("empty")).unwrap();
    let mut report = String::new();
    for round in 0..3 {
        let order = if round % 2 == 0 { [0, 1] } else { [1, 0] };
        for index in order {
            let (name, tool) = [("baseline", &baseline), ("candidate", &candidate)][index];
            let host_load = fs::read_to_string("/proc/loadavg").unwrap_or_default();
            let started = Instant::now();
            let output = Command::new("timeout")
                .args(["--signal=TERM", "--kill-after=2s", "30s"])
                .arg(tool)
                .args(["cleanup", "--root"])
                .arg(root.join("empty"))
                .stdin(Stdio::null())
                .output()
                .unwrap();
            let seconds = started.elapsed().as_secs_f64();
            fs::write(root.join(format!("{round}-{name}.stdout.toml")), &output.stdout).unwrap();
            fs::write(root.join(format!("{round}-{name}.stderr.txt")), &output.stderr).unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
            report.push_str(&format!(
                "[[sample]]\nround = {round}\nvariant = {name:?}\nbinary = {:?}\nseconds = {seconds:.9}\nhost_load_before = {host_load:?}\n",
                tool.to_str().unwrap()
            ));
            fs::write(root.join("measurements.toml"), &report).unwrap();
            println!("round={round} {name}: {seconds:.6}s");
        }
    }
}
