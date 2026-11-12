use super::*;
use data::assets::tilesets::{BRINSTAR, CERES, SHARED_BLOCKS};
use data::rom::Rom;

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    Some(Rom::new(std::fs::read(path).ok()?).unwrap())
}

#[test]
fn an_entry_is_four_little_endian_words_in_reading_order_each_a_cell() {
    let looks = looks(&[1, 0x20, 2, 0x44, 3, 0x88, 4, 0xdc, 9]);
    let cell = |tile, palette, priority, flip_x, flip_y| Cell {
        tile: TileRef(tile),
        palette: PaletteRow(palette),
        priority,
        flip_x,
        flip_y,
    };
    assert_eq!(
        looks,
        [BlockLook::from_cells([
            cell(1, 0, true, false, false),
            cell(2, 1, false, true, false),
            cell(3, 2, false, false, true),
            cell(4, 7, false, true, true),
        ])]
    );
}

#[test]
fn a_tile_takes_each_pixel_from_a_bit_of_each_of_four_planes() {
    let mut bytes = [0; TILE_BYTES];
    // Row 0: pixel 0 has plane 0, pixel 1 plane 1; row 7: pixel 7 has planes 2 and 3.
    bytes[0] = 0x80;
    bytes[1] = 0x40;
    bytes[HIGH_PLANES + 14] = 0x01;
    bytes[HIGH_PLANES + 15] = 0x01;
    let tile = tile_of(&bytes);
    assert_eq!(tile.0[0][..2], [1, 2]);
    assert_eq!(tile.0[7][7], 12);
    let lit = tile.0.iter().flatten().filter(|&&pixel| pixel != 0).count();
    assert_eq!(lit, 3);
}

#[test]
fn tiles_and_palettes_that_end_partway_are_refused() {
    assert_eq!(sheet(&[0; TILE_BYTES]).map(|sheet| sheet.0.len()), Some(1));
    assert_eq!(sheet(&[0; TILE_BYTES + 1]), None);
    let palette = palettes(&[0x1f, 0x80].repeat(ROW_COLOURS)).unwrap();
    assert_eq!(palette.0[0].0[ROW_COLOURS - 1], Colour(0x801f));
    assert_eq!(palettes(&[0; ROW_BYTES - 2]), None);
}

#[test]
fn every_registered_tileset_imports_from_the_rom() {
    let Some(cartridge) = rom() else {
        return;
    };
    let world = crate::world::import(cartridge.lorom(), &crate::world::LOADED).unwrap();
    let tilesets = import(cartridge.lorom(), &world).unwrap();
    assert_eq!(tilesets.shared.looks.len(), usize::from(SHARED_BLOCKS));
    assert_eq!(tilesets.sets.len(), TILESETS.len());
    for (entry, tileset) in TILESETS.iter().zip(&tilesets.sets) {
        assert_eq!(tileset.id, entry.id);
        assert!(!tileset.sheet.0.is_empty(), "{}", entry.name);
        assert!(!tileset.palettes.0.is_empty(), "{}", entry.name);
        assert_eq!(tileset.uses_shared, entry.id == BRINSTAR, "{}", entry.name);
    }
    // Slice 0's Brinstar tileset draws every tile number; Ceres's draws from block 0.
    for number in 0..0x400 {
        assert!(tilesets.block(BRINSTAR, number).is_some());
    }
    // Compared as a bool, so a failure shows no imported content.
    let first = tilesets.get(CERES).unwrap().looks.first().copied();
    let drawn_first = tilesets.block(CERES, 0) == first;
    assert!(drawn_first);
}
