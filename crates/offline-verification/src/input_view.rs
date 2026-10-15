//! Owned source bytes and checked streamed assets for one invocation. Logical
//! evidence paths stay distinct from the private compiler input directory.
use crate::{
    HostInputs,
    declarations::Declarations,
    family_model::{Manifest, Source},
};
use anyhow::{Context, Result, ensure};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    sync::{Arc, atomic::Ordering},
};
use tool_execution::bounded;

#[path = "input_view_native.rs"]
mod retained;
pub(crate) use retained::NativeCapturePlan;

thread_local! {
    static CURRENT: RefCell<Option<Arc<InputView>>> = const { RefCell::new(None) };
}

#[derive(Clone)]
struct File {
    physical: PathBuf,
    sha256: String,
    bytes: Option<Arc<Vec<u8>>>,
    maximum: u64,
    extent: u64,
}

/// The directory is private materialization of retained owned bytes. Neither
/// permissions nor remembered digests authenticate it: every compiler boundary
/// checks its bytes, and logical inputs are separately rechecked for freshness.
pub(crate) struct InputView {
    checkout: PathBuf,
    source: PathBuf,
    directory: PathBuf,
    files: Arc<BTreeMap<PathBuf, File>>,
    namespace: Arc<Vec<PathBuf>>,
    source_identity: Arc<Source>,
    declarations: Arc<Declarations>,
    builds: Arc<tool_execution::builds::BuildContext>,
    reserved_bytes: u64,
    metadata_bytes: u64,
    compiler_inputs: Arc<BTreeSet<PathBuf>>,
    common_inputs: Arc<BTreeSet<PathBuf>>,
    family_inputs: Arc<BTreeMap<(PathBuf, Vec<String>), BTreeSet<PathBuf>>>,
    selected_inputs: Option<BTreeSet<PathBuf>>,
    retained: bool,
    confined: bool,
    /// A generated family may add a trusted asset while sharing the same
    /// compiler tree. Its parent owns that directory and the original bytes.
    parent: Option<Arc<InputView>>,
}

impl Drop for InputView {
    fn drop(&mut self) {
        // Only this invocation creates this directory. All family workers hold
        // an Arc, so it cannot be removed while a compiler or fixture uses it.
        if self.parent.is_none() && !self.retained {
            let _ = fs::remove_dir_all(&self.directory);
        }
        self.declarations.release(self.reserved_bytes);
        self.declarations.release_metadata(self.metadata_bytes);
    }
}

