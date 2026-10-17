//! Fixed synchronous directional transition boundary and passive evidence.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum DoorScrollingEntry {
    FixDoorsMovingUp = 0,
    ScrollingSetup = 1,
    FollowTransition = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum DoorScrollingRoutine {
    FixDoorsMovingUp = 0,
    ScrollingSetup = 1,
    FollowTransition = 2,
    SetupRight = 3,
    SetupLeft = 4,
    SetupDown = 5,
    SetupUp = 6,
    UpdatePreviousBlocks = 7,
    UpdateBgOffsets = 8,
    MoveRight = 9,
    MoveLeft = 10,
    MoveDown = 11,
    MoveUp = 12,
}
impl TryFrom<u32> for DoorScrollingRoutine {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, String> {
        use DoorScrollingRoutine::*;
        [
            FixDoorsMovingUp,
            ScrollingSetup,
            FollowTransition,
            SetupRight,
            SetupLeft,
            SetupDown,
            SetupUp,
            UpdatePreviousBlocks,
            UpdateBgOffsets,
            MoveRight,
            MoveLeft,
            MoveDown,
            MoveUp,
        ]
        .get(value as usize)
        .copied()
        .ok_or_else(|| "invalid door-scrolling routine ID".into())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorScrollingObservation {
    Enter { routine: DoorScrollingRoutine, counter: u16 },
    Return { routine: DoorScrollingRoutine, result: u8, counter: u16 },
}

/// Child counts are Rust attempts/completions, indexed by the named service.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct DoorScrollingCounters {
    pub c_entries: [u64; 3],
    pub rust_entries: [u64; 3],
    pub rust_completed: [u64; 3],
    pub rust_refused: [u64; 3],
    pub routine_c_entries: [u64; 13],
    pub routine_c_completed: [u64; 13],
    pub routine_rust_entries: [u64; 13],
    pub routine_rust_completed: [u64; 13],
    pub service_requests: [u64; 4],
    pub service_completions: [u64; 4],
    pub foreign_requests: [u64; 4],
    pub foreign_completions: [u64; 4],
    pub child_entries: [u64; 4],
    pub child_completions: [u64; 4],
    pub coordinator_entries: [u64; 3],
    pub standalone_entries: [u64; 3],
}
