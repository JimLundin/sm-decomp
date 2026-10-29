//! The statues' tables in bank $AA: the Tourian entrance statue's instruction
//! lists and the chozo statue's instruction lists and Samus-carrying movement.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $AA:D810: the Tourian entrance statue's instruction lists for the base
/// decoration, Ridley and Phantoon, indexed by enemy parameter 1 / 2.
pub const TOURIAN_STATUE_ADDRESS: u32 = 0xaa_d810;
pub const TOURIAN_STATUE_ROWS: usize = 3;
pub const TOURIAN_STATUE_BYTES: usize = TOURIAN_STATUE_ROWS * 2;

/// $AA:E7A2: the chozo statue's instruction lists, indexed by enemy parameter 2 / 2.
pub const CHOZO_STATUE_ADDRESS: u32 = 0xaa_e7a2;
pub const CHOZO_STATUE_ROWS: usize = 2;
pub const CHOZO_STATUE_BYTES: usize = CHOZO_STATUE_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Statue {
    /// Bank $AA pointer to the enemy's instruction list.
    pub instruction_list: u16,
}

pub type TourianStatues = Rows<Statue, TOURIAN_STATUE_ROWS>;
pub type ChozoStatues = Rows<Statue, CHOZO_STATUE_ROWS>;

pub fn import_tourian_statues(bytes: &[u8]) -> Result<TourianStatues, LengthError> {
    Rows::import(bytes, |[instruction_list]| Statue { instruction_list })
}

pub fn import_chozo_statues(bytes: &[u8]) -> Result<ChozoStatues, LengthError> {
    Rows::import(bytes, |[instruction_list]| Statue { instruction_list })
}

/// $AA:E630 X velocities, $AA:E670 Samus X offsets and $AA:E6B0 Samus Y offsets
/// for the chozo statue's movement indices: 0..Fh Wrecked Ship, 10h..1Fh
/// Lower Norfair.
pub const MOVEMENT_ADDRESS: u32 = 0xaa_e630;
pub const SAMUS_X_OFFSET_ADDRESS: u32 = 0xaa_e670;
pub const SAMUS_Y_OFFSET_ADDRESS: u32 = 0xaa_e6b0;
pub const MOVEMENT_ROWS: usize = 32;
pub const MOVEMENT_BYTES: usize = MOVEMENT_ROWS * 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Movement {
    /// 1/100h px/frame; its absolute value is the Y velocity.
    pub x_velocity: u16,
    /// Samus's position relative to the statue.
    pub samus_x_offset: u16,
    pub samus_y_offset: u16,
}

pub type Movements = Rows<Movement, MOVEMENT_ROWS>;

pub fn import_movements(bytes: &[u8]) -> Result<Movements, LengthError> {
    Rows::import_columns(bytes, |[x_velocity, samus_x_offset, samus_y_offset]| {
        Movement {
            x_velocity,
            samus_x_offset,
            samus_y_offset,
        }
    })
}
