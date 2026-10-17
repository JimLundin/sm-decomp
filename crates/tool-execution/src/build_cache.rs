//! Shared immutable compiler facts, with operation-local publication and aborts.
use crate::{atomic_toml, bounded, digest, executable, metrics};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Mutex, MutexGuard, TryLockError,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CacheRequest {
    pub root: PathBuf,
    pub checkout: PathBuf,
    pub command: Vec<String>,
    pub inputs: Vec<PathBuf>,
    pub destination: PathBuf,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CacheResult {
    pub key: String,
    pub hit: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
struct CompilerIdentity {
    sha256: String,
    version: String,
}
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CacheIdentity {
    pub version: u32,
    command: Vec<String>,
    sources: BTreeMap<String, String>,
    compiler: CompilerIdentity,
    environment_sha256: String,
    platform: [String; 2],
    #[serde(skip_serializing_if = "Option::is_none")]
    rustc: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    working_directory: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path_contract: Option<String>,
}
#[derive(Deserialize, Serialize)]
pub struct CacheSeal {
    pub inputs: CacheIdentity,
    pub sha256: String,
}

/// The C recipe supplies typed path operands and real `-ffile-prefix-map`
/// arguments. Only those operands may use logical identity. Arbitrary argument
/// substrings (including macro values and remap targets) always remain literal.
#[derive(Default)]
pub(crate) struct ArtifactPaths {
    pub(crate) directory: Option<PathBuf>,
    mappings: Vec<(PathBuf, PathBuf)>,
}

impl ArtifactPaths {
    pub(crate) fn literal(directory: &Path) -> Result<Self> {
        Ok(Self { directory: Some(fs::canonicalize(directory)?), mappings: Vec::new() })
    }

    pub(crate) fn c(directory: &Path, flags: &[String]) -> Result<Self> {
        let mut paths = Self { directory: Some(fs::canonicalize(directory)?), mappings: Vec::new() };
        // Debug/switch recording can expose paths beyond macro/file remapping.
        // Keep physical identity until that larger contract has its own proof.
        if flags.iter().any(|flag| flag.starts_with("-g") || flag == "-frecord-gcc-switches") {
            return Ok(paths);
        }
        for flag in flags {
            if let Some(mapping) = flag.strip_prefix("-ffile-prefix-map=") {
                let Some((physical, logical)) = mapping.split_once('=') else { continue };
                let physical = PathBuf::from(physical);
                let logical = PathBuf::from(logical);
                if !physical.is_absolute() || !logical.is_absolute() || fs::canonicalize(&physical)? != physical {
                    // The actual flag remains in the key; no equivalence is
                    // claimed for relative or redirected mapping selectors.
                    paths.mappings.clear();
                    return Ok(paths);
                }
                paths.mappings.push((physical, logical));
            }
        }
        Ok(paths)
    }

    pub(crate) fn bind_locations(&mut self, inputs: &[PathBuf], locations: Option<&[PathBuf]>) {
        let Some(locations) = locations else {
            self.mappings.clear();
            return;
        };
        let unsupported = self.mappings.iter().any(|(physical, _)| {
            // Preserve unused selectors verbatim: they may affect a literal
            // #line filename or a lexical prefix outside a directory segment.
            !inputs.iter().any(|input| input.starts_with(physical))
                || locations.iter().any(|location| {
                    location.to_string_lossy().starts_with(physical.to_string_lossy().as_ref())
                        && !location.starts_with(physical)
                })
        });
        if unsupported {
            self.mappings.clear();
        }
    }

    fn logical(&self, path: &Path) -> PathBuf {
        self.mappings
            .iter()
            .rev()
            .find_map(|(physical, logical)| path.strip_prefix(physical).ok().map(|suffix| logical.join(suffix)))
            .unwrap_or_else(|| path.to_owned())
    }

    fn command(&self, command: &[String]) -> Result<Vec<String>> {
        if self.mappings.is_empty() {
            return Ok(command.to_vec());
        }
        let mut result = Vec::with_capacity(command.len());
        let mut path_operand = false;
        for (index, argument) in command.iter().enumerate() {
            if index == 0 {
                result.push(argument.clone());
            } else if path_operand {
                let path = Path::new(argument);
                ensure!(path.is_absolute(), "mapped compiler path operand is not absolute");
                result.push(self.logical(path).display().to_string());
                path_operand = false;
            } else if let Some(mapping) = argument.strip_prefix("-ffile-prefix-map=") {
                let (_, logical) = mapping.split_once('=').context("invalid compiler prefix map")?;
                result.push(format!("-ffile-prefix-map={{mapped-input-prefix}}={logical}"));
            } else {
                path_operand = ["-I", "-iquote", "-isystem", "-c"].contains(&argument.as_str());
                result.push(argument.clone());
            }
        }
        ensure!(!path_operand, "mapped compiler path operand missing");
        Ok(result)
    }
}

pub(crate) fn environment_digest() -> Result<String> {
    let environment: BTreeMap<_, _> = std::env::vars()
        .filter(|(name, _)| {
            name.starts_with("CARGO_")
                || name.starts_with("RUST")
                || name.starts_with("CC_")
                || [
                    "CC",
                    "AR",
                    "CFLAGS",
                    "CPPFLAGS",
                    "LDFLAGS",
                    "PATH",
                    "HOME",
                    "CPATH",
                    "C_INCLUDE_PATH",
                    "CPLUS_INCLUDE_PATH",
                    "LIBRARY_PATH",
                    "SOURCE_DATE_EPOCH",
                    "GCC_EXEC_PREFIX",
                    "COMPILER_PATH",
                    "SDKROOT",
                    "MACOSX_DEPLOYMENT_TARGET",
                ]
                .contains(&name.as_str())
        })
        .collect();
    Ok(format!("{:x}", Sha256::digest(toml::to_string(&environment)?.as_bytes())))
}

type CompilerContextKey = (PathBuf, String);
type CompilerContextSlot = Arc<Mutex<Option<Arc<CompilerContext>>>>;

/// One invocation shares immutable facts, not failures or producer authority.
/// Compatibility includes the canonical checkout and requested compiler name;
/// changed executable selection, bytes or environment invalidates the context.
#[derive(Default)]
pub(crate) struct CompilerPreparations {
    contexts: Mutex<BTreeMap<CompilerContextKey, CompilerContextSlot>>,
}

impl CompilerPreparations {
    pub(crate) fn prepare(&self, root: &Path, checkout: &Path, compiler_name: &str) -> Result<Preparation> {
        let checkout = fs::canonicalize(checkout)?;
        let key = (checkout.clone(), compiler_name.to_owned());
        let slot = {
            let mut contexts = checked_lock(&self.contexts)?;
            if !contexts.contains_key(&key) {
                ensure!(contexts.len() < 64, "shared compiler context extent exceeded");
            }
            contexts.entry(key).or_default().clone()
        };
        // Only the identical preparation waits here. Other compilers can make
        // progress, and no producer lease is held by a context waiter.
        let context = {
            let mut prepared = checked_lock(&slot)?;
            if let Some(context) = &*prepared {
                context.clone()
            } else {
                let context = Arc::new(CompilerContext::new(&checkout, compiler_name)?);
                *prepared = Some(context.clone());
                context
            }
        };
        context.authenticate()?;
        Ok(Preparation::with_context(root, context))
    }
}

pub(crate) fn checked_lock<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>> {
    loop {
        metrics::check_cancelled()?;
        bounded::check()?;
        match mutex.try_lock() {
            Ok(guard) => return Ok(guard),
            Err(TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(20)),
            Err(TryLockError::Poisoned(_)) => anyhow::bail!("compiler preparation lock poisoned"),
        }
    }
}

struct CompilerContext {
    compiler_name: String,
    compiler: PathBuf,
    identity: CompilerIdentity,
    environment_sha256: String,
    rustc: Option<String>,
    inputs: Mutex<BTreeMap<PathBuf, String>>,
}

/// Only this family owns its cancellation and observed closure. Shared input
/// facts freeze expected bytes; every cache wait/build/publication reopens the
/// family's actual inputs. Timestamps never replace authenticity checks.
pub struct Preparation {
    root: PathBuf,
    context: Arc<CompilerContext>,
    observed: Mutex<BTreeMap<PathBuf, String>>,
    aborted: AtomicBool,
}
#[derive(Debug)]
pub struct PreparationAborted;
impl std::fmt::Display for PreparationAborted {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("prepared build cancelled after another compiler failed")
    }
}
impl std::error::Error for PreparationAborted {}
impl CompilerContext {
    fn new(checkout: &Path, compiler_name: &str) -> Result<Self> {
        metrics::measure("compiler-preparation", || Self::prepare(checkout, compiler_name))
    }

    fn prepare(checkout: &Path, compiler_name: &str) -> Result<Self> {
        let checkout = fs::canonicalize(checkout)?;
        let environment_sha256 = environment_digest()?;
        // Preserve argv[0] for toolchain proxies such as rustup's rustc link.
        let compiler = executable(compiler_name)?;
        let compiler_sha256 = digest(&compiler)?;
        let version = metrics::command_output(
            Command::new(&compiler).arg("--version").current_dir(&checkout),
            "cache-compiler-identity",
        )?;
        ensure!(version.status.success(), "compiler version failed");
        let identity = CompilerIdentity { sha256: compiler_sha256, version: String::from_utf8(version.stdout)? };
        let rustc = if ["cargo", "rustc"]
            .contains(&Path::new(compiler_name).file_name().and_then(|s| s.to_str()).unwrap_or_default())
        {
            let rustc = if Path::new(compiler_name).file_name().is_some_and(|name| name == "rustc") {
                compiler.clone()
            } else {
                executable("rustc")?
            };
            let info = metrics::command_output(
                Command::new(&rustc).arg("-vV").current_dir(&checkout),
                "cache-rustc-identity",
            )?;
            ensure!(info.status.success(), "rustc version failed");
            Some(String::from_utf8(info.stdout)?)
        } else {
            None
        };
        let context = Self {
            compiler_name: compiler_name.into(),
            compiler,
            identity,
            environment_sha256,
            rustc,
            inputs: Mutex::new(BTreeMap::new()),
        };
        context.authenticate()?;
        Ok(context)
    }

    fn authenticate(&self) -> Result<()> {
        bounded::check()?;
        metrics::check_cancelled()?;
        ensure!(executable(&self.compiler_name)? == self.compiler, "prepared compiler selection changed");
        ensure!(digest(&self.compiler)? == self.identity.sha256, "prepared compiler changed");
        ensure!(environment_digest()? == self.environment_sha256, "prepared compiler environment changed");
        Ok(())
    }

    fn observe(&self, path: &Path) -> Result<String> {
        let mut inputs = checked_lock(&self.inputs)?;
        if let Some(hash) = inputs.get(path) {
            return Ok(hash.clone());
        }
        ensure!(inputs.len() < 65536, "shared compiler input extent exceeded");
        let hash = digest(path)?;
        inputs.insert(path.into(), hash.clone());
        Ok(hash)
    }
}

impl Preparation {
    pub fn new(root: &Path, checkout: &Path, compiler_name: &str) -> Result<Self> {
        Ok(Self::with_context(root, Arc::new(CompilerContext::new(checkout, compiler_name)?)))
    }

    fn with_context(root: &Path, context: Arc<CompilerContext>) -> Self {
        Self { root: root.into(), context, observed: Mutex::new(BTreeMap::new()), aborted: AtomicBool::new(false) }
    }

    /// Freeze known roots before invoking dependency discovery. The compiler's
    /// actual emitted closure must still be checked before publishing a miss.
    pub fn freeze(&self, inputs: &[PathBuf]) -> Result<()> {
        for path in inputs {
            self.observe(&fs::canonicalize(path)?)?;
        }
        Ok(())
    }

    /// Authenticate one already frozen closure after a capacity wait and before
    /// a producer starts. This cannot discover or silently bind another input.
    pub fn validate(&self, inputs: &[PathBuf]) -> Result<()> {
        self.check_active()?;
        let observed = self.observed.lock().map_err(|_| anyhow::anyhow!("compiler input lock poisoned"))?;
        for path in inputs {
            let path = fs::canonicalize(path)?;
            let expected = observed.get(&path).context("compiler input was not frozen before validation")?;
            ensure!(digest(&path)? == *expected, "prepared compiler input changed after wait");
        }
        self.context.authenticate()
    }

    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Release);
    }

    fn check_active(&self) -> Result<()> {
        if self.aborted.load(Ordering::Acquire) {
            return Err(PreparationAborted.into());
        }
        metrics::check_cancelled()?;
        bounded::check()
    }

    fn lock(&self, path: &Path) -> Result<(File, bool)> {
        let file = fs::OpenOptions::new().create(true).append(true).open(path)?;
        let mut waited = false;
        loop {
            self.check_active()?;
            match file.try_lock() {
                Ok(()) => return Ok((file, waited)),
                Err(std::fs::TryLockError::WouldBlock) => {
                    waited = true;
                    std::thread::sleep(Duration::from_millis(20));
                }
                Err(error) => return Err(error.into()),
            }
        }
    }

    fn observe(&self, path: &Path) -> Result<String> {
        self.check_active()?;
        let mut observed = self.observed.lock().map_err(|_| anyhow::anyhow!("compiler input lock poisoned"))?;
        if let Some(hash) = observed.get(path) {
            return Ok(hash.clone());
        }
        let hash = self.context.observe(path)?;
        observed.insert(path.into(), hash.clone());
        Ok(hash)
    }

    pub fn artifact(
        &self,
        command: &[String],
        inputs: &[PathBuf],
        destination: &Path,
        build: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<CacheResult> {
        self.artifact_checked(command, inputs, destination, &ArtifactPaths::default(), |_| Ok(()), build)
    }

    pub(crate) fn artifact_checked(
        &self,
        command: &[String],
        inputs: &[PathBuf],
        destination: &Path,
        paths: &ArtifactPaths,
        ready: impl FnOnce(bool) -> Result<()>,
        build: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<CacheResult> {
        metrics::measure("compiler-cache", || self.artifact_inner(command, inputs, destination, paths, ready, build))
    }

    fn artifact_inner(
        &self,
        command: &[String],
        inputs: &[PathBuf],
        destination: &Path,
        paths: &ArtifactPaths,
        ready: impl FnOnce(bool) -> Result<()>,
        build: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<CacheResult> {
        ensure!(command.first() == Some(&self.context.compiler_name), "prepared compiler differs");
        let context = metrics::active();
        let lookup = context.as_ref().map(|c| c.stage("cache-lookup")).transpose()?;
        let mut sources = BTreeMap::new();
        let mut originals = BTreeMap::new();
        for name in inputs {
            let path = fs::canonicalize(name)?;
            if originals.contains_key(&path) {
                continue;
            }
            let hash = self.observe(&path)?;
            let key = paths.logical(&path).to_string_lossy().into_owned();
            ensure!(sources.insert(key, hash.clone()).is_none(), "conflicting mapped compiler input");
            originals.insert(path, hash);
        }
        let identity = CacheIdentity {
            version: 5,
            command: paths.command(command)?,
            sources,
            compiler: self.context.identity.clone(),
            environment_sha256: self.context.environment_sha256.clone(),
            platform: [std::env::consts::ARCH.into(), std::env::consts::OS.into()],
            rustc: self.context.rustc.clone(),
            working_directory: paths.directory.as_ref().map(|path| paths.logical(path).display().to_string()),
            path_contract: (!paths.mappings.is_empty()).then(|| "c-file-prefix-map-v2".into()),
        };
        let key = format!("{:x}", Sha256::digest(toml::to_string(&identity)?.as_bytes()));
        let root = self.root.join("build-cache");
        fs::create_dir_all(&root)?;
        let waiting = context.as_ref().map(|c| c.stage("cache-lock-wait")).transpose()?;
        let (_held, waited) = self.lock(&root.join(format!("{key}.lock")))?;
        if let Some(waiting) = waiting {
            waiting.finish("success")?;
        }
        for (path, expected) in &originals {
            ensure!(digest(path)? == *expected, "compiler input changed while waiting for cache");
        }
        self.context.authenticate()?;
        ready(waited)?;
        let entry = root.join(&key);
        let authenticated = (|| -> Result<String> {
            ensure!(!entry.symlink_metadata()?.file_type().is_symlink(), "cache symlink");
            let seal: CacheSeal = pipeline_model::read(&entry.join("seal.toml"))?;
            ensure!(!entry.join("artifact").symlink_metadata()?.file_type().is_symlink(), "artifact symlink");
            ensure!(
                seal.inputs == identity && seal.sha256 == digest(&entry.join("artifact"))?,
                "cache artifact differs"
            );
            Ok(seal.sha256)
        })()
        .ok();
        let hit = authenticated.is_some();
        if let Some(lookup) = lookup {
            lookup.finish("success")?;
        }
        if let Some(context) = &context {
            context.configuration(&format!("cache-{key}-hit"), hit);
        }
        let stage = root.join(format!("{key}-{}.pending", std::process::id()));
        let expected = if let Some(hash) = authenticated {
            hash
        } else {
            if stage.exists() {
                fs::remove_dir_all(&stage)?;
            }
            fs::create_dir(&stage)?;
            let result = (|| -> Result<String> {
                self.context.authenticate()?;
                let building = context.as_ref().map(|c| c.stage("cache-build")).transpose()?;
                build(&stage.join("artifact"))?;
                if let Some(building) = building {
                    building.finish("success")?;
                }
                for (path, expected) in &originals {
                    ensure!(digest(path)? == *expected, "compiler input changed during build");
                }
                self.context.authenticate()?;
                ensure!(!stage.join("artifact").symlink_metadata()?.file_type().is_symlink(), "build artifact symlink");
                let hash = digest(&stage.join("artifact"))?;
                File::open(stage.join("artifact"))?.sync_all()?;
                atomic_toml(&stage.join("seal.toml"), &CacheSeal { inputs: identity, sha256: hash.clone() })?;
                if entry.exists() {
                    fs::remove_dir_all(&entry)?;
                }
                fs::rename(&stage, &entry)?;
                File::open(&root)?.sync_all()?;
                Ok(hash)
            })();
            if result.is_err() {
                let _ = fs::remove_dir_all(&stage);
            }
            result?
        };
        let restoring = context.as_ref().map(|c| c.stage("cache-restore-authenticate")).transpose()?;
        fs::create_dir_all(destination.parent().context("destination parent missing")?)?;
        fs::copy(entry.join("artifact"), destination)?;
        // Compare to the seal authenticated above, not a second read of mutable
        // cache bytes. This also detects replacement during the copy itself.
        ensure!(digest(destination)? == expected, "copied artifact differs");
        self.context.authenticate()?;
        if let Some(restoring) = restoring {
            restoring.finish("success")?;
        }
        Ok(CacheResult { key, hit })
    }

    /// Recheck the complete shared closure before handing a prepared build to
    /// its caller. Inputs changing between translation units fail the operation.
    pub fn finish(&self) -> Result<()> {
        metrics::measure("compiler-input-authentication", || self.authenticate_inputs())
    }

    fn authenticate_inputs(&self) -> Result<()> {
        let observed = self.observed.lock().map_err(|_| anyhow::anyhow!("compiler input lock poisoned"))?;
        for (path, hash) in observed.iter() {
            ensure!(digest(path)? == *hash, "prepared compiler input changed");
        }
        self.context.authenticate()
    }
}

pub fn cached_artifact(request: &CacheRequest, build: impl FnOnce(&Path) -> Result<()>) -> Result<CacheResult> {
    let compiler = request.command.first().context("compiler missing")?;
    let preparation = Preparation::new(&request.root, &request.checkout, compiler)?;
    let result = preparation.artifact(&request.command, &request.inputs, &request.destination, build)?;
    preparation.finish()?;
    Ok(result)
}
