//! The importer of the new game's start: the room it begins in and her place across it, the
//! only code that knows where the original keeps them.

use data::assets::art::{Colour, ROW_COLOURS};
use data::assets::start::NewGame;
use data::rom::{AddressError, Lorom, RomAddress};
use data::room::RoomKey;

/// The table of each area's list of starts, and the station's place in it.
const AREAS: RomAddress = RomAddress::new(0x80, 0xc4b5);
const CERES: usize = 6;
/// A start's room, the view's place and her offsets, by their places in its entry.
const ROOM: usize = 0;
const VIEW_X: usize = 6;
const VIEW_Y: usize = 8;
const OFFSET_Y: usize = 10;
const OFFSET_X: usize = 12;
/// The colours a new game starts with in her row.
const HER_ROW: RomAddress = RomAddress::new(0x9a, 0x8180);
/// She starts across the middle of the view, offset by her start's own; down the view by her
/// start's offset alone.
const MIDDLE: u16 = 0x80;

/// The new game's start: the first of the Ceres station's.
///
/// # Errors
/// A table or start past the end of the image.
pub fn import(rom: Lorom<'_>) -> Result<NewGame, AddressError> {
    let list = word(rom, AREAS, CERES * 2)?;
    let start = RomAddress::new(0x80, list);
    let view = (word(rom, start, VIEW_X)?, word(rom, start, VIEW_Y)?);
    Ok(NewGame {
        room: RoomKey::from_room_pointer(word(rom, start, ROOM)?),
        view,
        her: (
            view.0
                .wrapping_add(MIDDLE)
                .wrapping_add(word(rom, start, OFFSET_X)?),
            view.1.wrapping_add(word(rom, start, OFFSET_Y)?),
        ),
        her_row: her_row(rom)?,
    })
}

/// The colours a new game starts with in her row, in order.
fn her_row(rom: Lorom<'_>) -> Result<[Colour; ROW_COLOURS], AddressError> {
    let mut colours = [Colour(0); ROW_COLOURS];
    let bytes = rom.read(HER_ROW, ROW_COLOURS * 2)?;
    for (colour, word) in colours.iter_mut().zip(bytes.as_chunks::<2>().0) {
        *colour = Colour(u16::from_le_bytes(*word));
    }
    Ok(colours)
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

// ROM sources for the data above, kept out of the public docs (the original's bank addresses):
// bank $80 log, the load stations `$80:C4B5`, one list per area; a station's entry is 14 bytes:
// the room pointer, door, door's BTS, the screens' x and y, her y offset and her x offset. A new
// game loads Ceres's station 0, room `DF45`, and puts her at the screen's x + 80h + the x
// offset and the screen's y + the y offset. Its setup loads the sprite palettes `$9A:8100`
// to the palette buffer and later copies her row (sprite palette 4, `$9A:8180`) to the target
// palettes; the target's other rows are the room's loading's.

#[cfg(test)]
mod tests {
    use data::rom::Lorom;

    /// An invented image: Ceres's list of starts names an entry whose room, view and offsets are
    /// invented, and her row's colours are invented too.
    #[test]
    fn the_start_is_ceres_first_with_her_in_the_middle_of_the_view_offset_by_its_own() {
        let mut image = vec![0_u8; 0x0d_0200];
        let mut put = |at: usize, word: u16| {
            image[at..at + 2].copy_from_slice(&word.to_le_bytes());
        };
        // The list of Ceres's starts, then its first entry.
        put(0x44b5 + 12, 0xd000);
        put(0x5000, 0xdf45);
        put(0x5006, 0x0100);
        put(0x5008, 0x0200);
        put(0x500a, 0x0048);
        put(0x500c, 0x0010);
        for place in 0..16 {
            put(
                0x0d_0180 + place * 2,
                0x0100 + u16::try_from(place).unwrap(),
            );
        }
        let start = super::import(Lorom::new(&image)).unwrap();
        assert_eq!(start.room, data::room::RoomKey::from_room_pointer(0xdf45));
        assert_eq!(start.view, (0x0100, 0x0200));
        assert_eq!(start.her, (0x0100 + 0x80 + 0x10, 0x0200 + 0x48));
        assert_eq!(start.her_row[3].0, 0x0103);
        assert!(super::import(Lorom::new(&image[..0x4000])).is_err());
    }
}
