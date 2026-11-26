//! Read the Ceres flyby's path and art from the player's ROM.

use data::assets::art::Colour;
use data::assets::ridley_flyby::{Flyby, PathPicture, Tiles};
use data::assets::world::Pair;
use data::rom::{AddressError, Lorom, RomAddress};

const BANK: u8 = 0xa6;
const ZOOM: u16 = 0xae4d;
const X_SHIFT: u16 = 0xb00f;
const Y_SHIFT: u16 = 0xaf2f;
const BODY: u16 = 0xb107;
const BABY: u16 = 0xacda;
const WINGS: u16 = 0xad45;
const PICTURES: usize = 112;
const DESCRIPTOR_BYTES: usize = 9;

/// A flyby source table is missing or malformed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// An asset read leaves the image.
    Address(AddressError),
    /// The path has an unexpected length, terminator or zoom.
    Path,
    /// A body colour exceeds 15 bits.
    Body,
    /// A tile descriptor does not describe the expected low-byte update.
    Tiles,
}

impl From<AddressError> for Fault {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

/// Import the complete Ceres flyby path, palettes and tilemap updates.
///
/// # Errors
/// Returns the table's fault if a source is unreadable or malformed.
pub fn import(rom: Lorom<'_>) -> Result<Flyby, Fault> {
    let path = (0..PICTURES)
        .map(|picture| {
            let offset = picture.saturating_mul(2);
            let zoom = word(rom, ZOOM, offset)?;
            if zoom > 2048 {
                return Err(Fault::Path);
            }
            let shift =
                |table| word(rom, table, offset).map(|bits| i16::from_le_bytes(bits.to_le_bytes()));
            Ok(PathPicture {
                shift: Pair {
                    x: shift(X_SHIFT)?.wrapping_neg(),
                    y: shift(Y_SHIFT)?,
                },
                zoom,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    if word(rom, ZOOM, PICTURES.saturating_mul(2))? != u16::MAX {
        return Err(Fault::Path);
    }
    let mut body = [[Colour::default(); 15]; 9];
    for (set, colours) in body.iter_mut().enumerate() {
        for (index, output) in colours.iter_mut().enumerate() {
            let offset = set
                .saturating_mul(32)
                .saturating_add(index.saturating_mul(2));
            let colour = word(rom, BODY, offset)?;
            if colour > 0x7fff {
                return Err(Fault::Body);
            }
            *output = Colour(colour);
        }
    }
    Ok(Flyby {
        path,
        body,
        baby: looks(rom, BABY, 2)?,
        wings: looks(rom, WINGS, 6)?,
    })
}

fn looks<const N: usize>(
    rom: Lorom<'_>,
    table: u16,
    writes: usize,
) -> Result<[Vec<Tiles>; N], Fault> {
    (0..N)
        .map(|look: usize| {
            let pointer = word(rom, table, look.saturating_mul(2))?;
            let patches = (0..writes)
                .map(|index: usize| tiles(rom, pointer, index.saturating_mul(DESCRIPTOR_BYTES)))
                .collect::<Result<Vec<_>, _>>()?;
            let end = address(pointer, writes.saturating_mul(DESCRIPTOR_BYTES))?;
            if rom.read(end, 1)? != [0] {
                return Err(Fault::Tiles);
            }
            Ok(patches)
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .map_err(|_| Fault::Tiles)
}

fn tiles(rom: Lorom<'_>, table: u16, offset: usize) -> Result<Tiles, Fault> {
    let bytes = rom.read(address(table, offset)?, DESCRIPTOR_BYTES)?;
    let [
        0x80,
        low,
        high,
        bank,
        count_low,
        count_high,
        start_low,
        start_high,
        0,
    ] = *bytes
    else {
        return Err(Fault::Tiles);
    };
    let count = u16::from_le_bytes([count_low, count_high]);
    let start = u16::from_le_bytes([start_low, start_high]);
    if count == 0 || start.checked_add(count).is_none_or(|end| end > 0x4000) {
        return Err(Fault::Tiles);
    }
    let source = RomAddress::from_long([low, high, bank]);
    Ok(Tiles {
        start,
        low: rom.read(source, usize::from(count))?.to_vec(),
    })
}

fn word(rom: Lorom<'_>, table: u16, offset: usize) -> Result<u16, Fault> {
    let place = address(table, offset)?;
    let &[low, high] = rom.read(place, 2)? else {
        return Err(Fault::Address(AddressError::PastEnd(place)));
    };
    Ok(u16::from_le_bytes([low, high]))
}

fn address(table: u16, offset: usize) -> Result<RomAddress, Fault> {
    let base = RomAddress::new(BANK, table);
    base.advance(offset)
        .ok_or(Fault::Address(AddressError::PastEnd(base)))
}

#[cfg(test)]
mod tests {
    use super::{PICTURES, import};
    use data::rom::Rom;

    #[test]
    #[ignore = "player ROM"]
    fn native_flyby_imports_with_complete_path_and_art() {
        let path = std::env::var_os("SM_ROM").unwrap();
        let image = Rom::new(std::fs::read(path).unwrap()).unwrap();
        let flyby = import(image.lorom()).unwrap();
        assert_eq!(flyby.path.len(), PICTURES);
        assert!(flyby.baby.iter().all(|look| look.len() == 2));
        assert!(flyby.wings.iter().all(|look| look.len() == 6));
    }
}