impl InputView {
    pub fn capture(
        host: &HostInputs,
        checkout: &Path,
        source: &Path,
        manifests: &[&Manifest],
        generated_plm: bool,
        declarations: &Arc<Declarations>,
    ) -> Result<Arc<Self>> {
        decomp_source::context::limited(|| {
            bounded::safe(checkout)?;
            bounded::safe(source)?;
            let checkout = fs::canonicalize(checkout)?;
            let source = fs::canonicalize(source)?;
            let source_identity = decomp_source::verify_source(&checkout, &source)?;
            let namespace = bounded::entries(&checkout.join("crates/port/src"))?;
            ensure!(namespace.len() <= 4096, "captured gameplay namespace exceeded");
            let directory = std::env::temp_dir().join(format!(
                "sm-offline-inputs-{}-{}",
                std::process::id(),
                tool_execution::unique_id()
            ));
            bounded::safe(&directory)?;
            fs::create_dir(&directory)?;
            let mut view = Self {
                checkout,
                source,
                directory,
                files: Arc::new(BTreeMap::new()),
                namespace: Arc::new(namespace),
                source_identity: Arc::new(source_identity),
                declarations: declarations.clone(),
                builds: Arc::new(tool_execution::builds::BuildContext::default()),
                reserved_bytes: 0,
                metadata_bytes: 0,
                compiler_inputs: Arc::new(BTreeSet::new()),
                common_inputs: Arc::new(BTreeSet::new()),
                family_inputs: Arc::new(BTreeMap::new()),
                selected_inputs: None,
                retained: false,
                confined: false,
                parent: None,
            };
            let mut paths = BTreeMap::<PathBuf, Option<String>>::new();
            for (name, hash) in &view.source_identity.files {
                paths.insert(view.source.join(name), Some(hash.clone()));
            }
            for path in view.namespace.iter() {
                bounded::safe(path)?;
                if path.extension().is_some_and(|extension| extension == "rs") {
                    ensure!(path.symlink_metadata()?.is_file(), "captured gameplay input is not regular");
                }
            }
            for (name, _) in crate::inputs::required_sources(host) {
                paths.insert(view.checkout.join(name), None);
            }
            for name in [
                "tools/dependencies.toml",
                "tools/decomp/headless.c",
                "tools/decomp/headless_frame.h",
                "rust-toolchain.toml",
                "rust-toolchain",
            ] {
                let path = view.checkout.join(name);
                if path.exists() {
                    paths.insert(path, None);
                }
            }
            view.compiler_inputs = Arc::new(paths.keys().cloned().collect());
            view.common_inputs = view.compiler_inputs.clone();
            if generated_plm {
                Self::bind_asset(&mut paths, view.checkout.join("rom/sm.sfc"), crate::plm_fixtures::BASE_ROM_SHA256)?;
            }
            for manifest in manifests {
                let module = view.logical_input(Path::new(&manifest.rust_module));
                let selected = crate::inputs::ordinary_sources(&view.checkout, &[module], &manifest.rust_imports)?;
                let key = view.family_key(manifest);
                Arc::make_mut(&mut view.family_inputs).insert(key, selected.iter().cloned().collect());
                for path in selected {
                    Arc::make_mut(&mut view.compiler_inputs).insert(path.clone());
                    paths.insert(path, None);
                }
                for path in &manifest.rust_dependencies {
                    paths.entry(view.logical_input(path)).or_insert(None);
                }
                if let Some(rom) = &manifest.rom {
                    Self::bind_asset(&mut paths, view.logical_input(&rom.file), &rom.sha256)?;
                }
                for case in manifest.cases.iter().chain(manifest.routines.iter().flat_map(|routine| &routine.cases)) {
                    for (path, hash) in [
                        (&case.before_file, &case.before_sha256),
                        (&case.expected_after_file, &case.expected_after_sha256),
                    ] {
                        if let Some(path) = path {
                            Self::bind_asset(
                                &mut paths,
                                view.logical_input(path),
                                hash.as_deref().context("captured fixture identity missing")?,
                            )?;
                        }
                    }
                }
            }
            ensure!(paths.len() <= 16384, "captured input inventory exceeded");
            let metadata_bytes = view.namespace.iter().chain(paths.keys()).try_fold(4096u64, |total, path| {
                total
                    .checked_add(path.as_os_str().len() as u64 * 8 + 1024)
                    .context("captured source metadata extent overflow")
            })?;
            view.declarations.reserve_metadata(metadata_bytes)?;
            view.metadata_bytes += metadata_bytes;
            for (path, expected) in paths {
                view.capture_file(path, expected.as_deref())?;
            }
            for manifest in manifests {
                view.validate_rust_inputs(manifest)?;
            }
            view.validate_current()?;
            view.validate_materialized()?;
            Ok(Arc::new(view))
        })
    }

