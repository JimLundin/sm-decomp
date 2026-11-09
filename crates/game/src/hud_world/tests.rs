use data::level::Block;
use data::map::{BEYOND_WORDS, MAP_TILEMAP_WORDS};
use data::room::MapPosition;

use super::*;
use crate::state::room::BlockColumns;
use crate::state::samus::Fixed;

/// A map of `area` whose every tilemap word is its own index, flipped both ways, whose beyond
/// words are 300h past theirs, and whose station reveals the cell in column 40 of row 3.
fn rom_map(area: Area) -> data::map::AreaMap {
    let flipped = |index: usize| u16::try_from(index).unwrap() | 0xC000;
    let mut station = MapCells::default();
    station.0[3] = 1 << 40;
    data::map::AreaMap {
        area,
        tiles: (0..MAP_TILEMAP_WORDS).map(flipped).collect(),
        beyond: (0..BEYOND_WORDS)
            .map(|index| flipped(index.wrapping_add(0x300)))
            .collect(),
        station,
    }
}

fn cell(column: i32, row: i32) -> MapCell {
    MapCell::new(column, row).unwrap()
}

fn picture_at(column: i32, row: i32) -> PicturePos {
    PicturePos::new(column, row).unwrap()
}

#[test]
fn the_maps_import_pictures_charts_and_beyond() {
    let maps = AreaMaps::import(&data::map::AreaMaps(vec![rom_map(Area::Brinstar)]));
    let map = maps.of(Area::Brinstar);
    // Column 40 of row 3 is the right page's column 8: word 400h + 3 × 20h + 8.
    assert_eq!(
        map.pictures.get(picture_at(40, 3)),
        MapPicture {
            picture: Picture::new(0x068),
            flip: Flip::Both,
        }
    );
    assert_eq!(
        map.pictures.get(picture_at(5, 2)).picture,
        Picture::new(0x045)
    );
    assert!(map.charted.get(cell(40, 3)));
    assert!(!map.charted.get(cell(40, 2)));
    // Column 65 is the beyond pictures' column 1.
    assert_eq!(
        map.pictures.get(picture_at(65, 2)).picture,
        Picture::new(0x341)
    );
    assert_eq!(maps.of(Area::Debug), &HerMap::default());
}

#[test]
fn the_explored_cells_round_trip() {
    let mut cells = MapCells::default();
    for (row, word) in cells.0.iter_mut().enumerate() {
        *word = 0x9E37_79B9_7F4A_7C15_u64.rotate_left(u32::try_from(row).unwrap());
    }
    let set = explored(&cells);
    assert!(set.get(cell(0, 0)));
    assert_eq!(engine_cells(&set), cells);
}

#[test]
fn every_tile_round_trips() {
    for word in (0..=u16::MAX).filter(|word| word & 0x2000 != 0) {
        let cell = TilemapEntry::from_word(word);
        assert_eq!(entry(tile(cell).unwrap()), cell);
    }
}

/// Samus in the middle of a room of one screen at map cell (13, 4), with 99 of 99 energy, the
/// HUD's tilemap blank and its last energy 0.
fn state() -> State {
    let mut state = State::default();
    state.room.width = BlockColumns(16);
    state.room.blocks = vec![Block::default(); 256];
    state.samus.motion.position = Point {
        x: Fixed {
            pixels: 0x80,
            subpixels: 0,
        },
        y: Fixed {
            pixels: 0x80,
            subpixels: 0,
        },
    }
    .into();
    state.samus.equipment.energy = Gauge {
        current: 99,
        max: 99,
    };
    state.hud.mini_map.room = MapPosition { x: 13, y: 4 };
    state.hud.tilemap.cells = [TilemapEntry::from_word(0x2C0F); 96];
    state
}

fn content() -> Content {
    Content {
        area_maps: AreaMaps::import(&data::map::AreaMaps(vec![rom_map(Area::Crateria)])),
        ..Content::default()
    }
}

fn word(state: &State, row: usize, column: usize) -> u16 {
    state.hud.tilemap.cells[row.wrapping_mul(32).wrapping_add(column)].word()
}

#[test]
fn the_mini_map_turn_explores_her_cell_and_draws_only_the_window() {
    let mut state = state();
    mini_map_turn(&mut state, &content()).unwrap();
    // Her cell is the room's (13, 5), one row down for the map's padding row.
    assert_eq!(state.hud.mini_map.explored.0[5], 1 << 13);
    // Her window cell shows the map's word for (13, 5), explored (colours 2), blinking at frame
    // 0 (colours 7).
    assert_eq!(word(&state, 1, 28), 0xFC00 | 0xAD);
    // The status bar is the HUD phase's: the energy isn't drawn yet.
    assert_eq!(word(&state, 2, 6), 0x2C0F);
    assert_eq!(state.hud.drawn.energy, 0);
    assert_eq!(state.video.batch.vram.len(), 0);
}

#[test]
fn the_hud_turn_draws_the_status_bar_and_queues_the_upload() {
    let mut state = state();
    hud_turn(&mut state, &content()).unwrap();
    assert_eq!((word(&state, 2, 6), word(&state, 2, 7)), (0x2C08, 0x2C08));
    assert_eq!(state.hud.drawn.energy, 99);
    // The mini-map has had its turn: the window and the explored cells stay as they were.
    assert_eq!(word(&state, 1, 28), 0x2C0F);
    assert_eq!(state.hud.mini_map.explored, MapCells::default());
    assert_eq!(state.video.batch.vram.len(), 1);
}

#[test]
fn a_count_past_its_cells_is_a_fault() {
    let mut state = state();
    state.samus.equipment.super_missiles = Gauge {
        current: 100,
        max: 100,
    };
    assert_eq!(
        hud_turn(&mut state, &content()),
        Err(HudFault::CountPastCells)
    );
}
