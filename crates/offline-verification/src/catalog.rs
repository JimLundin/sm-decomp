//! Read-only catalog declarations and bounded Fixture discovery.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use tool_execution::bounded;
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum CatalogEntry {
    Kind(String),
    Details(FamilyEntry),
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FamilyEntry {
    pub kind: String,
    pub manifest: PathBuf,
    pub live: Option<String>,
    pub prepare: Option<String>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Catalog {
    pub families: BTreeMap<String, CatalogEntry>,
    pub profiles: BTreeMap<String, Vec<String>>,
}
pub fn catalog(checkout: &Path) -> Result<Catalog> {
    Ok(capture(CatalogRequest::new(checkout)?)?.catalog().clone())
}

pub fn catalog_until(checkout: &Path, deadline: u64, max_bytes: u64) -> Result<Catalog> {
    Ok(capture(CatalogRequest {
        checkout: checkout.into(),
        deadline_unix_ms: deadline,
        max_input_bytes: max_bytes,
        max_retained_bytes: bounded::DEFAULT_BYTES,
    })?
    .catalog()
    .clone())
}

/// Keep catalog discovery identical for the supervisor's bounded source reader.
pub fn catalog_using(checkout: &Path, mut read: impl FnMut(&Path) -> Result<Vec<u8>>) -> Result<Catalog> {
    let mut config: Catalog = toml::from_str(std::str::from_utf8(&read(&checkout.join("tools/port/batch.toml"))?)?)?;
    let paths = bounded::entries(&checkout.join("tools/port"))?;
    ensure!(
        paths.len() <= 4096 && config.families.len() <= 4096 && config.profiles.len() <= 1024,
        "native catalog discovery inventory exceeded"
    );
    for path in paths {
        bounded::check()?;
        if path.extension().is_none_or(|v| v != crate::fixture_format::EXTENSION) {
            continue;
        }
        let name = path.file_stem().unwrap().to_string_lossy();
        if config.families.contains_key(name.as_ref()) {
            continue;
        }
        let manifest = crate::fixture_format::decode(read(&path)?)
            .with_context(|| format!("invalid discovery manifest {}", path.display()))?;
        let specialized = manifest.uses_dispatcher();
        config.families.insert(
            name.into_owned(),
            CatalogEntry::Details(FamilyEntry {
                kind: if specialized { "dispatcher" } else { "family" }.into(),
                manifest: path.strip_prefix(checkout)?.into(),
                live: None,
                prepare: None,
            }),
        );
        ensure!(config.families.len() <= 4096, "native catalog family inventory exceeded");
    }
    config.profiles.insert("offline-all".into(), config.families.keys().cloned().collect());
    Ok(config)
}
pub fn entry(config: &Catalog, name: &str) -> Result<FamilyEntry> {
    Ok(match config.families.get(name).with_context(|| format!("unknown family: {name}"))? {
        CatalogEntry::Kind(kind) => FamilyEntry {
            kind: kind.clone(),
            manifest: format!("tools/port/{name}.smf").into(),
            live: ["family", "callback", "projectile"].contains(&kind.as_str()).then(|| kind.clone()),
            prepare: None,
        },
        CatalogEntry::Details(item) => item.clone(),
    })
}

pub use crate::declarations::CatalogStatistics;
use crate::declarations::{Declaration, Declarations};

/// Bounds for capturing catalog/declaration inputs, separate from every
/// family's execution and independent evidence-admission scope.
pub struct CatalogRequest {
    pub checkout: PathBuf,
    pub deadline_unix_ms: u64,
    pub max_input_bytes: u64,
    pub max_retained_bytes: u64,
}

impl CatalogRequest {
    pub fn new(checkout: &Path) -> Result<Self> {
        Ok(Self {
            checkout: checkout.into(),
            deadline_unix_ms: bounded::now_unix_ms()?.checked_add(3_600_000).context("catalog deadline overflow")?,
            max_input_bytes: bounded::DEFAULT_BYTES,
            max_retained_bytes: bounded::DEFAULT_BYTES,
        })
    }
}

type Loaded = std::result::Result<Arc<Declaration>, String>;
struct Binding {
    path: PathBuf,
    identity: std::result::Result<String, String>,
    declaration: OnceLock<Loaded>,
}

struct CapturedCatalog {
    checkout: PathBuf,
    explicit: Catalog,
    catalog: Catalog,
    configuration_sha256: String,
    namespace: Vec<PathBuf>,
    bindings: BTreeMap<PathBuf, Arc<Binding>>,
    declarations: Arc<Declarations>,
    deadline: u64,
    maximum: u64,
    input_bytes: AtomicU64,
    exhausted: Mutex<Option<String>>,
}

/// One read-only invocation view. Clones share immutable decoded declarations
/// and per-path identities; this type cannot be constructed by deserialization.
#[derive(Clone)]
pub struct CatalogSnapshot(Arc<CapturedCatalog>);

/// A registered entry bound to its original declaration bytes. Recipe-specific
/// normalized manifests never mutate this shared declaration.
pub struct FamilyFacts {
    name: String,
    entry: FamilyEntry,
    declaration: Arc<Declaration>,
}

impl FamilyFacts {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn entry(&self) -> &FamilyEntry {
        &self.entry
    }
    pub fn manifest_sha256(&self) -> &str {
        &self.declaration.sha256
    }
    pub fn declaration(&self) -> &crate::fixtures::Manifest {
        &self.declaration.manifest
    }
    pub(crate) fn captured(&self) -> Arc<Declaration> {
        self.declaration.clone()
    }
}

pub fn capture(request: CatalogRequest) -> Result<CatalogSnapshot> {
    let CatalogRequest { checkout, deadline_unix_ms, max_input_bytes, max_retained_bytes } = request;
    ensure!(
        max_input_bytes > 0 && max_input_bytes <= 8 * bounded::DEFAULT_BYTES,
        "catalog input budget missing or excessive"
    );
    let declarations = Arc::new(Declarations::new(max_retained_bytes)?);
    decomp_source::context::until(deadline_unix_ms, max_input_bytes, || {
        bounded::safe(&checkout)?;
        let checkout = std::fs::canonicalize(checkout)?;
        let configuration = checkout.join("tools/port/batch.toml");
        let (bytes, configuration_sha256) = bounded::read_with_digest(&configuration)?;
        let mut input_bytes = bytes.len() as u64;
        declarations.reserve(bytes.len() as u64 * 8 + 4096)?;
        let explicit: Catalog = toml::from_str(std::str::from_utf8(&bytes)?)?;
        declarations.counters.catalog_parses.fetch_add(1, Ordering::Relaxed);
        let mut catalog = explicit.clone();
        let namespace = bounded::entries(&checkout.join("tools/port"))?;
        ensure!(
            namespace.len() <= 4096 && catalog.families.len() <= 4096 && catalog.profiles.len() <= 1024,
            "native catalog discovery inventory exceeded"
        );
        let mut bindings = BTreeMap::new();
        // Bind explicit paths now without constructing their potentially large
        // case vectors. Missing declarations remain lazy family errors, as in
        // the application catalog's existing metadata-only contract.
        for name in explicit.families.keys() {
            let path = checkout.join(entry(&explicit, name)?.manifest);
            if bindings.contains_key(&path) {
                continue;
            }
            let identity = (|| -> Result<String> {
                bounded::safe(&path)?;
                let bytes = std::fs::symlink_metadata(&path)?.len();
                input_bytes = input_bytes.checked_add(bytes).context("catalog input extent overflow")?;
                ensure!(input_bytes <= max_input_bytes, "catalog aggregate input budget exhausted");
                bounded::digest(&path)
            })()
            .map_err(|error| format!("{error:#}"));
            bounded::check()?;
            ensure!(input_bytes <= max_input_bytes, "catalog aggregate input budget exhausted");
            bindings.insert(path.clone(), Arc::new(Binding { path, identity, declaration: OnceLock::new() }));
        }
        for path in &namespace {
            bounded::check()?;
            if path.extension().is_none_or(|value| value != crate::fixture_format::EXTENSION) {
                continue;
            }
            let name = path.file_stem().context("discovery manifest stem missing")?.to_string_lossy();
            if catalog.families.contains_key(name.as_ref()) {
                continue;
            }
            let declaration = if let Some(binding) = bindings.get(path) {
                declarations.read(path, Some(binding.identity.as_ref().map_err(|error| anyhow::anyhow!("{error}"))?))?
            } else {
                declarations.read(path, None)?
            };
            let cell = OnceLock::new();
            let _ = cell.set(Ok(declaration.clone()));
            bindings.insert(
                path.clone(),
                Arc::new(Binding { path: path.clone(), identity: Ok(declaration.sha256.clone()), declaration: cell }),
            );
            catalog.families.insert(
                name.into_owned(),
                CatalogEntry::Details(FamilyEntry {
                    kind: if declaration.manifest.uses_dispatcher() { "dispatcher" } else { "family" }.into(),
                    manifest: path.strip_prefix(&checkout)?.into(),
                    live: None,
                    prepare: None,
                }),
            );
            ensure!(catalog.families.len() <= 4096, "native catalog family inventory exceeded");
        }
        input_bytes = input_bytes
            .checked_add(declarations.counters.input_bytes.load(Ordering::Relaxed))
            .context("catalog input extent overflow")?;
        ensure!(input_bytes <= max_input_bytes, "catalog aggregate input budget exhausted");
        declarations.counters.input_bytes.store(input_bytes, Ordering::Relaxed);
        catalog.profiles.insert("offline-all".into(), catalog.families.keys().cloned().collect());
        Ok(CatalogSnapshot(Arc::new(CapturedCatalog {
            checkout,
            explicit,
            catalog,
            configuration_sha256,
            namespace,
            bindings,
            declarations,
            deadline: deadline_unix_ms,
            maximum: max_input_bytes,
            input_bytes: AtomicU64::new(input_bytes),
            exhausted: Mutex::new(None),
        })))
    })
}

impl CatalogSnapshot {
    pub fn checkout(&self) -> &Path {
        &self.0.checkout
    }
    pub fn catalog(&self) -> &Catalog {
        &self.0.catalog
    }
    pub fn statistics(&self) -> CatalogStatistics {
        self.0.declarations.counters.statistics()
    }
    pub(crate) fn explicit(&self) -> &Catalog {
        &self.0.explicit
    }
    pub(crate) fn declarations(&self) -> &Arc<Declarations> {
        &self.0.declarations
    }

    /// Input refusal must stay a hard error in callers that otherwise turn
    /// missing dependency knowledge into conservative affected-family reasons.
    pub fn check_budget(&self) -> Result<()> {
        if let Some(error) = &*self.0.exhausted.lock().map_err(|_| anyhow::anyhow!("catalog budget lock poisoned"))? {
            anyhow::bail!("{error}");
        }
        Ok(())
    }

    pub fn family(&self, name: &str) -> Result<FamilyFacts> {
        self.check_budget()?;
        let entry = entry(&self.0.catalog, name)?;
        let binding =
            self.0.bindings.get(&self.0.checkout.join(&entry.manifest)).context("catalog path binding missing")?;
        let declaration = binding
            .declaration
            .get_or_init(|| {
                (|| -> Result<Arc<Declaration>> {
                    let expected = binding.identity.as_ref().map_err(|error| anyhow::anyhow!("{error}"))?;
                    let read = decomp_source::context::until(self.0.deadline, self.0.maximum, || {
                        bounded::safe(&binding.path)?;
                        let bytes = std::fs::symlink_metadata(&binding.path)?.len();
                        self.0
                            .input_bytes
                            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                                used.checked_add(bytes).filter(|next| *next <= self.0.maximum)
                            })
                            .map_err(|_| anyhow::anyhow!("catalog aggregate input budget exhausted"))?;
                        self.0.declarations.read(&binding.path, Some(expected))
                    });
                    if let Err(error) = &read {
                        if format!("{error:#}").contains("budget") || format!("{error:#}").contains("deadline") {
                            *self.0.exhausted.lock().unwrap() = Some(format!("{error:#}"));
                        }
                    }
                    read
                })()
                .map_err(|error| format!("{error:#}"))
            })
            .clone()
            .map_err(anyhow::Error::msg)?;
        Ok(FamilyFacts { name: name.into(), entry, declaration })
    }

    pub fn input_identities(&self) -> BTreeMap<PathBuf, String> {
        let mut result =
            BTreeMap::from([(PathBuf::from("tools/port/batch.toml"), self.0.configuration_sha256.clone())]);
        for binding in self.0.bindings.values() {
            if let Ok(hash) = &binding.identity {
                result
                    .insert(binding.path.strip_prefix(&self.0.checkout).unwrap_or(&binding.path).into(), hash.clone());
            }
        }
        result
    }

    fn current_hash(&self, path: &Path) -> Result<String> {
        bounded::safe(path)?;
        let bytes = std::fs::symlink_metadata(path)?.len();
        let hash = bounded::digest(path)?;
        self.0.declarations.counters.freshness_reads.fetch_add(1, Ordering::Relaxed);
        self.0.declarations.counters.freshness_bytes.fetch_add(bytes, Ordering::Relaxed);
        Ok(hash)
    }

    pub(crate) fn validate_family(&self, name: &str) -> Result<()> {
        decomp_source::context::limited(|| {
            let facts = self.family(name)?;
            let path = self.0.checkout.join(&facts.entry.manifest);
            ensure!(
                self.current_hash(&path)? == facts.manifest_sha256(),
                "Offline family manifest changed after capture"
            );
            Ok(())
        })
    }

    /// Rehash without redecoding. Exact configuration bytes are deliberately
    /// stricter than semantic TOML equality, including edits to comments.
    /// This is a fresh read scope, not the preparation scope's output budget.
    pub fn validate_current(&self) -> Result<()> {
        self.check_budget()?;
        decomp_source::context::limited(|| {
            ensure!(
                bounded::entries(&self.0.checkout.join("tools/port"))? == self.0.namespace,
                "Offline family catalog namespace changed after capture"
            );
            ensure!(
                self.current_hash(&self.0.checkout.join("tools/port/batch.toml"))? == self.0.configuration_sha256,
                "Offline family catalog changed after capture"
            );
            for binding in self.0.bindings.values() {
                if let Ok(expected) = &binding.identity {
                    ensure!(
                        self.current_hash(&binding.path)? == *expected,
                        "Offline family manifest changed after capture: {}",
                        binding.path.display()
                    );
                }
            }
            Ok(())
        })
    }
}
