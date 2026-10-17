//! Bounded source-only exchange. The supervisor owns all filesystem mutations.
use crate::model::digest_valid;
pub use crate::model::{
    CapsuleBinding, Replacement, RewriteCapsule, RewriteLimits, SourceApplied, SourceFile, SourceReplacements,
};
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::MetadataExt,
    },
    path::{Component, Path},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_FILES: usize = 128;
pub const MAX_PROTOCOL_BYTES: u64 = 64 * 1024 * 1024;

fn read_deadline(deadline_unix_ms: Option<u64>) -> Result<Instant> {
    let maximum = Duration::from_secs(120);
    let remaining = if let Some(deadline) = deadline_unix_ms {
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        ensure!(now < deadline, "rewrite source time budget exhausted before reading");
        Duration::from_millis(deadline - now).min(maximum)
    } else {
        maximum
    };
    Ok(Instant::now() + remaining)
}

fn check_deadline(deadline: Instant) -> Result<()> {
    ensure!(Instant::now() < deadline, "rewrite source time budget exhausted");
    Ok(())
}

fn original_bytes(file: &mut File, maximum: u64, deadline: Instant) -> Result<Vec<u8>> {
    check_deadline(deadline)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && before.nlink() == 1, "source must be a regular unaliased file");
    ensure!(before.len() <= maximum, "source exceeds remaining aggregate or per-file limit");
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 65536];
    loop {
        check_deadline(deadline)?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        ensure!(count as u64 <= maximum.saturating_sub(bytes.len() as u64), "source grew beyond limit");
        bytes.extend_from_slice(&buffer[..count]);
    }
    let after = file.metadata()?;
    ensure!(
        bytes.len() as u64 == before.len()
            && after.len() == before.len()
            && after.nlink() == 1
            && after.mtime() == before.mtime()
            && after.mtime_nsec() == before.mtime_nsec()
            && after.ctime() == before.ctime()
            && after.ctime_nsec() == before.ctime_nsec(),
        "source changed while reading"
    );
    check_deadline(deadline)?;
    Ok(bytes)
}

fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn relative(path: &str) -> Result<()> {
    ensure!(!path.is_empty() && path.len() <= 4096 && !path.contains(['\\', '\0']), "invalid replacement path");
    let p = Path::new(path);
    ensure!(p.components().all(|c| matches!(c, Component::Normal(_))), "replacement path must be root-relative");
    ensure!(
        p.to_str() == Some(path) && path.split('/').all(|part| !part.is_empty() && part != "." && part != ".."),
        "noncanonical replacement path"
    );
    ensure!(
        !p.components().any(|c| c
            .as_os_str()
            .to_str()
            .is_some_and(|v| matches!(v, ".git" | "out" | "target" | ".scratch" | ".codex"))),
        "replacement path includes protected state"
    );
    ensure!(
        p.extension().is_some_and(|extension| extension == "rs")
            && !p.components().any(|component| component
                .as_os_str()
                .to_str()
                .is_some_and(|part| { matches!(part, "tests" | "test" | "manifests" | "fixtures") })),
        "replacement must name Rust source outside frozen verification declarations"
    );
    Ok(())
}

fn limits_valid(limits: RewriteLimits) -> Result<()> {
    ensure!(
        limits.max_context_bytes > 0
            && limits.max_response_bytes > 0
            && limits.max_file_bytes > 0
            && limits.max_total_bytes >= limits.max_file_bytes,
        "rewrite byte limits must be positive and consistent"
    );
    ensure!(
        [limits.max_context_bytes, limits.max_response_bytes, limits.max_file_bytes, limits.max_total_bytes]
            .iter()
            .all(|limit| *limit <= MAX_PROTOCOL_BYTES),
        "rewrite limits exceed the protocol ceiling"
    );
    Ok(())
}

fn capsule_id(capsule: &RewriteCapsule) -> Result<String> {
    Ok(hash(&serde_json::to_vec(&(
        &capsule.run_id,
        capsule.attempt,
        &capsule.policy_sha256,
        &capsule.source_sha256,
        &capsule.files,
        &capsule.context,
    ))?))
}

