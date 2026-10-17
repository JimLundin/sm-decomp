use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "sm-restart-test-{}-{}",
            std::process::id(),
            SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn doctor(&self) -> Output {
        run([
            OsStr::new("doctor"),
            OsStr::new("--rom"),
            self.0.join("input ROM.sfc").as_os_str(),
            OsStr::new("--reference-archive"),
            self.0.join("reference.tar.gz").as_os_str(),
        ])
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn run(args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sm-restart")).args(args).output().unwrap()
}

fn stdout(output: &Output) -> &str {
    std::str::from_utf8(&output.stdout).unwrap()
}

fn stderr(output: &Output) -> &str {
    std::str::from_utf8(&output.stderr).unwrap()
}

#[test]
fn help_describes_the_limited_claim() {
    for args in [vec!["--help"], vec!["doctor", "--help"]] {
        let output = run(args);
        assert!(output.status.success());
        assert!(stdout(&output).contains("Gameplay is not tested"));
    }
}

#[test]
fn usage_errors_are_distinct_from_input_failures() {
    for args in [
        vec![],
        vec!["play"],
        vec!["run", "--library", "a", "--rom", "b", "--out", "c", "--steps", "0"],
        vec!["run", "--library", "a", "--rom", "b", "--out", "c", "--scenario", "invalid"],
        vec!["prepare", "--reference-archive", "--out", "c"],
        vec!["doctor"],
        vec!["doctor", "--rom"],
        vec!["doctor", "--rom", "--reference-archive", "archive"],
        vec!["doctor", "--rom", "a", "--rom", "b"],
        vec!["doctor", "--reference-archive", "a", "--reference-archive", "b"],
        vec!["doctor", "--unknown", "a"],
        vec!["doctor", "--rom", "a", "--reference-archive", "b", "extra"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(stderr(&output).contains("Usage:"));
        assert!(!stdout(&output).contains("Input checks: passed"));
    }
}

#[test]
fn missing_inputs_fail_and_report_both_inputs() {
    let directory = TestDirectory::new();
    let output = directory.doctor();
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("ROM: failed"));
    assert!(stdout(&output).contains("Recovered C archive: failed"));
    assert_eq!(stderr(&output).matches("cannot open input").count(), 2);
    assert!(stdout(&output).contains("Extracted C source: not validated"));
    assert!(stdout(&output).contains("Gameplay: not tested"));
}

#[test]
fn correctly_sized_wrong_rom_and_wrong_archive_fail_hash_checks() {
    let directory = TestDirectory::new();
    let rom = directory.0.join("input ROM.sfc");
    let archive = directory.0.join("reference.tar.gz");
    fs::File::create(&rom).unwrap().set_len(3_145_728).unwrap();
    let archive_bytes = b"private synthetic archive content";
    fs::write(&archive, archive_bytes).unwrap();
    let output = directory.doctor();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(stderr(&output).matches("SHA-256 mismatch").count(), 2);
    assert!(!stderr(&output).contains(std::str::from_utf8(archive_bytes).unwrap()));
    assert!(stdout(&output).contains("Input checks: failed"));
    assert_eq!(fs::metadata(&rom).unwrap().len(), 3_145_728);
    assert_eq!(fs::read(&archive).unwrap(), archive_bytes);
}

#[test]
fn wrong_rom_size_and_directory_input_are_rejected() {
    let directory = TestDirectory::new();
    fs::write(directory.0.join("input ROM.sfc"), b"abc").unwrap();
    fs::create_dir(directory.0.join("reference.tar.gz")).unwrap();
    let output = directory.doctor();
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("size mismatch: expected 3145728 bytes, observed 3"));
    assert!(stderr(&output).contains("input must be a regular file"));
}
