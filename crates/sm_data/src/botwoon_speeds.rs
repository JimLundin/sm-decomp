//! Botwoon speed table at $B3:94BB, indexed by its speed level. It drives
//! both Botwoon's movement and its spit projectiles.

/// LoROM address of level 0.
pub const ADDRESS: u32 = 0xb3_94bb;
pub const LEVELS: usize = 3;
pub const BYTES: usize = 4 * LEVELS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BotwoonSpeed {
    pub speed: u16,
    /// Frames a body part takes to reach the next part's position: 30h / speed.
    pub body_travel_time: u16,
}

/// The importer was given a byte count other than `BYTES`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LengthError(pub usize);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BotwoonSpeeds {
    levels: [BotwoonSpeed; LEVELS],
}

impl BotwoonSpeeds {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        if bytes.len() != BYTES {
            return Err(LengthError(bytes.len()));
        }
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        Ok(Self {
            levels: core::array::from_fn(|level| BotwoonSpeed {
                speed: word(4 * level),
                body_travel_time: word(4 * level + 2),
            }),
        })
    }

    pub fn level(&self, level: usize) -> Option<BotwoonSpeed> {
        self.levels.get(level).copied()
    }
}
