//! Phantoon's random timers at $A7:CD41 and flame-rain positions at $A7:CDAD.
pub use crate::table::LengthError;
use crate::table::Rows;

/// Three tables of eight frame counts, each picked by a random index. The
/// first is followed by a zero word that no index reaches.
pub const TIMERS_ADDRESS: u32 = 0xa7_cd41;
pub const TIMERS_BYTES: usize = 0x32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Timer {
    /// $CD41: how long the figure-8 leaves Phantoon vulnerable.
    FigureEightVulnerable,
    /// $CD53: how long the eye stays closed.
    EyeClosed,
    /// $CD63: how long Phantoon hides during flame rain.
    FlameRainHiding,
}

impl Timer {
    /// Byte offset of this table from `TIMERS_ADDRESS`.
    pub const fn offset(self) -> usize {
        match self {
            Self::FigureEightVulnerable => 0,
            Self::EyeClosed => 0x12,
            Self::FlameRainHiding => 0x22,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Timers([[u16; 8]; 3]);

impl Timers {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        if bytes.len() != TIMERS_BYTES {
            return Err(LengthError(bytes.len()));
        }
        let table = |timer: Timer| {
            core::array::from_fn(|index| {
                let at = timer.offset() + 2 * index;
                u16::from_le_bytes([bytes[at], bytes[at + 1]])
            })
        };
        Ok(Self([
            table(Timer::FigureEightVulnerable),
            table(Timer::EyeClosed),
            table(Timer::FlameRainHiding),
        ]))
    }

    /// Entry `index` of `timer`, or `None` past its eight entries.
    pub fn frames(&self, timer: Timer, index: usize) -> Option<u16> {
        self.0[timer as usize].get(index).copied()
    }
}

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
