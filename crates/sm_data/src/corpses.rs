//! The dead monsters' tables in bank $A9: the sidehopper corpse's hops and the
//! zoomer, ripper and skree corpses' instruction lists and rotting definitions.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $A9:D951 Y and $A9:D959 X initial velocities of the sidehopper corpse's hops.
pub const HOP_ADDRESS: u32 = 0xa9_d951;
pub const HOP_X_VELOCITY_ADDRESS: u32 = 0xa9_d959;
pub const HOP_ROWS: usize = 4;
pub const HOP_BYTES: usize = HOP_ROWS * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hop {
    pub y_velocity: u16,
    pub x_velocity: u16,
}

pub type Hops = Rows<Hop, HOP_ROWS>;

pub fn import_hops(bytes: &[u8]) -> Result<Hops, LengthError> {
    Rows::import_columns(bytes, |[y_velocity, x_velocity]| Hop {
        y_velocity,
        x_velocity,
    })
}

/// One corpse variant, chosen by enemy parameter 1 / 2.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorpseVariant {
    /// Bank $A9 pointer to the enemy's instruction list.
    pub instruction_list: u16,
    /// Bank $A9 pointer to the corpse rotting definition.
    pub rotting: u16,
}

/// $A9:D86A instruction lists then $A9:D870 rotting definitions (zoomer corpse).
pub const ZOOMER_ADDRESS: u32 = 0xa9_d86a;
pub const ZOOMER_ROTTING_ADDRESS: u32 = 0xa9_d870;
pub const ZOOMER_ROWS: usize = 3;
/// $A9:D897 instruction lists then $A9:D89B rotting definitions (ripper corpse).
pub const RIPPER_ADDRESS: u32 = 0xa9_d897;
pub const RIPPER_ROTTING_ADDRESS: u32 = 0xa9_d89b;
pub const RIPPER_ROWS: usize = 2;
/// $A9:D8C0 instruction lists then $A9:D8C6 rotting definitions (skree corpse).
pub const SKREE_ADDRESS: u32 = 0xa9_d8c0;
pub const SKREE_ROTTING_ADDRESS: u32 = 0xa9_d8c6;
pub const SKREE_ROWS: usize = 3;

pub type Variants<const ROWS: usize> = Rows<CorpseVariant, ROWS>;

pub fn import_variants<const ROWS: usize>(bytes: &[u8]) -> Result<Variants<ROWS>, LengthError> {
    Rows::import_columns(bytes, |[instruction_list, rotting]| CorpseVariant {
        instruction_list,
        rotting,
    })
}
