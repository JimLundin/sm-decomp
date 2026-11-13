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

#[test]
fn tilesets_come_back_from_the_content_codec_as_they_went_in() {
    let mut tile = Tile::default();
    tile.0[2][5] = 15;
    let look = BlockLook::from_cells(
        [Cell {
            tile: TileRef(0x3ff),
            palette: PaletteRow(7),
            priority: true,
            flip_x: false,
            flip_y: true,
        }; 4],
    );
    let mut palette = Palette::default();
    palette.0[ROW_COLOURS - 1] = Colour(0x801f);
    let tilesets = Tilesets {
        shared: SharedElements { looks: vec![look] },
        sets: vec![
            Tileset::default(),
            Tileset {
                id: CERES,
                sheet: TileSheet(vec![tile, Tile::default()]),
                looks: vec![BlockLook::default(), look],
                palettes: PaletteSet(vec![palette]),
                uses_shared: true,
            },
        ],
    };
    let encoding = canonical::to_bytes(&tilesets);
    assert_eq!(canonical::from_bytes(&encoding), Ok(tilesets));
}

#[test]
fn the_cached_tilesets_are_the_imported_ones_and_the_second_load_reads_them() {
    use crate::cache::Source;
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let Ok(image) = std::fs::read(path) else {
        return;
    };
    let world = crate::world::import(Lorom::new(&image), &crate::world::LOADED).unwrap();
    let imported = import(Lorom::new(&image), &world).unwrap();
    // Outside the checkout: the files hold the ROM's content.
    let dir = std::env::temp_dir().join(format!("sm-tileset-cache-{}", std::process::id()));
    let cache = Cache::new(&dir);
    let first = cached(&cache, &image, &world).unwrap();
    let second = cached(&cache, &image, &world).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    // Compared as bools, so a failure shows no imported content.
    let (built_alike, read_alike) = (first.content == imported, second.content == imported);
    assert!(built_alike);
    assert!(read_alike);
    assert!(matches!(first.source, Source::Built { .. }));
    assert_eq!(second.source, Source::Cache);
    assert_eq!(first.fingerprint, second.fingerprint);
}
