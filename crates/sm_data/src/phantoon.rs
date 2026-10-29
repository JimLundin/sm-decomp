//! Phantoon's flame-rain positions at $A7:CDAD.
pub use crate::table::LengthError;
use crate::table::Rows;

pub const FLAME_RAIN_ADDRESS: u32 = 0xa7_cdad;
pub const FLAME_RAIN_ROWS: usize = 8;
/// Four words per row; the fourth is always zero.
pub const FLAME_RAIN_BYTES: usize = FLAME_RAIN_ROWS * 8;

/// Where Phantoon materialises to rain flames.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FlameRainPosition {
    pub figure_eight_index: u16,
    pub x_position: u16,
    pub y_position: u16,
}

pub type FlameRainPositions = Rows<FlameRainPosition, FLAME_RAIN_ROWS>;

pub fn import_flame_rain(bytes: &[u8]) -> Result<FlameRainPositions, LengthError> {
    Rows::import(bytes, |[figure_eight_index, x_position, y_position, _]| {
        FlameRainPosition {
            figure_eight_index,
            x_position,
            y_position,
        }
    })
}
