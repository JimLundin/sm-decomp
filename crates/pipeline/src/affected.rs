//! Read-only affected Offline family selection. Planning never admits retained evidence.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Component, Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::checkout;

const DEFAULT_PLANNING_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PLANNING_ENTRIES: u64 = 65536;
const MAX_CLOSURE_CACHE_BYTES: usize = 16 * 1024 * 1024;

/// A version fence supplements an actual digest read during this invocation.
/// It never substitutes for reading a declared asset or admitting evidence.
#[derive(Clone, Debug, PartialEq, Eq)]
struct InputVersion {
    device: u64,
    inode: u64,
    length: u64,
    modified: (i64, i64),
    changed: (i64, i64),
}

impl InputVersion {
    fn of(metadata: &fs::Metadata) -> Self {
        Self {
            device: metadata.dev(),
            inode: metadata.ino(),
            length: metadata.len(),
            modified: (metadata.mtime(), metadata.mtime_nsec()),
            changed: (metadata.ctime(), metadata.ctime_nsec()),
        }
    }
}

struct AssetObservation {
    sha256: String,
    version: InputVersion,
}

/// Source discovery belongs to the whole stage budget, including rereads that
/// check stability. Resource exhaustion cannot become ordinary uncertainty.
struct PlanningBudget {
    started: Instant,
    deadline_unix_ms: u64,
    deadline: Instant,
    maximum: u64,
    bytes: Cell<u64>,
    entries: Cell<u64>,
    exhausted: Cell<bool>,
}

impl PlanningBudget {
    fn new(deadline_unix_ms: u64, maximum: u64) -> Result<Self> {
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        ensure!(now < deadline_unix_ms && maximum > 0, "affected planning budget exhausted before discovery");
        Ok(Self {
            started: Instant::now(),
            deadline_unix_ms,
            deadline: Instant::now() + Duration::from_millis(deadline_unix_ms - now).min(Duration::from_secs(120)),
            maximum,
            bytes: Cell::new(0),
            entries: Cell::new(0),
            exhausted: Cell::new(false),
        })
    }

    fn check(&self) -> Result<()> {
        ensure!(
            !self.exhausted.get() && Instant::now() < self.deadline,
            "affected planning aggregate byte, inode or time budget exhausted (bytes {}/{}, entries {}/{}, elapsed {:.3}s)",
            self.bytes.get(),
            self.maximum,
            self.entries.get(),
            MAX_PLANNING_ENTRIES,
            self.started.elapsed().as_secs_f64()
        );
        Ok(())
    }

    fn charge(&self, bytes: u64) -> Result<()> {
        self.check()?;
        let total = self.bytes.get().checked_add(bytes).context("affected planning extent overflow")?;
        if total > self.maximum {
            self.exhausted.set(true);
        }
        self.bytes.set(total);
        self.check()
    }

    fn entry(&self) -> Result<()> {
        self.check()?;
        let count = self.entries.get() + 1;
        if count > MAX_PLANNING_ENTRIES {
            self.exhausted.set(true);
        }
        self.entries.set(count);
        self.check()
    }

