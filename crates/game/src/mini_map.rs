//! The mini-map: the explored cells Samus moves through, and the five by three cells of the
//! area map around her drawn into the HUD tilemap, her own cell blinking.
//!
//! Bank $90 log (<https://patrickjohnston.org/bank/90>): every Samus new state handler runs
//! update mini-map (`$90:A91B`) after the movement handler (`$90:E734`, `$90:E8F8` riding the
//! elevator). It marks Samus's cell explored (`$90:A97E`), reads the explored and the map
//! station cells of the window's three rows (`$90:A9A2`, `$90:A9D6`), takes the cells that
//! spill onto the other map page (`$90:AB75`), then update HUD mini-map tilemap (`$90:AA43`)
//! draws the window and gives Samus's cell palette 7 on half the frames (`$90:AB4A`).
//!
//! The original walks the map in [`LayoutIndex`] order, so at a page's edge its window reads
//! cells a page or a row away; [`Crossing`] names what it reads there.

use data::map::{AreaMap, AreaMaps, LayoutIndex, MapCells, PAGE_COLUMNS, Page};

use crate::state::hud::{Cell, HudTilemap, MiniMap};
use crate::state::samus::Point;
use crate::state::video::{BgPalette, TileNumber, TilemapEntry};

/// The window's top-left cell in the HUD tilemap (`$90:AAB7`).
pub const ORIGIN: Cell = Cell { row: 0, column: 26 };
/// The window's columns, centred on Samus's (`$90:AA98`).
pub const COLUMNS: usize = 5;
/// The window's rows: the one above Samus's, hers and the one below.
pub const ROWS: usize = 3;
/// Samus's cell, the window's centre (`$90:AB52`).
pub const SAMUS_CELL: Cell = ORIGIN.offset(1, 2);
/// The window's columns either side of Samus's.
const REACH: u8 = 2;

/// The tile a cell shows when it is neither explored nor revealed (`$90:AAAE`).
const BLANK_TILE: u16 = 0x001F;
/// A cell its map station reveals (`$90:AAB4`).
const REVEALED_PALETTE: BgPalette = BgPalette(3);
/// An explored cell (`$90:AAC4`).
const EXPLORED_PALETTE: BgPalette = BgPalette(2);
/// Samus's cell while it blinks (`$90:AB56`).
const SAMUS_PALETTE: BgPalette = BgPalette(7);
/// The tiles whose cell above Samus counts as explored too, used in a diagonal room each of
/// Crateria and Norfair (`$90:AB00`). The original compares the tile number's low nine bits
/// with 28h, so tile 228h counts as well.
const DIAGONAL_TILES: [TileNumber; 2] = [TileNumber(0x0028), TileNumber(0x0228)];
/// The move from the start of the row below to the same row's start on the right page: the
/// original's step onto the other page.
const TO_OTHER_PAGE: i32 = LayoutIndex::PAGE - LayoutIndex::ROW;
/// The original computes the tilemap's row in eight bits (`$90:AA4B`: `XBA`), wrapping at
/// 100h rows.
const TILE_ROW_WRAP: i32 = 0x100 * LayoutIndex::ROW;
/// The original keeps its tilemap cursor as a 16-bit byte offset, wrapping at 8000h words.
const TILE_CURSOR_WRAP: i32 = 0x8000;

/// A room's size in blocks, which bounds where the mini-map updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomBlocks {
    /// Blocks across.
    pub width: u16,
    /// Blocks down.
    pub height: u16,
}

/// Whether Samus's cell shows in her palette: for eight requested NMIs, then not for the
/// next eight (`$90:AB4D`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Blink {
    /// In palette 7.
    Lit,
    /// In the explored cells' palette.
    Unlit,
}

impl Blink {
    /// The requested NMIs each phase lasts.
    const PHASE: u8 = 8;

    /// The blink after `requested` requested NMIs, counted in eight bits.
    #[must_use]
    pub const fn at(requested: u8) -> Self {
        if (requested / Self::PHASE).is_multiple_of(2) {
            Self::Lit
        } else {
            Self::Unlit
        }
    }
}

