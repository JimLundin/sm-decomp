//! Typed decompressor admission contracts.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Fixture {
    pub id: String,
    pub source: u32,
    pub destination: usize,
    pub initial_fill: u8,
    pub stream: Vec<u8>,
    pub expected_bytes: Vec<u8>,
    pub expected_written: usize,
    pub expected_final_source: u32,
    pub expected_cases: BTreeSet<String>,
    pub captured: bool,
    pub source_offsets: Vec<usize>,
    pub stream_sha256: String,
    pub read_extents: Vec<[usize; 2]>,
    pub capture: Option<crate::baseline::CaptureCall>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Job {
    pub schema: String,
    pub routine: String,
    pub address: u64,
    pub source_revision: String,
    pub source_archive_sha256: String,
    pub notice: String,
    pub dependencies: Vec<String>,
    pub memory: BTreeMap<String, String>,
    pub numeric: BTreeMap<String, String>,
    pub limits: BTreeMap<String, u64>,
    pub required_cases: BTreeSet<String>,
    pub implemented_scope: BTreeSet<String>,
    pub exclusions: Vec<String>,
    pub fixtures: Vec<Fixture>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    pub checkout: PathBuf,
    pub output: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub timeout: u64,
    pub fault: Option<String>,
    pub manifest: Option<PathBuf>,
    pub captures: Option<PathBuf>,
    pub original_rom: Option<PathBuf>,
    pub accept: bool,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            checkout: PathBuf::new(),
            output: PathBuf::new(),
            root: PathBuf::new(),
            workers: 4,
            timeout: 60,
            fault: None,
            manifest: None,
            captures: None,
            original_rom: None,
            accept: false,
        }
    }
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FixtureRecord {
    pub id: String,
    pub status: String,
    pub fixture_sha256: String,
    pub rom_sha256: String,
    pub initial_sha256: String,
    pub expected_sha256: String,
    pub subject_hashes: BTreeMap<String, String>,
    pub observed_cases: BTreeSet<String>,
    pub c_ordered_writes_sha256: String,
    pub rust_ordered_writes_sha256: String,
    pub write_count: usize,
    pub write_set: BTreeSet<usize>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Fault {
    pub fault: String,
    pub main_sha256: String,
    pub lib_sha256: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Diagnostic {
    pub status: String,
    pub written: usize,
    pub final_source: u32,
    pub cases: BTreeSet<String>,
    pub instructions: u64,
    pub final_pc: u64,
    pub stack_written_min: usize,
    pub stack_written_max: usize,
    pub return_address: u64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StreamCheck {
    pub id: String,
    pub stream_sha256: String,
    pub bytes: usize,
    pub matches_unmodified_rom: bool,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OriginalRecord {
    pub diagnostic: Diagnostic,
    pub id: String,
    pub source: u32,
    pub destination: usize,
    pub stream_sha256: String,
    pub stream_matches_unmodified_rom: bool,
    pub excluded_offsets: BTreeSet<usize>,
    pub raw_mismatch_offsets: Vec<usize>,
    pub unexcluded_mismatch_offsets: Vec<usize>,
    pub sha256: String,
    pub status: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Original {
    pub status: String,
    pub rom_sha256: String,
    pub cpu_sha256: String,
    pub shim_sha256: String,
    pub binary_sha256: String,
    pub oracle: String,
    pub limit: String,
    pub fixtures: Vec<OriginalRecord>,
    pub ineligible: Vec<String>,
    pub source_stream_checks: Vec<StreamCheck>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Control {
    pub fault: String,
    pub rejected: bool,
    pub run: PathBuf,
    pub verdict_sha256: String,
    pub error: Option<String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Verdict {
    pub contract: String,
    pub status: String,
    pub routine_accepted: bool,
    pub run: PathBuf,
    pub stages: Vec<String>,
    pub fixtures: Vec<FixtureRecord>,
    pub observed_cases: BTreeSet<String>,
    pub missing_cases: BTreeSet<String>,
    pub original_rom_parity: Option<Original>,
    pub live_rust_ownership: String,
    pub manifest_sha256: String,
    pub source_revision: String,
    pub source_identities: BTreeMap<String, String>,
    pub implementation_identities: BTreeMap<String, String>,
    pub captures_manifest_sha256: Option<String>,
    pub fault_injection: Option<Fault>,
    pub binaries: BTreeMap<String, String>,
    pub synthetic_verified: bool,
    pub captured_calls_verified: usize,
    pub negative_controls: Vec<Control>,
    pub acceptance_sha256: Option<String>,
    pub error: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pointer {
    pub run: PathBuf,
    pub verdict_sha256: String,
    pub routine_accepted: bool,
    pub status: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contract {
    pub schema: String,
    pub routine: String,
    pub address: u64,
    pub job_sha256: String,
    pub capture_manifest_sha256: Option<String>,
    pub original_rom_sha256: String,
    pub synthetic_fixture_count: usize,
    pub captured_call_count: usize,
    pub original_call_count: usize,
    pub comparison: String,
    pub exclusions: Vec<String>,
    pub predecessor: String,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seal {
    pub contract: Contract,
    pub contract_sha256: String,
    pub run: PathBuf,
    pub job_sha256: String,
    pub source: BTreeMap<String, String>,
    pub implementation: BTreeMap<String, String>,
    pub binaries: BTreeMap<String, String>,
    pub fixtures: Vec<FixtureRecord>,
    pub observed_cases: BTreeSet<String>,
    pub negative_controls: Vec<Control>,
    pub capture_manifest_sha256: String,
    pub artifact_inventory_schema: String,
    pub artifacts: BTreeMap<String, String>,
    pub original_rom_parity: Original,
    pub status: String,
    pub routine_accepted: bool,
}
