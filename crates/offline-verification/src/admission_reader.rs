//! One independent retained inventory, with bounded owned comparison windows.
//! Namespace and source freshness are separate from reuse of authenticated bytes.
use crate::family::ArtifactLimits;
use anyhow::{Context, Result, ensure};
use serde::de::DeserializeOwned;
use std::{
    cell::Cell,
    collections::{BTreeMap, BTreeSet},
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};
use tool_execution::bounded;

// A window never grows with a family. Callback pairs release their payloads
// between observations; predecessor payloads live only in the current case.
const MAX_WINDOW_BYTES: u64 = 4 * bounded::MAX_FILE_BYTES;

struct Reservation {
    live_bytes: Rc<Cell<u64>>,
    bytes: u64,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        self.live_bytes.set(self.live_bytes.get() - self.bytes);
    }
}

#[derive(Clone)]
pub(crate) struct Payload {
    bytes: Arc<Vec<u8>>,
    hash: String,
    version: Arc<fs::Metadata>,
    _reservation: Rc<Reservation>,
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
        let names = discover(run, limits)?;
        ensure!(names.iter().eq(expected.keys()), "family artifact inventory differs");
        Ok(Self {
            run,
            expected,
            consumed: BTreeSet::new(),
            versions: BTreeMap::new(),
            limits,
            stats: ReadStats::default(),
        })
    }

    pub(crate) fn window(&mut self) -> Window<'_, 'a> {
        Window {
            run: self.run,
            admission: Some(self),
            payloads: BTreeMap::new(),
            live_bytes: Rc::new(Cell::new(0)),
            maximum: MAX_WINDOW_BYTES,
        }
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
    live_bytes: Rc<Cell<u64>>,
    maximum: u64,
}

