//! One independent retained inventory, with bounded owned comparison windows.
//! Namespace and source freshness are separate from reuse of authenticated bytes.
use crate::family::ArtifactLimits;
use anyhow::{Context, Result, ensure};
use serde::de::DeserializeOwned;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};
use tool_execution::bounded;

// A window never grows with a family. Callback pairs release their payloads
// between observations; predecessor payloads live only in the current case.
const MAX_WINDOW_BYTES: u64 = 4 * bounded::MAX_FILE_BYTES;

#[derive(Clone)]
pub(crate) struct Payload {
    bytes: Arc<Vec<u8>>,
    hash: String,
    version: Arc<fs::Metadata>,
}

impl Deref for Payload {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.bytes
    }
}
impl AsRef<[u8]> for Payload {
    fn as_ref(&self) -> &[u8] {
        self
    }
}
impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}
impl Payload {
    pub(crate) fn hash(&self) -> &str {
        &self.hash
    }
    pub(crate) fn shared_bytes(&self) -> Arc<Vec<u8>> {
        self.bytes.clone()
    }
}

#[derive(Default, Clone, Copy)]
pub(crate) struct ReadStats {
    pub reads: u64,
    pub bytes: u64,
    pub cache_hits: u64,
    pub peak_window_bytes: u64,
}

pub(crate) struct Admission<'a> {
    run: &'a Path,
    expected: &'a BTreeMap<String, String>,
    consumed: BTreeSet<String>,
    versions: BTreeMap<String, Arc<fs::Metadata>>,
    limits: ArtifactLimits,
    stats: ReadStats,
}

impl<'a> Admission<'a> {
    pub(crate) fn open(run: &'a Path, expected: &'a BTreeMap<String, String>, limits: ArtifactLimits) -> Result<Self> {
        limits.validate()?;
        let mut actual = BTreeMap::new();
        crate::family::inventory_with_limits(run, run, &mut actual, limits)?;
        ensure!(actual == *expected, "family artifact inventory differs");
        Ok(Self {
            run,
            expected,
            consumed: expected.keys().cloned().collect(),
            versions: BTreeMap::new(),
            limits,
            stats: ReadStats::default(),
        })
    }

    pub(crate) fn window(&mut self) -> Window<'_, 'a> {
        Window { run: self.run, admission: Some(self), payloads: BTreeMap::new(), bytes: 0 }
    }

    fn consume(&mut self, path: &Path, hash: &str, version: Arc<fs::Metadata>) -> Result<()> {
        let name = path.strip_prefix(self.run)?.to_str().context("family artifact name is not UTF-8")?;
        ensure!(self.expected.get(name).is_some_and(|expected| expected == hash), "family artifact changed: {name}");
        self.consumed.insert(name.to_owned());
        self.versions.insert(name.to_owned(), version);
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<ReadStats> {
        for (name, expected) in self.expected {
            bounded::check()?;
            if self.consumed.contains(name) {
                continue;
            }
            let path = self.run.join(name);
            let before = path.symlink_metadata()?;
            let hash = crate::inputs::artifact_digest(self.run, &path)?;
            let after = path.symlink_metadata()?;
            ensure!(bounded::same_file_version(&before, &after), "family artifact changed during admission: {name}");
            self.consume(&path, &hash, Arc::new(after))?;
            ensure!(hash == *expected, "family artifact inventory differs: {name}");
            self.stats.reads += 1;
            self.stats.bytes += before.len();
        }
        for (name, version) in &self.versions {
            let path = self.run.join(name);
            bounded::safe(&path)?;
            ensure!(
                bounded::same_file_version(version, &path.symlink_metadata()?),
                "family artifact changed after comparison: {name}"
            );
        }
        let current = discover(self.run, self.limits)?;
        ensure!(current.iter().eq(self.expected.keys()), "family artifact inventory differs");
        ensure!(self.consumed.len() == self.expected.len(), "family artifact authentication incomplete");
        bounded::check()?;
        Ok(self.stats)
    }
}

fn discover(run: &Path, limits: ArtifactLimits) -> Result<BTreeSet<String>> {
    fn visit(run: &Path, base: &Path, depth: usize, limit: usize, names: &mut BTreeSet<String>) -> Result<()> {
        ensure!(depth <= 32, "family artifact inventory depth exceeded");
        for path in bounded::entries(base)? {
            bounded::check()?;
            let metadata = path.symlink_metadata()?;
            ensure!(!metadata.file_type().is_symlink(), "run artifact symlink");
            if metadata.is_dir() {
                visit(run, &path, depth + 1, limit, names)?;
            } else if path != run.join("verdict.toml")
                && !path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(".metrics.toml"))
            {
                ensure!(metadata.is_file(), "invalid family artifact");
                let name = path.strip_prefix(run)?.to_str().context("family artifact name is not UTF-8")?;
                ensure!(names.insert(name.to_owned()), "duplicate family artifact");
                ensure!(names.len() <= limit, "family artifact inventory count exceeded");
            }
        }
        Ok(())
    }
    bounded::discovery_with_limit(limits.paths, || {
        let mut names = BTreeSet::new();
        visit(run, run, 0, limits.files, &mut names)?;
        Ok(names)
    })
}

