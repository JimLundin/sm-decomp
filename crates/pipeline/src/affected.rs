//! Read-only affected Offline family selection. Planning never admits retained evidence.
use crate::{batch, family_model};
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
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use translation_execution::checkout;

const DEFAULT_PLANNING_BYTES: u64 = 256 * 1024 * 1024;
const MAX_PLANNING_ENTRIES: u64 = 65536;

/// Source discovery belongs to the whole stage budget, including rereads that
/// check stability. Resource exhaustion cannot become ordinary uncertainty.
struct PlanningBudget {
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
            "affected planning aggregate byte, inode or time budget exhausted"
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

    fn read(&self, path: &Path, capture: bool) -> Result<(Vec<u8>, String)> {
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
            length == before.len()
                && after.len() == before.len()
                && after.mtime() == before.mtime()
                && after.mtime_nsec() == before.mtime_nsec()
                && after.ctime() == before.ctime()
                && after.ctime_nsec() == before.ctime_nsec(),
            "planning input changed while reading: {}",
            path.display()
        );
        self.check()?;
        Ok((bytes, format!("{:x}", hash.finalize())))
    }

    fn text(&self, path: &Path) -> Result<(String, String)> {
        let (bytes, hash) = self.read(path, true)?;
        Ok((String::from_utf8(bytes)?, hash))
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
    sources: BTreeMap<PathBuf, String>,
    hashes: BTreeMap<PathBuf, String>,
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
            hashes: BTreeMap::new(),
        })
    }

    fn source(&mut self, path: &Path) -> Result<String> {
        ensure!(!path.is_symlink(), "Rust discovery source is a symlink: {}", path.display());
        ensure!(
            self.history.contains(&normalize(self.checkout, path)),
            "Rust source is ignored or outside changed-since history"
        );
        if let Some(source) = self.sources.get(path) {
            return Ok(source.clone());
        }
        let (source, before) = self.budget.text(path)?;
        ensure!(before == self.budget.hash(path)?, "Rust discovery source changed while reading: {}", path.display());
        self.hashes.insert(normalize(self.checkout, path), before);
        // Unsupported file inputs cannot be excluded by top-level discovery.
        // This conservative check may also match comments, which only adds work.
        ensure!(
            !source.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).any(|word| [
                "mod",
                "include",
                "include_str",
                "include_bytes",
                "env",
                "option_env",
                "fs",
                "File",
                "macro_rules"
            ]
            .contains(&word)),
            "Rust source has inputs beyond supported module discovery: {}",
            path.display()
        );
        self.sources.insert(path.into(), source.clone());
        Ok(source)
    }

    fn dependencies(&mut self, manifest: &family_model::Manifest, kind: &str) -> Result<BTreeSet<PathBuf>> {
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
        self.source(&memory)?;
        result.insert(normalize(self.checkout, &memory));
        let root = self.checkout.join(&manifest.rust_module);
        ensure!(!root.is_symlink(), "Rust root module is a symlink");
        let mut pending = vec![root];
        // PLM uses the same discovery implementation, but has no explicit imports.
        if kind != "plm" {
            for name in &manifest.rust_imports {
                pending.push(self.available.get(name).with_context(|| format!("unknown Rust import: {name}"))?.clone());
            }
        }
        let mut visited = BTreeSet::new();
        while let Some(path) = pending.pop() {
            self.budget.entry()?;
            ensure!(!path.is_symlink(), "Rust discovery module is a symlink");
            let lexical = normalize(self.checkout, &path);
            let path = fs::canonicalize(&path)?;
            ensure!(lexical == normalize(self.checkout, &path), "Rust discovery module has a filesystem alias");
            ensure!(path.starts_with(self.checkout), "Rust source outside checkout lacks changed-since history");
            if !visited.insert(path.clone()) {
                continue;
            }
            result.insert(normalize(self.checkout, &path));
            let source = self.source(&path)?;
            for word in source.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
                if let Some(dependency) = self.available.get(word) {
                    pending.push(dependency.clone());
                }
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
            self.hashes.insert(path.clone(), self.budget.hash(&self.checkout.join(&path))?);
            result.insert(path);
        }
        if let Some(rom) = &manifest.rom {
            self.asset(&rom.file, Some(&rom.sha256))?;
            result.insert(normalize(self.checkout, &rom.file));
        }
        for case in manifest.cases.iter().chain(manifest.routines.iter().flat_map(|routine| &routine.cases)) {
            for (path, expected) in
                [(&case.before_file, &case.before_sha256), (&case.expected_after_file, &case.expected_after_sha256)]
            {
                let Some(path) = path else {
                    continue;
                };
                self.asset(path, expected.as_deref())?;
                result.insert(normalize(self.checkout, path));
            }
        }
        Ok(result)
    }

    fn asset(&mut self, path: &Path, expected: Option<&str>) -> Result<()> {
        let actual = self.checkout.join(path);
        ensure!(!actual.is_symlink(), "family asset is a symlink");
        let hash = self.budget.hash(&actual)?;
        ensure!(
            expected == Some(hash.as_str()),
            "family asset is missing a matching declared digest: {}",
            actual.display()
        );
        self.hashes.insert(normalize(self.checkout, path), hash);
        Ok(())
    }
}

