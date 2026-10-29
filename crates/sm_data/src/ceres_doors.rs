//! The Ceres steam and door tables in bank $A6.
use crate::table::{LengthError, Rows};

/// $A6:EFF5: steam instruction lists then initial functions ($F001), per
/// enemy parameter 1.
pub const STEAM_ADDRESS: u32 = 0xa6_eff5;
pub const STEAM_KINDS: usize = 6;
pub const STEAM_BYTES: usize = STEAM_KINDS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SteamKind {
    pub instruction_list: u16,
    pub function: u16,
}

pub type Steam = Rows<SteamKind, STEAM_KINDS>;

pub fn import_steam(bytes: &[u8]) -> Result<Steam, LengthError> {
    Rows::import_columns(bytes, |[instruction_list, function]| SteamKind {
        instruction_list,
        function,
    })
}

/// Ceres door kinds, indexed by enemy parameter 1.
pub const DOOR_KINDS: usize = 7;
/// $A6:F52C: each door kind's instruction list.
pub const DOOR_INSTRUCTION_ADDRESS: u32 = 0xa6_f52c;
/// $A6:F72B: each door kind's function.
pub const DOOR_FUNCTION_ADDRESS: u32 = 0xa6_f72b;

/// $A6:F840: the rotating elevator room's explosion spawn offsets, X then Y.
pub const EXPLOSION_OFFSET_ADDRESS: u32 = 0xa6_f840;
pub const EXPLOSION_OFFSETS: usize = 4;

/// $A6:F900: the elevator platform's two animation frames of tile transfers.
pub const PLATFORM_FRAME_ADDRESS: u32 = 0xa6_f900;
pub const PLATFORM_FRAMES: usize = 2;
