//! Trusted detached checkouts. Git and source writes remain supervisor-owned.
use crate::{executor::ExecutionPending, model::*, retention, rewrite};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

thread_local! {
    static OUTER_DEADLINE: Cell<Option<Instant>> = const { Cell::new(None) };
}

struct DeadlineScope(Option<Instant>);
impl Drop for DeadlineScope {
    fn drop(&mut self) {
        OUTER_DEADLINE.with(|current| current.set(self.0));
    }
}

fn deadline() -> Instant {
    OUTER_DEADLINE.with(|current| current.get().unwrap_or_else(|| Instant::now() + REVISION_TIMEOUT))
}

fn check_deadline() -> Result<()> {
    ensure!(Instant::now() < deadline(), "controller Git total deadline exhausted");
    Ok(())
}

fn with_deadline<T>(outer: Option<u64>, action: impl FnOnce() -> Result<T>) -> Result<T> {
    let entered = Instant::now();
    let mut cap = entered + REVISION_TIMEOUT;
    if let Some(outer) = outer {
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        cap = cap.min(entered + Duration::from_millis(outer.saturating_sub(now)).min(REVISION_TIMEOUT));
    }
    let previous = OUTER_DEADLINE.with(|current| {
        let previous = current.get();
        if let Some(inherited) = previous {
            cap = cap.min(inherited);
        }
        current.set(Some(cap));
        previous
    });
    let _scope = DeadlineScope(previous);
    check_deadline()?;
    let result = action();
    if result.is_ok() {
        check_deadline()?;
    }
    result
}

fn exact_commit(commit: &str) -> bool {
    commit.len() == 40 && commit.bytes().all(|b| b.is_ascii_hexdigit())
}
fn protected(path: &Path) -> Result<()> {
    retention::safe_path(path)
}
fn git_command(checkout: &Path, argv: &[&str]) -> Command {
    let mut command = Command::new("/usr/bin/git");
    command
        .args(["-c", "core.hooksPath=/dev/null", "-c", "commit.gpgsign=false", "-c", "protocol.file.allow=always"])
        .args(["-c", "core.fsmonitor=false"])
        .args(["-c", "gc.auto=0", "-c", "maintenance.auto=false"])
        .args(argv)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .env("GIT_AUTHOR_NAME", "Translation supervisor")
        .env("GIT_AUTHOR_EMAIL", "translation@localhost")
        .env("GIT_COMMITTER_NAME", "Translation supervisor")
        .env("GIT_COMMITTER_EMAIL", "translation@localhost")
        .env("GIT_AUTHOR_DATE", "2000-01-01T00:00:00Z")
        .env("GIT_COMMITTER_DATE", "2000-01-01T00:00:00Z")
        .stdin(std::process::Stdio::null())
        .current_dir(checkout);
    command
}
fn git_output(checkout: &Path, argv: &[&str]) -> Result<Output> {
    let result = git_bounded_output(checkout, argv, None, MAX_REVISION_RECORD_BYTES, None, deadline(), None)?;
    ensure!(result.status.success(), "supervisor Git operation failed: {}", String::from_utf8_lossy(&result.stderr));
    Ok(result)
}
fn git(checkout: &Path, argv: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git_output(checkout, argv)?.stdout)?.trim().into())
}
fn git_dir(checkout: &Path) -> Result<PathBuf> {
    protected(checkout)?;
    ensure!(
        checkout.join(".git").is_dir() && !checkout.join(".git").is_symlink(),
        "supervisor checkout must own isolated Git metadata"
    );
    let head = git_bounded_output(checkout, &["symbolic-ref", "-q", "HEAD"], None, 4096, None, deadline(), None)?;
    ensure!(head.status.code() == Some(1), "supervisor checkout must use detached source");
    Ok(checkout.join(".git"))
}
fn lock(checkout: &Path) -> Result<File> {
    let path = git_dir(checkout)?.join("translation.lock");
    controller_lock(&path)
}

/// Hold every controller Git writer lease while finalization observes or
/// retires a possibly incomplete checkout. This also covers an attached or
/// unborn HEAD left by interrupted creation; it launches no Git operation.
pub fn retirement_leases(checkout: &Path) -> Result<Vec<File>> {
    protected(checkout)?;
    let parent = checkout.parent().context("checkout parent missing")?;
    let key = sha256(checkout.as_os_str().as_encoded_bytes());
    let mut leases = Vec::new();
    for name in [format!(".translation-create-{key}.lock"), format!(".translation-restore-{key}.lock")] {
        let path = parent.join(name);
        if path.exists() {
            leases.push(controller_lock(&path)?);
        }
    }
    let metadata = checkout.join(".git");
    if metadata.exists() {
        protected(&metadata)?;
        ensure!(metadata.is_dir(), "owned checkout Git metadata is not a directory");
        let path = metadata.join("translation.lock");
        if path.exists() {
            leases.push(controller_lock(&path)?);
        }
    }
    Ok(leases)
}
fn names(checkout: &Path, argv: &[&str]) -> Result<Vec<String>> {
    let output = git_output(checkout, argv)?;
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| Ok(std::str::from_utf8(name)?.to_owned()))
        .collect()
}
fn dirty(checkout: &Path) -> Result<BTreeSet<String>> {
    let mut paths = names(checkout, &["diff", "--name-only", "--no-renames", "-z", "--"])?;
    paths.extend(names(checkout, &["diff", "--cached", "--name-only", "--no-renames", "-z", "--"])?);
    paths.extend(names(checkout, &["ls-files", "--others", "--exclude-standard", "-z"])?);
    Ok(paths.into_iter().collect())
}
fn checked_file(checkout: &Path, path: &str) -> Result<Vec<u8>> {
    check_deadline()?;
    ensure!(relative_file(path), "source path is not root-relative");
    let target = checkout.join(path);
    protected(&target)?;
    let metadata = fs::metadata(&target)?;
    ensure!(metadata.is_file(), "source snapshot requires regular files");
    use std::os::unix::fs::MetadataExt;
    ensure!(metadata.nlink() == 1, "source snapshot cannot contain hardlink aliases");
    read_source(&target, metadata.len())
}

fn read_source(path: &Path, maximum: u64) -> Result<Vec<u8>> {
    check_deadline()?;
    protected(path)?;
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let expected = file.metadata()?;
    ensure!(expected.is_file() && expected.len() <= maximum, "source file extent changed");
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check_deadline()?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        ensure!((bytes.len() as u64).saturating_add(count as u64) <= maximum, "source file extent grew during import");
        bytes.extend_from_slice(&buffer[..count]);
    }
    let observed = file.metadata()?;
    ensure!(
        bytes.len() as u64 == expected.len()
            && observed.len() == expected.len()
            && observed.mtime() == expected.mtime()
            && observed.mtime_nsec() == expected.mtime_nsec()
            && observed.ctime() == expected.ctime()
            && observed.ctime_nsec() == expected.ctime_nsec(),
        "source file changed during import"
    );
    check_deadline()?;
    Ok(bytes)
}

fn seal_source(path: &Path, maximum: u64) -> Result<retention::Reference> {
    check_deadline()?;
    protected(path)?;
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && before.len() <= maximum, "retained source extent exceeded or nonregular file");
    let mut hash = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check_deadline()?;
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).context("source seal extent overflow")?;
        ensure!(count <= maximum, "source exceeded budget while sealing");
        hash.update(&buffer[..n]);
    }
    let after = file.metadata()?;
    ensure!(
        count == before.len()
            && after.len() == before.len()
            && after.mtime() == before.mtime()
            && after.mtime_nsec() == before.mtime_nsec()
            && after.ctime() == before.ctime()
            && after.ctime_nsec() == before.ctime_nsec(),
        "source changed while sealing"
    );
    check_deadline()?;
    Ok(retention::Reference { path: path.to_path_buf(), sha256: format!("{:x}", hash.finalize()), bytes: count })
}

fn authenticate_reference(reference: &retention::Reference, maximum: u64) -> Result<()> {
    ensure!(digest_valid(&reference.sha256) && reference.bytes <= maximum, "invalid source retention reference");
    ensure!(seal_source(&reference.path, reference.bytes)? == *reference, "retained source content seal changed");
    Ok(())
}

fn write_retained(path: &Path, bytes: &[u8], maximum: u64) -> Result<()> {
    check_deadline()?;
    protected(path)?;
    ensure!(bytes.len() as u64 <= maximum, "source record byte budget exceeded");
    let parent = path.parent().context("source record parent missing")?;
    retention::directory(parent)?;
    if path.try_exists()? {
        ensure!(read_source(path, maximum)? == bytes, "immutable source record changed");
        return Ok(());
    }
    let pending = path.with_extension("pending");
    protected(&pending)?;
    if pending.try_exists()? {
        let metadata = pending.symlink_metadata()?;
        ensure!(metadata.is_file() && metadata.len() <= maximum, "source pending extent changed");
        check_deadline()?;
        fs::remove_file(&pending)?;
    }
    let mut file =
        OpenOptions::new().create_new(true).write(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    for block in bytes.chunks(64 * 1024) {
        check_deadline()?;
        file.write_all(block)?;
    }
    file.sync_all()?;
    file.set_permissions(fs::Permissions::from_mode(0o400))?;
    check_deadline()?;
    fs::rename(&pending, path)?;
    File::open(parent)?.sync_all()?;
    check_deadline()
}

fn same_pending(path: &Path, expected: &[u8]) -> Result<()> {
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_file()
            && metadata.nlink() == 1
            && metadata.uid() == unsafe { libc::geteuid() }
            && metadata.mode() & 0o022 == 0
            && metadata.len() <= expected.len() as u64,
        "source publication has foreign pending metadata"
    );
    let bytes = read_source(path, expected.len() as u64)?;
    ensure!(expected.starts_with(&bytes), "source publication pending metadata changed");
    Ok(())
}

const MAX_REVISION_OBJECTS: usize = 1024;
const MAX_REVISION_PATHS: usize = 1024;
const MAX_REVISION_RECORD_BYTES: u64 = 4 * 1024 * 1024;
const REVISION_TIMEOUT: Duration = Duration::from_secs(120);

fn controller_lock(path: &Path) -> Result<File> {
    check_deadline()?;
    protected(path)?;
    check_deadline()?;
    retention::directory(path.parent().context("controller lock parent missing")?)?;
    check_deadline()?;
    let file = OpenOptions::new().create(true).append(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(path)?;
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file() && metadata.uid() == unsafe { libc::geteuid() } && metadata.nlink() == 1,
        "controller lock has foreign ownership or aliases"
    );
    if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(ExecutionPending { reason: "controller Git operation has a surviving owner".into() }.into());
    }
    Ok(file)
}

