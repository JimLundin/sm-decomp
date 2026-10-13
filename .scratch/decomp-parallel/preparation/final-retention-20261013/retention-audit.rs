//! Read-only frozen-source and process inspection; output stays in caller-owned evidence.
use std::{collections::BTreeMap, fs, io::Write, path::Path, process::{Command, Stdio}};

fn sha(bytes: &[u8]) -> String {
    let mut child = Command::new("sha256sum").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    let result = child.wait_with_output().unwrap();
    assert!(result.status.success());
    String::from_utf8(result.stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}

fn collect(root: &Path, path: &Path, files: &mut BTreeMap<String, String>) {
    assert!(!path.is_symlink(), "source symlink: {}", path.display());
    if path.is_dir() {
        for entry in fs::read_dir(path).unwrap() { collect(root, &entry.unwrap().path(), files); }
    } else if path.is_file() {
        files.insert(path.strip_prefix(root).unwrap().display().to_string(), sha(&fs::read(path).unwrap()));
    }
}

fn source(root: &Path, expected: &str, output: &Path) {
    let mut files = BTreeMap::new();
    for relative in ["crates/pipeline/src", "crates/pipeline-model/src", "crates/port/src", "crates/port/tests", "tools", ".cargo", "Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "crates/port/Cargo.toml"] {
        let path = root.join(relative);
        if path.exists() { collect(root, &path, &mut files); }
    }
    let serialized: String = files.iter().map(|(path, hash)| format!("{path:?} = {hash:?}\n")).collect();
    let fingerprint = sha(serialized.as_bytes());
    assert_eq!(fingerprint, expected, "frozen source fingerprint at {}", root.display());
    let manifest: String = files.iter().map(|(path, hash)| format!("{hash}  {}\n", root.join(path).display())).collect();
    fs::write(output, manifest).unwrap();
    let head = Command::new("git").arg("-C").arg(root).args(["rev-parse", "HEAD"]).output().unwrap();
    assert!(head.status.success());
    println!("checkout = {:?}\nhead = {:?}\nfingerprint = {:?}\nsource_files = {}\nauthenticated = true", root.display().to_string(), String::from_utf8(head.stdout).unwrap().trim(), fingerprint, files.len());
}

fn processes(root: &Path) {
    let own = std::process::id();
    let mut references = 0;
    let mut inaccessible = 0;
    for entry in fs::read_dir("/proc").unwrap() {
        let path = entry.unwrap().path();
        let Some(pid) = path.file_name().unwrap().to_str().and_then(|name| name.parse::<u32>().ok()) else { continue; };
        if pid == own { continue; }
        let Ok(stat) = fs::read_to_string(path.join("stat")) else { continue; };
        let Some(rest) = stat.rsplit_once(") ").map(|(_, rest)| rest) else { continue; };
        let fields: Vec<_> = rest.split_whitespace().collect();
        let start_ticks = fields.get(19).unwrap_or(&"unknown");
        let mut evidence = Vec::new();
        if let Ok(cwd) = fs::read_link(path.join("cwd")) { if cwd.starts_with(root) { evidence.push(format!("cwd:{}", cwd.display())); } }
        match fs::read_dir(path.join("fd")) {
            Ok(fds) => for fd in fds.flatten() { if let Ok(target) = fs::read_link(fd.path()) { if target.starts_with(root) { evidence.push(format!("fd:{}", target.display())); } } },
            Err(_) => inaccessible += 1,
        }
        if !evidence.is_empty() {
            references += 1;
            println!("[[references]]\npid = {pid}\nstart_ticks = {start_ticks:?}\npaths = {evidence:?}");
        }
    }
    println!("[summary]\ncheckout = {:?}\nreferencing_processes = {references}\ninaccessible_fd_directories = {inaccessible}\ninspection = \"cwd and open fd references; inaccessible unrelated processes remain a stated observation limit\"", root.display().to_string());
}

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args[0].as_str() {
        "source" => source(Path::new(&args[1]), &args[2], Path::new(&args[3])),
        "processes" => processes(Path::new(&args[1])),
        _ => panic!("usage: retention-audit source CHECKOUT FINGERPRINT MANIFEST | processes CHECKOUT"),
    }
}
