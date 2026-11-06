//! The mini-map: the explored cells Samus moves through, and the five by three cells of the
//! area map around her drawn into the HUD tilemap, her own cell blinking.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): every Samus new state handler runs
//! update mini-map (`$90:A91B`) after the movement handler (`$90:E734`, `$90:E8F8` riding the
//! elevator). It marks Samus's cell explored (`$90:A97E`), reads the explored and the map
//! station bits of the window's three rows (`$90:A9A2`, `$90:A9D6`), takes the bits that spill
//! onto the other map page (`$90:AB75`), then update HUD mini-map tilemap (`$90:AA43`) draws
//! the window and gives Samus's cell palette 7 on half the frames (`$90:AB4A`).

use data::map::{AreaMap, AreaMaps, MapBits, MapByte};

use crate::state::hud::{Cell, HudTilemap, MiniMap};
use crate::state::samus::Point;
use crate::state::video::{BgPalette, TilemapEntry};

/// The window's top-left cell in the HUD tilemap (`$90:AAB7`).
pub const ORIGIN: Cell = Cell { row: 0, column: 26 };
/// The window's columns, centred on Samus's (`$90:AA98`).
pub const COLUMNS: usize = 5;
/// The window's rows: the one above Samus's, hers and the one below.
pub const ROWS: usize = 3;
/// Samus's cell, the window's centre (`$90:AB52`).
pub const SAMUS_CELL: Cell = ORIGIN.offset(1, 2);

/// The tile a cell shows when it is neither explored nor revealed (`$90:AAAE`).
const BLANK_TILE: u16 = 0x001F;
/// A cell its map station reveals (`$90:AAB4`).
const REVEALED_PALETTE: BgPalette = BgPalette(3);
/// An explored cell (`$90:AAC4`).
const EXPLORED_PALETTE: BgPalette = BgPalette(2);
/// Samus's cell while it blinks (`$90:AB56`).
const SAMUS_PALETTE: BgPalette = BgPalette(7);
/// Samus's cell blinks while this bit of the requested-NMI byte is clear (`$90:AB4D`).
const BLINK_BIT: u8 = 0x08;
/// The tile whose cell above Samus counts as explored too, used in a diagonal room each of
/// Crateria and Norfair (`$90:AB00`), compared on the tile number's low nine bits.
const DIAGONAL_TILE: u16 = 0x0028;
const DIAGONAL_TILE_BITS: u16 = 0x01FF;
/// The right map page's bit in a map column (`$90:A94F`).
const RIGHT_PAGE: u16 = 0x20;
/// The window's bits in a row's two bytes: six bits, though five are drawn (`$90:AC0C`).
const WINDOW_BITS: u16 = 0xFC00;
/// A map bitmap's bytes per row.
const ROW_BYTES: u16 = 4;
/// A map tilemap's bytes per row, and from the end of a page's row to the start of the next
/// page's (`$90:AB3E`).
const TILEMAP_ROW_BYTES: u16 = 0x40;
const TILEMAP_PAGE_SKIP: u16 = 0x07C0;

/// A room's size in blocks, which bounds where the mini-map updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomBlocks {
    /// Blocks across.
    pub width: u16,
    /// Blocks down.
    pub height: u16,
}

/// Samus's cell on the area map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    /// 0 on the left page, [`RIGHT_PAGE`] on the right.
    page: u16,
    /// The column within the page.
    column: u16,
    /// The row, counting the map's padding row above the first.
    row: u16,
    /// The column's bit within its bitmap byte, from the most significant.
    bit: u16,
    /// The cell's bitmap byte.
    byte: MapByte,
}

