//! Scoped reporting receipt retention and identified disposable-output cleanup.
//! No gameplay run directory, evidence store or original source audit is mutated.
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
fn bytes(path: &Path) -> u64 {
    let metadata = fs::symlink_metadata(path).unwrap();
    let mut total = metadata.len();
    if metadata.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            total += bytes(&entry.unwrap().path());
        }
    }
    total
}
fn logs(directory: &Path, paths: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            if entry.file_name() != "chromium-profile" {
                logs(&path, paths);
            }
        } else if kind.is_file()
            && (path.extension().is_some_and(|v| v == "stdout") || entry.file_name() == "chromium.log")
        {
            paths.push(path);
        }
    }
}
fn main() {
    let mode = env::args().nth(1).unwrap();
    let root = Path::new("/home/jim/sm-decomp");
    let report = root.join(".scratch/decomp-progress/burndown");
    if mode == "receipts" {
        let archive = report.join("raw-review-receipts-20261013.tar.gz");
        assert!(!archive.exists(), "preserve existing archive");
        let mut paths = Vec::new();
        for directory in ["compact-velocity", "state-audit", "burndown"] {
            logs(&root.join(".scratch/decomp-progress").join(directory), &mut paths);
        }
        paths.sort();
        let mut names = Vec::new();
        let mut hashes = String::new();
        for path in &paths {
            let relative = path.strip_prefix(root).unwrap();
            names.extend(relative.to_string_lossy().as_bytes());
            names.push(0);
            let output = Command::new("sha256sum").arg(relative).current_dir(root).output().unwrap();
            assert!(output.status.success());
            hashes.push_str(std::str::from_utf8(&output.stdout).unwrap());
        }
        let list = report.join("raw-review-receipts.files");
        fs::write(&list, names).unwrap();
        fs::write(report.join("raw-review-receipts.sha256"), hashes).unwrap();
        let result = Command::new("tar")
            .args(["--create", "--gzip", "--null", "--file"])
            .arg(&archive)
            .arg("--files-from")
            .arg(&list)
            .current_dir(root)
            .status()
            .unwrap();
        assert!(result.success());
        let digest =
            Command::new("sha256sum").arg(archive.strip_prefix(root).unwrap()).current_dir(root).output().unwrap();
        assert!(digest.status.success());
        fs::write(report.join("raw-review-archive.sha256"), digest.stdout).unwrap();
        println!("Archived {} exact raw receipt files", paths.len());
    } else if mode == "cleanup" {
        let paths = [
            Path::new("/tmp/sm-progress-burndown-build"),
            Path::new("/tmp/sm-burndown-prototype"),
            Path::new("/home/jim/sm-decomp/.scratch/decomp-progress/burndown/prototype"),
        ];
        let mut receipt = String::from(
            "# Identified disposable reporting output cleanup\n\n| Path | Apparent bytes reclaimed |\n| --- | ---: |\n",
        );
        let mut total = 0;
        for path in paths {
            if path.exists() {
                let metadata = fs::symlink_metadata(path).unwrap();
                assert!(!metadata.file_type().is_symlink());
                let size = bytes(path);
                total += size;
                if metadata.is_dir() {
                    fs::remove_dir_all(path).unwrap();
                } else {
                    fs::remove_file(path).unwrap();
                }
                receipt.push_str(&format!("| `{}` | {} |\n", path.display(), size));
            }
        }
        receipt.push_str(&format!("\nTotal: {total} apparent bytes. Prototype source/captures are retained on the throwaway branch; raw receipts are retained byte-for-byte in the hashed archive. Main source, pinned inventory, sealed evidence and the intended report server are preserved.\n"));
        let disk = Command::new("df").args(["-B1", "--output=avail", "/home/jim/sm-decomp"]).output().unwrap();
        assert!(disk.status.success());
        receipt
            .push_str(&format!("\nRemaining disk space:\n\n```text\n{}```\n", String::from_utf8(disk.stdout).unwrap()));
        fs::write(report.join("cleanup/disposable-outputs.md"), receipt).unwrap();
        println!("Removed {total} identified disposable bytes");
    } else {
        panic!("usage: receipts|cleanup");
    }
}
