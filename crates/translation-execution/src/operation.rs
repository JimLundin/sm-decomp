//! Durable execution of one installed, confined program. An immutable intent
//! precedes process creation. The runner inherits the operation lock, so a lost
//! parent cannot authorize another writer; it owns bounded final publication.
use crate::{
    ExecutionPending,
    confinement::{self, Program, RunLimits},
    model::{OperationContext, digest_valid, sealed, sha256},
    retention::{self, Reference},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Read,
    os::unix::{
        fs::{MetadataExt, OpenOptionsExt},
        io::{AsRawFd, FromRawFd},
        process::CommandExt,
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_METADATA: u64 = retention::MAX_METADATA_BYTES;
const MAX_IDENTITY_METADATA: u64 = 32 * 1024 * 1024;
const MAX_INPUT_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_TREE_ENTRIES: usize = 100_000;
const MAX_WRITABLE_BYTES: u64 = 1024 * 1024 * 1024;
const MAX_STDIN_BYTES: u64 = 64 * 1024 * 1024;
pub const MAX_DIAGNOSTIC_BYTES: usize = 4096;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProcessIdentity {
    pub pid: u32,
    pub start_ticks: u64,
    pub boot_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OperationResult {
    pub passed: bool,
    pub status_raw: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub outputs: BTreeMap<PathBuf, Vec<u8>>,
    pub diagnostic: String,
    pub result_sha256: String,
    pub retained_bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Stamp {
    device: u64,
    inode: u64,
    mode: u32,
    bytes: u64,
    modified: i64,
    modified_ns: i64,
    changed: i64,
    changed_ns: i64,
}
impl Stamp {
    fn new(m: &fs::Metadata) -> Self {
        Self {
            device: m.dev(),
            inode: m.ino(),
            mode: m.mode(),
            bytes: m.len(),
            modified: m.mtime(),
            modified_ns: m.mtime_nsec(),
            changed: m.ctime(),
            changed_ns: m.ctime_nsec(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: PathBuf,
    stamp: Stamp,
    sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct InputIdentity {
    source: PathBuf,
    runtime: bool,
    sha256: String,
    bytes: u64,
    entries: Vec<Entry>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Intent {
    schema: String,
    context: OperationContext,
    request_sha256: String,
    program: Program,
    limits: RunLimits,
    input: Reference,
    identities: Vec<Reference>,
    deadline_boot_ns: u64,
    deadline_boot_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Started {
    schema: String,
    request_sha256: String,
    intent_sha256: String,
    process: ProcessIdentity,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct ResultRecord {
    schema: String,
    context: OperationContext,
    request_sha256: String,
    intent_sha256: String,
    passed: bool,
    status_raw: Option<i32>,
    diagnostic: String,
    stdout: Option<Reference>,
    stderr: Option<Reference>,
    outputs: BTreeMap<PathBuf, Reference>,
    result_sha256: String,
}

fn bounded(value: &str) -> String {
    if value.len() <= MAX_DIAGNOSTIC_BYTES {
        return value.to_owned();
    }
    let mut end = MAX_DIAGNOSTIC_BYTES;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

fn pending(reason: &str) -> anyhow::Error {
    ExecutionPending { reason: bounded(reason) }.into()
}

fn valid_context(context: &OperationContext) -> Result<()> {
    ensure!(
        context.attempt > 0
            && digest_valid(&context.run_id)
            && digest_valid(&context.policy_sha256)
            && digest_valid(&context.operation_key),
        "durable operation requires exact context identities"
    );
    Ok(())
}

pub fn directory(root: &Path, context: &OperationContext) -> Result<PathBuf> {
    valid_context(context)?;
    retention::safe_path(root)?;
    ensure!(root.as_os_str().len() <= 4096, "controller root extent exceeded");
    Ok(root.join(&context.operation_key))
}

fn lock(root: &Path) -> Result<File> {
    let path = root.join("lock");
    retention::safe_path(&path)?;
    let file = OpenOptions::new().append(true).create(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&path)?;
    ensure!(file.metadata()?.is_file(), "operation lock is not regular");
    // A CLOEXEC lock can be inherited momentarily by another concurrent fork
    // before that child execs. Give this short kernel transition a bounded
    // grace period; a surviving runner still returns Pending without launch.
    let deadline = Instant::now() + Duration::from_millis(30);
    while unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
            if Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(2));
                continue;
            }
            return Err(pending("operation has a surviving or ambiguous runner; retain ownership"));
        }
        return Err(error.into());
    }
    Ok(file)
}

fn request_hash(context: &OperationContext, program: &Program, limits: &RunLimits, input: &[u8]) -> Result<String> {
    sealed(&(context, program, limits, sha256(input)))
}

fn check_limits(limits: &RunLimits, input: &[u8]) -> Result<()> {
    ensure!(
        !limits.timeout.is_zero() && limits.timeout <= Duration::from_secs(3600),
        "operation timeout outside reviewed hard bound"
    );
    ensure!(
        (4096..=MAX_WRITABLE_BYTES).contains(&limits.writable_bytes),
        "operation writable budget outside hard bound"
    );
    ensure!(
        limits.stdin_bytes as u64 <= MAX_STDIN_BYTES && input.len() <= limits.stdin_bytes,
        "operation stdin budget exceeded"
    );
    ensure!(
        limits.stdout_bytes as u64 <= limits.writable_bytes && limits.stderr_bytes as u64 <= limits.writable_bytes,
        "operation log extent exceeds aggregate budget"
    );
    Ok(())
}

/// Conservative reservation before production. The estimate uses bounded
/// metadata discovery, not runtime output telemetry: file hashes always occupy
/// 64 bytes, and all declared material output and bounded failure metadata are
/// reserved together. Callers aggregate this with source snapshots and their
/// retained native evidence before launching the first writer.
pub fn reserve_bytes(program: &Program, limits: &RunLimits, input_bytes: u64) -> Result<u64> {
    reserve_bytes_with_deadline(program, limits, input_bytes, discovery_deadline(limits, None)?)
}

/// Discover the installed input closure under the caller's persisted stage
/// deadline. This read-only preflight allocates no operation or cache state.
pub fn reserve_bytes_until(
    program: &Program,
    limits: &RunLimits,
    input_bytes: u64,
    deadline_unix_ms: u64,
) -> Result<u64> {
    reserve_bytes_with_deadline(program, limits, input_bytes, discovery_deadline(limits, Some(deadline_unix_ms))?)
}

fn reserve_bytes_with_deadline(
    program: &Program,
    limits: &RunLimits,
    input_bytes: u64,
    deadline_boot_ns: u64,
) -> Result<u64> {
    check_discovery_deadline(deadline_boot_ns)?;
    ensure!(
        input_bytes <= limits.stdin_bytes as u64 && input_bytes <= MAX_STDIN_BYTES,
        "operation input reservation exceeded"
    );
    ensure!(
        program.inputs.len() + program.runtime.len() <= 256 && program.outputs.len() <= 1024,
        "operation declaration count exceeds hard bound"
    );
    ensure!(limits.writable_bytes <= MAX_WRITABLE_BYTES, "operation reservation exceeds writable hard bound");
    let mut sources = vec![
        (program.executable.as_path(), false),
        (program.helper.as_path(), false),
        (Path::new("/usr/bin/bwrap"), false),
    ];
    sources.extend(program.inputs.iter().map(|b| (b.source.as_path(), false)));
    sources.extend(program.runtime.iter().map(|b| (b.source.as_path(), true)));
    let mut unique = BTreeSet::new();
    let mut identity_bytes = 0u64;
    for (source, runtime) in &sources {
        check_discovery_deadline(deadline_boot_ns)?;
        if !unique.insert((*source, *runtime)) {
            continue;
        }
        let (mut entries, bytes) = collect_entries(source, *runtime, false, deadline_boot_ns)?;
        for entry in &mut entries {
            check_discovery_deadline(deadline_boot_ns)?;
            entry.sha256 = "f".repeat(64);
        }
        let identity =
            InputIdentity { source: source.to_path_buf(), runtime: *runtime, sha256: "f".repeat(64), bytes, entries };
        let count = serde_json::to_vec(&identity)?.len() as u64;
        ensure!(count <= MAX_IDENTITY_METADATA, "installed identity exceeds bounded metadata extent");
        identity_bytes = identity_bytes.checked_add(count).context("operation identity reservation overflow")?;
        ensure!(identity_bytes <= MAX_IDENTITY_METADATA, "aggregate installed identity metadata exceeds hard bound");
    }
    let program_bytes = serde_json::to_vec(program)?.len() as u64;
    // Each reference reserves the original source path plus a SHA-keyed cache
    // filename and escaped JSON. Protected root paths are separately bounded
    // to 4096 bytes by run(), so this does not omit caller-controlled metadata.
    let references = (sources.len() as u64 + program.outputs.len() as u64 + 4)
        .checked_mul(8192 + 512)
        .context("operation reference reservation overflow")?;
    let records = program_bytes
        .checked_add(references)
        .and_then(|n| n.checked_add(MAX_DIAGNOSTIC_BYTES as u64 + 16 * 1024))
        .context("operation metadata reservation overflow")?;
    ensure!(program_bytes + references <= MAX_METADATA * 2, "operation intent/result reservation exceeded");
    check_discovery_deadline(deadline_boot_ns)?;
    identity_bytes
        .checked_add(records)
        .and_then(|n| n.checked_add(input_bytes))
        .and_then(|n| n.checked_add(limits.writable_bytes * 2))
        .context("operation total reservation overflow")
}

fn boot_ns() -> Result<u64> {
    let mut time = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    ensure!(
        unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut time) } == 0,
        "cannot establish durable operation deadline"
    );
    u64::try_from(time.tv_sec)?
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(time.tv_nsec as u64))
        .context("durable clock extent overflow")
}

fn boot_id() -> Result<String> {
    Ok(fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim().to_owned())
}

fn discovery_deadline(limits: &RunLimits, outer_deadline_unix_ms: Option<u64>) -> Result<u64> {
    ensure!(
        !limits.timeout.is_zero() && limits.timeout <= Duration::from_secs(3600),
        "operation timeout outside reviewed hard bound"
    );
    let entered = boot_ns()?;
    let local_deadline =
        entered.checked_add(u64::try_from(limits.timeout.as_nanos())?).context("operation total deadline overflow")?;
    Ok(if let Some(outer) = outer_deadline_unix_ms {
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        let remaining = outer.saturating_sub(now).saturating_mul(1_000_000);
        local_deadline.min(entered.saturating_add(remaining))
    } else {
        local_deadline
    })
}

fn check_discovery_deadline(deadline_boot_ns: u64) -> Result<()> {
    ensure!(boot_ns()? < deadline_boot_ns, "operation total deadline exhausted during declared input discovery");
    Ok(())
}

fn input_hash_until(path: &Path, deadline_boot_ns: u64) -> Result<String> {
    check_discovery_deadline(deadline_boot_ns)?;
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && before.len() <= MAX_INPUT_BYTES, "declared input file extent exceeded");
    let mut hash = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        check_discovery_deadline(deadline_boot_ns)?;
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        count = count.checked_add(read as u64).context("declared input file extent overflow")?;
        ensure!(count <= before.len(), "declared input grew during identity collection");
        hash.update(&buffer[..read]);
    }
    ensure!(
        count == before.len() && Stamp::new(&file.metadata()?) == Stamp::new(&before),
        "declared input changed during identity collection"
    );
    check_discovery_deadline(deadline_boot_ns)?;
    Ok(format!("{:x}", hash.finalize()))
}

fn collect_entries(
    source: &Path,
    runtime: bool,
    hash_content: bool,
    deadline_boot_ns: u64,
) -> Result<(Vec<Entry>, u64)> {
    check_discovery_deadline(deadline_boot_ns)?;
    retention::safe_path(source)?;
    let mut entries = Vec::new();
    let mut bytes = 0u64;
    fn walk(
        root: &Path,
        path: &Path,
        runtime: bool,
        hash_content: bool,
        deadline_boot_ns: u64,
        entries: &mut Vec<Entry>,
        bytes: &mut u64,
    ) -> Result<()> {
        check_discovery_deadline(deadline_boot_ns)?;
        ensure!(path.as_os_str().len() <= 4096, "declared input path extent exceeded");
        ensure!(entries.len() < MAX_TREE_ENTRIES, "declared input tree count exceeds bounded identity extent");
        let m = path.symlink_metadata()?;
        let stamp = Stamp::new(&m);
        let relative = path.strip_prefix(root)?.to_path_buf();
        let hash = if m.is_file() {
            *bytes = bytes.checked_add(m.len()).context("declared input byte extent overflow")?;
            ensure!(*bytes <= MAX_INPUT_BYTES, "declared input bytes exceed bounded identity extent");
            if hash_content { input_hash_until(path, deadline_boot_ns)? } else { String::new() }
        } else if m.file_type().is_symlink() {
            ensure!(runtime, "candidate input tree contains a symlink");
            let target = fs::read_link(path)?;
            ensure!(target.as_os_str().len() <= 4096, "runtime link extent exceeded");
            sha256(target.as_os_str().as_encoded_bytes())
        } else {
            ensure!(m.is_dir(), "declared input tree contains a device or socket");
            sha256("directory")
        };
        entries.push(Entry { path: relative, stamp, sha256: hash });
        if m.is_dir() {
            let mut paths = Vec::new();
            for entry in fs::read_dir(path)? {
                check_discovery_deadline(deadline_boot_ns)?;
                ensure!(
                    paths.len() + entries.len() < MAX_TREE_ENTRIES,
                    "declared input directory count exceeds bounded identity extent"
                );
                paths.push(entry?.path());
            }
            paths.sort();
            for child in paths {
                walk(root, &child, runtime, hash_content, deadline_boot_ns, entries, bytes)?;
            }
        }
        ensure!(
            Stamp::new(&path.symlink_metadata()?) == Stamp::new(&m),
            "declared input changed during identity collection"
        );
        check_discovery_deadline(deadline_boot_ns)?;
        Ok(())
    }
    walk(source, source, runtime, hash_content, deadline_boot_ns, &mut entries, &mut bytes)?;
    Ok((entries, bytes))
}

fn identity(source: &Path, runtime: bool, deadline_boot_ns: u64) -> Result<InputIdentity> {
    let (entries, bytes) = collect_entries(source, runtime, true, deadline_boot_ns)?;
    // Content identity excludes mutable host timestamps while metadata stamps
    // permit cached installed runtime verification without rehashing its bytes.
    let sha256 =
        sealed(&entries.iter().map(|e| (&e.path, e.stamp.mode & libc::S_IFMT, &e.sha256)).collect::<Vec<_>>())?;
    Ok(InputIdentity { source: source.to_path_buf(), runtime, sha256, bytes, entries })
}

fn verify_identity(expected: &InputIdentity, deadline_boot_ns: u64) -> Result<()> {
    let (observed, bytes) = collect_entries(&expected.source, expected.runtime, false, deadline_boot_ns)?;
    ensure!(
        observed.len() == expected.entries.len() && bytes == expected.bytes,
        "declared input identity extent changed"
    );
    for (entry, sealed) in observed.iter().zip(&expected.entries) {
        check_discovery_deadline(deadline_boot_ns)?;
        ensure!(entry.path == sealed.path && entry.stamp == sealed.stamp, "declared input metadata changed after seal");
        if !entry.sha256.is_empty() {
            ensure!(entry.sha256 == sealed.sha256, "runtime link or directory changed after seal");
        }
    }
    Ok(())
}

fn import_identities(
    root: &Path,
    program: &Program,
    deadline_boot_ns: u64,
    identities: &mut Vec<Reference>,
) -> Result<()> {
    check_discovery_deadline(deadline_boot_ns)?;
    let mut sources = vec![
        (program.executable.as_path(), false),
        (program.helper.as_path(), false),
        (Path::new("/usr/bin/bwrap"), false),
    ];
    sources.extend(program.inputs.iter().map(|b| (b.source.as_path(), false)));
    sources.extend(program.runtime.iter().map(|b| (b.source.as_path(), true)));
    let cache = root.join("installed-runtime");
    retention::directory(&cache)?;
    // Concurrent fixtures may import the same immutable identity. This is a
    // cache publication lock, not evidence of a surviving operation runner.
    let _lock = loop {
        check_discovery_deadline(deadline_boot_ns)?;
        match lock(&cache) {
            Ok(held) => break held,
            Err(error) if error.downcast_ref::<ExecutionPending>().is_some() => continue,
            Err(error) => return Err(error),
        }
    };
    for (source, runtime) in sources {
        check_discovery_deadline(deadline_boot_ns)?;
        // Every declared input is read-only inside the producer. Reuse its
        // protected manifest only after checking the full tree's device,
        // inode, size, mode, modification and change-time stamps. Repeated
        // Fixtures share helpers, binaries and ROMs as well as runtime files.
        // A changed input gets a new manifest; prior gates keep their own seal.
        let (stamps, _) = collect_entries(source, runtime, false, deadline_boot_ns)?;
        let cache_key = sealed(&(source, runtime, &stamps))?;
        let path = cache.join(format!("{cache_key}.json"));
        if path.try_exists()? {
            let value: InputIdentity = serde_json::from_slice(&retention::read_bounded(&path, MAX_IDENTITY_METADATA)?)?;
            ensure!(value.source == source && value.runtime == runtime, "installed input cache identity changed");
            verify_identity(&value, deadline_boot_ns)?;
        } else {
            let value = identity(source, runtime, deadline_boot_ns)?;
            retention::write_once(&path, &serde_json::to_vec(&value)?, MAX_IDENTITY_METADATA)?;
        }
        identities.push(retention::seal_file(&path, MAX_IDENTITY_METADATA)?);
    }
    check_discovery_deadline(deadline_boot_ns)
}

/// Linux process identity includes boot id and kernel start ticks, not a bare
/// PID. A zombie owns no writer and is treated as stopped.
pub fn process_identity(pid: u32) -> Result<Option<ProcessIdentity>> {
    let stat = match fs::read_to_string(format!("/proc/{pid}/stat")) {
        Ok(stat) => stat,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let fields =
        stat.rsplit_once(") ").context("process stat has no comm terminator")?.1.split_whitespace().collect::<Vec<_>>();
    ensure!(fields.len() > 19, "process stat lacks exact start identity");
    if fields[0] == "Z" || fields[0] == "X" {
        return Ok(None);
    }
    let boot_id = fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim().to_owned();
    Ok(Some(ProcessIdentity { pid, start_ticks: fields[19].parse()?, boot_id }))
}

fn is_live(identity: &ProcessIdentity) -> Result<bool> {
    Ok(process_identity(identity.pid)?.is_some_and(|observed| observed == *identity))
}

fn started(root: &Path, name: &str, intent: &Intent) -> Result<Option<ProcessIdentity>> {
    let path = root.join(name);
    if !path.try_exists()? {
        return Ok(None);
    }
    let started: Started = serde_json::from_slice(&retention::read_bounded(&path, MAX_METADATA)?)?;
    ensure!(
        started.schema == "translation-process-start-v1"
            && started.request_sha256 == intent.request_sha256
            && started.intent_sha256 == sealed(intent)?,
        "process start receipt changed"
    );
    Ok(Some(started.process))
}

fn write_started(root: &Path, name: &str, intent: &Intent, pid: u32) -> Result<()> {
    let process = process_identity(pid)?.context("spawned process exited before durable identity observation")?;
    let started = Started {
        schema: "translation-process-start-v1".into(),
        request_sha256: intent.request_sha256.clone(),
        intent_sha256: sealed(intent)?,
        process,
    };
    retention::write_once(&root.join(name), &serde_json::to_vec(&started)?, MAX_METADATA)
}

fn read_intent(root: &Path, context: &OperationContext) -> Result<Intent> {
    let intent: Intent = serde_json::from_slice(&retention::read_bounded(&root.join("intent.json"), MAX_METADATA)?)?;
    ensure!(
        intent.schema == "translation-operation-intent-v1" && intent.context == *context,
        "operation intent context changed"
    );
    retention::authenticate_reference(&intent.input, MAX_STDIN_BYTES)?;
    for reference in &intent.identities {
        retention::authenticate_reference(reference, MAX_IDENTITY_METADATA)?;
    }
    let input = retention::read_bounded(&intent.input.path, intent.limits.stdin_bytes as u64)?;
    ensure!(
        request_hash(context, &intent.program, &intent.limits, &input)? == intent.request_sha256,
        "operation intent request seal changed"
    );
    ensure!(intent.input.path == root.join("stdin.bin"), "operation stdin reference redirected");
    check_limits(&intent.limits, &input)?;
    Ok(intent)
}

fn result_seal(record: &ResultRecord) -> Result<String> {
    let mut unsealed = record.clone();
    unsealed.result_sha256.clear();
    sealed(&unsealed)
}

fn publish(root: &Path, intent: &Intent, output: Result<confinement::RunOutput>) -> Result<OperationResult> {
    use std::os::unix::process::ExitStatusExt;
    let mut record = ResultRecord {
        schema: "translation-operation-result-v1".into(),
        context: intent.context.clone(),
        request_sha256: intent.request_sha256.clone(),
        intent_sha256: sealed(intent)?,
        passed: false,
        status_raw: None,
        diagnostic: String::new(),
        stdout: None,
        stderr: None,
        outputs: BTreeMap::new(),
        result_sha256: String::new(),
    };
    let mut payload = Vec::new();
    match output {
        Ok(output) => {
            record.status_raw = Some(output.status.into_raw());
            record.passed = output.status.success();
            if record.passed {
                let total = output.stdout.len() as u64
                    + output.stderr.len() as u64
                    + output.outputs.values().map(|b| b.len() as u64).sum::<u64>();
                ensure!(total <= intent.limits.writable_bytes, "confined material output exceeded aggregate bound");
                let data = root.join("output");
                retention::directory(&data)?;
                let make_ref = |path: PathBuf, bytes: Vec<u8>| -> (Reference, Vec<u8>) {
                    (Reference { path, sha256: sha256(&bytes), bytes: bytes.len() as u64 }, bytes)
                };
                let (reference, bytes) = make_ref(data.join("stdout.bin"), output.stdout);
                record.stdout = Some(reference.clone());
                payload.push((reference, bytes));
                let (reference, bytes) = make_ref(data.join("stderr.bin"), output.stderr);
                record.stderr = Some(reference.clone());
                payload.push((reference, bytes));
                for (index, (path, bytes)) in output.outputs.into_iter().enumerate() {
                    let (reference, bytes) = make_ref(data.join(format!("member-{index:04}.bin")), bytes);
                    record.outputs.insert(path, reference.clone());
                    payload.push((reference, bytes));
                }
            } else {
                // A rejected gate retains only its bounded diagnostic extent,
                // never oversized logs or declared output from a failed writer.
                let prefix = format!("confined program failed ({:?}): ", record.status_raw);
                let stderr = &output.stderr[..output.stderr.len().min(MAX_DIAGNOSTIC_BYTES)];
                let stdout = &output.stdout[..output.stdout.len().min(MAX_DIAGNOSTIC_BYTES)];
                record.diagnostic =
                    bounded(&format!("{prefix}{}{}", String::from_utf8_lossy(stderr), String::from_utf8_lossy(stdout)));
            }
        }
        Err(error) => record.diagnostic = bounded(&format!("{error:#}")),
    }
    record.result_sha256 = result_seal(&record)?;
    let encoded = serde_json::to_vec(&record)?;
    ensure!(encoded.len() as u64 <= MAX_METADATA, "operation result metadata exceeded hard bound");
    for (reference, bytes) in payload {
        retention::write_once(&reference.path, &bytes, intent.limits.writable_bytes)?;
    }
    retention::write_once(&root.join("result.json"), &encoded, MAX_METADATA)?;
    load_result(root, intent)
}

fn load_result(root: &Path, intent: &Intent) -> Result<OperationResult> {
    let bytes = retention::read_bounded(&root.join("result.json"), MAX_METADATA)?;
    let record: ResultRecord = serde_json::from_slice(&bytes)?;
    ensure!(
        record.schema == "translation-operation-result-v1"
            && record.context == intent.context
            && record.request_sha256 == intent.request_sha256
            && record.result_sha256 == result_seal(&record)?
            && record.intent_sha256 == sealed(intent)?
            && record.diagnostic.len() <= MAX_DIAGNOSTIC_BYTES,
        "durable operation result seal changed"
    );
    if record.passed {
        use std::os::unix::process::ExitStatusExt;
        ensure!(
            record.status_raw.is_some_and(|raw| std::process::ExitStatus::from_raw(raw).success()),
            "successful result has no successful program status"
        );
        ensure!(record.stdout.is_some() && record.stderr.is_some(), "successful result lacks exact logs");
        ensure!(
            record.outputs.keys().cloned().collect::<BTreeSet<_>>() == intent.program.outputs.iter().cloned().collect(),
            "successful result output declarations changed"
        );
    } else {
        ensure!(
            record.stdout.is_none() && record.stderr.is_none() && record.outputs.is_empty(),
            "failed result retains material output beyond bounded diagnostic"
        );
    }
    let mut retained_bytes = bytes.len() as u64;
    let mut read = |reference: &Reference, expected: PathBuf, maximum: u64| -> Result<Vec<u8>> {
        ensure!(reference.path == expected, "operation output reference redirected");
        retention::authenticate_reference(reference, maximum)?;
        retained_bytes = retained_bytes.checked_add(reference.bytes).context("operation retained extent overflow")?;
        retention::read_bounded(&reference.path, maximum)
    };
    let stdout = record
        .stdout
        .as_ref()
        .map(|r| read(r, root.join("output/stdout.bin"), intent.limits.stdout_bytes as u64))
        .transpose()?
        .unwrap_or_default();
    let stderr = record
        .stderr
        .as_ref()
        .map(|r| read(r, root.join("output/stderr.bin"), intent.limits.stderr_bytes as u64))
        .transpose()?
        .unwrap_or_default();
    let mut outputs = BTreeMap::new();
    for (index, (path, reference)) in record.outputs.iter().enumerate() {
        outputs.insert(
            path.clone(),
            read(reference, root.join(format!("output/member-{index:04}.bin")), intent.limits.writable_bytes)?,
        );
    }
    ensure!(
        retained_bytes <= intent.limits.writable_bytes + MAX_METADATA,
        "retained operation output exceeds aggregate bound"
    );
    Ok(OperationResult {
        passed: record.passed,
        status_raw: record.status_raw,
        stdout,
        stderr,
        outputs,
        diagnostic: record.diagnostic,
        result_sha256: record.result_sha256,
        retained_bytes,
    })
}

fn reconcile(root: &Path, intent: &Intent) -> Result<OperationResult> {
    if root.join("result.json").try_exists()? {
        return load_result(root, intent);
    }
    for name in ["runner.json", "sandbox.json"] {
        if let Some(identity) = started(root, name, intent)?
            && is_live(&identity).map_err(|_| pending("operation process liveness unavailable; retain ownership"))?
        {
            return Err(pending("operation runner or sandbox is still live; retain ownership"));
        }
    }
    // Intent without a result can be an interrupted spawn or an interrupted
    // result write. It never authorizes relaunch, even if no PID was published.
    // Partial material output is recognized by exact declared member paths and
    // sealed bounded references before retirement; no arbitrary sweep occurs.
    discard_partial(root, intent)?;
    publish(
        root,
        intent,
        Err(anyhow::anyhow!("recorded operation stopped before sealed completion; no gate was rerun")),
    )
}

fn discard_partial(root: &Path, intent: &Intent) -> Result<()> {
    let mut files = vec![
        root.join("result.pending"),
        root.join("output/stdout.bin"),
        root.join("output/stdout.pending"),
        root.join("output/stderr.bin"),
        root.join("output/stderr.pending"),
    ];
    for index in 0..intent.program.outputs.len() {
        files.push(root.join(format!("output/member-{index:04}.bin")));
        files.push(root.join(format!("output/member-{index:04}.pending")));
    }
    let mut selected = Vec::new();
    let mut bytes = 0u64;
    for path in files {
        if path.try_exists()? {
            let reference = retention::seal_file(&path, intent.limits.writable_bytes + MAX_METADATA)?;
            bytes = bytes.checked_add(reference.bytes).context("partial result extent overflow")?;
            ensure!(bytes <= intent.limits.writable_bytes + MAX_METADATA, "partial output exceeds operation budget");
            selected.push(reference);
        }
    }
    let record = root.join("interrupted-output.json");
    if record.try_exists()? {
        let saved: Vec<Reference> = serde_json::from_slice(&retention::read_bounded(&record, MAX_METADATA)?)?;
        ensure!(
            selected.iter().all(|reference| saved.contains(reference)),
            "partial output changed after cleanup intent"
        );
        selected = saved;
    } else if !selected.is_empty() {
        retention::write_once(&record, &serde_json::to_vec(&selected)?, MAX_METADATA)?;
    }
    if !selected.is_empty() {
        for reference in &selected {
            if reference.path.try_exists()? {
                retention::authenticate_reference(reference, intent.limits.writable_bytes + MAX_METADATA)?;
                fs::remove_file(&reference.path)?;
            }
        }
        File::open(root)?.sync_all()?;
        retention::write_once(
            &root.join("interrupted-output-cleanup.json"),
            &serde_json::to_vec(&serde_json::json!({
                "schema":"translation-interrupted-output-cleanup-v1",
                "intent_sha256":sealed(intent)?,
                "retired_bytes":selected.iter().map(|reference|reference.bytes).sum::<u64>(),
                "retirement_sha256":sealed(&selected)?,
            }))?,
            MAX_METADATA,
        )?;
    }
    Ok(())
}

/// Execute or authenticate the exact existing operation. All paths/programs
/// come from a trusted installed adapter; rewrite responses supply only stdin.
pub fn run(
    root: &Path,
    context: &OperationContext,
    program: &Program,
    limits: &RunLimits,
    input: &[u8],
) -> Result<OperationResult> {
    run_with_deadline(root, context, program, limits, input, None)
}

/// Cap the first launch by a persisted adapter's absolute outer deadline. The
/// installed Program/limits request identity is unchanged; retries reconcile
/// the original sealed BOOTTIME cap rather than extending it.
pub fn run_until(
    root: &Path,
    context: &OperationContext,
    program: &Program,
    limits: &RunLimits,
    input: &[u8],
    deadline_unix_ms: u64,
) -> Result<OperationResult> {
    run_with_deadline(root, context, program, limits, input, Some(deadline_unix_ms))
}

fn run_with_deadline(
    root: &Path,
    context: &OperationContext,
    program: &Program,
    limits: &RunLimits,
    input: &[u8],
    outer_deadline_unix_ms: Option<u64>,
) -> Result<OperationResult> {
    let deadline_boot_ns = discovery_deadline(limits, outer_deadline_unix_ms)?;
    check_limits(limits, input)?;
    let operation = directory(root, context)?;
    retention::directory(root)?;
    retention::directory(&operation)?;
    let lock = lock(&operation)?;
    let request = request_hash(context, program, limits, input)?;
    let path = operation.join("intent.json");
    if path.try_exists()? {
        let intent = read_intent(&operation, context)?;
        ensure!(intent.request_sha256 == request, "same operation key retried with changed request");
        return reconcile(&operation, &intent);
    }
    ensure!(
        !operation.join("result.json").exists() && !operation.join("runner.json").exists(),
        "operation directory has output without an intent"
    );
    // An adapter's aggregate reservation includes the same bounded discovery;
    // direct callers still cannot import an unbounded runtime identity closure.
    let mut identities = Vec::new();
    let imported = reserve_bytes_with_deadline(program, limits, input.len() as u64, deadline_boot_ns)
        .and_then(|_| import_identities(root, program, deadline_boot_ns, &mut identities));
    let stdin_path = operation.join("stdin.bin");
    retention::write_once(&stdin_path, input, MAX_STDIN_BYTES)?;
    let intent = Intent {
        schema: "translation-operation-intent-v1".into(),
        context: context.clone(),
        request_sha256: request,
        program: program.clone(),
        limits: limits.clone(),
        input: retention::seal_file(&stdin_path, MAX_STDIN_BYTES)?,
        identities,
        deadline_boot_ns,
        deadline_boot_id: boot_id()?,
    };
    retention::write_once(&path, &serde_json::to_vec(&intent)?, MAX_METADATA)?;
    if let Err(error) = imported {
        return publish(&operation, &intent, Err(error));
    }
    if boot_ns()? >= intent.deadline_boot_ns {
        return publish(
            &operation,
            &intent,
            Err(anyhow::anyhow!("operation total deadline exhausted during declared input import")),
        );
    }
    // The lock's open file description is shared with the spawned runner.
    // Clearing CLOEXEC makes ownership durable through exec; the runner closes
    // inheritance before launching candidate namespaces.
    let fd = lock.as_raw_fd();
    let mut command = Command::new(&program.helper);
    command
        .args(["--operation-runner"])
        .arg(&operation)
        .arg(fd.to_string())
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(fd, libc::F_GETFD);
            if flags < 0 || libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::setpgid(0, 0) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let spawn = command.spawn();
    let mut child = match spawn {
        Ok(child) => child,
        Err(error) => return publish(&operation, &intent, Err(error.into())),
    };
    let deadline = intent.deadline_boot_ns.saturating_add(10_000_000_000);
    loop {
        if child.try_wait()?.is_some() {
            break;
        }
        if boot_ns()? >= deadline {
            // A live recorded runner retains ownership; do not turn a stalled
            // bootstrap into a duplicate writer or an unauthenticated success.
            return Err(pending(
                "durable runner has not finalized within its bound; reconcile or stop its exact identity",
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    reconcile(&operation, &intent)
}

/// Trusted helper mode called on the host. It cannot be invoked successfully
/// from a candidate namespace: no controller path or inherited host lock exists.
pub fn runner_main() -> Result<()> {
    let arguments = std::env::args().skip(2).collect::<Vec<_>>();
    ensure!(arguments.len() == 2, "invalid durable runner request");
    let root = PathBuf::from(&arguments[0]);
    retention::directory(&root)?;
    let fd: i32 = arguments[1].parse()?;
    ensure!(fd >= 3, "runner lock is not a protected descriptor");
    let file = unsafe { File::from_raw_fd(fd) };
    let held = file.metadata()?;
    let path = root.join("lock").symlink_metadata()?;
    ensure!(held.is_file() && held.dev() == path.dev() && held.ino() == path.ino(), "runner lock identity changed");
    ensure!(
        unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "runner does not hold exclusive operation ownership"
    );
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    ensure!(
        flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) } == 0,
        "runner cannot restrict lock inheritance"
    );
    let raw: Intent = serde_json::from_slice(&retention::read_bounded(&root.join("intent.json"), MAX_METADATA)?)?;
    let intent = read_intent(&root, &raw.context)?;
    let self_executable = fs::canonicalize("/proc/self/exe")?;
    ensure!(self_executable == intent.program.helper, "uninstalled helper attempted durable runner mode");
    ensure!(
        !root.join("runner.json").exists() && !root.join("result.json").exists(),
        "durable operation already launched"
    );
    write_started(&root, "runner.json", &intent, std::process::id())?;
    let result = (|| {
        for reference in &intent.identities {
            retention::authenticate_reference(reference, MAX_IDENTITY_METADATA)?;
            let identity: InputIdentity =
                serde_json::from_slice(&retention::read_bounded(&reference.path, MAX_IDENTITY_METADATA)?)?;
            verify_identity(&identity, intent.deadline_boot_ns)?;
        }
        let input = retention::read_bounded(&intent.input.path, intent.limits.stdin_bytes as u64)?;
        ensure!(boot_id()? == intent.deadline_boot_id, "operation deadline belongs to another boot");
        let remaining = intent
            .deadline_boot_ns
            .checked_sub(boot_ns()?)
            .context("operation total deadline exhausted before candidate launch")?;
        ensure!(remaining > 0, "operation total deadline exhausted before candidate launch");
        let mut limits = intent.limits.clone();
        limits.timeout = limits.timeout.min(Duration::from_nanos(remaining));
        confinement::run_observed(&intent.program, &limits, &input, |pid| {
            write_started(&root, "sandbox.json", &intent, pid)
        })
    })();
    publish(&root, &intent, result)?;
    drop(file);
    Ok(())
}

pub fn authenticate(root: &Path, context: &OperationContext) -> Result<OperationResult> {
    let path = directory(root, context)?;
    let intent = read_intent(&path, context)?;
    ensure!(path.join("result.json").try_exists()?, "operation has no sealed completion");
    load_result(&path, &intent)
}

/// Read-only trust transition for a prescribed adapter invocation. Matching
/// context alone does not authorize another installed program or argv.
pub fn authenticate_request(
    root: &Path,
    context: &OperationContext,
    program: &Program,
    limits: &RunLimits,
    input: &[u8],
) -> Result<OperationResult> {
    let path = directory(root, context)?;
    let intent = read_intent(&path, context)?;
    ensure!(
        intent.request_sha256 == request_hash(context, program, limits, input)?,
        "cached operation differs from prescribed request"
    );
    ensure!(path.join("result.json").try_exists()?, "operation has no sealed completion");
    load_result(&path, &intent)
}

pub fn reference(root: &Path, context: &OperationContext) -> Result<Reference> {
    authenticate(root, context)?;
    retention::seal_file(&directory(root, context)?.join("result.json"), MAX_METADATA)
}

/// Exact transitive dependencies needed to authenticate a receipt after its
/// source checkout/runtime is retired. Material output is referenced only for
/// passing gates; failed gates retain one bounded diagnostic.
pub fn references(root: &Path, context: &OperationContext) -> Result<Vec<Reference>> {
    let path = directory(root, context)?;
    let intent = read_intent(&path, context)?;
    load_result(&path, &intent)?;
    let record: ResultRecord =
        serde_json::from_slice(&retention::read_bounded(&path.join("result.json"), MAX_METADATA)?)?;
    let mut references = vec![
        retention::seal_file(&path.join("intent.json"), MAX_METADATA)?,
        intent.input,
        retention::seal_file(&path.join("result.json"), MAX_METADATA)?,
    ];
    references.extend(intent.identities);
    references.extend(record.stdout);
    references.extend(record.stderr);
    references.extend(record.outputs.into_values());
    for name in ["runner.json", "sandbox.json", "interrupted-output.json", "interrupted-output-cleanup.json"] {
        let record = path.join(name);
        if record.try_exists()? {
            references.push(retention::seal_file(&record, MAX_METADATA)?);
        }
    }
    Ok(references)
}

/// Stop only exact recorded kernel identities. pidfd signaling cannot hit a
/// reused PID, and namespace death tears down detached candidate descendants.
pub fn stop(root: &Path, context: &OperationContext) -> Result<OperationResult> {
    let path = directory(root, context)?;
    let intent = read_intent(&path, context)?;
    if path.join("result.json").try_exists()? {
        return load_result(&path, &intent);
    }
    for name in ["sandbox.json", "runner.json"] {
        if let Some(identity) = started(&path, name, &intent)? {
            if !is_live(&identity).map_err(|_| pending("cannot establish exact process liveness for stop"))? {
                continue;
            }
            let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, identity.pid, 0) as i32 };
            ensure!(fd >= 0, "kernel cannot establish race-free process stop");
            let owned = unsafe { File::from_raw_fd(fd) };
            if is_live(&identity)? {
                ensure!(
                    unsafe {
                        libc::syscall(
                            libc::SYS_pidfd_send_signal,
                            owned.as_raw_fd(),
                            libc::SIGKILL,
                            std::ptr::null::<libc::siginfo_t>(),
                            0,
                        )
                    } == 0,
                    "cannot stop exact recorded process"
                );
            }
        }
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match lock(&path) {
            Ok(_lock) => return reconcile(&path, &intent),
            Err(error) if error.downcast_ref::<ExecutionPending>().is_some() && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(error) => return Err(error),
        }
    }
}
