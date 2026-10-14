//! Trusted preparation IO. Nested operations inherit the earliest deadline
//! and one logical IO/path budget; source identities retain their original
//! SHA-256 values. No candidate code or process is executed here.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{Read, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Component, Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

pub(crate) const MAX_FILE_BYTES: u64 = 64 * 1024 * 1024;
pub(crate) const MAX_DOCUMENT_BYTES: u64 = 32 * 1024 * 1024;
pub(crate) const DEFAULT_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_PATHS: usize = 65536;
pub(crate) const MAX_DECLARED_PATHS: usize = 4 * 1024 * 1024;
const DEFAULT_SECONDS: u64 = 3600;

#[derive(Default)]
struct Usage {
    bytes: u64,
    pinned: BTreeMap<(PathBuf, PathBuf, String), crate::family_model::Source>,
}
#[derive(Clone)]
struct Scope {
    deadline: Instant,
    max_bytes: u64,
    usage: Arc<Mutex<Usage>>,
    paths: Arc<AtomicUsize>,
    max_paths: usize,
}
thread_local! {
    static CURRENT: RefCell<Option<Scope>> = const { RefCell::new(None) };
}
struct Restore(Option<Scope>);
impl Drop for Restore {
    fn drop(&mut self) {
        CURRENT.with(|current| *current.borrow_mut() = self.0.take());
    }
}

/// A worker inherits the exact active counters and deadline. It cannot turn
/// one family's bounded input budget into an independent budget per thread.
#[derive(Clone)]
pub(crate) struct SharedScope(Scope);

pub(crate) fn shared_scope() -> Result<SharedScope> {
    CURRENT.with(|current| current.borrow().clone().map(SharedScope).context("missing shared input scope"))
}

impl SharedScope {
    pub(crate) fn enter<T>(&self, action: impl FnOnce() -> Result<T>) -> Result<T> {
        let previous = CURRENT.with(|current| -> Result<_> {
            ensure!(current.borrow().is_none(), "input worker already has an active scope");
            Ok(current.borrow_mut().replace(self.0.clone()))
        })?;
        let _restore = Restore(previous);
        check()?;
        let result = action();
        if result.is_ok() {
            check()?;
        }
        result
    }
}

pub(crate) fn until<T>(deadline_unix_ms: u64, max_bytes: u64, action: impl FnOnce() -> Result<T>) -> Result<T> {
    ensure!(max_bytes > 0 && max_bytes <= 8 * DEFAULT_BYTES, "trusted input aggregate budget missing or excessive");
    let entered = Instant::now();
    let now = now_unix_ms()?;
    let remaining = Duration::from_millis(deadline_unix_ms.saturating_sub(now));
    let mut deadline = entered + remaining.min(Duration::from_secs(DEFAULT_SECONDS));
    let previous = CURRENT.with(|current| {
        let previous = current.borrow().clone();
        let (max_bytes, usage, paths, max_paths) = if let Some(inherited) = &previous {
            deadline = deadline.min(inherited.deadline);
            (max_bytes.min(inherited.max_bytes), inherited.usage.clone(), inherited.paths.clone(), inherited.max_paths)
        } else {
            (max_bytes, Arc::new(Mutex::new(Usage::default())), Arc::new(AtomicUsize::new(0)), MAX_PATHS)
        };
        *current.borrow_mut() = Some(Scope { deadline, max_bytes, usage, paths, max_paths });
        previous
    });
    let _restore = Restore(previous);
    check()?;
    let result = action();
    if result.is_ok() {
        check()?;
    }
    result
}

pub(crate) fn limited<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    if CURRENT.with(|current| current.borrow().is_some()) {
        check()?;
        let result = action();
        if result.is_ok() {
            check()?;
        }
        return result;
    }
    let deadline = now_unix_ms()?.checked_add(DEFAULT_SECONDS * 1000).context("trusted input deadline overflow")?;
    until(deadline, DEFAULT_BYTES, action)
}

/// One explicit retained-tree pass may revisit paths discovered by another
/// pass. Keep the ordinary per-pass path ceiling while sharing every byte
/// charge and the earliest enclosing deadline. Nested helpers within this
/// pass still share its path counter; this never authorizes a producer.
pub(crate) fn discovery<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    discovery_with_limit(MAX_PATHS, action)
}

/// A reviewed, finite declared workload can set an explicit discovery ceiling.
/// Generic callers keep MAX_PATHS; byte accounting and deadlines remain shared.
pub(crate) fn discovery_with_limit<T>(max_paths: usize, action: impl FnOnce() -> Result<T>) -> Result<T> {
    ensure!(max_paths > 0 && max_paths <= MAX_DECLARED_PATHS, "declared discovery path ceiling exceeded");
    limited(|| {
        let previous = CURRENT.with(|current| {
            let previous = current.borrow().clone().expect("limited discovery scope");
            let mut pass = previous.clone();
            pass.paths = Arc::new(AtomicUsize::new(0));
            pass.max_paths = max_paths;
            *current.borrow_mut() = Some(pass);
            previous
        });
        let _restore = Restore(Some(previous));
        let result = action();
        if result.is_ok() {
            check()?;
        }
        result
    })
}