pub(crate) fn validate_capsule(capsule: &RewriteCapsule, limits: RewriteLimits) -> Result<()> {
    limits_valid(limits)?;
    ensure!(!capsule.run_id.is_empty() && capsule.attempt > 0, "capsule lacks attempt identity");
    ensure!(
        digest_valid(&capsule.policy_sha256) && digest_valid(&capsule.source_sha256),
        "capsule lacks frozen identity"
    );
    ensure!(capsule.context.len() as u64 <= limits.max_context_bytes, "rewrite context exceeds limit");
    ensure!(!capsule.files.is_empty() && capsule.files.len() <= MAX_FILES, "invalid capsule file count");
    let mut names = BTreeSet::new();
    let mut bytes = 0u64;
    for file in &capsule.files {
        relative(&file.path)?;
        ensure!(names.insert(&file.path), "duplicate capsule source path");
        ensure!(file.content.len() as u64 <= limits.max_file_bytes, "capsule source exceeds file limit");
        bytes = bytes.checked_add(file.content.len() as u64).context("capsule size overflow")?;
        ensure!(hash(file.content.as_bytes()) == file.sha256, "capsule source hash differs");
    }
    ensure!(bytes <= limits.max_total_bytes, "capsule sources exceed aggregate limit");
    ensure!(capsule_id(capsule)? == capsule.id, "capsule identity differs from contents");
    Ok(())
}

/// Capture only the approved source files. No directories, aliases or hardlinks are admitted.
pub fn capture(
    root: &Path,
    binding: CapsuleBinding,
    paths: &[String],
    context: String,
    limits: RewriteLimits,
) -> Result<RewriteCapsule> {
    capture_inner(root, binding, paths, context, limits, read_deadline(None)?)
}

/// Source capture shares the supervisor's persisted preparation deadline.
pub fn capture_until(
    root: &Path,
    binding: CapsuleBinding,
    paths: &[String],
    context: String,
    limits: RewriteLimits,
    deadline_unix_ms: u64,
) -> Result<RewriteCapsule> {
    capture_inner(root, binding, paths, context, limits, read_deadline(Some(deadline_unix_ms))?)
}