    fn read(&self, path: &Path, capture: bool) -> Result<(Vec<u8>, String, InputVersion)> {
        self.check()?;
        ensure!(!path.is_symlink(), "planning input is a symlink: {}", path.display());
        let mut file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)?;
        let before = file.metadata()?;
        ensure!(before.is_file(), "planning input is not regular: {}", path.display());
        if before.len() > self.maximum.saturating_sub(self.bytes.get()) {
            self.exhausted.set(true);
        }
        self.check()?;
        let mut bytes = Vec::new();
        let mut hash = Sha256::new();
        let mut length = 0u64;
        let mut buffer = [0u8; 65536];
        loop {
            self.check()?;
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            self.charge(count as u64)?;
            length += count as u64;
            hash.update(&buffer[..count]);
            if capture {
                bytes.extend_from_slice(&buffer[..count]);
            }
        }
        let after = file.metadata()?;
        ensure!(
            length == before.len() && InputVersion::of(&after) == InputVersion::of(&before),
            "planning input changed while reading: {}",
            path.display()
        );
        self.check()?;
        Ok((bytes, format!("{:x}", hash.finalize()), InputVersion::of(&after)))
    }

    fn hash(&self, path: &Path) -> Result<String> {
        Ok(self.read(path, false)?.1)
    }

    fn paths(&self, checkout: &Path, argv: &[&str]) -> Result<Vec<u8>> {
        self.check()?;
        let output = checkout::read_only_paths_until(
            checkout,
            argv,
            self.maximum.saturating_sub(self.bytes.get()),
            self.deadline_unix_ms,
        );
        let output = match output {
            Ok(output) => output,
            Err(error) => {
                self.exhausted.set(true);
                return Err(error.context("bounded affected planning Git query failed"));
            }
        };
        self.charge(output.len() as u64)?;
        Ok(output)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct Reason {
    pub code: String,
    pub detail: String,
    pub paths: Vec<PathBuf>,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub(crate) struct FamilyDecision {
    pub family: String,
    pub reasons: Vec<Reason>,
    pub dependencies: Vec<PathBuf>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Report {
    pub schema_version: u32,
    pub status: String,
    pub scope: String,
    pub checkout: PathBuf,
    pub changed_since: Option<String>,
    pub changed_paths: Vec<PathBuf>,
    pub selected: Vec<FamilyDecision>,
    pub excluded: Vec<FamilyDecision>,
    pub planning_inputs: BTreeMap<PathBuf, String>,
    #[serde(default)]
    pub planning_statistics: PlanningStatistics,
}

/// Advisory work counters; this read-only report never grants reuse admission.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct PlanningStatistics {
    pub elapsed_seconds: f64,
    pub input_bytes: u64,
    pub entries: u64,
    pub rust_source_parses: usize,
    pub rust_source_cache_hits: usize,
    #[serde(default)]
    pub rust_closure_cache_hits: usize,
    #[serde(default)]
    pub shared_input_cache_hits: usize,
    #[serde(default)]
    pub asset_declarations: usize,
    #[serde(default)]
    pub asset_reads: usize,
}

impl Report {
    pub fn families(&self) -> Vec<String> {
        self.selected.iter().map(|decision| decision.family.clone()).collect()
    }
}

fn reason(code: &str, detail: impl Into<String>, paths: Vec<PathBuf>) -> Reason {
    Reason { code: code.into(), detail: detail.into(), paths }
}

// NUL records preserve whitespace/newlines. Disabling rename inference deliberately
// retains both the removed and added paths, including staged and unstaged changes.
fn changed_paths(checkout: &Path, revision: &str, budget: &PlanningBudget) -> Result<BTreeSet<PathBuf>> {
    let output = budget.paths(checkout, &["diff", "--name-only", "--no-renames", "-z", revision, "--"])?;
    let untracked = budget.paths(checkout, &["ls-files", "--others", "--exclude-standard", "-z"])?;
    let mut result = BTreeSet::new();
    for bytes in [output, untracked] {
        for name in bytes.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
            budget.entry()?;
            #[cfg(unix)]
            let path = {
                use std::os::unix::ffi::OsStrExt;
                PathBuf::from(std::ffi::OsStr::from_bytes(name))
            };
            #[cfg(not(unix))]
            let path = PathBuf::from(std::str::from_utf8(name).context("non-UTF8 changed path")?);
            ensure!(path.components().all(|part| matches!(part, Component::Normal(_))), "unsafe Git changed path");
            result.insert(path);
        }
    }
    Ok(result)
}

fn history_paths(checkout: &Path, budget: &PlanningBudget) -> Result<BTreeSet<PathBuf>> {
    let output = budget.paths(checkout, &["ls-files", "--cached", "--others", "--exclude-standard", "-z"])?;
    output
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
            budget.entry()?;
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                Ok(PathBuf::from(std::ffi::OsStr::from_bytes(name)))
            }
            #[cfg(not(unix))]
            {
                Ok(PathBuf::from(std::str::from_utf8(name).context("non-UTF8 source history path")?))
            }
        })
        .collect()
}

fn normalize(checkout: &Path, path: &Path) -> PathBuf {
    let absolute = if path.is_absolute() { path.to_path_buf() } else { checkout.join(path) };
    let mut normalized = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => {}
            _ => normalized.push(part.as_os_str()),
        }
    }
    normalized.strip_prefix(checkout).unwrap_or(&normalized).to_path_buf()
}

fn shared(path: &Path) -> bool {
    if path.starts_with("tools/port")
        && path.extension().is_some_and(|extension| extension == "toml" || extension == "smf")
    {
        return path == Path::new("tools/port/batch.toml");
    }
    path.starts_with("tools")
        || path.starts_with("crates/pipeline")
        || path.starts_with("crates/offline-verification")
        || path.starts_with("crates/tool-execution")
        || path.starts_with("crates/decomp-source")
        || path.starts_with("crates/pipeline-model")
        || path.starts_with(".cargo")
        || path.starts_with("deps")
        || ["Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "crates/port/src/memory.rs", "crates/port/Cargo.toml"]
            .iter()
            .any(|shared| path == Path::new(shared))
}

struct Discovery<'a> {
    checkout: &'a Path,
    budget: &'a PlanningBudget,
    available: BTreeMap<String, PathBuf>,
    namespace: BTreeMap<String, PathBuf>,
    history: BTreeSet<PathBuf>,
    sources: BTreeMap<PathBuf, Arc<offline_verification::source_facts::RustSourceFacts>>,
    closures: BTreeMap<PathBuf, Arc<BTreeSet<PathBuf>>>,
    closure_bytes: usize,
    hashes: BTreeMap<PathBuf, String>,
    assets: BTreeMap<PathBuf, BTreeMap<String, String>>,
    asset_observations: BTreeMap<PathBuf, AssetObservation>,
    asset_declarations: usize,
    asset_reads: usize,
    cache_hits: usize,
    closure_cache_hits: usize,
    input_cache_hits: usize,
}

