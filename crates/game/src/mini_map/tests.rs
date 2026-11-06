use data::map::{AreaMap, AreaMaps, MAP_BIT_BYTES, MAP_TILEMAP_WORDS, MapCell, MapCells};
use data::room::{Area, MapPosition};

use super::{Blink, ORIGIN, RoomBlocks, SAMUS_CELL, update};
use crate::state::hud::{Cell, HudTilemap, MiniMap};
use crate::state::samus::{Fixed, Point};
use crate::state::video::{BgPalette, TileNumber, TilemapEntry};

/// Brinstar's map, each tile numbered by its layout index's low ten bits.
fn maps(station: MapCells) -> AreaMaps {
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

const fn cell(x: u8, y: u8) -> MapCell {
    MapCell { x, y }
}

/// The window's HUD cell `columns` right of Samus's, or left for a negative count.
const fn beside_samus(columns: isize) -> Cell {
    ORIGIN.offset(1, 2_usize.wrapping_add_signed(columns))
}

/// The tile number the test maps hold at `cell`.
fn tile_of(cell: MapCell) -> u16 {
    u16::try_from(cell.layout().0).unwrap() & 0x03FF
}

fn entry(tile: u16, palette: u8) -> TilemapEntry {
    TilemapEntry {
        tile: TileNumber(tile),
        palette: BgPalette(palette),
        priority: true,
        flip_x: false,
        flip_y: false,
    }
}

/// One update of `state` for Samus in screen (`x`, `y`) of its room, her cell unlit.
fn draw(state: &mut MiniMap, x: u16, y: u16, maps: &AreaMaps) -> HudTilemap {
    let mut tilemap = HudTilemap::default();
    update(
        state,
        &mut tilemap,
        samus((x << 8) | 0x80, (y << 8) | 0x80),
        ROOM,
        Blink::Unlit,
        maps,
    );
    tilemap
}

#[test]
fn samus_cell_is_explored_and_drawn_from_the_map() {
    let mut state = mini_map();
    let tilemap = draw(&mut state, 2, 1, &maps(MapCells::default()));
    // Map row 2, counting the padding row: tilemap index 66.
    assert!(state.explored.contains(cell(2, 2)));
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
        Blink::Lit,
        &maps(MapCells::default()),
    );
    assert_eq!(tilemap.get(SAMUS_CELL), Some(entry(66, 7)));
}

#[test]
fn the_blink_takes_eight_requested_nmis_each_way() {
    assert_eq!(Blink::at(0), Blink::Lit);
    assert_eq!(Blink::at(7), Blink::Lit);
    assert_eq!(Blink::at(8), Blink::Unlit);
    assert_eq!(Blink::at(15), Blink::Unlit);
    assert_eq!(Blink::at(16), Blink::Lit);
}

#[test]
fn revealed_cells_show_only_once_the_map_is_collected() {
    let mut station = MapCells::default();
    for x in 0..8 {
        station.insert(cell(x, 2));
    }
    let maps = maps(station);
    let mut state = mini_map();
    // Samus is a row below: the window's top row is the revealed one.
    let tilemap = draw(&mut state, 2, 2, &maps);
    assert_eq!(tilemap.get(ORIGIN), Some(entry(0x1F, 3)));
    state.map_collected = true;
    let tilemap = draw(&mut state, 2, 2, &maps);
    assert_eq!(tilemap.get(ORIGIN), Some(entry(64, 3)));
    assert_eq!(tilemap.get(ORIGIN.offset(0, 4)), Some(entry(68, 3)));
}

#[test]
fn the_window_reads_into_the_right_page() {
    // Samus in the left page's last column, the cell right of hers explored.
    let mut state = mini_map();
    state.room = MapPosition { x: 30, y: 0 };
    state.explored.insert(cell(32, 2));
    let tilemap = draw(&mut state, 1, 1, &maps(MapCells::default()));
    assert_eq!(
        tilemap.get(beside_samus(1)),
        Some(entry(tile_of(cell(32, 2)), 2))
    );
    assert_eq!(
        tilemap.get(SAMUS_CELL),
        Some(entry(tile_of(cell(31, 2)), 2))
    );
}