    /// Only the complete trusted PLM generator can extend a captured family.
    /// This is not a fallback for an input discovered during compilation.
    pub(crate) fn generated_plm(self: &Arc<Self>, generated: &crate::family_model::PlmGeneration) -> Result<Arc<Self>> {
        ensure!(
            !self.files.contains_key(&generated.rom)
                && generated.rom.file_name().is_some_and(|name| name == "plm-instructions-synthetic.sfc"),
            "generated PLM asset collides with captured inputs"
        );
        // The cloned inventory owns only path/hash metadata; source buffers
        // stay shared, with their original reservation held by the parent.
        let metadata_bytes = self.files.iter().try_fold(1024u64, |total, (path, file)| {
            total
                .checked_add((path.as_os_str().len() + file.physical.as_os_str().len()) as u64 * 2 + 512)
                .context("generated input metadata reservation overflow")
        })?;
        self.declarations.reserve_metadata(metadata_bytes)?;
        let mut view = Self {
            checkout: self.checkout.clone(),
            source: self.source.clone(),
            directory: self.directory.clone(),
            files: self.files.clone(),
            namespace: self.namespace.clone(),
            source_identity: self.source_identity.clone(),
            declarations: self.declarations.clone(),
            builds: self.builds.clone(),
            reserved_bytes: 0,
            metadata_bytes,
            compiler_inputs: self.compiler_inputs.clone(),
            common_inputs: self.common_inputs.clone(),
            family_inputs: self.family_inputs.clone(),
            selected_inputs: self.selected_inputs.clone(),
            retained: self.retained,
            confined: self.confined,
            parent: Some(self.clone()),
        };
        view.capture_file(generated.rom.clone(), Some(&generated.rom_sha256))?;
        Ok(Arc::new(view))
    }

    fn bind_asset(paths: &mut BTreeMap<PathBuf, Option<String>>, path: PathBuf, hash: &str) -> Result<()> {
        ensure!(
            hash.len() == 64 && hash.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "captured asset identity missing"
        );
        if let Some(Some(previous)) = paths.get(&path) {
            ensure!(previous == hash, "conflicting captured asset identities");
        }
        paths.insert(path, Some(hash.into()));
        Ok(())
    }

    fn capture_file(&mut self, path: PathBuf, expected: Option<&str>) -> Result<()> {
        bounded::safe(&path)?;
        let extent = fs::symlink_metadata(&path)?.len();
        let physical = self.destination(&path)?;
        fs::create_dir_all(physical.parent().context("captured input parent missing")?)?;
        let (bytes, hash, maximum) = if !self.compiler_inputs.contains(&path) {
            let maximum = if path == Path::new(crate::inputs::CAPTURED_RIDLEY_EVENT) {
                128 * 1024 * 1024
            } else {
                bounded::MAX_FILE_BYTES
            };
            let hash = if let Some(expected) = expected {
                expected.to_owned()
            } else {
                self.source_hash(&path, maximum, false)?
            };
            bounded::immutable_copy(&path, &physical, &hash, maximum)?;
            self.declarations.counters.source_reads.fetch_add(1, Ordering::Relaxed);
            self.declarations.counters.source_bytes.fetch_add(extent, Ordering::Relaxed);
            (None, hash, maximum)
        } else {
            self.declarations.reserve(extent)?;
            self.reserved_bytes += extent;
            let (bytes, hash) = bounded::read_with_digest_limit(&path, extent)?;
            self.declarations.counters.source_reads.fetch_add(1, Ordering::Relaxed);
            self.declarations.counters.source_bytes.fetch_add(bytes.len() as u64, Ordering::Relaxed);
            bounded::immutable_bytes(&physical, &bytes)?;
            (Some(Arc::new(bytes)), hash, bounded::MAX_FILE_BYTES)
        };
        if let Some(expected) = expected {
            ensure!(hash == expected, "captured input identity differs: {}", path.display());
        }
        Arc::make_mut(&mut self.files).insert(path, File { physical, sha256: hash, bytes, maximum, extent });
        Ok(())
    }

    fn destination(&self, path: &Path) -> Result<PathBuf> {
        if let Ok(relative) = path.strip_prefix(&self.source) {
            return Ok(self.physical_source().join(relative));
        }
        if let Ok(relative) = path.strip_prefix(&self.checkout) {
            return Ok(self.physical_checkout().join(relative));
        }
        // Preserve external asset basename while giving every logical path its
        // own namespace, independent of contents shared by declarations.
        let key = translation_execution::model::sha256(path.as_os_str().as_encoded_bytes());
        Ok(self.directory.join("external").join(key).join(path.file_name().context("external input filename missing")?))
    }

