use data::hud::ItemIcons;
use data::level::Block;
use data::map::{BEYOND_WORDS, MAP_TILEMAP_WORDS};
use data::room::MapPosition;
use types::Picture as Number;

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

/// The look of invented picture `number`, unflipped.
fn look(number: u16) -> Look {
    Look::unflipped(Number::new(number).unwrap())
}

/// Invented HUD pictures: the blank 1E0h, digit d 1D0h + d, the rest from 1F0h on.
fn invented() -> data::hud::HudPictures {
    let icon = |first: u16| [[look(first); 2]; 2];
    data::hud::HudPictures {
        blank: Number::new(0x1E0).unwrap(),
        digits: core::array::from_fn(|d| look(0x1D0_u16.wrapping_add(u16::try_from(d).unwrap()))),
        full_tank: look(0x1F0),
        empty_tank: look(0x1F1),
        reserve_sign: [[look(0x1F2); 2]; 3],
        icons: ItemIcons {
            missiles: [[look(0x1F3); 3]; 2],
            super_missiles: icon(0x1F4),
            power_bombs: icon(0x1F5),
            grapple_beam: icon(0x1F6),
            x_ray_scope: icon(0x1F7),
        },
        map_blank: look(0x1F8),
        diagonal: Number::new(0x1F9).unwrap(),
    }
}

/// A HUD word of picture `number` in colours 3, above the background.
const fn word_of(number: u16) -> u16 {
    0x2C00 | number
}

fn at(column: i32, row: i32) -> Place {
    Pair { x: column, y: row }
}

#[test]
fn the_maps_import_pictures_charts_and_beyond() {
    let content = HudContent::import(
        &data::map::AreaMaps(vec![rom_map(Area::Brinstar)]),
        &invented(),
    );
    let (map, pictures) = content.of(Area::Brinstar);
    let flipped = |number| Look {
        picture: Number::new(number).unwrap(),
        flip: Flip::Both,
    };
    // Column 40 of row 3 is the right page's column 8: word 400h + 3 × 20h + 8.
    assert_eq!(map.pictures.get(at(40, 3)), Some(flipped(0x068)));
    assert_eq!(map.pictures.get(at(5, 2)), Some(flipped(0x045)));
    assert!(map.charted.contains(at(40, 3)));
    assert!(!map.charted.contains(at(40, 2)));
    // Column 65 is the beyond pictures' column 1.
    assert_eq!(map.pictures.get(at(65, 2)), Some(flipped(0x341)));
    let (debug, _) = content.of(Area::Debug);
    assert_eq!(debug.pictures.get(at(5, 2)), Some(look(0)));
    assert_eq!(debug.charted, MapCellSet::default());
    assert_eq!(pictures.digits[9], look(0x1D9));
}

#[test]
fn the_explored_cells_round_trip() {
    let mut cells = MapCells::default();
    for (row, word) in cells.0.iter_mut().enumerate() {
        *word = 0x9E37_79B9_7F4A_7C15_u64.rotate_left(u32::try_from(row).unwrap());
    }
    let set = explored(&cells);
    assert!(set.contains(at(0, 0)));
    assert_eq!(engine_cells(&set), cells);
}

#[test]
fn every_tile_round_trips() {
    for word in (0..=u16::MAX).filter(|word| word & 0x2000 != 0) {
        let cell = TilemapEntry::from_word(word);
        assert_eq!(entry(tile(cell).unwrap()), cell);
    }
}

#[test]
fn an_auto_cancel_item_is_the_selected_one() {
    assert_eq!(
        selection(Some(HudItem::GrappleBeam), Some(HudItem::GrappleBeam)),
        Ok(Selection::OneShot(SelectableItem::GrappleBeam))
    );
    assert_eq!(
        selection(Some(HudItem::GrappleBeam), Some(HudItem::Missiles)),
        Err(HudFault::OneShotNotSelected)
    );
    assert_eq!(
        selection(None, Some(HudItem::Missiles)),
        Err(HudFault::OneShotNotSelected)
    );
}

/// Samus in the middle of a room of one screen at map cell (13, 4), with 99 of 99 energy, the
/// HUD's tilemap blank and its last energy 0.
fn state() -> State {
    let mut state = State::default();
    state.room.width = BlockColumns(16);
    state.room.blocks = vec![Block::default(); 256];
    state.samus.motion.position = Position {
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
    state.hud.tilemap.cells = [TilemapEntry::from_word(word_of(0x1E0)); 96];
    state
}

fn content() -> Content {
    Content {
        hud: HudContent::import(
            &data::map::AreaMaps(vec![rom_map(Area::Crateria)]),
            &invented(),
        ),
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
    assert_eq!(word(&state, 2, 6), word_of(0x1E0));
    assert_eq!(state.hud.drawn.energy, 0);
    assert_eq!(state.video.batch.vram.len(), 0);
    assert!(!state.hud.mini_map.disabled);
}

#[test]
fn a_mini_map_a_boss_turned_off_stays_off_and_still() {
    let mut state = state();
    state.hud.mini_map.disabled = true;
    mini_map_turn(&mut state, &content()).unwrap();
    assert_eq!(state.hud.mini_map.explored, MapCells::default());
    assert_eq!(word(&state, 1, 28), word_of(0x1E0));
    assert!(state.hud.mini_map.disabled);
}

#[test]
fn the_hud_turn_draws_the_status_bar_and_queues_the_upload() {
    let mut state = state();
    hud_turn(&mut state, &content()).unwrap();
    // 99: digit 9 twice, the invented 1D9h.
    assert_eq!(
        (word(&state, 2, 6), word(&state, 2, 7)),
        (word_of(0x1D9), word_of(0x1D9))
    );
    assert_eq!(state.hud.drawn.energy, 99);
    // The mini-map has had its turn: the window and the explored cells stay as they were.
    assert_eq!(word(&state, 1, 28), word_of(0x1E0));
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
