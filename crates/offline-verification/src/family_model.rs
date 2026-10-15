//! Typed fixture and offline family evidence contracts.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
#[path = "fixture_writes.rs"]
mod writes;
pub use writes::Writes;
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Write {
    pub offset: usize,
    pub data: Vec<u8>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Case {
    pub name: String,
    pub a: i64,
    pub b: i64,
    pub fill: Option<u8>,
    pub writes: Writes,
    pub host_words: Option<Vec<u16>>,
    pub continue_from: Option<String>,
    pub expected_return: Option<u64>,
    pub expected_amount: Option<i64>,
    pub expected_flags: Option<i64>,
    pub minimum_callbacks: u64,
    pub allocator: Option<String>,
    pub before_file: Option<PathBuf>,
    pub before_sha256: Option<String>,
    pub expected_after_file: Option<PathBuf>,
    pub expected_after_sha256: Option<String>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Routine {
    pub symbol: String,
    pub args: usize,
    pub arg_types: Option<Vec<String>>,
    #[serde(rename = "return")]
    pub returns: String,
    pub return_type: Option<String>,
    pub domain: Option<String>,
    pub cases: Vec<Case>,
    pub live_eligible: Option<bool>,
    pub live_limitation: Option<String>,
    pub pointer_bank: Option<u64>,
    pub packed_u16_args: Option<u64>,
    pub packed_args: Option<String>,
    pub pair_u16_return: bool,
    pub original_address: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub file: PathBuf,
    pub sha256: String,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Manifest {
    pub family: String,
    pub domain: Option<String>,
    pub rust_module: String,
    pub bank: String,
    pub c_support: Vec<String>,
    pub rust_imports: Vec<String>,
    pub rust_dependencies: Vec<PathBuf>,
    pub rom: Option<Asset>,
    pub routines: Vec<Routine>,
    pub cases: Vec<Case>,
    pub projectile_host: bool,
    pub projectile_rendering: bool,
    pub case_order: Option<Vec<String>>,
    pub infrastructure_probe: bool,
    pub callback_adapter: Option<String>,
    pub plm_amount_context: Option<String>,
    pub admitted_domains: BTreeMap<String, String>,
    pub source: Option<String>,
    pub license: Option<PathBuf>,
    pub scope: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generation {
    pub manifest: PathBuf,
    pub family: String,
    pub cases: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlmGeneration {
    pub manifest: PathBuf,
    pub rom: PathBuf,
    pub rom_sha256: String,
}
pub use decomp_source::Source;
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Implementation {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixture_execution: Option<String>,
    pub dependency_contract: String,
    pub callback_adapter: Option<String>,
    pub source_root: PathBuf,
    pub rust_dependencies: BTreeMap<String, String>,
    pub harness_sources: BTreeMap<String, String>,
    pub module: PathBuf,
    pub module_sha256: String,
    pub memory_sha256: String,
    pub binaries: BTreeMap<String, String>,
    /// Original manifest freshness; absent only in retained historical seals.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manifest: Option<Asset>,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Mismatch {
    pub offset: usize,
    pub c: u8,
    pub rust: u8,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CallbackMetrics {
    #[serde(rename = "return")]
    pub returns: u64,
    pub c_gameplay_callbacks: u64,
    pub cpu_fallback_calls: u64,
    pub cpu_fallback_instructions: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpawnMetrics {
    pub flags: i64,
    pub amount: i64,
    pub cpu_fallback_calls: u64,
    pub cpu_fallback_instructions: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct CaseRecord {
    pub routine: String,
    #[serde(rename = "case")]
    pub name: String,
    #[serde(rename = "return")]
    pub returns: BTreeMap<String, u64>,
    pub before_sha256: String,
    pub after_sha256: BTreeMap<String, String>,
    pub first_mismatch: Option<Mismatch>,
    pub capture_after_sha256: Option<String>,
    pub results: BTreeMap<String, CallbackMetrics>,
    pub spawn_results: BTreeMap<String, SpawnMetrics>,
    pub callbacks: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RomIdentity {
    pub sha256: String,
    pub bytes: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Verdict {
    pub status: String,
    pub run: PathBuf,
    pub live_exercised: bool,
    pub original_rom_parity: String,
    pub family: String,
    pub source: Option<Source>,
    pub implementation: Option<Implementation>,
    pub rom: Option<RomIdentity>,
    pub gameplay_translation: Option<bool>,
    pub cases: Vec<CaseRecord>,
    pub verified_cases: usize,
    pub verified_routines: usize,
    pub rejected_abi_inputs: usize,
    pub native_rust_spawn_calls: usize,
    pub remaining_c_gameplay_callbacks: usize,
    pub allocator_cases: BTreeMap<String, usize>,
    pub scope: String,
    pub error: Option<String>,
    pub artifacts: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct VerifyRequest {
    pub execution: String,
    pub checkout: PathBuf,
    pub source: PathBuf,
    pub output: PathBuf,
    pub manifest: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub case_jobs: usize,
    pub adapter: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthenticateRequest {
    pub checkout: PathBuf,
    pub verdict: PathBuf,
}
