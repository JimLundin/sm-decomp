//! Shared bounded inputs, authenticated builds, process capacity and accounting.
use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

pub mod bounded;
pub mod build_cache;
pub mod builds;
mod leases;
pub mod metrics;
pub mod resource_sampler;
pub use leases::{acquire_lease, acquire_lease_until, acquire_leases_until};

#[cfg(any(test, feature = "test-support"))]
pub mod testing;

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// Process-local nonce for naming a caller's temporary publication.
pub fn unique_id() -> usize {
    NEXT.fetch_add(1, Ordering::Relaxed)
}

pub fn digest(path: &Path) -> Result<String> {
    let mut hash = Sha256::new();
    let mut input = File::open(path)?;
    let mut buffer = [0u8; 65536];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

pub fn lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new().create(true).append(true).open(path)?;
    metrics::lock(&file)?;
    Ok(file)
}

pub fn atomic_toml<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    ensure!(path.extension().is_some_and(|e| e == "toml"), "metadata must use .toml");
    let pending = path.with_extension(format!("pending-{}", std::process::id()));
    let result = (|| -> Result<()> {
        let mut out = OpenOptions::new().create_new(true).write(true).open(&pending)?;
        out.write_all(toml::to_string_pretty(value)?.as_bytes())?;
        out.sync_all()?;
        fs::rename(&pending, path)?;
        File::open(path.parent().context("path parent missing")?)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&pending);
    }
    result
}

pub fn executable(name: &str) -> Result<PathBuf> {
    if name.contains('/') {
        return Ok(PathBuf::from(name));
    }
    for directory in std::env::split_paths(&std::env::var_os("PATH").context("PATH missing")?) {
        let candidate = directory.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    bail!("compiler not found: {name}")
}

/// Exact compiled source inputs to retain in higher-level evidence contracts.
pub const IMPLEMENTATION_SOURCES: &[(&str, &str)] = &[
    ("crates/tool-execution/Cargo.toml", include_str!("../Cargo.toml")),
    ("crates/tool-execution/src/lib.rs", include_str!("lib.rs")),
    ("crates/tool-execution/src/bounded.rs", include_str!("bounded.rs")),
    ("crates/tool-execution/src/builds.rs", include_str!("builds.rs")),
    ("crates/tool-execution/src/build_cache.rs", include_str!("build_cache.rs")),
    ("crates/tool-execution/src/leases.rs", include_str!("leases.rs")),
    ("crates/tool-execution/src/metrics.rs", include_str!("metrics.rs")),
    ("crates/tool-execution/src/resource_sampler.rs", include_str!("resource_sampler.rs")),
];
