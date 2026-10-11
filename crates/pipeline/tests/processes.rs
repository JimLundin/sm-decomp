//! Exercise the real CLI protocol, OS locks and durable cache with two workers.
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Directory(PathBuf);
impl Directory {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "sm-pipeline-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn start(root: &Path, checkout: &Path, option: &str) -> Child {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("cache")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "{}", json!({"root":root,"checkout":checkout,"command":["rustc",option],"inputs":[checkout.join("source")],"destination":checkout.join("output")})).unwrap();
    child
}
fn reply(child: &mut Child) -> Value {
    let mut line = Vec::new();
    let mut byte = [0u8];
    while child.stdout.as_mut().unwrap().read_exact(&mut byte).is_ok() {
        if byte[0] == b'\n' {
            break;
        }
        line.push(byte[0]);
    }
    serde_json::from_slice(&line).unwrap()
}
fn commit(child: &mut Child, first: &Value) -> Value {
    if !first["hit"].as_bool().unwrap() {
        fs::write(first["artifact"].as_str().unwrap(), "compiled").unwrap();
        writeln!(child.stdin.as_mut().unwrap(), "commit").unwrap();
    }
    let completed = reply(child);
    assert!(child.wait().unwrap().success());
    completed
}

#[test]
fn equivalent_worktrees_share_one_build_and_corruption_is_rejected() {
    let root = Directory::new();
    let a = Directory::new();
    let b = Directory::new();
    for p in [&a.0, &b.0] {
        fs::write(p.join("source"), "same source").unwrap();
    }
    let mut first = start(&root.0, &a.0, "test");
    let initial = reply(&mut first);
    assert_eq!(initial["hit"], false);
    let mut second = start(&root.0, &b.0, "test");
    std::thread::sleep(Duration::from_millis(150));
    assert!(second.try_wait().unwrap().is_none());
    let result = commit(&mut first, &initial);
    let hit = reply(&mut second);
    assert_eq!(hit["hit"], true);
    let result2 = commit(&mut second, &hit);
    assert_eq!(result["key"], result2["key"]);
    fs::write(root.0.join("build-cache").join(result["key"].as_str().unwrap()).join("artifact"), "corrupt").unwrap();
    let mut third = start(&root.0, &a.0, "test");
    let miss = reply(&mut third);
    assert_eq!(miss["hit"], false);
    commit(&mut third, &miss);
    let mut changed = start(&root.0, &a.0, "changed-options");
    let miss = reply(&mut changed);
    assert_eq!(miss["hit"], false);
    commit(&mut changed, &miss);
}

#[test]
fn mutation_and_abandoned_publication_never_become_hits() {
    let root = Directory::new();
    fs::write(root.0.join("source"), "before").unwrap();
    let mut child = start(&root.0, &root.0, "mutation");
    let miss = reply(&mut child);
    fs::write(miss["artifact"].as_str().unwrap(), "partial").unwrap();
    fs::write(root.0.join("source"), "after").unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "commit").unwrap();
    assert!(!child.wait().unwrap().success());
    assert!(!root.0.join("output").exists());
    let mut abandoned = start(&root.0, &root.0, "abandoned");
    let miss = reply(&mut abandoned);
    fs::write(miss["artifact"].as_str().unwrap(), "partial").unwrap();
    drop(abandoned.stdin.take());
    assert!(!abandoned.wait().unwrap().success());
    let mut retry = start(&root.0, &root.0, "abandoned");
    let miss = reply(&mut retry);
    assert_eq!(miss["hit"], false);
    commit(&mut retry, &miss);
}

#[test]
fn killed_lease_holder_releases_capacity() {
    let root = Directory::new();
    let spawn = || {
        Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .args(["lease", root.0.to_str().unwrap(), "1"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap()
    };
    let mut first = spawn();
    reply(&mut first);
    let mut second = spawn();
    std::thread::sleep(Duration::from_millis(150));
    assert!(second.try_wait().unwrap().is_none());
    first.kill().unwrap();
    first.wait().unwrap();
    reply(&mut second);
    drop(second.stdin.take());
    assert!(second.wait().unwrap().success());
}

#[test]
fn real_c_objects_preserve_options_and_invalidate_on_header_edits() {
    let root = Directory::new();
    let pinned = root.0.join("deps");
    fs::create_dir(&pinned).unwrap();
    fs::write(pinned.join("value.h"), "#define VALUE 3\n").unwrap();
    fs::write(pinned.join("value.c"), "#include \"value.h\"\nint value(void){return VALUE + EXTRA;}\n").unwrap();
    fs::write(root.0.join("main.c"), "int value(void); int main(void){return value();}\n").unwrap();
    let request = json!({"root":root.0,"checkout":root.0,"run":root.0,"workers":1,
        "command":["cc","-O2","-DEXTRA=2","-iquote",pinned,pinned.join("value.c"),root.0.join("main.c"),"-o",root.0.join("program")]});
    let run = || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .arg("c-build")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        writeln!(child.stdin.as_mut().unwrap(), "{request}").unwrap();
        assert!(child.wait().unwrap().success());
        serde_json::from_slice::<Value>(&fs::read(root.0.join("c-cache.json")).unwrap()).unwrap()
    };
    let first = run();
    assert_eq!(first[0]["hit"], false);
    assert_eq!(Command::new(root.0.join("program")).status().unwrap().code(), Some(5));
    assert_eq!(run()[0]["hit"], true);
    fs::write(pinned.join("value.h"), "#define VALUE 8\n").unwrap();
    let changed = run();
    assert_eq!(changed[0]["hit"], false);
    assert_eq!(Command::new(root.0.join("program")).status().unwrap().code(), Some(10));
}