fn capture_inner(
    root: &Path,
    binding: CapsuleBinding,
    paths: &[String],
    context: String,
    limits: RewriteLimits,
    deadline: Instant,
) -> Result<RewriteCapsule> {
    check_deadline(deadline)?;
    limits_valid(limits)?;
    ensure!(!paths.is_empty() && paths.len() <= MAX_FILES, "invalid approved source file count");
    ensure!(context.len() as u64 <= limits.max_context_bytes, "rewrite context exceeds limit");
    let mut files = Vec::new();
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    for path in paths {
        check_deadline(deadline)?;
        relative(path)?;
        ensure!(names.insert(path), "duplicate approved source path");
        let (parent, name) = parent(root, path)?;
        let mut file = open_file(&parent, &name)?;
        let bytes = original_bytes(
            &mut file,
            limits.max_file_bytes.min(limits.max_total_bytes.saturating_sub(total)),
            deadline,
        )?;
        total = total.checked_add(bytes.len() as u64).context("source size overflow")?;
        ensure!(total <= limits.max_total_bytes, "sources exceed aggregate limit");
        files.push(SourceFile { path: path.clone(), sha256: hash(&bytes), content: String::from_utf8(bytes)? });
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut capsule = RewriteCapsule {
        id: String::new(),
        run_id: binding.run_id,
        attempt: binding.attempt,
        policy_sha256: binding.policy_sha256,
        source_sha256: binding.source_sha256,
        files,
        context,
    };
    capsule.id = capsule_id(&capsule)?;
    validate_capsule(&capsule, limits)?;
    check_deadline(deadline)?;
    Ok(capsule)
}

pub fn encode(capsule: &RewriteCapsule, limits: RewriteLimits) -> Result<Vec<u8>> {
    validate_capsule(capsule, limits)?;
    let bytes = serde_json::to_vec(capsule)?;
    ensure!(bytes.len() as u64 <= MAX_PROTOCOL_BYTES, "encoded capsule exceeds protocol ceiling");
    Ok(bytes)
}

/// Parse capsule text within the same immutable source-only protocol as capture.
pub fn decode_capsule(bytes: &[u8], limits: RewriteLimits) -> Result<RewriteCapsule> {
    ensure!(bytes.len() as u64 <= MAX_PROTOCOL_BYTES, "encoded capsule exceeds protocol ceiling");
    let capsule: RewriteCapsule = serde_json::from_slice(bytes).context("invalid rewrite capsule")?;
    validate_capsule(&capsule, limits)?;
    Ok(capsule)
}

pub fn decode(capsule: &RewriteCapsule, bytes: &[u8], limits: RewriteLimits) -> Result<SourceReplacements> {
    validate_capsule(capsule, limits)?;
    ensure!(bytes.len() as u64 <= limits.max_response_bytes, "rewrite response exceeds limit");
    let response: SourceReplacements = serde_json::from_slice(bytes).context("invalid source replacement response")?;
    validate_replacements(capsule, &response, limits)?;
    Ok(response)
}

fn validate_replacements(capsule: &RewriteCapsule, response: &SourceReplacements, limits: RewriteLimits) -> Result<()> {
    ensure!(response.capsule_id == capsule.id, "stale or foreign rewrite capsule");
    ensure!(
        !response.files.is_empty() && response.files.len() <= capsule.files.len(),
        "invalid replacement file count"
    );
    let approved: BTreeMap<_, _> = capsule.files.iter().map(|f| (&f.path, f)).collect();
    let mut paths = BTreeSet::new();
    let mut total = 0u64;
    for replacement in &response.files {
        relative(&replacement.path)?;
        ensure!(paths.insert(&replacement.path), "duplicate replacement path");
        let original = approved.get(&replacement.path).context("replacement names an unapproved source file")?;
        ensure!(replacement.base_sha256 == original.sha256, "replacement base hash differs");
        ensure!(replacement.content.len() as u64 <= limits.max_file_bytes, "replacement exceeds file limit");
        total = total.checked_add(replacement.content.len() as u64).context("replacement size overflow")?;
    }
    ensure!(total <= limits.max_total_bytes, "replacements exceed aggregate limit");
    ensure!(serde_json::to_vec(response)?.len() as u64 <= limits.max_response_bytes, "rewrite response exceeds limit");
    Ok(())
}

fn cstr(text: &str) -> Result<CString> {
    CString::new(text).context("path includes NUL")
}

// Directory descriptors keep path traversal rooted and O_NOFOLLOW rejects aliases.
fn parent(root: &Path, relative_path: &str) -> Result<(File, CString)> {
    ensure!(root.is_absolute(), "source root must be absolute");
    let normalized = root.components().collect::<std::path::PathBuf>();
    ensure!(normalized.as_os_str() == root.as_os_str(), "source root is noncanonical");
    // SAFETY: valid NUL-terminated name; returned descriptor is owned below.
    let descriptor =
        unsafe { libc::open(c"/".as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC) };
    ensure!(descriptor >= 0, "cannot open unaliased source root: {}", std::io::Error::last_os_error());
    // SAFETY: descriptor was successfully opened and is uniquely owned.
    let mut directory = unsafe { File::from_raw_fd(descriptor) };
    for component in root.components() {
        if matches!(component, Component::RootDir) {
            continue;
        }
        ensure!(matches!(component, Component::Normal(_)), "source root is aliased");
        let name = CString::new(component.as_os_str().as_encoded_bytes())?;
        // SAFETY: each ancestor is opened from its already checked descriptor.
        let next = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        ensure!(next >= 0, "source root ancestor is absent or aliased: {}", std::io::Error::last_os_error());
        // SAFETY: next is a newly owned descriptor.
        directory = unsafe { File::from_raw_fd(next) };
    }
    let mut parts = relative_path.split('/').peekable();
    while let Some(part) = parts.next() {
        let name = cstr(part)?;
        if parts.peek().is_none() {
            return Ok((directory, name));
        }
        // SAFETY: directory is live and name is NUL-terminated.
        let next = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        ensure!(next >= 0, "source ancestor is absent or aliased: {}", std::io::Error::last_os_error());
        // SAFETY: next is a newly owned descriptor.
        directory = unsafe { File::from_raw_fd(next) };
    }
    anyhow::bail!("source filename missing")
}

fn open_file(parent: &File, name: &CString) -> Result<File> {
    // SAFETY: parent is a directory descriptor and name is NUL-terminated.
    let fd = unsafe {
        libc::openat(
            parent.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK,
        )
    };
    ensure!(fd >= 0, "source file is absent or aliased: {}", std::io::Error::last_os_error());
    // SAFETY: successful open transfers unique ownership to File.
    Ok(unsafe { File::from_raw_fd(fd) })
}

/// Validate every original before changing any approved source file.
/// The caller owns the source checkout exclusively and journals application intent.
pub fn apply(
    root: &Path,
    capsule: &RewriteCapsule,
    response: &SourceReplacements,
    limits: RewriteLimits,
) -> Result<SourceApplied> {
    apply_inner(root, capsule, response, limits, read_deadline(None)?)
}

pub fn apply_until(
    root: &Path,
    capsule: &RewriteCapsule,
    response: &SourceReplacements,
    limits: RewriteLimits,
    deadline_unix_ms: u64,
) -> Result<SourceApplied> {
    apply_inner(root, capsule, response, limits, read_deadline(Some(deadline_unix_ms))?)
}

fn apply_inner(
    root: &Path,
    capsule: &RewriteCapsule,
    response: &SourceReplacements,
    limits: RewriteLimits,
    deadline: Instant,
) -> Result<SourceApplied> {
    check_deadline(deadline)?;
    validate_capsule(capsule, limits)?;
    validate_replacements(capsule, response, limits)?;
    let mut originals = BTreeMap::new();
    for original in &capsule.files {
        check_deadline(deadline)?;
        let (directory, name) = parent(root, &original.path)?;
        let mut file = open_file(&directory, &name)?;
        let metadata = file.metadata()?;
        ensure!(metadata.is_file() && metadata.nlink() == 1, "source is aliased or not a regular file");
        let bytes = original_bytes(&mut file, limits.max_file_bytes, deadline)?;
        ensure!(hash(&bytes) == original.sha256, "source changed since rewrite capsule: {}", original.path);
        originals.insert(original.path.clone(), (directory, name, metadata));
    }
    let mut applied: BTreeMap<String, String> =
        capsule.files.iter().map(|f| (f.path.clone(), f.sha256.clone())).collect();
    for replacement in &response.files {
        check_deadline(deadline)?;
        let (directory, name, metadata) = &originals[&replacement.path];
        let pending = cstr(&format!(
            ".translation-rewrite-{}-{}",
            std::process::id(),
            &hash(replacement.content.as_bytes())[..16]
        ))?;
        // SAFETY: directory descriptor and NUL-terminated filename are valid.
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                pending.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                metadata.mode() & 0o777,
            )
        };
        ensure!(fd >= 0, "cannot stage source replacement: {}", std::io::Error::last_os_error());
        // SAFETY: newly opened descriptor is uniquely owned.
        let mut file = unsafe { File::from_raw_fd(fd) };
        let result = (|| -> Result<()> {
            file.write_all(replacement.content.as_bytes())?;
            file.sync_all()?;
            let current = open_file(directory, name)?.metadata()?;
            ensure!(
                current.ino() == metadata.ino() && current.dev() == metadata.dev() && current.nlink() == 1,
                "source path changed during replacement"
            );
            // SAFETY: both names live in the same directory descriptor.
            let status = unsafe {
                libc::renameat(directory.as_raw_fd(), pending.as_ptr(), directory.as_raw_fd(), name.as_ptr())
            };
            ensure!(status == 0, "cannot publish replacement: {}", std::io::Error::last_os_error());
            directory.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            // SAFETY: remove only the owned staged filename in this directory.
            unsafe {
                libc::unlinkat(directory.as_raw_fd(), pending.as_ptr(), 0);
            }
        }
        result?;
        applied.insert(replacement.path.clone(), hash(replacement.content.as_bytes()));
    }
    check_deadline(deadline)?;
    Ok(SourceApplied { files_sha256: applied })
}

/// A production transport can send this schema to an external text-only worker.
/// It intentionally contains no command, evidence, authority or stage fields.
pub fn response_schema() -> serde_json::Value {
    serde_json::json!({
        "type":"object", "additionalProperties":false, "required":["capsule_id","files"],
        "properties":{
            "capsule_id":{"type":"string"},
            "files":{"type":"array","minItems":1,"maxItems":MAX_FILES,"items":{
                "type":"object","additionalProperties":false,"required":["path","base_sha256","content"],
                "properties":{"path":{"type":"string"},"base_sha256":{"type":"string"},"content":{"type":"string"}}
            }}
        }
    })
}
