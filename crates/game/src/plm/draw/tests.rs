use data::plm::ItemPlm;

use super::*;
use crate::state::room::{BlockColumns, ITEM_GFX_SLOTS, ItemGfx, ItemGfxSlot, TileNumber};
use crate::state::video::Scroll;

/// A wide room whose item GFX slot 0 holds the Morph Ball, so block `8Eh` draws tiles `3E0h`
/// to `3E3h`.
fn room() -> RoomScoped {
    let mut loaded = [None; ITEM_GFX_SLOTS];
    loaded[0] = Some(ItemPlm::MorphBall);
    RoomScoped {
        width: BlockColumns(64),
        item_gfx: ItemGfx {
            next: ItemGfxSlot(1),
            loaded,
        },
        ..RoomScoped::default()
    }
}

const fn tile(number: u16) -> Tile {
    Tile {
        number: TileNumber(number),
        x_flip: false,
        y_flip: false,
    }
}

/// The item block, which slot 0 rewrote.
const ITEM: Tile = tile(0x8e);

/// The uploads a draw of `tiles` starting at (`x`, `y`) queues, as (destination, source
/// offset, words).
fn queued(room: &RoomScoped, x: u16, y: u16, tiles: &[Tile]) -> Vec<(u16, u16, Vec<u16>)> {
    let mut video = Video::default();
    let tables = TileTables::default();
    let block = BlockIndex(y.wrapping_mul(room.width.0).wrapping_add(x));
    Drawing::new(&mut video, &tables).row(room, block, tiles);
    video
        .batch
        .vram
        .into_iter()
        .filter_map(|transfer| match transfer.source {
            VramSource::Words { from, words } => {
                assert_eq!(from.bank, 0x7E);
                Some((transfer.to.0, from.offset, words))
            }
            VramSource::Content(_) => None,
        })
        .collect()
}

#[test]
fn an_on_screen_block_queues_its_top_then_bottom_halves() {
    // Row 2 is 2 * 40h words down, column 4 is 8 words across.
    assert_eq!(
        queued(&room(), 4, 2, &[ITEM]),
        [
            (0x5088, 0xC6C8, vec![0x3e0, 0x3e1]),
            (0x50A8, 0xC6CC, vec![0x3e2, 0x3e3]),
        ]
    );
}

#[test]
fn a_block_below_the_screens_last_row_or_above_its_first_is_not_drawn() {
    let mut room = room();
    assert_eq!(queued(&room, 4, 15, &[ITEM]).len(), 2);
    assert_eq!(queued(&room, 4, 16, &[ITEM]), []);
    room.camera.y.pixels = 0x40;
    assert_eq!(queued(&room, 4, 3, &[ITEM]), []);
    assert_eq!(queued(&room, 4, 4, &[ITEM]).len(), 2);
}

#[test]
fn columns_from_the_16th_land_in_the_second_tilemap_screen() {
    let mut room = room();
    // The screen starts at block 10h, so blocks up to 20h show.
    room.camera.x.pixels = 0x100;
    // (12h * 2 - 20h) words into the second screen, row 1 down.
    assert_eq!(queued(&room, 0x12, 1, &[ITEM])[0].0, 0x5400 + 4 + 0x40);
    // The 32nd column is the second screen's last and the 33rd wraps to the first.
    assert_eq!(queued(&room, 0x1f, 0, &[ITEM])[0].0, 0x541E);
    assert_eq!(queued(&room, 0x20, 0, &[ITEM])[0].0, 0x5000);
}

#[test]
fn bg1_scrolled_a_screen_across_swaps_the_tilemap_screens() {
    let mut room = room();
    room.camera.bg1_offset = Scroll { x: 0x100, y: 0 };
    assert_eq!(queued(&room, 4, 2, &[ITEM])[0].0, 0x5488);
    room.camera.x.pixels = 0x100;
    assert_eq!(queued(&room, 0x12, 2, &[ITEM])[0].0, 0x5084);
}

#[test]
fn a_block_outside_the_columns_around_the_screen_is_not_drawn() {
    let mut room = room();
    // The block before the screen is (100h + Fh) / 10h - 1 = Fh, and 11h on from it is 20h.
    room.camera.x.pixels = 0x100;
    assert_eq!(queued(&room, 2, 0, &[ITEM]), []);
    assert_eq!(queued(&room, 0xf, 0, &[ITEM]).len(), 2);
    assert_eq!(queued(&room, 0x20, 0, &[ITEM]).len(), 2);
    assert_eq!(queued(&room, 0x21, 0, &[ITEM]), []);
}

#[test]
fn a_row_clipped_at_the_left_draws_from_the_block_before_the_screen() {
    let mut room = room();
    room.camera.x.pixels = 0x100;
    // Blocks Dh, Eh and Fh, of which only Fh shows: the third tile.
    let uploads = queued(&room, 0xd, 0, &[tile(0x90), tile(0x90), ITEM]);
    assert_eq!(
        uploads,
        [
            (0x501E, 0xC6C8, vec![0x3e0, 0x3e1]),
            (0x503E, 0xC6CC, vec![0x3e2, 0x3e3]),
        ]
    );
}

#[test]
fn a_row_that_runs_off_its_tilemap_screen_continues_on_the_other() {
    // Column Fh is the first screen's last: two words fit, two continue at the other screen's
    // start, for the top halves and then the bottom.
    assert_eq!(
        queued(&room(), 0xf, 0, &[ITEM; 2]),
        [
            (0x501E, 0xC6C8, vec![0x3e0, 0x3e1]),
            (0x5400, 0xC6CC, vec![0x3e0, 0x3e1]),
            (0x503E, 0xC6D0, vec![0x3e2, 0x3e3]),
            (0x5420, 0xC6D4, vec![0x3e2, 0x3e3]),
        ]
    );
}

#[test]
fn later_draws_in_a_pass_take_later_buffer_bytes() {
    let room = room();
    let mut video = Video::default();
    let tables = TileTables::default();
    let mut drawing = Drawing::new(&mut video, &tables);
    drawing.block(&room, BlockIndex(4), ITEM);
    drawing.block(&room, BlockIndex(5), ITEM);
    let sources: Vec<u16> = video
        .batch
        .vram
        .iter()
        .filter_map(|transfer| match &transfer.source {
            VramSource::Words { from, .. } => Some(from.offset),
            VramSource::Content(_) => None,
        })
        .collect();
    assert_eq!(sources, [0xC6C8, 0xC6CC, 0xC6D0, 0xC6D4]);
}

#[test]
fn a_full_write_table_draws_nothing() {
    let room = room();
    let mut video = Video::default();
    let full = usize::from(TABLE_FULL / TABLE_ENTRY_BYTES) + 1;
    video.batch.vram.resize(full, VramTransfer::default());
    let tables = TileTables::default();
    Drawing::new(&mut video, &tables).block(&room, BlockIndex(4), ITEM);
    assert_eq!(video.batch.vram.len(), full);
}
