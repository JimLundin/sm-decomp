//! Retire only this merged worker checkout, retaining every source dependency.
use anyhow::{Context, Result, ensure};
use std::{collections::BTreeSet, fs, path::{Path, PathBuf}, process::Command};

fn bytes(path: &Path) -> Result<u64> {
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() { return Ok(0) }
    if meta.is_file() { return Ok(meta.len()) }
    fs::read_dir(path)?.try_fold(0, |sum, entry| Ok(sum + bytes(&entry?.path())?))
}
fn main() -> Result<()> {
    let root = Path::new("/home/jim/sm-decomp");
    let worktree = Path::new("/tmp/sm-decomp-parallel-pause-equipment-render-offline-1");
    let package = root.join(".scratch/decomp-coverage/renderer");
    let retained = package.join("retained-source");
    ensure!(!retained.exists(), "source retention already exists");
    let status = Command::new("git").args(["status", "--porcelain"]).current_dir(worktree).output()?;
    ensure!(status.status.success() && status.stdout.is_empty(), "worker source is not clean");
    let merged = Command::new("git").args(["merge-base", "--is-ancestor", "fa6e9cd747acabcb4568ccd2c0fbf532a9525c18", "HEAD"]).current_dir(root).status()?;
    ensure!(merged.success(), "submitted source is not on main");
    let verdict: toml::Value = toml::from_str(&fs::read_to_string(package.join("family-authenticate-main.stdout.toml"))?)?;
    ensure!(verdict["status"].as_str() == Some("offline-verified"), "passing retained verdict required");
    let mut paths: BTreeSet<PathBuf> = ["crates/port/src/pause_equipment_render.rs", "crates/port/src/memory.rs", "crates/port/tests/pause_equipment_render.rs", "tools/port/pause-equipment-render.toml", "tools/dependencies.toml"].into_iter().map(PathBuf::from).collect();
    for table in ["rust_dependencies", "harness_sources"] {
        for name in verdict["implementation"][table].as_table().context("source dependency seals absent")?.keys() {
            let path = Path::new(name);
            paths.insert(if path.is_absolute() { path.strip_prefix(worktree)?.to_owned() } else { path.to_owned() });
        }
    }
    for path in &paths {
        ensure!(path.components().all(|part| matches!(part, std::path::Component::Normal(_))), "unsafe retained source");
        let original = worktree.join(path);
        ensure!(original.is_file() && !original.is_symlink(), "source must be regular");
        let dest = retained.join(path);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::copy(&original, &dest)?;
        ensure!(fs::read(original)? == fs::read(dest)?, "source retention differs");
    }
    let target = worktree.join("target");
    let build_bytes = if target.exists() { bytes(&target)? } else { 0 };
    let removed = Command::new("git").args(["worktree", "remove"]).arg(worktree).current_dir(root).output()?;
    ensure!(removed.status.success(), "worker removal failed: {}", String::from_utf8_lossy(&removed.stderr));
    fs::create_dir_all(worktree)?;
    for path in &paths {
        let dest = worktree.join(path);
        fs::create_dir_all(dest.parent().unwrap())?;
        fs::copy(retained.join(path), &dest)?;
    }
    std::os::unix::fs::symlink(root.join("deps"), worktree.join("deps"))?;
    ensure!(!worktree.join(".git").exists() && !target.exists(), "worker metadata/build survived");
    let mut deleted = vec![(target, build_bytes)];
    for path in [
        PathBuf::from("/tmp/sm-decomp-offline-coverage-build-20261013"),
        PathBuf::from("/tmp/sm-decomp-coordinator-audit-build-20261013"),
        package.join("browser-report/chromium-profile"),
    ] {
        if path.exists() {
            ensure!(!path.is_symlink(), "disposable build must be a real directory");
            let size = bytes(&path)?;
            fs::remove_dir_all(&path)?;
            deleted.push((path, size));
        }
    }
    let disk = Command::new("df").args(["-B1", "--output=avail", "/home/jim/sm-decomp"]).output()?;
    ensure!(disk.status.success(), "disk observation failed");
    let free: u64 = String::from_utf8(disk.stdout)?.lines().last().unwrap().trim().parse()?;
    let total: u64 = deleted.iter().map(|(_, size)| size).sum();
    let mut value = toml::Table::new();
    value.insert("schema".into(), "decomp-coverage-retirement-v1".into());
    value.insert("status".into(), "source-retained-worker-retired".into());
    value.insert("source_files".into(), (paths.len() as i64).into());
    value.insert("build_bytes_removed".into(), (total as i64).into());
    value.insert("free_bytes".into(), (free as i64).into());
    value.insert("worktree".into(), worktree.display().to_string().into());
    value.insert("retained".into(), retained.display().to_string().into());
    value.insert("removed".into(), toml::Value::Array(deleted.into_iter().map(|(path, size)| toml::Value::Table(toml::Table::from_iter([("path".into(), path.display().to_string().into()), ("bytes".into(), (size as i64).into())]))).collect()));
    fs::write(package.join("retirement.toml"), toml::to_string_pretty(&value)?)?;
    println!("retained_sources={} build_bytes_removed={total} free_bytes={free}", paths.len());
    Ok(())
}
