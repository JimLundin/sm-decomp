use data::map::{AreaMap, AreaMaps, MAP_TILEMAP_WORDS, MapBits, MapByte};
use data::room::{Area, MapPosition};

use super::{ORIGIN, RoomBlocks, SAMUS_CELL, update};
use crate::state::hud::{HudTilemap, MiniMap};
use crate::state::samus::{Fixed, Point};
use crate::state::video::{BgPalette, TileNumber, TilemapEntry};

/// Brinstar's map, each tile numbered by its tilemap index's low ten bits.
fn maps(station: MapBits) -> AreaMaps {
    let tiles = (0..MAP_TILEMAP_WORDS)
        .map(|index| u16::try_from(index).unwrap() & 0x03FF)
        .collect();
    AreaMaps(vec![AreaMap {
        area: Area::Brinstar,
        tiles,
        station,
    }])
}

fn mini_map() -> MiniMap {
    MiniMap {
        area: Area::Brinstar,
        ..MiniMap::default()
    }
}

/// Samus at pixel (`x`, `y`).
fn samus(x: u16, y: u16) -> Point {
    Point {
        x: Fixed {
            pixels: x,
            subpixels: 0,
        },
        y: Fixed {
            pixels: y,
            subpixels: 0,
        },
    }
}

const ROOM: RoomBlocks = RoomBlocks {
    width: 0x80,
    height: 0x40,
};

/// Bit 3 set: Samus's cell isn't blinking.
const STEADY: u8 = 0x08;

fn entry(tile: u16, palette: u8) -> TilemapEntry {
    TilemapEntry {
        tile: TileNumber(tile),
        palette: BgPalette(palette),
        priority: true,
        flip_x: false,
        flip_y: false,
    }
}

#[test]
fn samus_cell_is_explored_and_drawn_from_the_map() {
    // Map cell (2, 1): row 2 counting the padding row, byte 2 * 4, bit 2.
    let mut state = mini_map();
    let mut tilemap = HudTilemap::default();
    update(
        &mut state,
        &mut tilemap,
        samus(0x0280, 0x0180),
        ROOM,
        STEADY,
        &maps(MapBits::default()),
    );
    assert_eq!(state.explored.byte(MapByte(8)), Some(0x20));
    // Her cell shows the map's row 2, column 2: tilemap index 66, in the explored palette.
    assert_eq!(tilemap.get(SAMUS_CELL), Some(entry(66, 2)));
    // The cells around her are neither explored nor revealed: blank, in palette 3.
    assert_eq!(tilemap.get(ORIGIN), Some(entry(0x1F, 3)));
    assert_eq!(tilemap.get(ORIGIN.offset(2, 4)), Some(entry(0x1F, 3)));
}

#[test]
fn samus_cell_blinks_in_palette_7() {
    let mut state = mini_map();
    let mut tilemap = HudTilemap::default();
    update(
        &mut state,
        &mut tilemap,
        samus(0x0280, 0x0180),
        ROOM,
        0x07,
        &maps(MapBits::default()),
    );
    assert_eq!(tilemap.get(SAMUS_CELL), Some(entry(66, 7)));
}

#[test]
fn revealed_cells_show_only_once_the_map_is_collected() {
    // The station reveals map cells (0, 1) to (7, 1): byte 1 * 4 + 4.
    let mut station = MapBits::default();
    station.mark(MapByte(8), 0xFF);
    let maps = maps(station);
    let mut state = mini_map();
    let mut tilemap = HudTilemap::default();
    update(
        &mut state,
        &mut tilemap,
        samus(0x0280, 0x0280),
        ROOM,
        STEADY,
        &maps,
    );
    // Samus is a row below: the window's top row is the revealed one.
    assert_eq!(tilemap.get(ORIGIN), Some(entry(0x1F, 3)));
    state.map_collected = true;
    update(
        &mut state,
        &mut tilemap,
        samus(0x0280, 0x0280),
        ROOM,
        STEADY,
        &maps,
    );
    // The window's top-left is map cell (0, 1): tilemap row 2, index 64.
    assert_eq!(tilemap.get(ORIGIN), Some(entry(64, 3)));
    assert_eq!(tilemap.get(ORIGIN.offset(0, 4)), Some(entry(68, 3)));
}

#[test]
fn the_window_reads_across_the_map_pages() {
    // Samus at map cell (31, 1), the left page's last column; cell (32, 1) is the right
    // page's first, explored: byte 2 * 4 + 80h. The room's left column is the map's 30.
    let mut state = mini_map();
    state.room = MapPosition { x: 30, y: 0 };
    state.explored.mark(MapByte(0x88), 0x80);
    let mut tilemap = HudTilemap::default();
    update(
        &mut state,
        &mut tilemap,
        samus(0x0180, 0x0180),
        ROOM,
        STEADY,
        &maps(MapBits::default()),
    );
    // The column right of hers is the right page's row 2, column 0: index 400h + 40h.
    assert_eq!(tilemap.get(SAMUS_CELL.offset(0, 1)), Some(entry(0x0040, 2)));
    assert_eq!(tilemap.get(SAMUS_CELL), Some(entry(66 + 29, 2)));
}

#[test]
fn nothing_updates_outside_the_room_or_while_disabled() {
    let maps = maps(MapBits::default());
    let mut tilemap = HudTilemap::default();
    let mut state = mini_map();
    update(
        &mut state,
        &mut tilemap,
        samus(0x0800, 0x0180),
        ROOM,
        STEADY,
        &maps,
    );
    state.disabled = true;
    update(
        &mut state,
        &mut tilemap,
        samus(0x0280, 0x0180),
        ROOM,
        STEADY,
        &maps,
    );
    assert_eq!(state.explored, MapBits::default());
    assert_eq!(tilemap, HudTilemap::default());
}

#[test]
fn the_diagonal_tile_explores_the_cell_above() {
    let mut tiles = maps(MapBits::default());
    // Samus's cell, tilemap index 66, holds the diagonal tile.
    if let Some(map) = tiles.0.first_mut() {
        map.tiles[66] = 0x0028;
    }
    let mut state = mini_map();
    let mut tilemap = HudTilemap::default();
    update(
        &mut state,
        &mut tilemap,
        samus(0x0280, 0x0180),
        ROOM,
        STEADY,
        &tiles,
    );
    assert_eq!(state.explored.byte(MapByte(4)), Some(0x20));
}
