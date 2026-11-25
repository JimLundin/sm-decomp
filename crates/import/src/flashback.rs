//! Import the Zebes flashback encounter's named sprite clips and normal boss colours.

use data::assets::flashback::{Colour, ROW_COLOURS, RowColours, Scene};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

const ART_BANK: u8 = 0x8c;
const BRAIN: [u16; 3] = [0x8c00, 0x8c2f, 0x8c5e];
const RING: [u16; 3] = [0x8c8d, 0x8ca3, 0x8cb9];
const SMALL: [u16; 6] = [0x97f7, 0x97fe, 0x9805, 0x981b, 0x9831, 0x9847];
const LARGE: [u16; 6] = [0x985d, 0x9864, 0x987a, 0x9890, 0x98a6, 0x98bc];
const NORMAL_BOSS: u16 = 0xe5c9;

/// The encounter content could not be imported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// The image does not contain the whole sprite map or palette.
    Address(AddressError),
    /// The map is empty or too large.
    Map,
    /// A palette word sets the unused high bit.
    Colour,
}

impl From<AddressError> for ImportError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Read the encounter's art as named clips.
///
/// # Errors
/// Invalid or incomplete maps and colours are refused.
pub fn load(rom: Lorom<'_>) -> Result<Scene, ImportError> {
    let bytes = rom.read(RomAddress::new(ART_BANK, NORMAL_BOSS), ROW_COLOURS * 2)?;
    let mut colours = [Colour::BLACK; ROW_COLOURS];
    for (colour, &word) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *colour = Colour::new(u16::from_le_bytes(word)).ok_or(ImportError::Colour)?;
    }
    Ok(Scene {
        brain: clip(rom, BRAIN)?,
        ring: clip(rom, RING)?,
        small_burst: clip(rom, SMALL)?,
        large_burst: clip(rom, LARGE)?,
        normal_boss: RowColours(colours),
    })
}

fn clip<const N: usize>(rom: Lorom<'_>, pictures: [u16; N]) -> Result<[Spritemap; N], ImportError> {
    let mut maps = std::array::from_fn(|_| Spritemap::default());
    for (map, at) in maps.iter_mut().zip(pictures) {
        *map = Spritemap::load(rom, RomAddress::new(ART_BANK, at))?;
        if !(1..=128).contains(&map.entries.len()) {
            return Err(ImportError::Map);
        }
    }
    Ok(maps)
}
