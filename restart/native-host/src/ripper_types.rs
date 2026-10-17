//! Scheduled initialization/patrol roots for both Ripper variants.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum RipperEntry {
    JetInit = 0,
    JetMain = 1,
    RipperInit = 2,
    RipperMain = 3,
}
impl RipperEntry {
    pub const ALL: [Self; 4] = [Self::JetInit, Self::JetMain, Self::RipperInit, Self::RipperMain];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum RipperPhase {
    RootBegin = 0,
    RootEnd = 1,
    ServiceBegin = 2,
    ServiceEnd = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct RipperObservation {
    pub phase: RipperPhase,
    pub entry: RipperEntry,
    pub slot: u16,
    pub amount: i32,
    pub result: u8,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct RipperCounters {
    pub entries: [u64; 4],
    pub c_entries: [u64; 4],
    pub rust_entries: [u64; 4],
    pub rust_refused: [u64; 4],
    pub rust_completed: [u64; 4],
    pub terrain_unavailable: [u64; 4],
    pub service_requests: u64,
    pub service_completions: u64,
    pub invalid_requests: u64,
}
