use super::*;
use crate::rom::Rom;

/// The ROM, from `SM_ROM` or the checkout's `rom/sm.smc`; `None` where it isn't present.
fn rom() -> Option<Rom> {
    let path = std::env::var_os("SM_ROM").map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../rom/sm.smc"),
        std::path::PathBuf::from,
    );
    let image = std::fs::read(path).ok()?;
    Rom::new(image).ok()
}

/// A block whose four words name it and the table it came from.
fn tiles(table: u16, block: u16) -> BlockTiles {
    let word = (table << 12) | block;
    BlockTiles {
        top_left: word,
        top_right: word,
        bottom_left: word,
        bottom_right: word,
    }
}

fn tables() -> TileTables {
    TileTables {
        cre: TileTable((0..CRE_BLOCKS).map(|block| tiles(1, block)).collect()),
        tilesets: vec![
            TileTable(Vec::new()),
            TileTable((0..0x300).map(|block| tiles(2, block)).collect()),
        ],
    }
}

#[test]
fn an_entry_is_four_little_endian_words_in_reading_order() {
    let table = TileTable::from_bytes(&[1, 0x20, 2, 0x40, 3, 0x80, 4, 0xc0, 9]);
    assert_eq!(
        table.0,
        [BlockTiles {
            top_left: 0x2001,
            top_right: 0x4002,
            bottom_left: 0x8003,
            bottom_right: 0xc004,
        }]
    );
}

#[test]
fn outside_ceres_the_cre_draws_the_first_100h_blocks_and_the_tileset_the_rest() {
    let tables = tables();
    let tileset = TilesetIndex(1);
    assert_eq!(
        tables.block(Area::Brinstar, tileset, 0xff),
        Some(tiles(1, 0xff))
    );
    assert_eq!(
        tables.block(Area::Brinstar, tileset, 0x100),
        Some(tiles(2, 0))
    );
    assert_eq!(
        tables.block(Area::Brinstar, tileset, 0x3ff),
        Some(tiles(2, 0x2ff))
    );
    assert_eq!(
        tables.block(Area::Ceres, tileset, 0x10),
        Some(tiles(2, 0x10))
    );
    assert_eq!(tables.block(Area::Ceres, tileset, 0x300), None);
    assert_eq!(tables.block(Area::Brinstar, TilesetIndex(0), 0x100), None);
    assert_eq!(tables.block(Area::Brinstar, TilesetIndex(2), 0), None);
}

#[test]
fn every_tile_table_decompresses_from_the_rom() {
    let Some(cartridge) = rom() else {
        return;
    };
    let tables = TileTables::load(cartridge.lorom()).unwrap();
    assert_eq!(tables.cre.0.len(), usize::from(CRE_BLOCKS));
    assert_eq!(tables.tilesets.len(), TILESETS);
    for table in &tables.tilesets {
        assert_ne!(table.0.len(), 0);
        assert!(table.0.len() <= BLOCKS);
    }
    // Slice 0's Brinstar tileset fills every tile number outside Ceres.
    let brinstar = TilesetIndex(6);
    for number in 0..u16::try_from(BLOCKS).unwrap() {
        assert!(tables.block(Area::Brinstar, brinstar, number).is_some());
    }
}
