//! Read the steam vent figures from the shared mounted and ordinary puff art.

use data::assets::shaft::VentPictures;
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

const BANK: u8 = 0xa6;
const FIGURE_BYTES: usize = 10;
const SPRITEMAP_POINTER: usize = 6;

/// Read the sprite maps of the seven puff figures in all four directions.
///
/// # Errors
/// A descriptor or its referenced sprite map is outside the image.
pub fn import(rom: Lorom<'_>) -> Result<VentPictures, AddressError> {
    Ok(VentPictures {
        up: direction(rom, 0xf142)?,
        left: direction(rom, 0xf188)?,
        down: direction(rom, 0xf1ce)?,
        right: direction(rom, 0xf214)?,
    })
}

fn direction(
    rom: Lorom<'_>,
    start: u16,
) -> Result<[Spritemap; VentPictures::FIGURES], AddressError> {
    let first = RomAddress::new(BANK, start);
    let mut figures = [const {
        Spritemap {
            entries: Vec::new(),
        }
    }; VentPictures::FIGURES];
    for (figure, map) in figures.iter_mut().enumerate() {
        let pointer = first
            .advance(
                figure
                    .wrapping_mul(FIGURE_BYTES)
                    .wrapping_add(SPRITEMAP_POINTER),
            )
            .ok_or(AddressError::PastEnd(first))?;
        let bytes = rom
            .read(pointer, 2)?
            .try_into()
            .map_err(|_| AddressError::PastEnd(pointer))?;
        *map = Spritemap::load(rom, RomAddress::new(BANK, u16::from_le_bytes(bytes)))?;
    }
    Ok(figures)
}
