//! Exercise the real CLI protocol, OS locks and durable cache with two workers.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};
use toml::Value;

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
#[derive(Serialize)]
struct CacheRequest<'a> {
    root: &'a Path,
    checkout: &'a Path,
    command: Vec<&'a str>,
    inputs: Vec<PathBuf>,
    destination: PathBuf,
}
#[derive(Deserialize)]
struct CacheOffer {
    hit: bool,
    artifact: Option<PathBuf>,
}
#[derive(Deserialize)]
struct CacheResult {
    key: String,
}
fn send<T: Serialize>(child: &mut Child, request: &T) {
    let bytes = toml::to_string(request).unwrap().into_bytes();
    let input = child.stdin.as_mut().unwrap();
    input.write_all(&(bytes.len() as u32).to_le_bytes()).unwrap();
    input.write_all(&bytes).unwrap();
    input.flush().unwrap();
}
fn start(root: &Path, checkout: &Path, option: &str) -> Child {
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("cache")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    send(
        &mut child,
        &CacheRequest {
            root,
            checkout,
            command: vec!["rustc", option],
            inputs: vec![checkout.join("source")],
            destination: checkout.join("output"),
        },
    );
    child
}
fn reply<T: serde::de::DeserializeOwned>(child: &mut Child) -> T {
    let mut header = [0; 4];
    child.stdout.as_mut().unwrap().read_exact(&mut header).unwrap();
    let mut bytes = vec![0; u32::from_le_bytes(header) as usize];
    child.stdout.as_mut().unwrap().read_exact(&mut bytes).unwrap();
    toml::from_str(std::str::from_utf8(&bytes).unwrap()).unwrap()
}
fn commit(child: &mut Child, first: &CacheOffer) -> CacheResult {
    if !first.hit {
        fs::write(first.artifact.as_ref().unwrap(), "compiled").unwrap();
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
    let initial: CacheOffer = reply(&mut first);
    assert!(!initial.hit);
    let mut second = start(&root.0, &b.0, "test");
    std::thread::sleep(Duration::from_millis(150));
    assert!(second.try_wait().unwrap().is_none());
    let result = commit(&mut first, &initial);
    let hit: CacheOffer = reply(&mut second);
    assert!(hit.hit);
    let result2 = commit(&mut second, &hit);
    assert_eq!(result.key, result2.key);
    fs::write(root.0.join("build-cache").join(&result.key).join("artifact"), "corrupt").unwrap();
    let mut third = start(&root.0, &a.0, "test");
    let miss: CacheOffer = reply(&mut third);
    assert!(!miss.hit);
    commit(&mut third, &miss);
    let mut changed = start(&root.0, &a.0, "changed-options");
    let miss: CacheOffer = reply(&mut changed);
    assert!(!miss.hit);
    commit(&mut changed, &miss);
}

#[test]
fn mutation_and_abandoned_publication_never_become_hits() {
    let root = Directory::new();
    fs::write(root.0.join("source"), "before").unwrap();
    let mut child = start(&root.0, &root.0, "mutation");
    let miss: CacheOffer = reply(&mut child);
    fs::write(miss.artifact.as_ref().unwrap(), "partial").unwrap();
    fs::write(root.0.join("source"), "after").unwrap();
    writeln!(child.stdin.as_mut().unwrap(), "commit").unwrap();
    assert!(!child.wait().unwrap().success());
    assert!(!root.0.join("output").exists());
    let mut abandoned = start(&root.0, &root.0, "abandoned");
    let miss: CacheOffer = reply(&mut abandoned);
    fs::write(miss.artifact.as_ref().unwrap(), "partial").unwrap();
    drop(abandoned.stdin.take());
    assert!(!abandoned.wait().unwrap().success());
    let mut retry = start(&root.0, &root.0, "abandoned");
    let miss: CacheOffer = reply(&mut retry);
    assert!(!miss.hit);
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
    let _: Value = reply(&mut first);
    let mut second = spawn();
    std::thread::sleep(Duration::from_millis(150));
    assert!(second.try_wait().unwrap().is_none());
    first.kill().unwrap();
    first.wait().unwrap();
    let _: Value = reply(&mut second);
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
    #[derive(Serialize)]
    struct CRequest {
        root: PathBuf,
        checkout: PathBuf,
        run: PathBuf,
        workers: usize,
        command: Vec<String>,
    }
    #[derive(Deserialize)]
    struct ObjectRecord {
        hit: bool,
    }
    #[derive(Deserialize)]
    struct CacheRecords {
        objects: Vec<ObjectRecord>,
    }
    let request = CRequest {
        root: root.0.clone(),
        checkout: root.0.clone(),
        run: root.0.clone(),
        workers: 1,
        command: vec![
            "cc".into(),
            "-O2".into(),
            "-DEXTRA=2".into(),
            "-iquote".into(),
            pinned.to_str().unwrap().into(),
            pinned.join("value.c").to_str().unwrap().into(),
            root.0.join("main.c").to_str().unwrap().into(),
            "-o".into(),
            root.0.join("program").to_str().unwrap().into(),
        ],
    };
    let run = || {
        let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
            .arg("c-build")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
        assert!(child.wait().unwrap().success());
        toml::from_str::<CacheRecords>(&fs::read_to_string(root.0.join("c-cache.toml")).unwrap()).unwrap()
    };
    let first = run();
    assert!(!first.objects[0].hit);
    assert_eq!(Command::new(root.0.join("program")).status().unwrap().code(), Some(5));
    assert!(run().objects[0].hit);
    fs::write(pinned.join("value.h"), "#define VALUE 8\n").unwrap();
    let changed = run();
    assert!(!changed.objects[0].hit);
    assert_eq!(Command::new(root.0.join("program")).status().unwrap().code(), Some(10));
}