fn bounded_text(value: &str, maximum: usize) -> &str {
    let mut end = value.len().min(maximum);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

/// Every mutating Git child inherits the controller lease. A dead controller
/// cannot leave a writer racing a retry; bounded pipe readers and one deadline
/// also cover raw object extraction and import.
fn git_bounded(
    checkout: &Path,
    argv: &[&str],
    input: Option<&[u8]>,
    maximum: u64,
    lease: &File,
    deadline: Instant,
) -> Result<Vec<u8>> {
    let result = git_bounded_output(checkout, argv, input, maximum, Some(lease), deadline, None)?;
    ensure!(
        result.status.success(),
        "controller Git operation failed: {}",
        bounded_text(&String::from_utf8_lossy(&result.stderr), 2048)
    );
    Ok(result.stdout)
}

fn git_bounded_output(
    checkout: &Path,
    argv: &[&str],
    input: Option<&[u8]>,
    maximum: u64,
    lease: Option<&File>,
    deadline: Instant,
    index: Option<&Path>,
) -> Result<Output> {
    use std::os::unix::process::CommandExt;
    let deadline = OUTER_DEADLINE.with(|current| current.get().map_or(deadline, |outer| outer.min(deadline)));
    ensure!(Instant::now() < deadline, "controller Git total deadline exhausted");
    let mut command = git_command(checkout, argv);
    if let Some(index) = index {
        protected(index)?;
        command.env("GIT_INDEX_FILE", index);
    }
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut entered: libc::timespec = unsafe { std::mem::zeroed() };
    ensure!(
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut entered) } == 0,
        "controller cannot read the Git child deadline clock"
    );
    let remaining = deadline.saturating_duration_since(Instant::now());
    let child_deadline = (entered.tv_sec as u128)
        .saturating_mul(1_000_000_000)
        .saturating_add(entered.tv_nsec as u128)
        .saturating_add(remaining.as_nanos());
    let fd = lease.map(AsRawFd::as_raw_fd);
    unsafe {
        command.pre_exec(move || {
            if let Some(fd) = fd {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let mut now: libc::timespec = std::mem::zeroed();
            if libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut now) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            let now = (now.tv_sec as u128).saturating_mul(1_000_000_000).saturating_add(now.tv_nsec as u128);
            if now >= child_deadline {
                return Err(std::io::Error::from_raw_os_error(libc::ETIMEDOUT));
            }
            Ok(())
        });
    }
    ensure!(Instant::now() < deadline, "controller Git total deadline exhausted before launch");
    let mut child = command.spawn()?;
    drop(command);
    let stdout = child.stdout.take().context("controller Git stdout missing")?;
    let stderr = child.stderr.take().context("controller Git stderr missing")?;
    let stdin = child.stdin.take();
    let exceeded = Arc::new(AtomicBool::new(false));
    let drain = |mut stream: Box<dyn Read + Send>, maximum: u64, exceeded: Arc<AtomicBool>| -> Result<Vec<u8>> {
        let mut bytes = Vec::new();
        stream.by_ref().take(maximum.saturating_add(1)).read_to_end(&mut bytes)?;
        if bytes.len() as u64 > maximum {
            exceeded.store(true, Ordering::SeqCst);
            bail!("controller Git output budget exceeded");
        }
        Ok(bytes)
    };
    std::thread::scope(|scope| {
        let stdout = scope.spawn(|| drain(Box::new(stdout), maximum, exceeded.clone()));
        let stderr = scope.spawn(|| drain(Box::new(stderr), 4096, exceeded.clone()));
        let stdin = scope.spawn(move || -> Result<()> {
            if let (Some(mut stream), Some(bytes)) = (stdin, input) {
                stream.write_all(bytes)?;
            }
            Ok(())
        });
        let status = loop {
            if exceeded.load(Ordering::SeqCst) || Instant::now() >= deadline {
                unsafe {
                    libc::kill(-(child.id() as i32), libc::SIGKILL);
                }
                let _ = child.wait();
                break Err(anyhow::anyhow!("controller Git output budget or total deadline exhausted"));
            }
            match child.try_wait() {
                Ok(Some(status)) => break Ok(status),
                Ok(None) => std::thread::sleep(Duration::from_millis(5)),
                Err(error) => {
                    unsafe {
                        libc::kill(-(child.id() as i32), libc::SIGKILL);
                    }
                    let _ = child.wait();
                    break Err(error.into());
                }
            }
        };
        // No trusted Git helper may keep a pipe or writer lease alive after its
        // observed leader exits. Automatic maintenance and hooks are disabled.
        unsafe {
            libc::kill(-(child.id() as i32), libc::SIGKILL);
        }
        let stdout = stdout.join().map_err(|_| anyhow::anyhow!("controller Git stdout collector panicked"))?;
        let stderr = stderr.join().map_err(|_| anyhow::anyhow!("controller Git stderr collector panicked"))?;
        let stdin = stdin.join().map_err(|_| anyhow::anyhow!("controller Git stdin writer panicked"))?;
        let status = status?;
        stdin?;
        Ok(Output { status, stdout: stdout?, stderr: stderr? })
    })
}

fn git_leased(checkout: &Path, argv: &[&str], lease: &File, deadline: Instant) -> Result<String> {
    Ok(String::from_utf8(git_bounded(checkout, argv, None, MAX_REVISION_RECORD_BYTES, lease, deadline)?)?.trim().into())
}

/// Measure the exact baseline working tree before allocating a clone. Git
/// output, file count, modes and logical source bytes are bounded as observed;
/// this read-only operation does not touch the source repository's index.
pub fn checkout_bytes(repository: &Path, baseline: &str, max_bytes: u64) -> Result<u64> {
    with_deadline(None, || checkout_bytes_inner(repository, baseline, max_bytes))
}

pub fn checkout_bytes_until(repository: &Path, baseline: &str, max_bytes: u64, deadline_unix_ms: u64) -> Result<u64> {
    with_deadline(Some(deadline_unix_ms), || checkout_bytes_inner(repository, baseline, max_bytes))
}