#[test]
fn the_window_reads_into_the_left_page() {
    // Samus in the right page's first column, the cell left of hers explored.
    let mut state = mini_map();
    state.room = MapPosition { x: 32, y: 4 };
    state.explored.insert(cell(31, 5));
    let tilemap = draw(&mut state, 0, 0, &maps(MapCells::default()));
    assert_eq!(
        tilemap.get(beside_samus(-1)),
        Some(entry(tile_of(cell(31, 5)), 2))
    );
    assert_eq!(
        tilemap.get(SAMUS_CELL),
        Some(entry(tile_of(cell(32, 5)), 2))
    );
}

#[test]
fn past_the_left_edge_the_window_reads_a_row_up() {
    // Samus at the map's left edge: her own column comes from the right page's row above,
    // the column left of it from the left page's row above, at its far end.
    let mut state = mini_map();
    state.room = MapPosition { x: 0, y: 4 };
    state.explored.insert(cell(32, 4));
    state.explored.insert(cell(31, 4));
    let tilemap = draw(&mut state, 0, 0, &maps(MapCells::default()));
    assert!(state.explored.contains(cell(0, 5)));
    assert_eq!(
        tilemap.get(SAMUS_CELL),
        Some(entry(tile_of(cell(32, 4)), 2))
    );
    assert_eq!(
        tilemap.get(beside_samus(-1)),
        Some(entry(tile_of(cell(31, 4)), 2))
    );
}

#[test]
fn past_the_right_edge_the_window_reads_a_row_down() {
    // Samus at the map's right edge: her own column's cell comes from the left page's row
    // below, the column right of it from the right page's row below, at its start. Its
    // tiles run on past the map, so that column's is blank.
    let mut state = mini_map();
    state.room = MapPosition { x: 63, y: 4 };
    state.explored.insert(cell(31, 6));
    state.explored.insert(cell(32, 6));
    let tilemap = draw(&mut state, 0, 0, &maps(MapCells::default()));
    assert!(state.explored.contains(cell(63, 5)));
    assert_eq!(
        tilemap.get(SAMUS_CELL),
        Some(entry(tile_of(cell(63, 5)), 2))
    );
    assert_eq!(tilemap.get(beside_samus(1)), Some(entry(0x1F, 2)));
}

#[test]
fn off_the_map_the_window_reads_no_cells_and_blank_tiles() {
    // Samus on the right page's bottom row: the window's bottom row is past the map.
    let mut state = mini_map();
    state.room = MapPosition { x: 36, y: 30 };
    state.map_collected = true;
    state.explored = MapCells::from_bitmap(&[0xFF; MAP_BIT_BYTES]);
    let tilemap = draw(
        &mut state,
        0,
        0,
        &maps(MapCells::from_bitmap(&[0xFF; MAP_BIT_BYTES])),
    );
    assert_eq!(
        tilemap.get(SAMUS_CELL),
        Some(entry(tile_of(cell(36, 31)), 2))
    );
    assert_eq!(tilemap.get(ORIGIN.offset(2, 0)), Some(entry(0x1F, 3)));
}

#[test]
fn nothing_updates_outside_the_room_or_while_disabled() {
    let maps = maps(MapCells::default());
    let mut state = mini_map();
    let outside = draw(&mut state, 8, 1, &maps);
    state.disabled = true;
    let disabled = draw(&mut state, 2, 1, &maps);
    assert_eq!(state.explored, MapCells::default());
    assert_eq!(outside, HudTilemap::default());
    assert_eq!(disabled, HudTilemap::default());
}

#[test]
fn the_diagonal_tile_explores_the_cell_above() {
    for tile in [0x0028, 0x0228] {
        let mut tiles = maps(MapCells::default());
        // Samus's cell, tilemap index 66, holds the diagonal tile.
        if let Some(map) = tiles.0.first_mut() {
            map.tiles[66] = tile;
        }
        let mut state = mini_map();
        draw(&mut state, 2, 1, &tiles);
        assert!(state.explored.contains(cell(2, 1)));
    }
}