/// Trusted receipt publication/authentication after durable process teardown.
/// This scope cannot authorize a producer: launchers still check their saved
/// whole-stage deadline. Keep finalization finite even after that deadline.
pub(crate) fn reconcile<T>(max_bytes: u64, action: impl FnOnce() -> Result<T>) -> Result<T> {
    let previous = CURRENT.with(|current| current.borrow_mut().take());
    let _restore = Restore(previous);
    let deadline = now_unix_ms()?.checked_add(60_000).context("trusted reconciliation deadline overflow")?;
    until(deadline, max_bytes, action)
}

pub(crate) fn now_unix_ms() -> Result<u64> {
    Ok(u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?)
}

pub(crate) fn check() -> Result<()> {
    CURRENT.with(|current| {
        if let Some(scope) = &*current.borrow() {
            ensure!(Instant::now() < scope.deadline, "trusted input whole-stage deadline exhausted");
            ensure!(
                scope.usage.lock().unwrap().bytes <= scope.max_bytes,
                "trusted input aggregate byte budget exhausted"
            );
        }
        Ok(())
    })
}

#[cfg(test)]
pub(crate) fn charged_bytes() -> u64 {
    CURRENT.with(|current| current.borrow().as_ref().map_or(0, |scope| scope.usage.lock().unwrap().bytes))
}

pub(crate) fn account(bytes: u64) -> Result<()> {
    check()?;
    CURRENT.with(|current| {
        if let Some(scope) = &*current.borrow() {
            let mut usage = scope.usage.lock().unwrap();
            usage.bytes = usage.bytes.checked_add(bytes).context("trusted input byte budget overflow")?;
            ensure!(usage.bytes <= scope.max_bytes, "trusted input aggregate byte budget exhausted");
        } else {
            ensure!(bytes <= MAX_FILE_BYTES, "trusted input local byte budget exceeded");
        }
        Ok(())
    })
}

pub(crate) fn count(amount: usize) -> Result<()> {
    check()?;
    CURRENT.with(|current| {
        if let Some(scope) = &*current.borrow() {
            let previous = scope
                .paths
                .try_update(Ordering::Relaxed, Ordering::Relaxed, |value| value.checked_add(amount))
                .map_err(|_| anyhow::anyhow!("trusted input path budget overflow"))?;
            let paths = previous + amount;
            ensure!(paths <= scope.max_paths, "trusted input discovery path budget exhausted");
        } else {
            ensure!(amount <= MAX_PATHS, "trusted input local path budget exceeded");
        }
        Ok(())
    })
}

