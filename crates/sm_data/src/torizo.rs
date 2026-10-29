//! The Torizos' instruction lists in bank $AA.
pub use crate::table::LengthError;
use crate::table::Rows;

/// $AA:C967: the initial instruction lists of Bomb Torizo (Crateria) and
/// Golden Torizo (Norfair), indexed by area index / 2.
pub const INSTRUCTION_LIST_ADDRESS: u32 = 0xaa_c967;
pub const TORIZO_ROWS: usize = 2;
pub const INSTRUCTION_LIST_BYTES: usize = TORIZO_ROWS * 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Torizo {
    /// Bank $AA pointer to the Torizo's initial instruction list.
    pub instruction_list: u16,
}

pub type Torizos = Rows<Torizo, TORIZO_ROWS>;

pub fn import_torizos(bytes: &[u8]) -> Result<Torizos, LengthError> {
    Rows::import(bytes, |[instruction_list]| Torizo { instruction_list })
}