    pub fn checkout(&self) -> &Path {
        &self.checkout
    }
    pub fn source(&self) -> &Path {
        &self.source
    }
    pub fn source_identity(&self) -> &Source {
        &self.source_identity
    }
    pub(crate) fn build_context(&self) -> &tool_execution::builds::BuildContext {
        &self.builds
    }
    pub fn physical_checkout(&self) -> PathBuf {
        self.directory.join("checkout")
    }
    pub fn physical_source(&self) -> PathBuf {
        self.physical_checkout().join("deps/snesrev-sm")
    }

    fn logical_input(&self, path: &Path) -> PathBuf {
        if path.is_absolute() { path.into() } else { self.checkout.join(path) }
    }

    fn family_key(&self, manifest: &Manifest) -> (PathBuf, Vec<String>) {
        let mut imports = manifest.rust_imports.clone();
        imports.sort();
        imports.dedup();
        (self.logical_input(Path::new(&manifest.rust_module)), imports)
    }

    /// Select an already captured family without rediscovering or reading the
    /// shared compiler union. Full invocation checks stay at batch barriers.
    pub(crate) fn for_family(self: &Arc<Self>, manifest: &Manifest) -> Result<Arc<Self>> {
        let mut selected = (*self.common_inputs).clone();
        selected.extend(
            self.family_inputs
                .get(&self.family_key(manifest))
                .context("family source scope was not captured")?
                .iter()
                .cloned(),
        );
        let reserved_bytes = selected.iter().try_fold(512u64, |total, path| {
            total
                .checked_add(path.as_os_str().len() as u64 * 2 + 128)
                .context("family input metadata reservation overflow")
        })?;
        self.declarations.reserve_metadata(reserved_bytes)?;
        Ok(Arc::new(Self {
            checkout: self.checkout.clone(),
            source: self.source.clone(),
            directory: self.directory.clone(),
            files: self.files.clone(),
            namespace: self.namespace.clone(),
            source_identity: self.source_identity.clone(),
            declarations: self.declarations.clone(),
            builds: self.builds.clone(),
            reserved_bytes: 0,
            metadata_bytes: reserved_bytes,
            compiler_inputs: self.compiler_inputs.clone(),
            common_inputs: self.common_inputs.clone(),
            family_inputs: self.family_inputs.clone(),
            selected_inputs: Some(selected),
            parent: Some(self.clone()),
            retained: self.retained,
            confined: self.confined,
        }))
    }

    fn selected_compiler_inputs(&self) -> &BTreeSet<PathBuf> {
        self.selected_inputs.as_ref().unwrap_or(&self.compiler_inputs)
    }

    fn source_hash(&self, path: &Path, maximum: u64, freshness: bool) -> Result<String> {
        let bytes = fs::symlink_metadata(path)?.len();
        let hash = bounded::digest_limit(path, maximum)?;
        let counters = &self.declarations.counters;
        if freshness {
            counters.source_freshness_reads.fetch_add(1, Ordering::Relaxed);
            counters.source_freshness_bytes.fetch_add(bytes, Ordering::Relaxed);
        } else {
            counters.source_reads.fetch_add(1, Ordering::Relaxed);
            counters.source_bytes.fetch_add(bytes, Ordering::Relaxed);
        }
        Ok(hash)
    }

    fn materialized_hash(&self, file: &File) -> Result<String> {
        let hash = bounded::digest_limit(&file.physical, file.maximum)?;
        self.declarations.counters.materialized_reads.fetch_add(1, Ordering::Relaxed);
        self.declarations.counters.materialized_bytes.fetch_add(file.extent, Ordering::Relaxed);
        Ok(hash)
    }

    pub fn physical(&self, path: &Path) -> Result<PathBuf> {
        if let Some(file) = self.files.get(path) {
            return Ok(file.physical.clone());
        }
        if self.files.values().any(|file| file.physical == path) {
            return Ok(path.into());
        }
        anyhow::bail!("unbound captured compiler input: {}", path.display())
    }