pub(crate) struct Window<'a, 'r> {
    run: &'r Path,
    admission: Option<&'a mut Admission<'r>>,
    payloads: BTreeMap<PathBuf, Payload>,
    bytes: u64,
}

impl<'r> Window<'_, 'r> {
    /// Immediate post-producer comparison has no published inventory yet.
    /// A later retained admission creates its own reader and reopens all bytes.
    pub(crate) fn unsealed(run: &'r Path) -> Self {
        Self { run, admission: None, payloads: BTreeMap::new(), bytes: 0 }
    }

    pub(crate) fn read(&mut self, path: impl AsRef<Path>) -> Result<Payload> {
        let path = path.as_ref();
        ensure!(path.starts_with(self.run), "comparison input outside retained run");
        bounded::safe(path)?;
        if let Some(payload) = self.payloads.get(path) {
            ensure!(
                bounded::same_file_version(&payload.version, &path.symlink_metadata()?),
                "family artifact changed inside comparison window"
            );
            if let Some(admission) = &mut self.admission {
                admission.stats.cache_hits += 1;
            }
            return Ok(payload.clone());
        }
        let before = path.symlink_metadata()?;
        let total = self.bytes.checked_add(before.len()).context("comparison window extent overflow")?;
        ensure!(total <= MAX_WINDOW_BYTES, "comparison window byte budget exhausted");
        let (bytes, hash) = bounded::read_with_digest(path)?;
        let version = Arc::new(path.symlink_metadata()?);
        ensure!(bounded::same_file_version(&before, &version), "family artifact changed during comparison");
        let payload = Payload { bytes: Arc::new(bytes), hash, version };
        if let Some(admission) = &mut self.admission {
            admission.consume(path, &payload.hash, payload.version.clone())?;
            admission.stats.reads += 1;
            admission.stats.bytes += payload.len() as u64;
            admission.stats.peak_window_bytes = admission.stats.peak_window_bytes.max(total);
        }
        self.bytes = total;
        self.payloads.insert(path.to_path_buf(), payload.clone());
        Ok(payload)
    }

    pub(crate) fn text(&mut self, path: impl AsRef<Path>) -> Result<String> {
        let bytes = self.read(path)?;
        ensure!(bytes.len() as u64 <= bounded::MAX_DOCUMENT_BYTES, "comparison document extent exceeded");
        Ok(std::str::from_utf8(&bytes)?.to_owned())
    }

    pub(crate) fn object<T: DeserializeOwned>(&mut self, path: impl AsRef<Path>) -> Result<T> {
        Ok(toml::from_str(&self.text(path)?)?)
    }

    /// Call only after paired callback payloads leave scope. Consumed member
    /// seals remain in Admission; no callback family payload map is retained.
    pub(crate) fn clear(&mut self) {
        self.payloads.clear();
        self.bytes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_and_comparison_read_each_member_once_per_independent_admission() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        fs::write(root.join("before"), b"before state")?;
        fs::write(root.join("after"), b"after state")?;
        fs::write(root.join("diagnostic"), b"unconsumed diagnostic")?;
        let expected = ["before", "after", "diagnostic"]
            .into_iter()
            .map(|name| Ok((name.to_owned(), bounded::digest(&root.join(name))?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        let total = expected.keys().map(|name| fs::metadata(root.join(name)).unwrap().len()).sum::<u64>();
        for _ in 0..2 {
            bounded::until(bounded::now_unix_ms()? + 5_000, 1024, || {
                let mut admission = Admission::open(root, &expected, ArtifactLimits::default())?;
                {
                    let mut window = admission.window();
                    let before = window.read(root.join("before"))?;
                    let after = window.read(root.join("after"))?;
                    ensure!(before.as_ref() == b"before state" && after.as_ref() == b"after state", "wrong bytes");
                    ensure!(window.read(root.join("after"))? == after, "cached comparison differs");
                }
                admission.finish()?;
                assert_eq!(bounded::charged_bytes(), total, "one complete admission reread retained payload bytes");
                Ok(())
            })?;
        }
        Ok(())
    }
}
