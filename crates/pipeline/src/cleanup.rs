//! Finished-run cleanup. Sealed paths and symlinks are never disposable.
use crate::digest;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};
use toml::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);

fn document(path: &Path) -> Result<Value> {
    toml::from_str(&fs::read_to_string(path)?).with_context(|| format!("read {}", path.display()))
}

fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => (),
            Component::ParentDir => {
                result.pop();
            }
            other => result.push(other.as_os_str()),
        }
    }
    result
}

fn inventories(value: &Value, result: &mut Vec<String>) {
    match value {
        Value::Table(fields) => {
            for (key, value) in fields {
                if ["artifacts", "inventory"].contains(&key.as_str()) {
                    if let Some(entries) = value.as_table() {
                        result.extend(entries.keys().cloned());
                    } else {
                        inventories(value, result);
                    }
                } else {
                    inventories(value, result);
                }
            }
        }
        Value::Array(values) => {
            for value in values {
                inventories(value, result);
            }
        }
        _ => (),
    }
}

fn entries(path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(path)?.map(|v| Ok(v?.path())).collect::<Result<Vec<_>>>()?;
    paths.sort();
    Ok(paths)
}

fn finished(value: &Value, rejected: bool) -> bool {
    matches!(
        value.get("status").and_then(Value::as_str),
        Some("accepted" | "offline-verified" | "mixed-C-Rust-integration")
    ) || rejected && matches!(value.get("status").and_then(Value::as_str), Some("rejected" | "failed"))
}

fn scan(path: &Path, targets: &mut Vec<PathBuf>, protected: &mut BTreeSet<PathBuf>) -> Result<()> {
    let verdict = path.join("verdict.toml");
    let has_verdict = verdict.is_file() && !verdict.is_symlink();
    let manifest = path.join("manifest.toml");
    let native = if has_verdict && manifest.is_file() && !manifest.is_symlink() {
        let manifest = document(&manifest)?;
        let verdict = document(&verdict)?;
        if manifest.get("state").and_then(Value::as_str) == Some("complete")
            && manifest.get("run_id").and_then(Value::as_str).is_some()
            && manifest.get("run_id") == verdict.get("run_id")
            && [Some("pass"), Some("divergence"), Some("uncovered"), Some("incomplete"), Some("error")]
                .contains(&verdict.get("status").and_then(Value::as_str))
        {
            for entries in [
                manifest.get("identities").and_then(Value::as_table),
                manifest.get("check_evidence").and_then(Value::as_table),
            ]
            .into_iter()
            .flatten()
            {
                for entry in entries.values() {
                    if let Some(relative) =
                        entry.get("evidence").and_then(Value::as_str).or(entry.get("path").and_then(Value::as_str))
                    {
                        protected.insert(normalize(&path.join(relative)));
                    }
                }
            }
            true
        } else {
            false
        }
    } else {
        false
    };
    for p in entries(path)? {
        let meta = p.symlink_metadata()?;
        if meta.file_type().is_symlink() {
            continue;
        }
        let name = p.file_name().unwrap().to_string_lossy();
        if meta.is_dir() {
            if name == "target" || native && name == "build" {
                targets.push(p);
            } else {
                scan(&p, targets, protected)?;
            }
        } else {
            if has_verdict && ["libfamilies.a", "librng.a"].contains(&name.as_ref()) {
                targets.push(p.clone());
            }
            if ["verdict.toml", "acceptance.toml"].contains(&name.as_ref()) {
                let mut names = Vec::new();
                inventories(&document(&p)?, &mut names);
                protected.extend(names.into_iter().map(|name| normalize(&path.join(name))));
            }
        }
    }
    Ok(())
}

fn candidates(root: &Path) -> Result<Vec<PathBuf>> {
    let mut targets = Vec::new();
    let mut protected = BTreeSet::new();
    scan(root, &mut targets, &mut protected)?;
    let mut result = Vec::new();
    for target in targets {
        let verdict = target.parent().unwrap().join("verdict.toml");
        if !verdict.is_file() || verdict.is_symlink() {
            continue;
        }
        let data = document(&verdict)?;
        let native = target.file_name().is_some_and(|n| n == "build") && {
            let path = target.parent().unwrap().join("manifest.toml");
            if path.is_file() && !path.is_symlink() {
                let manifest = document(&path)?;
                manifest.get("state").and_then(Value::as_str) == Some("complete")
                    && manifest.get("run_id").and_then(Value::as_str).is_some()
                    && manifest.get("run_id") == data.get("run_id")
                    && [Some("pass"), Some("divergence"), Some("uncovered"), Some("incomplete"), Some("error")]
                        .contains(&data.get("status").and_then(Value::as_str))
            } else {
                false
            }
        };
        if !finished(&data, true) && !native {
            continue;
        }
        if target.file_name().unwrap() != "target"
            && data.get("status").and_then(Value::as_str) != Some("mixed-C-Rust-integration")
            && !native
        {
            continue;
        }
        if !protected.iter().any(|p| p.starts_with(&target)) {
            result.push(target);
        }
    }
    Ok(result)
}

fn size(path: &Path) -> Result<u64> {
    let meta = path.symlink_metadata()?;
    if meta.file_type().is_symlink() {
        return Ok(0);
    }
    if meta.is_file() {
        return Ok(meta.len());
    }
    entries(path)?.into_iter().try_fold(0, |total, p| Ok(total + size(&p)?))
}

