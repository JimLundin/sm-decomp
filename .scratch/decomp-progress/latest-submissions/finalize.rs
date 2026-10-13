//! Retain this reporting run and retire its identified unsealed intermediates.
//! Evidence identities and storage belong exclusively to the evidence crate.
use std::{fs, io, path::{Path, PathBuf}, process::Command};

const ROOT: &str = "/home/jim/sm-decomp/.scratch/decomp-progress/latest-submissions";

fn bytes(path: &Path) -> io::Result<u64> {
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() { return Ok(0); }
    if meta.is_file() { return Ok(meta.len()); }
    fs::read_dir(path)?.try_fold(0, |sum, entry| Ok(sum + bytes(&entry?.path())?))
}

fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name() == "chromium-profile" { continue; }
        let kind = entry.file_type()?;
        if kind.is_symlink() { return Err(io::Error::other("unexpected symlink in reporting evidence")); }
        let dest = target.join(entry.file_name());
        if kind.is_dir() { copy_tree(&entry.path(), &dest)?; }
        else if kind.is_file() { fs::copy(entry.path(), dest)?; }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let root = Path::new(ROOT);
    let evidence = root.join("artifacts");
    assert!(!evidence.exists(), "preserve any existing sealed package");
    fs::create_dir(&evidence)?;
    let mut profiles: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_text = name.to_string_lossy();
        if entry.file_type()?.is_dir() && name_text.starts_with("browser-") {
            copy_tree(&entry.path(), &evidence.join(&name))?;
            profiles.push(entry.path().join("chromium-profile"));
        } else if entry.file_type()?.is_file()
            && [Some("txt"), Some("toml"), Some("lock"), Some("rs")].contains(&entry.path().extension().and_then(|v| v.to_str())) {
            fs::copy(entry.path(), evidence.join(&name))?;
        }
    }
    copy_tree(&root.join("source"), &evidence.join("source"))?;
    fs::copy("/home/jim/sm-decomp/.scratch/decomp-progress/registry.json", evidence.join("registry.json"))?;
    fs::copy("/home/jim/sm-decomp/.scratch/decomp-progress/index.html", evidence.join("index.html"))?;
    fs::copy("/home/jim/sm-decomp/target/debug/sm-evidence", root.join("bin/sm-evidence"))?;

    let mut paths = vec![root.join("pipeline-target"), root.join("reviewer-target")];
    paths.extend(profiles);
    let mut total = 0;
    let mut receipt = String::from("# Reporting run cleanup\n\nBoth Rust pipeline cleanup passes succeeded with zero eligible bytes and zero snapshots.\nThis report/browser run has no gameplay verdict; only these identified unsealed outputs are removed.\n\n| Path | Logical file bytes removed |\n| --- | ---: |\n");
    for path in paths {
        if !path.exists() { continue; }
        assert!(!path.is_symlink(), "only real disposable directories may be removed");
        let size = bytes(&path)?;
        fs::remove_dir_all(&path)?;
        total += size;
        receipt.push_str(&format!("| `{}` | {size} |\n", path.display()));
    }
    receipt.push_str(&format!("\nTotal logical file bytes removed: {total}.\n\nRetained: exact source, browser captures/checks and failure diagnostics, Rust test/publication logs, registry/HTML snapshots, Rust reviewer source/lockfile and the exact sm-evidence reader. The intended progress server PID803313 and its sm-pipeline binary remain available. Shared out/evidence packs are preserved.\n"));
    let disk = Command::new("df").args(["-B1", "--output=avail", "/home/jim/sm-decomp"]).output()?;
    assert!(disk.status.success());
    receipt.push_str(&format!("\nRemaining disk space at cleanup:\n\n```text\n{}```\n", String::from_utf8_lossy(&disk.stdout)));
    fs::write(root.join("disposable-cleanup.md"), &receipt)?;
    fs::write(evidence.join("disposable-cleanup.md"), receipt)?;
    println!("Removed {total} logical reporting file bytes; retained frozen artifacts and reader");
    Ok(())
}