/// The edge of its page the window crosses, which decides where the original reads the
/// cells either side of it (`$90:A9F9`, `$90:AB75`).
///
/// The original assumes a window on the left page can only cross its right edge, and one on
/// the right page only its left. So it moves the cells on the other page's side of the edge
/// a page over toward it and one row back ([`TO_OTHER_PAGE`]), undoing the layout's step
/// onto the next row. It moves them in eight bits of the bitmap's bytes, wrapping around the
/// whole map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Crossing {
    /// The window is inside its page.
    Inside,
    /// The window crosses into the other page: the cells past the edge are the other page's,
    /// on the same rows.
    IntoOtherPage,
    /// The window crosses the map's outer edge, the left page's left or the right page's
    /// right. The cells past it read on in layout order, from the same page's row above or
    /// below, and the window's own cells move: the left page's are read from the right
    /// page's row above, the right page's from the left page's row below.
    PastMap,
}

/// Samus's cell on the area map.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Position {
    page: Page,
    /// The column within the page.
    column: u8,
    /// Her place in layout order; a row past the map's runs on (`$90:A943` to `$90:A97B`).
    at: LayoutIndex,
}

impl Position {
    /// Samus's cell: her screen in the room, from the room's top-left cell, one row down for
    /// the map's padding row.
    fn of(room: data::room::MapPosition, samus: Point) -> Self {
        let x = screen(samus.x.pixels).wrapping_add(u16::from(room.x));
        let row = screen(samus.y.pixels)
            .wrapping_add(u16::from(room.y))
            .wrapping_add(1);
        let columns = u16::from(PAGE_COLUMNS);
        let column = x
            .checked_rem(columns)
            .and_then(|column| u8::try_from(column).ok())
            .unwrap_or(0);
        let page = if x.checked_div(columns).unwrap_or(0).is_multiple_of(2) {
            Page::Left
        } else {
            Page::Right
        };
        Self {
            page,
            column,
            at: LayoutIndex::on_page(page, row, column),
        }
    }

    const fn crossing(self) -> Crossing {
        let near_left = self.column < REACH;
        let near_right = self.column >= PAGE_COLUMNS - REACH;
        match self.page {
            Page::Left if near_right => Crossing::IntoOtherPage,
            Page::Right if near_left => Crossing::IntoOtherPage,
            Page::Left if near_left => Crossing::PastMap,
            Page::Right if near_right => Crossing::PastMap,
            Page::Left | Page::Right => Crossing::Inside,
        }
    }

    /// The step toward the other page.
    const fn toward_other_page(self) -> i32 {
        match self.page {
            Page::Left => TO_OTHER_PAGE,
            Page::Right => -TO_OTHER_PAGE,
        }
    }

    /// Where the original reads window cell `column` of `row`, both counted from the
    /// window's top-left (`$90:A98D` to `$90:AA41`).
    fn source(self, row: i32, column: i32) -> LayoutIndex {
        let offset = column.wrapping_sub(i32::from(REACH));
        let top = self.at.rows(-1).add(offset);
        let in_page =
            (0..i32::from(PAGE_COLUMNS)).contains(&i32::from(self.column).wrapping_add(offset));
        let moved = match self.crossing() {
            Crossing::Inside => false,
            Crossing::IntoOtherPage => !in_page,
            Crossing::PastMap => in_page,
        };
        if moved {
            top.add(self.toward_other_page())
                .wrapped(LayoutIndex::MAP)
                .rows(row)
        } else {
            top.rows(row)
        }
    }

    /// The window's top-left tile: on the right page's first columns, moved onto the left
    /// page's row as the cells are (`$90:AA43` to `$90:AA72`).
    fn first_tile(self) -> TileCursor {
        let at = self
            .at
            .wrapped(TILE_ROW_WRAP)
            .rows(-1)
            .sub(i32::from(REACH));
        let at = match (self.page, self.crossing()) {
            (Page::Right, Crossing::IntoOtherPage) => at.sub(TO_OTHER_PAGE),
            (Page::Left | Page::Right, _) => at,
        };
        TileCursor(at.wrapped(TILE_CURSOR_WRAP))
    }
}