impl<'a> Discovery<'a> {
    fn new(checkout: &'a Path, budget: &'a PlanningBudget) -> Result<Self> {
        let mut available = BTreeMap::new();
        for entry in fs::read_dir(checkout.join("crates/port/src"))? {
            budget.entry()?;
            let path = entry?.path();
            if path.extension().is_some_and(|extension| extension == "rs") {
                let name = path.file_stem().context("source stem missing")?.to_str().context("non-UTF8 module name")?;
                if !["lib", "memory"].contains(&name) {
                    available.insert(name.into(), path);
                }
            }
        }
        Ok(Self {
            checkout,
            budget,
            namespace: available.clone(),
            available,
            history: history_paths(checkout, budget)?,
            sources: BTreeMap::new(),
            closures: BTreeMap::new(),
            closure_bytes: 0,
            hashes: BTreeMap::new(),
            assets: BTreeMap::new(),
            asset_observations: BTreeMap::new(),
            asset_declarations: 0,
            asset_reads: 0,
            cache_hits: 0,
            closure_cache_hits: 0,
            input_cache_hits: 0,
        })
    }

    fn source(&mut self, path: &Path) -> Result<Arc<offline_verification::source_facts::RustSourceFacts>> {
        ensure!(!path.is_symlink(), "Rust discovery source is a symlink: {}", path.display());
        ensure!(
            self.history.contains(&normalize(self.checkout, path)),
            "Rust source is ignored or outside changed-since history"
        );
        if let Some(source) = self.sources.get(path) {
            self.cache_hits += 1;
            return Ok(source.clone());
        }
        let (source, before, _) = self.budget.read(path, true)?;
        ensure!(before == self.budget.hash(path)?, "Rust discovery source changed while reading: {}", path.display());
        self.hashes.insert(normalize(self.checkout, path), before);
        let facts = tool_execution::bounded::until(
            self.budget.deadline_unix_ms,
            self.budget.maximum.saturating_sub(self.budget.bytes.get()).max(1),
            || offline_verification::source_facts::scan(&normalize(self.checkout, path), &source),
        );
        let facts = match facts {
            Ok(facts) => Arc::new(facts),
            Err(error) => {
                // Scanner errors are resource/deadline failures. Unsupported
                // syntax returns explicit facts, never an error that can hide
                // exhaustion as an ordinary uncertain family.
                self.budget.exhausted.set(true);
                return Err(error);
            }
        };
        self.sources.insert(path.into(), facts.clone());
        Ok(facts)
    }

    fn rust_closure(&mut self, root: &Path) -> Result<Arc<BTreeSet<PathBuf>>> {
        self.budget.check()?;
        let root = normalize(self.checkout, root);
        if let Some(closure) = self.closures.get(&root) {
            self.closure_cache_hits += 1;
            return Ok(closure.clone());
        }
        let mut pending = BTreeSet::from([(root.clone(), None)]);
        let mut result = BTreeSet::new();
        let mut placements = BTreeSet::new();
        while let Some((path, parent)) = pending.pop_first() {
            self.budget.check()?;
            if !placements.insert((path.clone(), parent.clone())) {
                continue;
            }
            self.budget.entry()?;
            let actual = self.checkout.join(&path);
            ensure!(!actual.is_symlink(), "Rust discovery module is a symlink");
            let actual = fs::canonicalize(actual)?;
            ensure!(path == normalize(self.checkout, &actual), "Rust discovery module has a filesystem alias");
            ensure!(actual.starts_with(self.checkout), "Rust source outside checkout lacks changed-since history");
            let facts = self.source(&actual)?;
            if let Some(parent) = &parent {
                facts.require_complete_with_parent(
                    self.sources.get(&self.checkout.join(parent)).context("Rust module parent was not scanned")?,
                )?;
            } else {
                facts.require_complete()?;
            }
            if !result.insert(path.clone()) {
                continue;
            }
            if let Some(closure) = self.closures.get(&path) {
                self.closure_cache_hits += 1;
                for dependency in closure.iter() {
                    self.budget.entry()?;
                    result.insert(dependency.clone());
                }
                continue;
            }
            for word in facts.identifiers() {
                if let Some(dependency) = self.available.get(word) {
                    let dependency = normalize(self.checkout, dependency);
                    if !placements.contains(&(dependency.clone(), None)) {
                        pending.insert((dependency, None));
                    }
                }
            }
            for dependency in facts.file_dependencies() {
                if !placements.contains(&(dependency.clone(), Some(path.clone()))) {
                    pending.insert((dependency.clone(), Some(path.clone())));
                }
            }
        }
        // Only complete transitive closures are reusable. This optional cache
        // has its own retained-memory ceiling; uncached work still uses the
        // same whole-stage budget. Every reported membership is charged below.
        let retained = 512 + root.capacity() + result.iter().map(|path| 256 + path.capacity()).sum::<usize>();
        let result = Arc::new(result);
        if retained <= MAX_CLOSURE_CACHE_BYTES.saturating_sub(self.closure_bytes) {
            self.closure_bytes += retained;
            self.closures.insert(root, result.clone());
        }
        Ok(result)
    }

