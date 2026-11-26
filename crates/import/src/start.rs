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

/// The basic beam tiles installed by a new file's equipment update.
///
/// # Errors
/// The source pointer and the full tile strip must be readable.
pub fn beam_tiles(rom: Lorom<'_>) -> Result<(data::dma::BusAddress, Vec<u16>), AddressError> {
    let source = RomAddress::new(0x9a, word(rom, RomAddress::new(0x90, 0xc3b1), 0)?);
    let words = rom
        .read(source, 0x100)?
        .as_chunks::<2>()
        .0
        .iter()
        .copied()
        .map(u16::from_le_bytes)
        .collect();
    Ok((
        data::dma::BusAddress {
            bank: source.bank,
            offset: source.offset,
        },
        words,
    ))
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

/// A malformed opening HUD or an unsupported starting room's setup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudError {
    /// An address from the image could not be read.
    Address(AddressError),
    /// The fresh room's load recipe is outside Ceres's supported startup.
    Room,
    /// The room data could not be decoded.
    RoomData,
    /// The opening art or map window is malformed.
    Art,
}

impl From<AddressError> for HudError {
    fn from(error: AddressError) -> Self {
        Self::Address(error)
    }
}

/// Import the opening frame and first mini-map window from the selected load station.
///
/// # Errors
/// The start is outside the supported Ceres load recipe, or its content is malformed.
pub fn hud(rom: Lorom<'_>, start: &NewGame) -> Result<data::assets::start::Hud, HudError> {
    let loaded = data::room::decode(rom, start.room).map_err(|_| HudError::RoomData)?;
    if loaded.area != data::room::Area::Ceres {
        return Err(HudError::Room);
    }
    Ok(data::assets::start::Hud {
        frame: opening_frame(rom)?,
        preview: preview(rom, &loaded, start, map_masks(rom, &loaded.default)?)?,
    })
}

/// Each of the three preview rows reads the five highest mask bits left to right.
fn map_masks(rom: Lorom<'_>, state: &data::room::RoomState) -> Result<[u16; 3], HudError> {
    // The elevator FX table entry registers its effect in the HDMA bank, which becomes
    // the high byte of the first mask. Its registration is the supported load recipe.
    const ELEVATOR_FX: u8 = 0x2a;
    const FX_SCRIPT_BANK: RomAddress = RomAddress::new(0x89, 0xac45);
    const FX_ENTRIES: RomAddress = RomAddress::new(0x83, 0xac18);
    const REGISTER_FX: [u8; 4] = [0x22, 0x35, 0x84, 0x88];
    let fx_type = byte(rom, state.fx, 9)?;
    if fx_type != ELEVATOR_FX {
        return Err(HudError::Room);
    }
    let bank = byte(rom, FX_SCRIPT_BANK, 0)?;
    let entry = RomAddress::new(bank, word(rom, FX_ENTRIES, usize::from(fx_type))?);
    if rom.read(entry, REGISTER_FX.len())? != REGISTER_FX {
        return Err(HudError::Room);
    }

    // The graphics set leaves its last enemy's graphics bank and the terminator's offset.
    let mut graphics = state.enemy_set;
    let mut last_bank = None;
    loop {
        let enemy = word(rom, graphics, 0)?;
        if enemy == u16::MAX {
            break;
        }
        last_bank = Some(byte(rom, RomAddress::new(0xa0, enemy), 0x38)?);
        graphics = graphics.advance(4).ok_or(HudError::Room)?;
    }
    Ok([
        u16::from(bank) << 8,
        u16::from(last_bank.ok_or(HudError::Room)?),
        graphics.offset,
    ])
}

fn preview(
    rom: Lorom<'_>,
    loaded: &data::room::Room,
    start: &NewGame,
    masks: [u16; 3],
) -> Result<data::assets::start::MapPreview, HudError> {
    use data::map::{AreaMaps, MAP_ROWS, PAGE_COLUMNS};

    let map = AreaMaps::load(rom)?
        .0
        .into_iter()
        .find(|map| map.area == loaded.area)
        .ok_or(HudError::Art)?;
    let left = usize::from(loaded.map_position.x)
        .saturating_add(usize::from(start.her.0 / 256))
        .checked_sub(2)
        .ok_or(HudError::Art)?;
    // The padding row in the center's Y cancels the shift to the preview's top row.
    let top = usize::from(loaded.map_position.y).saturating_add(usize::from(start.her.1 / 256));
    let columns = usize::from(PAGE_COLUMNS);
    let mut preview = [[None; 5]; 3];
    for (y, (row, mask)) in preview.iter_mut().zip(masks).enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            if mask & (0x8000 >> x) != 0 {
                let x = left.saturating_add(x);
                let y = top.saturating_add(y);
                if x >= columns || y >= usize::from(MAP_ROWS) {
                    return Err(HudError::Art);
                }
                let index = y.saturating_mul(columns).saturating_add(x);
                let value = map.tiles.get(index).ok_or(HudError::Art)?;
                *cell = Some(data::hud::look(*value).ok_or(HudError::Art)?);
            }
        }
    }
    Ok(preview)
}

fn opening_frame(rom: Lorom<'_>) -> Result<[[data::assets::start::HudCell; 32]; 4], HudError> {
    use data::assets::start::HudCell;

    const FRAME: RomAddress = RomAddress::new(0x80, 0x988b);
    let mut rows = Vec::new();
    for bytes in rom.read(FRAME, 4 * 64)?.as_chunks::<64>().0 {
        let cells = bytes
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&bytes| {
                let [_, high] = bytes;
                let word = u16::from_le_bytes(bytes);
                // The output encoder sets priority for every HUD cell.
                if word & 0x2000 == 0 {
                    return Err(HudError::Art);
                }
                Ok(HudCell {
                    look: data::hud::look(word).ok_or(HudError::Art)?,
                    colours: (high >> 2) & 7,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        rows.push(cells.try_into().map_err(|_| HudError::Art)?);
    }
    rows.try_into().map_err(|_| HudError::Art)
}

fn byte(rom: Lorom<'_>, at: RomAddress, offset: usize) -> Result<u8, AddressError> {
    let at = at.advance(offset).ok_or(AddressError::PastEnd(at))?;
    rom.read(at, 1)?
        .first()
        .copied()
        .ok_or(AddressError::PastEnd(at))
}
