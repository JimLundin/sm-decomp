//! Typed producer identities and readable v2 publication receipts.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub revision: String,
    pub archive_sha256: String,
    pub files: BTreeMap<String, String>,
    pub freshness: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binary_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reader_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rom_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inputs_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceIdentity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<u64>,
}
impl Default for Provenance {
    fn default() -> Self {
        Self {
            schema_version: 2,
            subject: None,
            compiler: None,
            binary_sha256: None,
            reader_sha256: None,
            rom_sha256: None,
            inputs_sha256: None,
            source_sha256: None,
            source: None,
            frames: None,
        }
    }
}
impl Provenance {
    pub fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(self.schema_version == 2, "unknown provenance version");
        for hash in
            [&self.binary_sha256, &self.reader_sha256, &self.rom_sha256, &self.inputs_sha256, &self.source_sha256]
                .into_iter()
                .flatten()
        {
            anyhow::ensure!(
                hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                "invalid provenance digest"
            );
        }
        if let Some(source) = &self.source {
            for hash in std::iter::once(&source.archive_sha256).chain(source.files.values()) {
                anyhow::ensure!(
                    hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()),
                    "invalid source provenance digest"
                );
            }
            anyhow::ensure!(
                !source.revision.is_empty() && !source.freshness.is_empty(),
                "missing source provenance identity"
            );
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Pointer {
    pub format: String,
    pub run: String,
    pub observations: String,
    pub files: u64,
    pub bytes: u64,
    pub artifacts: BTreeMap<String, String>,
    pub sizes: BTreeMap<String, u64>,
    #[serde(default)]
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pipeline_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub producer_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seal_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stored_added_bytes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worker_wait_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store_wait_seconds: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publication_seconds: Option<f64>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Inventory {
    pub observations: String,
    pub files: u64,
    pub bytes: u64,
    pub artifacts: BTreeMap<String, String>,
    pub provenance: Provenance,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct IngestReceipt {
    pub run: String,
    pub observations: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<String>,
    pub new_objects: u64,
    pub shared_objects: u64,
    pub seconds: f64,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ImportReceipt {
    pub run: String,
    pub observations: String,
    pub files: u64,
    pub bytes: u64,
    pub seconds: f64,
    pub provenance: Provenance,
    pub artifacts: BTreeMap<String, String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TraceReceipt {
    pub stream: String,
    pub bytes: u64,
    pub sha256: String,
}