    pub fn logical(&self, path: &Path) -> Result<PathBuf> {
        self.files
            .iter()
            .find(|(_, file)| file.physical == path)
            .map(|(logical, _)| logical.clone())
            .or_else(|| self.files.contains_key(path).then(|| path.into()))
            .with_context(|| format!("unbound compiler dependency: {}", path.display()))
    }

    pub fn read(&self, path: &Path, expected: Option<&str>) -> Result<Vec<u8>> {
        let logical = self.logical_input(path);
        let file = self
            .files
            .get(&logical)
            .or_else(|| self.files.values().find(|file| file.physical == path))
            .with_context(|| format!("unbound captured asset: {}", path.display()))?;
        if let Some(expected) = expected {
            ensure!(file.sha256 == expected, "captured asset identity differs");
        }
        if file.bytes.is_none() {
            ensure!(file.maximum <= bounded::MAX_FILE_BYTES, "large captured event requires bounded streaming");
            let (bytes, hash) = bounded::read_with_digest(&file.physical)?;
            ensure!(hash == file.sha256, "captured asset content changed");
            return Ok(bytes);
        }
        let bytes = file.bytes.as_ref().unwrap();
        // Returning a per-case image charges the execution scope while the
        // shared preparation owns the immutable original buffer.
        bounded::account(bytes.len() as u64)?;
        Ok(bytes.as_ref().clone())
    }

    pub fn validate_current(&self) -> Result<()> {
        ensure!(
            bounded::entries(&self.checkout.join("crates/port/src"))? == *self.namespace,
            "gameplay source namespace changed after capture"
        );
        for (path, file) in self.files.iter() {
            ensure!(
                self.source_hash(path, file.maximum, true)? == file.sha256,
                "source or asset changed after capture: {}",
                path.display()
            );
        }
        Ok(())
    }

    pub fn validate_materialized(&self) -> Result<()> {
        for path in self.selected_compiler_inputs() {
            let file = &self.files[path];
            ensure!(self.materialized_hash(file)? == file.sha256, "captured compiler bytes changed");
        }
        Ok(())
    }

    pub fn validate_compiler_current(&self) -> Result<()> {
        ensure!(
            bounded::entries(&self.checkout.join("crates/port/src"))? == *self.namespace,
            "gameplay source namespace changed after capture"
        );
        for path in self.selected_compiler_inputs() {
            let file = &self.files[path];
            ensure!(
                self.source_hash(path, file.maximum, true)? == file.sha256,
                "source changed after capture: {}",
                path.display()
            );
        }
        Ok(())
    }

    pub fn validate_manifest_inputs(&self, manifest: &Manifest) -> Result<()> {
        self.validate_compiler_current()?;
        let mut paths = manifest.rust_dependencies.iter().map(|path| self.logical_input(path)).collect::<BTreeSet<_>>();
        if let Some(rom) = &manifest.rom {
            paths.insert(self.logical_input(&rom.file));
        }
        for case in manifest.cases.iter().chain(manifest.routines.iter().flat_map(|routine| &routine.cases)) {
            for path in [&case.before_file, &case.expected_after_file].into_iter().flatten() {
                paths.insert(self.logical_input(path));
            }
        }
        for path in paths {
            let file = self.files.get(&path).context("family dependency was not captured")?;
            ensure!(
                self.source_hash(&path, file.maximum, true)? == file.sha256,
                "asset changed after capture: {}",
                path.display()
            );
        }
        Ok(())
    }

