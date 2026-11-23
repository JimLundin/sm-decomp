//! The importer: the colours of the sprites other than hers from the ROM, and the species a
//! room's version loads, the only code that knows where the original keeps them.
//!
//! A species' header names its palette; the graphics set a room state names lists the species
//! it loads, each with the sprite palette row it draws with there. The common sprites' and
//! the flashing enemies' palettes sit at fixed places, and the Ceres doors' two colour sets
//! beside their species' code.

use data::assets::actors::{
    Actors, BLUE_BRINSTAR_FACE_BLOCK, CERES_DOOR, CERES_RIDLEY, CeresDoorColours, DoorShades,
    ELEVATOR, EnemyGraphics, GEEGA, MORPH_BALL_EYE, ROACH, SIDEHOPPER, SUPER_SIDEHOPPER, Species,
    SpriteRow, ZOOMER,
};
use data::assets::art::{Colour, Palette, ROW_COLOURS};
use data::id::SpeciesId;
use data::rom::{AddressError, Lorom, RomAddress};

/// The original's place for each registered species, in id order: its header's pointer in the
/// species bank, which a graphics set names it by.
const PLACES: [(SpeciesId, u16); 10] = [
    (MORPH_BALL_EYE, 0xe6bf),
    (ELEVATOR, 0xd73f),
    (BLUE_BRINSTAR_FACE_BLOCK, 0xea7f),
    (SUPER_SIDEHOPPER, 0xd9bf),
    (CERES_DOOR, 0xe23f),
    (CERES_RIDLEY, 0xe13f),
    (SIDEHOPPER, 0xd93f),
    (GEEGA, 0xf253),
    (ZOOMER, 0xdcff),
    (ROACH, 0xd87f),
];

/// The species headers' bank.
const SPECIES_BANK: u8 = 0xa0;
/// Where a header holds its palette's pointer, and its bank.
const PALETTE_POINTER: usize = 2;
const PALETTE_BANK: usize = 0x0c;
/// The word that ends a graphics set in place of a species.
const END_OF_SET: u16 = 0xffff;
/// The bytes of a graphics set's entry: the species' pointer and its palette word.
const ENTRY_BYTES: usize = 4;
/// The common sprites' palette and the flashing enemies' white palette.
const COMMON: RomAddress = RomAddress::new(0x9a, 0x81a0);
const FLASH: RomAddress = RomAddress::new(0x9a, 0xfc00);
/// Where a bank's ROM half starts.
const ROM_HALF: u16 = 0x8000;
/// The bytes of a palette row: a word a colour.
const ROW_BYTES: usize = 2 * ROW_COLOURS;

/// Why the actors' colours or a graphics set don't import.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorsError {
    /// A table is outside the image.
    Address(AddressError),
    /// A graphics set names a species the registry has no id for.
    Unregistered(RomAddress),
    /// A graphics set gives a species a palette row past the eight.
    Row(RomAddress),
}

impl From<AddressError> for ActorsError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Every registered species' palette, the shared sprite palettes and the Ceres doors' colours.
///
/// # Errors
///
/// [`ActorsError::Address`] when a table is outside the image.
pub fn import(rom: Lorom<'_>) -> Result<Actors, ActorsError> {
    let species = PLACES
        .iter()
        .map(|&(id, pointer)| {
            let header = RomAddress::new(SPECIES_BANK, pointer);
            let [bank, _] = word(rom, header, PALETTE_BANK)?.to_le_bytes();
            let pointer = word(rom, header, PALETTE_POINTER)?;
            // A pointer outside a bank's ROM half names no palette of the species' own.
            let palette = (pointer >= ROM_HALF)
                .then(|| palette(rom, RomAddress::new(bank, pointer)))
                .transpose()?;
            Ok(Species { id, palette })
        })
        .collect::<Result<_, AddressError>>()?;
    let doors = data::palette::CeresDoorColours::load(rom)?;
    Ok(Actors {
        species,
        common: palette(rom, COMMON)?,
        flash: palette(rom, FLASH)?,
        ceres_doors: CeresDoorColours {
            calm: shades(doors.calm),
            escape: shades(doors.escape),
        },
    })
}

/// The graphics set at `at`, a room state's: each species it loads, in order, with its row.
///
/// # Errors
///
/// [`ActorsError`] when the set runs outside the image, or names a species we have no id for
/// or a row past the eight.
pub fn graphics_set(rom: Lorom<'_>, at: RomAddress) -> Result<Vec<EnemyGraphics>, ActorsError> {
    let mut set = Vec::new();
    let mut place = at;
    loop {
        let species = word(rom, place, 0)?;
        if species == END_OF_SET {
            return Ok(set);
        }
        let named = RomAddress::new(SPECIES_BANK, species);
        let id = PLACES
            .iter()
            .find_map(|&(id, pointer)| (pointer == species).then_some(id))
            .ok_or(ActorsError::Unregistered(named))?;
        let [row, _] = word(rom, place, 2)?.to_le_bytes();
        set.push(EnemyGraphics {
            species: id,
            row: SpriteRow::new(row).ok_or(ActorsError::Row(named))?,
        });
        place = place
            .advance(ENTRY_BYTES)
            .ok_or(AddressError::PastEnd(place))?;
    }
}

/// The palette row at `at`: 16 little-endian colour words.
pub(crate) fn palette(rom: Lorom<'_>, at: RomAddress) -> Result<Palette, AddressError> {
    let bytes: &[u8; ROW_BYTES] = rom
        .read(at, ROW_BYTES)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(at))?;
    Ok(palette_of(bytes))
}

/// The colours of a row's 32 bytes.
fn palette_of(bytes: &[u8; ROW_BYTES]) -> Palette {
    let mut colours = [Colour::default(); ROW_COLOURS];
    for (colour, &word) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *colour = Colour(u16::from_le_bytes(word));
    }
    Palette(colours)
}

/// A door colour set's words as colours.
fn shades(words: data::palette::DoorShades) -> DoorShades {
    words.map(Colour)
}

/// The little-endian word `offset` bytes past `at`.
fn word(rom: Lorom<'_>, at: RomAddress, offset: usize) -> Result<u16, AddressError> {
    let place = at.advance(offset).ok_or(AddressError::PastEnd(at))?;
    let bytes: &[u8; 2] = rom
        .read(place, 2)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(place))?;
    Ok(u16::from_le_bytes(*bytes))
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and RAM
// addresses): bank $A0 log, process enemy set `$A0:8D64`: a set's entry at `$B4:[$07D1]` is
// the species' header pointer (bank $A0) and a word whose low byte is the sprite palette
// row; the species' palette is 20h bytes at the header's `+2` pointer in its `+Ch` bank, set
// at target colours `$7E:C300 + 20h * row`. Bank $82 log, `$82:E139`: the common sprites'
// palette `$9A:81A0` to target sprite palette 5, the flashing enemies' `$9A:FC00` to sprite
// palette 0. The Ceres doors' sets are `data::palette`'s.
