//! Bounded, read-only capture of original claims for reviewed migration preparation.
//! This writes only an isolated evidence package; liveness observations are inputs.
use crate::{assignment, translation_migration as migration};
use ::translation_execution::{
    ClaimLineage,
    model::{digest_valid, sealed, sha256},
    operation,
    retention::Reference,
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, DirBuilder, File, OpenOptions},
    io::{Read, Write},
    os::unix::{
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        io::AsRawFd,
        process::CommandExt,
    },
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const MAX_RECORD: u64 = 4 * 1024 * 1024;
const MAX_BYTES: u64 = 256 * 1024 * 1024;
const MAX_FILES: usize = 4096;
const AUTHORITY: &str = ".scratch/decomp-parallel/authority.toml";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClaimRequest {
    pub id: String,
    /// Exact current integration commit, or an explicitly checkpointed running
    /// implementation. A submitted result always supplies its original commit.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct BuildLimits {
    pub timeout_seconds: u64,
    pub max_bytes: u64,
    pub max_files: usize,
    pub max_source_file_bytes: u64,
}

impl Default for BuildLimits {
    fn default() -> Self {
        Self {
            timeout_seconds: 60,
            max_bytes: 64 * 1024 * 1024,
            max_files: MAX_FILES,
            max_source_file_bytes: MAX_RECORD,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildRequest {
    pub schema: String,
    pub checkout: PathBuf,
    pub effort: String,
    pub claims: Vec<ClaimRequest>,
    pub stopped_owners: Vec<Reference>,
    pub proof_installation: Reference,
    pub proof_completion: Reference,
    /// A private directory below this architecture worktree's reviewed
    /// production-preparation root. The generated package lives in `package/`.
    pub output_root: PathBuf,
    #[serde(default)]
    pub limits: BuildLimits,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct BuildReceipt {
    pub schema: String,
    pub status: String,
    pub activated: bool,
    pub request_sha256: String,
    pub migration: Reference,
    pub inventory: Reference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preparation_report: Option<Reference>,
    pub validation_status: String,
    pub retained_bytes: u64,
    pub referenced_files: usize,
    pub blockers: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct Authority {
    schema: String,
    state: String,
    preparation: assignment::authority::Preparation,
    roles: BTreeMap<String, assignment::authority::Role>,
    drill: Option<assignment::Artifact>,
}

#[derive(Serialize, Deserialize)]
struct Ledger {
    schema: String,
    capacity: usize,
    jobs: BTreeMap<String, assignment::Entry>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourcePlan {
    id: String,
    source_commit: String,
    resolved_baseline: Option<String>,
    current_integration_head: bool,
    files: Vec<String>,
}

#[derive(Deserialize)]
struct SavedIntent {
    schema: String,
    request: BuildRequest,
    request_reference: Option<Reference>,
    authority: Reference,
    ledger: Reference,
    sources: Vec<SourcePlan>,
}

#[derive(Serialize)]
struct Intent<'a> {
    schema: &'static str,
    request: &'a BuildRequest,
    request_reference: &'a Option<Reference>,
    authority: &'a Reference,
    ledger: &'a Reference,
    sources: &'a [SourcePlan],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedInput {
    claim_id: String,
    key: String,
    original_value: String,
    original_path: PathBuf,
    reference: Reference,
    captured_from_commit: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputDecision {
    schema: String,
    claim_id: String,
    key: String,
    original_value: String,
    original_path: PathBuf,
    captured: Option<CapturedInput>,
    blocker: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ValidationDecision {
    schema: String,
    migration: Reference,
    status: String,
    report: Option<Reference>,
    blockers: Vec<String>,
}

#[derive(Serialize)]
struct Inventory<'a> {
    schema: &'static str,
    request_sha256: &'a str,
    sources: &'a [SourcePlan],
    inputs: &'a [CapturedInput],
    references: Vec<&'a Reference>,
    validation_status: &'a str,
    blockers: &'a [String],
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SavedInventory {
    schema: String,
    request_sha256: String,
    sources: Vec<SourcePlan>,
    inputs: Vec<CapturedInput>,
    references: Vec<Reference>,
    validation_status: String,
    blockers: Vec<String>,
}

struct Extent {
    limits: BuildLimits,
    deadline: Instant,
    references: BTreeMap<PathBuf, Reference>,
    bytes: u64,
    // An interrupted package remains charged before its outputs are admitted in
    // deterministic order. Inventory and receipt never count themselves early.
    unadmitted: BTreeMap<PathBuf, Reference>,
    unadmitted_bytes: u64,
}

impl Extent {
    fn new(limits: &BuildLimits) -> Result<Self> {
        ensure!((1..=300).contains(&limits.timeout_seconds), "migration builder timeout is outside its hard bound");
        ensure!(limits.max_bytes > 0 && limits.max_bytes <= MAX_BYTES, "migration builder aggregate extent invalid");
        ensure!((1..=MAX_FILES).contains(&limits.max_files), "migration builder file bound invalid");
        ensure!((1..=MAX_RECORD).contains(&limits.max_source_file_bytes), "migration source file bound invalid");
        Ok(Self {
            limits: limits.clone(),
            deadline: Instant::now() + Duration::from_secs(limits.timeout_seconds),
            references: BTreeMap::new(),
            bytes: 0,
            unadmitted: BTreeMap::new(),
            unadmitted_bytes: 0,
        })
    }
    fn check(&self) -> Result<()> {
        ensure!(Instant::now() < self.deadline, "migration builder deadline exhausted");
        Ok(())
    }
    fn room(&self, path: &Path, bytes: u64) -> Result<()> {
        self.room_for(path, bytes, false)
    }
    fn room_for(&self, path: &Path, bytes: u64, publishing: bool) -> Result<()> {
        self.check()?;
        if let Some(prior) = self.references.get(path) {
            ensure!(prior.bytes == bytes, "migration file extent changed");
        }
        let displaced = std::iter::once(path.to_path_buf())
            .chain(publishing.then(|| pending_path(path)))
            .filter_map(|path| self.unadmitted.get(&path))
            .collect::<Vec<_>>();
        let displaced_bytes: u64 = displaced.iter().map(|reference| reference.bytes).sum();
        let additional_file = usize::from(!self.references.contains_key(path));
        ensure!(
            self.references.len() + self.unadmitted.len() - displaced.len() + additional_file <= self.limits.max_files,
            "migration builder file count exhausted"
        );
        let additional_bytes = if additional_file == 0 { 0 } else { bytes };
        ensure!(
            self.bytes
                .checked_add(self.unadmitted_bytes - displaced_bytes)
                .and_then(|total| total.checked_add(additional_bytes))
                .is_some_and(|total| total <= self.limits.max_bytes),
            "migration builder aggregate extent exhausted"
        );
        Ok(())
    }
    fn add(&mut self, reference: Reference) -> Result<Reference> {
        self.room(&reference.path, reference.bytes)?;
        ensure!(digest_valid(&reference.sha256), "migration reference digest invalid");
        if let Some(prior) = self.references.get(&reference.path) {
            ensure!(prior == &reference, "migration reference conflict");
        } else {
            if let Some(existing) = self.unadmitted.remove(&reference.path) {
                ensure!(existing == reference, "retained migration package file changed");
                self.unadmitted_bytes -= existing.bytes;
            }
            self.bytes += reference.bytes;
            self.references.insert(reference.path.clone(), reference.clone());
        }
        Ok(reference)
    }
    fn authenticate(&mut self, reference: &Reference) -> Result<Reference> {
        self.room(&reference.path, reference.bytes)?;
        ensure!(
            seal(&reference.path, reference.bytes, self.deadline)? == *reference,
            "migration supplied evidence changed"
        );
        self.add(reference.clone())
    }
    fn file(&mut self, path: &Path, maximum: u64) -> Result<Reference> {
        let reference = seal(path, maximum.min(self.limits.max_bytes), self.deadline)?;
        self.add(reference)
    }
    fn artifact(&mut self, artifact: &assignment::Artifact) -> Result<Reference> {
        let reference = self.file(&artifact.path, self.limits.max_bytes)?;
        ensure!(
            reference.sha256 == artifact.sha256,
            "original historical artifact changed: {}",
            artifact.path.display()
        );
        Ok(reference)
    }
    fn publish(&mut self, path: &Path, bytes: &[u8]) -> Result<Reference> {
        self.room_for(path, bytes.len() as u64, true)?;
        write_once(path, bytes, self.deadline)?;
        if let Some(pending) = self.unadmitted.remove(&pending_path(path)) {
            self.unadmitted_bytes -= pending.bytes;
        }
        self.add(Reference { path: path.into(), sha256: sha256(bytes), bytes: bytes.len() as u64 })
    }
    fn record(&mut self, path: &Path, value: &impl Serialize) -> Result<Reference> {
        let bytes = serde_json::to_vec(value)?;
        ensure!(bytes.len() as u64 <= MAX_RECORD, "migration generated record exceeds fixed bound");
        self.publish(path, &bytes)
    }
    fn recheck(&self) -> Result<()> {
        for reference in self.references.values() {
            self.check()?;
            ensure!(
                seal(&reference.path, reference.bytes, self.deadline)? == *reference,
                "migration input or retained output changed before return"
            );
        }
        Ok(())
    }
    fn remember(&mut self, reference: Reference) -> Result<()> {
        if let Some(prior) = self.references.get(&reference.path) {
            ensure!(prior == &reference, "migration package/input reference conflict");
            return Ok(());
        }
        ensure!(
            self.references.len() + self.unadmitted.len() < self.limits.max_files,
            "migration builder file count exhausted by interrupted package"
        );
        ensure!(
            self.bytes
                .checked_add(self.unadmitted_bytes)
                .and_then(|total| total.checked_add(reference.bytes))
                .is_some_and(|total| total <= self.limits.max_bytes),
            "migration builder aggregate extent exhausted by interrupted package"
        );
        self.unadmitted_bytes += reference.bytes;
        ensure!(
            self.unadmitted.insert(reference.path.clone(), reference).is_none(),
            "duplicate migration package entry"
        );
        Ok(())
    }
}

fn package_files(root: &Path, extent: &Extent) -> Result<BTreeMap<PathBuf, Reference>> {
    let mut pending = vec![root.to_path_buf()];
    let mut entries_seen = 0usize;
    let mut files = BTreeMap::new();
    let mut bytes = 0u64;
    while let Some(path) = pending.pop() {
        extent.check()?;
        private(&path)?;
        ensure!(path.strip_prefix(root)?.components().count() <= 68, "migration package directory depth exceeded");
        let metadata = path.symlink_metadata()?;
        if metadata.is_file() {
            ensure!(files.len() < extent.limits.max_files, "migration package file count exceeded");
            bytes = bytes.checked_add(metadata.len()).context("migration package byte overflow")?;
            ensure!(bytes <= extent.limits.max_bytes, "migration package aggregate extent exceeded");
            files.insert(path.clone(), seal(&path, metadata.len(), extent.deadline)?);
        } else {
            for entry in fs::read_dir(&path)? {
                extent.check()?;
                entries_seen = entries_seen.checked_add(1).context("migration package entry count overflow")?;
                ensure!(entries_seen <= MAX_FILES * 4, "migration package directory count exceeded");
                pending.push(entry?.path());
            }
        }
    }
    Ok(files)
}

fn authenticate_package(root: &Path, extent: &Extent) -> Result<()> {
    for (path, reference) in package_files(root, extent)? {
        ensure!(extent.references.get(&path) == Some(&reference), "migration package has a foreign or changed file");
    }
    Ok(())
}

fn authenticate_intent(
    request: &BuildRequest,
    request_reference: &Option<Reference>,
    package: &Path,
    authority: &Reference,
    ledger: &Reference,
    sources: &[SourcePlan],
    extent: &Extent,
) -> Result<()> {
    let saved: SavedIntent = object(&package.join("build-intent.json"), extent.deadline)?;
    ensure!(
        saved.schema == "translation-migration-build-intent-v1"
            && sealed(&saved.request)? == sealed(request)?
            && saved.request_reference == *request_reference
            && saved.authority == *authority
            && saved.ledger == *ledger
            && sealed(&saved.sources)? == sealed(&sources)?,
        "retained migration package belongs to another build request or source authority"
    );
    Ok(())
}

fn resume_package(
    request: &BuildRequest,
    package: &Path,
    plans: &[SourcePlan],
    extent: &mut Extent,
) -> Result<BuildReceipt> {
    let receipt: BuildReceipt = object(&package.join("build-receipt.json"), extent.deadline)?;
    ensure!(
        receipt.schema == "translation-migration-build-receipt-v1"
            && receipt.status == "prepared-with-blockers"
            && receipt.request_sha256 == sealed(request)?
            && !receipt.activated
            && receipt.migration.path == package.join("migration.json")
            && receipt.inventory.path == package.join("inventory.json"),
        "retained migration receipt identity changed"
    );
    extent.authenticate(&receipt.inventory)?;
    let inventory: SavedInventory = object(&receipt.inventory.path, extent.deadline)?;
    ensure!(
        inventory.schema == "translation-migration-build-inventory-v1"
            && inventory.request_sha256 == receipt.request_sha256
            && sealed(&inventory.sources)? == sealed(&plans)?
            && inventory.validation_status == receipt.validation_status
            && inventory.blockers == receipt.blockers
            && inventory.references.len() <= extent.limits.max_files
            && inventory.inputs.len() <= extent.limits.max_files,
        "retained migration inventory identity changed"
    );
    let mut paths = BTreeSet::new();
    for reference in &inventory.references {
        ensure!(paths.insert(&reference.path), "retained inventory has duplicate references");
        extent.authenticate(reference)?;
    }
    for plan in plans {
        for name in &plan.files {
            let source = blob(
                &request.checkout,
                &plan.source_commit,
                name,
                extent.limits.max_source_file_bytes,
                extent.deadline,
            )?;
            let path = package.join("source").join(&plan.id).join(name);
            let retained = extent.file(&path, extent.limits.max_source_file_bytes)?;
            ensure!(retained.sha256 == sha256(&source), "retained source snapshot differs from its original commit");
        }
    }
    for input in &inventory.inputs {
        authenticate_captured(request, package, plans, input, extent)?;
    }
    extent.authenticate(&receipt.migration)?;
    if let Some(report) = &receipt.preparation_report {
        extent.authenticate(report)?;
    }
    extent.file(&package.join("build-receipt.json"), MAX_RECORD)?;
    authenticate_package(package, extent)?;
    ensure!(
        extent.bytes == receipt.retained_bytes && extent.references.len() == receipt.referenced_files,
        "retained migration package extent or files changed"
    );
    // Identical recovery authenticates the retained actual validator decision.
    // A new read-only validation remains available through validate-preparation.
    let decision: ValidationDecision = object(&package.join("validation.json"), extent.deadline)?;
    ensure!(
        decision.schema == "translation-migration-validation-decision-v1"
            && decision.migration == receipt.migration
            && decision.status == receipt.validation_status
            && decision.report == receipt.preparation_report,
        "retained preparation validation decision changed"
    );
    extent.recheck()?;
    Ok(receipt)
}

fn name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 160
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn exact_commit(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn relative(path: &Path) -> bool {
    let bytes = path.as_os_str().as_encoded_bytes();
    !bytes.is_empty()
        && bytes.len() <= 4096
        && !bytes.contains(&0)
        && !bytes.contains(&b'\\')
        && path.components().collect::<PathBuf>().as_os_str() == path.as_os_str()
        && path.components().all(|component| matches!(component, Component::Normal(_)))
        && !path.components().any(|component| component.as_os_str() == ".git")
        && path.components().count() <= 64
}

fn safe(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute() && path.as_os_str().as_encoded_bytes().len() <= 4096,
        "migration builder path must be bounded and absolute"
    );
    ensure!(path.components().collect::<PathBuf>().as_os_str() == path.as_os_str(), "migration builder path alias");
    let mut prefix = PathBuf::new();
    for component in path.components() {
        ensure!(matches!(component, Component::RootDir | Component::Normal(_)), "migration builder path alias");
        prefix.push(component);
        match prefix.symlink_metadata() {
            Ok(metadata) => ensure!(!metadata.file_type().is_symlink(), "migration builder path crosses a symlink"),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn private(path: &Path) -> Result<()> {
    safe(path)?;
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o022 == 0,
        "migration builder resource must be operator-owned and protected"
    );
    ensure!(
        metadata.is_dir() || metadata.is_file() && metadata.nlink() == 1,
        "migration builder resource is nonregular or aliased"
    );
    Ok(())
}

fn directory(path: &Path) -> Result<()> {
    safe(path)?;
    DirBuilder::new().recursive(true).mode(0o700).create(path)?;
    private(path)
}

fn same_file(before: &fs::Metadata, after: &fs::Metadata) -> bool {
    before.len() == after.len()
        && before.dev() == after.dev()
        && before.ino() == after.ino()
        && before.mtime() == after.mtime()
        && before.mtime_nsec() == after.mtime_nsec()
        && before.ctime() == after.ctime()
        && before.ctime_nsec() == after.ctime_nsec()
}

fn seal(path: &Path, maximum: u64, deadline: Instant) -> Result<Reference> {
    safe(path)?;
    ensure!(Instant::now() < deadline, "migration evidence deadline exhausted before read");
    let observed = path.symlink_metadata()?;
    ensure!(observed.is_file() && observed.len() <= maximum, "migration evidence file is nonregular or excessive");
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && same_file(&observed, &before), "migration evidence file changed before read");
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0; 65536];
    loop {
        ensure!(Instant::now() < deadline, "migration evidence deadline exhausted");
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        bytes = bytes.checked_add(count as u64).context("migration evidence byte overflow")?;
        ensure!(bytes <= maximum, "migration evidence grew beyond its bound");
        digest.update(&buffer[..count]);
    }
    ensure!(bytes == before.len() && same_file(&before, &file.metadata()?), "migration evidence changed during read");
    Ok(Reference { path: path.into(), sha256: format!("{:x}", digest.finalize()), bytes })
}

fn read_bytes(path: &Path, maximum: u64, deadline: Instant) -> Result<Vec<u8>> {
    safe(path)?;
    ensure!(Instant::now() < deadline, "migration record deadline exhausted before read");
    let observed = path.symlink_metadata()?;
    ensure!(observed.is_file() && observed.len() <= maximum, "migration record is nonregular or excessive");
    let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
    let before = file.metadata()?;
    ensure!(before.is_file() && same_file(&observed, &before), "migration record changed before read");
    let mut bytes = Vec::new();
    let mut buffer = [0; 65536];
    loop {
        ensure!(Instant::now() < deadline, "migration record deadline exhausted");
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        ensure!(count as u64 <= maximum.saturating_sub(bytes.len() as u64), "migration record grew beyond its bound");
        bytes.extend_from_slice(&buffer[..count]);
    }
    ensure!(
        bytes.len() as u64 == before.len() && same_file(&before, &file.metadata()?),
        "migration record changed during read"
    );
    Ok(bytes)
}

fn object<T: DeserializeOwned>(path: &Path, deadline: Instant) -> Result<T> {
    let bytes = read_bytes(path, MAX_RECORD, deadline)?;
    if path.extension().is_some_and(|extension| extension == "toml") {
        Ok(toml::from_str(std::str::from_utf8(&bytes)?)?)
    } else {
        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn pending_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".pending");
    PathBuf::from(name)
}

fn remove_pending(path: &Path, bytes: &[u8], deadline: Instant) -> Result<()> {
    if path.try_exists()? {
        private(path)?;
        let partial = read_bytes(path, bytes.len() as u64, deadline)?;
        ensure!(bytes.starts_with(&partial), "migration pending file is not an interrupted identical publication");
        fs::remove_file(path)?;
    }
    Ok(())
}

fn write_once(path: &Path, bytes: &[u8], deadline: Instant) -> Result<()> {
    ensure!(Instant::now() < deadline, "migration publication deadline exhausted");
    let parent = path.parent().context("migration publication parent missing")?;
    directory(parent)?;
    safe(path)?;
    let pending = pending_path(path);
    if path.try_exists()? {
        private(path)?;
        ensure!(
            read_bytes(path, bytes.len() as u64, deadline)? == bytes,
            "identical migration retry changed retained bytes"
        );
        remove_pending(&pending, bytes, deadline)?;
        return Ok(());
    }
    remove_pending(&pending, bytes, deadline)?;
    let mut file =
        OpenOptions::new().create_new(true).write(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&pending)?;
    for part in bytes.chunks(65536) {
        ensure!(Instant::now() < deadline, "migration publication deadline exhausted");
        file.write_all(part)?;
    }
    file.sync_all()?;
    file.set_permissions(fs::Permissions::from_mode(0o400))?;
    fs::rename(&pending, path)?;
    File::open(parent)?.sync_all()?;
    Ok(())
}

fn drain(reader: &mut impl Read, output: &mut Vec<u8>, done: &mut bool, maximum: u64, deadline: Instant) -> Result<()> {
    if *done {
        return Ok(());
    }
    let mut buffer = [0; 8192];
    loop {
        ensure!(Instant::now() < deadline, "migration subprocess deadline exhausted while draining output");
        match reader.read(&mut buffer) {
            Ok(0) => {
                *done = true;
                return Ok(());
            }
            Ok(count) => {
                ensure!(
                    count as u64 <= maximum.saturating_sub(output.len() as u64),
                    "migration subprocess output bound exceeded"
                );
                output.extend_from_slice(&buffer[..count]);
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
}

// The trusted validator can launch its own Git process groups. All stay in this
// unique session; retain the unreaped leader while terminating that session.
fn stop_session(session: u32) {
    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.take(65536).flatten() {
            let Some(pid) = entry.file_name().to_str().and_then(|name| name.parse::<u32>().ok()) else {
                continue;
            };
            let Ok(stat) = fs::read_to_string(entry.path().join("stat")) else {
                continue;
            };
            let Some((_, fields)) = stat.rsplit_once(") ") else {
                continue;
            };
            if fields.split_whitespace().nth(3).and_then(|field| field.parse::<u32>().ok()) == Some(session) {
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }
    }
}

fn command_output(mut command: Command, maximum: u64, deadline: Instant) -> Result<Vec<u8>> {
    ensure!(Instant::now() < deadline, "migration subprocess deadline exhausted before launch");
    command
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_NO_REPLACE_OBJECTS", "1")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn()?;
    let mut reaped = false;
    let result = (|| -> Result<Vec<u8>> {
        let mut stdout = child.stdout.take().context("migration subprocess stdout missing")?;
        let mut stderr = child.stderr.take().context("migration subprocess stderr missing")?;
        for fd in [stdout.as_raw_fd(), stderr.as_raw_fd()] {
            let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
            ensure!(
                flags >= 0 && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } == 0,
                "migration subprocess pipe setup failed"
            );
        }
        let mut output = Vec::new();
        let mut diagnostic = Vec::new();
        let mut stdout_done = false;
        let mut stderr_done = false;
        loop {
            drain(&mut stdout, &mut output, &mut stdout_done, maximum, deadline)?;
            drain(&mut stderr, &mut diagnostic, &mut stderr_done, 4096, deadline)?;
            if stdout_done
                && stderr_done
                && let Some(status) = child.try_wait()?
            {
                reaped = true;
                ensure!(status.success(), "migration subprocess failed: {}", String::from_utf8_lossy(&diagnostic));
                return Ok(output);
            }
            ensure!(Instant::now() < deadline, "migration subprocess deadline exhausted");
            std::thread::sleep(Duration::from_millis(5));
        }
    })();
    if !reaped {
        stop_session(child.id());
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn git(checkout: &Path, arguments: &[&str], maximum: u64, deadline: Instant) -> Result<Vec<u8>> {
    safe(checkout)?;
    let mut command = Command::new("/usr/bin/git");
    command
        .arg("--no-pager")
        .args([
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "gc.auto=0",
            "-c",
            "maintenance.auto=false",
        ])
        .args(arguments)
        .current_dir(checkout);
    command_output(command, maximum, deadline)
}

fn resolve(checkout: &Path, revision: &str, deadline: Instant) -> Result<String> {
    ensure!(
        (7..=40).contains(&revision.len()) && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
        "original baseline must be an exact or legacy abbreviated Git object id"
    );
    let bytes = git(
        checkout,
        &["rev-parse", "--verify", "--end-of-options", &format!("{revision}^{{commit}}")],
        256,
        deadline,
    )?;
    let commit = std::str::from_utf8(&bytes)?.trim().to_owned();
    ensure!(exact_commit(&commit), "original source did not resolve to one full commit");
    Ok(commit)
}

fn blob(checkout: &Path, commit: &str, path: &str, maximum: u64, deadline: Instant) -> Result<Vec<u8>> {
    ensure!(exact_commit(commit) && relative(Path::new(path)), "committed migration source path invalid");
    let tree = git(checkout, &["ls-tree", "-z", commit, "--", path], 8192, deadline)?;
    let entries = tree.split(|byte| *byte == 0).filter(|entry| !entry.is_empty()).collect::<Vec<_>>();
    ensure!(entries.len() == 1, "committed migration source file is absent or ambiguous");
    let split = entries[0].iter().position(|byte| *byte == b'\t').context("migration tree entry malformed")?;
    let fields = std::str::from_utf8(&entries[0][..split])?.split_whitespace().collect::<Vec<_>>();
    ensure!(
        fields.len() == 3
            && matches!(fields[0], "100644" | "100755")
            && fields[1] == "blob"
            && exact_commit(fields[2])
            && &entries[0][split + 1..] == path.as_bytes(),
        "committed migration source has an unsafe mode or path"
    );
    git(checkout, &["show", &format!("{commit}:{path}")], maximum, deadline)
}

fn source_plan(
    request: &BuildRequest,
    selected: &ClaimRequest,
    entry: &assignment::Entry,
    deadline: Instant,
) -> Result<SourcePlan> {
    let claim = entry.claim.as_ref().context("selected original job has no held claim")?;
    ensure!(
        entry.job.id == selected.id
            && claim.id == selected.id
            && claim.revision == entry.job.revision
            && claim.attempt == entry.attempts
            && claim.attempt > 0
            && matches!(
                entry.state,
                assignment::State::Reserved | assignment::State::Running | assignment::State::Submitted
            )
            && matches!(entry.job.kind, assignment::Kind::Implement | assignment::Kind::Integrate)
            && !entry.stale
            && entry.pending_job.is_none(),
        "selected original claim is stale, unheld or mismatched"
    );
    ensure!(
        match (&entry.state, &entry.result) {
            (assignment::State::Submitted, Some(assignment::WorkResult::Submitted { .. })) =>
                entry.job.kind == assignment::Kind::Implement,
            (assignment::State::Reserved | assignment::State::Running, None) => true,
            _ => false,
        },
        "selected original state and result are inconsistent"
    );
    let baseline =
        entry.job.inputs.get("base-head").map(|value| resolve(&request.checkout, value, deadline)).transpose()?;
    let commit = match &entry.result {
        Some(assignment::WorkResult::Submitted { commit, .. }) => {
            ensure!(
                exact_commit(commit) && selected.source_commit.as_ref().is_none_or(|requested| requested == commit),
                "request changed the original submitted commit"
            );
            commit.clone()
        }
        Some(_) => anyhow::bail!("held migration result is not an original submission"),
        None if entry.job.kind == assignment::Kind::Integrate => selected
            .source_commit
            .clone()
            .context("running integration requires an explicit exact current source commit")?,
        None => {
            selected.source_commit.clone().or(baseline.clone()).context("implementation source baseline is missing")?
        }
    };
    ensure!(
        exact_commit(&commit) && resolve(&request.checkout, &commit, deadline)? == commit,
        "original source commit is unavailable"
    );
    if let Some(baseline) = &baseline {
        git(&request.checkout, &["merge-base", "--is-ancestor", baseline, &commit], 256, deadline)?;
    }
    let current_integration_head = entry.job.kind == assignment::Kind::Integrate && entry.result.is_none();
    if current_integration_head {
        let current = git(&request.checkout, &["rev-parse", "--verify", "HEAD^{commit}"], 256, deadline)?;
        ensure!(
            std::str::from_utf8(&current)?.trim() == commit,
            "explicit integration source is not the current committed HEAD"
        );
    }
    let files = entry
        .job
        .writable_paths
        .iter()
        .map(|path| {
            ensure!(relative(path), "original writable ownership contains an unsafe path");
            Ok(path.to_str().context("original writable path is not UTF-8")?.to_owned())
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(
        !files.is_empty() && files.len() <= 128 && files.iter().collect::<BTreeSet<_>>().len() == files.len(),
        "original writable file set is empty, duplicate or excessive"
    );
    Ok(SourcePlan {
        id: selected.id.clone(),
        source_commit: commit,
        resolved_baseline: baseline,
        current_integration_head,
        files,
    })
}

fn stop_evidence(
    request: &BuildRequest,
    entry: &assignment::Entry,
    stops: &[migration::OriginalOwnerStop],
    extent: &mut Extent,
) -> Result<()> {
    let matching = stops.iter().filter(|stop| entry.claim.as_ref() == Some(&stop.claim)).collect::<Vec<_>>();
    ensure!(matching.len() == 1, "migration builder requires one supplied exact original-owner observation per claim");
    let stop = matching[0];
    ensure!(
        stop.schema == "translation-original-owner-stop-v1"
            && stop.checkout == request.checkout
            && stop.effort == request.effort
            && stop.agent == entry.agent
            && !stop.observed_at.is_empty()
            && stop.observed_at.len() <= 128
            && matches!(stop.status.as_str(), "completed" | "failed" | "interrupted" | "not-launched")
            && !stop.evidence.is_empty()
            && stop.evidence.len() <= MAX_FILES
            && stop.processes.len() <= 128
            && (entry.agent.is_none() || stop.status != "not-launched"),
        "supplied original owner observation is ambiguous or mismatched"
    );
    for reference in &stop.evidence {
        extent.authenticate(reference)?;
    }
    if let Some(agent) = &entry.agent {
        let reference =
            stop.session.as_ref().context("started original owner lacks a supplied real session observation")?;
        extent.authenticate(reference)?;
        let session: assignment::authority::SessionObservation = object(&reference.path, extent.deadline)?;
        ensure!(
            session.schema == "decomp-session-observation-v1"
                && assignment::authority::real_session_id(&session.session_id)
                && (!assignment::authority::real_session_id(agent) || session.session_id == *agent)
                && session.status == stop.status
                && session.observed_at == stop.observed_at
                && !session.evidence.is_empty(),
            "supplied original session observation differs"
        );
        for artifact in &session.evidence {
            extent.artifact(artifact)?;
        }
    } else {
        ensure!(stop.session.is_none(), "unstarted original claim names another session");
    }
    for process in &stop.processes {
        extent.check()?;
        ensure!(
            process.identity.pid > 0
                && process.identity.start_ticks > 0
                && !process.identity.boot_id.is_empty()
                && matches!(process.status.as_str(), "exited" | "stopped" | "completed" | "failed" | "not-running"),
            "supplied original writer is running or ambiguous"
        );
        extent.authenticate(&process.observation)?;
        ensure!(
            operation::process_identity(process.identity.pid)?.as_ref() != Some(&process.identity),
            "exact original producer process remains live"
        );
    }
    Ok(())
}

fn historical(entry: &assignment::Entry, checkout: &Path, extent: &mut Extent) -> Result<()> {
    if let Some(result) = &entry.result {
        for artifact in assignment::result_artifacts(result) {
            extent.artifact(artifact)?;
        }
    }
    for artifact in &entry.observations {
        extent.artifact(artifact)?;
    }
    for prior in &entry.history {
        extent.check()?;
        if let Some(result) = &prior.result {
            for artifact in assignment::result_artifacts(result) {
                extent.artifact(artifact)?;
            }
        }
        for artifact in &prior.observations {
            extent.artifact(artifact)?;
        }
    }
    if let Some(ticket) = &entry.job.ticket {
        ensure!(relative(ticket), "original ticket path invalid");
        extent.file(&checkout.join(ticket), MAX_RECORD)?;
    }
    Ok(())
}

fn authenticate_captured(
    request: &BuildRequest,
    package: &Path,
    plans: &[SourcePlan],
    input: &CapturedInput,
    extent: &mut Extent,
) -> Result<()> {
    let plan = plans.iter().find(|plan| plan.id == input.claim_id).context("retained input names another claim")?;
    ensure!(
        digest_valid(&input.original_value) && input.reference.sha256 == input.original_value,
        "retained input changed its original declared digest"
    );
    let source = input.original_path.strip_prefix(&request.checkout).ok().filter(|path| relative(path));
    let retained = if let Some(source) = source {
        package.join("inputs").join(&plan.id).join("checkout").join(source)
    } else {
        package.join("inputs").join(&plan.id).join("external").join(sha256(input.key.as_bytes()))
    };
    ensure!(input.reference.path == retained, "retained input path differs from its original mapping");
    extent.authenticate(&input.reference)?;
    if let Some(commit) = &input.captured_from_commit {
        ensure!(
            commit == &plan.source_commit || plan.resolved_baseline.as_ref() == Some(commit),
            "retained input source is foreign to the original claim"
        );
        let source = source.context("external input cannot be captured from the original Git source")?;
        let bytes = blob(
            &request.checkout,
            commit,
            source.to_str().context("input source is not UTF-8")?,
            extent.limits.max_source_file_bytes,
            extent.deadline,
        )?;
        ensure!(sha256(&bytes) == input.original_value, "retained input differs from its original Git bytes");
    }
    Ok(())
}

fn capture_inputs(
    request: &BuildRequest,
    entry: &assignment::Entry,
    plan: &SourcePlan,
    package: &Path,
    extent: &mut Extent,
    captured: &mut Vec<CapturedInput>,
    blockers: &mut Vec<String>,
) -> Result<()> {
    ensure!(entry.job.inputs.len() <= 512, "original source input count excessive");
    for (key, expected) in &entry.job.inputs {
        extent.check()?;
        let path = if let Some(path) = key.strip_prefix("file:") {
            Some(PathBuf::from(path))
        } else if let Some(prefix) = key.strip_suffix("-sha256") {
            entry.job.inputs.get(&format!("{prefix}-file")).map(PathBuf::from)
        } else {
            None
        };
        let Some(path) = path else {
            continue;
        };
        ensure!(digest_valid(expected), "original source input digest invalid");
        let original_path = if path.is_absolute() {
            path.clone()
        } else {
            ensure!(relative(&path), "original source input path invalid");
            request.checkout.join(&path)
        };
        safe(&original_path)?;
        let decision_path =
            package.join("input-records").join(&plan.id).join(format!("{}.json", sha256(key.as_bytes())));
        if decision_path.try_exists()? {
            let decision: InputDecision = object(&decision_path, extent.deadline)?;
            ensure!(
                decision.schema == "translation-migration-input-decision-v1"
                    && decision.claim_id == plan.id
                    && decision.key == *key
                    && decision.original_value == *expected
                    && decision.original_path == original_path
                    && decision.captured.is_some() != decision.blocker.is_some(),
                "retained input decision belongs to another original input"
            );
            if let Some(input) = &decision.captured {
                ensure!(
                    input.claim_id == plan.id
                        && input.key == *key
                        && input.original_value == *expected
                        && input.original_path == original_path,
                    "retained input mapping changed"
                );
                authenticate_captured(request, package, std::slice::from_ref(plan), input, extent)?;
                captured.push(input.clone());
            }
            if let Some(blocker) = decision.blocker {
                ensure!(blocker.len() <= 8192, "retained input blocker is excessive");
                blockers.push(blocker);
            }
            extent.file(&decision_path, MAX_RECORD)?;
            continue;
        }
        let source_path = original_path.strip_prefix(&request.checkout).ok().filter(|path| relative(path));
        let snapshot = if let Some(source_path) = source_path {
            package.join("inputs").join(&plan.id).join("checkout").join(source_path)
        } else {
            package.join("inputs").join(&plan.id).join("external").join(sha256(key.as_bytes()))
        };
        let retained = snapshot
            .try_exists()?
            .then(|| read_bytes(&snapshot, extent.limits.max_source_file_bytes, extent.deadline))
            .transpose()?;
        let present = if retained.is_none() {
            read_bytes(&original_path, extent.limits.max_source_file_bytes, extent.deadline).ok()
        } else {
            None
        };
        let mut recovered = if let Some(bytes) = retained {
            ensure!(sha256(&bytes) == *expected, "interrupted input capture differs from its declared digest");
            Some((bytes, None))
        } else {
            present.filter(|bytes| sha256(bytes) == *expected).map(|bytes| (bytes, None))
        };
        if recovered.is_none()
            && let Some(source_path) = source_path
        {
            let name = source_path.to_str().context("original source input is not UTF-8")?;
            for commit in std::iter::once(&plan.source_commit).chain(plan.resolved_baseline.iter()) {
                extent.check()?;
                if let Ok(bytes) =
                    blob(&request.checkout, commit, name, extent.limits.max_source_file_bytes, extent.deadline)
                    && sha256(&bytes) == *expected
                {
                    recovered = Some((bytes, Some(commit.clone())));
                    break;
                }
            }
        }
        let (input, blocker) = if let Some((bytes, commit)) = recovered {
            let reference = extent.publish(&snapshot, &bytes)?;
            let input = CapturedInput {
                claim_id: plan.id.clone(),
                key: key.clone(),
                original_value: expected.clone(),
                original_path: original_path.clone(),
                reference,
                captured_from_commit: commit,
            };
            captured.push(input.clone());
            (Some(input), None)
        } else {
            let blocker = format!(
                "Original claim {} input {key} has no authenticatable retained bytes for its unchanged declared digest.",
                plan.id
            );
            blockers.push(blocker.clone());
            (None, Some(blocker))
        };
        extent.record(
            &decision_path,
            &InputDecision {
                schema: "translation-migration-input-decision-v1".into(),
                claim_id: plan.id.clone(),
                key: key.clone(),
                original_value: expected.clone(),
                original_path,
                captured: input,
                blocker,
            },
        )?;
    }
    Ok(())
}

fn allowed_output() -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("architecture worktree root missing")?;
    Ok(root.join(".scratch/translation-scope-execution/production-preparation"))
}

pub(crate) fn build_from_request(path: &Path) -> Result<BuildReceipt> {
    private(path)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    let reference = seal(path, MAX_RECORD, deadline)?;
    let request: BuildRequest = object(path, deadline)?;
    build_inner(request, Some(reference), &allowed_output()?, true)
}

pub(crate) fn build_preparation(request: BuildRequest) -> Result<BuildReceipt> {
    build_inner(request, None, &allowed_output()?, true)
}

fn validation(package: &Path, extent: &Extent) -> Result<Vec<u8>> {
    let mut command = Command::new(std::env::current_exe()?);
    command.args(["translation-migration", "validate-preparation", "--migration"]).arg(package.join("migration.json"));
    command_output(command, MAX_RECORD, extent.deadline)
}

fn validation_decision(
    package: &Path,
    migration: &Reference,
    extent: &mut Extent,
    validate: bool,
) -> Result<ValidationDecision> {
    let path = package.join("validation.json");
    if path.try_exists()? {
        let decision: ValidationDecision = object(&path, extent.deadline)?;
        ensure!(
            decision.schema == "translation-migration-validation-decision-v1" && decision.migration == *migration,
            "retained validation belongs to another migration"
        );
        ensure!(
            decision.status.len() <= 128
                && decision.blockers.len() <= 512
                && decision.blockers.iter().all(|blocker| blocker.len() <= 8192),
            "retained validation exceeds its bounds"
        );
        if let Some(report) = &decision.report {
            ensure!(report.path == package.join("preparation-report.toml"), "retained validation report path changed");
            extent.authenticate(report)?;
            report_identity(&read_bytes(&report.path, MAX_RECORD, extent.deadline)?, migration, &decision.status)?;
        } else {
            ensure!(
                matches!(decision.status.as_str(), "validation-refused" | "validation-not-run"),
                "retained validation lacks its actual report"
            );
        }
        extent.file(&path, MAX_RECORD)?;
        return Ok(decision);
    }
    let mut blockers = Vec::new();
    let report_path = package.join("preparation-report.toml");
    let (status, report) = if report_path.try_exists()? {
        let bytes = read_bytes(&report_path, MAX_RECORD, extent.deadline)?;
        let status = report_status(&bytes, migration, &mut blockers)?;
        (status, Some(extent.publish(&report_path, &bytes)?))
    } else if validate {
        match validation(package, extent) {
            Ok(bytes) => {
                let status = report_status(&bytes, migration, &mut blockers)?;
                (status, Some(extent.publish(&report_path, &bytes)?))
            }
            Err(error) => {
                extent.check()?;
                blockers.push(format!(
                    "Existing read-only preparation validation refused this package: {}",
                    bounded(&format!("{error:#}"), 4096)
                ));
                ("validation-refused".into(), None)
            }
        }
    } else {
        blockers.push(
            "Existing preparation validator was not run; this captured package has no validation acceptance.".into(),
        );
        ("validation-not-run".into(), None)
    };
    let decision = ValidationDecision {
        schema: "translation-migration-validation-decision-v1".into(),
        migration: migration.clone(),
        status,
        report,
        blockers,
    };
    extent.record(&path, &decision)?;
    Ok(decision)
}

fn report_identity(bytes: &[u8], migration: &Reference, status: &str) -> Result<toml::Value> {
    let report: toml::Value = toml::from_str(std::str::from_utf8(bytes)?)?;
    ensure!(
        report.get("schema").and_then(toml::Value::as_str) == Some("translation-migration-preparation-v1")
            && report.get("status").and_then(toml::Value::as_str) == Some(status)
            && report.get("activated").and_then(toml::Value::as_bool) == Some(false)
            && report.get("migration_sha256").and_then(toml::Value::as_str) == Some(migration.sha256.as_str()),
        "migration validator report has another identity or activation state"
    );
    Ok(report)
}

fn report_status(bytes: &[u8], migration: &Reference, blockers: &mut Vec<String>) -> Result<String> {
    let report: toml::Value = toml::from_str(std::str::from_utf8(bytes)?)?;
    let status = report.get("status").and_then(toml::Value::as_str).context("migration report has no status")?;
    ensure!(status.len() <= 128, "migration report status is excessive");
    report_identity(bytes, migration, status)?;
    if let Some(found) = report.get("blockers").and_then(toml::Value::as_array) {
        ensure!(found.len() <= 512, "migration report blocker count excessive");
        for blocker in found {
            let text = blocker.as_str().context("migration report blocker is not text")?;
            ensure!(text.len() <= 8192, "migration report blocker is excessive");
            blockers.push(text.to_owned());
        }
    }
    Ok(status.to_owned())
}

fn build_inner(
    mut request: BuildRequest,
    request_reference: Option<Reference>,
    allowed: &Path,
    validate: bool,
) -> Result<BuildReceipt> {
    let mut extent = Extent::new(&request.limits)?;
    ensure!(
        request.schema == "translation-migration-build-v1" && name(&request.effort),
        "migration builder request identity invalid"
    );
    ensure!(
        !request.claims.is_empty()
            && request.claims.len() <= 128
            && request.claims.len() == request.stopped_owners.len(),
        "migration builder needs exact claims and one supplied owner observation each"
    );
    ensure!(
        request
            .claims
            .iter()
            .all(|claim| name(&claim.id) && claim.source_commit.as_ref().is_none_or(|commit| exact_commit(commit))),
        "migration builder selected claim/source invalid"
    );
    ensure!(
        request.claims.iter().map(|claim| &claim.id).collect::<BTreeSet<_>>().len() == request.claims.len(),
        "migration builder selected claim duplicated"
    );
    safe(&request.checkout)?;
    safe(&request.output_root)?;
    safe(allowed)?;
    ensure!(
        request.output_root.starts_with(allowed)
            && request.output_root != *allowed
            && !request.output_root.starts_with(&request.checkout),
        "migration evidence must remain in the isolated architecture output root"
    );
    let common = git(&request.checkout, &["rev-parse", "--git-common-dir"], 8192, extent.deadline)?;
    let common = Path::new(std::str::from_utf8(&common)?.trim());
    let common = if common.is_absolute() { common.to_path_buf() } else { request.checkout.join(common) };
    safe(&common)?;
    ensure!(common == request.checkout.join(".git"), "migration builder checkout is not canonical main");
    request.claims.sort_by(|left, right| left.id.cmp(&right.id));
    request.stopped_owners.sort_by(|left, right| left.path.cmp(&right.path));
    let request_sha256 = sealed(&request)?;
    if let Some(reference) = &request_reference {
        extent.authenticate(reference)?;
    }
    let authority_reference = extent.file(&request.checkout.join(AUTHORITY), MAX_RECORD)?;
    let authority: Authority = object(&authority_reference.path, extent.deadline)?;
    ensure!(
        authority.schema == "decomp-session-authority-v1"
            && authority.state == "active"
            && (authority.preparation.canonical_effort == request.effort
                || authority.preparation.admitted_efforts.contains(&request.effort))
            && !authority.preparation.drill_efforts.contains(&request.effort),
        "migration builder needs current active authority for the original effort"
    );
    let roles = authority
        .roles
        .values()
        .filter(|role| role.kind == assignment::authority::RoleKind::Integrator)
        .collect::<Vec<_>>();
    ensure!(roles.len() == 1, "migration builder needs one exact current integrator");
    let role = roles[0];
    ensure!(
        name(&role.token) && assignment::authority::real_session_id(&role.session_id) && role.generation > 0,
        "current integrator role is incomplete"
    );
    extent.artifact(&authority.preparation.bridge)?;
    extent.artifact(&role.registration)?;
    for transfer in &role.transfers {
        extent.artifact(transfer)?;
    }
    if let Some(transfer) = &role.last_transfer {
        extent.artifact(&transfer.observation)?;
    }
    extent.artifact(authority.drill.as_ref().context("current authority activation drill is missing")?)?;
    let ledger_reference =
        extent.file(&request.checkout.join(format!(".scratch/{}/ledger.toml", request.effort)), MAX_RECORD)?;
    let ledger: Ledger = object(&ledger_reference.path, extent.deadline)?;
    ensure!(
        ledger.schema == "decomp-assignment-v1"
            && (1..=64).contains(&ledger.capacity)
            && ledger.jobs.iter().all(|(id, entry)| id == &entry.job.id),
        "original assignment ledger invalid"
    );
    let plans = request
        .claims
        .iter()
        .map(|claim| {
            source_plan(
                &request,
                claim,
                ledger.jobs.get(&claim.id).context("selected original claim is absent")?,
                extent.deadline,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let mut stops = Vec::new();
    for reference in &request.stopped_owners {
        extent.authenticate(reference)?;
        stops.push(object::<migration::OriginalOwnerStop>(&reference.path, extent.deadline)?);
    }
    for plan in &plans {
        let entry = &ledger.jobs[&plan.id];
        stop_evidence(&request, entry, &stops, &mut extent)?;
        historical(entry, &request.checkout, &mut extent)?;
    }
    extent.authenticate(&request.proof_installation)?;
    extent.authenticate(&request.proof_completion)?;
    let package = request.output_root.join("package");
    directory(&package)?;
    let lock_path = package.join("builder.lock");
    let lock =
        OpenOptions::new().create(true).append(true).mode(0o600).custom_flags(libc::O_NOFOLLOW).open(&lock_path)?;
    private(&lock_path)?;
    lock.try_lock().context("migration package already has a live builder")?;
    extent.file(&lock_path, 0)?;
    let package_files = package_files(&package, &extent)?;
    let intent_path = package.join("build-intent.json");
    if intent_path.try_exists()? {
        authenticate_intent(
            &request,
            &request_reference,
            &package,
            &authority_reference,
            &ledger_reference,
            &plans,
            &extent,
        )?;
        extent.file(&intent_path, MAX_RECORD)?;
        if package.join("build-receipt.json").try_exists()? {
            return resume_package(&request, &package, &plans, &mut extent);
        }
    } else {
        ensure!(
            package_files.keys().all(|path| path == &lock_path || *path == pending_path(&intent_path)),
            "migration package lacks an authenticated intent for existing files"
        );
    }
    for reference in package_files.into_values() {
        extent.remember(reference)?;
    }
    extent.record(
        &intent_path,
        &Intent {
            schema: "translation-migration-build-intent-v1",
            request: &request,
            request_reference: &request_reference,
            authority: &authority_reference,
            ledger: &ledger_reference,
            sources: &plans,
        },
    )?;
    let mut blockers = Vec::new();
    let mut inputs = Vec::new();
    let mut claims = Vec::new();
    let historical_start = extent.references.keys().cloned().collect::<BTreeSet<_>>();
    for plan in &plans {
        let entry = &ledger.jobs[&plan.id];
        let source_checkout = package.join("source").join(&plan.id);
        let mut source_files = BTreeMap::new();
        for name in &plan.files {
            let bytes = blob(
                &request.checkout,
                &plan.source_commit,
                name,
                extent.limits.max_source_file_bytes,
                extent.deadline,
            )?;
            let reference = extent.publish(&source_checkout.join(name), &bytes)?;
            source_files.insert(name.clone(), reference.sha256);
        }
        capture_inputs(&request, entry, plan, &package, &mut extent, &mut inputs, &mut blockers)?;
        let original = migration::OriginalClaim {
            schema: "translation-original-claim-v1".into(),
            checkout: request.checkout.clone(),
            effort: request.effort.clone(),
            job: entry.job.clone(),
            claim: entry.claim.clone().context("held original claim missing")?,
            state: entry.state.clone(),
            agent: entry.agent.clone(),
            result: entry.result.clone(),
            observations: entry.observations.clone(),
            source_checkout,
            source_commit: plan.source_commit.clone(),
            source_files,
        };
        let reference = extent.record(&package.join("originals").join(format!("{}.json", plan.id)), &original)?;
        claims.push(migration::MigratedClaim {
            claim: ClaimLineage {
                id: original.claim.id,
                revision: original.claim.revision,
                attempt: original.claim.attempt,
                coordinator: original.claim.coordinator,
            },
            scope_id: format!("original-held-{}", plan.id),
            policy_sha256: "0".repeat(64),
            source_commit: plan.source_commit.clone(),
            writable_files: entry.job.writable_paths.clone(),
            original_claim: reference,
            installation: None,
            execution: None,
        });
        blockers.push(format!("Original claim {} has no reviewed production installation or policy; its source, inputs and held lineage are preserved.", plan.id));
    }
    for (index, left) in claims.iter().enumerate() {
        for right in &claims[index + 1..] {
            if left
                .writable_files
                .iter()
                .any(|a| right.writable_files.iter().any(|b| a.starts_with(b) || b.starts_with(a)))
            {
                blockers.push(format!("Original held claims {} and {} share writable ownership; preparation preserves both and grants no concurrent dispatch.", left.claim.id, right.claim.id));
            }
        }
    }
    let proof = extent.record(
        &package.join("representative-proof.json"),
        &migration::RetainedProof {
            schema: "translation-supervisor-proof-v1".into(),
            installation: request.proof_installation.clone(),
            completion: request.proof_completion.clone(),
        },
    )?;
    let historical = extent
        .references
        .values()
        .filter(|reference| {
            historical_start.contains(&reference.path)
                || inputs.iter().any(|input| input.reference.path == reference.path)
        })
        .filter(|reference| reference.path != lock_path && reference.path != package.join("build-intent.json"))
        .cloned()
        .collect::<Vec<_>>();
    let migration = migration::Migration { schema: "translation-execution-migration-v1".into(), state: "prepared".into(), checkout: request.checkout.clone(), effort: request.effort.clone(),
        coordinator: role.token.clone(), session_id: role.session_id.clone(), generation: role.generation, authority: authority_reference, ledger: ledger_reference,
        claims, historical, stopped_owners: request.stopped_owners.clone(), proof,
        limitations: vec!["Read-only preparation preserves original claims, results, observations and legacy inputs; production remains unchanged.".into(),
            "No production installation or policy is prepared. Zero policy digests are explicit unavailable-policy sentinels, never acceptance seals.".into(),
            "Representative proof authenticates only its original isolated scope. Production activation requires separate reviewed approval and current authority.".into()] };
    let migration_reference = extent.record(&package.join("migration.json"), &migration)?;
    let decision = validation_decision(&package, &migration_reference, &mut extent, validate)?;
    let validation_status = decision.status;
    let preparation_report = decision.report;
    blockers.extend(decision.blockers);
    blockers.sort();
    blockers.dedup();
    ensure!(
        blockers.len() <= 512 && blockers.iter().all(|text| text.len() <= 8192),
        "migration blocker report exceeds its bound"
    );
    for plan in &plans {
        ensure!(
            resolve(&request.checkout, &plan.source_commit, extent.deadline)? == plan.source_commit,
            "original source object changed before return"
        );
        if plan.current_integration_head {
            let head = git(&request.checkout, &["rev-parse", "--verify", "HEAD^{commit}"], 256, extent.deadline)?;
            ensure!(
                std::str::from_utf8(&head)?.trim() == plan.source_commit,
                "current integration HEAD changed during capture"
            );
        }
        for name in &plan.files {
            let bytes = blob(
                &request.checkout,
                &plan.source_commit,
                name,
                extent.limits.max_source_file_bytes,
                extent.deadline,
            )?;
            let reference = &extent.references[&package.join("source").join(&plan.id).join(name)];
            ensure!(
                sha256(&bytes) == reference.sha256,
                "captured source differs from its original commit before return"
            );
        }
        stop_evidence(&request, &ledger.jobs[&plan.id], &stops, &mut extent)?;
    }
    extent.recheck()?;
    ensure!(
        extent.unadmitted.keys().all(|path| *path == package.join("inventory.json")
            || *path == pending_path(&package.join("inventory.json"))
            || *path == pending_path(&package.join("build-receipt.json"))),
        "interrupted migration package contains a foreign output"
    );
    let inventory_bytes = serde_json::to_vec(&Inventory {
        schema: "translation-migration-build-inventory-v1",
        request_sha256: &request_sha256,
        sources: &plans,
        inputs: &inputs,
        references: extent.references.values().collect(),
        validation_status: &validation_status,
        blockers: &blockers,
    })?;
    ensure!(inventory_bytes.len() as u64 <= MAX_RECORD, "migration inventory exceeds its fixed record bound");
    let inventory = extent.publish(&package.join("inventory.json"), &inventory_bytes)?;
    let mut receipt = BuildReceipt {
        schema: "translation-migration-build-receipt-v1".into(),
        status: "prepared-with-blockers".into(),
        activated: false,
        request_sha256,
        migration: migration_reference,
        inventory,
        preparation_report,
        validation_status,
        retained_bytes: extent.bytes,
        referenced_files: extent.references.len() + 1,
        blockers,
    };
    for _ in 0..32 {
        let bytes = serde_json::to_vec(&receipt)?;
        let total = extent.bytes.checked_add(bytes.len() as u64).context("migration receipt extent overflow")?;
        if total == receipt.retained_bytes {
            extent.publish(&package.join("build-receipt.json"), &bytes)?;
            ensure!(extent.unadmitted.is_empty(), "migration package retains an unrecognized output");
            authenticate_package(&package, &extent)?;
            extent.recheck()?;
            return Ok(receipt);
        }
        receipt.retained_bytes = total;
    }
    anyhow::bail!("migration receipt byte accounting did not converge")
}

fn bounded(value: &str, maximum: usize) -> String {
    let mut end = value.len().min(maximum);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        os::unix::fs::symlink,
        sync::atomic::{AtomicU64, Ordering},
    };

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const AGENT: &str = "01234567-89ab-cdef-0123-456789abcdef";
    const SESSION: &str = "abcdef01-2345-6789-abcd-ef0123456789";
    const ORIGINAL: &str = "pub fn value() -> u32 { 42 }\n";
    const INPUT: &str = "frozen = true\n";

    struct Fixture {
        root: PathBuf,
        allowed: PathBuf,
        request: BuildRequest,
        entries: BTreeMap<String, assignment::Entry>,
        baseline: String,
        submitted: String,
    }

    fn deadline() -> Instant {
        Instant::now() + Duration::from_secs(10)
    }

    fn test_git(checkout: &Path, args: &[&str]) -> String {
        String::from_utf8(git(checkout, args, MAX_RECORD, deadline()).unwrap()).unwrap().trim().to_owned()
    }

    fn commit(checkout: &Path, message: &str) -> String {
        test_git(
            checkout,
            &["-c", "user.name=builder fixture", "-c", "user.email=builder@localhost", "commit", "-q", "-m", message],
        );
        test_git(checkout, &["rev-parse", "HEAD"])
    }

    fn write(path: &Path, bytes: &[u8]) {
        directory(path.parent().unwrap()).unwrap();
        if path.try_exists().unwrap() {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
        }
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    fn reference(path: &Path) -> Reference {
        seal(path, MAX_RECORD, deadline()).unwrap()
    }

    fn artifact(reference: &Reference) -> assignment::Artifact {
        assignment::Artifact { path: reference.path.clone(), sha256: reference.sha256.clone() }
    }

    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "sm-migration-builder-tests-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            let checkout = root.join("checkout");
            let allowed = root.join("architecture");
            directory(&checkout).unwrap();
            directory(&allowed).unwrap();
            write(&checkout.join("src/domain.rs"), b"pub fn value() -> u32 { 7 }\n");
            write(&checkout.join("tools/frozen.toml"), INPUT.as_bytes());
            test_git(&checkout, &["init", "-q"]);
            test_git(&checkout, &["add", "src/domain.rs", "tools/frozen.toml"]);
            let baseline = commit(&checkout, "baseline");
            write(&checkout.join("src/domain.rs"), ORIGINAL.as_bytes());
            test_git(&checkout, &["add", "src/domain.rs"]);
            let submitted = commit(&checkout, "original submission");
            write(&checkout.join(".git/info/exclude"), b"/.scratch/\n");
            let evidence_path = root.join("observations/evidence.txt");
            write(&evidence_path, b"Typed synthetic fixture observation; no production owner is asserted.\n");
            let evidence = reference(&evidence_path);
            let proof_installation_path = root.join("proof/installation.json");
            let proof_completion_path = root.join("proof/completion.json");
            write(&proof_installation_path, b"{}");
            write(&proof_completion_path, b"{}");
            let claim = assignment::Claim {
                id: "owned-subsystem".into(),
                revision: "legacy-v1".into(),
                attempt: 1,
                coordinator: "original-owner".into(),
            };
            let job = assignment::Job {
                id: claim.id.clone(),
                revision: claim.revision.clone(),
                kind: assignment::Kind::Implement,
                subject: "Implement whole subsystem".into(),
                writable_paths: vec!["src/domain.rs".into()],
                blockers: vec![],
                inputs: BTreeMap::from([
                    ("base-head".into(), baseline[..7].to_owned()),
                    ("file:tools/frozen.toml".into(), sha256(INPUT.as_bytes())),
                ]),
                instructions: "Original frozen job instructions".into(),
                ticket: Some(".scratch/owned-effort/issues/01-subsystem.md".into()),
            };
            write(&checkout.join(job.ticket.as_ref().unwrap()), b"Status: claimed\nOriginal ticket.\n");
            let entry: assignment::Entry = serde_json::from_value(serde_json::json!({
                "job": job, "claim": claim, "state": "submitted", "agent": AGENT, "pending_job": null, "stale": false,
                "attempts": 1, "result": {"result": "submitted", "commit": submitted, "evidence": [artifact(&evidence)]},
                "observations": [artifact(&evidence)], "history": [], "last_update": null,
                "adopted_ticket": false, "projection_pending": false
            })).unwrap();
            let authority = serde_json::json!({
                "schema": "decomp-session-authority-v1", "state": "active", "prepared_head": baseline,
                "preparation": {"canonical_effort": "decomp-parallel", "capacity": 3,
                    "admitted_efforts": ["owned-effort"], "drill_efforts": ["private-drill"], "bridge": artifact(&evidence),
                    "integrator": {"token": "current-integrator", "kind": "integrator", "session_id": SESSION, "observation": artifact(&evidence)}},
                "roles": {"current-integrator": {"token": "current-integrator", "kind": "integrator", "session_id": SESSION,
                    "generation": 2, "registration": artifact(&evidence), "transfers": [], "last_transfer": null}},
                "drill": artifact(&evidence)
            });
            let authority: Authority = serde_json::from_value(authority).unwrap();
            write(&checkout.join(AUTHORITY), toml::to_string(&authority).unwrap().as_bytes());
            let request = BuildRequest {
                schema: "translation-migration-build-v1".into(),
                checkout,
                effort: "owned-effort".into(),
                claims: vec![ClaimRequest { id: claim.id.clone(), source_commit: None }],
                stopped_owners: vec![],
                proof_installation: reference(&proof_installation_path),
                proof_completion: reference(&proof_completion_path),
                output_root: allowed.join("package-output"),
                limits: BuildLimits::default(),
            };
            let mut fixture =
                Self { root, allowed, request, entries: BTreeMap::from([(claim.id, entry)]), baseline, submitted };
            fixture.write_ledger();
            fixture.add_stop("owned-subsystem");
            fixture
        }

        fn write_ledger(&self) {
            let ledger = Ledger { schema: "decomp-assignment-v1".into(), capacity: 3, jobs: self.entries.clone() };
            write(
                &self.request.checkout.join(".scratch/owned-effort/ledger.toml"),
                toml::to_string(&ledger).unwrap().as_bytes(),
            );
        }

        fn add_stop(&mut self, id: &str) {
            let entry = &self.entries[id];
            let evidence = reference(&self.root.join("observations/evidence.txt"));
            let session_path = self.root.join("observations").join(format!("{id}-session.json"));
            let session = assignment::authority::SessionObservation {
                schema: "decomp-session-observation-v1".into(),
                session_id: entry.agent.clone().unwrap(),
                status: "interrupted".into(),
                observed_at: "2026-10-13 20:00:00 UTC".into(),
                evidence: vec![artifact(&evidence)],
            };
            write(&session_path, &serde_json::to_vec(&session).unwrap());
            let stop = migration::OriginalOwnerStop {
                schema: "translation-original-owner-stop-v1".into(),
                checkout: self.request.checkout.clone(),
                effort: self.request.effort.clone(),
                claim: entry.claim.clone().unwrap(),
                agent: entry.agent.clone(),
                observed_at: session.observed_at,
                status: session.status,
                session: Some(reference(&session_path)),
                evidence: vec![evidence],
                processes: vec![],
            };
            let stop_path = self.root.join("observations").join(format!("{id}-stop.json"));
            write(&stop_path, &serde_json::to_vec(&stop).unwrap());
            self.request.stopped_owners.push(reference(&stop_path));
        }

        fn add_integration(&mut self) {
            let mut entry = self.entries["owned-subsystem"].clone();
            entry.job.id = "integrate-owned-subsystem".into();
            entry.job.revision = "legacy-v1-1".into();
            entry.job.kind = assignment::Kind::Integrate;
            entry.job.subject = format!("Integrate {}", entry.job.subject);
            entry.job.ticket = None;
            entry.job.inputs = BTreeMap::from([
                ("submission-id".into(), "owned-subsystem".into()),
                ("submission-revision".into(), "legacy-v1".into()),
                ("submission-attempt".into(), "1".into()),
            ]);
            entry.state = assignment::State::Running;
            entry.result = None;
            entry.observations.clear();
            entry.claim = Some(assignment::Claim {
                id: entry.job.id.clone(),
                revision: entry.job.revision.clone(),
                attempt: 1,
                coordinator: "original-owner".into(),
            });
            let id = entry.job.id.clone();
            self.entries.insert(id.clone(), entry);
            self.request.claims.push(ClaimRequest { id: id.clone(), source_commit: Some(self.submitted.clone()) });
            self.write_ledger();
            self.add_stop(&id);
        }

        fn build(&self) -> Result<BuildReceipt> {
            build_inner(self.request.clone(), None, &self.allowed, false)
        }
        fn package(&self) -> PathBuf {
            self.request.output_root.join("package")
        }
        fn snapshot(&self) -> BTreeMap<PathBuf, Vec<u8>> {
            package_files(&self.package(), &Extent::new(&self.request.limits).unwrap())
                .unwrap()
                .into_keys()
                .map(|path| {
                    let bytes = fs::read(&path).unwrap();
                    (path, bytes)
                })
                .collect()
        }
        fn main_bytes(&self) -> Vec<Vec<u8>> {
            [AUTHORITY, ".scratch/owned-effort/ledger.toml", ".git/index", "src/domain.rs", "tools/frozen.toml"]
                .into_iter()
                .map(|path| fs::read(self.request.checkout.join(path)).unwrap())
                .collect()
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn submitted_capture_preserves_original_job_and_reads_commit_despite_dirty_main() {
        let fixture = Fixture::new();
        write(&fixture.request.checkout.join("src/domain.rs"), b"dirty source outside original commit\n");
        write(&fixture.request.checkout.join("tools/frozen.toml"), b"dirty verification declaration\n");
        let before = fixture.main_bytes();
        let receipt = fixture.build().unwrap();
        assert!(!receipt.activated);
        assert_eq!(receipt.status, "prepared-with-blockers");
        assert_eq!(receipt.validation_status, "validation-not-run");
        assert!(receipt.preparation_report.is_none());
        let original: migration::OriginalClaim =
            object(&fixture.package().join("originals/owned-subsystem.json"), deadline()).unwrap();
        let entry = &fixture.entries["owned-subsystem"];
        assert_eq!(original.job, entry.job);
        assert_eq!(original.claim, entry.claim.clone().unwrap());
        assert_eq!(original.result, entry.result);
        assert_eq!(original.observations, entry.observations);
        assert_eq!(original.source_commit, fixture.submitted);
        assert_eq!(original.job.inputs["base-head"], fixture.baseline[..7]);
        assert_eq!(fs::read(original.source_checkout.join("src/domain.rs")).unwrap(), ORIGINAL.as_bytes());
        assert!(!original.source_checkout.join(".git").exists());
        let inventory: SavedInventory = object(&receipt.inventory.path, deadline()).unwrap();
        assert_eq!(inventory.sources[0].resolved_baseline.as_ref(), Some(&fixture.baseline));
        assert_eq!(inventory.inputs.len(), 1);
        assert_eq!(fs::read(&inventory.inputs[0].reference.path).unwrap(), INPUT.as_bytes());
        assert_eq!(inventory.inputs[0].original_value, entry.job.inputs["file:tools/frozen.toml"]);
        assert_eq!(inventory.inputs[0].captured_from_commit.as_ref(), Some(&fixture.submitted));
        assert_eq!(fixture.main_bytes(), before);
    }

    #[test]
    fn identical_and_interrupted_retries_preserve_all_package_bytes() {
        let fixture = Fixture::new();
        write(&fixture.request.checkout.join("tools/frozen.toml"), b"dirty before original capture\n");
        let receipt = fixture.build().unwrap();
        let before = fixture.snapshot();
        write(&fixture.request.checkout.join("tools/frozen.toml"), b"different dirty bytes after capture\n");
        assert_eq!(fixture.build().unwrap(), receipt);
        assert_eq!(fixture.snapshot(), before);
        let receipt_path = fixture.package().join("build-receipt.json");
        fs::remove_file(&receipt_path).unwrap();
        write(&pending_path(&receipt_path), &before[&receipt_path][..73]);
        let source_path = fixture.package().join("source/owned-subsystem/src/domain.rs");
        fs::remove_file(&source_path).unwrap();
        write(&pending_path(&source_path), &ORIGINAL.as_bytes()[..9]);
        let inventory_path = fixture.package().join("inventory.json");
        fs::remove_file(&inventory_path).unwrap();
        write(&pending_path(&inventory_path), &before[&inventory_path][..89]);
        assert_eq!(fixture.build().unwrap(), receipt);
        assert_eq!(fixture.snapshot(), before);
    }

    #[test]
    fn completed_or_interrupted_packages_reject_foreign_files_and_tamper() {
        for complete in [true, false] {
            let fixture = Fixture::new();
            fixture.build().unwrap();
            if !complete {
                fs::remove_file(fixture.package().join("build-receipt.json")).unwrap();
            }
            let foreign = fixture.package().join("foreign.bin");
            write(&foreign, b"unaccounted output");
            let before = fixture.main_bytes();
            assert!(fixture.build().is_err());
            assert_eq!(fixture.main_bytes(), before);
            assert!(foreign.exists());
        }
        let fixture = Fixture::new();
        fixture.build().unwrap();
        write(&fixture.package().join("source/owned-subsystem/src/domain.rs"), b"pub fn value() -> u32 { 99 }\n");
        assert!(fixture.build().is_err());
    }

    #[test]
    fn request_authority_and_ledger_changes_cannot_resume_the_previous_package() {
        let fixture = Fixture::new();
        fixture.build().unwrap();
        let before = fixture.snapshot();
        let mut changed = fixture.request.clone();
        changed.limits.timeout_seconds += 1;
        assert!(build_inner(changed, None, &fixture.allowed, false).is_err());
        let authority_path = fixture.request.checkout.join(AUTHORITY);
        let authority = fs::read(&authority_path).unwrap();
        write(
            &authority_path,
            &String::from_utf8(authority.clone()).unwrap().replace("generation = 2", "generation = 3").into_bytes(),
        );
        assert!(fixture.build().is_err());
        write(&authority_path, &authority);
        let ledger_path = fixture.request.checkout.join(".scratch/owned-effort/ledger.toml");
        let ledger = fs::read(&ledger_path).unwrap();
        write(
            &ledger_path,
            &String::from_utf8(ledger.clone()).unwrap().replacen("capacity = 3", "capacity = 4", 1).into_bytes(),
        );
        assert!(fixture.build().is_err());
        assert_eq!(fixture.snapshot(), before);
    }

    #[test]
    fn linked_integration_preserves_original_inputs_and_requires_exact_current_head() {
        let mut fixture = Fixture::new();
        fixture.add_integration();
        let mut missing = fixture.request.clone();
        missing.claims[1].source_commit = None;
        assert!(build_inner(missing, None, &fixture.allowed, false).is_err());
        let mut stale = fixture.request.clone();
        stale.claims[1].source_commit = Some(fixture.baseline.clone());
        assert!(build_inner(stale, None, &fixture.allowed, false).is_err());
        let before = fixture.main_bytes();
        let receipt = fixture.build().unwrap();
        let migration: migration::Migration = object(&receipt.migration.path, deadline()).unwrap();
        assert_eq!(migration.claims.len(), 2);
        assert!(receipt.blockers.iter().any(|blocker| blocker.contains("share writable ownership")));
        let original: migration::OriginalClaim =
            object(&fixture.package().join("originals/integrate-owned-subsystem.json"), deadline()).unwrap();
        assert_eq!(original.job, fixture.entries["integrate-owned-subsystem"].job);
        assert_eq!(original.job.inputs.len(), 3);
        assert!(!original.job.inputs.contains_key("base-head"));
        assert!(original.result.is_none());
        assert_eq!(original.source_commit, fixture.submitted);
        assert_eq!(fixture.main_bytes(), before);
        write(&fixture.request.checkout.join("src/next.rs"), b"pub fn next() {}\n");
        test_git(&fixture.request.checkout, &["add", "src/next.rs"]);
        commit(&fixture.request.checkout, "later main commit");
        assert!(fixture.build().is_err());
    }

    #[test]
    fn foreign_submission_and_git_symlink_mode_are_rejected() {
        let fixture = Fixture::new();
        let mut changed = fixture.request.clone();
        changed.claims[0].source_commit = Some(fixture.baseline.clone());
        assert!(build_inner(changed, None, &fixture.allowed, false).is_err());
        let mut entry = fixture.entries["owned-subsystem"].clone();
        entry.result = Some(assignment::WorkResult::Submitted { commit: "f".repeat(40), evidence: vec![] });
        assert!(source_plan(&fixture.request, &fixture.request.claims[0], &entry, deadline()).is_err());
        let mut fixture = fixture;
        fs::remove_file(fixture.request.checkout.join("src/domain.rs")).unwrap();
        symlink("../tools/frozen.toml", fixture.request.checkout.join("src/domain.rs")).unwrap();
        test_git(&fixture.request.checkout, &["add", "src/domain.rs"]);
        let commit = commit(&fixture.request.checkout, "unsafe symlink source");
        fixture.entries.get_mut("owned-subsystem").unwrap().result =
            Some(assignment::WorkResult::Submitted { commit, evidence: vec![] });
        fixture.write_ledger();
        assert!(fixture.build().is_err());
        assert!(!fixture.package().join("source/owned-subsystem/src/domain.rs").exists());
    }

    #[test]
    fn owner_observations_reject_foreign_session_and_exact_live_kernel_process() {
        let fixture = Fixture::new();
        let entry = &fixture.entries["owned-subsystem"];
        let stop: migration::OriginalOwnerStop = object(&fixture.request.stopped_owners[0].path, deadline()).unwrap();
        assert!(
            stop_evidence(
                &fixture.request,
                entry,
                std::slice::from_ref(&stop),
                &mut Extent::new(&fixture.request.limits).unwrap()
            )
            .is_ok()
        );
        let mut wrong = stop.clone();
        wrong.claim.attempt += 1;
        assert!(
            stop_evidence(&fixture.request, entry, &[wrong], &mut Extent::new(&fixture.request.limits).unwrap())
                .is_err()
        );
        let mut session: assignment::authority::SessionObservation =
            object(&stop.session.as_ref().unwrap().path, deadline()).unwrap();
        session.session_id = SESSION.into();
        let path = fixture.root.join("observations/foreign-session.json");
        write(&path, &serde_json::to_vec(&session).unwrap());
        let mut wrong = stop.clone();
        wrong.session = Some(reference(&path));
        assert!(
            stop_evidence(&fixture.request, entry, &[wrong], &mut Extent::new(&fixture.request.limits).unwrap())
                .is_err()
        );
        let mut live = stop;
        live.processes.push(migration::OwnerProcess {
            identity: operation::process_identity(std::process::id()).unwrap().unwrap(),
            status: "stopped".into(),
            observation: reference(&fixture.root.join("observations/evidence.txt")),
        });
        assert!(
            stop_evidence(&fixture.request, entry, &[live], &mut Extent::new(&fixture.request.limits).unwrap())
                .is_err()
        );
    }

    #[test]
    fn bounded_extent_counts_interrupted_files_and_never_publishes_over_budget() {
        let mut fixture = Fixture::new();
        fixture.request.limits.max_files = 32;
        let receipt = fixture.build().unwrap();
        assert!(receipt.referenced_files <= 32);
        assert!(receipt.retained_bytes <= fixture.request.limits.max_bytes);
        fs::remove_file(fixture.package().join("build-receipt.json")).unwrap();
        for number in 0..33 {
            write(&fixture.package().join(format!("foreign-{number}.bin")), b"");
        }
        assert!(fixture.build().is_err());
        let fixture = Fixture::new();
        let mut bytes = fixture.request.clone();
        bytes.limits.max_bytes = 32;
        assert!(build_inner(bytes, None, &fixture.allowed, false).is_err());
        let mut source = fixture.request.clone();
        source.limits.max_source_file_bytes = 1;
        assert!(build_inner(source, None, &fixture.allowed, false).is_err());
        assert!(!fixture.package().join("build-receipt.json").exists());
        let invalid = BuildLimits { timeout_seconds: 0, ..BuildLimits::default() };
        assert!(Extent::new(&invalid).is_err());
    }

    #[test]
    fn pending_publications_require_identical_prefix_and_paths_reject_aliases() {
        let fixture = Fixture::new();
        let output = fixture.allowed.join("pending-test.json");
        write(&pending_path(&output), b"foreign prefix");
        assert!(write_once(&output, b"expected bytes", deadline()).is_err());
        assert!(!output.exists());
        write(&pending_path(&output), b"expected");
        write_once(&output, b"expected bytes", deadline()).unwrap();
        assert_eq!(fs::read(&output).unwrap(), b"expected bytes");
        assert!(!pending_path(&output).exists());
        let alias = fixture.root.join("alias");
        symlink(&fixture.allowed, &alias).unwrap();
        let mut request = fixture.request.clone();
        request.output_root = alias.join("output");
        assert!(build_inner(request, None, &fixture.allowed, false).is_err());
        let fifo = fixture.root.join("observation-fifo");
        let encoded = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(encoded.as_ptr(), 0o600) }, 0);
        let started = Instant::now();
        assert!(seal(&fifo, MAX_RECORD, deadline()).is_err());
        assert!(read_bytes(&fifo, MAX_RECORD, deadline()).is_err());
        assert!(started.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn bounded_subprocess_refuses_expired_deadline_and_terminates_overflow_or_timeout() {
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "sleep 10"]);
        let started = Instant::now();
        assert!(command_output(command, 128, started + Duration::from_millis(60)).is_err());
        assert!(started.elapsed() < Duration::from_secs(2));
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "printf '%07000d' 0"]);
        assert!(command_output(command, 32, deadline()).is_err());
        let mut command = Command::new("/bin/sh");
        command.args(["-c", "exit 7"]);
        assert!(command_output(command, 128, deadline()).is_err());
        assert!(command_output(Command::new("/bin/true"), 128, Instant::now()).is_err());
    }
}
