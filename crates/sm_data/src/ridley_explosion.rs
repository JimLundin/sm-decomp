//! Ridley's explosion tables in bank $A6: lifetimes and, for the parts placed
//! by facing, X offsets and instruction lists.
use crate::table::{LengthError, Rows};

/// $A6:C6CE: the explosion's lifetime per initialisation index (enemy parameter 1).
pub const LIFETIME_ADDRESS: u32 = 0xa6_c6ce;
pub const LIFETIMES: usize = 12;

/// The parts placed by facing, each two X offsets then two instruction lists.
pub const WINGS_ADDRESS: u32 = 0xa6_c804;
pub const LEGS_ADDRESS: u32 = 0xa6_c836;
pub const MOUTH_ADDRESS: u32 = 0xa6_c868;
pub const TORSO_ADDRESS: u32 = 0xa6_c89a;
pub const HAND_ADDRESS: u32 = 0xa6_c8cc;
pub const PART_ADDRESSES: [u32; 5] = [
    WINGS_ADDRESS,
    LEGS_ADDRESS,
    MOUTH_ADDRESS,
    TORSO_ADDRESS,
    HAND_ADDRESS,
];
/// Facing left, then any other facing.
pub const PART_SIDES: usize = 2;
pub const PART_BYTES: usize = PART_SIDES * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PartPlacement {
    /// Added to Ridley's X position.
    pub x_offset: u16,
    pub instruction_list: u16,
}

pub type Part = Rows<PartPlacement, PART_SIDES>;

pub fn import_part(bytes: &[u8]) -> Result<Part, LengthError> {
    Rows::import_columns(bytes, |[x_offset, instruction_list]| PartPlacement {
        x_offset,
        instruction_list,
    })
}
