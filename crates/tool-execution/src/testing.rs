//! Filesystem fixtures for tests of consumers' bounded operations.
use anyhow::Result;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
pub struct Directory(pub PathBuf);
impl Directory {
    pub fn new() -> Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "sm-bounded-fixture-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
