//! Authenticated artifact preparation for one frozen compiler operation.
use crate::{atomic_toml, digest, executable, metrics};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Mutex,
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
}
#[derive(Deserialize, Serialize)]
pub struct CacheSeal {
    pub inputs: CacheIdentity,
    pub sha256: String,
}

fn environment_digest() -> Result<String> {
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

/// The preparation is deliberately operation-local. Reusing an input digest
/// here freezes its expected bytes; it never bypasses validation after a lock
/// wait or build. No filesystem timestamps are used as an authenticity cache.
pub struct Preparation {
    root: PathBuf,
    checkout: PathBuf,
    repository: PathBuf,
    compiler_name: String,
    compiler: PathBuf,
    identity: CompilerIdentity,
    environment_sha256: String,
    rustc: Option<String>,
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
impl Preparation {
    pub fn new(root: &Path, checkout: &Path, compiler_name: &str) -> Result<Self> {
        metrics::measure("compiler-preparation", || Self::prepare(root, checkout, compiler_name))
    }

    fn prepare(root: &Path, checkout: &Path, compiler_name: &str) -> Result<Self> {
        let checkout = fs::canonicalize(checkout)?;
        let common = metrics::command_output(
            Command::new("git").args(["rev-parse", "--git-common-dir"]).current_dir(&checkout),
            "cache-repository-identity",
        )?;
        let repository = if common.status.success() {
            let common = PathBuf::from(String::from_utf8(common.stdout)?.trim());
            fs::canonicalize(checkout.join(common))?.parent().context("git common parent missing")?.to_path_buf()
        } else {
            checkout.clone()
        };
        // Preserve argv[0] for toolchain proxies such as rustup's rustc link.
        let compiler = executable(compiler_name)?;
        let version = metrics::command_output(
            Command::new(&compiler).arg("--version").current_dir(&checkout),
            "cache-compiler-identity",
        )?;
        ensure!(version.status.success(), "compiler version failed");
        let identity = CompilerIdentity { sha256: digest(&compiler)?, version: String::from_utf8(version.stdout)? };
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
        Ok(Self {
            root: root.into(),
            checkout,
            repository,
            compiler_name: compiler_name.into(),
            compiler,
            identity,
            environment_sha256: environment_digest()?,
            rustc,
            observed: Mutex::new(BTreeMap::new()),
            aborted: AtomicBool::new(false),
        })
    }

    /// Freeze known roots before invoking dependency discovery. The compiler's
    /// actual emitted closure must still be checked before publishing a miss.
    pub fn freeze(&self, inputs: &[PathBuf]) -> Result<()> {
        for path in inputs {
            self.observe(&fs::canonicalize(path)?)?;
        }
        Ok(())
    }

    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Release);
    }

    fn check_active(&self) -> Result<()> {
        if self.aborted.load(Ordering::Acquire) {
            return Err(PreparationAborted.into());
        }
        metrics::check_cancelled()
    }

    fn lock(&self, path: &Path) -> Result<File> {
        let file = fs::OpenOptions::new().create(true).append(true).open(path)?;
        loop {
            self.check_active()?;
            match file.try_lock() {
                Ok(()) => return Ok(file),
                Err(std::fs::TryLockError::WouldBlock) => std::thread::sleep(Duration::from_millis(20)),
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
        let hash = digest(path)?;
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
        metrics::measure("compiler-cache", || self.artifact_inner(command, inputs, destination, build))
    }

    fn artifact_inner(
        &self,
        command: &[String],
        inputs: &[PathBuf],
        destination: &Path,
        build: impl FnOnce(&Path) -> Result<()>,
    ) -> Result<CacheResult> {
        ensure!(command.first() == Some(&self.compiler_name), "prepared compiler differs");
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
            let key = path
                .strip_prefix(&self.checkout)
                .or_else(|_| path.strip_prefix(&self.repository))
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            if let Some(previous) = sources.insert(key, hash.clone()) {
                ensure!(previous == hash, "conflicting normalized compiler input");
            }
            originals.insert(path, hash);
        }
        let identity = CacheIdentity {
            version: 4,
            command: command
                .iter()
                .map(|arg| {
                    arg.replace(self.checkout.to_string_lossy().as_ref(), "{checkout}")
                        .replace(self.repository.to_string_lossy().as_ref(), "{checkout}")
                })
                .collect(),
            sources,
            compiler: self.identity.clone(),
            environment_sha256: self.environment_sha256.clone(),
            platform: [std::env::consts::ARCH.into(), std::env::consts::OS.into()],
            rustc: self.rustc.clone(),
        };
        let key = format!("{:x}", Sha256::digest(toml::to_string(&identity)?.as_bytes()));
        let root = self.root.join("build-cache");
        fs::create_dir_all(&root)?;
        let waiting = context.as_ref().map(|c| c.stage("cache-lock-wait")).transpose()?;
        let _held = self.lock(&root.join(format!("{key}.lock")))?;
        if let Some(waiting) = waiting {
            waiting.finish("success")?;
        }
        for (path, expected) in &originals {
            ensure!(digest(path)? == *expected, "compiler input changed while waiting for cache");
        }
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
                ensure!(digest(&self.compiler)? == self.identity.sha256, "prepared compiler changed before build");
                ensure!(environment_digest()? == self.environment_sha256, "prepared compiler environment changed");
                let building = context.as_ref().map(|c| c.stage("cache-build")).transpose()?;
                build(&stage.join("artifact"))?;
                if let Some(building) = building {
                    building.finish("success")?;
                }
                for (path, expected) in &originals {
                    ensure!(digest(path)? == *expected, "compiler input changed during build");
                }
                ensure!(digest(&self.compiler)? == self.identity.sha256, "prepared compiler changed during build");
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
        ensure!(digest(&self.compiler)? == self.identity.sha256, "prepared compiler changed");
        ensure!(environment_digest()? == self.environment_sha256, "prepared compiler environment changed");
        Ok(())
    }
}

pub fn cached_artifact(request: &CacheRequest, build: impl FnOnce(&Path) -> Result<()>) -> Result<CacheResult> {
    let compiler = request.command.first().context("compiler missing")?;
    let preparation = Preparation::new(&request.root, &request.checkout, compiler)?;
    let result = preparation.artifact(&request.command, &request.inputs, &request.destination, build)?;
    preparation.finish()?;
    Ok(result)
}
