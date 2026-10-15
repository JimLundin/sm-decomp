//! Deterministic retained Native source views. The intent binds the complete
//! record before publication; independent readers only reopen retained bytes.
use super::*;
use crate::family_model::Asset;
use serde::{Deserialize, Serialize};

const RECORD: &str = "captured-inputs.toml";
const DIRECTORY: &str = "captured-inputs";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedFile {
    sha256: String,
    bytes: u64,
    maximum: u64,
    compiler: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    version: String,
    checkout: PathBuf,
    source: PathBuf,
    files: BTreeMap<PathBuf, CapturedFile>,
    namespace: Vec<PathBuf>,
}

/// The producer uses this plan only after its entire retained extent has been
/// reserved by the Native intent. The record is published after every file.
pub(crate) struct NativeCapturePlan {
    view: Arc<InputView>,
    run: PathBuf,
    record: Vec<u8>,
    reserved_metadata: u64,
    retained_bytes: u64,
}

impl Drop for NativeCapturePlan {
    fn drop(&mut self) {
        self.view.declarations.release_metadata(self.reserved_metadata);
    }
}

impl NativeCapturePlan {
    pub(crate) fn asset(&self) -> Asset {
        Asset { file: self.run.join(RECORD), sha256: translation_execution::model::sha256(&self.record) }
    }

    pub(crate) fn retained_bytes(&self) -> u64 {
        self.retained_bytes
    }

    pub(crate) fn publish(&self) -> Result<Arc<InputView>> {
        let mut captured = InputView {
            checkout: self.view.checkout.clone(),
            source: self.view.source.clone(),
            directory: self.run.join(DIRECTORY),
            files: Arc::new(BTreeMap::new()),
            namespace: self.view.namespace.clone(),
            source_identity: self.view.source_identity.clone(),
            declarations: self.view.declarations.clone(),
            builds: self.view.builds.clone(),
            reserved_bytes: 0,
            metadata_bytes: 0,
            compiler_inputs: self.view.compiler_inputs.clone(),
            common_inputs: self.view.common_inputs.clone(),
            family_inputs: self.view.family_inputs.clone(),
            selected_inputs: None,
            retained: true,
            confined: true,
            parent: Some(self.view.clone()),
        };
        let metadata = metadata_bound(self.view.files.keys())?;
        captured.declarations.reserve_metadata(metadata)?;
        captured.metadata_bytes = metadata;
        for (logical, file) in self.view.files.iter() {
            bounded::check()?;
            let destination = captured.destination(logical)?;
            fs::create_dir_all(destination.parent().context("retained input parent missing")?)?;
            if let Some(bytes) = &file.bytes {
                bounded::immutable_bytes(&destination, bytes)?;
            } else {
                bounded::immutable_copy(&file.physical, &destination, &file.sha256, file.maximum)?;
            }
            let mut file = file.clone();
            file.physical = destination;
            Arc::make_mut(&mut captured.files).insert(logical.clone(), file);
        }
        // Membership is exact, including failed/retried partial publication.
        captured.validate_retained_membership()?;
        bounded::immutable_bytes(&self.asset().file, &self.record)?;
        captured.validate_current()?;
        captured.validate_materialized()?;
        Ok(Arc::new(captured))
    }
}

fn metadata_bound<'a>(paths: impl IntoIterator<Item = &'a PathBuf>) -> Result<u64> {
    paths.into_iter().try_fold(8192u64, |bytes, path| {
        bytes.checked_add(path.as_os_str().len() as u64 * 12 + 2048).context("retained input metadata extent overflow")
    })
}

