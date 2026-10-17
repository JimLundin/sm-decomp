//! Scheduled vertical roots and compact branch/transport observations.
#[cfg(feature = "legacy-evidence")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum PlayerMotionEntry {
    Up = 0,
    Down = 1,
}
impl PlayerMotionEntry {
    pub const ALL: [Self; 2] = [Self::Up, Self::Down];
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum PlayerMotionPhase {
    RootBegin = 0,
    RootEnd = 1,
    TerrainBegin = 2,
    TerrainEnd = 3,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct PlayerMotionObservation {
    pub phase: PlayerMotionPhase,
    pub entry: PlayerMotionEntry,
    pub amount: i32,
    /// Signed terrain return or root displacement; zero at Begin phases.
    pub result: i32,
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "legacy-evidence", derive(Serialize, Deserialize))]
pub struct PlayerMotionCounters {
    pub entries: [u64; 2],
    pub c_entries: [u64; 2],
    pub rust_entries: [u64; 2],
    pub rust_refused: [u64; 2],
    pub rust_completed: [u64; 2],
    /// Incoming roots outside the terrain subset, including enemy-hit roots
    /// that can still finish in Rust without requesting terrain.
    pub terrain_unavailable: [u64; 2],
    pub terrain_requests: u64,
    pub terrain_completions: u64,
    pub terrain_invalid_requests: u64,
    /// These four arrays count Rust-handled roots only, using the completed
    /// enemy/terrain branch. They never credit C fallback as Rust helper reach.
    pub enemy_hit_returns: [u64; 2],
    pub terrain_returns: [u64; 2],
    pub feedback_returns: [u64; 2],
    pub no_collision_move_returns: [u64; 2],
}
