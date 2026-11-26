//! Import the single-sprite clip for wall-jump dust.

use core::num::NonZeroU16;
use data::assets::samus::WallJumpDustArt;
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::SpriteAttributes;

const LOOKS: RomAddress = RomAddress::new(0x90, 0x8c17);
const INITIAL_CLOCK: RomAddress = RomAddress::new(0x91, 0xfa9d);
const FOLLOWING_CLOCKS: RomAddress = RomAddress::new(0x90, 0x8bdf);

/// Why the dust clip could not be admitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// An attribute or duration lies outside the image.
    Address(AddressError),
    /// A look has no visible drawing visits.
    EmptyLook,
}

impl From<AddressError> for ImportError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Read the four dust looks and their positive display durations.
///
/// # Errors
/// Returns a fault for incomplete content or a look with no visible drawing visits.
pub fn import(rom: Lorom<'_>) -> Result<WallJumpDustArt, ImportError> {
    let looks = words(rom, LOOKS)?.map(SpriteAttributes::from_word);
    let [initial] = words(rom, INITIAL_CLOCK)?;
    let [second, third, fourth] = words(rom, FOLLOWING_CLOCKS)?;
    let positive = |time| NonZeroU16::new(time).ok_or(ImportError::EmptyLook);
    // Creation spends a tick on the initial clock before the first visible visit.
    let first = positive(initial.saturating_sub(1))?;
    Ok(WallJumpDustArt {
        looks,
        visits: [
            first,
            positive(second)?,
            positive(third)?,
            positive(fourth)?,
        ],
    })
}

fn words<const N: usize>(rom: Lorom<'_>, start: RomAddress) -> Result<[u16; N], AddressError> {
    let bytes = rom.read(start, N.saturating_mul(2))?;
    let words: [[u8; 2]; N] = bytes
        .as_chunks::<2>()
        .0
        .try_into()
        .map_err(|_| AddressError::PastEnd(start))?;
    Ok(words.map(u16::from_le_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dust_looks_keep_their_order_attributes_and_visible_durations() {
        let mut image = vec![0; data::rom::Rom::SIZE];
        let rom = Lorom::new(&image);
        let at = rom.file_offset(LOOKS).unwrap();
        let initial = rom.file_offset(INITIAL_CLOCK).unwrap();
        let following = rom.file_offset(FOLLOWING_CLOCKS).unwrap();
        image.truncate(initial + 2);
        image[at..at + 8].copy_from_slice(&[1, 0xa2, 3, 0x44, 5, 0x89, 7, 0x30]);
        image[initial..].copy_from_slice(&[5, 0]);
        image[following..following + 6].copy_from_slice(&[2, 0, 8, 0, 1, 0]);

        let art = import(Lorom::new(&image)).unwrap();
        assert_eq!(
            art.looks.map(SpriteAttributes::word),
            [0xa201, 0x4403, 0x8905, 0x3007]
        );
        assert_eq!(art.visits.map(NonZeroU16::get), [4, 2, 8, 1]);
        assert_eq!(
            import(Lorom::new(&image[..image.len() - 1])),
            Err(ImportError::Address(AddressError::PastEnd(INITIAL_CLOCK)))
        );
        image[initial] = 1;
        assert_eq!(import(Lorom::new(&image)), Err(ImportError::EmptyLook));
        image[initial] = 5;
        image[following + 2] = 0;
        assert_eq!(import(Lorom::new(&image)), Err(ImportError::EmptyLook));
    }
}