    pub(crate) fn validate_c_inputs(&self, paths: &[PathBuf], generated: &BTreeSet<PathBuf>) -> Result<()> {
        for path in paths {
            bounded::safe(path)?;
            if let Some((logical, file)) = self.files.iter().find(|(_, file)| &file.physical == path) {
                ensure!(
                    self.selected_compiler_inputs().contains(logical),
                    "C dependency is outside selected captured inputs"
                );
                ensure!(self.materialized_hash(file)? == file.sha256, "captured C dependency changed");
            } else if generated.contains(path) {
                ensure!(path.symlink_metadata()?.is_file(), "generated C input is not regular");
            } else {
                // These are the same installed header roots as the confined
                // compiler runtime. The generic preparation reads and freezes
                // every actual header identity and rechecks it after waits and
                // builds; membership alone never authenticates header bytes.
                ensure!(
                    ["/usr/include", "/usr/lib/gcc/x86_64-linux-gnu/9/include"]
                        .iter()
                        .any(|root| path.starts_with(root)),
                    "C dependency is outside captured inputs: {}",
                    path.display()
                );
                bounded::digest(path)?;
            }
        }
        Ok(())
    }

    fn validate_rust_inputs(&self, manifest: &Manifest) -> Result<()> {
        // Match ordinary wrappers, including standalone external candidates.
        // Native keeps its direct-module ownership check in canonical_sources.
        let available = self
            .namespace
            .iter()
            .filter_map(|path| {
                let name = path.file_stem()?.to_str()?;
                (path.extension().is_some_and(|extension| extension == "rs") && !["lib", "memory"].contains(&name))
                    .then(|| (name.to_owned(), path.clone()))
            })
            .collect::<BTreeMap<_, _>>();
        let mut pending =
            vec![self.logical_input(Path::new(&manifest.rust_module)), self.checkout.join("crates/port/src/memory.rs")];
        for name in &manifest.rust_imports {
            pending.push(available.get(name).context("unknown gameplay import")?.clone());
        }
        let mut visited = BTreeSet::new();
        while let Some(path) = pending.pop() {
            if !visited.insert(path.clone()) {
                continue;
            }
            bounded::check()?;
            let file = self.files.get(&path).context("Rust discovery found an unbound captured input")?;
            let facts =
                crate::source_facts::scan(&path, file.bytes.as_ref().context("Rust source cannot be streamed asset")?)?;
            ensure!(facts.sha256() == file.sha256, "captured Rust source facts differ from bound bytes");
            facts.require_complete()?;
            for word in facts.identifiers() {
                if let Some(path) = available.get(word) {
                    if !visited.contains(path) {
                        pending.push(path.clone());
                    }
                }
            }
            for dependency in facts.file_dependencies() {
                ensure!(self.files.contains_key(dependency), "Rust module path names an unbound captured input");
                if !visited.contains(dependency) {
                    pending.push(dependency.clone());
                }
            }
        }
        Ok(())
    }

    pub fn remap_arguments(&self, rust: bool) -> Vec<String> {
        // Confined Programs already expose the stable /inputs/rust and
        // /inputs/c paths. Host capture directories must never enter argv.
        if self.confined {
            return Vec::new();
        }
        let prefix = if rust { "--remap-path-prefix=" } else { "-ffile-prefix-map=" };
        let mut arguments = vec![
            format!("{prefix}{}={}", self.physical_checkout().display(), self.checkout.display()),
            format!("{prefix}{}={}", self.physical_source().display(), self.source.display()),
        ];
        for logical in self.selected_compiler_inputs() {
            if !logical.starts_with(&self.checkout) && !logical.starts_with(&self.source) {
                arguments.push(format!("{prefix}{}={}", self.files[logical].physical.display(), logical.display()));
            }
        }
        arguments
    }

    pub fn enter<T>(self: &Arc<Self>, action: impl FnOnce() -> Result<T>) -> Result<T> {
        struct Restore(Option<Arc<InputView>>);
        impl Drop for Restore {
            fn drop(&mut self) {
                CURRENT.with(|current| *current.borrow_mut() = self.0.take());
            }
        }
        let _restore = Restore(CURRENT.with(|current| current.borrow_mut().replace(self.clone())));
        action()
    }
}

pub(crate) fn current() -> Option<Arc<InputView>> {
    CURRENT.with(|current| current.borrow().clone())
}
