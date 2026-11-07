//! The Blue Brinstar face block's instruction lists, by what its AI starts them for, and its
//! glow colours.

use crate::rom::{AddressError, Lorom, RomAddress};

use super::program::{EnemyPrograms, ProgramError, Target};

/// The face block's AI bank.
const BANK: u8 = 0xa8;

/// The face block's lists as places in the loaded programs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FaceBlockLists {
    /// Turning to look left, then sleeping.
    pub turning_left: Target,
    /// Turning to look right, then sleeping.
    pub turning_right: Target,
    /// Facing forward, sleeping, which its init starts.
    pub facing_forward: Target,
}

impl FaceBlockLists {
    /// The face block's lists in `programs`.
    ///
    /// # Errors
    ///
    /// [`ProgramError::Unloaded`] for a list `programs` doesn't hold.
    pub fn resolve(programs: &EnemyPrograms) -> Result<Self, ProgramError> {
        let place = |offset: u16| {
            let at = RomAddress::new(BANK, offset);
            programs.target(at).ok_or(ProgramError::Unloaded(at))
        };
        Ok(Self {
            turning_left: place(0xe80c)?,
            turning_right: place(0xe81a)?,
            facing_forward: place(0xe828)?,
        })
    }
}

/// The face block's glow colours: eight sets of four, which its palette cycle
/// copies to colours 9 to 12 of its sprite palette in turn.
const GLOW_COLOURS: RomAddress = RomAddress::new(BANK, 0xe7cc);

/// Sets in the glow's cycle.
pub const GLOW_SETS: usize = 8;

/// Colours in a set.
pub const GLOW_SET_COLOURS: usize = 4;

/// The table's bytes: a word per colour.
const GLOW_BYTES: usize = GLOW_SETS * GLOW_SET_COLOURS * 2;

/// The face block's glow: each set's four colours, in the order its palette cycle shows them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GlowColours(pub [[u16; GLOW_SET_COLOURS]; GLOW_SETS]);

impl GlowColours {
    /// The glow's colours from the ROM.
    ///
    /// # Errors
    ///
    /// [`AddressError`] when the table is outside the image.
    pub fn load(rom: Lorom<'_>) -> Result<Self, AddressError> {
        let bytes = rom.read(GLOW_COLOURS, GLOW_BYTES)?;
        let mut sets = [[0; GLOW_SET_COLOURS]; GLOW_SETS];
        let words = bytes.as_chunks::<2>().0.iter();
        for (colour, &[low, high]) in sets.iter_mut().flatten().zip(words) {
            *colour = u16::from_le_bytes([low, high]);
        }
        Ok(Self(sets))
    }
}

#[cfg(test)]
mod tests {
    use super::GlowColours;
    use crate::rom::Rom;

    /// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
    fn rom() -> Option<Rom> {
        let path = std::env::var_os("SM_ROM").map_or_else(
            || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
            std::path::PathBuf::from,
        );
        let image = std::fs::read(path).ok()?;
        Rom::new(image).ok()
    }

    #[test]
    fn the_glow_rises_and_falls_as_the_bank_log_says() {
        let Some(cartridge) = rom() else {
            return;
        };
        let GlowColours(sets) = GlowColours::load(cartridge.lorom()).unwrap();
        // $A8:E7CC: the first set is the dimmest, the middle two the brightest, and the
        // second half mirrors the first.
        assert_eq!(sets[0], [0x001f, 0x0012, 0x000a, 0x002b]);
        assert_eq!(sets[3], [0x0f3f, 0x01bf, 0x001f, 0x0012]);
        for (rising, falling) in sets.iter().zip(sets.iter().rev()) {
            assert_eq!(rising, falling);
        }
    }

    #[test]
    fn a_short_image_has_no_glow() {
        assert!(GlowColours::load(crate::rom::Lorom::new(&[0; 0x100])).is_err());
    }
}

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// glow colours (bank $A8 log, <https://patrickjohnston.org/bank/A8>, `$A8:E7CC` to
// `$A8:E828`).
// Turning to look left, then sleeping (`$A8:E80C`).
// Turning to look right, then sleeping (`$A8:E81A`).
// Facing forward, sleeping (`$A8:E828`), which its init starts.
// The face block's glow colours (`$A8:E7CC`): eight sets of four, which its palette cycle
// copies to colours 9 to 12 of its sprite palette in turn (`$A8:E86E`).
