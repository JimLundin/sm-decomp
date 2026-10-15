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
    sync::Arc,
};
use tool_execution::bounded;

thread_local! {
    static CURRENT: RefCell<Option<Arc<InputView>>> = const { RefCell::new(None) };
}

#[derive(Clone)]
struct File {
    physical: PathBuf,
    sha256: String,
    bytes: Option<Arc<Vec<u8>>>,
    maximum: u64,
}

/// The directory is private materialization of retained owned bytes. Neither
/// permissions nor remembered digests authenticate it: every compiler boundary
/// checks its bytes, and logical inputs are separately rechecked for freshness.
pub(crate) struct InputView {
    checkout: PathBuf,
    source: PathBuf,
    directory: PathBuf,
    files: BTreeMap<PathBuf, File>,
    namespace: Vec<PathBuf>,
    source_identity: Source,
    declarations: Arc<Declarations>,
    reserved_bytes: u64,
    compiler_inputs: BTreeSet<PathBuf>,
    /// A generated family may add a trusted asset while sharing the same
    /// compiler tree. Its parent owns that directory and the original bytes.
    parent: Option<Arc<InputView>>,
}

impl Drop for InputView {
    fn drop(&mut self) {
        // Only this invocation creates this directory. All family workers hold
        // an Arc, so it cannot be removed while a compiler or fixture uses it.
        if self.parent.is_none() {
            let _ = fs::remove_dir_all(&self.directory);
        }
        self.declarations.release(self.reserved_bytes);
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
                files: BTreeMap::new(),
                namespace,
                source_identity,
                declarations: declarations.clone(),
                reserved_bytes: 0,
                compiler_inputs: BTreeSet::new(),
                parent: None,
            };
            let mut paths = BTreeMap::<PathBuf, Option<String>>::new();
            for (name, hash) in &view.source_identity.files {
                paths.insert(view.source.join(name), Some(hash.clone()));
            }
            for path in &view.namespace {
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
            view.compiler_inputs = paths.keys().cloned().collect();
            if generated_plm {
                Self::bind_asset(&mut paths, view.checkout.join("rom/sm.sfc"), crate::plm_fixtures::BASE_ROM_SHA256)?;
            }
            for manifest in manifests {
                let module = view.logical_input(Path::new(&manifest.rust_module));
                for path in crate::inputs::ordinary_sources(&view.checkout, &[module], &manifest.rust_imports)? {
                    view.compiler_inputs.insert(path.clone());
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
        let reserved_bytes = (self.files.len() as u64)
            .checked_mul(16 * 1024)
            .and_then(|bytes| bytes.checked_add(16 * 1024))
            .context("generated input metadata reservation overflow")?;
        self.declarations.reserve(reserved_bytes)?;
        let mut view = Self {
            checkout: self.checkout.clone(),
            source: self.source.clone(),
            directory: self.directory.clone(),
            files: self.files.clone(),
            namespace: self.namespace.clone(),
            source_identity: self.source_identity.clone(),
            declarations: self.declarations.clone(),
            reserved_bytes,
            compiler_inputs: self.compiler_inputs.clone(),
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
        let physical = self.destination(&path)?;
        fs::create_dir_all(physical.parent().context("captured input parent missing")?)?;
        let (bytes, hash, maximum) = if !self.compiler_inputs.contains(&path) {
            let (hash, maximum) = if path == Path::new(crate::inputs::CAPTURED_RIDLEY_EVENT) {
                (crate::inputs::captured_ridley_event_digest(&path)?, 128 * 1024 * 1024)
            } else {
                (bounded::digest(&path)?, bounded::MAX_FILE_BYTES)
            };
            bounded::immutable_copy(&path, &physical, &hash, maximum)?;
            (None, hash, maximum)
        } else {
            let extent = fs::symlink_metadata(&path)?.len();
            self.declarations.reserve(extent)?;
            self.reserved_bytes += extent;
            let (bytes, hash) = bounded::read_with_digest(&path)?;
            // Absolute C includes would bypass the captured tree. The existing
            // conservative contract deliberately refuses ambiguous directives;
            // syntax-aware precision belongs to dependency discovery.
            if path.starts_with(&self.source) && path.extension().is_some_and(|ext| ext == "c" || ext == "h") {
                for line in std::str::from_utf8(&bytes)?.lines() {
                    let line = line.trim_start();
                    if line.starts_with('#') && line[1..].trim_start().starts_with("include") {
                        let include = line.split_once("include").unwrap().1.trim_start();
                        ensure!(
                            !include.starts_with("\"/") && !include.starts_with("</"),
                            "absolute C include is outside captured inputs"
                        );
                        if let Some(name) = include.strip_prefix('"').and_then(|text| text.split('"').next()) {
                            let mut resolved = path.parent().unwrap().to_path_buf();
                            for component in Path::new(name).components() {
                                match component {
                                    Component::Normal(name) => resolved.push(name),
                                    Component::ParentDir => {
                                        ensure!(resolved.pop(), "escaping C include");
                                    }
                                    Component::CurDir => {}
                                    _ => anyhow::bail!("absolute C include is outside captured inputs"),
                                }
                            }
                            ensure!(
                                resolved.starts_with(&self.source),
                                "escaping C include is outside captured inputs"
                            );
                        }
                    }
                }
            }
            bounded::immutable_bytes(&physical, &bytes)?;
            (Some(Arc::new(bytes)), hash, bounded::MAX_FILE_BYTES)
        };
        if let Some(expected) = expected {
            ensure!(hash == expected, "captured input identity differs: {}", path.display());
        }
        self.files.insert(path, File { physical, sha256: hash, bytes, maximum });
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
    pub fn physical_checkout(&self) -> PathBuf {
        self.directory.join("checkout")
    }
    pub fn physical_source(&self) -> PathBuf {
        self.physical_checkout().join("deps/snesrev-sm")
    }

    fn logical_input(&self, path: &Path) -> PathBuf {
        if path.is_absolute() { path.into() } else { self.checkout.join(path) }
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
            bounded::entries(&self.checkout.join("crates/port/src"))? == self.namespace,
            "gameplay source namespace changed after capture"
        );
        for (path, file) in &self.files {
            ensure!(
                bounded::digest_limit(path, file.maximum)? == file.sha256,
                "source or asset changed after capture: {}",
                path.display()
            );
        }
        Ok(())
    }

    pub fn validate_materialized(&self) -> Result<()> {
        for file in self.files.iter().filter(|(path, _)| self.compiler_inputs.contains(*path)).map(|(_, file)| file) {
            if let Some(bytes) = &file.bytes {
                ensure!(
                    bounded::read(&file.physical)?.as_slice() == bytes.as_slice(),
                    "captured compiler bytes changed"
                );
            } else {
                ensure!(
                    bounded::digest_limit(&file.physical, file.maximum)? == file.sha256,
                    "captured streamed input changed"
                );
            }
        }
        Ok(())
    }

    pub fn validate_compiler_current(&self) -> Result<()> {
        ensure!(
            bounded::entries(&self.checkout.join("crates/port/src"))? == self.namespace,
            "gameplay source namespace changed after capture"
        );
        for path in &self.compiler_inputs {
            let file = &self.files[path];
            ensure!(bounded::digest(path)? == file.sha256, "source changed after capture: {}", path.display());
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
                bounded::digest_limit(&path, file.maximum)? == file.sha256,
                "asset changed after capture: {}",
                path.display()
            );
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
            let text = std::str::from_utf8(file.bytes.as_ref().context("Rust source cannot be streamed asset")?)?;
            for word in text.split(|value: char| !(value.is_ascii_alphanumeric() || value == '_')) {
                if let Some(path) = available.get(word) {
                    if !visited.contains(path) {
                        pending.push(path.clone());
                    }
                }
            }
            ensure!(
                !["include!", "include_str!", "include_bytes!", "env!", "option_env!"]
                    .iter()
                    .any(|token| text.contains(token)),
                "captured Rust source has an unsupported include or build-time environment dependency"
            );
            for attribute in text.split("#[path").skip(1) {
                let attribute = attribute.split_once(']').context("unsupported Rust path attribute")?.0;
                let name = attribute.split('"').nth(1).context("unsupported Rust path attribute")?;
                let relative = Path::new(name);
                ensure!(
                    !relative.is_absolute() && relative.components().all(|part| matches!(part, Component::Normal(_))),
                    "absolute or escaping Rust source path is outside captured inputs"
                );
                let dependency = path.parent().unwrap().join(relative);
                ensure!(self.files.contains_key(&dependency), "Rust path attribute names an unbound captured input");
            }
        }
        Ok(())
    }

    pub fn remap_arguments(&self, rust: bool) -> Vec<String> {
        let prefix = if rust { "--remap-path-prefix=" } else { "-ffile-prefix-map=" };
        let mut arguments = vec![
            format!("{prefix}{}={}", self.physical_checkout().display(), self.checkout.display()),
            format!("{prefix}{}={}", self.physical_source().display(), self.source.display()),
        ];
        for logical in &self.compiler_inputs {
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