pub(crate) fn select(checkout: &Path, names: &[String], revision: Option<&str>) -> Result<Report> {
    let now = u64::try_from(SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis())?;
    select_until(checkout, names, revision, now + 120_000, DEFAULT_PLANNING_BYTES)
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
    select_changed(&checkout, Some(names), revision, &changed, revision.is_none(), &budget)
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
    let mut report = select_changed(&checkout, None, None, &changed, false, &budget)?;
    report.scope = "potentially affected full catalog for approved writable paths; no verification, evidence reuse or Live replay admission".into();
    Ok(report)
}

fn select_changed(
    checkout: &Path,
    names: Option<&[String]>,
    revision: Option<&str>,
    changed: &BTreeSet<PathBuf>,
    requested: bool,
    budget: &PlanningBudget,
) -> Result<Report> {
    let global = changed.iter().filter(|path| shared(path)).cloned().collect::<Vec<_>>();
    let catalog_path = checkout.join("tools/port/batch.toml");
    let catalog_hash = budget.hash(&catalog_path)?;
    let mut catalog_inputs = BTreeMap::new();
    let config = batch::catalog_using(checkout, |path| {
        budget.entry()?;
        let (bytes, hash) = budget.read(path, true)?;
        catalog_inputs.insert(normalize(checkout, path), hash);
        Ok(bytes)
    })?;
    ensure!(catalog_hash == budget.hash(&catalog_path)?, "Offline family catalog changed during selection");
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
    };
    report.planning_inputs = catalog_inputs;
    report.planning_inputs.insert("tools/port/batch.toml".into(), catalog_hash);
    let catalog_names = config.families.keys().cloned().collect::<Vec<_>>();
    let names = names.unwrap_or(&catalog_names);
    let mut seen = BTreeSet::new();
    for name in names.iter().filter(|name| seen.insert((*name).clone())) {
        budget.entry()?;
        let mut decision = FamilyDecision { family: name.clone(), reasons: vec![], dependencies: vec![] };
        let examined = (|| -> Result<BTreeSet<PathBuf>> {
            let item = batch::entry(&config, name)?;
            ensure!(item.prepare.is_none(), "unknown fixture preparation inputs");
            let manifest_path = normalize(checkout, &item.manifest);
            let path = checkout.join(&manifest_path);
            ensure!(!path.is_symlink(), "manifest is a symlink");
            let (bytes, manifest_hash) = budget.read(&path, true)?;
            let manifest = crate::fixture_format::decode(bytes)?;
            ensure!(manifest_hash == budget.hash(&path)?, "Offline family manifest changed during selection");
            report.planning_inputs.insert(manifest_path.clone(), manifest_hash);
            let mut dependencies = discovery
                .as_mut()
                .context("Rust module namespace is unavailable")?
                .dependencies(&manifest, &item.kind)?;
            dependencies.insert(manifest_path);
            Ok(dependencies)
        })();
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
    if let Some(discovery) = discovery {
        report.planning_inputs.extend(discovery.hashes);
        let namespace = Discovery::new(checkout, budget).map(|current| current.available);
        budget.check()?;
        let mut stable = namespace.is_ok_and(|namespace| namespace == discovery.namespace);
        for (path, expected) in &report.planning_inputs {
            stable &= budget.hash(&checkout.join(path)).is_ok_and(|actual| actual == *expected);
            budget.check()?;
        }
        if !stable {
            for mut decision in std::mem::take(&mut report.excluded) {
                decision.reasons = vec![reason(
                    "uncertain-inputs",
                    "Planning source or module namespace changed during selection",
                    vec![],
                )];
                report.selected.push(decision);
            }
        }
    }
    budget.check()?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approved_paths_plan_the_full_catalog_and_keep_unknown_inputs() -> Result<()> {
        let directory = crate::inputs::bounded::tests::Directory::new()?;
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
                crate::fixture_format::encode(&family_model::Manifest {
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