fn snapshot(path: &Path) -> bool {
    let name = path.file_name().unwrap().to_string_lossy();
    path.extension().is_some_and(|v| v == "wram")
        || ["terminal.bin", "rom-view.bin", "hardware.log", "effects.log", "port-effects.log", "routine-calls.log"]
            .contains(&name.as_ref())
        || name.starts_with("frame-") && path.extension().is_some_and(|v| v == "bin")
}

fn safe_snapshot(base: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(base) else {
        return false;
    };
    if relative.components().any(|c| !matches!(c, Component::Normal(_))) {
        return false;
    }
    let mut current = base.to_path_buf();
    for component in relative.components() {
        current.push(component);
        if current.is_symlink() {
            return false;
        }
    }
    path.is_file() && snapshot(path)
}

#[cfg(unix)]
fn deduplicate_tree(root: &Path, apply: bool) -> Result<(u64, usize)> {
    use std::os::unix::fs::MetadataExt;
    let mut pending = vec![root.to_path_buf()];
    let mut seen: BTreeMap<(u64, u64, String), PathBuf> = BTreeMap::new();
    let mut reclaimed_inodes = BTreeSet::new();
    let mut total = 0;
    let mut checked = 0;
    while let Some(base) = pending.pop() {
        let verdict = base.join("verdict.toml");
        let value = if verdict.is_file() && !verdict.is_symlink() { document(&verdict).ok() } else { None };
        let inventory =
            value.as_ref().filter(|v| finished(v, false)).and_then(|v| v.get("artifacts").and_then(Value::as_table));
        if let Some(inventory) = inventory {
            let mixed =
                value.as_ref().unwrap().get("status").and_then(Value::as_str) == Some("mixed-C-Rust-integration");
            for (relative, expected) in inventory {
                let choices = if mixed {
                    ["c-0", "c-1", "mixed-0", "mixed-1"]
                        .into_iter()
                        .map(|subject| base.join(subject).join(relative))
                        .collect()
                } else {
                    vec![base.join(relative)]
                };
                for path in choices {
                    if !safe_snapshot(&base, &path) {
                        continue;
                    }
                    checked += 1;
                    let identity = digest(&path)?;
                    if expected.as_str() != Some(&identity) {
                        continue;
                    }
                    let meta = path.metadata()?;
                    let key = (meta.dev(), meta.len(), identity.clone());
                    let Some(previous) = seen.get(&key) else {
                        seen.insert(key, path);
                        continue;
                    };
                    if previous.metadata()?.ino() == meta.ino() {
                        continue;
                    }
                    let payload = fs::read(&path)?;
                    if fs::read(previous)? != payload {
                        continue;
                    }
                    if meta.nlink() == 1 && reclaimed_inodes.insert((meta.dev(), meta.ino())) {
                        total += meta.len();
                    }
                    if apply {
                        ensure!(safe_snapshot(&base, &path) && !previous.is_symlink(), "snapshot path changed");
                        ensure!(
                            digest(&path)? == identity && fs::read(previous)? == payload,
                            "snapshot changed: {}",
                            path.display()
                        );
                        let temporary = path.with_file_name(format!(
                            "{}.dedup-link-{}-{}",
                            path.file_name().unwrap().to_string_lossy(),
                            std::process::id(),
                            NEXT.fetch_add(1, Ordering::Relaxed)
                        ));
                        fs::hard_link(previous, &temporary)?;
                        let result = fs::rename(&temporary, &path);
                        if result.is_err() {
                            let _ = fs::remove_file(&temporary);
                        }
                        result?;
                    }
                }
            }
        } else {
            for path in entries(&base)? {
                if path.is_dir() && !path.is_symlink() && path.file_name().unwrap() != "target" {
                    pending.push(path);
                }
            }
        }
    }
    Ok((total, checked))
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CleanupRequest {
    pub root: PathBuf,
    #[serde(default)]
    pub apply: bool,
    #[serde(default)]
    pub deduplicate: bool,
}
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CleanupTarget {
    pub path: PathBuf,
    pub bytes: u64,
}
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct CleanupReceipt {
    pub apply: bool,
    pub bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshots_checked: Option<usize>,
    #[serde(default)]
    pub targets: Vec<CleanupTarget>,
}
pub fn cleanup(request: CleanupRequest) -> Result<CleanupReceipt> {
    let requested = request.root;
    ensure!(requested.is_dir() && !requested.is_symlink(), "root must be an existing real directory");
    let root = fs::canonicalize(requested)?;
    let apply = request.apply;
    if request.deduplicate {
        #[cfg(unix)]
        {
            let (bytes, checked) = deduplicate_tree(&root, apply)?;
            return Ok(CleanupReceipt { apply, bytes, snapshots_checked: Some(checked), targets: vec![] });
        }
        #[cfg(not(unix))]
        anyhow::bail!("snapshot deduplication requires Unix file identities");
    }
    let targets = candidates(&root)?;
    let mut removed = Vec::new();
    let mut total = 0;
    for target in targets {
        let bytes = size(&target)?;
        if apply {
            ensure!(candidates(&root)?.contains(&target), "cleanup eligibility changed: {}", target.display());
            if target.is_dir() {
                fs::remove_dir_all(&target)?;
            } else {
                fs::remove_file(&target)?;
            }
        }
        total += bytes;
        removed.push(CleanupTarget { path: target, bytes });
    }
    Ok(CleanupReceipt { apply, bytes: total, snapshots_checked: None, targets: removed })
}
