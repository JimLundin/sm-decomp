//! Mode 7 VRAM and palette transfers, packed in the original RAM queue.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

const QUEUE: usize = sm_game::display::transfer::MODE7_QUEUE;

#[forbid(unsafe_code)]
pub(super) fn write_word(tail: u16, field: usize, value: u16, game: &mut Game) {
    let at = QUEUE + usize::from(tail) + field;
    let bytes = value.to_ne_bytes();
    game.ram().write_byte(at, bytes[0]);
    game.ram().write_byte(at + 1, bytes[1]);
}

#[forbid(unsafe_code)]
pub(crate) fn advance_ending_tile_upload(game: &mut Game) {
    use crate::ram_map::word;
    let page = word!(game.CINEMATIC_FUNCTION_TIMER);
    let tail = word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL);
    let high_plane = word!(game.CINEMATIC_FUNCTION_TIMER) as i32 - 8 & 0x8000 != 0;
    write_word(tail, 0, if high_plane { 192 } else { 128 }, game);
    // Both original tables have 16 pages; bounds failure follows the first store.
    assert!(page < 16);
    let source = page * 0x800;
    write_word(tail, 1, source, game);
    write_word(tail, 3, 127, game);
    write_word(tail, 4, 2048, game);
    write_word(tail, 6, source & 0x3fff, game);
    write_word(tail, 8, if high_plane { 128 } else { 0 }, game);
    word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL = tail.wrapping_add(9));
    if word!(game.CINEMATIC_FUNCTION_TIMER += 1, reread) as i32 - 16 & 0x8000 == 0 {
        game.ram().write_byte(0x55, 1);
        word! {game; CINEMATIC_FUNCTION_TIMER = 0; CINEMATIC_FUNCTION = 0xdb9d; }
    }
}

pub(super) use sm_game::display::transfer::enqueue_mode7_rom as enqueue_rom;

pub(super) unsafe fn process(game: &mut Game) {
    if word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL) == 0 {
        return;
    }
    sm_game::display::transfer::drain_mode7(&mut unsafe {
        crate::game::display_transfers::RawQueue::new(game)
    });
    game.ram().fill(QUEUE..QUEUE + 2, 0);
    word!(game.MODE7_VRAM_WRITE_QUEUE_TAIL = 0);
}
