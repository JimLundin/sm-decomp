//! Typed live admission and four-process observation records.
use decomp_source::Source;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};
use tool_execution::build_cache::CacheResult;
use tool_execution::builds::ExecutionRecord;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Request {
    pub checkout: PathBuf,
    pub run: PathBuf,
    pub output: PathBuf,
    pub root: PathBuf,
    pub workers: usize,
    pub frames: u64,
    pub live_jobs: usize,
    pub accepted: PathBuf,
    pub family_verdicts: Vec<PathBuf>,
    pub dispatcher_verdicts: Vec<PathBuf>,
    pub callback_verdicts: Vec<PathBuf>,
    pub plm_verdict: Option<PathBuf>,
    pub projectile_verdict: Option<PathBuf>,
    pub capture_active_eprojs: bool,
    pub room_plm: bool,
    pub rng_predecessor: Option<PathBuf>,
    pub retain_private_stores: bool,
    pub evidence_store: Option<PathBuf>,
}
impl Default for Request {
    fn default() -> Self {
        Self {
            checkout: PathBuf::new(),
            run: PathBuf::new(),
            output: PathBuf::new(),
            root: PathBuf::new(),
            workers: 4,
            frames: 9000,
            live_jobs: 4,
            accepted: PathBuf::new(),
            family_verdicts: vec![],
            dispatcher_verdicts: vec![],
            callback_verdicts: vec![],
            plm_verdict: None,
            projectile_verdict: None,
            capture_active_eprojs: false,
            room_plm: false,
            rng_predecessor: None,
            retain_private_stores: false,
            evidence_store: None,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Predecessor {
    pub path: PathBuf,
    pub sha256: String,
    pub kind: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transformation {
    pub original_sha256: String,
    pub transformed_sha256: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Generated {
    pub source: Source,
    pub symbols: Vec<String>,
    pub dispatcher_symbols: BTreeSet<String>,
    pub transformations: BTreeMap<String, Transformation>,
    pub host_dependencies: BTreeMap<String, String>,
    pub units: Vec<PathBuf>,
    pub predecessors: Vec<Predecessor>,
    pub room_plm_installed: bool,
    pub projectile_native_installed: bool,
    pub bounded_symbols: BTreeSet<String>,
    pub ridley_native_installed: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub c: u64,
    pub rust: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Ownership {
    pub total: u64,
    pub c: u64,
    pub rust: u64,
    pub rust_setups: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct PriorHit {
    pub decompressor_c: u64,
    pub decompressor_rust: u64,
    pub rng_c: u64,
    pub rng_rust: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectileHit {
    pub total: u64,
    pub c: u64,
    pub rust: u64,
    pub refused: u64,
    pub active_c: u64,
    pub active_rust: u64,
    pub combat_c: u64,
    pub combat_rust: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectileIdHit {
    pub c: u64,
    pub rust: u64,
    pub refused: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectileAllocationHit {
    pub total: u64,
    pub c: u64,
    pub rust: u64,
    pub refused: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FrameHit {
    pub total: u64,
    pub c: u64,
    pub rust: u64,
    pub refused: u64,
    pub active_c: u64,
    pub active_rust: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Progress {
    pub frame: u64,
    pub game_state: u64,
    pub room: u64,
    pub samus_x: u64,
    pub samus_y: u64,
    pub rng: u64,
    pub decompression_calls: u64,
    pub fallback_calls: u64,
    pub fallback_instructions: u64,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Counters {
    pub hits: BTreeMap<String, Hit>,
    pub progress: Vec<Progress>,
    pub predecessor_hits: PriorHit,
    pub plm_callback_hits: Ownership,
    pub room_plm_hits: BTreeMap<String, Ownership>,
    #[serde(default)]
    pub projectile_hits: BTreeMap<String, ProjectileHit>,
    #[serde(default)]
    pub projectile_id_hits: BTreeMap<String, ProjectileIdHit>,
    #[serde(default)]
    pub projectile_allocation_hits: BTreeMap<String, ProjectileAllocationHit>,
    #[serde(default)]
    pub frame_hits: BTreeMap<String, FrameHit>,
    #[serde(default)]
    pub ridley_opcode_hits: BTreeMap<String, Hit>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StorageSubject {
    pub run: String,
    pub observations: String,
    pub pointer_sha256: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Storage {
    pub format: String,
    pub subjects: BTreeMap<String, StorageSubject>,
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Verdict {
    pub status: String,
    pub run: PathBuf,
    pub live_exercised: bool,
    pub original_rom_parity: String,
    pub commands: Vec<ExecutionRecord>,
    pub source: Option<Source>,
    pub predecessors: Vec<Predecessor>,
    pub decompressor_predecessor_sha256: String,
    pub transformations: BTreeMap<String, Transformation>,
    pub host_dependencies: BTreeMap<String, String>,
    pub dispatcher_symbols: BTreeSet<String>,
    #[serde(default)]
    pub routine_symbols: Vec<String>,
    pub room_plm_installed: bool,
    pub projectile_native_installed: bool,
    #[serde(default)]
    pub bounded_symbols: BTreeSet<String>,
    #[serde(default)]
    pub ridley_native_installed: bool,
    pub projectile_abi: Option<String>,
    pub active_eproj_captures: bool,
    pub rom_sha256: String,
    pub inputs_sha256: String,
    pub frames: u64,
    pub hardware_storage: String,
    pub evidence_cache: Option<CacheResult>,
    pub evidence_reader_sha256: String,
    pub rust_dependencies: BTreeMap<String, String>,
    pub decompressor_cache: Option<CacheResult>,
    pub rng_source_sha256: String,
    pub rng_predecessor_sha256: String,
    pub host_cache: Option<ExecutionRecord>,
    pub binary_sha256: String,
    pub capture_seconds: f64,
    pub state_effects_match: bool,
    pub artifacts: BTreeMap<String, String>,
    pub counters: BTreeMap<String, Counters>,
    pub evidence_storage: Storage,
    pub storage_metrics: BTreeMap<String, evidence::Pointer>,
    #[serde(rename = "remaining_CPU_fallback")]
    pub remaining_cpu_fallback: u64,
    pub scope: String,
    pub error: Option<String>,
}
