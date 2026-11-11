use super::*;
use crate::state::video::Scroll;

/// An invented look whose four cells' words are `first` to `first + 3`, top left first.
fn look(first: u16) -> BlockLook {
    let cell = |offset: u16| TilemapEntry::from_word(first.wrapping_add(offset));
    BlockLook {
        top_left: cell(0),
        top_right: cell(1),
        bottom_left: cell(2),
        bottom_right: cell(3),
    }
}

/// The item's look, an invented one.
const ITEM: u16 = 0x3e0;

/// The uploads a row of `looks` starting at (`x`, `y`) queues under `camera`, as
/// (destination, source offset, words).
fn queued(camera: &Camera, x: u16, y: u16, looks: &[u16]) -> Vec<(u16, u16, Vec<u16>)> {
    let mut batch = NmiBatch::default();
    let write = TileWrite::Blocks {
        at: BlockPoint { x, y },
        looks: looks.iter().map(|&first| look(first)).collect(),
    };
    TileQueue::new(&mut batch).write(camera, &write);
    uploads(&batch)
}

/// The batch's queued uploads, as (destination, source offset, words).
fn uploads(batch: &NmiBatch) -> Vec<(u16, u16, Vec<u16>)> {
    batch
        .vram
        .iter()
        .filter_map(|transfer| match &transfer.source {
            VramSource::Words { from, words } => {
                assert_eq!(from.bank, 0x7E);
                assert_eq!(transfer.step, VramStep::Row);
                Some((transfer.to.0, from.offset, words.clone()))
            }
            VramSource::Content(_) => None,
        })
        .collect()
}

#[test]
fn an_on_screen_block_queues_its_top_then_bottom_halves() {
    // Row 2 is 2 * 40h words down, column 4 is 8 words across.
    assert_eq!(
        queued(&Camera::default(), 4, 2, &[ITEM]),
        [
            (0x5088, 0xC6C8, vec![0x3e0, 0x3e1]),
            (0x50A8, 0xC6CC, vec![0x3e2, 0x3e3]),
        ]
    );
}

#[test]
fn a_block_below_the_screens_last_row_or_above_its_first_is_not_drawn() {
    let mut camera = Camera::default();
    assert_eq!(queued(&camera, 4, 15, &[ITEM]).len(), 2);
    assert_eq!(queued(&camera, 4, 16, &[ITEM]), []);
    camera.y.pixels = 0x40;
    assert_eq!(queued(&camera, 4, 3, &[ITEM]), []);
    assert_eq!(queued(&camera, 4, 4, &[ITEM]).len(), 2);
}

#[test]
fn columns_from_the_16th_land_in_the_second_tilemap_screen() {
    let mut camera = Camera::default();
    // The screen starts at block 10h, so blocks up to 20h show.
    camera.x.pixels = 0x100;
    // (12h * 2 - 20h) words into the second screen, row 1 down.
    assert_eq!(queued(&camera, 0x12, 1, &[ITEM])[0].0, 0x5400 + 4 + 0x40);
    // The 32nd column is the second screen's last and the 33rd wraps to the first.
    assert_eq!(queued(&camera, 0x1f, 0, &[ITEM])[0].0, 0x541E);
    assert_eq!(queued(&camera, 0x20, 0, &[ITEM])[0].0, 0x5000);
}

#[test]
fn bg1_scrolled_a_screen_across_swaps_the_tilemap_screens() {
    let mut camera = Camera {
        bg1_offset: Scroll { x: 0x100, y: 0 },
        ..Camera::default()
    };
    assert_eq!(queued(&camera, 4, 2, &[ITEM])[0].0, 0x5488);
    camera.x.pixels = 0x100;
    assert_eq!(queued(&camera, 0x12, 2, &[ITEM])[0].0, 0x5084);
}

#[test]
fn a_block_outside_the_columns_around_the_screen_is_not_drawn() {
    let mut camera = Camera::default();
    // The block before the screen is (100h + Fh) / 10h - 1 = Fh, and 11h on from it is 20h.
    camera.x.pixels = 0x100;
    assert_eq!(queued(&camera, 2, 0, &[ITEM]), []);
    assert_eq!(queued(&camera, 0xf, 0, &[ITEM]).len(), 2);
    assert_eq!(queued(&camera, 0x20, 0, &[ITEM]).len(), 2);
    assert_eq!(queued(&camera, 0x21, 0, &[ITEM]), []);
}

#[test]
fn a_row_clipped_at_the_left_draws_from_the_block_before_the_screen() {
    let mut camera = Camera::default();
    camera.x.pixels = 0x100;
    // Blocks Dh, Eh and Fh, of which only Fh shows: the third look.
    assert_eq!(
        queued(&camera, 0xd, 0, &[0x100, 0x200, ITEM]),
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
        queued(&Camera::default(), 0xf, 0, &[ITEM, 0x100]),
        [
            (0x501E, 0xC6C8, vec![0x3e0, 0x3e1]),
            (0x5400, 0xC6CC, vec![0x100, 0x101]),
            (0x503E, 0xC6D0, vec![0x3e2, 0x3e3]),
            (0x5420, 0xC6D4, vec![0x102, 0x103]),
        ]
    );
}

#[test]
fn later_draws_in_a_pass_take_later_buffer_bytes() {
    let mut batch = NmiBatch::default();
    let mut queue = TileQueue::new(&mut batch);
    for x in [4, 5] {
        let write = TileWrite::Blocks {
            at: BlockPoint { x, y: 0 },
            looks: vec![look(ITEM)],
        };
        queue.write(&Camera::default(), &write);
    }
    let sources: Vec<u16> = uploads(&batch).iter().map(|&(_, from, _)| from).collect();
    assert_eq!(sources, [0xC6C8, 0xC6CC, 0xC6D0, 0xC6D4]);
}

#[test]
fn a_full_write_queue_draws_nothing() {
    let mut batch = NmiBatch::default();
    let full = usize::from(QUEUE_FULL / QUEUE_ENTRY_BYTES) + 1;
    batch.vram.resize(full, VramTransfer::default());
    let write = TileWrite::Blocks {
        at: BlockPoint { x: 4, y: 0 },
        looks: vec![look(ITEM)],
    };
    TileQueue::new(&mut batch).write(&Camera::default(), &write);
    assert_eq!(batch.vram.len(), full);
}

#[test]
fn a_row_that_needs_four_uploads_needs_more_room() {
    let mut batch = NmiBatch::default();
    let nearly_full = usize::from(QUEUE_FULL_FOR_SPLIT / QUEUE_ENTRY_BYTES) + 1;
    batch.vram.resize(nearly_full, VramTransfer::default());
    let row = |x| TileWrite::Blocks {
        at: BlockPoint { x, y: 0 },
        looks: vec![look(ITEM); 2],
    };
    // Column Fh's row splits and is dropped; column 4's doesn't, and its two uploads fit.
    TileQueue::new(&mut batch).write(&Camera::default(), &row(0xf));
    assert_eq!(batch.vram.len(), nearly_full);
    TileQueue::new(&mut batch).write(&Camera::default(), &row(4));
    assert_eq!(batch.vram.len(), nearly_full + 2);
}

#[test]
fn the_overlays_rows_go_up_in_one_upload_from_the_huds_buffer() {
    let mut batch = NmiBatch::default();
    let cells = (0..96).map(TilemapEntry::from_word).collect();
    let write = TileWrite::Overlay { row: 1, cells };
    TileQueue::new(&mut batch).write(&Camera::default(), &write);
    let words: Vec<u16> = (0..96).collect();
    assert_eq!(uploads(&batch), [(0x5820, 0xC608, words)]);
}