    fn input_hash(&mut self, path: &Path) -> Result<String> {
        self.budget.check()?;
        let path = normalize(self.checkout, path);
        if let Some(hash) = self.hashes.get(&path) {
            self.input_cache_hits += 1;
            return Ok(hash.clone());
        }
        let hash = self.budget.hash(&self.checkout.join(&path))?;
        self.hashes.insert(path, hash.clone());
        Ok(hash)
    }

    fn validate_current(&self, inputs: &BTreeMap<PathBuf, String>) -> Result<bool> {
        let namespace = Self::new(self.checkout, self.budget).map(|current| current.available);
        self.budget.check()?;
        let mut stable = namespace.is_ok_and(|namespace| namespace == self.namespace);
        // Cached source closures also retain their filesystem alias checks.
        // Replacing a source directory with a same-content symlink cannot make
        // a cached closure eligible for a changed-since exclusion.
        for path in self.sources.keys() {
            self.budget.entry()?;
            stable &= !path.is_symlink() && fs::canonicalize(path).is_ok_and(|current| current == *path);
        }
        for (path, expected) in inputs {
            stable &= if let Some(observed) = self.asset_observations.get(path) {
                observed.sha256 == *expected
            } else {
                self.budget.hash(&self.checkout.join(path)).is_ok_and(|actual| actual == *expected)
            };
            self.budget.check()?;
        }
        Ok(stable && self.validate_asset_versions()?)
    }

    fn dependencies(
        &mut self,
        manifest: &offline_verification::fixtures::Manifest,
        kind: &str,
        family: &str,
    ) -> Result<BTreeSet<PathBuf>> {
        ensure!(
            [
                "family",
                "offline-family",
                "dispatcher",
                "plm-instructions",
                "projectile",
                "offline-projectile",
                "callback",
                "plm",
                "power-bomb"
            ]
            .contains(&kind),
            "unknown Offline family adapter: {kind}"
        );
        ensure!(kind != "plm-instructions", "generated PLM fixture inputs require verification");
        let mut result = BTreeSet::new();
        let memory = self.checkout.join("crates/port/src/memory.rs");
        let root = self.checkout.join(&manifest.rust_module);
        let mut roots = BTreeSet::from([root, memory]);
        // PLM uses the same discovery implementation, but has no explicit imports.
        if kind != "plm" {
            for name in &manifest.rust_imports {
                roots.insert(self.available.get(name).with_context(|| format!("unknown Rust import: {name}"))?.clone());
            }
        }
        for root in roots {
            for path in self.rust_closure(&root)?.iter() {
                self.budget.entry()?;
                result.insert(path.clone());
            }
        }
        for path in &manifest.rust_dependencies {
            let path = normalize(self.checkout, path);
            ensure!(!path.is_absolute(), "external declared dependency lacks changed-since history");
            ensure!(self.history.contains(&path), "declared dependency is ignored by changed-since history");
            let actual = self.checkout.join(&path);
            ensure!(
                !actual.is_symlink() && normalize(self.checkout, &fs::canonicalize(&actual)?) == path,
                "declared dependency has a filesystem alias"
            );
            ensure!(self.checkout.join(&path).is_file(), "declared dependency is missing: {}", path.display());
            self.input_hash(&path)?;
            result.insert(path);
        }
        if let Some(rom) = &manifest.rom {
            self.asset(&rom.file, Some(&rom.sha256), family)?;
            result.insert(normalize(self.checkout, &rom.file));
        }
        for case in manifest.cases.iter().chain(manifest.routines.iter().flat_map(|routine| &routine.cases)) {
            for (path, expected) in
                [(&case.before_file, &case.before_sha256), (&case.expected_after_file, &case.expected_after_sha256)]
            {
                let Some(path) = path else {
                    continue;
                };
                self.asset(path, expected.as_deref(), family)?;
                result.insert(normalize(self.checkout, path));
            }
        }
        Ok(result)
    }