/// The screen a coordinate is in.
const fn screen(pixels: u16) -> u16 {
    pixels >> 8
}

/// The block a coordinate is in.
const fn block(pixels: u16) -> u16 {
    pixels >> 4
}

/// The map tilemap place of a window column's top cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TileCursor(LayoutIndex);

impl TileCursor {
    /// The next column's: past a row's end, the same row of the other page (`$90:AB3E`).
    const fn next(self) -> Self {
        let next = self.0.add(1).wrapped(TILE_CURSOR_WRAP);
        if next.starts_row() {
            Self(next.add(TO_OTHER_PAGE).wrapped(TILE_CURSOR_WRAP))
        } else {
            Self(next)
        }
    }

    /// The map tilemap word `row` rows down; off the map, the blank tile.
    // The original reads the following ROM there (docs/LEGACY.md, the mini-map's edges).
    fn tile(self, map: Option<&AreaMap>, row: i32) -> u16 {
        map.and_then(|map| map.tile(self.0.rows(row)))
            .unwrap_or(BLANK_TILE)
    }
}

/// `word` drawn in `palette` above the background, keeping its tile and flips (`$90:AAB1`:
/// `AND #$E3FF`, `ORA`).
const fn in_palette(word: u16, palette: BgPalette) -> TilemapEntry {
    let mut entry = TilemapEntry::from_word(word);
    entry.palette = palette;
    entry.priority = true;
    entry
}

/// What the original reads of one window cell before it draws any.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Seen {
    explored: bool,
    revealed: bool,
}

/// Update the mini-map for Samus at `samus` in a room of `room`'s size: mark her cell
/// explored and draw the window around her into `tilemap` (`$90:A91B`).
pub fn update(
    mini_map: &mut MiniMap,
    tilemap: &mut HudTilemap,
    samus: Point,
    room: RoomBlocks,
    blink: Blink,
    maps: &AreaMaps,
) {
    if mini_map.disabled
        || block(samus.x.pixels) >= room.width
        || block(samus.y.pixels) >= room.height
    {
        return;
    }
    let at = Position::of(mini_map.room, samus);
    if let Some(cell) = at.at.cell() {
        mini_map.explored.insert(cell);
    }
    let map = maps.of(mini_map.area);
    let empty = MapCells::default();
    let station = map.map_or(&empty, |map| &map.station);
    let mut seen = [[Seen::default(); COLUMNS]; ROWS];
    for (row, cells) in (0_i32..).zip(seen.iter_mut()) {
        for (column, seen) in (0_i32..).zip(cells.iter_mut()) {
            let source = at.source(row, column);
            *seen = Seen {
                explored: mini_map.explored.contains_at(source),
                revealed: station.contains_at(source),
            };
        }
    }
    // open: the original leaves the tilemap index of Samus's cell in `$7E:060B` (`$90:AA4F`),
    // a word other routines reuse; nothing in slice 0 reads it.
    let mut cursor = at.first_tile();
    for column in 0..COLUMNS {
        for (row, (window_row, cells)) in (0_i32..).zip((0..ROWS).zip(&seen)) {
            let Some(seen) = cells.get(column) else {
                continue;
            };
            let cell = ORIGIN.offset(window_row, column);
            let word = cursor.tile(map, row);
            let shown = if seen.revealed && mini_map.map_collected {
                word
            } else {
                BLANK_TILE
            };
            tilemap.set(cell, in_palette(shown, REVEALED_PALETTE));
            if seen.explored {
                let entry = in_palette(word, EXPLORED_PALETTE);
                tilemap.set(cell, entry);
                // `$90:AB5F`: the cell above Samus.
                if cell == SAMUS_CELL
                    && DIAGONAL_TILES.contains(&entry.tile)
                    && let Some(above) = at.at.rows(-1).cell()
                {
                    mini_map.explored.insert(above);
                }
            }
        }
        cursor = cursor.next();
    }
    if blink == Blink::Lit
        && let Some(mut entry) = tilemap.get(SAMUS_CELL)
    {
        entry.palette = SAMUS_PALETTE;
        tilemap.set(SAMUS_CELL, entry);
    }
}

#[cfg(test)]
mod tests;
