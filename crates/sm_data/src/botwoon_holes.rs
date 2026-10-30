//! Botwoon hole hitboxes at $B3:949B: the four wall holes it moves between.

/// LoROM address of the first hole.
pub const ADDRESS: u32 = 0xb3_949b;
pub const HOLES: usize = 4;
pub const BYTES: usize = 8 * HOLES;

/// A hole's boundaries in room pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Hole {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
}

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BotwoonHoles {
    holes: [Hole; HOLES],
}

impl BotwoonHoles {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        Ok(Self {
            holes: core::array::from_fn(|hole| Hole {
                left: word(8 * hole),
                right: word(8 * hole + 2),
                top: word(8 * hole + 4),
                bottom: word(8 * hole + 6),
            }),
        })
    }

    pub fn hole(&self, index: usize) -> Option<Hole> {
        self.holes.get(index).copied()
    }
}
