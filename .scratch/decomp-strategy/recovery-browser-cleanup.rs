use std::{fs, io, path::{Path, PathBuf}, process::{Command, Stdio}};
use std::os::unix::fs::MetadataExt;

const ROOT: &str = "/home/jim/sm-decomp";
const PROBES: [&str; 6] = [
    "sm-progress-context-file-check",
    "sm-progress-context-file-check-v2",
    "sm-progress-context-inspection",
    "sm-progress-context-inspection-v2",
    "sm-progress-context-inspection-v3",
    "sm-progress-context-served-inspection",
];

fn command(program: &str, arguments: &[&str]) -> io::Result<Vec<u8>> {
    let output = Command::new(program).args(arguments).current_dir(ROOT).output()?;
    if !output.status.success() {
        return Err(io::Error::other(format!("{program} failed: {}", String::from_utf8_lossy(&output.stderr))));
    }
    Ok(output.stdout)
}

fn sizes(path: &Path) -> io::Result<(u64, u64, usize)> {
    let metadata = fs::symlink_metadata(path)?;
    let mut result = (metadata.len(), metadata.blocks() * 512, 1);
    if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            let child = sizes(&entry?.path())?;
            result.0 += child.0;
            result.1 += child.1;
            result.2 += child.2;
        }
    }
    Ok(result)
}

fn hash(path: &Path) -> io::Result<String> {
    let output = command("sha256sum", &[path.to_str().unwrap()])?;
    Ok(String::from_utf8_lossy(&output).split_whitespace().next().unwrap().to_owned())
}

fn processes_using(paths: &[PathBuf]) -> io::Result<Vec<String>> {
    let mut found = Vec::new();
    for entry in fs::read_dir("/proc")? {
        let process = entry?.path();
        let Some(pid) = process.file_name().and_then(|name| name.to_str()).and_then(|name| name.parse::<u32>().ok()) else { continue };
        if pid == std::process::id() { continue; }
        if let Ok(bytes) = fs::read(process.join("cmdline")) {
            let text = String::from_utf8_lossy(&bytes).replace('\0', " ");
            if paths.iter().any(|path| text.contains(path.to_str().unwrap())) {
                found.push(format!("pid {pid} command: {text}"));
            }
        }
        let mut links = vec![process.join("cwd"), process.join("root")];
        if let Ok(entries) = fs::read_dir(process.join("fd")) {
            links.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
        }
        for link in links {
            if let Ok(destination) = fs::read_link(&link) {
                if paths.iter().any(|path| destination.starts_with(path)) {
                    found.push(format!("pid {pid} link {} -> {}", link.display(), destination.display()));
                }
            }
        }
    }
    Ok(found)
}

