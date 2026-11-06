//! Samus's horizontal speed tables: how fast each movement type accelerates, its top speed
//! and how fast it slows, and the extra run speed holding the run button adds.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): `$90:9F55` "Samus X speed table -
//! normal", one twelve-byte entry a movement type (X acceleration, max X speed, X
//! deceleration, each a pixel word then a subpixel word), read by `$90:9BD1` "Determine
//! Samus X speed table entry pointer" at the table pointer plus 12 times the movement type;
//! `$90:9F01`, the extra run speed's acceleration and caps, read by `$90:973E` "Handle Samus X
//! extra run speed".

use canonical::Canonical;

use crate::pose::MovementType;
use crate::rom::{AddressError, Lorom, RomAddress};

/// A horizontal speed or acceleration: pixels a frame with a 1/65536 fraction, as the
/// original keeps it in a pixel word and a subpixel word.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Speed(pub u32);

impl Speed {
    /// Still.
    pub const ZERO: Self = Self(0);

    /// The speed of `pixels` and `subpixels` a frame.
    #[must_use]
    pub const fn new(pixels: u16, subpixels: u16) -> Self {
        let [pixels_high, pixels_low] = pixels.to_be_bytes();
        let [subpixels_high, subpixels_low] = subpixels.to_be_bytes();
        Self(u32::from_be_bytes([
            pixels_high,
            pixels_low,
            subpixels_high,
            subpixels_low,
        ]))
    }

    /// Whole pixels a frame: the original's speed word.
    #[must_use]
    pub const fn pixels(self) -> u16 {
        let [high, low, _, _] = self.0.to_be_bytes();
        u16::from_be_bytes([high, low])
    }

    /// The fraction, in 1/65536ths of a pixel: the original's subspeed word.
    #[must_use]
    pub const fn subpixels(self) -> u16 {
        let [_, _, high, low] = self.0.to_be_bytes();
        u16::from_be_bytes([high, low])
    }
}

/// One movement type's horizontal speeds (`$90:9F55`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpeedEntry {
    /// Added to the base speed each frame while accelerating.
    pub acceleration: Speed,
    /// The base speed's cap.
    pub max: Speed,
    /// Taken from the base speed each frame while decelerating or turning.
    pub deceleration: Speed,
}

/// The speed table's first entry, for neither water nor lava (bank $90 log).
pub const NORMAL_SPEEDS: RomAddress = RomAddress::new(0x90, 0x9F55);

/// The extra run speed's constants (bank $90 log, `$90:9F01`).
pub const EXTRA_RUN_SPEEDS: RomAddress = RomAddress::new(0x90, 0x9F01);

/// The movement types the handler indexes the table with: 0 to $1B.
///
/// The normal table holds 26 entries (`$90:9F55` to `$90:A08D`); types $1A and $1B read on
/// into the water table's first two, as the original does.
pub const MOVEMENT_TYPES: usize = 0x1C;

/// Bytes an entry.
const ENTRY_BYTES: usize = 12;

/// The extra run constants' bytes: six words of increments (pixels then subpixels, air,
/// water, lava/acid), then six of speed booster caps and six of ordinary caps, the same way.
pub const EXTRA_RUN_BYTES: usize = 36;

/// Every movement type's horizontal speeds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpeedTable {
    entries: Vec<SpeedEntry>,
}

impl SpeedTable {
    /// The table from its bytes, twelve a movement type; a trailing partial entry is
    /// ignored.
    #[must_use]
    pub fn decode(bytes: &[u8]) -> Self {
        let (entries, _) = bytes.as_chunks::<ENTRY_BYTES>();
        let entries = entries
            .iter()
            .map(|entry| {
                let speed =
                    |at: usize| Speed::new(word(entry, at), word(entry, at.wrapping_add(2)));
                SpeedEntry {
                    acceleration: speed(0),
                    max: speed(4),
                    deceleration: speed(8),
                }
            })
            .collect();
        Self { entries }
    }

    /// The speeds of `movement`; a table too short for it gives none.
    #[must_use]
    pub fn of(&self, movement: MovementType) -> Option<&SpeedEntry> {
        self.entries.get(usize::from(u8::from(movement)))
    }
}

/// The extra run speed holding the run button adds while running in air (`$90:9F01`; the
/// handler reads the water and lava columns only after its own liquid test sends it away).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ExtraRun {
    /// Added each frame until the cap.
    pub acceleration: Speed,
    /// The cap with the speed booster equipped.
    pub booster_max: Speed,
    /// The cap without it.
    pub max: Speed,
}

impl ExtraRun {
    /// The constants from their bytes (the air column).
    #[must_use]
    pub fn decode(bytes: &[u8; EXTRA_RUN_BYTES]) -> Self {
        let speed = |at: usize| Speed::new(word(bytes, at), word(bytes, at.wrapping_add(6)));
        Self {
            acceleration: speed(0),
            booster_max: speed(12),
            max: speed(24),
        }
    }
}

/// Samus's horizontal speeds: the normal speed table and the extra run constants.
// open: the water (`$90:A08D`) and lava/acid (`$90:A1DD`) tables join with liquids; no
// slice 0 room has one.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HorizontalSpeeds {
    /// Out of water and lava.
    pub normal: SpeedTable,
    /// The extra run speed.
    pub extra_run: ExtraRun,
}

impl HorizontalSpeeds {
    /// The tables in `rom`.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when a table is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let normal = rom.read(NORMAL_SPEEDS, MOVEMENT_TYPES.wrapping_mul(ENTRY_BYTES))?;
        let extra = rom
            .read(EXTRA_RUN_SPEEDS, EXTRA_RUN_BYTES)?
            .first_chunk()
            .ok_or(AddressError::PastEnd(EXTRA_RUN_SPEEDS))?;
        Ok(Self {
            normal: SpeedTable::decode(normal),
            extra_run: ExtraRun::decode(extra),
        })
    }
}

/// The little-endian word at `at`, zero past the end.
fn word(bytes: &[u8], at: usize) -> u16 {
    match bytes.get(at..at.wrapping_add(2)) {
        Some(&[low, high]) => u16::from_le_bytes([low, high]),
        _ => 0,
    }
}

#[cfg(test)]
mod tests;