pub(crate) fn safe(path: &Path) -> Result<()> {
    check()?;
    ensure!(path.is_absolute(), "trusted input path must be absolute");
    let raw = path.as_os_str().as_bytes();
    ensure!(
        raw.len() <= 4096 && !raw.split(|byte| *byte == b'/').any(|part| part == b"." || part == b".."),
        "trusted input path alias or extent refused"
    );
    let mut current = PathBuf::new();
    for part in path.components() {
        check()?;
        ensure!(matches!(part, Component::RootDir | Component::Normal(_)), "trusted input path alias refused");
        current.push(part);
        match current.symlink_metadata() {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "trusted input symlink refused"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

pub(crate) fn entries(path: &Path) -> Result<Vec<PathBuf>> {
    limited(|| {
        safe(path)?;
        let mut result = Vec::new();
        for entry in fs::read_dir(path)? {
            check()?;
            count(1)?;
            result.push(entry?.path());
        }
        result.sort();
        Ok(result)
    })
}

pub(crate) fn read(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    read_limit(path.as_ref(), MAX_FILE_BYTES)
}

/// Return the exact bytes and their SHA-256 from one bounded file read.
/// The identity belongs to this snapshot; each call opens the current file.
pub(crate) fn read_with_digest(path: impl AsRef<Path>) -> Result<(Vec<u8>, String)> {
    limited(|| {
        let mut hash = Sha256::new();
        let bytes = read_chunks(path.as_ref(), MAX_FILE_BYTES, |chunk| hash.update(chunk))?;
        Ok((bytes, format!("{:x}", hash.finalize())))
    })
}

pub(crate) fn text(path: impl AsRef<Path>) -> Result<String> {
    Ok(String::from_utf8(read_limit(path.as_ref(), MAX_DOCUMENT_BYTES)?)?)
}

pub(crate) fn read_limit(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    read_chunks(path, maximum, |_| {})
}

fn read_chunks(path: &Path, maximum: u64, mut observe: impl FnMut(&[u8])) -> Result<Vec<u8>> {
    limited(|| {
        safe(path)?;
        let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
        let before = file.metadata()?;
        ensure!(before.is_file() && before.len() <= maximum.min(MAX_FILE_BYTES), "trusted input file extent exceeded");
        account(before.len())?;
        let mut bytes = Vec::with_capacity(usize::try_from(before.len())?);
        let mut buffer = [0u8; 64 * 1024];
        loop {
            check()?;
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            ensure!((bytes.len() as u64).saturating_add(n as u64) <= before.len(), "trusted input grew during read");
            observe(&buffer[..n]);
            bytes.extend_from_slice(&buffer[..n]);
        }
        let after = file.metadata()?;
        ensure!(stable(&before, &after) && bytes.len() as u64 == before.len(), "trusted input changed during read");
        Ok(bytes)
    })
}

fn stable(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    before.dev() == after.dev()
        && before.ino() == after.ino()
        && before.len() == after.len()
        && before.mtime() == after.mtime()
        && before.mtime_nsec() == after.mtime_nsec()
        && before.ctime() == after.ctime()
        && before.ctime_nsec() == after.ctime_nsec()
}

pub(crate) fn digest(path: &Path) -> Result<String> {
    digest_limit(path, MAX_FILE_BYTES)
}

/// A separately reviewed large retained input may be streamed without raising
/// the default read, document, write, or hash limit for other callers.
pub(crate) fn digest_limit(path: &Path, maximum: u64) -> Result<String> {
    limited(|| {
        ensure!(maximum > 0 && maximum <= 128 * 1024 * 1024, "trusted streaming hash bound exceeded");
        safe(path)?;
        let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
        let before = file.metadata()?;
        ensure!(before.is_file() && before.len() <= maximum, "trusted input hash extent exceeded");
        account(before.len())?;
        let mut hash = Sha256::new();
        let mut count = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            check()?;
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            count = count.checked_add(n as u64).context("trusted input hash extent overflow")?;
            ensure!(count <= before.len(), "trusted input grew during hash");
            hash.update(&buffer[..n]);
        }
        ensure!(stable(&before, &file.metadata()?) && count == before.len(), "trusted input changed during hash");
        Ok(format!("{:x}", hash.finalize()))
    })
}

/// Publish an exact retained input through bounded buffers. Existing final
/// bytes are immutable; a failed copy removes only its own unsealed pending
/// file. A crash's identified pending file is recovered on an identical retry.
pub(crate) fn immutable_copy(source: &Path, destination: &Path, expected_sha256: &str, maximum: u64) -> Result<()> {
    limited(|| {
        ensure!(maximum > 0 && maximum <= 128 * 1024 * 1024, "trusted streaming copy bound exceeded");
        safe(source)?;
        safe(destination)?;
        if destination.exists() {
            ensure!(digest_limit(destination, maximum)? == expected_sha256, "immutable streamed input changed");
            return Ok(());
        }
        let pending = destination.with_file_name(format!(
            "{}.pending",
            destination.file_name().context("streamed input name missing")?.to_string_lossy()
        ));
        safe(&pending)?;
        if pending.exists() {
            let metadata = pending.symlink_metadata()?;
            ensure!(metadata.is_file() && metadata.len() <= maximum, "streamed pending input extent changed");
            fs::remove_file(&pending)?;
        }
        let mut input = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(source)?;
        let before = input.metadata()?;
        ensure!(before.is_file() && before.len() <= maximum, "streamed input extent exceeded");
        account(before.len().checked_mul(2).context("streamed input accounting overflow")?)?;
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&pending)?;
        let copied = (|| -> Result<()> {
            let mut hash = Sha256::new();
            let mut count = 0u64;
            let mut buffer = [0u8; 64 * 1024];
            loop {
                check()?;
                let read = input.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                count = count.checked_add(read as u64).context("streamed input extent overflow")?;
                ensure!(count <= before.len(), "streamed input grew during publication");
                hash.update(&buffer[..read]);
                output.write_all(&buffer[..read])?;
            }
            ensure!(
                stable(&before, &input.metadata()?) && count == before.len(),
                "streamed input changed during publication"
            );
            ensure!(format!("{:x}", hash.finalize()) == expected_sha256, "streamed input identity changed");
            output.sync_all()?;
            check()?;
            fs::rename(&pending, destination)?;
            fs::File::open(destination.parent().context("streamed input parent missing")?)?.sync_all()?;
            Ok(())
        })();
        drop(output);
        if copied.is_err() {
            let _ = fs::remove_file(&pending);
        }
        copied
    })
}

pub(crate) fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    limited(|| {
        safe(path)?;
        ensure!(bytes.len() as u64 <= MAX_FILE_BYTES, "trusted input output extent exceeded");
        account(bytes.len() as u64)?;
        let mut file =
            OpenOptions::new().write(true).create(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
        ensure!(file.metadata()?.is_file(), "trusted input output is not a regular file");
        file.set_len(0)?;
        for block in bytes.chunks(64 * 1024) {
            check()?;
            file.write_all(block)?;
        }
        file.sync_all()?;
        Ok(())
    })
}

pub(crate) fn cached_source(key: &(PathBuf, PathBuf, String)) -> Option<crate::family_model::Source> {
    CURRENT.with(|current| {
        current.borrow().as_ref().and_then(|scope| scope.usage.lock().unwrap().pinned.get(key).cloned())
    })
}
pub(crate) fn cache_source(key: (PathBuf, PathBuf, String), source: crate::family_model::Source) {
    CURRENT.with(|current| {
        if let Some(scope) = &*current.borrow() {
            scope.usage.lock().unwrap().pinned.insert(key, source);
        }
    });
}

pub(crate) fn tree(root: &Path, maximum_files: usize, maximum_bytes: u64) -> Result<(usize, u64)> {
    limited(|| {
        let mut pending = vec![(root.to_path_buf(), 0usize)];
        let mut visited = BTreeSet::new();
        let (mut files, mut bytes) = (0usize, 0u64);
        while let Some((path, depth)) = pending.pop() {
            check()?;
            ensure!(depth <= 32, "trusted input tree depth exceeded");
            safe(&path)?;
            let metadata = path.symlink_metadata()?;
            if metadata.is_dir() {
                ensure!(visited.insert((metadata.dev(), metadata.ino())), "trusted input directory alias");
                for child in entries(&path)? {
                    pending.push((child, depth + 1));
                }
            } else {
                ensure!(metadata.is_file(), "trusted input tree contains special files");
                files = files.checked_add(1).context("trusted input tree file extent overflow")?;
                bytes = bytes.checked_add(metadata.len()).context("trusted input tree byte extent overflow")?;
                ensure!(files <= maximum_files && bytes <= maximum_bytes, "trusted input tree extent exceeded");
            }
        }
        account(bytes)?;
        Ok((files, bytes))
    })
}

const MAX_ARCHIVE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ARCHIVE_EXPANDED_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ARCHIVE_MEMBERS: usize = 4096;

/// Validate a gzip/USTAR/PAX archive against an existing pinned source tree.
/// It writes no extracted filesystem paths and retains only one 64 KiB block
/// plus bounded archive/header metadata while comparing each regular member.
pub(crate) fn authenticate_tar_source(
    archive: &Path,
    source: &Path,
    expected_sha256: &str,
) -> Result<BTreeMap<String, String>> {
    limited(|| {
        safe(source)?;
        let packed = read_limit(archive, MAX_ARCHIVE_BYTES)?;
        ensure!(format!("{:x}", Sha256::digest(&packed)) == expected_sha256, "pinned archive identity changed");
        let packed_bytes = packed.len() as u64;
        let mut decoder = flate2::bufread::GzDecoder::new(std::io::Cursor::new(packed));
        let mut expanded = 0u64;
        let mut files = BTreeMap::new();
        let mut names = BTreeSet::new();
        let mut root: Option<String> = None;
        let mut local_pax = BTreeMap::new();
        let mut zeros = 0;
        let mut members = 0usize;
        loop {
            check()?;
            let mut header = [0u8; 512];
            let observed = archive_read(&mut decoder, &mut header, &mut expanded)?;
            ensure!(observed == 512, "pinned tar header is truncated");
            if header.iter().all(|byte| *byte == 0) {
                zeros += 1;
                if zeros == 2 {
                    break;
                }
                continue;
            }
            ensure!(zeros == 0, "pinned tar archive contains entries after its terminator");
            members += 1;
            count(1)?;
            ensure!(members <= MAX_ARCHIVE_MEMBERS, "pinned archive member count exceeded");
            let checksum = tar_number(&header[148..156])?;
            let unsigned = header
                .iter()
                .enumerate()
                .map(|(index, byte)| if (148..156).contains(&index) { 32u64 } else { u64::from(*byte) })
                .sum::<u64>();
            ensure!(checksum == unsigned, "pinned tar checksum changed");
            ensure!(
                [&b"ustar\0"[..], &b"ustar "[..], &b"\0\0\0\0\0\0"[..]].contains(&&header[257..263]),
                "pinned tar header format unsupported"
            );
            let mut name = tar_text(&header[..100])?;
            let prefix = tar_text(&header[345..500])?;
            if !prefix.is_empty() {
                name = format!("{prefix}/{name}");
            }
            let mut bytes = tar_number(&header[124..136])?;
            let kind = header[156];
            if kind == b'x' || kind == b'g' {
                ensure!(bytes <= 65536, "pinned PAX metadata extent exceeded");
                let mut data = vec![0u8; usize::try_from(bytes)?];
                ensure!(
                    archive_read(&mut decoder, &mut data, &mut expanded)? == data.len(),
                    "pinned PAX record truncated"
                );
                let fields = pax_fields(&data)?;
                if kind == b'g' {
                    ensure!(
                        !fields.keys().any(|key| ["path", "size", "linkpath"].contains(&key.as_str())),
                        "pinned global PAX metadata changes member ownership"
                    );
                } else {
                    ensure!(local_pax.is_empty(), "pinned duplicate local PAX metadata");
                    local_pax = fields;
                }
                archive_padding(&mut decoder, bytes, &mut expanded)?;
                continue;
            }
            if let Some(path) = local_pax.remove("path") {
                name = path;
            }
            if let Some(size) = local_pax.remove("size") {
                bytes = size.parse()?;
            }
            ensure!(!local_pax.contains_key("linkpath"), "pinned source archive link metadata refused");
            local_pax.clear();
            ensure!(name.len() <= 4096 && !name.is_empty(), "pinned archive source path extent exceeded");
            let normalized = name.trim_end_matches('/');
            let path = Path::new(normalized);
            ensure!(
                !path.is_absolute()
                    && normalized.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
                    && path.components().all(|part| matches!(part, Component::Normal(_))),
                "unsafe source archive member"
            );
            let mut parts = path.components();
            let top = parts
                .next()
                .context("pinned archive root missing")?
                .as_os_str()
                .to_str()
                .context("archive root UTF8")?;
            if let Some(root) = &root {
                ensure!(root == top, "pinned archive has multiple source roots");
            } else {
                root = Some(top.into());
            }
            let relative = parts.as_path();
            ensure!(names.insert(normalized.to_owned()), "pinned archive repeats a source path");
            match kind {
                b'5' => ensure!(bytes == 0, "pinned archive directory has payload"),
                0 | b'0' => {
                    ensure!(
                        !relative.as_os_str().is_empty() && bytes <= MAX_FILE_BYTES,
                        "pinned source member extent exceeded"
                    );
                    let target = source.join(relative);
                    safe(&target)?;
                    let mut material = OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                        .open(&target)?;
                    let before = material.metadata()?;
                    ensure!(
                        before.is_file() && before.len() == bytes,
                        "pinned source extent changed: {}",
                        relative.display()
                    );
                    account(bytes)?;
                    let mut hash = Sha256::new();
                    let mut remaining = bytes;
                    let mut expected = [0u8; 64 * 1024];
                    let mut current = [0u8; 64 * 1024];
                    while remaining > 0 {
                        check()?;
                        let chunk = usize::try_from(remaining.min(expected.len() as u64))?;
                        ensure!(
                            archive_read(&mut decoder, &mut expected[..chunk], &mut expanded)? == chunk,
                            "pinned archive member is truncated"
                        );
                        material.read_exact(&mut current[..chunk])?;
                        ensure!(expected[..chunk] == current[..chunk], "pinned source changed: {}", relative.display());
                        hash.update(&current[..chunk]);
                        remaining -= chunk as u64;
                    }
                    ensure!(stable(&before, &material.metadata()?), "pinned source changed during authentication");
                    files.insert(
                        relative.to_str().context("pinned member path UTF8")?.to_owned(),
                        format!("{:x}", hash.finalize()),
                    );
                }
                _ => anyhow::bail!("pinned source archive contains unsupported links or special entries"),
            }
            archive_padding(&mut decoder, bytes, &mut expanded)?;
        }
        ensure!(local_pax.is_empty() && !files.is_empty(), "empty or incomplete pinned source archive");
        let mut trailing = [0u8; 64 * 1024];
        loop {
            let n = archive_read(&mut decoder, &mut trailing, &mut expanded)?;
            if n == 0 {
                break;
            }
            ensure!(trailing[..n].iter().all(|byte| *byte == 0), "pinned archive trailing data changed");
        }
        ensure!(decoder.get_ref().position() == packed_bytes, "pinned gzip has trailing or concatenated data");
        Ok(files)
    })
}

fn archive_read(reader: &mut impl Read, buffer: &mut [u8], expanded: &mut u64) -> Result<usize> {
    let mut count = 0usize;
    while count < buffer.len() {
        check()?;
        let n = reader.read(&mut buffer[count..])?;
        if n == 0 {
            break;
        }
        *expanded = expanded.checked_add(n as u64).context("pinned archive expanded extent overflow")?;
        ensure!(*expanded <= MAX_ARCHIVE_EXPANDED_BYTES, "pinned archive expanded byte budget exceeded");
        account(n as u64)?;
        count += n;
    }
    Ok(count)
}
fn archive_padding(reader: &mut impl Read, bytes: u64, expanded: &mut u64) -> Result<()> {
    let count = usize::try_from((512 - bytes % 512) % 512)?;
    let mut padding = [0u8; 512];
    ensure!(archive_read(reader, &mut padding[..count], expanded)? == count, "pinned archive padding truncated");
    ensure!(padding[..count].iter().all(|byte| *byte == 0), "pinned archive padding changed");
    Ok(())
}
fn tar_text(bytes: &[u8]) -> Result<String> {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    ensure!(bytes[end..].iter().all(|byte| *byte == 0), "pinned archive path has trailing bytes");
    Ok(std::str::from_utf8(&bytes[..end])?.into())
}
fn tar_number(bytes: &[u8]) -> Result<u64> {
    ensure!(bytes.first().is_none_or(|byte| byte & 0x80 == 0), "pinned archive binary numeric fields unsupported");
    let text = std::str::from_utf8(bytes)?.trim_matches(|c| c == '\0' || c == ' ');
    if text.is_empty() {
        return Ok(0);
    }
    Ok(u64::from_str_radix(text, 8)?)
}
fn pax_fields(bytes: &[u8]) -> Result<BTreeMap<String, String>> {
    let mut fields = BTreeMap::new();
    let mut offset = 0usize;
    while offset < bytes.len() {
        check()?;
        let space =
            bytes[offset..].iter().position(|byte| *byte == b' ').context("pinned PAX length missing")? + offset;
        let length = std::str::from_utf8(&bytes[offset..space])?.parse::<usize>()?;
        let end = offset.checked_add(length).context("pinned PAX length overflow")?;
        ensure!(end <= bytes.len() && end > space + 2 && bytes[end - 1] == b'\n', "pinned PAX extent changed");
        let (key, value) =
            std::str::from_utf8(&bytes[space + 1..end - 1])?.split_once('=').context("pinned PAX value missing")?;
        ensure!(
            !key.is_empty() && fields.insert(key.to_owned(), value.to_owned()).is_none() && fields.len() <= 128,
            "pinned PAX fields repeated or excessive"
        );
        ensure!(
            [
                "path",
                "size",
                "linkpath",
                "mtime",
                "atime",
                "ctime",
                "uid",
                "gid",
                "uname",
                "gname",
                "comment",
                "charset",
                "LIBARCHIVE.creationtime"
            ]
            .contains(&key),
            "pinned PAX field changes unsupported archive semantics"
        );
        offset = end;
    }
    Ok(fields)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn fixture_workers_share_byte_path_and_deadline_limits() -> Result<()> {
        let bytes = until(now_unix_ms()? + 1000, 8, || {
            let shared = shared_scope()?;
            let results = std::thread::scope(|scope| {
                let a = scope.spawn(|| shared.enter(|| account(5)));
                let b = scope.spawn(|| shared.enter(|| account(5)));
                [a.join().unwrap(), b.join().unwrap()]
            });
            ensure!(results.iter().any(Result::is_err), "workers allocated independent byte budgets");
            ensure!(charged_bytes() == 10, "worker read charges did not reach the parent");
            Ok(())
        });
        assert!(bytes.unwrap_err().to_string().contains("byte budget"));

        until(now_unix_ms()? + 1000, 8, || {
            discovery_with_limit(3, || {
                let shared = shared_scope()?;
                let results = std::thread::scope(|scope| {
                    let a = scope.spawn(|| shared.enter(|| count(2)));
                    let b = scope.spawn(|| shared.enter(|| count(2)));
                    [a.join().unwrap(), b.join().unwrap()]
                });
                assert_eq!(results.iter().filter(|result| result.is_err()).count(), 1);
                assert!(count(0).is_err(), "worker path charges did not reach the parent");
                Ok(())
            })
        })?;

        let mut shared = until(now_unix_ms()? + 1000, 8, shared_scope)?;
        shared.0.deadline = Instant::now();
        let result = std::thread::scope(|scope| {
            scope.spawn(|| shared.enter(|| until(u64::MAX, 8, || account(1)))).join().unwrap()
        });
        assert!(result.unwrap_err().to_string().contains("deadline"));
        Ok(())
    }

    pub(crate) struct Directory(pub(crate) PathBuf);
    impl Directory {
        pub(crate) fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!(
                "sm-bounded-fixture-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    pub(crate) fn tar_header(name: &str, kind: u8, size: u64) -> [u8; 512] {
        assert!(name.len() <= 100);
        let mut header = [0u8; 512];
        header[..name.len()].copy_from_slice(name.as_bytes());
        for (offset, extent, value) in [(100, 8, 0o644), (108, 8, 0), (116, 8, 0), (124, 12, size), (136, 12, 0)] {
            let text = format!("{:0width$o}\0", value, width = extent - 1);
            header[offset..offset + extent].copy_from_slice(text.as_bytes());
        }
        header[148..156].fill(b' ');
        header[156] = kind;
        header[257..263].copy_from_slice(b"ustar\0");
        header[263..265].copy_from_slice(b"00");
        let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
        header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
        header
    }

    pub(crate) fn append_tar(bytes: &mut Vec<u8>, name: &str, kind: u8, data: &[u8]) {
        bytes.extend_from_slice(&tar_header(name, kind, data.len() as u64));
        bytes.extend_from_slice(data);
        bytes.resize(bytes.len().div_ceil(512) * 512, 0);
    }

    pub(crate) fn gzip_archive(path: &Path, tar: &[u8]) -> Result<String> {
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        encoder.write_all(tar)?;
        let packed = encoder.finish()?;
        fs::write(path, &packed)?;
        Ok(format!("{:x}", Sha256::digest(packed)))
    }

    fn regular_archive(directory: &Path, data: &[u8]) -> Result<(PathBuf, PathBuf, String, Vec<u8>)> {
        let source = directory.join("source");
        fs::create_dir(&source)?;
        fs::write(source.join("file"), data)?;
        let archive = directory.join("source.tar.gz");
        let mut tar = Vec::new();
        append_tar(&mut tar, "root/", b'5', b"");
        append_tar(&mut tar, "root/file", b'0', data);
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&archive, &tar)?;
        Ok((archive, source, hash, tar))
    }

    #[test]
    fn expired_deadline_and_nested_budget_refuse_before_output() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "sm-bounded-input-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root)?;
        let source = root.join("source");
        let output = root.join("output");
        fs::write(&source, b"12345")?;
        assert!(until(now_unix_ms()?.saturating_sub(1), 1024, || write(&output, b"never")).is_err());
        assert!(!output.exists());
        assert!(
            until(now_unix_ms()? + 1000, 8, || {
                read(&source)?;
                until(now_unix_ms()? + 10000, 1024, || read(&source))
            })
            .is_err()
        );
        assert!(read_limit(&source, 4).is_err());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn implicit_helpers_preserve_explicit_budget_and_nested_deadline() -> Result<()> {
        until(now_unix_ms()? + 1000, 2 * DEFAULT_BYTES, || {
            account(DEFAULT_BYTES)?;
            limited(|| account(1))
        })?;
        let error = until(now_unix_ms()? + 1000, 1024, || until(now_unix_ms()?.saturating_sub(1), 1024, || Ok(())))
            .unwrap_err();
        assert!(error.to_string().contains("deadline"));
        let _ = until(now_unix_ms()? + 1000, 1024, || {
            count(MAX_PATHS)?;
            count(1)
        })
        .unwrap_err();
        let error = until(now_unix_ms()? + 1000, 4, || {
            account(3)?;
            until(now_unix_ms()? + 10000, 2, || Ok(()))
        })
        .unwrap_err();
        assert!(error.to_string().contains("budget"));
        Ok(())
    }

    #[test]
    fn digest_reads_share_one_snapshot_and_budget_without_caching_files() -> Result<()> {
        let directory = Directory::new()?;
        let path = directory.0.join("state.wram");
        let mut previous_hash = String::new();
        for value in [17, 39] {
            let expected = vec![value; 2 * 64 * 1024 + 37];
            fs::write(&path, &expected)?;
            until(now_unix_ms()? + 10_000, expected.len() as u64, || {
                let (bytes, hash) = read_with_digest(&path)?;
                assert_eq!(bytes, expected);
                assert_eq!(hash, format!("{:x}", Sha256::digest(&expected)));
                assert_ne!(hash, previous_hash, "each call must authenticate the current file");
                assert_eq!(charged_bytes(), expected.len() as u64);
                previous_hash = hash;
                Ok(())
            })?;
            assert!(until(now_unix_ms()? + 10_000, expected.len() as u64 - 1, || read_with_digest(&path)).is_err());
        }
        Ok(())
    }

    #[test]
    fn retained_tree_passes_share_bytes_and_deadline_but_not_previous_discovery() -> Result<()> {
        until(now_unix_ms()? + 1000, 4, || {
            count(MAX_PATHS)?;
            for _ in 0..3 {
                discovery(|| {
                    count(MAX_PATHS - 1)?;
                    limited(|| count(1))?;
                    account(1)
                })?;
            }
            Ok(())
        })?;
        let error = until(now_unix_ms()? + 1000, 4, || {
            account(3)?;
            discovery(|| account(2))
        })
        .unwrap_err();
        assert!(error.to_string().contains("byte budget"));
        let error = until(now_unix_ms()? + 1000, 4, || {
            discovery(|| {
                count(MAX_PATHS)?;
                limited(|| count(1))
            })
        })
        .unwrap_err();
        assert!(error.to_string().contains("path budget"));
        let error =
            until(now_unix_ms()? + 1000, 4, || discovery(|| until(now_unix_ms()?.saturating_sub(1), 4, || Ok(()))))
                .unwrap_err();
        assert!(error.to_string().contains("deadline"));
        Ok(())
    }

    #[test]
    fn declared_discovery_limits_keep_global_ceiling_and_outer_resource_guards() -> Result<()> {
        assert!(discovery_with_limit(MAX_DECLARED_PATHS + 1, || Ok(())).is_err());
        until(now_unix_ms()? + 1000, 8, || discovery_with_limit(MAX_PATHS + 1, || limited(|| count(MAX_PATHS + 1))))?;
        let error = until(now_unix_ms()? + 1000, 8, || {
            account(7)?;
            discovery_with_limit(MAX_PATHS + 1, || account(2))
        })
        .unwrap_err();
        assert!(error.to_string().contains("byte budget"));
        let error = until(now_unix_ms()? + 1000, 8, || {
            discovery_with_limit(MAX_PATHS + 1, || until(now_unix_ms()?.saturating_sub(1), 8, || Ok(())))
        })
        .unwrap_err();
        assert!(error.to_string().contains("deadline"));
        assert!(until(now_unix_ms()? + 1000, 8, || count(MAX_PATHS + 1)).is_err());
        Ok(())
    }

    #[test]
    fn explicit_streaming_hash_preserves_the_default_file_limit() -> Result<()> {
        let directory = Directory::new()?;
        let source = directory.0.join("large-input");
        fs::File::create(&source)?.set_len(MAX_FILE_BYTES + 1)?;
        assert!(digest(&source).unwrap_err().to_string().contains("hash extent"));
        assert!(digest_limit(&source, 128 * 1024 * 1024 + 1).is_err());
        assert!(read(&source).is_err());
        assert_eq!(digest_limit(&source, 128 * 1024 * 1024)?.len(), 64);
        Ok(())
    }

    #[test]
    fn immutable_streamed_copy_preserves_budget_identity_and_replay() -> Result<()> {
        let directory = Directory::new()?;
        let source = directory.0.join("source");
        let destination = directory.0.join("retained");
        let expected = format!("{:x}", Sha256::digest(b"exact"));
        fs::write(&source, b"exact")?;
        let error =
            until(now_unix_ms()? + 1000, 9, || immutable_copy(&source, &destination, &expected, 1024)).unwrap_err();
        assert!(error.to_string().contains("budget"));
        assert!(!destination.exists() && !directory.0.join("retained.pending").exists());
        fs::write(directory.0.join("retained.pending"), b"par")?;
        immutable_copy(&source, &destination, &expected, 1024)?;
        let modified = fs::metadata(&destination)?.modified()?;
        immutable_copy(&source, &destination, &expected, 1024)?;
        assert_eq!(fs::metadata(&destination)?.modified()?, modified);
        assert_eq!(fs::read(&destination)?, b"exact");
        fs::write(&source, b"other")?;
        let failed = directory.0.join("failed");
        assert!(immutable_copy(&source, &failed, &expected, 1024).is_err());
        assert!(!failed.exists() && !directory.0.join("failed.pending").exists());
        fs::write(&destination, b"other")?;
        assert!(immutable_copy(&source, &destination, &expected, 1024).is_err());
        assert!(until(0, 1024, || immutable_copy(&source, &failed, &expected, 1024)).is_err());
        Ok(())
    }

    #[test]
    fn nonregular_files_and_path_aliases_refuse_without_blocking() -> Result<()> {
        let directory = Directory::new()?;
        let file = directory.0.join("file");
        fs::write(&file, b"same")?;
        let fifo = directory.0.join("fifo");
        let name = std::ffi::CString::new(fifo.as_os_str().as_bytes())?;
        ensure!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) } == 0, "FIFO fixture creation failed");
        let entered = Instant::now();
        assert!(read(&fifo).is_err());
        assert!(digest(&fifo).is_err());
        assert!(write(&fifo, b"refused").is_err());
        assert!(read(&directory.0).is_err());
        assert!(read(Path::new("/dev/null")).is_err());
        assert!(entered.elapsed() < Duration::from_secs(1));
        let link = directory.0.join("link");
        std::os::unix::fs::symlink(&file, &link)?;
        assert!(read(&link).is_err());
        assert!(digest(&link).is_err());
        assert!(safe(&directory.0.join("./file")).is_err());
        assert!(safe(&directory.0.join("../file")).is_err());
        assert_eq!(read(&file)?, b"same");
        Ok(())
    }

    #[test]
    fn tar_compares_exact_bytes_and_rejects_changed_source_archive_and_gzip_tail() -> Result<()> {
        let directory = Directory::new()?;
        let (archive, source, hash, _) = regular_archive(&directory.0, b"original")?;
        let files = authenticate_tar_source(&archive, &source, &hash)?;
        assert_eq!(files, BTreeMap::from([("file".into(), format!("{:x}", Sha256::digest(b"original")))]));
        fs::write(source.join("file"), b"tampered")?;
        assert!(authenticate_tar_source(&archive, &source, &hash).is_err());
        fs::write(source.join("file"), b"original")?;
        assert!(authenticate_tar_source(&archive, &source, &"f".repeat(64)).is_err());
        let mut packed = fs::read(&archive)?;
        packed.extend_from_slice(b"trailing gzip bytes");
        fs::write(&archive, &packed)?;
        assert!(authenticate_tar_source(&archive, &source, &format!("{:x}", Sha256::digest(packed))).is_err());
        Ok(())
    }

    #[test]
    fn tar_rejects_links_duplicate_unsafe_and_truncated_members() -> Result<()> {
        let directory = Directory::new()?;
        let source = directory.0.join("source");
        fs::create_dir(&source)?;
        fs::write(source.join("file"), b"exact")?;
        let archive = directory.0.join("archive.gz");
        for (name, kind, duplicate) in [
            ("root/file", b'1', false),
            ("root/file", b'2', false),
            ("root/file", b'6', false),
            ("root/file", b'0', true),
            ("root/../file", b'0', false),
            ("root/./file", b'0', false),
            ("root//file", b'0', false),
            ("/root/file", b'0', false),
        ] {
            let mut tar = Vec::new();
            append_tar(&mut tar, name, kind, b"exact");
            if duplicate {
                append_tar(&mut tar, name, kind, b"exact");
            }
            tar.extend_from_slice(&[0u8; 1024]);
            let hash = gzip_archive(&archive, &tar)?;
            assert!(authenticate_tar_source(&archive, &source, &hash).is_err(), "accepted {name}/{kind}");
        }
        let mut tar = tar_header("root/file", b'0', 5).to_vec();
        tar.extend_from_slice(b"exa");
        let hash = gzip_archive(&archive, &tar)?;
        assert!(authenticate_tar_source(&archive, &source, &hash).is_err());
        let hash = gzip_archive(&archive, &[0u8; 511])?;
        assert!(authenticate_tar_source(&archive, &source, &hash).is_err());
        Ok(())
    }

    #[test]
    fn tar_expansion_and_member_count_share_outer_limits() -> Result<()> {
        let directory = Directory::new()?;
        let (archive, source, hash, _) = regular_archive(&directory.0, &[0u8; 4096])?;
        let cap = archive.metadata()?.len() + 1024;
        let error =
            until(now_unix_ms()? + 1000, cap, || authenticate_tar_source(&archive, &source, &hash)).unwrap_err();
        assert!(error.to_string().contains("budget"));
        let mut tar = Vec::new();
        for number in 0..=MAX_ARCHIVE_MEMBERS {
            append_tar(&mut tar, &format!("root/directory-{number}/"), b'5', b"");
        }
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&archive, &tar)?;
        let error = authenticate_tar_source(&archive, &source, &hash).unwrap_err();
        assert!(error.to_string().contains("member count"));
        Ok(())
    }

    #[test]
    fn pax_path_size_and_metadata_parse_without_extraction() -> Result<()> {
        fn field(key: &str, value: &str) -> String {
            let payload = format!(" {key}={value}\n");
            let mut length = payload.len() + 1;
            loop {
                let next = payload.len() + length.to_string().len();
                if next == length {
                    return format!("{length}{payload}");
                }
                length = next;
            }
        }
        let directory = Directory::new()?;
        let (archive, source, _, _) = regular_archive(&directory.0, b"exact")?;
        let fields = format!("{}{}{}", field("path", "root/file"), field("size", "5"), field("mtime", "12.5"));
        let mut tar = Vec::new();
        append_tar(&mut tar, "pax", b'x', fields.as_bytes());
        append_tar(&mut tar, "ignored", b'0', b"exact");
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&archive, &tar)?;
        assert!(authenticate_tar_source(&archive, &source, &hash)?.contains_key("file"));
        let duplicate = format!("{}{}", field("path", "root/file"), field("path", "other/file"));
        assert!(pax_fields(duplicate.as_bytes()).is_err());
        assert!(pax_fields(field("GNU.sparse.map", "0,1").as_bytes()).is_err());
        assert!(pax_fields(b"999 path=outside\n").is_err());
        Ok(())
    }

    #[test]
    fn tree_refuses_deep_or_oversized_discovery() -> Result<()> {
        let directory = Directory::new()?;
        fs::write(directory.0.join("file"), b"12345")?;
        assert!(tree(&directory.0, 1, 4).is_err());
        assert_eq!(tree(&directory.0, 1, 5)?, (1, 5));
        let mut child = directory.0.clone();
        for _ in 0..34 {
            child.push("child");
            fs::create_dir(&child)?;
        }
        assert!(tree(&directory.0, 10, 1024).is_err());
        Ok(())
    }
}
