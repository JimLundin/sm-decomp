//! Invocation-owned declarations. Path authentication and content decoding are
//! separate: aliases share decoded bytes, never another path's freshness check.
use crate::{fixture_format, fixtures::Manifest};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use tool_execution::bounded;

type Decoded = std::result::Result<Arc<Declaration>, String>;

pub(crate) struct Declaration {
    pub bytes: Arc<Vec<u8>>,
    pub sha256: String,
    pub manifest: Arc<Manifest>,
}

/// These counters describe work performed by this invocation, including failed
/// attempts. Freshness hashes are intentionally distinct from retained reads.
#[derive(Clone, Copy, Debug, Default, serde::Serialize, PartialEq, Eq)]
pub struct CatalogStatistics {
    pub catalog_parses: u64,
    pub manifest_reads: u64,
    pub manifest_decodes: u64,
    pub shared_content_hits: u64,
    pub input_bytes: u64,
    pub freshness_reads: u64,
    pub freshness_bytes: u64,
    /// Conservative reserved resident bytes, including decoded metadata.
    pub retained_bytes: u64,
}

#[derive(Default)]
pub(crate) struct Counters {
    pub catalog_parses: AtomicU64,
    pub manifest_reads: AtomicU64,
    pub manifest_decodes: AtomicU64,
    pub shared_content_hits: AtomicU64,
    pub input_bytes: AtomicU64,
    pub freshness_reads: AtomicU64,
    pub freshness_bytes: AtomicU64,
    retained_bytes: AtomicU64,
}

impl Counters {
    pub fn statistics(&self) -> CatalogStatistics {
        CatalogStatistics {
            catalog_parses: self.catalog_parses.load(Ordering::Relaxed),
            manifest_reads: self.manifest_reads.load(Ordering::Relaxed),
            manifest_decodes: self.manifest_decodes.load(Ordering::Relaxed),
            shared_content_hits: self.shared_content_hits.load(Ordering::Relaxed),
            input_bytes: self.input_bytes.load(Ordering::Relaxed),
            freshness_reads: self.freshness_reads.load(Ordering::Relaxed),
            freshness_bytes: self.freshness_bytes.load(Ordering::Relaxed),
            retained_bytes: self.retained_bytes.load(Ordering::Relaxed),
        }
    }
}

pub(crate) struct Declarations {
    maximum: u64,
    decoded: Mutex<BTreeMap<String, Arc<OnceLock<Decoded>>>>,
    pub counters: Counters,
}

impl Declarations {
    pub fn new(maximum: u64) -> Result<Self> {
        ensure!(maximum > 0 && maximum <= 8 * bounded::DEFAULT_BYTES, "declaration memory budget missing or excessive");
        Ok(Self { maximum, decoded: Mutex::new(BTreeMap::new()), counters: Counters::default() })
    }

    pub fn standard() -> Self {
        Self::new(bounded::DEFAULT_BYTES).expect("fixed declaration memory budget")
    }

    pub fn reserve(&self, amount: u64) -> Result<()> {
        self.counters
            .retained_bytes
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |used| {
                used.checked_add(amount).filter(|next| *next <= self.maximum)
            })
            .map_err(|_| anyhow::anyhow!("captured declaration resident memory budget exhausted"))?;
        Ok(())
    }

    pub fn decode(&self, bytes: Vec<u8>, sha256: String) -> Result<Arc<Declaration>> {
        let cell = {
            let mut decoded = self.decoded.lock().map_err(|_| anyhow::anyhow!("declaration cache lock poisoned"))?;
            decoded.entry(sha256.clone()).or_default().clone()
        };
        // Only this content's initializer holds the full decode. Other content
        // and path reads proceed independently of this once-cell.
        let mut initialized = false;
        let result = cell.get_or_init(|| {
            initialized = true;
            (|| -> Result<Arc<Declaration>> {
                self.reserve(fixture_format::retained_bound(&bytes)?)?;
                self.counters.manifest_decodes.fetch_add(1, Ordering::Relaxed);
                let bytes = Arc::new(bytes);
                let manifest = Arc::new(fixture_format::decode_shared(bytes.clone())?);
                Ok(Arc::new(Declaration { bytes, sha256, manifest }))
            })()
            .map_err(|error| format!("{error:#}"))
        });
        if !initialized {
            self.counters.shared_content_hits.fetch_add(1, Ordering::Relaxed);
        }
        result.clone().map_err(anyhow::Error::msg)
    }

    /// Every admission opens the retained file, even when its decoded content
    /// is already owned in this invocation. A cached digest cannot admit it.
    pub fn read(&self, path: &Path, expected: Option<&str>) -> Result<Arc<Declaration>> {
        let (bytes, hash) = bounded::read_with_digest(path)?;
        self.counters.manifest_reads.fetch_add(1, Ordering::Relaxed);
        self.counters.input_bytes.fetch_add(bytes.len() as u64, Ordering::Relaxed);
        if let Some(expected) = expected {
            ensure!(hash == expected, "captured declaration identity differs: {}", path.display());
        }
        self.decode(bytes, hash).with_context(|| format!("binary fixture {}", path.display()))
    }
}
