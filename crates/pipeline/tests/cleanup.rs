//! Cleanup preserves sealed paths and requires a completed run before deletion.
use serde::Serialize;
use sha2::{Digest, Sha256};
#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, symlink};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};
use toml::Value;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-cleanup-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, data: impl AsRef<[u8]>) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, data).unwrap();
    }
    fn run(&self, apply: bool, deduplicate: bool) -> Value {
        #[derive(Serialize)]
        struct Request<'a> {
            root: &'a PathBuf,
            apply: bool,
            deduplicate: bool,
        }
        let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .arg("cleanup")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(toml::to_string(&Request { root: &self.0, apply, deduplicate }).unwrap().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
#[test]
fn preview_and_apply_preserve_nested_seals_unfinished_builds_and_symlinks() {
    let root = Directory::new();
    for name in ["completed", "sealed", "unfinished"] {
        root.write(&format!("{name}/target/build.o"), b"build");
    }
    root.write("completed/verdict.toml", "status='mixed-C-Rust-integration'\n[artifacts]\n\"terminal.bin\"='digest'\n");
    root.write(
        "sealed/verdict.toml",
        "status='offline-verified'\n[[nested]]\n[nested.artifacts]\n\"other/../target/build.o\"='digest'\n",
    );
    root.write("completed/libfamilies.a", b"archive");
    root.write("completed/live", b"binary");
    #[cfg(unix)]
    symlink(root.0.join("completed"), root.0.join("linked")).unwrap();
    assert_eq!(root.run(false, false)["bytes"].as_integer(), Some(12));
    assert!(root.0.join("completed/target").exists());
    assert_eq!(root.run(true, false)["bytes"].as_integer(), Some(12));
    assert!(!root.0.join("completed/target").exists());
    assert!(!root.0.join("completed/libfamilies.a").exists());
    for name in ["completed/live", "sealed/target/build.o", "unfinished/target/build.o"] {
        assert!(root.0.join(name).exists());
    }
}
#[test]
fn unreadable_evidence_prevents_any_deletion() {
    let root = Directory::new();
    root.write("one/verdict.toml", "status='accepted'\n");
    root.write("one/target/build.o", b"build");
    root.write("two/acceptance.toml", b"invalid");
    let output = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["cleanup", "--root", root.0.to_str().unwrap(), "--apply"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(root.0.join("one/target/build.o").exists());
}
#[test]
fn native_terminal_runs_remove_only_unsealed_builds_and_preserve_binary_checkpoints() {
    let root = Directory::new();
    for (name, state) in [("pass", "complete"), ("failed", "complete"), ("sealed", "complete"), ("running", "running")]
    {
        root.write(&format!("{name}/build/generator/object.o"), b"build");
        let evidence = if name == "sealed" {
            "[identities.binary]\nevidence='build/generator/object.o'\n"
        } else {
            "[identities]\n"
        };
        root.write(
            &format!("{name}/manifest.toml"),
            format!("run_id='{name}'\nstate='{state}'\n[check_evidence]\n{evidence}"),
        );
        root.write(
            &format!("{name}/verdict.toml"),
            format!("run_id='{name}'\nstatus='{}'\n", if name == "failed" { "error" } else { "pass" }),
        );
        root.write(&format!("{name}/diagnostics/cache/pair.bin"), b"sealed binary state");
    }
    assert_eq!(root.run(false, false)["bytes"].as_integer(), Some(10));
    assert_eq!(root.run(true, false)["bytes"].as_integer(), Some(10));
    for name in ["sealed", "running"] {
        assert!(root.0.join(name).join("build/generator/object.o").exists());
    }
    for name in ["pass", "failed", "sealed", "running"] {
        assert!(root.0.join(name).join("diagnostics/cache/pair.bin").exists());
    }
}
#[test]
#[cfg(unix)]
fn deduplication_preserves_bytes_and_skips_invalid_seals_escapes_and_running_runs() {
    let root = Directory::new();
    let payload = vec![0u8; 0x20000];
    let hash = format!("{:x}", Sha256::digest(&payload));
    for name in ["one", "two", "running", "corrupt", "outside"] {
        root.write(&format!("{name}/before.wram"), &payload);
        root.write(
            &format!("{name}/verdict.toml"),
            format!(
                "status='{}'\n[artifacts]\n\"before.wram\"='{}'\n",
                if name == "running" { "running" } else { "offline-verified" },
                if name == "corrupt" { "invalid" } else { &hash }
            ),
        );
    }
    fs::remove_file(root.0.join("outside/verdict.toml")).unwrap();
    symlink(root.0.join("outside"), root.0.join("one/link")).unwrap();
    root.write("one/verdict.toml",format!("status='offline-verified'\n[artifacts]\n\"before.wram\"='{hash}'\n\"link/before.wram\"='{hash}'\n\"../outside/before.wram\"='{hash}'\n"));
    let seal = fs::read(root.0.join("one/verdict.toml")).unwrap();
    assert_eq!(root.run(false, true)["bytes"].as_integer(), Some(payload.len() as i64));
    assert_ne!(
        root.0.join("one/before.wram").metadata().unwrap().ino(),
        root.0.join("two/before.wram").metadata().unwrap().ino()
    );
    assert_eq!(root.run(true, true)["bytes"].as_integer(), Some(payload.len() as i64));
    assert_eq!(
        root.0.join("one/before.wram").metadata().unwrap().ino(),
        root.0.join("two/before.wram").metadata().unwrap().ino()
    );
    assert_eq!(fs::read(root.0.join("two/before.wram")).unwrap(), payload);
    assert_eq!(fs::read(root.0.join("one/verdict.toml")).unwrap(), seal);
    for name in ["running", "corrupt", "outside"] {
        assert_eq!(root.0.join(name).join("before.wram").metadata().unwrap().nlink(), 1);
    }
    assert_eq!(root.run(true, true)["bytes"].as_integer(), Some(0));
}
