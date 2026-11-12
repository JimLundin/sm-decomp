use data::assets::art::Cell;
use data::assets::tilesets::{SharedElements, Tileset};
use data::id::TilesetId;
use data::level::{Block, Layer2Block};

use super::*;
use crate::state::room::BlockColumns;

/// Blocks across the synthetic room: two screens.
const WIDTH: u16 = 32;
/// Blocks down.
const HEIGHT: u16 = 32;

/// The tile number the synthetic room's block `index` draws.
fn number(index: u16) -> u16 {
    index % 0xFB
}

/// The look of the four tilemap words `words`, in reading order.
fn look(words: [u16; 4]) -> art::BlockLook {
    art::BlockLook::from_cells(words.map(|word| Cell::from(TilemapEntry::from_word(word))))
}

/// The look block `number` has: four tile words in a row from four times it.
fn tiles(number: u16) -> art::BlockLook {
    let first = number.wrapping_mul(4);
    look([0, 1, 2, 3].map(|quarter| first.wrapping_add(quarter)))
}

/// The shared elements and one tileset that uses them, every look [`tiles`].
fn tables() -> Tilesets {
    let looks = || (0..0x100).map(tiles).collect();
    Tilesets {
        shared: SharedElements { looks: looks() },
        sets: vec![Tileset {
            id: TilesetId(0),
            looks: looks(),
            uses_shared: true,
            ..Tileset::default()
        }],
    }
}

/// An air block drawing `word`'s tile, with BTS `bts`.
fn air(word: u16, bts: u8) -> Block {
    Block {
        tile: Tile::from_word(word),
        ..Block::new(0, bts)
    }
}

/// A room of [`WIDTH`] by [`HEIGHT`] blocks, block `i` drawing tile `number(i)`, its BTS the
/// index's low byte.
fn room() -> RoomScoped {
    let blocks = (0..WIDTH * HEIGHT)
        .map(|index| air(number(index), index.to_le_bytes()[0]))
        .collect();
    RoomScoped {
        blocks,
        width: BlockColumns(WIDTH),
        area: data::room::Area::Brinstar,
        tileset: TilesetId(0),
        ..RoomScoped::default()
    }
}

/// The word `word` as an unflipped tilemap entry.
fn entry(word: u16) -> TilemapEntry {
    TilemapEntry::from_word(word)
}

/// The look of an unflipped block whose tileset look is `tiles`.
fn unflipped(tiles: art::BlockLook) -> BlockLook {
    let [top_left, top_right, bottom_left, bottom_right] = tiles.cells().map(TilemapEntry::from);
    BlockLook {
        top_left,
        top_right,
        bottom_left,
        bottom_right,
    }
}

/// The looks `room` draws for `layer`'s `line` from level block (`x`, `y`).
fn looks(room: &RoomScoped, layer: LevelLayer, line: BlockLine, x: u16, y: u16) -> Vec<BlockLook> {
    let exposure = LayerExposure {
        layer,
        line,
        level: BlockPoint { x, y },
        background: BlockPoint::default(),
    };
    exposed_looks(room, &tables(), &exposure).collect()
}

#[test]
fn a_column_reads_down_the_level_a_room_width_a_block() {
    let column = looks(&room(), LevelLayer::One, BlockLine::Column, 17, 0);
    assert_eq!(column.len(), 16);
    assert_eq!(
        column[..2],
        [tiles(number(17)), tiles(number(WIDTH + 17))].map(unflipped)
    );
}

#[test]
fn a_row_reads_along_the_level() {
    let row = looks(&room(), LevelLayer::One, BlockLine::Row, 5, 16);
    assert_eq!(row.len(), 17);
    assert_eq!(
        row[..2],
        [tiles(number(16 * WIDTH + 5)), tiles(number(16 * WIDTH + 6))].map(unflipped)
    );
}

#[test]
fn a_row_past_the_last_block_reads_the_level_data_after_it() {
    // The row under the room's last reads the BTS that follows layer 1: blocks 0 and 1's,
    // then 2 and 3's.
    let mut room = room();
    room.level_tail = vec![0x05, 0x00, 0x07, 0x00];
    let row = looks(&room, LevelLayer::One, BlockLine::Row, 0, HEIGHT);
    assert_eq!(row[..2], [tiles(5), tiles(7)].map(unflipped));
    // Past the decompressed data the buffer holds its fill, $8000: tile 0.
    assert_eq!(row[2], unflipped(tiles(0)));
}

#[test]
fn flipped_blocks_mirror_their_quarters_and_flip_each_word() {
    let tiles = look([1, 2, 3, 4]);
    let words = |tile: Tile| {
        let drawn = BlockLook::of(tiles, tile);
        [
            drawn.top_left,
            drawn.top_right,
            drawn.bottom_left,
            drawn.bottom_right,
        ]
        .map(TilemapEntry::word)
    };
    let tile = |x_flip, y_flip| Tile {
        x_flip,
        y_flip,
        ..Tile::default()
    };
    assert_eq!(words(tile(false, false)), [1, 2, 3, 4]);
    assert_eq!(words(tile(true, false)), [0x4002, 0x4001, 0x4004, 0x4003]);
    assert_eq!(words(tile(false, true)), [0x8003, 0x8004, 0x8001, 0x8002]);
    assert_eq!(words(tile(true, true)), [0xC004, 0xC003, 0xC002, 0xC001]);
    // A word already flipped flips back.
    let flipped = look([0x4001, 2, 3, 4]);
    assert_eq!(
        BlockLook::of(flipped, tile(true, false)).top_right.word(),
        1
    );
}

#[test]
fn layer_2_reads_its_own_level_data() {
    let mut room = room();
    room.layer2 = Some(
        (0..WIDTH * HEIGHT)
            .map(|index| Layer2Block {
                tile: Tile::from_word(number(index.wrapping_add(1))),
                unread: 0,
            })
            .collect(),
    );
    let column = looks(&room, LevelLayer::Two, BlockLine::Column, 17, 1);
    assert_eq!(column[0], unflipped(tiles(number(WIDTH + 17 + 1))));
}

/// A block an item's GFX load rewrote draws the load's entry, not the tile table's
/// (`$84:8764`); with no item loaded it draws the table's.
#[test]
fn a_column_draws_an_item_gfx_slots_blocks_through_its_loaded_entries() {
    let mut room = room();
    // Block column 17's top block draws the first slot's first block.
    room.blocks[17] = air(0x8e, 0);
    let column = looks(&room, LevelLayer::One, BlockLine::Column, 17, 0);
    assert_eq!(column[0], unflipped(tiles(0x8e)));
    crate::plm::item_gfx::load(&mut room.item_gfx, data::plm::ItemPlm::MorphBall);
    let column = looks(&room, LevelLayer::One, BlockLine::Column, 17, 0);
    assert_eq!(
        column[0],
        BlockLook {
            top_left: entry(0x3e0),
            top_right: entry(0x3e1),
            bottom_left: entry(0x3e2),
            bottom_right: entry(0x3e3),
        }
    );
}
