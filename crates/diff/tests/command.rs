use std::{
    fs,
    path::PathBuf,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("sm-parity-test-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        let mut rom = vec![0; 0x10000];
        rom[..2].copy_from_slice(&[0x80, 0xfe]);
        rom[0x7fc0..0x7fd5].copy_from_slice(b"SM RECOMP TEST       ");
        rom[0x7fd5] = 0x20;
        rom[0x7fd9] = 1;
        rom[0x7fdc..0x7fe0].copy_from_slice(&[0xcb, 0xed, 0x34, 0x12]);
        rom[0x7ffc..0x7ffe].copy_from_slice(&0x8000u16.to_le_bytes());
        fs::write(path.join("test.sfc"), rom).unwrap();
        fs::write(path.join("movie.inputs"), "0000\n1000\n").unwrap();
        Self(path)
    }
    fn run(&self, frames: &str, mode: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_diff"))
            .arg(self.0.join("movie.inputs"))
            .args([mode, "--frames", frames, "--rom"])
            .arg(self.0.join("test.sfc"))
            .arg("--out")
            .arg(self.0.join("evidence"))
            .output()
            .unwrap()
    }
    fn report(&self) -> toml::Value {
        toml::from_str(&fs::read_to_string(self.0.join("evidence/parity.toml")).unwrap()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn requested_scope_cannot_be_truncated_into_a_pass() {
    let fixture = Fixture::new();
    let result = fixture.run("3", "--fallback-only");
    assert_eq!(result.status.code(), Some(2));
    let report = fixture.report();
    assert_eq!(report["status"].as_str().unwrap(), "incomplete");
    assert_eq!(report["expected_frames"].as_integer().unwrap(), 3);
    assert_eq!(report["completed_frames"].as_integer().unwrap(), 0);
    assert_eq!(report["available_frames"].as_integer().unwrap(), 2);
}

#[test]
fn cold_fallback_reports_complete_frame_comparison() {
    let fixture = Fixture::new();
    let result = fixture.run("2", "--fallback-only");
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let report = fixture.report();
    assert_eq!(report["status"].as_str().unwrap(), "pass");
    assert!(report["complete"].as_bool().unwrap());
    assert_eq!(report["expected_frames"].as_integer().unwrap(), 2);
    assert_eq!(report["completed_frames"].as_integer().unwrap(), 2);
    assert_eq!(report["matched_frames"].as_integer().unwrap(), 2);
    assert_eq!(
        report["regions"].as_array().unwrap().iter().map(|x| x.as_str().unwrap()).collect::<Vec<_>>(),
        vec!["CPU+clocks", "WRAM", "framebuffer"]
    );
    assert_eq!(report["generated_steps"].as_integer().unwrap(), 0);
    assert!(report["interpreted_steps"].as_integer().unwrap() > 0);
    assert_eq!(report["identities"]["rom_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(report["identities"]["binary_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn prior_terminal_evidence_cannot_be_overwritten() {
    let fixture = Fixture::new();
    assert!(fixture.run("2", "--fallback-only").status.success());
    let original = fs::read(fixture.0.join("evidence/parity.toml")).unwrap();
    assert_eq!(fixture.run("3", "--fallback-only").status.code(), Some(2));
    assert_eq!(fs::read(fixture.0.join("evidence/parity.toml")).unwrap(), original);
}

#[test]
fn missing_rom_preserves_setup_error_evidence() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.0.join("test.sfc")).unwrap();
    assert_eq!(fixture.run("2", "--fallback-only").status.code(), Some(2));
    let report = fixture.report();
    assert_eq!(report["status"].as_str().unwrap(), "error");
    assert_eq!(report["stage"].as_str().unwrap(), "inputs");
    assert!(!report["complete"].as_bool().unwrap());
}

#[test]
fn empty_generated_table_cannot_validate_fallback_execution() {
    if !game::steps::STEP_TABLE.is_empty() {
        return;
    }
    let fixture = Fixture::new();
    assert_eq!(fixture.run("2", "--generated").status.code(), Some(1));
    let report = fixture.report();
    assert_eq!(report["status"].as_str().unwrap(), "uncovered");
    assert!(!report["complete"].as_bool().unwrap());
    assert_eq!(report["generated_steps"].as_integer().unwrap(), 0);
}

#[test]
fn frame_report_preserves_instruction_profile_separate_from_discovery() {
    let fixture = Fixture::new();
    assert!(fixture.run("2", "--fallback-only").status.success());
    let coverage: pipeline_model::InstructionProfile =
        pipeline_model::read(&fixture.0.join("evidence/coverage.toml")).unwrap();
    assert_eq!(coverage.schema_version, 1);
    assert_eq!(coverage.generated_instructions, 0);
    assert!(coverage.interpreted_instructions > 0);
    assert!(coverage.interpreted.iter().any(|entry| entry.opcode == 0x80));
    assert!(fixture.0.join("evidence/misses.csv").exists());
}