impl Position {
    /// Samus's cell (`$90:A943` to `$90:A97B`).
    fn of(room: data::room::MapPosition, samus: Point) -> Self {
        let x = (samus.x.pixels >> 8).wrapping_add(u16::from(room.x));
        let row = (samus.y.pixels >> 8)
            .wrapping_add(u16::from(room.y))
            .wrapping_add(1);
        let page = x & RIGHT_PAGE;
        let column = x & 0x1F;
        let byte = row
            .wrapping_add(page)
            .wrapping_mul(ROW_BYTES)
            .wrapping_add(column >> 3);
        Self {
            page,
            column,
            row,
            bit: column & 7,
            byte: MapByte(byte),
        }
    }

    const fn mask(self) -> u8 {
        0x80 >> self.bit
    }
}

/// A bitmap byte; an index past the bitmap reads as no cells.
// The original reads the neighbouring RAM or ROM there (docs/LEGACY.md, the mini-map's edges).
fn byte(bits: &MapBits, at: MapByte) -> u8 {
    bits.byte(at).unwrap_or(0)
}

/// The two bytes at `at` as one word, the left byte high (`$90:A9A5`: `XBA`).
fn row_word(bits: &MapBits, at: MapByte) -> u16 {
    u16::from_be_bytes([byte(bits, at), byte(bits, at.add(1))])
}

/// `word` with its high or low byte replaced by `with`.
const fn replace_byte(word: u16, high: bool, with: u8) -> u16 {
    let [low, top] = word.to_le_bytes();
    if high {
        u16::from_le_bytes([low, with])
    } else {
        u16::from_le_bytes([with, top])
    }
}

/// The window's explored and revealed bits, one word per row, the leftmost cell's bit at
/// 8000h.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WindowBits {
    explored: [u16; ROWS],
    revealed: [u16; ROWS],
}

impl WindowBits {
    /// The window's bits around `at` (`$90:A98D` to `$90:AA41`).
    fn around(at: Position, explored: &MapBits, station: &MapBits) -> Self {
        // One row up and two columns left, which may be the byte before.
        let crossed = at.bit < 2;
        let shift = at.bit.wrapping_sub(2) & 7;
        let origin = at.byte.sub(ROW_BYTES).sub(u16::from(crossed));
        let mut bits = Self::default();
        for (row, (explored_row, revealed_row)) in
            (0_u16..).zip(bits.explored.iter_mut().zip(bits.revealed.iter_mut()))
        {
            let at = origin.add(row.wrapping_mul(ROW_BYTES));
            *explored_row = row_word(explored, at) & (WINDOW_BITS >> shift);
            *revealed_row = row_word(station, at);
        }
        // The origin's byte ends its row: the window's second byte belongs to the same row
        // of the other page (`$90:A9F9`). The original tests the bit of whichever byte holds
        // the window's left end.
        let edge_bit = if crossed { shift } else { at.bit };
        if origin.0 & 3 == 3 && edge_bit >= 6 {
            let [low, _] = origin.0.to_le_bytes();
            let spill = if at.page == 0 {
                low.wrapping_add(0x7D)
            } else {
                low.wrapping_sub(0x7C)
            };
            let spill = MapByte(u16::from(spill));
            let high = at.page == RIGHT_PAGE;
            for (row, (explored_row, revealed_row)) in
                (0_u16..).zip(bits.explored.iter_mut().zip(bits.revealed.iter_mut()))
            {
                let at = spill.add(row.wrapping_mul(ROW_BYTES));
                *explored_row = replace_byte(*explored_row, high, byte(explored, at));
                *revealed_row = replace_byte(*revealed_row, high, byte(station, at));
            }
        }
        for row in bits.explored.iter_mut().chain(bits.revealed.iter_mut()) {
            *row <<= shift;
        }
        bits
    }

    /// Whether the next column's cell of `row` is set, moving on to the column after it.
    const fn take(word: &mut u16) -> bool {
        let set = *word & 0x8000 != 0;
        *word <<= 1;
        set
    }
}