    fn asset(&mut self, path: &Path, expected: Option<&str>, family: &str) -> Result<()> {
        self.budget.entry()?;
        let expected = expected.context("family asset is missing a declared digest")?;
        ensure!(
            expected.len() == 64 && expected.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "family asset has an invalid declared digest"
        );
        self.asset_declarations += 1;
        let path = normalize(self.checkout, path);
        if self.assets.contains_key(&path) {
            self.input_cache_hits += 1;
        }
        let requirements = self.assets.entry(path).or_default();
        if let Some(previous) = requirements.insert(family.into(), expected.into()) {
            ensure!(previous == expected, "family has conflicting asset requirements");
        }
        Ok(())
    }

    /// Discovery consumes declarations, not asset contents. Authenticate each
    /// distinct asset once after all expected digests have been captured. All
    /// families still check their own requirement; a shared path is not proof
    /// that different declared digests are equivalent.
    fn authenticate_assets(&mut self) -> Result<BTreeMap<String, String>> {
        self.asset_observations.clear();
        let mut failures = BTreeMap::new();
        for (path, requirements) in &self.assets {
            self.budget.entry()?;
            self.asset_reads += 1;
            let observed = self.budget.read(&self.checkout.join(path), false);
            self.budget.check()?;
            match observed {
                Ok((_, sha256, version)) => {
                    for (family, expected) in requirements {
                        if expected != &sha256 {
                            failures.entry(family.clone()).or_insert_with(|| {
                                format!("family asset differs from its declared digest: {}", path.display())
                            });
                        }
                    }
                    self.asset_observations.insert(path.clone(), AssetObservation { sha256, version });
                }
                Err(error) => {
                    for family in requirements.keys() {
                        failures.entry(family.clone()).or_insert_with(|| {
                            format!("family asset could not be authenticated at {}: {error:#}", path.display())
                        });
                    }
                }
            }
        }
        Ok(failures)
    }

    /// Preserve one coherent end-of-pass view after actual asset hashing and
    /// final source/catalog validation. Same-sized replacement, content edits
    /// and namespace aliases cannot reuse an earlier file observation.
    fn validate_asset_versions(&self) -> Result<bool> {
        let mut stable = true;
        for (path, observed) in &self.asset_observations {
            self.budget.entry()?;
            stable &= self
                .checkout
                .join(path)
                .symlink_metadata()
                .is_ok_and(|current| current.is_file() && InputVersion::of(&current) == observed.version);
        }
        Ok(stable)
    }
}

pub(crate) fn select(checkout: &Path, names: &[String], revision: Option<&str>) -> Result<Report> {
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
    select_until(checkout, names, revision, now + 120_000, DEFAULT_PLANNING_BYTES)
}

pub(crate) fn select_with_snapshot(
    snapshot: &offline_verification::CatalogSnapshot,
    names: &[String],
    revision: Option<&str>,
) -> Result<Report> {
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
    let budget = PlanningBudget::new(now + 120_000, DEFAULT_PLANNING_BYTES)?;
    let checkout = snapshot.checkout();
    let changed = revision.map(|revision| changed_paths(checkout, revision, &budget)).transpose()?.unwrap_or_default();
    select_changed(snapshot, Some(names), revision, &changed, revision.is_none(), &budget)
}

fn capture_catalog(checkout: &Path, budget: &PlanningBudget) -> Result<offline_verification::CatalogSnapshot> {
    let snapshot = offline_verification::catalog::capture(offline_verification::CatalogRequest {
        checkout: checkout.into(),
        deadline_unix_ms: budget.deadline_unix_ms,
        max_input_bytes: budget.maximum.saturating_sub(budget.bytes.get()),
        max_retained_bytes: tool_execution::bounded::DEFAULT_BYTES,
    })?;
    budget.charge(snapshot.statistics().input_bytes)?;
    Ok(snapshot)
}

pub(crate) fn select_until(
    checkout: &Path,
    names: &[String],
    revision: Option<&str>,
    deadline_unix_ms: u64,
    maximum: u64,
) -> Result<Report> {
    let budget = PlanningBudget::new(deadline_unix_ms, maximum)?;
    let checkout = fs::canonicalize(checkout)?;
    let changed = revision.map(|revision| changed_paths(&checkout, revision, &budget)).transpose()?.unwrap_or_default();
    let snapshot = capture_catalog(&checkout, &budget)?;
    select_changed(&snapshot, Some(names), revision, &changed, revision.is_none(), &budget)
}

