//! Read-only affected Offline family selection. Planning never admits retained evidence.
use crate::{batch, digest, family_model};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
};

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
fn changed_paths(checkout: &Path, revision: &str) -> Result<BTreeSet<PathBuf>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "--no-renames", "-z", revision, "--"])
        .current_dir(checkout)
        .output()?;
    ensure!(output.status.success(), "changed-since revision cannot be resolved");
    let untracked = Command::new("git")
        .args(["ls-files", "--others", "--exclude-standard", "-z"])
        .current_dir(checkout)
        .output()?;
    ensure!(untracked.status.success(), "cannot inspect untracked changed paths");
    let mut result = BTreeSet::new();
    for bytes in [output.stdout, untracked.stdout] {
        for name in bytes.split(|byte| *byte == 0).filter(|name| !name.is_empty()) {
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

fn history_paths(checkout: &Path) -> Result<BTreeSet<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard", "-z"])
        .current_dir(checkout)
        .output()?;
    ensure!(output.status.success(), "cannot inspect source history paths");
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|name| !name.is_empty())
        .map(|name| {
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
    if path.starts_with("tools/port") && path.extension().is_some_and(|extension| extension == "toml") {
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
    available: BTreeMap<String, PathBuf>,
    namespace: BTreeMap<String, PathBuf>,
    history: BTreeSet<PathBuf>,
    sources: BTreeMap<PathBuf, String>,
    hashes: BTreeMap<PathBuf, String>,
}

impl<'a> Discovery<'a> {
    fn new(checkout: &'a Path) -> Result<Self> {
        let mut available = BTreeMap::new();
        for entry in fs::read_dir(checkout.join("crates/port/src"))? {
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
            namespace: available.clone(),
            available,
            history: history_paths(checkout)?,
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
        let before = digest(path)?;
        let source = fs::read_to_string(path)?;
        ensure!(before == digest(path)?, "Rust discovery source changed while reading: {}", path.display());
        self.hashes.insert(normalize(self.checkout, path), before);
        // Unsupported file inputs cannot be excluded by top-level discovery.
        // This conservative check may also match comments, which only adds work.
        let words = source.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).collect::<Vec<_>>();
        ensure!(
            !words.iter().any(|word| [
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
            .contains(word)),
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
            self.hashes.insert(path.clone(), digest(&self.checkout.join(&path))?);
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
        let hash = digest(&actual)?;
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
    let checkout = fs::canonicalize(checkout)?;
    let changed = revision.map(|revision| changed_paths(&checkout, revision)).transpose()?.unwrap_or_default();
    let global = changed.iter().filter(|path| shared(path)).cloned().collect::<Vec<_>>();
    let config = batch::catalog(&checkout)?;
    let mut discovery = Discovery::new(&checkout).ok();
    if let Some(discovery) = &mut discovery {
        // Include removed names when traversing source words. A surviving source
        // that still mentions a deleted module remains affected, rather than
        // appearing independent because the current namespace lost that name.
        for path in &changed {
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
        checkout: checkout.clone(),
        changed_since: revision.map(str::to_owned),
        changed_paths: changed.iter().cloned().collect(),
        selected: vec![],
        excluded: vec![],
        planning_inputs: BTreeMap::new(),
    };
    report.planning_inputs.insert("tools/port/batch.toml".into(), digest(&checkout.join("tools/port/batch.toml"))?);
    let mut seen = BTreeSet::new();
    for name in names.iter().filter(|name| seen.insert((*name).clone())) {
        let mut decision = FamilyDecision { family: name.clone(), reasons: vec![], dependencies: vec![] };
        let examined = (|| -> Result<BTreeSet<PathBuf>> {
            let item = batch::entry(&config, name)?;
            ensure!(item.prepare.is_none(), "unknown fixture preparation inputs");
            let manifest_path = normalize(&checkout, &item.manifest);
            let path = checkout.join(&manifest_path);
            ensure!(!path.is_symlink(), "manifest is a symlink");
            let manifest: family_model::Manifest = pipeline_model::read(&path)?;
            report.planning_inputs.insert(manifest_path.clone(), digest(&path)?);
            let mut dependencies = discovery
                .as_mut()
                .context("Rust module namespace is unavailable")?
                .dependencies(&manifest, &item.kind)?;
            dependencies.insert(manifest_path);
            Ok(dependencies)
        })();
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
        if revision.is_none() {
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
        let namespace = Discovery::new(&checkout).map(|current| current.available);
        let stable = namespace.is_ok_and(|namespace| namespace == discovery.namespace)
            && report
                .planning_inputs
                .iter()
                .all(|(path, expected)| digest(&checkout.join(path)).is_ok_and(|actual| actual == *expected));
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
    Ok(report)
}
