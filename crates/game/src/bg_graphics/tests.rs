use data::level::{Block, Layer2Block};
use data::tileset::{TileTable, TilesetId};

use super::*;
use crate::state::room::{BlockColumns, Camera, DrawnBlocks, Layer2Scroll, Layer2Scrolls};
use crate::state::samus::Fixed;
use crate::state::video::UploadRequests;

/// Blocks across the synthetic room: two screens.
const WIDTH: u16 = 32;
/// Blocks down.
const HEIGHT: u16 = 32;

/// The tile number the synthetic room's block `index` draws.
fn number(index: u16) -> u16 {
    index % 0xFB
}

/// The tile words entry `number` holds: four in a row from four times it.
fn tiles(number: u16) -> BlockTiles {
    let first = number.wrapping_mul(4);
    BlockTiles {
        top_left: first,
        top_right: first.wrapping_add(1),
        bottom_left: first.wrapping_add(2),
        bottom_right: first.wrapping_add(3),
    }
}

/// The CRE's table and one tileset's, every entry [`tiles`].
fn tables() -> TileTables {
    let table = || TileTable((0..0x100).map(tiles).collect());
    TileTables {
        cre: table(),
        tilesets: vec![table()],
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

/// The camera at `x`, `y`, last built from the blocks at `drawn`.
fn camera(x: u16, y: u16, drawn: BlockPoint) -> Camera {
    Camera {
        x: Fixed {
            pixels: x,
            subpixels: 0,
        },
        y: Fixed {
            pixels: y,
            subpixels: 0,
        },
        drawn: DrawnBlocks {
            layer1: drawn,
            layer2: drawn,
        },
        layer2_scroll: Layer2Scrolls {
            x: Layer2Scroll::Fixed,
            y: Layer2Scroll::Fixed,
        },
        ..Camera::default()
    }
}

/// BG1 scrolled to `x`, `y`.
fn mirrors(x: u16, y: u16) -> Registers {
    let mut mirrors = Registers::default();
    mirrors.scroll[0] = Scroll { x, y };
    mirrors
}

/// The word `word` as an unflipped tilemap entry.
fn entry(word: u16) -> TilemapEntry {
    TilemapEntry::from_word(word)
}

/// The update from `room` at `camera`, with BG1 at the camera.
fn updated(mut room: RoomScoped, camera: Camera) -> (RoomScoped, LevelUploads) {
    let mirrors = mirrors(camera.x.pixels, camera.y.pixels);
    room.camera = camera;
    let mut uploads = LevelUploads::default();
    update(&mut room, &mirrors, &tables(), &mut uploads);
    (room, uploads)
}

const fn point(x: u16, y: u16) -> BlockPoint {
    BlockPoint { x, y }
}

#[test]
fn crossing_a_block_column_rightward_builds_the_column_coming_into_view() {
    let (room, uploads) = updated(room(), camera(0x10, 0, point(0, 0)));
    let column = uploads.bg1.column;
    // Block column 1 + 16 is on BG1's second screen, its first column.
    assert_eq!(
        column.first,
        TilemapPart {
            to: VramWord(0x5402),
            bytes: 0x40
        }
    );
    assert_eq!(
        column.wrapped,
        TilemapPart {
            to: VramWord(0x5402),
            bytes: 0
        }
    );
    let top = tiles(number(17));
    let below = tiles(number(WIDTH + 17));
    assert_eq!(
        column.left[..3],
        [
            entry(top.top_left),
            entry(top.bottom_left),
            entry(below.top_left)
        ]
    );
    assert_eq!(
        column.right[..2],
        [entry(top.top_right), entry(top.bottom_right)]
    );
    assert_eq!(column.requests, UploadRequests(1));
    assert_eq!(room.camera.drawn.layer1, point(1, 0));
    assert_eq!(uploads.bg1.row.requests, UploadRequests(0));
    assert_eq!(uploads.bg2, LayerUploads::default());
}

#[test]
fn crossing_a_block_column_leftward_builds_the_left_edge_and_wraps_down_the_tilemap() {
    let (room, uploads) = updated(room(), camera(0x4F, 0x30, point(5, 3)));
    let column = uploads.bg1.column;
    // Three block rows down, the last three blocks wrap to the tilemap's top.
    assert_eq!(
        column.first,
        TilemapPart {
            to: VramWord(0x50C8),
            bytes: 0x34
        }
    );
    assert_eq!(
        column.wrapped,
        TilemapPart {
            to: VramWord(0x5008),
            bytes: 0x0C
        }
    );
    let first = tiles(number(3 * WIDTH + 4));
    assert_eq!(column.left[0], entry(first.top_left));
    assert_eq!(room.camera.drawn.layer1, point(4, 3));
    assert_eq!(uploads.bg1.row.requests, UploadRequests(0));
}

#[test]
fn moving_down_a_block_builds_the_row_below_the_view_and_wraps_to_the_other_screen() {
    let (room, uploads) = updated(room(), camera(0x50, 0x10, point(5, 0)));
    let row = uploads.bg1.row;
    // Row 1 + 15 is the tilemap's first; five blocks across, 11 fit before the screen's
    // right edge.
    assert_eq!(
        row.first,
        TilemapPart {
            to: VramWord(0x500A),
            bytes: 44
        }
    );
    assert_eq!(
        row.wrapped,
        TilemapPart {
            to: VramWord(0x5400),
            bytes: 24
        }
    );
    let first = tiles(number(16 * WIDTH + 5));
    let second = tiles(number(16 * WIDTH + 6));
    assert_eq!(
        row.top[..3],
        [
            entry(first.top_left),
            entry(first.top_right),
            entry(second.top_left)
        ]
    );
    assert_eq!(
        row.bottom[..2],
        [entry(first.bottom_left), entry(first.bottom_right)]
    );
    assert_eq!(row.requests, UploadRequests(1));
    assert_eq!(uploads.bg1.column.requests, UploadRequests(0));
    assert_eq!(room.camera.drawn.layer1, point(5, 1));
}

#[test]
fn moving_up_a_block_builds_the_row_below_the_top() {
    let (_, uploads) = updated(room(), camera(0, 0x10, point(0, 2)));
    let first = tiles(number(2 * WIDTH));
    assert_eq!(uploads.bg1.row.top[0], entry(first.top_left));
    assert_eq!(uploads.bg1.row.first.to, VramWord(0x5000 + 2 * 0x40));
}

#[test]
fn a_row_past_the_last_block_reads_the_level_data_after_it() {
    // The row under the room's last reads the BTS that follows layer 1: blocks 0 and 1's,
    // then 2 and 3's.
    let mut room = room();
    room.level_tail = vec![0x05, 0x00, 0x07, 0x00];
    let (_, uploads) = updated(room, camera(0, (HEIGHT - 15) * 16, point(0, HEIGHT - 16)));
    let [first, second] = [5, 7].map(tiles);
    assert_eq!(
        uploads.bg1.row.top[..3],
        [
            entry(first.top_left),
            entry(first.top_right),
            entry(second.top_left)
        ]
    );
    // Past the decompressed data the buffer holds its fill, $8000: tile 0.
    assert_eq!(uploads.bg1.row.top[4], entry(tiles(0).top_left));
}

#[test]
fn flipped_blocks_mirror_their_quarters_and_flip_each_word() {
    let tiles = BlockTiles {
        top_left: 1,
        top_right: 2,
        bottom_left: 3,
        bottom_right: 4,
    };
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
    let flipped = BlockTiles {
        top_left: 0x4001,
        ..tiles
    };
    assert_eq!(
        BlockLook::of(flipped, tile(true, false)).top_right.word(),
        1
    );
}

#[test]
fn bg2_builds_from_layer_2_into_its_tilemap_only_while_it_shows_the_level() {
    let mut room = room();
    room.layer2 = Some(
        (0..WIDTH * HEIGHT)
            .map(|index| Layer2Block {
                tile: Tile::from_word(number(index.wrapping_add(1))),
                unread: 0,
            })
            .collect(),
    );
    room.tilemap_gap = VramWord(0x0800);
    let mut camera = camera(0x10, 0x10, point(0, 0));
    camera.layer2 = Scroll { x: 0x10, y: 0x10 };
    camera.layer2_scroll = Layer2Scrolls {
        x: Layer2Scroll::WithLayer1,
        y: Layer2Scroll::Scaled {
            rate: 0x80,
            custom: false,
        },
    };
    let mut mirrors = mirrors(0x10, 0x10);
    mirrors.scroll[1] = Scroll { x: 0x10, y: 0x10 };
    room.camera = camera;
    let mut uploads = LevelUploads::default();
    update(&mut room, &mirrors, &tables(), &mut uploads);
    let column = uploads.bg2.column;
    assert_eq!(column.requests, UploadRequests(1));
    assert_eq!(column.first.to, VramWord(0x4C00 + 0x40 + 2));
    assert_eq!(
        column.left[0],
        entry(tiles(number(WIDTH + 17 + 1)).top_left)
    );
    // Its Y scroll byte is the library background's, so no row.
    assert_eq!(uploads.bg2.row.requests, UploadRequests(0));
    assert_eq!(room.camera.drawn.layer2, point(1, 0));
    assert_eq!(uploads.bg1.row.requests, UploadRequests(1));
}

#[test]
fn a_layer_still_in_its_blocks_builds_nothing() {
    let (room, uploads) = updated(room(), camera(0x1F, 0x2F, point(1, 2)));
    assert_eq!(uploads, LevelUploads::default());
    assert_eq!(room.camera.drawn.layer1, point(1, 2));
}

/// A block an item's GFX load rewrote draws the load's entry, not the tile table's
/// (`$84:8764`); with no item loaded it draws the table's.
#[test]
fn a_column_draws_an_item_gfx_slots_blocks_through_its_loaded_entries() {
    let mut room = room();
    // Block column 17's top block draws the first slot's first block.
    room.blocks[17] = air(0x8e, 0);
    let (_, uploads) = updated(room.clone(), camera(0x10, 0, point(0, 0)));
    let static_tiles = tiles(0x8e);
    assert_eq!(uploads.bg1.column.left[0], entry(static_tiles.top_left));
    crate::plm::item_gfx::load(&mut room.item_gfx, data::plm::ItemPlm::MorphBall);
    let (_, uploads) = updated(room, camera(0x10, 0, point(0, 0)));
    let column = uploads.bg1.column;
    assert_eq!(
        [
            column.left[0],
            column.left[1],
            column.right[0],
            column.right[1]
        ],
        [entry(0x3e0), entry(0x3e2), entry(0x3e1), entry(0x3e3)]
    );
}