fn main() -> io::Result<()> {
    let evidence = Path::new(ROOT).join(".scratch/decomp-strategy/recovery-browser-evidence");
    fs::create_dir_all(evidence.join("cleanup"))?;
    let mut roots: Vec<PathBuf> = PROBES.iter().map(|name| Path::new("/tmp").join(name)).collect();
    roots.push(Path::new("/tmp/sm-progress-chrome-probe").to_owned());
    let mut disposable: Vec<PathBuf> = roots[..6].iter().map(|root| root.join("chromium-profile")).collect();
    disposable.push(roots[6].clone());
    let users = processes_using(&roots)?;
    fs::write(evidence.join("preflight-processes.txt"), if users.is_empty() { "No command, cwd, root or open file descriptor names any selected probe root.\n".to_owned() } else { users.join("\n") })?;
    if !users.is_empty() { return Err(io::Error::other("probe root has an active process")); }
    let mut hashes = String::new();
    let mut retained = String::new();
    for root in &roots[..6] {
        let target = evidence.join(root.file_name().unwrap());
        fs::create_dir_all(&target)?;
        let mut entries = fs::read_dir(root)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            if entry.file_type()?.is_dir() {
                if entry.file_name() != "chromium-profile" { return Err(io::Error::other("unrecognized probe subdirectory")); }
                continue;
            }
            if !entry.file_type()?.is_file() { return Err(io::Error::other("unexpected top-level link")); }
            let original = entry.path();
            let copied = target.join(entry.file_name());
            if copied.exists() { return Err(io::Error::other("refusing to overwrite retained evidence")); }
            let before = hash(&original)?;
            fs::copy(&original, &copied)?;
            let after = hash(&copied)?;
            if before != after { return Err(io::Error::other("retained copy hash mismatch")); }
            hashes.push_str(&format!("{after}  {}\n", copied.display()));
            retained.push_str(&format!("{}\t{}\t{}\t{}\n", original.display(), copied.display(), fs::metadata(&copied)?.len(), after));
        }
    }
    fs::write(evidence.join("artifact-hashes.sha256"), hashes)?;
    fs::write(evidence.join("retained-artifacts.tsv"), retained)?;
    let listing = command("ls", &["-la", roots[6].to_str().unwrap()])?;
    fs::write(evidence.join("chrome-probe-profile-top-level.txt"), listing)?;
    let original_evidence = Path::new(ROOT).join(".scratch/decomp-progress/recovery-browser");
    fs::write(evidence.join("historical-artifact-authentication.txt"), command("sha256sum", &["--check", original_evidence.join("artifact-hashes.sha256").to_str().unwrap()])?)?;
    let binary = Path::new(ROOT).join("target/debug/sm-pipeline");
    let mut receipts = String::new();
    for root in &roots {
        for deduplicate in [false, true] {
            let mut args = vec!["cleanup", "--root", root.to_str().unwrap()];
            if deduplicate { args.push("--deduplicate"); }
            args.push("--apply");
            let output = Command::new(&binary).args(&args).current_dir(ROOT).stdin(Stdio::null()).output()?;
            let kind = if deduplicate { "deduplicate" } else { "ordinary" };
            let receipt = evidence.join("cleanup").join(format!("{}-{kind}.toml", root.file_name().unwrap().to_str().unwrap()));
            fs::write(&receipt, &output.stdout)?;
            fs::write(receipt.with_extension("stderr"), &output.stderr)?;
            if !output.status.success() { return Err(io::Error::other("project cleanup failed")); }
            if !String::from_utf8_lossy(&output.stdout).lines().any(|line| line == "bytes = 0") { return Err(io::Error::other("unexpected recognized disposable output")); }
            receipts.push_str(&format!("{} {} -> {}\n", binary.display(), args.join(" "), receipt.display()));
            println!("Project cleanup {kind} succeeded for {}", root.display());
        }
    }
    fs::write(evidence.join("cleanup-commands.txt"), receipts)?;
    let users = processes_using(&roots)?;
    if !users.is_empty() { return Err(io::Error::other(format!("probe became active: {}", users.join("\n")))); }
    fs::write(evidence.join("before-disk.txt"), command("df", &["-B1", "--output=avail", ROOT])?)?;
    let mut totals = (0, 0, 0);
    let mut removals = String::from("path\tapparent_bytes\tallocated_bytes\tentries\n");
    for path in disposable {
        if path.is_symlink() { return Err(io::Error::other("refusing to remove profile symlink")); }
        let measured = sizes(&path)?;
        fs::remove_dir_all(&path)?;
        if path.exists() { return Err(io::Error::other("profile removal did not finish")); }
        totals.0 += measured.0;
        totals.1 += measured.1;
        totals.2 += measured.2;
        removals.push_str(&format!("{}\t{}\t{}\t{}\n", path.display(), measured.0, measured.1, measured.2));
    }
    removals.push_str(&format!("TOTAL\t{}\t{}\t{}\n", totals.0, totals.1, totals.2));
    fs::write(evidence.join("removed-profiles.tsv"), removals)?;
    fs::write(evidence.join("after-disk.txt"), command("df", &["-B1", "--output=avail", ROOT])?)?;
    fs::write(evidence.join("post-cleanup-artifact-authentication.txt"), command("sha256sum", &["--check", evidence.join("artifact-hashes.sha256").to_str().unwrap()])?)?;
    fs::write(evidence.join("post-cleanup-historical-artifact-authentication.txt"), command("sha256sum", &["--check", original_evidence.join("artifact-hashes.sha256").to_str().unwrap()])?)?;
    println!("Removed {} apparent bytes / {} allocated bytes across {} profile entries. Retained {}.", totals.0, totals.1, totals.2, evidence.display());
    Ok(())
}