impl<'r> Window<'_, 'r> {
    /// Immediate post-producer comparison has no published inventory yet.
    /// A later retained admission creates its own reader and reopens all bytes.
    pub(crate) fn unsealed(run: &'r Path) -> Self {
        Self {
            run,
            admission: None,
            payloads: BTreeMap::new(),
            live_bytes: Rc::new(Cell::new(0)),
            maximum: MAX_WINDOW_BYTES,
        }
    }

    pub(crate) fn read(&mut self, path: impl AsRef<Path>) -> Result<Payload> {
        self.read_limit(path.as_ref(), bounded::MAX_FILE_BYTES)
    }

    fn read_limit(&mut self, path: &Path, maximum: u64) -> Result<Payload> {
        ensure!(path.starts_with(self.run), "comparison input outside retained run");
        bounded::safe(path)?;
        if let Some(payload) = self.payloads.get(path) {
            ensure!(payload.len() as u64 <= maximum, "comparison input extent exceeded");
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
        ensure!(before.len() <= maximum, "comparison input extent exceeded");
        let total = self.live_bytes.get().checked_add(before.len()).context("comparison window extent overflow")?;
        ensure!(total <= self.maximum, "comparison window byte budget exhausted");
        self.live_bytes.set(total);
        let reservation = Rc::new(Reservation { live_bytes: self.live_bytes.clone(), bytes: before.len() });
        let (bytes, hash) = bounded::read_with_digest_limit(path, maximum)?;
        let version = Arc::new(path.symlink_metadata()?);
        ensure!(bounded::same_file_version(&before, &version), "family artifact changed during comparison");
        let payload = Payload { bytes: Arc::new(bytes), hash, version, _reservation: reservation };
        if let Some(admission) = &mut self.admission {
            admission.consume(path, &payload.hash, payload.version.clone())?;
            admission.stats.reads += 1;
            admission.stats.bytes += payload.len() as u64;
            admission.stats.peak_window_bytes = admission.stats.peak_window_bytes.max(total);
        }
        self.payloads.insert(path.to_path_buf(), payload.clone());
        Ok(payload)
    }

    pub(crate) fn text(&mut self, path: impl AsRef<Path>) -> Result<String> {
        let bytes = self.read_limit(path.as_ref(), bounded::MAX_DOCUMENT_BYTES)?;
        Ok(std::str::from_utf8(&bytes)?.to_owned())
    }

    pub(crate) fn object<T: DeserializeOwned>(&mut self, path: impl AsRef<Path>) -> Result<T> {
        Ok(toml::from_str(&self.text(path)?)?)
    }

    /// Consumed member seals remain in Admission. Any payload still held by
    /// its caller remains charged until its final owner releases it.
    pub(crate) fn clear(&mut self) {
        self.payloads.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seals(root: &Path, names: &[&str]) -> Result<BTreeMap<String, String>> {
        names.iter().map(|name| Ok(((*name).to_owned(), bounded::digest(&root.join(name))?))).collect()
    }

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

    #[test]
    fn unconsumed_bytes_and_final_namespace_are_required() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        fs::write(root.join("used"), b"used")?;
        fs::write(root.join("unused"), b"good")?;
        let expected = seals(root, &["used", "unused"])?;
        let mut admission = Admission::open(root, &expected, ArtifactLimits::default())?;
        admission.window().read(root.join("used"))?;
        fs::write(root.join("unused"), b"evil")?;
        assert!(admission.finish().is_err(), "uncompared bytes were trusted from the inventory claim");
        fs::write(root.join("unused"), b"good")?;
        let admission = Admission::open(root, &expected, ArtifactLimits::default())?;
        fs::write(root.join("undeclared"), b"extra")?;
        assert!(admission.finish().is_err(), "late undeclared member was admitted");
        assert!(Admission::open(root, &expected, ArtifactLimits::default()).is_err());
        fs::remove_file(root.join("undeclared"))?;
        fs::remove_file(root.join("unused"))?;
        assert!(Admission::open(root, &expected, ArtifactLimits::default()).is_err(), "missing member admitted");
        Ok(())
    }

    #[test]
    fn owned_bytes_do_not_authorize_mutation_or_a_later_admission() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        let path = root.join("member");
        fs::write(&path, b"good")?;
        let expected = seals(root, &["member"])?;
        let modified = path.metadata()?.modified()?;
        let mut admission = Admission::open(root, &expected, ArtifactLimits::default())?;
        {
            let mut window = admission.window();
            let owned = window.read(&path)?;
            fs::write(&path, b"evil")?;
            fs::File::options().write(true).open(&path)?.set_times(fs::FileTimes::new().set_modified(modified))?;
            assert_eq!(owned.as_ref(), b"good");
            assert!(window.read(&path).is_err(), "restored mtime authorized changed bytes inside a window");
        }
        assert!(admission.finish().is_err(), "changed consumed member survived final admission");
        let mut later = Admission::open(root, &expected, ArtifactLimits::default())?;
        assert!(later.window().read(&path).is_err(), "later admission used an earlier payload");
        fs::remove_file(&path)?;
        let outside = tool_execution::testing::Directory::new()?;
        fs::write(outside.0.join("member"), b"good")?;
        std::os::unix::fs::symlink(outside.0.join("member"), &path)?;
        assert!(Admission::open(root, &expected, ArtifactLimits::default()).is_err(), "redirected member admitted");
        Ok(())
    }

    #[test]
    fn retained_payload_owners_remain_charged_after_window_eviction() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        fs::write(root.join("first"), b"12345678")?;
        fs::write(root.join("second"), b"abcdefgh")?;
        let mut window = Window::unsealed(root);
        window.maximum = 8;
        let owned = window.read(root.join("first"))?;
        window.clear();
        assert!(window.read(root.join("second")).is_err(), "eviction hid a still-owned payload allocation");
        drop(owned);
        assert_eq!(window.read(root.join("second"))?.as_ref(), b"abcdefgh");
        assert_eq!(window.live_bytes.get(), 8);
        window.clear();
        assert_eq!(window.live_bytes.get(), 0);
        Ok(())
    }

    #[test]
    fn byte_deadline_and_namespace_limits_remain_hard_refusals() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let root = &directory.0;
        fs::write(root.join("first"), b"12345678")?;
        fs::write(root.join("second"), b"abcdefgh")?;
        let expected = seals(root, &["first", "second"])?;
        assert!(Admission::open(root, &expected, ArtifactLimits { files: 1, paths: 4 }).is_err());
        assert!(
            bounded::until(bounded::now_unix_ms()? + 5_000, 7, || {
                Admission::open(root, &expected, ArtifactLimits::default())?.finish()
            })
            .is_err()
        );
        assert!(
            bounded::until(bounded::now_unix_ms()?.saturating_sub(1), 1024, || {
                Admission::open(root, &expected, ArtifactLimits::default())?.finish()
            })
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn oversized_document_refuses_before_loading_or_charging_its_payload() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let path = directory.0.join("large.toml");
        fs::File::create(&path)?.set_len(bounded::MAX_DOCUMENT_BYTES + 1)?;
        bounded::until(bounded::now_unix_ms()? + 5_000, 1024, || {
            assert!(Window::unsealed(&directory.0).text(&path).is_err());
            assert_eq!(bounded::charged_bytes(), 0);
            Ok(())
        })
    }
}
