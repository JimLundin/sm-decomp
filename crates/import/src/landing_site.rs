//! Import the standard graphics reached by the Landing Site loader.

use data::assets::landing_site::Graphics;
use data::dma::BusAddress;
use data::rom::{AddressError, Lorom, RomAddress};

const SPRITES: RomAddress = RomAddress::new(0x9a, 0xd200);

/// Standard BG3 graphics and the ROM portion of the sprite transfer.
///
/// # Errors
/// The invariant graphics ranges must be readable in the supplied image.
pub fn import(rom: Lorom<'_>) -> Result<Graphics, AddressError> {
    let background = rom
        .read(RomAddress::new(0x9a, 0xb200), 0x2000)?
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect();
    let sprites = rom.read(SPRITES, 0x2e00)?.to_vec();
    Ok(Graphics {
        background,
        sprites,
        sprite_source: BusAddress {
            bank: SPRITES.bank,
            offset: SPRITES.offset,
        },
        sprite_length: 0x4000,
    })
}
