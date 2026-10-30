//! The cinematic shooting stars at $8B:E9CF: one four-word record per star.

/// LoROM address of the first record.
pub const ADDRESS: u32 = 0x8b_e9cf;
/// Stars the cinematic initialises.
pub const STARS: usize = 40;
pub const BYTES: usize = STARS * 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShootingStar {
    /// Added to the X velocity each frame, twice once the star is fast.
    pub x_acceleration: u16,
    /// Added to the Y velocity each frame, twice once the star is fast.
    pub y_acceleration: u16,
    /// Frames between animation steps.
    pub frame_delay: u16,
    /// Frames before the star appears; zero starts it at once.
    pub start_delay: u16,
}

pub use crate::table::LengthError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShootingStars {
    stars: Box<[ShootingStar; STARS]>,
}

impl ShootingStars {
    pub fn import(bytes: &[u8]) -> Result<Self, LengthError> {
        LengthError::check(bytes, BYTES)?;
        let word = |at: usize| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
        Ok(Self {
            stars: Box::new(core::array::from_fn(|star| ShootingStar {
                x_acceleration: word(8 * star),
                y_acceleration: word(8 * star + 2),
                frame_delay: word(8 * star + 4),
                start_delay: word(8 * star + 6),
            })),
        })
    }

    pub fn star(&self, index: usize) -> Option<ShootingStar> {
        self.stars.get(index).copied()
    }
}

/// $8B:E9A7: a star's OAM tile and attributes for each animation step, by
/// bits 9-15 of its identity.
pub const TILE_ADDRESS: u32 = 0x8b_e9a7;
pub const TILE_STEPS: usize = 20;
pub type Tiles = crate::table::Rows<u16, TILE_STEPS>;

pub fn import_tiles(bytes: &[u8]) -> Result<Tiles, crate::table::LengthError> {
    crate::table::Rows::import(bytes, |[tile]| tile)
}