impl InputView {
    /// The caller derives this exact set from its prepared contract and current
    /// approved candidate. No compiler-discovered input may extend it.
    pub(crate) fn capture_native(
        checkout: &Path,
        source: &Path,
        source_identity: &Source,
        expected: &BTreeMap<PathBuf, String>,
        compiler_inputs: &BTreeSet<PathBuf>,
        manifest: &Manifest,
        declarations: &Arc<Declarations>,
    ) -> Result<Arc<Self>> {
        ensure!(expected.len() <= 16384, "native captured input inventory exceeded");
        ensure!(compiler_inputs.iter().all(|path| expected.contains_key(path)), "native compiler input is unbound");
        bounded::safe(checkout)?;
        bounded::safe(source)?;
        let namespace = bounded::entries(&checkout.join("crates/port/src"))?;
        let directory = std::env::temp_dir().join(format!(
            "sm-offline-native-inputs-{}-{}",
            std::process::id(),
            tool_execution::unique_id()
        ));
        bounded::safe(&directory)?;
        fs::create_dir(&directory)?;
        let mut view = Self {
            checkout: checkout.into(),
            source: source.into(),
            directory,
            files: Arc::new(BTreeMap::new()),
            namespace: Arc::new(namespace),
            source_identity: Arc::new(source_identity.clone()),
            declarations: declarations.clone(),
            builds: Arc::new(tool_execution::builds::BuildContext::default()),
            reserved_bytes: 0,
            metadata_bytes: 0,
            compiler_inputs: Arc::new(compiler_inputs.clone()),
            common_inputs: Arc::new(compiler_inputs.clone()),
            family_inputs: Arc::new(BTreeMap::new()),
            selected_inputs: None,
            retained: false,
            confined: true,
            parent: None,
        };
        let metadata = metadata_bound(expected.keys())?;
        view.declarations.reserve_metadata(metadata)?;
        view.metadata_bytes = metadata;
        for (path, hash) in expected {
            view.capture_file(path.clone(), Some(hash))?;
        }
        view.validate_rust_inputs(manifest)?;
        view.validate_current()?;
        view.validate_materialized()?;
        Ok(Arc::new(view))
    }

    pub(crate) fn plan_native_retention(self: &Arc<Self>, run: &Path) -> Result<NativeCapturePlan> {
        ensure!(self.confined && !self.retained, "native source capture is already retained");
        let reserved_metadata = metadata_bound(self.files.keys())?;
        self.declarations.reserve_metadata(reserved_metadata)?;
        let result = (|| {
            let record = Record {
                version: "offline-native-inputs-v1".into(),
                checkout: self.checkout.clone(),
                source: self.source.clone(),
                files: self
                    .files
                    .iter()
                    .map(|(logical, file)| {
                        (
                            logical.clone(),
                            CapturedFile {
                                sha256: file.sha256.clone(),
                                bytes: file.extent,
                                maximum: file.maximum,
                                compiler: self.compiler_inputs.contains(logical),
                            },
                        )
                    })
                    .collect(),
                namespace: self
                    .namespace
                    .iter()
                    .map(|path| Ok(path.strip_prefix(&self.checkout)?.to_path_buf()))
                    .collect::<Result<Vec<_>>>()?,
            };
            let record = toml::to_string(&record)?.into_bytes();
            ensure!(record.len() as u64 <= reserved_metadata, "native capture record exceeded reservation");
            let retained_bytes = self.files.values().try_fold(record.len() as u64, |bytes, file| {
                bytes.checked_add(file.extent).context("native captured input reservation overflow")
            })?;
            Ok(NativeCapturePlan { view: self.clone(), run: run.into(), record, reserved_metadata, retained_bytes })
        })();
        if result.is_err() {
            self.declarations.release_metadata(reserved_metadata);
        }
        result
    }

    pub(crate) fn native_compiler_roots(run: &Path) -> (PathBuf, PathBuf) {
        let checkout = run.join(DIRECTORY).join("checkout");
        let source = checkout.join("deps/snesrev-sm");
        (checkout, source)
    }

