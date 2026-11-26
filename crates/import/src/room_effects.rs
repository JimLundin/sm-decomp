//! Import the shaft's two finite enemy-projectile clips as ordinary sprite pictures.

use core::num::NonZeroU16;
use data::assets::room_effects::{Picture, ShaftBurst};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

const SMALL_EXPLOSION: RomAddress = RomAddress::new(0x86, 0xe138);
const SMOKE: RomAddress = RomAddress::new(0x86, 0xe1ea);
const SPRITEMAP_BANK: u8 = 0x8d;
const MIN_POINTER: u16 = 0x8000;
const DELETE: u16 = 0x8154;
const FRAME_BYTES: usize = 4;
const MAX_PIECES: usize = 128;

/// A malformed clip or a source range outside the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImportError {
    /// The clip or sprite picture could not be read.
    Address(AddressError),
    /// A frame has an invalid duration, sprite picture, or terminal marker.
    Clip(RomAddress),
}

impl From<AddressError> for ImportError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Read both fixed clips from the authenticated release's `LoROM` view.
///
/// # Errors
/// Returns a fault for out-of-bounds content, invalid durations or pictures, or a clip
/// that does not terminate after its last picture.
pub fn shaft_burst(rom: Lorom<'_>) -> Result<ShaftBurst, ImportError> {
    Ok(ShaftBurst {
        smoke: finite_clip(rom, SMOKE)?,
        small_explosion: finite_clip(rom, SMALL_EXPLOSION)?,
    })
}

fn finite_clip<const N: usize>(
    rom: Lorom<'_>,
    start: RomAddress,
) -> Result<[Picture; N], ImportError> {
    let mut cursor = start;
    let mut pictures = Vec::with_capacity(N);
    for _ in 0..N {
        pictures.push(picture(rom, cursor)?);
        cursor = cursor
            .advance(FRAME_BYTES)
            .ok_or(ImportError::Clip(cursor))?;
    }
    if rom.read(cursor, 2)? != DELETE.to_le_bytes() {
        return Err(ImportError::Clip(cursor));
    }
    pictures.try_into().map_err(|_| ImportError::Clip(start))
}

fn picture(rom: Lorom<'_>, cursor: RomAddress) -> Result<Picture, ImportError> {
    let [low, high, map_low, map_high] = rom.read(cursor, FRAME_BYTES)? else {
        return Err(ImportError::Clip(cursor));
    };
    let hold = NonZeroU16::new(u16::from_le_bytes([*low, *high]))
        .filter(|duration| duration.get() < MIN_POINTER)
        .ok_or(ImportError::Clip(cursor))?;
    let sprites = Spritemap::load(
        rom,
        RomAddress::new(SPRITEMAP_BANK, u16::from_le_bytes([*map_low, *map_high])),
    )?;
    if sprites.entries.is_empty() || sprites.entries.len() > MAX_PIECES {
        return Err(ImportError::Clip(cursor));
    }
    Ok(Picture { hold, sprites })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "requires the player's ROM"]
    fn authenticated_release_supplies_both_effects() {
        let path = std::env::var_os("SM_ROM").unwrap();
        let rom = data::rom::Rom::new(std::fs::read(path).unwrap()).unwrap();
        let art = shaft_burst(rom.lorom()).unwrap();
        assert!(
            art.smoke
                .iter()
                .all(|frame| frame.sprites.entries.len() == 1)
        );
        assert!(
            art.small_explosion
                .iter()
                .any(|frame| frame.sprites.entries.len() > 1)
        );
    }

    #[test]
    fn finite_clip_preserves_pictures_and_rejects_extra_control() {
        let mut image = vec![0; data::rom::Rom::SIZE];
        let map = RomAddress::new(SPRITEMAP_BANK, MIN_POINTER);
        let clip_start = Lorom::new(&image).file_offset(SMOKE).unwrap();
        let map_start = Lorom::new(&image).file_offset(map).unwrap();
        image[map_start] = 1;
        let mut bytes = [5, 0, 0, 0x80, 0, 0];
        bytes[4..].copy_from_slice(&DELETE.to_le_bytes());
        image[clip_start..clip_start + bytes.len()].copy_from_slice(&bytes);
        let [frame] = finite_clip(Lorom::new(&image), SMOKE).unwrap();
        assert_eq!(frame.hold.get(), 5);
        assert_eq!(frame.sprites.entries.len(), 1);

        image[clip_start + 4] = 0;
        assert!(finite_clip::<1>(Lorom::new(&image), SMOKE).is_err());
    }
}
