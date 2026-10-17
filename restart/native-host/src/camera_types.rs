//! Pure normal-camera root and its fixed passive routine catalog.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum CameraRoutine {
    MainScrolling = 0,
    CallFinishedHook = 1,
    SporeSpawnFinish = 2,
    HandleX = 3,
    HandleY = 4,
    DistanceX = 5,
    DistanceY = 6,
    AutoX = 7,
    TriggerRight = 8,
    TriggerLeft = 9,
    AutoY = 10,
    TriggerDown = 11,
    TriggerUp = 12,
}
impl TryFrom<u32> for CameraRoutine {
    type Error = String;
    fn try_from(value: u32) -> Result<Self, String> {
        use CameraRoutine::*;
        [
            MainScrolling,
            CallFinishedHook,
            SporeSpawnFinish,
            HandleX,
            HandleY,
            DistanceX,
            DistanceY,
            AutoX,
            TriggerRight,
            TriggerLeft,
            AutoY,
            TriggerDown,
            TriggerUp,
        ]
        .get(value as usize)
        .copied()
        .ok_or_else(|| "invalid camera routine ID".into())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CameraObservation {
    Enter { routine: CameraRoutine, operand: u16 },
    Return { routine: CameraRoutine },
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct CameraCounters {
    pub c_entries: u64,
    pub rust_entries: u64,
    pub rust_completed: u64,
    pub rust_refused: u64,
    pub routine_c_entries: [u64; 13],
    pub routine_c_completed: [u64; 13],
    pub routine_rust_entries: [u64; 13],
    pub routine_rust_completed: [u64; 13],
}