/// Plan against every catalog family before a rewrite can touch any approved
/// source. This retains the same conservative discovery and exclusion reasons
/// as changed-since planning, without substituting an empty current Git diff
/// for the full future writable ownership.
pub(crate) fn select_paths_until(
    checkout: &Path,
    approved_paths: &[String],
    deadline_unix_ms: u64,
    maximum: u64,
) -> Result<Report> {
    let budget = PlanningBudget::new(deadline_unix_ms, maximum)?;
    let checkout = fs::canonicalize(checkout)?;
    let mut changed = BTreeSet::new();
    for name in approved_paths {
        budget.entry()?;
        let path = PathBuf::from(name);
        ensure!(
            !name.is_empty() && path.components().all(|part| matches!(part, Component::Normal(_))),
            "approved affected-planning path is not relative"
        );
        ensure!(changed.insert(path), "duplicate approved affected-planning path");
    }
    ensure!(!changed.is_empty(), "approved affected-planning ownership is empty");
    let snapshot = capture_catalog(&checkout, &budget)?;
    let mut report = select_changed(&snapshot, None, None, &changed, false, &budget)?;
    report.scope = "potentially affected full catalog for approved writable paths; no verification, evidence reuse or Live replay admission".into();
    Ok(report)
}

fn select_changed(
    snapshot: &offline_verification::CatalogSnapshot,
    names: Option<&[String]>,
    revision: Option<&str>,
    changed: &BTreeSet<PathBuf>,
    requested: bool,
    budget: &PlanningBudget,
) -> Result<Report> {
    let checkout = snapshot.checkout();
    let global = changed.iter().filter(|path| shared(path)).cloned().collect::<Vec<_>>();
    let config = snapshot.catalog();
    let before = snapshot.statistics();
    let mut discovery = Discovery::new(checkout, budget).ok();
    budget.check()?;
    if let Some(discovery) = &mut discovery {
        // Include removed names when traversing source words. A surviving source
        // that still mentions a deleted module remains affected, rather than
        // appearing independent because the current namespace lost that name.
        for path in changed {
            if path.parent() == Some(Path::new("crates/port/src"))
                && path.extension().is_some_and(|extension| extension == "rs")
                && let Some(name) = path.file_stem().and_then(|name| name.to_str())
                && !["lib", "memory"].contains(&name)
            {
                discovery.available.entry(name.into()).or_insert_with(|| checkout.join(path));
            }
        }
    }
    let mut report = Report {
        schema_version: 1,
        status: "planned".into(),
        scope: "selection only; no verification, evidence reuse or Live replay admission".into(),
        checkout: checkout.into(),
        changed_since: revision.map(str::to_owned),
        changed_paths: changed.iter().cloned().collect(),
        selected: vec![],
        excluded: vec![],
        planning_inputs: BTreeMap::new(),
        planning_statistics: PlanningStatistics::default(),
    };
    report.planning_inputs = snapshot.input_identities();
    let catalog_names = config.families.keys().cloned().collect::<Vec<_>>();
    let names = names.unwrap_or(&catalog_names);
    let mut seen = BTreeSet::new();
    let mut family_order = BTreeMap::new();
    for name in names.iter().filter(|name| seen.insert((*name).clone())) {
        budget.entry()?;
        family_order.insert(name.clone(), family_order.len());
        let mut decision = FamilyDecision { family: name.clone(), reasons: vec![], dependencies: vec![] };
        let examined = (|| -> Result<BTreeSet<PathBuf>> {
            let facts = snapshot.family(name)?;
            let item = facts.entry();
            ensure!(item.prepare.is_none(), "unknown fixture preparation inputs");
            let manifest_path = normalize(checkout, &item.manifest);
            let manifest = facts.declaration();
            report.planning_inputs.insert(manifest_path.clone(), facts.manifest_sha256().into());
            let mut dependencies = discovery
                .as_mut()
                .context("Rust module namespace is unavailable")?
                .dependencies(manifest, &item.kind, name)?;
            dependencies.insert(manifest_path);
            Ok(dependencies)
        })();
        snapshot.check_budget()?;
        budget.check()?;
        match examined {
            Ok(dependencies) => {
                decision.dependencies = dependencies.iter().cloned().collect();
                let matches = changed.intersection(&dependencies).cloned().collect::<Vec<_>>();
                if !matches.is_empty() {
                    decision.reasons.push(reason(
                        "changed-dependency",
                        "Current conservative Rust discovery or declared family input changed",
                        matches,
                    ));
                }
            }
            Err(error) => decision.reasons.push(reason("uncertain-inputs", format!("{error:#}"), vec![])),
        }
        if requested {
            decision.reasons.push(reason("requested", "No changed-since filter was requested", vec![]));
        }
        if !global.is_empty() {
            decision.reasons.push(reason(
                "shared-input",
                "Shared tooling, source or configuration changed",
                global.clone(),
            ));
        }
        if decision.reasons.is_empty() {
            decision.reasons.push(reason(
                "unaffected",
                "No changed path intersects current conservative discovery or declared family inputs",
                vec![],
            ));
            report.excluded.push(decision);
        } else {
            report.selected.push(decision);
        }
    }
    let mut stable = true;
    if let Some(discovery) = &mut discovery {
        let failures = discovery.authenticate_assets()?;
        for decision in &mut report.selected {
            if let Some(detail) = failures.get(&decision.family) {
                decision.reasons.insert(0, reason("uncertain-inputs", detail.clone(), vec![]));
            }
        }
        for mut decision in std::mem::take(&mut report.excluded) {
            if let Some(detail) = failures.get(&decision.family) {
                decision.reasons = vec![reason("uncertain-inputs", detail.clone(), vec![])];
                report.selected.push(decision);
            } else {
                report.excluded.push(decision);
            }
        }
        report.planning_statistics.rust_source_parses = discovery.sources.len();
        report.planning_statistics.rust_source_cache_hits = discovery.cache_hits;
        report.planning_statistics.rust_closure_cache_hits = discovery.closure_cache_hits;
        report.planning_statistics.shared_input_cache_hits = discovery.input_cache_hits;
        report.planning_statistics.asset_declarations = discovery.asset_declarations;
        report.planning_statistics.asset_reads = discovery.asset_reads;
        report.planning_inputs.extend(
            discovery.asset_observations.iter().map(|(path, observed)| (path.clone(), observed.sha256.clone())),
        );
        report.planning_inputs.extend(discovery.hashes.iter().map(|(path, hash)| (path.clone(), hash.clone())));
        // The captured catalog owns fresh validation of its exact declarations.
        // Rehashing those same manifests here duplicates that independent pass.
        stable &= discovery.validate_current(&discovery.hashes)?;
    }
    snapshot.validate_current()?;
    let after = snapshot.statistics();
    budget.charge(after.input_bytes - before.input_bytes + after.freshness_bytes - before.freshness_bytes)?;
    if let Some(discovery) = &discovery {
        stable &= discovery.validate_asset_versions()?;
    }
    if !stable {
        for mut decision in std::mem::take(&mut report.excluded) {
            decision.reasons = vec![reason(
                "uncertain-inputs",
                "Planning source, asset or module namespace changed during selection",
                vec![],
            )];
            report.selected.push(decision);
        }
    }
    report.selected.sort_by_key(|decision| family_order[&decision.family]);
    budget.check()?;
    report.planning_statistics.elapsed_seconds = budget.started.elapsed().as_secs_f64();
    report.planning_statistics.input_bytes = budget.bytes.get();
    report.planning_statistics.entries = budget.entries.get();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reused_closures_and_assets_keep_independent_freshness_checks() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        let sources = root.join("crates/port/src");
        fs::create_dir_all(sources.join("nested"))?;
        ensure!(
            std::process::Command::new("git").args(["init", "-q"]).arg(root).status()?.success(),
            "test Git initialization failed"
        );
        fs::write(sources.join("memory.rs"), "pub struct Wram;")?;
        fs::write(
            sources.join("candidate.rs"),
            "#[path=\"nested/helper.rs\"] mod owned; pub fn step() { owned::step(); }",
        )?;
        let helper = sources.join("nested/helper.rs");
        fs::write(&helper, "pub fn step() {}")?;
        let asset = root.join("asset.bin");
        fs::write(&asset, b"original")?;
        let asset_hash = format!("{:x}", Sha256::digest(b"original"));
        let budget = PlanningBudget::new(tool_execution::bounded::now_unix_ms()? + 10_000, DEFAULT_PLANNING_BYTES)?;
        let mut discovery = Discovery::new(root, &budget)?;
        let manifest = offline_verification::fixtures::Manifest {
            rust_module: "crates/port/src/candidate.rs".into(),
            ..Default::default()
        };
        let first = discovery.dependencies(&manifest, "family", "candidate")?;
        assert!(first.contains(Path::new("crates/port/src/nested/helper.rs")));
        assert_eq!(first, discovery.dependencies(&manifest, "family", "candidate")?);
        assert!(discovery.closure_cache_hits > 0);
        discovery.asset(&asset, Some(&asset_hash), "candidate")?;
        discovery.asset(&asset, Some(&asset_hash), "candidate")?;
        assert!(discovery.input_cache_hits > 0);
        assert_eq!(discovery.asset_reads, 0, "asset declaration discovery consumed its bytes");
        assert!(discovery.authenticate_assets()?.is_empty());
        assert_eq!(discovery.asset_reads, 1);
        let inputs = discovery.hashes.clone();
        assert!(discovery.validate_current(&inputs)?);

        fs::write(&helper, "pub fn step() { changed(); }")?;
        assert!(!discovery.validate_current(&inputs)?);
        fs::write(&helper, "pub fn step() {}")?;
        assert!(discovery.validate_current(&inputs)?);
        fs::write(&asset, b"mutation")?;
        assert!(!discovery.validate_current(&inputs)?);
        fs::write(&asset, b"original")?;
        assert!(!discovery.validate_asset_versions()?, "restored bytes bypassed the end-of-pass version fence");
        assert!(discovery.authenticate_assets()?.is_empty());

        fs::rename(sources.join("nested"), sources.join("retained"))?;
        std::os::unix::fs::symlink(sources.join("retained"), sources.join("nested"))?;
        assert!(!discovery.validate_current(&inputs)?, "same-content ancestor alias kept a cached exclusion");
        fs::remove_file(sources.join("nested"))?;
        fs::rename(sources.join("retained"), sources.join("nested"))?;
        fs::remove_file(&helper)?;
        assert!(!discovery.validate_current(&inputs)?);
        fs::write(&helper, "pub fn step() {}")?;
        fs::write(sources.join("new_module.rs"), "pub fn added() {}")?;
        assert!(!discovery.validate_current(&inputs)?);
        fs::remove_file(sources.join("new_module.rs"))?;
        assert!(discovery.validate_current(&inputs)?);
        Ok(())
    }

    #[test]
    fn approved_paths_plan_the_full_catalog_and_keep_unknown_inputs() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        fs::create_dir_all(root.join("tools/port"))?;
        fs::create_dir_all(root.join("crates/port/src"))?;
        ensure!(
            std::process::Command::new("git").args(["init", "-q"]).arg(root).status()?.success(),
            "test Git initialization failed"
        );
        fs::write(
            root.join("tools/port/batch.toml"),
            "[families]\na='family'\nb='family'\nunknown='family'\n[profiles]\ngameplay=['a']\n",
        )?;
        for (name, source) in [
            ("memory", "pub struct Wram;"),
            ("a", "pub fn a() {}"),
            ("b", "pub fn b() { helper::run(); }"),
            ("helper", "pub fn run() {}"),
            ("unknown", "include!(\"dynamic.rs\");"),
        ] {
            fs::write(root.join(format!("crates/port/src/{name}.rs")), source)?;
        }
        for name in ["a", "b", "unknown"] {
            fs::write(
                root.join(format!("tools/port/{name}.smf")),
                offline_verification::fixtures::encode(&offline_verification::fixtures::Manifest {
                    family: name.into(),
                    bank: "sm_80.c".into(),
                    rust_module: format!("crates/port/src/{name}.rs"),
                    ..Default::default()
                })?,
            )?;
        }
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        let report =
            select_paths_until(root, &["crates/port/src/helper.rs".into()], now + 10_000, DEFAULT_PLANNING_BYTES)?;
        assert_eq!(report.families(), ["b", "unknown"]);
        assert_eq!(report.excluded.len(), 1);
        assert_eq!(report.excluded[0].family, "a");
        assert_eq!(report.excluded[0].reasons[0].code, "unaffected");
        assert_eq!(report.selected[1].reasons[0].code, "uncertain-inputs");
        assert!(report.planning_inputs.contains_key(Path::new("tools/port/a.smf")));
        assert_eq!(report.changed_paths, [PathBuf::from("crates/port/src/helper.rs")]);
        assert!(report.changed_since.is_none());
        assert!(!root.join("out").exists());
        assert!(select_paths_until(root, &["../escape".into()], now + 10_000, DEFAULT_PLANNING_BYTES).is_err());
        fs::write(root.join("crates/port/src/a.rs"), vec![b' '; 8 * 1024 * 1024 + 1])?;
        let exhausted = select_paths_until(root, &["docs/unrelated.md".into()], now + 10_000, DEFAULT_PLANNING_BYTES);
        assert!(exhausted.is_err(), "scanner resource exhaustion was converted into ordinary family uncertainty");
        Ok(())
    }

    #[test]
    fn expired_or_exhausted_planning_stops_without_output_or_family_exclusion() -> Result<()> {
        let root = std::env::temp_dir().join(format!("sm-bounded-planning-{}", std::process::id()));
        ensure!(!root.exists(), "planning fixture already exists");
        fs::create_dir_all(root.join("tools/port"))?;
        fs::write(root.join("tools/port/batch.toml"), "[families]\na='family'\n[profiles]\n")?;
        fs::write(
            root.join("tools/port/a.smf"),
            format!("family='a'\nrust_module='crates/port/src/a.rs'\nroutines=[]\n#{}", "x".repeat(8192)),
        )?;
        let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
        ensure!(select_until(&root, &["a".into()], None, 1, 65536).is_err(), "expired planning touched source");
        let error = select_until(&root, &["a".into()], None, now + 10_000, 1024).unwrap_err();
        ensure!(error.to_string().contains("budget exhausted"), "planning exhaustion was swallowed: {error:#}");
        ensure!(!root.join("out").exists() && !root.join(".git").exists(), "read-only planning created outputs");
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
