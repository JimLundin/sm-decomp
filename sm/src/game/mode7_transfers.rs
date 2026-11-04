//! Mode 7 VRAM and palette transfers, packed in the original RAM queue.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game_state::Game;
use crate::ram_map::checked_word as word;

const QUEUE: usize = sm_game::display::transfer::MODE7_QUEUE;

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