/// Exact NUL-delimited source-path queries for trusted affected-family
/// planning. No caller-selected Git command or mutating flag is admitted.
pub fn read_only_paths_until(
    repository: &Path,
    argv: &[&str],
    max_bytes: u64,
    deadline_unix_ms: u64,
) -> Result<Vec<u8>> {
    with_deadline(Some(deadline_unix_ms), || {
        protected(repository)?;
        ensure!(repository.is_dir() && max_bytes > 0, "path query requires a trusted repository and byte cap");
        let revision = |value: &str| {
            !value.is_empty()
                && value.len() <= 256
                && !value.starts_with('-')
                && !value.bytes().any(|byte| byte == 0 || byte == b'\n' || byte == b'\r')
        };
        let safe_diff = argv.len() == 7
            && argv[..5] == ["diff", "--name-only", "--no-renames", "-z", "--no-ext-diff"]
            && revision(argv[5])
            && argv[6] == "--";
        let legacy_diff = argv.len() == 6
            && argv[..4] == ["diff", "--name-only", "--no-renames", "-z"]
            && revision(argv[4])
            && argv[5] == "--";
        ensure!(
            safe_diff
                || legacy_diff
                || argv == ["ls-files", "--others", "--exclude-standard", "-z"]
                || argv == ["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
            "read-only source-path query is outside the admitted planning commands"
        );
        let mut command = argv.to_vec();
        if legacy_diff {
            command.insert(1, "--no-ext-diff");
        }
        let output = git_bounded_output(
            repository,
            &command,
            None,
            max_bytes.min(MAX_REVISION_RECORD_BYTES),
            None,
            deadline(),
            None,
        )?;
        ensure!(
            output.status.success(),
            "read-only source-path query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(output.stdout)
    })
}

fn checkout_bytes_inner(repository: &Path, baseline: &str, max_bytes: u64) -> Result<u64> {
    protected(repository)?;
    ensure!(
        repository.is_dir() && exact_commit(baseline) && max_bytes > 0,
        "checkout measurement needs exact trusted source and positive budget"
    );
    let lease = File::open(repository)?;
    let deadline = deadline();
    let listed = git_bounded(
        repository,
        &["ls-tree", "-r", "-l", "-z", baseline],
        None,
        MAX_REVISION_RECORD_BYTES,
        &lease,
        deadline,
    )?;
    let mut bytes = 0u64;
    let mut names = BTreeSet::new();
    for entry in listed.split(|byte| *byte == 0).filter(|entry| !entry.is_empty()) {
        check_deadline()?;
        let split = entry.iter().position(|byte| *byte == b'\t').context("checkout tree entry lacks a path")?;
        let fields = std::str::from_utf8(&entry[..split])?.split_whitespace().collect::<Vec<_>>();
        let path = std::str::from_utf8(&entry[split + 1..])?;
        ensure!(
            fields.len() == 4
                && ["100644", "100755"].contains(&fields[0])
                && fields[1] == "blob"
                && exact_commit(fields[2])
                && path.len() <= 4096
                && relative_file(path)
                && !Path::new(path).starts_with(".git")
                && names.insert(path.to_owned()),
            "checkout baseline contains unsafe source modes or paths"
        );
        ensure!(names.len() <= 65536, "checkout baseline file count exceeded");
        bytes = bytes.checked_add(fields[3].parse::<u64>()?).context("checkout tree byte extent overflow")?;
        ensure!(bytes <= max_bytes, "checkout baseline aggregate byte budget exceeded before clone");
    }
    Ok(bytes)
}

/// Create one detached controller checkout at its exact baseline. This does not
/// register a worktree or change the shared checkout's source, index or HEAD.
pub fn create(source_repository: &Path, destination: &Path, baseline: &str) -> Result<()> {
    with_deadline(None, || create_inner(source_repository, destination, baseline))
}

pub fn create_until(source_repository: &Path, destination: &Path, baseline: &str, deadline_unix_ms: u64) -> Result<()> {
    with_deadline(Some(deadline_unix_ms), || create_inner(source_repository, destination, baseline))
}

fn create_inner(source_repository: &Path, destination: &Path, baseline: &str) -> Result<()> {
    protected(source_repository)?;
    protected(destination)?;
    ensure!(exact_commit(baseline), "controller baseline must be exact");
    let parent = destination.parent().context("checkout destination parent missing")?;
    check_deadline()?;
    fs::create_dir_all(parent)?;
    let deadline = deadline();
    let key = sha256(destination.as_os_str().as_encoded_bytes());
    let lease = controller_lock(&parent.join(format!(".translation-create-{key}.lock")))?;
    let intent_path = parent.join(format!(".translation-create-{key}.json"));
    let identity = serde_json::to_vec(&(source_repository, destination, baseline))?;
    let owned = intent_path.exists();
    if owned {
        ensure!(fs::read(&intent_path)? == identity, "checkout creation intent differs");
    } else {
        ensure!(!destination.exists(), "controller destination already exists without creation ownership");
        atomic(&intent_path, &identity)?;
    }
    if destination.join(".git").is_dir() {
        if git_dir(destination).is_ok() {
            ensure!(
                git(destination, &["rev-parse", "HEAD"])? == baseline && dirty(destination)?.is_empty(),
                "existing prepared checkout differs from baseline"
            );
            return Ok(());
        }
        // A clone completed before detached checkout. Its working tree has no
        // tracked changes and every existing tracked file is baseline content.
        fn original_only(root: &Path, path: &Path, baseline: &str) -> Result<()> {
            protected(path)?;
            for entry in fs::read_dir(path)? {
                check_deadline()?;
                let entry = entry?;
                if entry.path() == root.join(".git") {
                    continue;
                }
                if entry.path().is_dir() {
                    original_only(root, &entry.path(), baseline)?;
                } else {
                    let relative =
                        entry.path().strip_prefix(root)?.to_str().context("clone source path UTF8")?.to_owned();
                    let original = git_output(root, &["show", &format!("{baseline}:{relative}")])?.stdout;
                    ensure!(checked_file(root, &relative)? == original, "incomplete clone contains unexpected source");
                }
            }
            Ok(())
        }
        original_only(destination, destination, baseline)?;
        git_leased(destination, &["checkout", "--detach", baseline], &lease, deadline)?;
        ensure!(dirty(destination)?.is_empty(), "recovered prepared source is dirty");
        return Ok(());
    }
    if destination.exists() {
        ensure!(
            owned && destination.is_dir() && fs::read_dir(destination)?.next().is_none(),
            "interrupted clone contains unreconciled files"
        );
        check_deadline()?;
        fs::remove_dir(destination)?;
    }
    git_leased(
        parent,
        &[
            "clone",
            "--shared",
            "--no-checkout",
            source_repository.to_str().context("source path UTF8")?,
            destination.to_str().context("destination path UTF8")?,
        ],
        &lease,
        deadline,
    )?;
    git_leased(destination, &["checkout", "--detach", baseline], &lease, deadline)?;
    ensure!(dirty(destination)?.is_empty(), "new prepared source is dirty");
    Ok(())
}

pub fn snapshot(checkout: &Path, approved_paths: &[String]) -> Result<SourceSnapshot> {
    with_deadline(None, || snapshot_inner(checkout, approved_paths))
}

pub fn snapshot_until(checkout: &Path, approved_paths: &[String], deadline_unix_ms: u64) -> Result<SourceSnapshot> {
    with_deadline(Some(deadline_unix_ms), || snapshot_inner(checkout, approved_paths))
}

fn snapshot_inner(checkout: &Path, approved_paths: &[String]) -> Result<SourceSnapshot> {
    git_dir(checkout)?;
    ensure!(dirty(checkout)?.is_empty(), "controller source must be clean");
    let commit = git(checkout, &["rev-parse", "HEAD"])?;
    ensure!(exact_commit(&commit), "controller source commit is inexact");
    let tree = git(checkout, &["rev-parse", "HEAD^{tree}"])?;
    let tracked = names(checkout, &["ls-files", "-z"])?;
    let mut full = BTreeMap::new();
    for path in tracked {
        full.insert(path.clone(), sha256(checked_file(checkout, &path)?));
    }
    let mut files_sha256 = BTreeMap::new();
    for path in approved_paths {
        ensure!(!files_sha256.contains_key(path), "duplicate approved file");
        files_sha256.insert(path.clone(), full.get(path).context("approved source is not tracked")?.clone());
    }
    ensure!(!files_sha256.is_empty(), "approved source snapshot is empty");
    Ok(SourceSnapshot { sha256: sealed(&(&commit, &tree, &full))?, commit, files_sha256, clean: true })
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourceIntent {
    operation_key: String,
    base_commit: String,
    base_source_sha256: String,
    expected_files: BTreeMap<String, String>,
    original_files: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imported_submission: Option<ImportedSubmission>,
}

fn atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    check_deadline()?;
    protected(path)?;
    let pending = path.with_extension("pending");
    protected(&pending)?;
    if pending.exists() {
        fs::remove_file(&pending)?;
    }
    let mut file = OpenOptions::new().create_new(true).write(true).open(&pending)?;
    for block in bytes.chunks(64 * 1024) {
        check_deadline()?;
        file.write_all(block)?;
    }
    file.sync_all()?;
    check_deadline()?;
    fs::rename(pending, path)?;
    File::open(path.parent().context("atomic parent missing")?)?.sync_all()?;
    Ok(())
}
fn source_intent(checkout: &Path, intent: &SourceIntent) -> Result<()> {
    let path = git_dir(checkout)?.join(format!("translation-{}.json", intent.operation_key));
    if path.exists() {
        let saved: SourceIntent = serde_json::from_slice(&fs::read(path)?)?;
        ensure!(saved == *intent, "source operation retry differs from durable intent");
    } else {
        atomic(&path, &serde_json::to_vec(intent)?)?;
    }
    Ok(())
}
fn recorded_intent(checkout: &Path, operation_key: &str) -> Result<Option<SourceIntent>> {
    let path = git_dir(checkout)?.join(format!("translation-{operation_key}.json"));
    protected(&path)?;
    if path.exists() { Ok(Some(serde_json::from_slice(&fs::read(path)?)?)) } else { Ok(None) }
}
fn keyed_commit(checkout: &Path, key: &str) -> Result<bool> {
    Ok(git(checkout, &["show", "-s", "--format=%B", "HEAD"])?
        .lines()
        .any(|line| line == format!("Translation-Operation: {key}")))
}
fn commit(checkout: &Path, intent: &SourceIntent, lease: &File, deadline: Instant) -> Result<()> {
    let paths: Vec<_> = intent.expected_files.keys().map(String::as_str).collect();
    let mut argv = vec!["add", "--"];
    argv.extend(paths);
    git_leased(checkout, &argv, lease, deadline)?;
    let message = format!("Apply Translation scope source\n\nTranslation-Operation: {}", intent.operation_key);
    git_leased(checkout, &["commit", "--no-verify", "-m", &message], lease, deadline)?;
    Ok(())
}
fn intended_dirty(checkout: &Path, intent: &SourceIntent) -> Result<()> {
    ensure!(git(checkout, &["rev-parse", "HEAD"])? == intent.base_commit, "source operation HEAD changed");
    for path in dirty(checkout)? {
        ensure!(intent.expected_files.contains_key(&path), "source operation includes foreign dirty path");
    }
    for (path, expected) in &intent.expected_files {
        let current = sha256(checked_file(checkout, path)?);
        ensure!(
            current == *expected || intent.original_files.get(path) == Some(&current),
            "source is neither original nor intended replacement"
        );
    }
    Ok(())
}
fn completed_source(checkout: &Path, intent: &SourceIntent, paths: &[String]) -> Result<SourceSnapshot> {
    ensure!(keyed_commit(checkout, &intent.operation_key)?, "source commit operation trailer differs");
    ensure!(git(checkout, &["rev-parse", "HEAD^"])? == intent.base_commit, "source commit baseline differs");
    let source = snapshot(checkout, paths)?;
    ensure!(source.files_sha256 == intent.expected_files, "committed replacements differ");
    let changed = names(checkout, &["diff", "--name-only", "--no-renames", "-z", &intent.base_commit, "HEAD", "--"])?;
    ensure!(changed.iter().all(|path| intent.expected_files.contains_key(path)), "source commit changed foreign files");
    Ok(source)
}

/// Apply/recover source replacements and make one deterministic scoped commit.
pub fn apply_commit(
    checkout: &Path,
    context: &OperationContext,
    capsule: &RewriteCapsule,
    replacements: &SourceReplacements,
    limits: RewriteLimits,
) -> Result<SourceSnapshot> {
    with_deadline(None, || apply_commit_inner(checkout, context, capsule, replacements, limits))
}

pub fn apply_commit_until(
    checkout: &Path,
    context: &OperationContext,
    capsule: &RewriteCapsule,
    replacements: &SourceReplacements,
    limits: RewriteLimits,
    deadline_unix_ms: u64,
) -> Result<SourceSnapshot> {
    with_deadline(Some(deadline_unix_ms), || apply_commit_inner(checkout, context, capsule, replacements, limits))
}

fn apply_commit_inner(
    checkout: &Path,
    context: &OperationContext,
    capsule: &RewriteCapsule,
    replacements: &SourceReplacements,
    limits: RewriteLimits,
) -> Result<SourceSnapshot> {
    let deadline = deadline();
    let lease = lock(checkout)?;
    ensure!(digest_valid(&context.operation_key), "source operation key must be sealed");
    ensure!(
        capsule.run_id == context.run_id
            && capsule.attempt == context.attempt
            && capsule.policy_sha256 == context.policy_sha256,
        "source operation differs from rewrite attempt identity"
    );
    rewrite::decode(capsule, &serde_json::to_vec(replacements)?, limits)?;
    let paths: Vec<_> = capsule.files.iter().map(|file| file.path.clone()).collect();
    let mut expected: BTreeMap<_, _> =
        capsule.files.iter().map(|file| (file.path.clone(), file.sha256.clone())).collect();
    for file in &replacements.files {
        expected.insert(file.path.clone(), sha256(file.content.as_bytes()));
    }
    let intent = if let Some(saved) = recorded_intent(checkout, &context.operation_key)? {
        ensure!(
            saved.expected_files == expected
                && saved.base_source_sha256 == capsule.source_sha256
                && saved.imported_submission.is_none(),
            "replacement retry differs"
        );
        saved
    } else {
        let before = snapshot(checkout, &paths)?;
        ensure!(
            before.sha256 == capsule.source_sha256
                && before.files_sha256
                    == capsule.files.iter().map(|file| (file.path.clone(), file.sha256.clone())).collect(),
            "capsule source changed before application"
        );
        let intent = SourceIntent {
            operation_key: context.operation_key.clone(),
            base_commit: before.commit,
            base_source_sha256: before.sha256,
            expected_files: expected,
            original_files: before.files_sha256,
            imported_submission: None,
        };
        source_intent(checkout, &intent)?;
        intent
    };
    if keyed_commit(checkout, &intent.operation_key)? {
        return completed_source(checkout, &intent, &paths);
    }
    intended_dirty(checkout, &intent)?;
    for replacement in &replacements.files {
        let current = sha256(checked_file(checkout, &replacement.path)?);
        if current == sha256(replacement.content.as_bytes()) {
            continue;
        }
        ensure!(current == replacement.base_sha256, "source changed before replacement recovery");
        atomic(&checkout.join(&replacement.path), replacement.content.as_bytes())?;
    }
    commit(checkout, &intent, &lease, deadline)?;
    completed_source(checkout, &intent, &paths)
}

/// Copy only the verified owned files into a distinct controller checkout. Its
/// expected HEAD is checked before writes; an exact completed retry is reused.
pub fn integrate(
    candidate_checkout: &Path,
    integration_checkout: &Path,
    context: &OperationContext,
    candidate: &SourceSnapshot,
    expected_head: &str,
    approved_paths: &[String],
) -> Result<SourceSnapshot> {
    with_deadline(None, || {
        integrate_inner(
            candidate_checkout,
            integration_checkout,
            context,
            candidate,
            expected_head,
            approved_paths,
            None,
        )
    })
}

pub fn integrate_until(
    candidate_checkout: &Path,
    integration_checkout: &Path,
    context: &OperationContext,
    candidate: &SourceSnapshot,
    expected_head: &str,
    approved_paths: &[String],
    deadline_unix_ms: u64,
) -> Result<SourceSnapshot> {
    with_deadline(Some(deadline_unix_ms), || {
        integrate_inner(
            candidate_checkout,
            integration_checkout,
            context,
            candidate,
            expected_head,
            approved_paths,
            None,
        )
    })
}

/// Integrate an authenticated existing submission. A frozen baseline already
/// containing its exact owned files is retained without inventing a new commit.
/// The caller must authenticate the admission against its frozen installation;
/// this layer binds that admission and source to the durable Git operation.
pub fn integrate_submitted(
    candidate_checkout: &Path,
    integration_checkout: &Path,
    context: &OperationContext,
    imported: &ImportedSubmission,
    expected_head: &str,
    approved_paths: &[String],
) -> Result<SourceSnapshot> {
    with_deadline(None, || {
        integrate_inner(
            candidate_checkout,
            integration_checkout,
            context,
            &imported.source,
            expected_head,
            approved_paths,
            Some(imported),
        )
    })
}

pub fn integrate_submitted_until(
    candidate_checkout: &Path,
    integration_checkout: &Path,
    context: &OperationContext,
    imported: &ImportedSubmission,
    expected_head: &str,
    approved_paths: &[String],
    deadline_unix_ms: u64,
) -> Result<SourceSnapshot> {
    with_deadline(Some(deadline_unix_ms), || {
        integrate_inner(
            candidate_checkout,
            integration_checkout,
            context,
            &imported.source,
            expected_head,
            approved_paths,
            Some(imported),
        )
    })
}

fn integrate_inner(
    candidate_checkout: &Path,
    integration_checkout: &Path,
    context: &OperationContext,
    candidate: &SourceSnapshot,
    expected_head: &str,
    approved_paths: &[String],
    imported: Option<&ImportedSubmission>,
) -> Result<SourceSnapshot> {
    ensure!(
        candidate_checkout != integration_checkout
            && exact_commit(expected_head)
            && digest_valid(&context.operation_key),
        "integration needs distinct checkouts and exact identities"
    );
    ensure!(
        imported.is_none_or(|imported| digest_valid(&imported.admission_sha256) && imported.source == *candidate),
        "submitted integration requires its exact sealed admission and source"
    );
    let candidate_now = snapshot(candidate_checkout, approved_paths)?;
    ensure!(candidate_now == *candidate, "candidate source changed before integration");
    let deadline = deadline();
    let lease = lock(integration_checkout)?;
    let original = if let Some(saved) = recorded_intent(integration_checkout, &context.operation_key)? {
        ensure!(
            saved.base_commit == expected_head
                && saved.expected_files == candidate.files_sha256
                && saved.imported_submission.as_ref() == imported,
            "integration retry source differs"
        );
        saved
    } else {
        let before = snapshot(integration_checkout, approved_paths)?;
        ensure!(before.commit == expected_head, "integration source HEAD differs from expected clean freeze");
        let intent = SourceIntent {
            operation_key: context.operation_key.clone(),
            base_commit: before.commit,
            base_source_sha256: before.sha256,
            expected_files: candidate.files_sha256.clone(),
            original_files: before.files_sha256,
            imported_submission: imported.cloned(),
        };
        source_intent(integration_checkout, &intent)?;
        intent
    };
    if keyed_commit(integration_checkout, &context.operation_key)? {
        return completed_source(integration_checkout, &original, approved_paths);
    }
    if imported.is_some() && original.original_files == original.expected_files {
        let source = snapshot(integration_checkout, approved_paths)?;
        ensure!(
            source.commit == original.base_commit
                && source.sha256 == original.base_source_sha256
                && source.files_sha256 == original.expected_files,
            "already integrated submission differs from its exact frozen baseline"
        );
        return Ok(source);
    }
    intended_dirty(integration_checkout, &original)?;
    for (path, expected) in &original.expected_files {
        let bytes = checked_file(candidate_checkout, path)?;
        ensure!(sha256(&bytes) == *expected, "candidate file changed during integration");
        if sha256(checked_file(integration_checkout, path)?) != *expected {
            atomic(&integration_checkout.join(path), &bytes)?;
        }
    }
    ensure!(!dirty(integration_checkout)?.is_empty(), "integration contains no changed owned source");
    commit(integration_checkout, &original, &lease, deadline)?;
    completed_source(integration_checkout, &original, approved_paths)
}

/// Stable protected publication. An identical retry authenticates original bytes.
pub fn publish_completion(path: &Path, context: &OperationContext, completion: &Completion) -> Result<Publication> {
    protected(path)?;
    ensure!(
        context.run_id == completion.run_id
            && context.attempt == completion.attempt
            && context.policy_sha256 == completion.policy_sha256,
        "completion publication lineage differs"
    );
    let bytes = serde_json::to_vec(completion)?;
    if path.exists() {
        ensure!(fs::read(path)? == bytes, "completion publication retry changed accepted result")
    } else {
        fs::create_dir_all(path.parent().context("publication parent missing")?)?;
        atomic(path, &bytes)?;
    }
    Ok(Publication {
        seal: OutcomeSeal {
            operation_key: context.operation_key.clone(),
            policy_sha256: context.policy_sha256.clone(),
            source_sha256: completion.integrated.sha256.clone(),
            evidence_sha256: sha256(&bytes),
        },
        observation_ids: vec![format!("{}-integrated", context.operation_key)],
        retained_bytes: bytes.len() as u64,
    })
}

/// Preserve exact source paths before an implementer checkout is retired. Caller
/// authenticates evidence against these retained snapshots before deletion.
pub fn retain_source(checkout: &Path, retained: &Path, paths: &[String]) -> Result<BTreeMap<String, String>> {
    with_deadline(None, || retain_source_inner(checkout, retained, paths))
}

pub fn retain_source_until(
    checkout: &Path,
    retained: &Path,
    paths: &[String],
    deadline_unix_ms: u64,
) -> Result<BTreeMap<String, String>> {
    with_deadline(Some(deadline_unix_ms), || retain_source_inner(checkout, retained, paths))
}

fn retain_source_inner(checkout: &Path, retained: &Path, paths: &[String]) -> Result<BTreeMap<String, String>> {
    protected(retained)?;
    let source = snapshot(checkout, paths)?;
    for path in paths {
        check_deadline()?;
        let target = retained.join(path);
        protected(&target)?;
        fs::create_dir_all(target.parent().context("source parent missing")?)?;
        let bytes = checked_file(checkout, path)?;
        if target.exists() {
            ensure!(fs::read(&target)? == bytes, "retained source differs")
        } else {
            atomic(&target, &bytes)?;
        }
    }
    Ok(source.files_sha256)
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RevisionObjectKind {
    Commit,
    Tree,
    Blob,
}
impl RevisionObjectKind {
    fn as_str(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Tree => "tree",
            Self::Blob => "blob",
        }
    }
    fn parse(value: &str) -> Result<Self> {
        match value {
            "commit" => Ok(Self::Commit),
            "tree" => Ok(Self::Tree),
            "blob" => Ok(Self::Blob),
            _ => bail!("revision archive contains unsupported Git object type"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RevisionObject {
    pub oid: String,
    pub kind: RevisionObjectKind,
    pub reference: retention::Reference,
}

/// Exact new Git objects relative to a retained repository baseline. Object
/// files contain raw commit/tree/blob bytes, never Git directories or packs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RevisionArchive {
    pub source: SourceSnapshot,
    pub baseline: String,
    pub approved_paths: Vec<String>,
    pub objects: Vec<RevisionObject>,
    pub intent: retention::Reference,
    pub manifest: retention::Reference,
    pub max_bytes: u64,
    pub retained_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ObjectExtent {
    oid: String,
    kind: RevisionObjectKind,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RevisionIntent {
    schema: String,
    producer: PathBuf,
    archive: PathBuf,
    source: SourceSnapshot,
    baseline: String,
    approved_paths: Vec<String>,
    objects: Vec<ObjectExtent>,
    max_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RevisionManifest {
    schema: String,
    source: SourceSnapshot,
    baseline: String,
    approved_paths: Vec<String>,
    objects: Vec<RevisionObject>,
    intent: retention::Reference,
    max_bytes: u64,
    retained_bytes: u64,
}

fn approved_revision_paths(paths: &[String]) -> Result<Vec<String>> {
    ensure!(!paths.is_empty() && paths.len() <= MAX_REVISION_PATHS, "revision approved path count exceeded");
    let mut unique = BTreeSet::new();
    for path in paths {
        ensure!(
            path.len() <= 4096
                && relative_file(path)
                && !Path::new(path).starts_with(".git")
                && unique.insert(path.clone()),
            "revision approved path is unsafe or repeated"
        );
    }
    Ok(unique.into_iter().collect())
}

fn revision_shape(intent: &RevisionIntent) -> Result<()> {
    protected(&intent.producer)?;
    protected(&intent.archive)?;
    ensure!(
        !intent.producer.starts_with(&intent.archive) && !intent.archive.starts_with(&intent.producer),
        "revision archive overlaps producer checkout"
    );
    ensure!(intent.schema == "translation-revision-intent-v1" && intent.max_bytes > 0, "invalid revision intent");
    ensure!(
        exact_commit(&intent.baseline)
            && exact_commit(&intent.source.commit)
            && digest_valid(&intent.source.sha256)
            && intent.source.clean
            && approved_revision_paths(&intent.approved_paths)? == intent.approved_paths
            && intent.source.files_sha256.keys().cloned().collect::<Vec<_>>() == intent.approved_paths
            && intent.source.files_sha256.values().all(|value| digest_valid(value)),
        "revision source or ownership identity changed"
    );
    ensure!(intent.objects.len() <= MAX_REVISION_OBJECTS, "revision object count exceeded");
    let mut previous: Option<&str> = None;
    let mut bytes = 0u64;
    for object in &intent.objects {
        ensure!(
            exact_commit(&object.oid) && previous.is_none_or(|old| old < object.oid.as_str()),
            "revision object identity is invalid or repeated"
        );
        previous = Some(&object.oid);
        bytes = bytes.checked_add(object.bytes).context("revision archive byte extent overflow")?;
        ensure!(bytes <= intent.max_bytes, "revision object aggregate byte budget exceeded");
    }
    ensure!(
        intent.source.commit == intent.baseline
            || intent
                .objects
                .iter()
                .any(|object| object.oid == intent.source.commit && object.kind == RevisionObjectKind::Commit),
        "revision archive lacks its exact source commit"
    );
    Ok(())
}

fn verify_revision_commits(
    checkout: &Path,
    baseline: &str,
    source: &SourceSnapshot,
    paths: &[String],
    lease: &File,
    deadline: Instant,
) -> Result<()> {
    git_bounded(checkout, &["merge-base", "--is-ancestor", baseline, &source.commit], None, 4096, lease, deadline)?;
    let commits =
        git_leased(checkout, &["rev-list", "--reverse", &source.commit, &format!("^{baseline}")], lease, deadline)?;
    let commits = commits.lines().collect::<Vec<_>>();
    ensure!(commits.len() <= MAX_REVISION_OBJECTS, "revision commit count exceeded");
    let mut parent = baseline;
    for commit in commits {
        ensure!(exact_commit(commit), "revision ancestry contains inexact commit");
        let parents = git_leased(checkout, &["rev-list", "--parents", "-n", "1", commit], lease, deadline)?;
        ensure!(
            parents.split_whitespace().collect::<Vec<_>>() == vec![commit, parent],
            "revision archive requires linear scoped commits"
        );
        let diff = git_bounded(
            checkout,
            &["diff-tree", "--no-commit-id", "--no-renames", "--no-abbrev", "--raw", "-r", "-z", parent, commit, "--"],
            None,
            MAX_REVISION_RECORD_BYTES,
            lease,
            deadline,
        )?;
        let mut fields = diff.split(|byte| *byte == 0).filter(|field| !field.is_empty());
        while let Some(header) = fields.next() {
            let header = std::str::from_utf8(header)?.split_whitespace().collect::<Vec<_>>();
            let path = std::str::from_utf8(fields.next().context("revision diff lacks source path")?)?;
            ensure!(
                header.len() == 5
                    && [":100644", ":100755"].contains(&header[0])
                    && header[0].strip_prefix(':') == Some(header[1])
                    && header[4] == "M"
                    && paths.binary_search_by(|approved| approved.as_str().cmp(path)).is_ok(),
                "revision commit changes foreign files or source modes"
            );
        }
        parent = commit;
    }
    ensure!(parent == source.commit, "revision source is not descended from exact baseline");
    Ok(())
}

fn revision_inventory(
    checkout: &Path,
    source: &SourceSnapshot,
    baseline: &str,
    maximum: u64,
    lease: &File,
    deadline: Instant,
) -> Result<Vec<ObjectExtent>> {
    let listed = git_bounded(
        checkout,
        &["rev-list", "--objects", "--no-object-names", &source.commit, &format!("^{baseline}")],
        None,
        (MAX_REVISION_OBJECTS as u64).saturating_mul(41),
        lease,
        deadline,
    )?;
    let mut objects = BTreeSet::new();
    for oid in std::str::from_utf8(&listed)?.lines() {
        ensure!(
            exact_commit(oid) && objects.insert(oid.to_owned()),
            "revision object inventory is invalid or repeated"
        );
        ensure!(objects.len() <= MAX_REVISION_OBJECTS, "revision object count exceeded");
    }
    let mut total = 0u64;
    let mut inventory = Vec::new();
    for oid in objects {
        let kind = RevisionObjectKind::parse(&git_leased(checkout, &["cat-file", "-t", &oid], lease, deadline)?)?;
        let bytes = git_leased(checkout, &["cat-file", "-s", &oid], lease, deadline)?.parse::<u64>()?;
        total = total.checked_add(bytes).context("revision object extent overflow")?;
        ensure!(total <= maximum, "revision object aggregate byte budget exceeded before import");
        inventory.push(ObjectExtent { oid, kind, bytes });
    }
    Ok(inventory)
}

fn revision_metadata(manifest: &RevisionManifest) -> Result<Vec<u8>> {
    let encoded = serde_json::to_vec(manifest)?;
    ensure!(encoded.len() as u64 <= MAX_REVISION_RECORD_BYTES, "revision manifest metadata budget exceeded");
    Ok(encoded)
}

fn revision_total(intent_bytes: u64, manifest: &mut RevisionManifest) -> Result<u64> {
    let objects = manifest.objects.iter().try_fold(0u64, |total, object| {
        total.checked_add(object.reference.bytes).context("revision object extent overflow")
    })?;
    let base = intent_bytes.checked_add(objects).context("revision archive extent overflow")?;
    // Decimal length of this field affects its own encoded extent. Iteration
    // converges after crossing at most twenty decimal boundaries.
    manifest.retained_bytes = 0;
    for _ in 0..32 {
        let next =
            base.checked_add(revision_metadata(manifest)?.len() as u64).context("revision archive extent overflow")?;
        if next == manifest.retained_bytes {
            return Ok(next);
        }
        manifest.retained_bytes = next;
    }
    bail!("revision archive byte accounting did not converge")
}

fn archive_from_manifest(manifest: RevisionManifest, reference: retention::Reference) -> RevisionArchive {
    RevisionArchive {
        source: manifest.source,
        baseline: manifest.baseline,
        approved_paths: manifest.approved_paths,
        objects: manifest.objects,
        intent: manifest.intent,
        manifest: reference,
        max_bytes: manifest.max_bytes,
        retained_bytes: manifest.retained_bytes,
    }
}

fn archive_contents(root: &Path, objects: &[ObjectExtent], complete: bool) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_str().context("revision archive filename is not UTF8")?;
        if name == "objects" {
            ensure!(entry.file_type()?.is_dir(), "revision object directory is not regular");
            for object in fs::read_dir(entry.path())? {
                let object = object?;
                let name = object.file_name();
                let name = name.to_str().context("revision object filename is not UTF8")?;
                let oid =
                    name.strip_suffix(".bin").or_else(|| if complete { None } else { name.strip_suffix(".pending") });
                ensure!(
                    object.file_type()?.is_file()
                        && oid.is_some_and(|oid| objects.iter().any(|object| object.oid == oid)),
                    "revision archive contains unowned object paths"
                );
            }
        } else {
            ensure!(
                entry.file_type()?.is_file()
                    && (["intent.json", "revision.json", "lock"].contains(&name)
                        || !complete && ["intent.pending", "revision.pending"].contains(&name)),
                "revision archive contains unowned paths"
            );
        }
    }
    Ok(())
}

/// Seal exact newly introduced objects before retiring a controller checkout.
/// The immutable inventory reserves object and metadata bytes before import.
pub fn retain_revision(
    checkout: &Path,
    archive_dir: &Path,
    baseline: &str,
    approved: &[String],
    max_bytes: u64,
) -> Result<RevisionArchive> {
    with_deadline(None, || retain_revision_inner(checkout, archive_dir, baseline, approved, max_bytes))
}

pub fn retain_revision_until(
    checkout: &Path,
    archive_dir: &Path,
    baseline: &str,
    approved: &[String],
    max_bytes: u64,
    deadline_unix_ms: u64,
) -> Result<RevisionArchive> {
    with_deadline(Some(deadline_unix_ms), || {
        retain_revision_inner(checkout, archive_dir, baseline, approved, max_bytes)
    })
}

fn retain_revision_inner(
    checkout: &Path,
    archive_dir: &Path,
    baseline: &str,
    approved: &[String],
    max_bytes: u64,
) -> Result<RevisionArchive> {
    let deadline = deadline();
    protected(checkout)?;
    protected(archive_dir)?;
    ensure!(exact_commit(baseline) && max_bytes > 0, "revision needs exact baseline and a positive byte budget");
    ensure!(
        !checkout.starts_with(archive_dir) && !archive_dir.starts_with(checkout),
        "revision archive overlaps producer checkout"
    );
    let paths = approved_revision_paths(approved)?;
    retention::directory(archive_dir)?;
    let lease = controller_lock(&archive_dir.join("lock"))?;
    let intent_path = archive_dir.join("intent.json");
    let manifest_path = archive_dir.join("revision.json");
    if manifest_path.try_exists()? {
        let manifest: RevisionManifest =
            serde_json::from_slice(&read_source(&manifest_path, MAX_REVISION_RECORD_BYTES)?)?;
        let archive = archive_from_manifest(manifest, seal_source(&manifest_path, MAX_REVISION_RECORD_BYTES)?);
        let intent: RevisionIntent = serde_json::from_slice(&read_source(&intent_path, MAX_REVISION_RECORD_BYTES)?)?;
        ensure!(
            intent.producer == checkout
                && intent.archive == archive_dir
                && archive.baseline == baseline
                && archive.approved_paths == paths
                && archive.max_bytes == max_bytes,
            "revision archive retry request changed"
        );
        authenticate_revision(&archive)?;
        return Ok(archive);
    }
    let _source_lock = lock(checkout)?;
    let source = snapshot(checkout, &paths)?;
    verify_revision_commits(checkout, baseline, &source, &paths, &lease, deadline)?;
    let objects = revision_inventory(checkout, &source, baseline, max_bytes, &lease, deadline)?;
    let intent = RevisionIntent {
        schema: "translation-revision-intent-v1".into(),
        producer: checkout.into(),
        archive: archive_dir.into(),
        source: source.clone(),
        baseline: baseline.into(),
        approved_paths: paths.clone(),
        objects,
        max_bytes,
    };
    revision_shape(&intent)?;
    archive_contents(archive_dir, &intent.objects, false)?;
    let intent_bytes = serde_json::to_vec(&intent)?;
    ensure!(intent_bytes.len() as u64 <= MAX_REVISION_RECORD_BYTES, "revision intent metadata budget exceeded");
    let intent_ref = retention::Reference {
        path: intent_path.clone(),
        sha256: sha256(&intent_bytes),
        bytes: intent_bytes.len() as u64,
    };
    let mut manifest = RevisionManifest {
        schema: "translation-revision-archive-v1".into(),
        source,
        baseline: baseline.into(),
        approved_paths: paths,
        objects: intent
            .objects
            .iter()
            .map(|object| RevisionObject {
                oid: object.oid.clone(),
                kind: object.kind,
                reference: retention::Reference {
                    path: archive_dir.join("objects").join(format!("{}.bin", object.oid)),
                    sha256: "0".repeat(64),
                    bytes: object.bytes,
                },
            })
            .collect(),
        intent: intent_ref,
        max_bytes,
        retained_bytes: 0,
    };
    ensure!(
        revision_total(intent_bytes.len() as u64, &mut manifest)? <= max_bytes,
        "revision archive aggregate byte budget exceeded before publication"
    );
    write_retained(&intent_path, &intent_bytes, MAX_REVISION_RECORD_BYTES)?;
    retention::directory(&archive_dir.join("objects"))?;
    for object in &mut manifest.objects {
        let bytes = git_bounded(
            checkout,
            &["cat-file", object.kind.as_str(), &object.oid],
            None,
            object.reference.bytes,
            &lease,
            deadline,
        )?;
        ensure!(bytes.len() as u64 == object.reference.bytes, "revision raw object extent changed");
        let oid = String::from_utf8(git_bounded(
            checkout,
            &["hash-object", "-t", object.kind.as_str(), "--stdin"],
            Some(&bytes),
            4096,
            &lease,
            deadline,
        )?)?;
        ensure!(oid.trim() == object.oid, "revision raw object identity changed during extraction");
        object.reference.sha256 = sha256(&bytes);
        write_retained(&object.reference.path, &bytes, object.reference.bytes)?;
    }
    ensure!(
        snapshot(checkout, &manifest.approved_paths)? == manifest.source,
        "revision source changed during archival"
    );
    write_retained(&manifest_path, &revision_metadata(&manifest)?, MAX_REVISION_RECORD_BYTES)?;
    let archive = archive_from_manifest(manifest, seal_source(&manifest_path, MAX_REVISION_RECORD_BYTES)?);
    authenticate_revision(&archive)?;
    Ok(archive)
}

/// Authenticate only the bounded immutable archive. The producer and its Git
/// metadata may already have been retired.
pub fn authenticate_revision(archive: &RevisionArchive) -> Result<()> {
    with_deadline(None, || authenticate_revision_inner(archive))
}

pub fn authenticate_revision_until(archive: &RevisionArchive, deadline_unix_ms: u64) -> Result<()> {
    with_deadline(Some(deadline_unix_ms), || authenticate_revision_inner(archive))
}

fn authenticate_revision_inner(archive: &RevisionArchive) -> Result<()> {
    authenticate_reference(&archive.manifest, MAX_REVISION_RECORD_BYTES)?;
    authenticate_reference(&archive.intent, MAX_REVISION_RECORD_BYTES)?;
    let root = archive.manifest.path.parent().context("revision archive root missing")?;
    protected(root)?;
    let metadata = root.symlink_metadata()?;
    ensure!(
        metadata.is_dir() && metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "revision archive directory has foreign ownership or write authority"
    );
    ensure!(
        archive.manifest.path == root.join("revision.json") && archive.intent.path == root.join("intent.json"),
        "revision metadata references redirected"
    );
    let manifest: RevisionManifest =
        serde_json::from_slice(&read_source(&archive.manifest.path, MAX_REVISION_RECORD_BYTES)?)?;
    ensure!(
        manifest.schema == "translation-revision-archive-v1"
            && archive_from_manifest(manifest.clone(), archive.manifest.clone()) == *archive,
        "revision archive differs from immutable manifest"
    );
    let intent: RevisionIntent =
        serde_json::from_slice(&read_source(&archive.intent.path, MAX_REVISION_RECORD_BYTES)?)?;
    revision_shape(&intent)?;
    ensure!(
        intent.archive == root
            && intent.source == archive.source
            && intent.baseline == archive.baseline
            && intent.approved_paths == archive.approved_paths
            && intent.max_bytes == archive.max_bytes
            && intent.objects.len() == archive.objects.len(),
        "revision archive differs from its durable intent"
    );
    let mut manifest = manifest;
    let retained_bytes = revision_total(archive.intent.bytes, &mut manifest)?;
    ensure!(
        retained_bytes == archive.retained_bytes
            && retained_bytes <= archive.max_bytes
            && archive.manifest.bytes == revision_metadata(&manifest)?.len() as u64,
        "revision archive aggregate retained extent changed"
    );
    archive_contents(root, &intent.objects, true)?;
    for (extent, object) in intent.objects.iter().zip(&archive.objects) {
        ensure!(
            extent.oid == object.oid
                && extent.kind == object.kind
                && extent.bytes == object.reference.bytes
                && object.reference.path == root.join("objects").join(format!("{}.bin", object.oid)),
            "revision object reference redirected"
        );
        authenticate_reference(&object.reference, archive.max_bytes)?;
    }
    Ok(())
}

pub fn revision_references(archive: &RevisionArchive) -> Result<Vec<retention::Reference>> {
    authenticate_revision(archive)?;
    let mut refs = vec![archive.intent.clone(), archive.manifest.clone()];
    refs.extend(archive.objects.iter().map(|object| object.reference.clone()));
    Ok(refs)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct RestoreIntent {
    schema: String,
    repository: PathBuf,
    destination: PathBuf,
    archive: retention::Reference,
    baseline: String,
    source: SourceSnapshot,
    approved_paths: Vec<String>,
}

fn restore_partial_source(checkout: &Path, archive: &RevisionArchive) -> Result<()> {
    git_dir(checkout)?;
    let head = git(checkout, &["rev-parse", "HEAD"])?;
    ensure!(head == archive.baseline || head == archive.source.commit, "revision restore HEAD changed unexpectedly");
    if head == archive.source.commit {
        ensure!(
            snapshot(checkout, &archive.approved_paths)? == archive.source,
            "completed revision restore source changed"
        );
        return Ok(());
    }
    for path in dirty(checkout)? {
        let expected =
            archive.source.files_sha256.get(&path).context("partial revision restore contains foreign paths")?;
        let baseline = git_output(checkout, &["show", &format!("{}:{path}", archive.baseline)])?.stdout;
        let current = sha256(checked_file(checkout, &path)?);
        ensure!(
            current == *expected || current == sha256(baseline),
            "partial revision restore contains unexpected source"
        );
    }
    Ok(())
}

/// Recreate a clean detached checkout from the trusted baseline and raw sealed
/// objects, then prove the complete SourceSnapshot before returning authority.
pub fn restore_revision(
    repository: &Path,
    destination: &Path,
    archive: &RevisionArchive,
    approved: &[String],
) -> Result<SourceSnapshot> {
    with_deadline(None, || restore_revision_inner(repository, destination, archive, approved))
}

pub fn restore_revision_until(
    repository: &Path,
    destination: &Path,
    archive: &RevisionArchive,
    approved: &[String],
    deadline_unix_ms: u64,
) -> Result<SourceSnapshot> {
    with_deadline(Some(deadline_unix_ms), || restore_revision_inner(repository, destination, archive, approved))
}

fn restore_revision_inner(
    repository: &Path,
    destination: &Path,
    archive: &RevisionArchive,
    approved: &[String],
) -> Result<SourceSnapshot> {
    let deadline = deadline();
    authenticate_revision(archive)?;
    let paths = approved_revision_paths(approved)?;
    ensure!(paths == archive.approved_paths, "revision restore approved ownership changed");
    protected(repository)?;
    protected(destination)?;
    let root = archive.manifest.path.parent().context("revision archive root missing")?;
    ensure!(
        !destination.starts_with(root)
            && !root.starts_with(destination)
            && !destination.starts_with(repository)
            && !repository.starts_with(destination),
        "revision restore destination overlaps archive or repository"
    );
    let parent = destination.parent().context("revision restore destination parent missing")?;
    retention::directory(parent)?;
    let key = sha256(destination.as_os_str().as_encoded_bytes());
    let _lease = controller_lock(&parent.join(format!(".translation-restore-{key}.lock")))?;
    let intent_path = parent.join(format!(".translation-restore-{key}.json"));
    let intent = RestoreIntent {
        schema: "translation-revision-restore-v1".into(),
        repository: repository.into(),
        destination: destination.into(),
        archive: archive.manifest.clone(),
        baseline: archive.baseline.clone(),
        source: archive.source.clone(),
        approved_paths: paths.clone(),
    };
    if intent_path.try_exists()? {
        let saved: RestoreIntent = serde_json::from_slice(&read_source(&intent_path, MAX_REVISION_RECORD_BYTES)?)?;
        ensure!(saved == intent, "revision restore retry differs from original intent");
    } else {
        ensure!(!destination.try_exists()?, "revision restore destination exists without ownership");
        write_retained(&intent_path, &serde_json::to_vec(&intent)?, MAX_REVISION_RECORD_BYTES)?;
    }
    if !destination.join(".git").is_dir() || git_dir(destination).is_err() {
        create(repository, destination, &archive.baseline)?;
    }
    let source_lease = lock(destination)?;
    restore_partial_source(destination, archive)?;
    if git(destination, &["rev-parse", "HEAD"])? != archive.source.commit {
        for object in &archive.objects {
            let bytes = read_source(&object.reference.path, object.reference.bytes)?;
            let oid = String::from_utf8(git_bounded(
                destination,
                &["hash-object", "-w", "-t", object.kind.as_str(), "--stdin"],
                Some(&bytes),
                4096,
                &source_lease,
                deadline,
            )?)?;
            ensure!(oid.trim() == object.oid, "revision restore raw object differs from exact Git identity");
        }
        // The original restore intent and exact old/new approved contents have
        // already been checked. An interrupted Git checkout may leave its own
        // intended source modifications, which ordinary checkout refuses.
        git_bounded(
            destination,
            &["checkout", "--detach", "--force", &archive.source.commit],
            None,
            4096,
            &source_lease,
            deadline,
        )?;
    }
    let source = snapshot(destination, &paths)?;
    ensure!(source == archive.source, "restored revision differs from sealed complete source");
    write_retained(
        &parent.join(format!(".translation-restore-{key}-result.json")),
        &serde_json::to_vec(&(&intent, &source))?,
        MAX_REVISION_RECORD_BYTES,
    )?;
    Ok(source)
}

/// Publish authenticated source revisions into their original shared branch.
/// The caller must hold the assignment mutation lock and current authority.
pub fn publish_revisions(
    repository: &Path,
    candidate: &RevisionArchive,
    integrated: &RevisionArchive,
    context: &OperationContext,
    approved_paths: &[String],
    publication_root: &Path,
    deadline_unix_ms: u64,
) -> Result<Vec<retention::Reference>> {
    publish_revisions_with_controller_until(
        repository,
        candidate,
        integrated,
        context,
        approved_paths,
        publication_root,
        &ControllerProjection::default(),
        deadline_unix_ms,
    )
}

/// Exact mutable controller paths, authenticated by the managed caller under
/// assignment ownership. An absent future path needs no current reference.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ControllerProjection {
    pub paths: Vec<PathBuf>,
    pub references: Vec<retention::Reference>,
    /// Pre-existing retained metadata frozen by an explicit continuation.
    /// These hashes cannot change during source publication or its recovery.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preserved: Vec<retention::Reference>,
}

#[allow(clippy::too_many_arguments)]
pub fn publish_revisions_with_controller_until(
    repository: &Path,
    candidate: &RevisionArchive,
    integrated: &RevisionArchive,
    context: &OperationContext,
    approved_paths: &[String],
    publication_root: &Path,
    controller: &ControllerProjection,
    deadline_unix_ms: u64,
) -> Result<Vec<retention::Reference>> {
    with_deadline(Some(deadline_unix_ms), || {
        publish_revisions_inner(
            repository,
            candidate,
            integrated,
            context,
            approved_paths,
            publication_root,
            controller,
        )
    })
}

/// Authenticate the exact current shared commit/index/source and explicitly
/// admitted mutable controller projection. This creates no lock, intent or
/// source/ref/index mutation.
pub fn authenticate_shared_revision_until(
    repository: &Path,
    integrated: &RevisionArchive,
    approved_paths: &[String],
    controller: &ControllerProjection,
    deadline_unix_ms: u64,
) -> Result<()> {
    with_deadline(Some(deadline_unix_ms), || {
        authenticate_revision(integrated)?;
        let approved = approved_revision_paths(approved_paths)?;
        ensure!(approved == integrated.approved_paths, "shared revision approved ownership changed");
        let metadata = shared_metadata(repository)?;
        shared_branch(repository)?;
        let names = authenticate_controller_projection(repository, &metadata, &approved, controller)?;
        let read_lease = File::open(repository)?;
        ensure!(
            git_leased(repository, &["rev-parse", "HEAD"], &read_lease, deadline())? == integrated.source.commit,
            "shared authenticated source HEAD changed"
        );
        verify_imported_archive(repository, integrated, &read_lease)?;
        let files = tree_files(repository, &integrated.source.commit, &read_lease)?;
        let hashes = committed_hashes(repository, &files, &read_lease)?;
        ensure!(
            committed_snapshot(repository, &integrated.source.commit, &approved, &files, &read_lease)?
                == integrated.source,
            "shared committed revision differs from immutable archive"
        );
        verify_shared_projection(repository, (&files, &hashes), (&files, &hashes), &approved, &names, &read_lease, true)
    })
}

const MAX_SHARED_SOURCE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_INDEX_BYTES: u64 = 32 * 1024 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct TreeFile {
    mode: String,
    oid: String,
    bytes: u64,
}

fn tree_files(repository: &Path, commit: &str, lease: &File) -> Result<BTreeMap<String, TreeFile>> {
    ensure!(exact_commit(commit), "publication tree requires an exact commit");
    let listed = git_bounded(
        repository,
        &["ls-tree", "-r", "-l", "-z", commit],
        None,
        MAX_REVISION_RECORD_BYTES,
        lease,
        deadline(),
    )?;
    let mut files = BTreeMap::new();
    let mut total = 0u64;
    for entry in listed.split(|byte| *byte == 0).filter(|entry| !entry.is_empty()) {
        check_deadline()?;
        let split = entry.iter().position(|byte| *byte == b'\t').context("publication tree path missing")?;
        let fields = std::str::from_utf8(&entry[..split])?.split_whitespace().collect::<Vec<_>>();
        let path = std::str::from_utf8(&entry[split + 1..])?;
        ensure!(
            fields.len() == 4
                && ["100644", "100755"].contains(&fields[0])
                && fields[1] == "blob"
                && exact_commit(fields[2])
                && path.len() <= 4096
                && relative_file(path)
                && !Path::new(path).starts_with(".git"),
            "publication tree contains unsafe source modes or paths"
        );
        let bytes = fields[3].parse::<u64>()?;
        total = total.checked_add(bytes).context("publication source byte extent overflow")?;
        ensure!(total <= MAX_SHARED_SOURCE_BYTES, "publication source byte budget exceeded");
        ensure!(
            files.insert(path.to_owned(), TreeFile { mode: fields[0].into(), oid: fields[2].into(), bytes }).is_none(),
            "publication source path is repeated"
        );
        ensure!(files.len() <= 65536, "publication source path count exceeded");
    }
    Ok(files)
}

fn committed_hashes(
    repository: &Path,
    files: &BTreeMap<String, TreeFile>,
    lease: &File,
) -> Result<BTreeMap<String, String>> {
    let maximum = files
        .values()
        .try_fold(0u64, |total, entry| total.checked_add(entry.bytes).context("publication blob extent overflow"))?
        .checked_add((files.len() as u64).saturating_mul(128))
        .context("publication batch extent overflow")?;
    let input = files.values().map(|entry| format!("{}\n", entry.oid)).collect::<String>();
    let output = git_bounded(repository, &["cat-file", "--batch"], Some(input.as_bytes()), maximum, lease, deadline())?;
    let mut offset = 0usize;
    let mut hashes = BTreeMap::new();
    for (path, entry) in files {
        check_deadline()?;
        let header_end = output[offset..]
            .iter()
            .position(|byte| *byte == b'\n')
            .and_then(|length| offset.checked_add(length))
            .context("publication batch header missing")?;
        let header = std::str::from_utf8(&output[offset..header_end])?.split_whitespace().collect::<Vec<_>>();
        ensure!(
            header.len() == 3
                && header[0] == entry.oid
                && header[1] == "blob"
                && header[2].parse::<u64>()? == entry.bytes,
            "publication batch blob identity changed"
        );
        let start = header_end + 1;
        let end = start.checked_add(usize::try_from(entry.bytes)?).context("publication batch extent overflow")?;
        ensure!(end < output.len() && output[end] == b'\n', "publication batch blob extent changed");
        hashes.insert(path.clone(), sha256(&output[start..end]));
        offset = end + 1;
    }
    ensure!(offset == output.len(), "publication batch contains foreign output");
    Ok(hashes)
}

fn committed_snapshot(
    repository: &Path,
    commit: &str,
    approved: &[String],
    files: &BTreeMap<String, TreeFile>,
    lease: &File,
) -> Result<SourceSnapshot> {
    let full = committed_hashes(repository, files, lease)?;
    let tree = git_leased(repository, &["rev-parse", &format!("{commit}^{{tree}}")], lease, deadline())?;
    ensure!(exact_commit(&tree), "publication tree identity is inexact");
    let files_sha256 = approved
        .iter()
        .map(|path| Ok((path.clone(), full.get(path).context("publication approved source is not tracked")?.clone())))
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(SourceSnapshot { commit: commit.into(), sha256: sealed(&(commit, &tree, &full))?, files_sha256, clean: true })
}

fn shared_metadata(repository: &Path) -> Result<PathBuf> {
    protected(repository)?;
    let metadata = PathBuf::from(git(repository, &["rev-parse", "--absolute-git-dir"])?);
    protected(&metadata)?;
    let actual = metadata.symlink_metadata()?;
    ensure!(
        actual.is_dir() && actual.uid() == unsafe { libc::geteuid() } && actual.mode() & 0o022 == 0,
        "shared publication Git metadata has foreign ownership or write authority"
    );
    Ok(metadata)
}

fn shared_branch(repository: &Path) -> Result<String> {
    let branch = git(repository, &["symbolic-ref", "-q", "HEAD"])?;
    ensure!(branch.starts_with("refs/heads/"), "publication requires the original attached shared branch");
    git(repository, &["check-ref-format", &branch])?;
    Ok(branch)
}

fn index_entries(repository: &Path, index: Option<&Path>, lease: &File) -> Result<BTreeMap<String, (String, String)>> {
    let output = git_bounded_output(
        repository,
        &["ls-files", "--stage", "-z"],
        None,
        MAX_REVISION_RECORD_BYTES,
        Some(lease),
        deadline(),
        index,
    )?;
    ensure!(output.status.success(), "publication index cannot be inspected");
    let mut entries = BTreeMap::new();
    for entry in output.stdout.split(|byte| *byte == 0).filter(|entry| !entry.is_empty()) {
        check_deadline()?;
        let split = entry.iter().position(|byte| *byte == b'\t').context("publication index path missing")?;
        let fields = std::str::from_utf8(&entry[..split])?.split_whitespace().collect::<Vec<_>>();
        let path = std::str::from_utf8(&entry[split + 1..])?;
        ensure!(
            fields.len() == 3
                && ["100644", "100755"].contains(&fields[0])
                && exact_commit(fields[1])
                && fields[2] == "0"
                && relative_file(path)
                && !Path::new(path).starts_with(".git")
                && entries.insert(path.into(), (fields[0].into(), fields[1].into())).is_none(),
            "publication index has conflicts, aliases or foreign modes"
        );
    }
    let flags = git_bounded_output(
        repository,
        &["ls-files", "-v", "-z"],
        None,
        MAX_REVISION_RECORD_BYTES,
        Some(lease),
        deadline(),
        index,
    )?;
    ensure!(flags.status.success(), "publication index flags cannot be inspected");
    ensure!(
        flags.stdout.split(|byte| *byte == 0).filter(|entry| !entry.is_empty()).all(|entry| entry.starts_with(b"H ")),
        "publication index contains hidden or sparse source flags"
    );
    Ok(entries)
}

fn verify_shared_projection(
    repository: &Path,
    old: (&BTreeMap<String, TreeFile>, &BTreeMap<String, String>),
    new: (&BTreeMap<String, TreeFile>, &BTreeMap<String, String>),
    approved: &[String],
    controller: &BTreeSet<String>,
    lease: &File,
    complete: bool,
) -> Result<()> {
    let (old, old_hashes) = old;
    let (new, new_hashes) = new;
    ensure!(old.keys().eq(new.keys()), "publication changes source path inventory");
    let entries = index_entries(repository, None, lease)?;
    ensure!(entries.keys().eq(old.keys()), "publication index source inventory changed");
    for path in dirty(repository)? {
        ensure!(
            approved.binary_search(&path).is_ok() || controller.contains(&path),
            "publication contains unrelated dirty paths"
        );
    }
    for (path, original) in old {
        check_deadline()?;
        let desired = new.get(path).context("publication source disappeared")?;
        let owned = approved.binary_search(path).is_ok();
        ensure!(original.mode == desired.mode, "publication source mode changed");
        if !owned {
            ensure!(original == desired, "publication changes unapproved source");
        }
        let (mode, oid) = &entries[path];
        ensure!(
            mode == &original.mode && (oid == &desired.oid || !complete && oid == &original.oid),
            "publication index contains foreign source changes"
        );
        let file = repository.join(path);
        protected(&file)?;
        let metadata = file.symlink_metadata()?;
        ensure!(
            metadata.is_file()
                && metadata.nlink() == 1
                && metadata.uid() == unsafe { libc::geteuid() }
                && (metadata.mode() & 0o111 != 0) == (original.mode == "100755"),
            "publication source has foreign ownership, aliases or modes"
        );
        if controller.contains(path) {
            continue;
        }
        let current = seal_source(&file, MAX_SHARED_SOURCE_BYTES)?.sha256;
        ensure!(
            current == new_hashes[path] || !complete && current == old_hashes[path],
            "publication source contains changed approved contents or unrelated edits"
        );
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourcePublicationIntent {
    schema: String,
    repository: PathBuf,
    metadata: PathBuf,
    branch: String,
    context: OperationContext,
    candidate: retention::Reference,
    integrated: retention::Reference,
    approved_paths: Vec<String>,
    baseline: String,
    source: SourceSnapshot,
    candidate_ref: String,
    integrated_ref: String,
    index_marker: retention::Reference,
    controller_paths: Vec<PathBuf>,
    initial_controller_references: Vec<retention::Reference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    preserved: Vec<retention::Reference>,
}

fn authenticate_controller_projection(
    repository: &Path,
    metadata: &Path,
    approved: &[String],
    controller: &ControllerProjection,
) -> Result<BTreeSet<String>> {
    ensure!(
        controller.paths.len() <= 1024 && controller.references.len() <= 1024,
        "controller projection inventory exceeded"
    );
    let mut paths = BTreeSet::new();
    let mut absolute = BTreeSet::new();
    for path in &controller.paths {
        protected(path)?;
        let relative = path.strip_prefix(repository)?.to_str().context("controller projection path is not UTF8")?;
        ensure!(
            relative_file(relative)
                && Path::new(relative).starts_with(".scratch")
                && !path.starts_with(metadata)
                && !metadata.starts_with(path)
                && approved.iter().all(|source| {
                    let source = repository.join(source);
                    !path.starts_with(&source) && !source.starts_with(path)
                })
                && paths.insert(relative.into())
                && absolute.insert(path.clone()),
            "controller projection path is outside exact metadata ownership"
        );
    }
    let mut references = BTreeMap::new();
    let mut bytes = 0u64;
    for reference in &controller.references {
        ensure!(
            absolute.contains(&reference.path) && references.insert(reference.path.clone(), reference).is_none(),
            "controller projection reference is outside frozen inventory or repeated"
        );
        bytes = bytes.checked_add(reference.bytes).context("controller projection extent overflow")?;
        ensure!(bytes <= MAX_INDEX_BYTES, "controller projection aggregate byte budget exceeded");
        authenticate_reference(reference, MAX_INDEX_BYTES)?;
    }
    for path in absolute {
        check_deadline()?;
        if path.try_exists()? {
            let file = path.symlink_metadata()?;
            ensure!(
                file.is_file()
                    && file.nlink() == 1
                    && file.uid() == unsafe { libc::geteuid() }
                    && file.mode() & 0o022 == 0
                    && references.contains_key(&path),
                "existing controller projection lacks exact hash, ownership or regular file mode"
            );
        } else {
            ensure!(!references.contains_key(&path), "controller projection reference lacks current material");
        }
    }
    ensure!(controller.preserved.len() <= 4096, "preserved historical metadata inventory exceeded");
    let mut preserved_bytes = 0u64;
    for reference in &controller.preserved {
        let path = &reference.path;
        protected(path)?;
        let name = path.strip_prefix(repository)?.to_str().context("preserved metadata path is not UTF8")?;
        ensure!(
            relative_file(name)
                && Path::new(name).starts_with(".scratch")
                && !path.starts_with(metadata)
                && !metadata.starts_with(path)
                && approved.iter().all(|source| {
                    let source = repository.join(source);
                    !path.starts_with(&source) && !source.starts_with(path)
                })
                && paths.insert(name.into()),
            "preserved metadata overlaps mutable or source ownership"
        );
        preserved_bytes = preserved_bytes.checked_add(reference.bytes).context("preserved metadata extent overflow")?;
        ensure!(preserved_bytes <= MAX_SHARED_SOURCE_BYTES, "preserved metadata byte budget exceeded");
        authenticate_reference(reference, MAX_SHARED_SOURCE_BYTES)?;
    }
    Ok(paths)
}

fn own_publication_controller_paths(
    repository: &Path,
    publication_root: &Path,
    intent: &SourcePublicationIntent,
    intent_path: &Path,
    result_path: &Path,
) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    for path in [intent_path, result_path, &intent.index_marker.path, &publication_root.join("lock")] {
        if let Ok(relative) = path.strip_prefix(repository) {
            ensure!(
                relative.starts_with(".scratch")
                    && relative_file(relative.to_str().context("publication metadata UTF8")?),
                "publication metadata is outside controller source ownership"
            );
            paths.insert(relative.to_str().context("publication metadata UTF8")?.into());
        }
        if path.try_exists()? {
            let metadata = path.symlink_metadata()?;
            ensure!(
                metadata.is_file() && metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
                "publication controller metadata has foreign ownership or modes"
            );
            if path == publication_root.join("lock") {
                ensure!(metadata.nlink() == 1 && metadata.len() == 0, "publication lock content changed");
            } else if path == intent.index_marker.path {
                ensure!(metadata.nlink() == 1 || metadata.nlink() == 2, "publication index marker aliases changed");
                authenticate_reference(&intent.index_marker, MAX_REVISION_RECORD_BYTES)?;
            } else {
                ensure!(metadata.nlink() == 1, "publication controller metadata has aliases");
            }
        }
    }
    let encoded = serde_json::to_vec(intent)?;
    let pending = intent_path.with_extension("pending");
    if pending.try_exists()? {
        same_pending(&pending, &encoded)?;
        if let Ok(relative) = pending.strip_prefix(repository) {
            paths.insert(relative.to_str().context("publication pending metadata UTF8")?.into());
        }
    }
    Ok(paths)
}

fn initial_publication_controller_paths(
    repository: &Path,
    publication_root: &Path,
    marker: &[u8],
) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::new();
    for (path, expected) in
        [(publication_root.join("lock"), &[][..]), (publication_root.join("index-lock.marker"), marker)]
    {
        if path.try_exists()? {
            let metadata = path.symlink_metadata()?;
            ensure!(
                metadata.is_file()
                    && metadata.nlink() == 1
                    && metadata.uid() == unsafe { libc::geteuid() }
                    && metadata.mode() & 0o022 == 0
                    && read_source(&path, MAX_REVISION_RECORD_BYTES)? == expected,
                "source publication pre-intent metadata changed"
            );
            if let Ok(relative) = path.strip_prefix(repository) {
                paths.insert(relative.to_str().context("publication metadata UTF8")?.into());
            }
        }
    }
    let pending = publication_root.join("index-lock.pending");
    if pending.try_exists()? {
        same_pending(&pending, marker)?;
        if let Ok(relative) = pending.strip_prefix(repository) {
            paths.insert(relative.to_str().context("publication metadata UTF8")?.into());
        }
    }
    Ok(paths)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct SourcePublicationResult {
    schema: String,
    intent: retention::Reference,
    context: OperationContext,
    branch: String,
    candidate_ref: String,
    integrated_ref: String,
    candidate: SourceSnapshot,
    integrated: SourceSnapshot,
}

struct SharedIndexLock {
    path: PathBuf,
    marker: PathBuf,
}
impl Drop for SharedIndexLock {
    fn drop(&mut self) {
        if let (Ok(lock), Ok(marker)) = (self.path.symlink_metadata(), self.marker.symlink_metadata())
            && lock.is_file()
            && lock.dev() == marker.dev()
            && lock.ino() == marker.ino()
        {
            let _ = fs::remove_file(&self.path);
            if let Some(parent) = self.path.parent() {
                let _ = File::open(parent).and_then(|file| file.sync_all());
            }
        }
    }
}

fn shared_index_lock(metadata: &Path, marker: &retention::Reference) -> Result<SharedIndexLock> {
    authenticate_reference(marker, MAX_REVISION_RECORD_BYTES)?;
    let path = metadata.join("index.lock");
    protected(&path)?;
    check_deadline()?;
    match fs::hard_link(&marker.path, &path) {
        Ok(()) => File::open(metadata)?.sync_all()?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let held = path.symlink_metadata()?;
            let owner = marker.path.symlink_metadata()?;
            if !(held.is_file() && held.dev() == owner.dev() && held.ino() == owner.ino()) {
                return Err(
                    ExecutionPending { reason: "shared Git index has a surviving or foreign writer".into() }.into()
                );
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(SharedIndexLock { path, marker: marker.path.clone() })
}

fn import_archive(repository: &Path, archive: &RevisionArchive, lease: &File) -> Result<()> {
    for object in &archive.objects {
        let bytes = read_source(&object.reference.path, object.reference.bytes)?;
        ensure!(sha256(&bytes) == object.reference.sha256, "publication raw archive object changed");
        let oid = git_bounded(
            repository,
            &["hash-object", "-w", "-t", object.kind.as_str(), "--stdin"],
            Some(&bytes),
            4096,
            lease,
            deadline(),
        )?;
        ensure!(std::str::from_utf8(&oid)?.trim() == object.oid, "publication raw Git object identity changed");
    }
    verify_imported_archive(repository, archive, lease)
}

fn verify_imported_archive(repository: &Path, archive: &RevisionArchive, lease: &File) -> Result<()> {
    verify_revision_commits(
        repository,
        &archive.baseline,
        &archive.source,
        &archive.approved_paths,
        lease,
        deadline(),
    )?;
    let actual =
        revision_inventory(repository, &archive.source, &archive.baseline, archive.max_bytes, lease, deadline())?;
    ensure!(
        actual.len() == archive.objects.len()
            && actual.iter().zip(&archive.objects).all(|(extent, object)| {
                extent.oid == object.oid && extent.kind == object.kind && extent.bytes == object.reference.bytes
            }),
        "publication archive object closure differs from exact scoped revision"
    );
    Ok(())
}

fn exact_ref(repository: &Path, name: &str, lease: &File) -> Result<Option<String>> {
    let output = git_bounded_output(
        repository,
        &["rev-parse", "--verify", "--quiet", name],
        None,
        4096,
        Some(lease),
        deadline(),
        None,
    )?;
    if output.status.code() == Some(1) {
        return Ok(None);
    }
    ensure!(output.status.success(), "publication stable ref cannot be inspected");
    let oid = String::from_utf8(output.stdout)?.trim().to_owned();
    ensure!(exact_commit(&oid), "publication stable ref has inexact target");
    Ok(Some(oid))
}

fn update_publication_refs(
    repository: &Path,
    intent: &SourcePublicationIntent,
    candidate: &RevisionArchive,
    lease: &File,
) -> Result<()> {
    ensure!(shared_branch(repository)? == intent.branch, "publication shared branch changed");
    let head = git_leased(repository, &["rev-parse", "HEAD"], lease, deadline())?;
    ensure!(head == intent.baseline || head == intent.source.commit, "publication shared HEAD changed");
    // update-ref --stdin is one reference transaction on the installed Git,
    // including versions before the explicit start/prepare/commit vocabulary.
    let mut input = String::new();
    for (name, desired) in
        [(&intent.candidate_ref, &candidate.source.commit), (&intent.integrated_ref, &intent.source.commit)]
    {
        let existing = exact_ref(repository, name, lease)?;
        ensure!(
            existing.as_ref().is_none_or(|oid| oid == desired),
            "publication stable claim ref already names another source"
        );
        input.push_str(&format!("update {name} {desired} {}\n", existing.as_deref().unwrap_or(&"0".repeat(40))));
    }
    input.push_str(&format!("update {} {} {head}\n", intent.branch, intent.source.commit));
    git_bounded(repository, &["update-ref", "--stdin"], Some(input.as_bytes()), 4096, lease, deadline())?;
    Ok(())
}

fn update_shared_index(
    repository: &Path,
    metadata: &Path,
    intent: &SourcePublicationIntent,
    entries: &BTreeMap<String, TreeFile>,
    lease: &File,
) -> Result<()> {
    let index = metadata.join("index");
    let original = seal_source(&index, MAX_INDEX_BYTES)?;
    let temporary = metadata.join(format!("translation-{}.index", intent.context.operation_key));
    let pending = temporary.with_extension("index.lock");
    protected(&temporary)?;
    protected(&pending)?;
    if pending.try_exists()? {
        let metadata = pending.symlink_metadata()?;
        ensure!(
            metadata.is_file() && metadata.nlink() == 1 && metadata.len() <= MAX_INDEX_BYTES,
            "publication alternate index has unowned writer output"
        );
        check_deadline()?;
        fs::remove_file(&pending)?;
    }
    atomic(&temporary, &read_source(&index, MAX_INDEX_BYTES)?)?;
    let mut input = Vec::new();
    for path in &intent.approved_paths {
        let entry = &entries[path];
        input.extend_from_slice(format!("{} {}\t{path}\0", entry.mode, entry.oid).as_bytes());
    }
    let output = git_bounded_output(
        repository,
        &["update-index", "-z", "--index-info"],
        Some(&input),
        4096,
        Some(lease),
        deadline(),
        Some(&temporary),
    )?;
    ensure!(
        output.status.success(),
        "publication scoped index update failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let staged = index_entries(repository, Some(&temporary), lease)?;
    ensure!(
        staged.len() == entries.len()
            && staged.iter().all(|(path, (mode, oid))| {
                entries.get(path).is_some_and(|entry| &entry.mode == mode && &entry.oid == oid)
            }),
        "publication alternate index changed unapproved source"
    );
    authenticate_reference(&original, MAX_INDEX_BYTES)?;
    File::open(&temporary)?.sync_all()?;
    check_deadline()?;
    fs::rename(&temporary, &index)?;
    File::open(metadata)?.sync_all()?;
    Ok(())
}

fn publish_revisions_inner(
    repository: &Path,
    candidate: &RevisionArchive,
    integrated: &RevisionArchive,
    context: &OperationContext,
    approved_paths: &[String],
    publication_root: &Path,
    controller: &ControllerProjection,
) -> Result<Vec<retention::Reference>> {
    authenticate_revision(candidate)?;
    authenticate_revision(integrated)?;
    let approved = approved_revision_paths(approved_paths)?;
    ensure!(
        approved == candidate.approved_paths
            && approved == integrated.approved_paths
            && candidate.source.files_sha256 == integrated.source.files_sha256
            && context.phase == ExecutionPhase::Publication
            && context.attempt > 0
            && digest_valid(&context.run_id)
            && digest_valid(&context.policy_sha256)
            && digest_valid(&context.operation_key),
        "source publication differs from authenticated claim ownership or attempt"
    );
    protected(publication_root)?;
    ensure!(
        !repository.starts_with(publication_root)
            && approved.iter().all(|path| {
                let source = repository.join(path);
                !publication_root.starts_with(&source) && !source.starts_with(publication_root)
            }),
        "source publication state overlaps approved source"
    );
    let metadata = shared_metadata(repository)?;
    ensure!(
        !publication_root.starts_with(&metadata) && !metadata.starts_with(publication_root),
        "source publication state overlaps shared Git metadata"
    );
    let mut controller_names = authenticate_controller_projection(repository, &metadata, &approved, controller)?;
    for path in controller.paths.iter().chain(controller.preserved.iter().map(|reference| &reference.path)) {
        ensure!(
            !path.starts_with(publication_root) && !publication_root.starts_with(path),
            "caller controller projection overlaps publisher-owned metadata"
        );
    }
    let branch = shared_branch(repository)?;
    let lease = controller_lock(&metadata.join("translation-publication.lock"))?;
    let old_files = tree_files(repository, &integrated.baseline, &lease)?;
    let old_hashes = committed_hashes(repository, &old_files, &lease)?;
    let prefix = format!("refs/translation-execution/{}/attempt-{}", context.run_id, context.attempt);
    let candidate_ref = format!("{prefix}/candidate");
    let integrated_ref = format!("{prefix}/integrated");
    let intent_path = publication_root.join("intent.json");
    let result_path = publication_root.join("result.json");
    let marker_bytes =
        serde_json::to_vec(&("translation-source-index-lock-v1", context, repository, publication_root))?;
    let existing = if intent_path.try_exists()? {
        Some(serde_json::from_slice::<SourcePublicationIntent>(&read_source(&intent_path, MAX_REVISION_RECORD_BYTES)?)?)
    } else {
        None
    };
    let head = git_leased(repository, &["rev-parse", "HEAD"], &lease, deadline())?;
    if existing.is_none() {
        controller_names.extend(initial_publication_controller_paths(repository, publication_root, &marker_bytes)?);
        let pending_intent = intent_path.with_extension("pending");
        if pending_intent.try_exists()? {
            let marker_path = publication_root.join("index-lock.marker");
            let marker = seal_source(&marker_path, MAX_REVISION_RECORD_BYTES)?;
            let mut controller_paths = controller.paths.clone();
            controller_paths.sort();
            let prospective = SourcePublicationIntent {
                schema: "translation-source-publication-intent-v1".into(),
                repository: repository.into(),
                metadata: metadata.clone(),
                branch: branch.clone(),
                context: context.clone(),
                candidate: candidate.manifest.clone(),
                integrated: integrated.manifest.clone(),
                approved_paths: approved.clone(),
                baseline: integrated.baseline.clone(),
                source: integrated.source.clone(),
                candidate_ref: candidate_ref.clone(),
                integrated_ref: integrated_ref.clone(),
                index_marker: marker,
                controller_paths,
                initial_controller_references: controller.references.clone(),
                preserved: controller.preserved.clone(),
            };
            same_pending(&pending_intent, &serde_json::to_vec(&prospective)?)?;
            if let Ok(relative) = pending_intent.strip_prefix(repository) {
                controller_names.insert(relative.to_str().context("publication metadata UTF8")?.into());
            }
        }
        ensure!(head == integrated.baseline, "publication shared baseline or unrelated dirty source changed");
        verify_shared_projection(
            repository,
            (&old_files, &old_hashes),
            (&old_files, &old_hashes),
            &approved,
            &controller_names,
            &lease,
            true,
        )?;
        ensure!(!result_path.exists(), "source publication has result without intent");
    }
    retention::directory(publication_root)?;
    let _publication_lease = controller_lock(&publication_root.join("lock"))?;
    let marker_path = publication_root.join("index-lock.marker");
    if existing.is_none() {
        write_retained(&marker_path, &marker_bytes, MAX_REVISION_RECORD_BYTES)?;
    }
    let marker = seal_source(&marker_path, MAX_REVISION_RECORD_BYTES)?;
    ensure!(marker.sha256 == sha256(&marker_bytes), "publication index lock identity changed");
    let mut controller_paths = controller.paths.clone();
    controller_paths.sort();
    let initial_controller_references = existing
        .as_ref()
        .map(|saved| saved.initial_controller_references.clone())
        .unwrap_or_else(|| controller.references.clone());
    let intent = SourcePublicationIntent {
        schema: "translation-source-publication-intent-v1".into(),
        repository: repository.into(),
        metadata: metadata.clone(),
        branch: branch.clone(),
        context: context.clone(),
        candidate: candidate.manifest.clone(),
        integrated: integrated.manifest.clone(),
        approved_paths: approved.clone(),
        baseline: integrated.baseline.clone(),
        source: integrated.source.clone(),
        candidate_ref,
        integrated_ref,
        index_marker: marker,
        controller_paths,
        initial_controller_references,
        preserved: controller.preserved.clone(),
    };
    if let Some(saved) = existing {
        ensure!(saved == intent, "source publication retry changed its sealed intent");
    } else {
        write_retained(&intent_path, &serde_json::to_vec(&intent)?, MAX_REVISION_RECORD_BYTES)?;
    }
    controller_names.extend(own_publication_controller_paths(
        repository,
        publication_root,
        &intent,
        &intent_path,
        &result_path,
    )?);
    ensure!(head == intent.baseline || head == intent.source.commit, "publication has foreign shared HEAD");
    let _index_lock = shared_index_lock(&metadata, &intent.index_marker)?;
    let complete = result_path.try_exists()?;
    if complete {
        verify_imported_archive(repository, candidate, &lease)?;
        verify_imported_archive(repository, integrated, &lease)?;
    } else {
        import_archive(repository, candidate, &lease)?;
        import_archive(repository, integrated, &lease)?;
    }
    git_bounded(
        repository,
        &["merge-base", "--is-ancestor", &candidate.baseline, &integrated.baseline],
        None,
        4096,
        &lease,
        deadline(),
    )?;
    let candidate_files = tree_files(repository, &candidate.source.commit, &lease)?;
    let new_files = tree_files(repository, &integrated.source.commit, &lease)?;
    ensure!(
        committed_snapshot(repository, &candidate.source.commit, &approved, &candidate_files, &lease)?
            == candidate.source
            && committed_snapshot(repository, &integrated.source.commit, &approved, &new_files, &lease)?
                == integrated.source,
        "publication archived source differs from complete committed revision"
    );
    let new_hashes = committed_hashes(repository, &new_files, &lease)?;
    let intent_reference = seal_source(&intent_path, MAX_REVISION_RECORD_BYTES)?;
    let result = SourcePublicationResult {
        schema: "translation-source-publication-result-v1".into(),
        intent: intent_reference.clone(),
        context: context.clone(),
        branch,
        candidate_ref: intent.candidate_ref.clone(),
        integrated_ref: intent.integrated_ref.clone(),
        candidate: candidate.source.clone(),
        integrated: integrated.source.clone(),
    };
    if complete {
        let saved: SourcePublicationResult =
            serde_json::from_slice(&read_source(&result_path, MAX_REVISION_RECORD_BYTES)?)?;
        ensure!(saved == result, "source publication retained result changed");
        ensure!(
            git_leased(repository, &["rev-parse", "HEAD"], &lease, deadline())? == intent.source.commit,
            "completed source publication shared HEAD changed"
        );
    }
    let pending_result = result_path.with_extension("pending");
    if pending_result.try_exists()? {
        same_pending(&pending_result, &serde_json::to_vec(&result)?)?;
        if let Ok(relative) = pending_result.strip_prefix(repository) {
            controller_names.insert(relative.to_str().context("publication metadata UTF8")?.into());
        }
    }
    verify_shared_projection(
        repository,
        (&old_files, &old_hashes),
        (&new_files, &new_hashes),
        &approved,
        &controller_names,
        &lease,
        complete,
    )?;
    if !complete {
        for path in &approved {
            ensure!(shared_branch(repository)? == intent.branch, "publication shared branch changed during copy");
            let head = git_leased(repository, &["rev-parse", "HEAD"], &lease, deadline())?;
            ensure!(
                head == intent.baseline || head == intent.source.commit,
                "publication shared HEAD changed during copy"
            );
            let current = seal_source(&repository.join(path), MAX_SHARED_SOURCE_BYTES)?.sha256;
            ensure!(
                current == old_hashes[path] || current == new_hashes[path],
                "publication approved source changed during copy"
            );
            if current != new_hashes[path] {
                let entry = &new_files[path];
                let bytes =
                    git_bounded(repository, &["cat-file", "blob", &entry.oid], None, entry.bytes, &lease, deadline())?;
                ensure!(sha256(&bytes) == new_hashes[path], "publication desired source changed during copy");
                ensure!(
                    seal_source(&repository.join(path), MAX_SHARED_SOURCE_BYTES)?.sha256 == current,
                    "publication approved source changed before atomic copy"
                );
                atomic(&repository.join(path), &bytes)?;
                fs::set_permissions(
                    repository.join(path),
                    fs::Permissions::from_mode(if entry.mode == "100755" { 0o755 } else { 0o644 }),
                )?;
            }
        }
        verify_shared_projection(
            repository,
            (&old_files, &old_hashes),
            (&new_files, &new_hashes),
            &approved,
            &controller_names,
            &lease,
            false,
        )?;
        update_shared_index(repository, &metadata, &intent, &new_files, &lease)?;
        update_publication_refs(repository, &intent, candidate, &lease)?;
        verify_shared_projection(
            repository,
            (&old_files, &old_hashes),
            (&new_files, &new_hashes),
            &approved,
            &controller_names,
            &lease,
            true,
        )?;
        write_retained(&result_path, &serde_json::to_vec(&result)?, MAX_REVISION_RECORD_BYTES)?;
    }
    ensure!(
        exact_ref(repository, &intent.candidate_ref, &lease)?.as_deref() == Some(&candidate.source.commit)
            && exact_ref(repository, &intent.integrated_ref, &lease)?.as_deref() == Some(&integrated.source.commit)
            && git_leased(repository, &["rev-parse", "HEAD"], &lease, deadline())? == integrated.source.commit,
        "source publication stable reachability or shared branch differs"
    );
    Ok(vec![intent_reference, intent.index_marker, seal_source(&result_path, MAX_REVISION_RECORD_BYTES)?])
}
