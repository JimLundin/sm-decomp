//! Native readiness run ownership and execution provenance.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

pub const RUN_VERSION: u32 = 2;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceFile {
    pub path: String,
    pub sha256: String,
    pub mode: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceInventory {
    pub schema_version: u32,
    pub sha256: String,
    pub files: Vec<SourceFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ProcessRecord {
    pub command: Vec<String>,
    pub cwd: PathBuf,
    pub started_utc: String,
    pub stdout: String,
    pub stderr: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub returncode: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signal: Option<i32>,
    pub complete: bool,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default)]
    pub worker_wait_seconds: f64,
    #[serde(default)]
    pub worker_lease: bool,
    #[serde(default)]
    pub elapsed_seconds: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_utc: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSeal {
    pub path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ProcessSeal {
    pub report: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

/// Check reports and origin-specific identities are heterogeneous TOML tables.
/// Ownership, process records, checksums and source inventory remain typed.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RunManifest {
    pub schema_version: u32,
    pub run_id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub scope: toml::Table,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_parent: Option<String>,
    pub started_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_utc: Option<String>,
    pub state: String,
    #[serde(default)]
    pub commands: Vec<ProcessRecord>,
    #[serde(default)]
    pub identities: BTreeMap<String, toml::Value>,
    #[serde(default)]
    pub check_evidence: BTreeMap<String, EvidenceSeal>,
    #[serde(default)]
    pub process_evidence: BTreeMap<String, ProcessSeal>,
    #[serde(default)]
    pub reused_evidence: Vec<toml::Table>,
    #[serde(default)]
    pub build_settings: toml::Table,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contract_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_baseline: Option<toml::Table>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment: Option<toml::Table>,
    #[serde(default)]
    pub parity_build_inputs: BTreeMap<String, toml::Table>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_profile_origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub increment_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_pointer: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Verdict {
    pub schema_version: u32,
    pub run_id: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub scope: toml::Table,
    pub checks: BTreeMap<String, toml::Value>,
    pub mandatory_checks: Vec<String>,
    pub acceptance_eligible: bool,
    pub finished_utc: String,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GateSummary {
    pub status: String,
    pub complete: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failures: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frames: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ArtifactIdentity {
    pub sha256: String,
    pub bytes: u64,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
}