/// The tilemap's byte offset of the window's top-left cell (`$90:AA43` to `$90:AA72`).
const fn window_tiles(at: Position) -> u16 {
    let bit_index = at
        .row
        .wrapping_add(at.page)
        .swap_bytes()
        .wrapping_shr(3)
        .wrapping_add(at.column);
    // One row up and two columns left; on the right page's first two columns, also one
    // row down and one page left, so the left columns come from the left page's row.
    let start = if at.page != 0 && bit_index & 0x1F < 2 {
        bit_index.wrapping_sub(0x0402)
    } else {
        bit_index.wrapping_sub(0x0022)
    };
    start.wrapping_mul(2)
}

/// The map tilemap word `offset` bytes plus `row` rows into `map`; past the tilemap it is the
/// blank tile.
// The original reads the following ROM there (docs/LEGACY.md, the mini-map's edges).
fn tile(map: Option<&AreaMap>, offset: u16, row: u16) -> u16 {
    let bytes = usize::from(offset).wrapping_add(usize::from(row.wrapping_mul(TILEMAP_ROW_BYTES)));
    map.and_then(|map| map.tile(bytes >> 1))
        .unwrap_or(BLANK_TILE)
}

/// `word` drawn in `palette` above the background, keeping its tile and flips (`$90:AAB1`:
/// `AND #$E3FF`, `ORA`).
const fn in_palette(word: u16, palette: BgPalette) -> TilemapEntry {
    let mut entry = TilemapEntry::from_word(word);
    entry.palette = palette;
    entry.priority = true;
    entry
}

/// Update the mini-map for Samus at `samus` in a room of `room`'s size: mark her cell
/// explored and draw the window around her into `tilemap` (`$90:A91B`). `requested` is the
/// requested-NMI byte the blink reads.
pub fn update(
    mini_map: &mut MiniMap,
    tilemap: &mut HudTilemap,
    samus: Point,
    room: RoomBlocks,
    requested: u8,
    maps: &AreaMaps,
) {
    if mini_map.disabled || samus.x.pixels >> 4 >= room.width || samus.y.pixels >> 4 >= room.height
    {
        return;
    }
    let at = Position::of(mini_map.room, samus);
    mini_map.explored.mark(at.byte, at.mask());
    let map = maps.of(mini_map.area);
    let empty = MapBits::default();
    let station = map.map_or(&empty, |map| &map.station);
    let mut bits = WindowBits::around(at, &mini_map.explored, station);
    // open: the original leaves the tilemap index of Samus's cell in `$7E:060B` (`$90:AA4F`),
    // a word other routines reuse; nothing in slice 0 reads it.
    let mut offset = window_tiles(at);
    for column in 0..COLUMNS {
        for (row, (explored, revealed)) in
            (0_u16..).zip(bits.explored.iter_mut().zip(bits.revealed.iter_mut()))
        {
            let cell = ORIGIN.offset(usize::from(row), column);
            let word = tile(map, offset, row);
            let shown = if WindowBits::take(revealed) && mini_map.map_collected {
                word
            } else {
                BLANK_TILE
            };
            tilemap.set(cell, in_palette(shown, REVEALED_PALETTE));
            if WindowBits::take(explored) {
                let entry = in_palette(word, EXPLORED_PALETTE);
                tilemap.set(cell, entry);
                if cell == SAMUS_CELL && entry.tile.0 & DIAGONAL_TILE_BITS == DIAGONAL_TILE {
                    // `$90:AB5F`: the cell above Samus.
                    mini_map.explored.mark(at.byte.sub(ROW_BYTES), at.mask());
                }
            }
        }
        offset = offset.wrapping_add(2);
        if offset & (TILEMAP_ROW_BYTES - 1) == 0 {
            offset = offset.wrapping_add(TILEMAP_PAGE_SKIP);
        }
    }
    if requested & BLINK_BIT == 0
        && let Some(mut entry) = tilemap.get(SAMUS_CELL)
    {
        entry.palette = SAMUS_PALETTE;
        tilemap.set(SAMUS_CELL, entry);
    }
}

#[cfg(test)]
mod tests;
