//! Actual allocation amounts and retained identities are independently checked.
#[path = "support/binary_fixture.rs"]
mod binary_fixture;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
use toml::Value;
struct Run(PathBuf);
impl Drop for Run {
    fn drop(&mut self) {
        if self.0.join("out").is_dir() {
            for deduplicate in [false, true] {
                let mut command = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"));
                command.args(["cleanup", "--root"]).arg(self.0.join("out")).arg("--apply");
                if deduplicate {
                    command.arg("--deduplicate");
                }
                let output = command.output().unwrap();
                assert!(output.status.success(), "cleanup: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn invoke(command: &str, request: &str, success: bool) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    writeln!(child.stdin.take().unwrap(), "{request}").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.success(), success, "{}", String::from_utf8_lossy(&output.stderr));
    if output.stdout.is_empty() {
        Value::Table(toml::Table::new())
    } else {
        toml::from_str(&String::from_utf8(output.stdout).unwrap()).unwrap()
    }
}
#[test]
fn actual_quicksand_allocation_rejects_wrong_amount_and_retained_tampering() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let rom = checkout.join("rom/sm.sfc");
    if !rom.exists() {
        return;
    }
    let root = Run(std::env::temp_dir()
        .join(format!("sm-plm-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
    fs::create_dir(&root.0).unwrap();
    let module = root.0.join("candidate.rs");
    let original = fs::read_to_string(checkout.join("crates/port/src/plm.rs")).unwrap();
    fs::write(&module, &original).unwrap();
    let manifest = root.0.join("manifest.smf");
    let rom_hash = format!("{:x}", Sha256::digest(fs::read(&rom).unwrap()));
    binary_fixture::write(
        &manifest,
        format!(
            r#"family = "native-test"
rust_module = {module:?}
rom = {{ file = {rom:?}, sha256 = {rom_hash:?} }}
[[cases]]
name = "quicksand"
a = 46891
b = 65536
writes = [{{ offset = 2818, data = [3, 0] }}]
expected_flags = 0
expected_amount = 12288
"#
        ),
    )
    .unwrap();
    let output = root.0.join("out");
    let workers = root.0.join("workers");
    let source = checkout.join("deps/snesrev-sm");
    let request = format!(
        "checkout = {checkout:?}\nsource = {source:?}\nmanifest = {manifest:?}\noutput = {output:?}\nadapter = \"plm\"\nroot = {workers:?}\nworkers = 2\ncase_jobs = 2\n"
    );
    let verdict = invoke("family-verify", &request, true);
    assert_eq!(verdict["allocator_cases"]["terrain"].as_integer(), Some(1));
    assert_eq!(verdict["rejected_abi_inputs"].as_integer(), Some(5));
    let path = Path::new(verdict["run"].as_str().unwrap()).join("verdict.toml");
    let authentication = format!("checkout = {checkout:?}\nverdict = {path:?}\n");
    invoke("family-authenticate", &authentication, true);
    let mut incomplete = verdict.clone();
    incomplete["artifacts"].as_table_mut().unwrap().remove("quicksand/rust.stdout");
    fs::write(&path, toml::to_string(&incomplete).unwrap()).unwrap();
    invoke("family-authenticate", &authentication, false);
    fs::write(&path, toml::to_string(&verdict).unwrap()).unwrap();
    fs::write(path.parent().unwrap().join("quicksand/rust-after.wram"), vec![0; 0x20000]).unwrap();
    invoke("family-authenticate", &authentication, false);
    let altered = original.replace("Ok(flags)\n}", "*amount=amount.wrapping_add(1);Ok(flags)\n}");
    assert_ne!(altered, original);
    fs::write(module, altered).unwrap();
    let rejected = invoke("family-verify", &request, false);
    assert!(rejected["error"].as_str().unwrap().contains("amount"));
}
