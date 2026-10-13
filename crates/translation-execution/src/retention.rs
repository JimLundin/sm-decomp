//! An exact retention closure for controller-owned outputs. The closure and both
//! cleanup observations survive checkout retirement; unknown files are never
//! inferred to be disposable from directory contents.
use crate::model::{OperationContext, digest_valid, sha256};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Component, Path, PathBuf},
};

pub(crate) const MAX_METADATA_BYTES: u64 = 4 * 1024 * 1024;
const MAX_REFERENCES: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RetainedKind {
    Accepted,
    Failed,
    Source,
    Historical,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Retained {
    pub kind: RetainedKind,
    pub reference: Reference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RetentionResult {
    pub manifest: Reference,
    pub cleanup_records: [Reference; 2],
    /// Logical bytes in the exact retained closure, including cleanup records.
    pub retained_bytes: u64,
    /// Measured logical bytes of explicitly removed files. The cleanup records
    /// separately retain allocated-byte measurements for hard-linked files.
    pub reclaimed_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Removal {
    reference: Reference,
    device: u64,
    inode: u64,
    allocated_bytes: u64,
    links: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    context: OperationContext,
    byte_limit: u64,
    retained: Vec<Retained>,
    removed: Vec<Removal>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cleanup {
    schema: String,
    manifest_sha256: String,
    pass: u8,
    retained_closure_bytes: u64,
    reclaimed_bytes: u64,
    allocated_bytes_reclaimed: u64,
    removed: Vec<Reference>,
    retained_authenticatable: bool,
}

pub(crate) fn safe_path(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "controller path must be absolute");
    let mut current = PathBuf::new();
    for component in path.components() {
        ensure!(matches!(component, Component::RootDir | Component::Normal(_)), "controller path alias refused");
        current.push(component);
        match current.symlink_metadata() {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "controller path crosses a symlink"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

pub(crate) fn directory(path: &Path) -> Result<()> {
    safe_path(path)?;
    fs::create_dir_all(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() },
        "controller directory has a foreign owner"
    );
    ensure!(metadata.mode() & 0o022 == 0, "controller directory is writable by other users");
    Ok(())
}

pub(crate) fn regular(path: &Path, maximum: u64) -> Result<File> {
    safe_path(path)?;
    let file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let metadata = file.metadata()?;
    ensure!(metadata.is_file() && metadata.len() <= maximum, "retained file extent exceeded or nonregular file");
    Ok(file)
}

pub(crate) fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    let mut file = regular(path, maximum)?;
    let expected = file.metadata()?.len();
    let mut bytes = Vec::new();
    (&mut file).take(maximum.saturating_add(1)).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 == expected && bytes.len() as u64 <= maximum,
        "retained file changed or exceeded its extent"
    );
    Ok(bytes)
}

/// Publish exactly once with fsync before and after rename. An orphaned pending
/// file is an identified controller intermediate, not permission to sweep its
/// enclosing directory.
pub(crate) fn write_once(path: &Path, bytes: &[u8], maximum: u64) -> Result<()> {
    safe_path(path)?;
    ensure!(bytes.len() as u64 <= maximum, "controller record budget exceeded");
    let parent = path.parent().context("controller record parent missing")?;
    directory(parent)?;
    if path.try_exists()? {
        ensure!(read_bounded(path, maximum)? == bytes, "immutable controller record changed");
        return Ok(());
    }
    let pending = path.with_extension("pending");
    safe_path(&pending)?;
    if pending.try_exists()? {
        let metadata = pending.symlink_metadata()?;
        ensure!(metadata.is_file() && metadata.len() <= maximum, "unrecognized controller pending extent");
        fs::remove_file(&pending)?;
    }
    let mut file =
        OpenOptions::new().create_new(true).write(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    file.set_permissions(fs::Permissions::from_mode(0o400))?;
    fs::rename(&pending, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

pub fn seal_file(path: &Path, maximum: u64) -> Result<Reference> {
    let mut file = regular(path, maximum)?;
    let before = file.metadata()?;
    let mut hash = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).context("retained extent overflow")?;
        ensure!(count <= maximum, "retained file exceeded budget while sealing");
        hash.update(&buffer[..n]);
    }
    let after = file.metadata()?;
    ensure!(
        count == before.len()
            && after.len() == before.len()
            && after.dev() == before.dev()
            && after.ino() == before.ino()
            && after.mtime() == before.mtime()
            && after.mtime_nsec() == before.mtime_nsec()
            && after.ctime() == before.ctime()
            && after.ctime_nsec() == before.ctime_nsec(),
        "retained file changed while sealing"
    );
    Ok(Reference { path: path.to_path_buf(), sha256: format!("{:x}", hash.finalize()), bytes: count })
}

pub fn authenticate_reference(reference: &Reference, maximum: u64) -> Result<()> {
    ensure!(digest_valid(&reference.sha256) && reference.bytes <= maximum, "invalid retention reference");
    ensure!(seal_file(&reference.path, reference.bytes)? == *reference, "retained content seal changed");
    Ok(())
}

pub fn authenticate(retained: &[Retained], byte_limit: u64) -> Result<u64> {
    ensure!(retained.len() <= MAX_REFERENCES, "retention reference count exceeded");
    let mut paths = BTreeMap::new();
    let mut bytes = 0u64;
    for item in retained {
        let reference = &item.reference;
        if let Some(existing) = paths.insert(reference.path.clone(), reference.clone()) {
            ensure!(existing == *reference, "conflicting retention reference");
            continue;
        }
        bytes = bytes.checked_add(reference.bytes).context("retention extent overflow")?;
        ensure!(bytes <= byte_limit, "aggregate retained evidence budget exceeded");
        authenticate_reference(reference, byte_limit)?;
    }
    Ok(bytes)
}

fn lock(path: &Path) -> Result<File> {
    safe_path(path)?;
    let file = OpenOptions::new().create(true).append(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(path)?;
    ensure!(file.metadata()?.is_file(), "retention lock is not a regular file");
    let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
    ensure!(result == 0, "retention closure has a surviving owner");
    Ok(file)
}

/// Finalize only the supplied closure and explicit file removals. Callers keep
/// source snapshots at retained paths before retiring their Git checkout.
pub fn finalize(
    root: &Path,
    context: &OperationContext,
    retained: &[Retained],
    disposable: &[Reference],
    byte_limit: u64,
) -> Result<RetentionResult> {
    ensure!(digest_valid(&context.operation_key) && context.attempt > 0, "invalid retention operation identity");
    directory(root)?;
    let record_root = root.join(format!("retention-{}", context.operation_key));
    directory(&record_root)?;
    let _lock = lock(&record_root.join("lock"))?;
    let path = record_root.join("manifest.json");
    let bytes = authenticate(retained, byte_limit)?;
    ensure!(disposable.len() <= MAX_REFERENCES, "disposable reference count exceeded");
    let mut retained = retained.to_vec();
    retained.sort_by(|a, b| a.reference.path.cmp(&b.reference.path));
    let mut disposable = disposable.to_vec();
    disposable.sort_by(|a, b| a.path.cmp(&b.path));
    let mut seen = BTreeSet::new();
    for item in &disposable {
        safe_path(&item.path)?;
        ensure!(
            item.path.starts_with(root) && !item.path.starts_with(&record_root),
            "disposable file is outside controller ownership"
        );
        ensure!(seen.insert(item.path.clone()), "duplicate disposable file");
        ensure!(!retained.iter().any(|r| r.reference.path == item.path), "retained file cannot be retired");
    }
    let manifest: Manifest = if path.try_exists()? {
        let saved: Manifest = serde_json::from_slice(&read_bounded(&path, MAX_METADATA_BYTES)?)?;
        ensure!(
            saved.schema == "translation-retention-v1"
                && saved.context == *context
                && saved.byte_limit == byte_limit
                && saved.retained == retained
                && saved.removed.iter().map(|r| r.reference.clone()).collect::<Vec<_>>() == disposable,
            "retention closure changed on retry"
        );
        saved
    } else {
        let protected_inodes = retained
            .iter()
            .map(|r| {
                let m = regular(&r.reference.path, r.reference.bytes)?.metadata()?;
                Ok((m.dev(), m.ino()))
            })
            .collect::<Result<BTreeSet<_>>>()?;
        let mut removed = Vec::new();
        for reference in &disposable {
            authenticate_reference(reference, byte_limit)?;
            let m = regular(&reference.path, reference.bytes)?.metadata()?;
            ensure!(!protected_inodes.contains(&(m.dev(), m.ino())), "disposable file aliases retained evidence");
            removed.push(Removal {
                reference: reference.clone(),
                device: m.dev(),
                inode: m.ino(),
                allocated_bytes: m.blocks().saturating_mul(512),
                links: m.nlink(),
            });
        }
        Manifest { schema: "translation-retention-v1".into(), context: context.clone(), byte_limit, retained, removed }
    };
    let manifest_bytes = serde_json::to_vec(&manifest)?;
    ensure!(manifest_bytes.len() as u64 <= MAX_METADATA_BYTES, "retention manifest metadata exceeds hard bound");
    let manifest_ref =
        Reference { path: path.clone(), sha256: sha256(&manifest_bytes), bytes: manifest_bytes.len() as u64 };
    let pass1_path = record_root.join("cleanup-1.json");
    let mut reclaimed = 0u64;
    let mut allocated = 0u64;
    for removal in &manifest.removed {
        reclaimed = reclaimed.checked_add(removal.reference.bytes).context("cleanup extent overflow")?;
        if removal.links == 1 {
            allocated = allocated.checked_add(removal.allocated_bytes).context("cleanup allocated extent overflow")?;
        }
    }
    let first = Cleanup {
        schema: "translation-cleanup-v1".into(),
        manifest_sha256: manifest_ref.sha256.clone(),
        pass: 1,
        retained_closure_bytes: bytes,
        reclaimed_bytes: reclaimed,
        allocated_bytes_reclaimed: allocated,
        removed: manifest.removed.iter().map(|r| r.reference.clone()).collect(),
        retained_authenticatable: true,
    };
    // A second checked pass verifies that no accepted dependency was changed
    // by retirement and that every selected controller intermediate is gone.
    // Deduplication of native evidence remains owned by its existing utility.
    let second = Cleanup {
        schema: "translation-cleanup-v1".into(),
        manifest_sha256: manifest_ref.sha256.clone(),
        pass: 2,
        retained_closure_bytes: authenticate(&manifest.retained, byte_limit)?,
        reclaimed_bytes: 0,
        allocated_bytes_reclaimed: 0,
        removed: vec![],
        retained_authenticatable: true,
    };
    let pass2_path = record_root.join("cleanup-2.json");
    let first_bytes = serde_json::to_vec(&first)?;
    let second_bytes = serde_json::to_vec(&second)?;
    ensure!(
        first_bytes.len() as u64 <= MAX_METADATA_BYTES && second_bytes.len() as u64 <= MAX_METADATA_BYTES,
        "cleanup record metadata exceeds hard bound"
    );
    let reserved = bytes
        .checked_add(manifest_ref.bytes)
        .and_then(|n| n.checked_add(first_bytes.len() as u64))
        .and_then(|n| n.checked_add(second_bytes.len() as u64))
        .context("cleanup metadata reservation overflow")?;
    ensure!(reserved <= byte_limit, "aggregate retained evidence plus cleanup metadata exceeds budget");
    // Reserve all retained bytes before any output publication or removal.
    write_once(&path, &manifest_bytes, MAX_METADATA_BYTES)?;
    for removal in &manifest.removed {
        if removal.reference.path.try_exists()? {
            let m = regular(&removal.reference.path, removal.reference.bytes)?.metadata()?;
            ensure!(
                m.dev() == removal.device && m.ino() == removal.inode,
                "disposable file was replaced after cleanup intent"
            );
            authenticate_reference(&removal.reference, byte_limit)?;
            fs::remove_file(&removal.reference.path)?;
            File::open(removal.reference.path.parent().context("disposable parent missing")?)?.sync_all()?;
        }
    }
    authenticate(&manifest.retained, byte_limit)?;
    write_once(&pass1_path, &first_bytes, MAX_METADATA_BYTES)?;
    ensure!(manifest.removed.iter().all(|r| !r.reference.path.exists()), "cleanup left a selected intermediate");
    authenticate(&manifest.retained, byte_limit)?;
    write_once(&pass2_path, &second_bytes, MAX_METADATA_BYTES)?;
    let cleanup_records = [seal_file(&pass1_path, MAX_METADATA_BYTES)?, seal_file(&pass2_path, MAX_METADATA_BYTES)?];
    let retained_bytes = bytes
        .checked_add(manifest_ref.bytes)
        .and_then(|n| n.checked_add(cleanup_records.iter().map(|r| r.bytes).sum::<u64>()))
        .context("retention metadata extent overflow")?;
    ensure!(retained_bytes <= byte_limit, "aggregate retained evidence plus cleanup metadata exceeds budget");
    Ok(RetentionResult { manifest: manifest_ref, cleanup_records, retained_bytes, reclaimed_bytes: reclaimed })
}

/// Independent authentication after source checkouts have been removed. It
/// follows the saved immutable closure, never an ambient directory traversal.
pub fn authenticate_finalization(result: &RetentionResult, byte_limit: u64) -> Result<()> {
    authenticate_reference(&result.manifest, MAX_METADATA_BYTES)?;
    let manifest: Manifest = serde_json::from_slice(&read_bounded(&result.manifest.path, MAX_METADATA_BYTES)?)?;
    ensure!(
        manifest.schema == "translation-retention-v1" && manifest.byte_limit == byte_limit,
        "retention policy changed"
    );
    let bytes = authenticate(&manifest.retained, byte_limit)?;
    let mut reclaimed = 0u64;
    for (index, reference) in result.cleanup_records.iter().enumerate() {
        authenticate_reference(reference, MAX_METADATA_BYTES)?;
        let cleanup: Cleanup = serde_json::from_slice(&read_bounded(&reference.path, MAX_METADATA_BYTES)?)?;
        ensure!(
            cleanup.schema == "translation-cleanup-v1"
                && cleanup.manifest_sha256 == result.manifest.sha256
                && cleanup.pass as usize == index + 1
                && cleanup.retained_authenticatable
                && cleanup.retained_closure_bytes == bytes,
            "cleanup observation changed"
        );
        reclaimed = reclaimed.checked_add(cleanup.reclaimed_bytes).context("cleanup extent overflow")?;
    }
    ensure!(manifest.removed.iter().all(|r| !r.reference.path.exists()), "retired intermediate was recreated");
    ensure!(reclaimed == result.reclaimed_bytes, "cleanup measured extent changed");
    let measured = bytes + result.manifest.bytes + result.cleanup_records.iter().map(|r| r.bytes).sum::<u64>();
    ensure!(measured == result.retained_bytes && measured <= byte_limit, "retained measured extent changed");
    Ok(())
}
