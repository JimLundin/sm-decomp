//! Complete Waver actor entries, preserving its instruction continuation.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum WaverEntry {
    Init = 0,
    Main = 1,
    Instruction = 2,
    SelectAnimation = 3,
}
impl WaverEntry {
    pub const ALL: [Self; 4] = [Self::Init, Self::Main, Self::Instruction, Self::SelectAnimation];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub enum WaverOutput {
    Done,
    Continue(u16),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum WaverPhase {
    RootBegin = 0,
    RootEnd = 1,
    ServiceBegin = 2,
    ServiceEnd = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum WaverService {
    MoveHorizontal = 0,
    Sine = 1,
    MoveVertical = 2,
}
impl WaverService {
    pub const ALL: [Self; 3] = [Self::MoveHorizontal, Self::Sine, Self::MoveVertical];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct WaverObservation {
    pub phase: WaverPhase,
    pub entry: WaverEntry,
    pub k: u16,
    pub jp: u16,
    pub service: Option<WaverService>,
    pub argument: i32,
    pub auxiliary: u16,
    pub result: u16,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct WaverCounters {
    pub entries: [u64; 4],
    pub c_entries: [u64; 4],
    pub rust_entries: [u64; 4],
    pub rust_refused: [u64; 4],
    pub rust_completed: [u64; 4],
    pub terrain_unavailable: [u64; 4],
    pub service_requests: [u64; 3],
    pub service_completions: [u64; 3],
    pub invalid_requests: u64,
}
