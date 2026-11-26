//! The countdown word and its ten digit drawings from bank 80.
use data::assets::countdown::Figures;
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

/// Load the drawings used by the countdown picture.
///
/// # Errors
/// A map or pointer outside the player image is refused.
pub fn import(rom: Lorom<'_>) -> Result<Figures, AddressError> {
    let frame = Spritemap::load(rom, RomAddress::new(0x80, 0xa060))?;
    let pointers = rom.read(RomAddress::new(0x80, 0x9fd4), 20)?;
    let mut digits = std::array::from_fn(|_| Spritemap::default());
    for (digit, pointer) in digits.iter_mut().zip(pointers.as_chunks::<2>().0) {
        *digit = Spritemap::load(rom, RomAddress::new(0x80, u16::from_le_bytes(*pointer)))?;
    }
    Ok(Figures { frame, digits })
}

/// Resolve a queued shaft platform map written to the low lane of Mode 7 VRAM.
///
/// # Errors
/// A source crossing the supplied player ROM bounds is rejected.
pub fn platform_tiles(rom: Lorom<'_>, source: RomAddress) -> Result<[u8; 4], AddressError> {
    let mut tiles = [0; 4];
    tiles.copy_from_slice(rom.read(source, 4)?);
    Ok(tiles)
}
