//! Dedicated four-routine WRAM/SRAM codec boundary.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum PersistenceEntry {
    Save = 0,
    Load = 1,
    Pack = 2,
    Unpack = 3,
}
impl PersistenceEntry {
    pub const ALL: [Self; 4] = [Self::Save, Self::Load, Self::Pack, Self::Unpack];
    pub fn address(self) -> u32 {
        [0x818000, 0x818085, 0x81834b, 0x8182e4][self as usize]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub enum PersistenceOutput {
    Done,
    Loaded(u8),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceOutcome {
    Handled(PersistenceOutput),
    Refused,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum PersistencePhase {
    Begin = 0,
    End = 1,
    DiskBegin = 2,
    DiskEnd = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct PersistenceObservation {
    pub phase: PersistencePhase,
    pub entry: PersistenceEntry,
    pub argument: u16,
    /// Present only at End; refusal is an ownership event, never Loaded(1).
    pub output: Option<PersistenceOutput>,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct PersistenceCounters {
    pub entries: [u64; 4],
    pub c_entries: [u64; 4],
    pub rust_entries: [u64; 4],
    pub rust_refused: [u64; 4],
    pub rust_completed: [u64; 4],
    pub write_requests: u64,
    pub write_completions: u64,
}
