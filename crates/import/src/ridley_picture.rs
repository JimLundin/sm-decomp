//! Import Ceres Ridley's ordinary sprite maps from the player's cartridge.

use data::assets::ridley_picture::{Art, BodyLooks};
use data::assets::world::Pair;
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::{ExtendedMap, ExtendedPiece, Spritemap, SpritemapEntry};

const BANK: u8 = 0xa6;
const WINGS: u16 = 0xdb02;
const TIPS: u16 = 0xdcba;

/// Why an ordinary fight picture could not import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// A source is outside the image.
    Address(AddressError),
    /// A map has an unexpected piece or submap count.
    Shape(RomAddress),
}

impl From<AddressError> for Fault {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Read the ordinary fight's art and validate its maps.
///
/// # Errors
/// Returns the source or shape fault for malformed art.
pub fn import(rom: Lorom<'_>) -> Result<Art, Fault> {
    let wings = |side: usize| {
        let counts = [6, 5, 3, 3, 5, 6, 5, 3, 3, 5];
        let addresses = RomAddress::new(BANK, WINGS);
        maps(counts.into_iter().enumerate().map(|(beat, count)| {
            let pointer = pointer(
                rom,
                addresses,
                side.saturating_mul(counts.len())
                    .saturating_add(beat)
                    .saturating_mul(2),
            )?;
            simple(rom, pointer, count)
        }))
    };
    let extended = |at, count| extended(rom, RomAddress::new(BANK, at), count);
    Ok(Art {
        baby: maps([0xbffd, 0xc018, 0xc033].map(|at| simple(rom, RomAddress::new(BANK, at), 5)))?,
        wings: [wings(0)?, wings(1)?],
        links: maps([0xdc90, 0xdc97, 0xdc9e].map(|at| simple(rom, RomAddress::new(BANK, at), 1)))?,
        tip: maps((0_usize..16).map(|look| {
            simple(
                rom,
                pointer(rom, RomAddress::new(BANK, TIPS), look.saturating_mul(2))?,
                1,
            )
        }))?,
        left: BodyLooks {
            rest: extended(0xe983, 4)?,
            open: extended(0xe9c7, 4)?,
            wide: extended(0xe9e9, 4)?,
            takeoff1: extended(0xea4f, 4)?,
            takeoff2: extended(0xea71, 4)?,
        },
        right: BodyLooks {
            rest: extended(0xe9a5, 4)?,
            open: extended(0xea0b, 4)?,
            wide: extended(0xea2d, 4)?,
            takeoff1: extended(0xea93, 4)?,
            takeoff2: extended(0xeab5, 4)?,
        },
        front: extended(0xead7, 1)?,
        attached_door: crate::actors::attached_ridley_door(rom)?,
    })
}

fn maps<const N: usize>(
    input: impl IntoIterator<Item = Result<Spritemap, Fault>>,
) -> Result<[Spritemap; N], Fault> {
    let maps = input.into_iter().collect::<Result<Vec<_>, _>>()?;
    maps.try_into()
        .map_err(|_| Fault::Shape(RomAddress::new(BANK, 0)))
}

fn simple(rom: Lorom<'_>, at: RomAddress, count: usize) -> Result<Spritemap, Fault> {
    if usize::from(word(rom, at, 0)?) != count {
        return Err(Fault::Shape(at));
    }
    Ok(Spritemap::load(rom, at)?)
}

fn extended(rom: Lorom<'_>, at: RomAddress, count: usize) -> Result<Vec<ExtendedMap>, Fault> {
    if usize::from(word(rom, at, 0)?) != count {
        return Err(Fault::Shape(at));
    }
    (0..count)
        .map(|index| {
            const SUBMAP_BYTES: usize = 8;
            let start = index.saturating_mul(SUBMAP_BYTES).saturating_add(2);
            let map = pointer(rom, at, start.saturating_add(4))?;
            let count = usize::from(word(rom, map, 0)?);
            if !(1..=32).contains(&count) {
                return Err(Fault::Shape(map));
            }
            let entries = map.advance(2).ok_or(AddressError::PastEnd(map))?;
            let pieces = rom
                .read(entries, count.saturating_mul(5))?
                .as_chunks::<5>()
                .0
                .iter()
                .map(|&bytes| {
                    let [low, high, ..] = bytes;
                    ExtendedPiece {
                        sprite: SpritemapEntry::from_bytes(bytes),
                        x_word: u16::from_le_bytes([low, high]),
                    }
                })
                .collect();
            Ok(ExtendedMap {
                offset: Pair {
                    x: word(rom, at, start)?.cast_signed(),
                    y: word(rom, at, start.saturating_add(2))?.cast_signed(),
                },
                pieces,
            })
        })
        .collect()
}

fn pointer(rom: Lorom<'_>, at: RomAddress, offset: usize) -> Result<RomAddress, Fault> {
    Ok(RomAddress::new(BANK, word(rom, at, offset)?))
}

fn word(rom: Lorom<'_>, at: RomAddress, offset: usize) -> Result<u16, AddressError> {
    let place = at.advance(offset).ok_or(AddressError::PastEnd(at))?;
    let &[low, high] = rom.read(place, 2)? else {
        return Err(AddressError::PastEnd(place));
    };
    Ok(u16::from_le_bytes([low, high]))
}

#[cfg(test)]
mod tests {
    use super::import;
    use data::rom::Rom;

    #[test]
    #[ignore = "player ROM"]
    fn imports_ordinary_ridley_art() {
        let path = std::env::var_os("SM_ROM").unwrap();
        let rom = Rom::new(std::fs::read(path).unwrap()).unwrap();
        let art = import(rom.lorom()).unwrap();
        for (maps, total) in [
            (art.left.rest, 26),
            (art.left.open, 27),
            (art.left.wide, 29),
            (art.left.takeoff1, 25),
            (art.left.takeoff2, 25),
        ] {
            assert_eq!(
                maps.iter().map(|map| map.pieces.len()).sum::<usize>(),
                total
            );
        }
        assert_eq!(art.attached_door.entries.len(), 8);
    }
}