    pub(crate) fn open_native_retained(
        run: &Path,
        asset: &Asset,
        source: &Path,
        source_identity: &Source,
        current_checkout: &Path,
        declarations: &Arc<Declarations>,
        inputs: impl FnOnce(&Path) -> Result<(BTreeMap<PathBuf, String>, BTreeSet<PathBuf>)>,
    ) -> Result<Arc<Self>> {
        ensure!(asset.file == run.join(RECORD), "native capture record path changed");
        let extent = fs::symlink_metadata(&asset.file)?.len();
        let _buffer = declarations.transient(
            extent
                .checked_mul(12)
                .and_then(|n| n.checked_add(8192))
                .context("native capture parse reservation overflow")?,
        )?;
        let (bytes, hash) = bounded::read_with_digest_limit(&asset.file, extent.min(bounded::MAX_DOCUMENT_BYTES))?;
        ensure!(hash == asset.sha256, "native capture record identity changed");
        let record: Record = toml::from_str(std::str::from_utf8(&bytes)?)?;
        let logical_checkout = &record.checkout;
        bounded::safe(logical_checkout)?;
        ensure!(logical_checkout.is_absolute(), "native capture checkout is not absolute");
        let (expected, compiler_inputs) = inputs(logical_checkout)?;
        ensure!(
            record.version == "offline-native-inputs-v1"
                && record.source == source
                && expected.len() <= 16384
                && record.files.len() == expected.len(),
            "native capture input context changed"
        );
        ensure!(
            record.namespace.iter().all(|path| !path.as_os_str().is_empty()
                && path.components().all(|part| matches!(part, Component::Normal(_)))),
            "native captured source namespace alias"
        );
        let current_namespace = bounded::entries(&current_checkout.join("crates/port/src"))?;
        let namespace = record.namespace.iter().map(|path| current_checkout.join(path)).collect::<Vec<_>>();
        ensure!(namespace == current_namespace, "native source namespace changed");
        let metadata = metadata_bound(expected.keys())?;
        declarations.reserve_metadata(metadata)?;
        let mut view = Self {
            checkout: logical_checkout.into(),
            source: source.into(),
            directory: run.join(DIRECTORY),
            files: Arc::new(BTreeMap::new()),
            namespace: Arc::new(record.namespace.iter().map(|path| logical_checkout.join(path)).collect()),
            source_identity: Arc::new(source_identity.clone()),
            declarations: declarations.clone(),
            builds: Arc::new(tool_execution::builds::BuildContext::default()),
            reserved_bytes: 0,
            metadata_bytes: metadata,
            compiler_inputs: Arc::new(compiler_inputs.clone()),
            common_inputs: Arc::new(compiler_inputs.clone()),
            family_inputs: Arc::new(BTreeMap::new()),
            selected_inputs: None,
            retained: true,
            confined: true,
            parent: None,
        };
        for (logical, expected_hash) in &expected {
            let captured = record.files.get(logical).context("native captured input member missing")?;
            let maximum = if logical == Path::new(crate::inputs::CAPTURED_RIDLEY_EVENT) {
                128 * 1024 * 1024
            } else {
                bounded::MAX_FILE_BYTES
            };
            ensure!(
                captured.sha256 == *expected_hash
                    && captured.maximum == maximum
                    && captured.compiler == compiler_inputs.contains(logical)
                    && captured.bytes <= maximum,
                "native captured input binding changed: {}",
                logical.display()
            );
            let physical = view.destination(logical)?;
            bounded::safe(&physical)?;
            ensure!(fs::symlink_metadata(&physical)?.len() == captured.bytes, "native captured input extent changed");
            let file = File {
                physical,
                sha256: captured.sha256.clone(),
                bytes: None,
                maximum,
                extent: captured.bytes,
                rust_facts: OnceLock::new(),
            };
            ensure!(view.materialized_hash(&file)? == *expected_hash, "native captured input bytes changed");
            Arc::make_mut(&mut view.files).insert(logical.clone(), file);
        }
        view.validate_retained_membership()?;
        Ok(Arc::new(view))
    }

    pub(crate) fn retained_native_bytes(&self, record: &Asset) -> Result<u64> {
        self.files.values().try_fold(fs::symlink_metadata(&record.file)?.len(), |bytes, file| {
            bytes.checked_add(file.extent).context("native captured input reservation overflow")
        })
    }

    fn validate_retained_membership(&self) -> Result<()> {
        let expected = self.files.values().map(|file| file.physical.clone()).collect::<BTreeSet<_>>();
        let mut actual = BTreeSet::new();
        let mut pending = vec![(self.directory.clone(), 0usize)];
        while let Some((directory, depth)) = pending.pop() {
            ensure!(depth <= 64, "native captured input namespace depth exceeded");
            for path in bounded::entries(&directory)? {
                bounded::safe(&path)?;
                let metadata = fs::symlink_metadata(&path)?;
                if metadata.is_dir() {
                    ensure!(expected.iter().any(|file| file.starts_with(&path)), "extra native captured directory");
                    pending.push((path, depth + 1));
                } else {
                    ensure!(metadata.is_file() && expected.contains(&path), "extra native captured input member");
                    actual.insert(path);
                }
            }
        }
        ensure!(actual == expected, "native captured input membership incomplete");
        Ok(())
    }
}
