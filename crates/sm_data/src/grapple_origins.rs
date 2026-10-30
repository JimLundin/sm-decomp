//! Where the grapple beam leaves Samus, per aim direction, at $9B:C122.
//!
//! Two sets follow each other, the second used while running. Each set is
//! four ten-word columns: origin X, origin Y, then the previous frame's X
//! and Y, all offsets from Samus's position.

/// LoROM address of the standard set's origin-X column.
pub const ADDRESS: u32 = 0x9b_c122;
pub const DIRECTIONS: usize = 10;
const COLUMN: usize = DIRECTIONS * 2;
const SET: usize = 4 * COLUMN;
pub const BYTES: usize = 2 * SET;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OriginSet {
    Standard,
    Running,
}

impl OriginSet {
    /// LoROM address of `column` (0 to 3) in this set.
    pub const fn column_address(self, column: usize) -> u32 {
        let set = match self {
            Self::Standard => 0,
            Self::Running => SET,
        };
        ADDRESS + (set + column * COLUMN) as u32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BeamOrigin {
    pub x: u16,
    pub y: u16,
    pub previous_x: u16,
    pub previous_y: u16,
}

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GrappleOrigins {
    sets: [[BeamOrigin; DIRECTIONS]; 2],
}

impl GrappleOrigins {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        Ok(Self {
            sets: core::array::from_fn(|set| {
                core::array::from_fn(|direction| {
                    let column = |column: usize| word(set * SET + column * COLUMN + 2 * direction);
                    BeamOrigin {
                        x: column(0),
                        y: column(1),
                        previous_x: column(2),
                        previous_y: column(3),
                    }
                })
            }),
        })
    }

    /// The origin for `direction`, or `None` past the ten directions.
    pub fn origin(&self, set: OriginSet, direction: usize) -> Option<BeamOrigin> {
        self.sets[set as usize].get(direction).copied()
    }
}
